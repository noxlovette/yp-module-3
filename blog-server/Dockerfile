FROM rust:1-slim-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends protobuf-compiler pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY blog-cli blog-cli
COPY blog-client blog-client
COPY blog-wasm blog-wasm
COPY blog-proto blog-proto
COPY blog-server blog-server

ENV SQLX_OFFLINE=true
RUN cargo build --release -p blog-server

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 appuser

COPY --from=builder /app/target/release/blog-server /usr/local/bin/blog-server

USER appuser
EXPOSE 8080 50051
ENTRYPOINT ["blog-server"]
