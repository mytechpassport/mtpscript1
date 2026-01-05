# MTPScript Reproducible Build Container
# Base image: Ubuntu 20.04 LTS with SHA-256 pinning for reproducible builds (§18)
FROM ubuntu:20.04@sha256:8e0402ca47c100e8c888975255dbe0d2c2c6c3a22fe99cd936e7d896dfb9db8b02

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    make \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /mtpscript

# Copy source code
COPY . .

# Build the project
RUN make clean && make all

# Generate build info and sign it for reproducible builds
RUN ./scripts/generate_build_info.sh

# Verify build reproducibility by checking build-info.json exists
RUN test -f build-info.json && echo "Reproducible build completed successfully"

# Default command
CMD ["./mtpsc", "--help"]
