# Stage 1: Build React frontend
FROM node:20-alpine AS frontend
WORKDIR /app
COPY web/package*.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# Stage 2: Build Rust backend
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
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --bin flinderax

# Final image
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN mkdir -p /data
COPY --from=builder /app/target/release/flinderax /usr/local/bin
COPY --from=frontend /app/dist ./dist
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/flinderax"]
