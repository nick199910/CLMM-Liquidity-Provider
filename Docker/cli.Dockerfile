# =============================================================================
# CLMM Liquidity Provider - CLI Dockerfile
# =============================================================================
# Multi-stage build for optimized production image
#
# Build: docker build -f Docker/cli.Dockerfile -t clmm-lp-cli .
# Run:   docker run clmm-lp-cli analyze --symbol-a SOL --symbol-b USDC
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
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin clmm-lp-cli

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
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false clmm

# Copy binary from builder
COPY --from=builder /app/target/release/clmm-lp-cli /app/clmm-lp-cli

# Set ownership
RUN chown -R clmm:clmm /app

# Switch to non-root user
USER clmm

# Environment variables
ENV RUST_LOG=info

# Run the binary
ENTRYPOINT ["/app/clmm-lp-cli"]
CMD ["--help"]
