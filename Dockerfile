# syntax=docker/dockerfile:1.11

# --- Builder Stage ---
# Use the official Rust image as the build environment
FROM rust:slim AS builder

# Install musl tools for static linking (ensures compatibility with minimal base images)
ARG TARGETRSARCH=aarch64-unknown-linux-musl
RUN <<RUN_CMD_EOF
set -ex
apt-get update
apt-get install -y musl-tools
rustup target add ${TARGETRSARCH}
rm -rf /var/lib/apt/lists/*
RUN_CMD_EOF

# Set the working directory inside the container
WORKDIR /usr/src/tdns

# Copy manifest files first to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs and build to cache dependencies
RUN <<RUN_CMD_EOF
mkdir src/
echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
cargo build --release --target ${TARGETRSARCH}
RUN_CMD_EOF

# Copy the actual source code and build the final application
COPY . .
# Build the application for the musl target
RUN <<RUN_CARGO_BUILD_EOF
cargo build --release --target ${TARGETRSARCH}

# Copy to a temp folder because arg/env variables cannot be referenced in the
# later stages of a multi-stage build.
mkdir /tmp/tdns
cp /usr/src/tdns/target/${TARGETRSARCH}/release/tdns /tmp/tdns/
RUN_CARGO_BUILD_EOF

# --- Final Stage ---
# Start from scratch for a minimal, secure final image (or use alpine)
FROM scratch
# Optional: Create a non-root user for security (if using an image like alpine)
# RUN adduser -D -s /bin/sh appuser
# USER appuser

# Copy the statically-linked binary from the builder stage
COPY --from=builder /tmp/tdns/tdns ./tdns

# Run the binary when the container starts
CMD ["./tdns"]
