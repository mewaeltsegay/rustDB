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

# Install CA certs and other utilities
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        curl \
        netcat-openbsd \
        dnsutils \
        iputils-ping \
        procps \
        && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/lab /usr/local/bin/lab

# Create directory for persistent data
RUN mkdir -p /data
WORKDIR /data

# Expose ports:
# - 8000-8002: Server ports (primary + replicas)
# - 8010-8019: Reserved for shards
EXPOSE 8000-8002 8010-8019

# Set environment variables
ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug

ENTRYPOINT ["/usr/local/bin/lab"]
