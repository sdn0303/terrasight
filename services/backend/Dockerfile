# -----------------------------------------------------------------------
# Stage 1: Builder — compile the Rust binary
# -----------------------------------------------------------------------
FROM rust:1.94-slim-bookworm AS builder

WORKDIR /app

# System dependencies required for sqlx (OpenSSL) and native-tls
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Dependency caching layer:
# Copy ALL workspace manifests first so that `cargo build --release` can
# cache dependency compilation independently of source changes.
COPY Cargo.toml Cargo.lock ./
COPY lib/domain/Cargo.toml lib/domain/Cargo.toml
COPY lib/server/Cargo.toml lib/server/Cargo.toml
COPY lib/geo/Cargo.toml lib/geo/Cargo.toml
COPY lib/mlit/Cargo.toml lib/mlit/Cargo.toml

# Build dummy sources to cache all dependency compilation.
# This layer is reused as long as Cargo.toml / Cargo.lock are unchanged.
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    mkdir -p lib/domain/src && echo '' > lib/domain/src/lib.rs && \
    mkdir -p lib/server/src && echo '' > lib/server/src/lib.rs && \
    mkdir -p lib/geo/src && echo '' > lib/geo/src/lib.rs && \
    mkdir -p lib/mlit/src && echo '' > lib/mlit/src/lib.rs && \
    cargo build --release && \
    rm -rf src lib

# Copy all application source and migration files
COPY src/ src/
COPY lib/ lib/
COPY migrations/ migrations/

# Invalidate all workspace crate fingerprints so Cargo recompiles from real sources
RUN touch src/main.rs && \
    find lib -name "lib.rs" -exec touch {} + && \
    cargo build --release

# -----------------------------------------------------------------------
# Stage 2: Runtime — minimal image with non-root user
# -----------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

LABEL maintainer="terrasight"
LABEL description="Terrasight API — Rust Axum + PostGIS backend"
LABEL version="0.1.0"

# ca-certificates: required for TLS connections to external APIs
# curl: used by Docker Compose healthcheck
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl && \
    rm -rf /var/lib/apt/lists/*

# Create a dedicated non-root user
RUN useradd --system --no-create-home --shell /usr/sbin/nologin appuser

COPY --from=builder /app/target/release/terrasight-api /usr/local/bin/terrasight-api

USER appuser

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["curl", "-f", "http://localhost:8000/api/v1/health"]

ENTRYPOINT ["terrasight-api"]
