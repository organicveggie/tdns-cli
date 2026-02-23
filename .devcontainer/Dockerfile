FROM rust:latest

RUN <<RUN_CMD_EOF
set -ex
curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/axodotdev/cargo-dist/releases/latest/download/cargo-dist-installer.sh | sh
cargo install cargo-release
RUN_CMD_EOF
