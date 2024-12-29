FROM rust:1.83.0 AS builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
  libssl-dev \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/data /app/data
COPY --from=builder /usr/src/app/configuration.yaml /app/configuration.yaml
WORKDIR /app

RUN apt-get update && apt-get install -y tini
ENTRYPOINT ["/usr/bin/tini", "--"]
STOPSIGNAL SIGTERM
