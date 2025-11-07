# Build stage
FROM rust:slim AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Copy actual source code
COPY src ./src

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/ /app/tezcatlipoca-auth

# Create directory for logs
RUN mkdir -p /app/logs

# Expose the default port
EXPOSE 8199

# Run the application
CMD ["/app/tezcatlipoca-auth/tezcatlipoca-auth"]
