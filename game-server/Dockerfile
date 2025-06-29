FROM rust:latest AS base

RUN rustup component add rustfmt

WORKDIR /root
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v31.1/protoc-31.1-linux-x86_64.zip && \
    unzip protoc-31.1-linux-x86_64.zip -d protoc && \
    cp protoc/bin/protoc /usr/local/bin/

WORKDIR /app
COPY . .

FROM base AS build

RUN cargo build

CMD ["/app/target/debug/spire-game-server"]