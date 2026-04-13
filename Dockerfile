FROM rust:1.90-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p rumary-api

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rumary-api /usr/local/bin/rumary-api
COPY --from=builder /app/crates/rumary-api/migrations ./migrations

EXPOSE 3000

CMD ["rumary-api"]

USER nobody