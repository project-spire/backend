FROM rust:trixie AS chef

RUN cargo install cargo-chef
WORKDIR app


FROM chef AS planner

COPY Cargo.toml .
COPY data data
COPY protocol protocol
COPY game-server/Cargo.dummy.toml game-server/Cargo.toml
COPY game-server/src/main.dummy.rs game-server/src/main.rs
COPY util util

COPY lobby-server lobby-server

RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder

WORKDIR /tool
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v32.1/protoc-32.1-linux-x86_64.zip && \
    unzip protoc-32.1-linux-x86_64.zip -d protoc && \
    cp protoc/bin/protoc /usr/local/bin/ && \
    cp -r protoc/include /usr/local

WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml .
COPY data data
COPY protocol protocol
COPY game-server/Cargo.dummy.toml game-server/Cargo.toml
COPY game-server/src/main.dummy.rs game-server/src/main.rs
COPY util util
COPY .sqlx .sqlx

COPY lobby-server lobby-server

RUN cargo build --release --bin lobby-server


FROM debian:trixie-slim AS runtime

RUN groupadd server && \
    useradd -m -g server server

WORKDIR /app
COPY --from=builder /app/target/release/lobby-server .
RUN chown -R server:server /app

USER server
CMD ["./lobby-server"]
