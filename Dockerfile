FROM rust:1.57

ENV LOAD_DIR=/opt/iroha2_load_rs

RUN mkdir ${LOAD_DIR}

WORKDIR ${LOAD_DIR}

COPY src src 
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY config.json config.json

CMD ["cargo", "run"]
