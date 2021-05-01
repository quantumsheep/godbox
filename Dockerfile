FROM buildpack-deps:buster as compilers

RUN set -xe && \
    apt-get update && \
    apt-get install -y --no-install-recommends git libcap-dev && \
    rm -rf /var/lib/apt/lists/* && \
    git clone https://github.com/judge0/isolate.git /tmp/isolate && \
    cd /tmp/isolate && \
    git checkout ad39cc4d0fbb577fb545910095c9da5ef8fc9a1a && \
    make -j$(nproc) install && \
    rm -rf /tmp/*

COPY modules modules

RUN for module in modules/*; do bash $module; done

FROM rustlang/rust:nightly as build

WORKDIR /usr/src/app

COPY src src

COPY Cargo.toml .

RUN cargo install --path .

FROM compilers

COPY --from=build /usr/local/cargo/bin/godbox /usr/local/bin/godbox

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080

CMD godbox
