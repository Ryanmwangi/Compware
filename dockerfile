# Build stage
FROM rust:1.83.0-slim-bullseye as builder

# Install essential build tools
RUN apt-get update && \
    apt-get install -y \
    libsqlite3-dev \
    build-essential \
    clang \
    libssl-dev \
    pkg-config \
    curl \
    cmake \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN rustup component add rust-src

# Install cargo-leptos & wasm-bindgen-cli 
RUN cargo install cargo-leptos --version 0.2.24 --locked
RUN cargo install wasm-bindgen-cli --version 0.2.99 --locked

# Build application
WORKDIR /app
COPY . .
# Explicitly set WASM target
RUN rustup target add wasm32-unknown-unknown
# Build project
ENV LEPTOS_OUTPUT_NAME="compareware"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3004"
ENV LEPTOS_SITE_ROOT="site"

# Build with release profile
RUN cargo leptos build --release

# Runtime stage
FROM debian:bullseye-slim 

# Install runtime dependencies in Debian
RUN apt-get update && \
    apt-get install -y \
    libssl-dev \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy build artifacts
COPY --from=builder /app/target/release/compareware /app/
COPY --from=builder /app/target/site /app/site
COPY assets /app/assets

# Configure container, expose port and set entrypoint
WORKDIR /app 
EXPOSE 3004
ENV LEPTOS_SITE_ADDR="0.0.0.0:3004"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_OPTIONS='{"site_addr":"0.0.0.0:3004"}'
CMD ["./compareware"]