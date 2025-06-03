FROM lukemathwalker/cargo-chef:latest-rust-1.85.1 AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef:latest-rust-1.85.1 AS cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends --assume-yes \
    libprotobuf-dev \
    build-essential \
    clang-tools-16 \
    git \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    libudev-dev \
    && apt-get clean
RUN cargo chef cook --release --recipe-path recipe.json

FROM docker.io/rust:1.85.1 AS builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target /app/target
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends --assume-yes \
    libprotobuf-dev \
    build-essential \
    clang-tools-16 \
    git \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    libudev-dev \
    && apt-get clean
RUN cargo build --release

FROM docker.io/debian:bookworm-slim
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y ca-certificates curl
WORKDIR /app
COPY --from=builder /app/target/release/namada-supply-webserver /app/namada-supply-webserver
ENTRYPOINT ["./app/namada-supply-webserver"]