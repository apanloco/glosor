FROM --platform=amd64 rust AS build

RUN apt update && apt upgrade -y
RUN mkdir /tmp/deleteme \
    && cd /tmp/deleteme \
    && cargo init \
    && cargo add eframe \
    && rm -rf /tmp/deleteme
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk
WORKDIR /app
COPY ./ /app/
RUN trunk build --release

FROM nginx:alpine

COPY --from=build /app/dist/* /usr/share/nginx/html/

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]

