# Build stage
FROM rust:1.82-alpine AS builder

# Install essential build tools
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl \
    curl \
    build-base \
    g++ \
    libc6-compat \
    pkgconfig

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl

# Install Rust toolchain
RUN rustup install stable
RUN rustup component add rust-src

# Install cargo-leptos
RUN cargo install cargo-leptos --version 0.2.29 --locked

WORKDIR /app
COPY . .


# Build project
ENV LEPTOS_OUTPUT_NAME="compareware"
RUN cargo leptos build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache openssl

WORKDIR /app
COPY --from=builder /app/target/release/compareware /app/compareware
COPY assets /app/assets

# Expose port and set entrypoint
EXPOSE 3000
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000
ENV LEPTOS_SITE_ROOT="site"
CMD ["/app/compareware"]