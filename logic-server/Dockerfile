FROM rust:latest AS base

WORKDIR /root
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v29.3/protoc-29.3-linux-x86_64.zip && \
    unzip protoc-29.3-linux-x86_64.zip -d protoc && \
    cp protoc/bin/protoc /usr/local/bin/

WORKDIR /app
COPY . .



FROM base AS build

RUN cargo build

ENTRYPOINT ["/app/target/debug/spire-game-server"]