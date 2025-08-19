###############################################
# Base with cargo-chef for better caching
###############################################
FROM rust:1.89 AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

###############################################
# Compute dependency recipe
###############################################
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
# Include sqlx-data.json if present (no-op if absent when using COPY . . in builder)
RUN cargo chef prepare --recipe-path recipe.json

###############################################
# Build dependencies layer
###############################################
FROM chef AS deps
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

###############################################
# Build application (auto SQLx offline/online)
###############################################
FROM rust:1.89 AS builder
WORKDIR /app

# System deps
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Reuse dependency cache
COPY --from=deps /app/target /app/target
COPY --from=deps /usr/local/cargo /usr/local/cargo

# Copy full source (brings sqlx-data.json if exists)
COPY . .

# Build args for SQLx mode and online DB
ARG SQLX_OFFLINE=auto
ARG DATABASE_URL

# Build with smart fallback: use offline when sqlx-data.json exists or SQLX_OFFLINE=true, otherwise require DATABASE_URL
RUN set -euo pipefail; \
    mode="${SQLX_OFFLINE}"; \
    if [ "$mode" = "true" ] || { [ "$mode" = "auto" ] && [ -f sqlx-data.json ]; }; then \
      export SQLX_OFFLINE=true; \
      echo "Building with SQLX_OFFLINE (using sqlx-data.json)"; \
    else \
      if [ -z "${DATABASE_URL:-}" ]; then \
        echo "ERROR: No sqlx-data.json and DATABASE_URL not provided. Set --build-arg DATABASE_URL=... or provide sqlx-data.json." >&2; \
        exit 1; \
      fi; \
      export SQLX_OFFLINE=false; \
      export DATABASE_URL="$DATABASE_URL"; \
      echo "Building with SQLx online using DATABASE_URL."; \
    fi; \
    cargo build --release

###############################################
# Runtime image
###############################################
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/trackivity ./trackivity
COPY --from=builder /app/migrations ./migrations

EXPOSE 3000
CMD ["./trackivity"]
