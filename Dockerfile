# Build stage
FROM rust:1.82-bookworm AS builder

RUN apt-get update && apt-get install -y \
    build-essential clang pkg-config libssl-dev sqlite3 \
    nettle-dev libgpg-error-dev libgcrypt-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libsqlite3-0 \
    libnettle8 libgpg-error0 libgcrypt20 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r jeebs \
    && useradd -r -g jeebs -s /sbin/nologin -d /var/lib/jeebs jeebs

COPY --from=builder /build/target/release/jeebs /usr/local/bin/jeebs
COPY --from=builder /build/webui /var/lib/jeebs/webui

RUN mkdir -p /var/lib/jeebs/plugins \
    && chown -R jeebs:jeebs /var/lib/jeebs

WORKDIR /var/lib/jeebs
USER jeebs

ENV PORT=8080
ENV DATABASE_URL=sqlite:/var/lib/jeebs/jeebs.db
ENV RUST_LOG=info

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/jeebs"]
