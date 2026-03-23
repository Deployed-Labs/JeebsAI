FROM rust:1.72 as builder
WORKDIR /usr/src/jeebs

# Pre-copy manifest to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src
RUN echo "fn main() {println!(\"dummy\");}" > src/main.rs

RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev build-essential ca-certificates --no-install-recommends && rm -rf /var/lib/apt/lists/*

# Attempt to fetch and compile dependencies (cached layer)
RUN cargo build --release || true

# Copy full source and build final binary
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl --no-install-recommends && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy binary and runtime assets
COPY --from=builder /usr/src/jeebs/target/release/jeebs /usr/local/bin/jeebs
COPY --from=builder /usr/src/jeebs/VERSION /app/VERSION

ENV RUST_LOG=info
EXPOSE 8080

VOLUME ["/data"]

COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["jeebs", "--port", "8080"]
