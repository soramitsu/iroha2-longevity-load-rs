FROM nwtgck/rust-musl-builder:1.60.0 as builder

COPY src src
COPY Cargo.toml Cargo.toml
COPY config.json config.json

RUN cargo build --release


FROM alpine:3.16

ENV PORT=8084

ENV LOAD_DIR=/opt/iroha2_load_rs

RUN apk --update --no-cache add ca-certificates && \
    adduser --disabled-password --gecos "" iroha --shell /bin/bash --home /app && \
    mkdir -p ${LOAD_DIR}

WORKDIR ${LOAD_DIR}

COPY --from=builder \
     /home/rust/src/target/x86_64-unknown-linux-musl/release/iroha2-longevity-load-rs .

USER iroha