# certificate-service — Rust (latest stable)
# Build and run the Axum API (e.g. for Docker Compose or CI).
# For SQLx compile-time check, pass DATABASE_URL at build time or use sqlx prepare.

FROM rust:bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

# Slim runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Non-root user for security
RUN adduser --disabled-password --gecos "" appuser

COPY --from=builder /app/target/release/certificate-service /usr/local/bin/
# /usr/local/bin is typically world-executable; appuser can run the binary
USER appuser

EXPOSE 3001
ENV PORT=3001
CMD ["certificate-service"]