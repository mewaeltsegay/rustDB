# Multi-stage Dockerfile for rustDB
# Stage 1: build
FROM rust:latest as builder

WORKDIR /usr/src/app

# Copy manifest and source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Stage 2: runtime
FROM debian:bookworm-slim

# Install CA certs for HTTPS clients
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/lab /usr/local/bin/lab

# Default workdir
WORKDIR /usr/local/bin

# Expose common ports for convenience (primary + replicas)
EXPOSE 8000 8001 8002

ENTRYPOINT ["/usr/local/bin/lab"]
