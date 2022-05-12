FROM rust:1.60

ENV LOAD_DIR=/opt/iroha2_load_rs

RUN mkdir ${LOAD_DIR}

WORKDIR ${LOAD_DIR}

COPY src src
COPY Cargo.toml Cargo.toml
COPY config.json config.json

RUN adduser --disabled-password --gecos "" iroha && \
   chown -R iroha ${LOAD_DIR}

USER iroha

CMD ["cargo", "run"]
