FROM rustlang/rust:nightly as build

WORKDIR /usr/src/app

COPY src src

COPY Cargo.toml .

RUN cargo install --path .

FROM quantumsheep/godbox-base:latest

COPY --from=build /usr/local/cargo/bin/godbox /usr/local/bin/godbox

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080

CMD godbox
