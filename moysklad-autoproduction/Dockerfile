# Build stage - use latest stable Rust
FROM rust:alpine AS builder

# Install build dependencies for static linking
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build in release mode with static linking
RUN cargo build --release

# Runtime stage - minimal Alpine image
FROM alpine:latest

# Install runtime dependencies and curl for healthcheck
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    curl

WORKDIR /app

# Copy binary file
COPY --from=builder /app/target/release/moysklad_autoproduction /app/

# Create unprivileged user
RUN adduser -D -s /bin/false appuser && \
    chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 8084

# Run application
CMD ["./moysklad_autoproduction"]
