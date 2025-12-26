# syntax=docker/dockerfile:1

############################
# 1) Build stage
############################
FROM rust:1-bookworm AS builder
WORKDIR /app

# If you use crates that compile against OpenSSL, this helps builds succeed.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
  && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release --locked --bin server

############################
# 2) Runtime stage
############################
FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/server /app/server

# Cloud Run sends traffic to PORT (typically 8080); your app must read PORT.
ENV PORT=8080
EXPOSE 8080

USER nonroot:nonroot
CMD ["/app/server"]
