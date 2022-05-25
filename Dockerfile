FROM nwtgck/rust-musl-builder:1.60.0 as builder

COPY src src
COPY Cargo.toml Cargo.toml
COPY config.json config.json

RUN sudo chown -R rust:rust /home/rust && \
    cargo build --release


FROM alpine:3.16

ENV LOAD_DIR=/opt/iroha2_load_rs

RUN mkdir ${LOAD_DIR} && \
    apk --no-cache add ca-certificates && \
    adduser --disabled-password --gecos "" iroha && \
    chown -R iroha ${LOAD_DIR}
    
USER iroha

WORKDIR ${LOAD_DIR}

COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/iroha2-longevity-load-rs \
    ${LOAD_DIR}