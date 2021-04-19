FROM debian:buster-slim as compilers

RUN set -xe && \
    apt-get update && \
    apt-get -y --no-install-recommends install build-essential && \
    rm -rf /var/lib/apt/lists/*

RUN set -xe && \
    apt-get update && \
    apt-get install -y --no-install-recommends git libcap-dev && \
    apt-get install -y --reinstall ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    git clone https://github.com/judge0/isolate.git /tmp/isolate && \
    cd /tmp/isolate && \
    git checkout ad39cc4d0fbb577fb545910095c9da5ef8fc9a1a && \
    make -j$(nproc) install && \
    rm -rf /tmp/*

FROM rustlang/rust:nightly as build

WORKDIR /usr/src/app

COPY src src

COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo install --path .

FROM compilers

COPY --from=build /usr/local/cargo/bin/godbox /usr/local/bin/godbox

CMD godbox
