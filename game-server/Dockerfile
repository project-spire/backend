FROM rust:trixie AS chef

RUN cargo install cargo-chef
WORKDIR app


FROM chef AS planner

COPY Cargo.toml .
COPY data data
COPY db db
COPY protocol protocol
COPY util util

COPY game-server game-server
COPY lobby-server/Cargo.dummy.toml lobby-server/Cargo.toml
COPY lobby-server/src/main.dummy.rs lobby-server/src/main.rs


RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder

WORKDIR /tool
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v33.2/protoc-33.2-linux-x86_64.zip && \
    unzip protoc-33.2-linux-x86_64.zip -d protoc && \
    cp protoc/bin/protoc /usr/local/bin/ && \
    cp -r protoc/include /usr/local

WORKDIR /app

COPY Cargo.toml .
COPY data data
COPY db db
COPY protocol protocol
COPY util util

COPY game-server game-server
COPY lobby-server/Cargo.dummy.toml lobby-server/Cargo.toml
COPY lobby-server/src/main.dummy.rs lobby-server/src/main.rs

RUN cargo build --release --bin game-server


FROM debian:trixie-slim AS runtime

RUN groupadd server && \
    useradd -m -g server server

WORKDIR /app
COPY --from=builder /app/target/release/game-server /app/
COPY --from=builder /app/game-server/config.toml /app/
RUN chown -R server:server /app

USER server
CMD ["./game-server"]
