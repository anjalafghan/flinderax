# One image, one server: the Rust binary serves the API and the built web app.

# Stage 1: build the web app with Bun
FROM oven/bun:1 AS frontend
WORKDIR /app
COPY package.json bun.lock ./
COPY web/package.json web/
RUN bun install --frozen-lockfile
COPY web/ web/
RUN bun run --cwd web build

# Stage 2: build the Rust server
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y protobuf-compiler

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release --recipe-path recipe.json

COPY . .
# Compile-time SQL checks use the committed .sqlx data (run `cargo sqlx prepare` after changing queries).
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --bin flinderax

# Final image
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN mkdir -p /data
COPY --from=builder /app/target/release/flinderax /usr/local/bin
COPY --from=frontend /app/web/dist ./dist
ENV STATIC_DIR=/app/dist
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/flinderax"]
