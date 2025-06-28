FROM rust:latest AS build

WORKDIR /root
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v29.3/protoc-29.3-linux-x86_64.zip && \
    unzip protoc-29.3-linux-x86_64.zip -d protoc && \
    cp protoc/bin/protoc /usr/local/bin/

WORKDIR /app
COPY . .

RUN cargo build

CMD ["/app/target/debug/spire-game-server"]