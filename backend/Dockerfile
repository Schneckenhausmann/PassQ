# Use latest stable Rust with slim Debian Bookworm
FROM rust:slim-bookworm AS builder

# Install PostgreSQL development libraries
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*

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

# Final minimal image
FROM debian:bookworm-slim

# Install PostgreSQL runtime libraries
RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*

# Set env vars
ENV DATABASE_URL=postgres://passq:securepassword@db:5432/passq \
    JWT_SECRET=your_very_secure_jwt_secret_here \
    ENCRYPTION_KEY=0123456789abcdef0123456789abcdef

# Copy the stripped binary and migrations
COPY --from=builder /app/target/release/backend /usr/local/bin/passq
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080

CMD ["passq"]
