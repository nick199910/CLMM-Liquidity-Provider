# =============================================================================
# CLMM Liquidity Provider - API Server Dockerfile
# =============================================================================
# Multi-stage build for optimized production image
#
# Build: docker build -f Docker/api.Dockerfile -t clmm-lp-api .
# Run:   docker run -p 8080:8080 clmm-lp-api
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Build
# -----------------------------------------------------------------------------
FROM rust:1.90-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY . .

# Build release binary
RUN cargo build --release --bin clmm-lp-api

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false clmm

# Copy binary from builder
COPY --from=builder /app/target/release/clmm-lp-api /app/clmm-lp-api

# Set ownership
RUN chown -R clmm:clmm /app

# Switch to non-root user
USER clmm

# Environment variables
ENV RUST_LOG=info
ENV API_HOST=0.0.0.0
ENV API_PORT=8080

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

# Run the binary
ENTRYPOINT ["/app/clmm-lp-api"]
