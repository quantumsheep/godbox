FROM rustlang/rust:stable as build

WORKDIR /usr/src/app

COPY src src

COPY Cargo.toml .

RUN cargo install --path .

FROM quantumsheep/godbox-base:latest

COPY --from=build /usr/local/cargo/bin/godbox /usr/local/bin/godbox

CMD godbox
