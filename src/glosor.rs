use crate::error::Error;

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq, Debug, Clone)]
#[serde(default)]
pub struct Glosor {
    pub language_from: String,
    pub language_to: String,
    pub glosor: Vec<Glosa>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq, Debug, Clone)]
#[serde(default)]
pub struct Glosa {
    pub from: String,
    pub to: String,
}

pub fn csv_to_glosor(input: &[u8]) -> Result<Glosor, Error> {
    let mut glosor = Vec::new();
    let mut reader = csv::Reader::from_reader(input);
    for result in reader.records() {
        match result {
            Ok(record) => {
                glosor.push(Glosa {
                    from: record.get(0).unwrap_or_default().to_owned(),
                    to: record.get(1).unwrap_or_default().to_owned(),
                });
            }
            Err(_) => return Err(Error::CsvParseFailed),
        }
    }
    if glosor.is_empty() {
        return Err(Error::CsvParseFailed);
    }

    Ok(Glosor {
        language_from: reader.headers().unwrap().get(0).unwrap().to_string(),
        language_to: reader.headers().unwrap().get(1).unwrap().to_string(),
        glosor,
    })
}

#[test]
fn test_csv_to_glosor() -> Result<(), Error> {
    let input = r#"engelska,svenska
hello,hej
spank,smisk
swim,simma
"#;
    let glosor = csv_to_glosor(input.as_bytes())?;
    assert_eq!(glosor.language_from, "engelska");
    assert_eq!(glosor.language_to, "svenska");
    assert_eq!(glosor.glosor.len(), 3);

    let entries = csv_to_glosor("invalid".as_bytes());
    assert_eq!(entries, Err(Error::CsvParseFailed));

    Ok(())
}
