FROM rust:1.88-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/openai-reasoning-proxy /usr/local/bin/openai-reasoning-proxy

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000

ENTRYPOINT ["openai-reasoning-proxy"]
