use crate::error::Error;
use egui::FontFamily::Monospace;
use egui::{FontId, TextStyle};
// TODO: style
// TODO: show popup explaining contents of csv
// TODO: download current glosor as csv
// TODO: test log
// TODO: bigger font
use crate::glosor::{csv_to_glosor, Glosa, Glosor};

pub mod built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
struct Preloaded {
    name: String,
    glosor: Glosor,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct GlosorApp {
    build_info: String,
    state: State,
    current_document: Option<Document>,
    selected_preloaded: String,
    preloaded: Vec<Preloaded>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq, Debug, Clone)]
#[serde(default)]
pub struct Document {
    glosor: Glosor,
    input: Vec<Glosa>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, PartialEq)]
enum State {
    #[default]
    Initial,
    Loaded,
    Testing,
    Results,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Initial => write!(f, "Initial"),
            State::Loaded => write!(f, "Loaded"),
            State::Testing => write!(f, "Testing"),
            State::Results => write!(f, "Results"),
        }
    }
}

fn generate_build_info() -> String {
    format!(
        "{} {}",
        built::PKG_VERSION,
        built::GIT_COMMIT_HASH_SHORT.unwrap_or("?"),
    )
}

fn preload(name: &str, bytes: &[u8]) -> Result<Preloaded, Error> {
    Ok(Preloaded {
        name: name.to_owned(),
        glosor: csv_to_glosor(bytes)?,
    })
}

fn heading2() -> TextStyle {
    TextStyle::Name("Heading2".into())
}

fn heading3() -> TextStyle {
    TextStyle::Name("ContextHeading".into())
}

fn configure_text_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(25.0, Monospace)),
        (heading2(), FontId::new(22.0, Monospace)),
        (heading3(), FontId::new(19.0, Monospace)),
        (TextStyle::Body, FontId::new(16.0, Monospace)),
        (TextStyle::Monospace, FontId::new(12.0, Monospace)),
        (TextStyle::Button, FontId::new(12.0, Monospace)),
        (TextStyle::Small, FontId::new(8.0, Monospace)),
    ]
        .into();
    ctx.set_style(style);
}

impl GlosorApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_text_styles(&cc.egui_ctx);

        let preloaded = vec![
            preload(
                "david-engelska-kap22.csv",
                include_bytes!("../data/david-engelska-kap22.csv"),
            )
                .unwrap(),
            preload("example1", include_bytes!("../data/example.csv")).unwrap(),
        ];

        if let Some(storage) = cc.storage {
            // TODO: duplicate code for preload
            let mut loaded: GlosorApp =
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            let mut selected_preloaded = "".to_string();
            if let Some(preloaded) = preloaded.first() {
                selected_preloaded = preloaded.name.to_owned();
            }

            loaded.preloaded = preloaded;
            loaded.selected_preloaded = selected_preloaded;
            loaded.build_info = generate_build_info();
            return loaded;
        }

        let mut selected_preloaded = "".to_string();
        if let Some(preloaded) = preloaded.first() {
            selected_preloaded = preloaded.name.to_owned();
        }

        GlosorApp {
            build_info: generate_build_info(),
            state: State::Initial,
            current_document: None,
            selected_preloaded,
            preloaded,
        }
    }
}

fn load_glosor(glosor: Glosor) -> Document {
    let input = glosor.glosor.clone();
    Document { glosor, input }
}

impl GlosorApp {
    pub fn shuffle_document(document: &Document) -> Document {
        use rand::seq::SliceRandom;
        let mut doc = document.clone();
        doc.glosor.glosor.shuffle(&mut rand::thread_rng());

        let mut true_and_false = Vec::with_capacity(document.glosor.glosor.len());
        for i in 0..doc.glosor.glosor.len() {
            if i % 2 == 0 {
                true_and_false.push(true);
            } else {
                true_and_false.push(false);
            }
        }
        true_and_false.shuffle(&mut rand::thread_rng());

        doc.input.clear();

        for (index, true_or_false) in true_and_false.iter().enumerate() {
            let glosa = &doc.glosor.glosor[index];
            if *true_or_false {
                doc.input.push(Glosa {
                    from: glosa.from.to_string(),
                    to: "".to_string(),
                });
            } else {
                doc.input.push(Glosa {
                    from: "".to_string(),
                    to: glosa.to.to_string(),
                });
            }
        }
        doc
    }

    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
        use egui::*;

        // Preview hovering files:
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            println!("dropping glosor");

            let text = "Drop Here 📂";

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                println!("dropped files: {:?}", i.raw.dropped_files);
                for f in &i.raw.dropped_files {
                    let glosor = if let Some(bytes) = &f.bytes {
                        csv_to_glosor(bytes).unwrap_or_default()
                    } else if let Some(path) = &f.path {
                        let contents = std::fs::read_to_string(path).unwrap_or_default();
                        csv_to_glosor(contents.as_bytes()).unwrap_or_default()
                    } else {
                        Default::default()
                    };
                    self.current_document = Some(load_glosor(glosor));
                    self.state = State::Loaded;
                }
            }
        });
    }
}

impl eframe::App for GlosorApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            //.min_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading("Glosor");
                    ui.add(egui::Label::new("[?]").sense(egui::Sense::click()))
                        .on_hover_text(
                            format!("Glosor online hjälper dig att plugga glosor.\nFörst laddar du glosorna, sen testar du dig.\nDu kan dra-o-släppa in glosor i CSV-format. Exempelfil (strecken ingår inte):\n---\nengelska,svenska\nhello,hej\nname,namn\n---\n\n{}", &self.build_info));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("")
                    .selected_text(&self.selected_preloaded)
                    .width(300.0)
                    .show_ui(ui, |ui| {
                        for preloaded in &self.preloaded {
                            ui.selectable_value(
                                &mut self.selected_preloaded,
                                preloaded.name.to_string(),
                                &preloaded.name,
                            );
                        }
                    });
                if ui.add(egui::Button::new("Load")).clicked() {
                    println!("selected: {}", self.selected_preloaded);
                    for preloaded in &self.preloaded {
                        if preloaded.name == self.selected_preloaded {
                            self.current_document = Some(load_glosor(preloaded.glosor.clone()));
                            self.state = State::Loaded;
                            break;
                        }
                    }
                }
            });

            if let Some(document) = &mut self.current_document {
                egui::Grid::new("glosor_grid")
                    .min_col_width(150.0)
                    .max_col_width(250.0)
                    .show(ui, |ui| {
                        ui.label(&document.glosor.language_from);
                        ui.label(&document.glosor.language_to);
                        ui.end_row();
                        for i in 0..document.glosor.glosor.len() {
                            let glosa = &mut document.glosor.glosor[i];
                            let input = &mut document.input[i];

                            if self.state == State::Loaded {
                                egui::TextEdit::singleline(&mut glosa.from).show(ui);
                                egui::TextEdit::singleline(&mut glosa.to).show(ui);
                            } else {
                                egui::TextEdit::singleline(&mut input.from)
                                    .interactive(self.state != State::Results)
                                    .show(ui);
                                egui::TextEdit::singleline(&mut input.to)
                                    .interactive(self.state != State::Results)
                                    .show(ui);
                            }

                            if self.state == State::Results {
                                let green = egui::Color32::from_rgb(100, 255, 100);
                                let red = egui::Color32::from_rgb(255, 100, 100);
                                let ok_label;
                                let ok_color;
                                if glosa.to.to_lowercase().trim() == input.to.to_lowercase().trim()
                                    && glosa.from.to_lowercase().trim()
                                    == input.from.to_lowercase().trim()
                                {
                                    ok_label = "✅";
                                    ok_color = green;
                                } else {
                                    ok_label = "❌";
                                    ok_color = red;
                                };

                                ui.label(egui::RichText::new(ok_label).color(ok_color))
                                    .on_hover_text(format!("{} / {}", &glosa.from, &glosa.to));
                            }

                            ui.end_row();
                        }
                    });

                if (self.state == State::Loaded || self.state == State::Results)
                    && ui.add(egui::Button::new("Testa Mig! 🐸")).clicked()
                {
                    self.current_document = Some(GlosorApp::shuffle_document(document));
                    self.state = State::Testing;
                }

                if self.state == State::Testing && ui.add(egui::Button::new("Klar! 🐸")).clicked()
                {
                    self.state = State::Results;
                }
            }

            ui.separator();
            egui::warn_if_debug_build(ui);
        });

        self.ui_file_drag_and_drop(ctx);
    }
}
