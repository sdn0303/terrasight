# syntax=docker/dockerfile:1

# -----------------------------------------------------------------------
# Stage 1: Builder — compile the Rust binary
# -----------------------------------------------------------------------
FROM rust:1.94-slim-bookworm AS builder

WORKDIR /app

# System dependencies required for sqlx (OpenSSL) and native-tls
RUN set -o pipefail && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Dependency caching layer:
# Copy ALL workspace manifests first so that `cargo build --release` can
# cache dependency compilation independently of source changes.
# The workspace has 6 members: root + 5 lib crates.
COPY Cargo.toml Cargo.lock ./
COPY lib/telemetry/Cargo.toml lib/telemetry/Cargo.toml
COPY lib/geo-math/Cargo.toml lib/geo-math/Cargo.toml
COPY lib/mlit-client/Cargo.toml lib/mlit-client/Cargo.toml
COPY lib/db/Cargo.toml lib/db/Cargo.toml
COPY lib/api-core/Cargo.toml lib/api-core/Cargo.toml

# Build dummy sources to cache all dependency compilation.
# This layer is reused as long as Cargo.toml / Cargo.lock are unchanged.
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    mkdir -p lib/telemetry/src && echo '' > lib/telemetry/src/lib.rs && \
    mkdir -p lib/geo-math/src && echo '' > lib/geo-math/src/lib.rs && \
    mkdir -p lib/mlit-client/src && echo '' > lib/mlit-client/src/lib.rs && \
    mkdir -p lib/db/src && echo '' > lib/db/src/lib.rs && \
    mkdir -p lib/api-core/src && echo '' > lib/api-core/src/lib.rs && \
    cargo build --release && \
    rm -rf src lib

# Copy all application source and migration files
COPY src/ src/
COPY lib/ lib/
COPY migrations/ migrations/

# Touch main.rs so Cargo notices the source change and relinks the binary
RUN touch src/main.rs && cargo build --release

# -----------------------------------------------------------------------
# Stage 2: Runtime — minimal image with non-root user
# -----------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

LABEL maintainer="realestate-api"
LABEL description="Real Estate Investment API — Rust Axum + PostGIS backend"
LABEL version="0.1.0"

# ca-certificates: required for TLS connections to external APIs (reinfolib)
# curl: used by Docker Compose healthcheck (CMD curl -f http://localhost:8000/api/health)
RUN set -o pipefail && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl && \
    rm -rf /var/lib/apt/lists/*

# Create a dedicated non-root user
RUN useradd --system --no-create-home --shell /usr/sbin/nologin appuser

COPY --from=builder /app/target/release/realestate-api /usr/local/bin/realestate-api

USER appuser

EXPOSE 8000

ENTRYPOINT ["realestate-api"]
