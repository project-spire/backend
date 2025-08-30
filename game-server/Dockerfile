FROM rust:bookworm AS builder

WORKDIR /tools
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v31.1/protoc-31.1-linux-x86_64.zip && \
    unzip protoc-31.1-linux-x86_64.zip -d protoc && \
    cp protoc/bin/protoc /usr/local/bin/

WORKDIR /workspace
COPY . .

RUN cargo build --release --bin game-server

FROM debian:bookworm-slim AS runtime

RUN groupadd server && \
    useradd -m -g server server

WORKDIR /app

COPY --from=builder /workspace/target/release/game-server /app/
COPY --from=builder /workspace/target/release/environment.ron /app/

RUN chown -R server:server /app
USER server

CMD ["./game-server"]
