# Stage 1: Build
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install build dependencies (OpenSSL, SQLite, pkg-config)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build for release
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libsqlite3-0 \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary and assets from builder
COPY --from=builder /app/target/release/jeebs /usr/local/bin/jeebs
COPY --from=builder /app/webui ./webui

# Create runtime directories and a non-root user
RUN mkdir -p /data /app/plugins /app/webui \
    && useradd -r -u 10001 -g nogroup jeebs \
    && chown -R jeebs:nogroup /data /app

ENV PORT=8080
ENV DATABASE_URL=sqlite:/data/jeebs.db

EXPOSE 8080

USER jeebs

CMD ["jeebs"]