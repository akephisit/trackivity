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
# Only copy essential files for dependency planning
RUN cargo chef prepare --recipe-path recipe.json

###############################################
# Build dependencies layer (cached when dependencies unchanged)
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

# Build with SQLx offline cache (much faster)
ENV SQLX_OFFLINE=true
RUN cargo build --release

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
