# syntax=docker/dockerfile:1

# Build stage
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# Create app directory
WORKDIR /app

# Copy Cargo files first for dependency caching
COPY Cargo.toml Cargo.lock* ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src
COPY rustfmt.toml clippy.toml ./

# Build the actual application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

# Create non-root user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Copy binary from builder
COPY --from=builder /app/target/release/workflow-compiler /usr/local/bin/workflow-compiler

# Set ownership
RUN chown appuser:appgroup /usr/local/bin/workflow-compiler

# Switch to non-root user
USER appuser

# Set environment variables
ENV RUST_LOG=info
ENV PORT=3000

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Run the application
CMD ["workflow-compiler"]
