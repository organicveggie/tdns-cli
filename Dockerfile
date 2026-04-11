# syntax=docker/dockerfile:1.11

# --- Chef Setup Stage ---
# Use the official Rust image as the build environment
FROM rust:slim AS chef

# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install --locked cargo-chef 
WORKDIR /app

# --- Chef Planner Stage ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- Target Mapper Stage ---
FROM chef AS target-mapper
ARG TARGETARCH
# We use a single RUN to export the mapping to a file
RUN <<RUN_CMD_EOF
if [ "${TARGETARCH}" = "arm64" ]; then 
    echo "aarch64-unknown-linux-musl" > /target_triple
elif [ "${TARGETARCH}" = "amd64" ]; then 
    echo "x86_64-unknown-linux-musl" > /target_triple
elif [ "${TARGETARCH}" = "arm" ]; then 
    echo "armv7-unknown-linux-musleabi" > /target_triple
else 
    echo "${TARGETARCH}-unknown-linux-musl" > /target_triple
fi
RUN_CMD_EOF

# --- Chef Builder Stage ---
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
COPY --from=target-mapper /target_triple /target_triple

# Install musl tools for static linking (ensures compatibility with minimal base images)
RUN <<RUN_CMD_EOF
set -ex
apt-get update
apt-get install -y musl-tools
rm -rf /var/lib/apt/lists/*
RUN_CMD_EOF

# Build dependencies - this is the caching Docker layer!
RUN <<RUN_CMD_EOF
rustup target add $(cat /target_triple)
cargo chef cook --release --target $(cat /target_triple) --recipe-path recipe.json
RUN_CMD_EOF

# Set the working directory inside the container
WORKDIR /usr/src/tdns

# Copy the actual source code and build the final application
COPY . .

# Build the application for the musl target
RUN <<RUN_CARGO_BUILD_EOF
TDNS_TARGET_ARCH=$(cat /target_triple)
cargo build --release --target ${TDNS_TARGET_ARCH}

# Copy to a temp folder because arg/env variables cannot be referenced in the
# later stages of a multi-stage build.
mkdir /tmp/tdns
cp /usr/src/tdns/target/${TDNS_TARGET_ARCH}/release/tdns /tmp/tdns/
RUN_CARGO_BUILD_EOF

# --- Final Stage ---
# Start from scratch for a minimal, secure final image (or use alpine)
FROM scratch

# Copy the statically-linked binary from the builder stage
COPY --from=builder /tmp/tdns/tdns /tdns

# Run the binary when the container starts
CMD ["/tdns"]
