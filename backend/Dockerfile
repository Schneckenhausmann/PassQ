# Use specific Rust version with security patches
FROM rust:1.82.0-slim-bookworm AS builder

# Create non-root user
RUN groupadd -r passq && useradd -r -g passq passq

# Install only necessary packages and clean up
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /app

# Copy manifest first to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs so `cargo fetch` works without full code
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Fetch and build dependencies (latest versions from Cargo.lock or crates.io)
RUN cargo update && cargo fetch

# Copy the rest of the source code
COPY . .

# Build in release mode and strip symbols to reduce size
RUN cargo build --release && strip target/release/backend

# Final minimal image with specific version
FROM debian:bookworm-20241202-slim

# Create non-root user in final image
RUN groupadd -r passq && useradd -r -g passq passq

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Copy binary and set ownership
COPY --from=builder --chown=passq:passq /app/target/release/backend /usr/local/bin/passq
COPY --from=builder --chown=passq:passq /app/migrations /app/migrations

# Switch to non-root user
USER passq

EXPOSE 8080

CMD ["passq"]
