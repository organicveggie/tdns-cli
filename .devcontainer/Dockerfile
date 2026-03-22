FROM rust:latest

RUN <<RUN_CMD_EOF
set -ex
apt update
apt install -y --fix-missing --no-install-recommends \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

# Setup repository for Docker CE
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc

# Add the repository to Apt sources:
tee /etc/apt/sources.list.d/docker.sources <<DOCKER_REPO_EOF
Types: deb
URIs: https://download.docker.com/linux/debian
Suites: $(. /etc/os-release && echo "$VERSION_CODENAME")
Components: stable
Signed-By: /etc/apt/keyrings/docker.asc
DOCKER_REPO_EOF

# Install Docker CLI tools
apt-get update
apt install -y docker-ce-cli

# Cleanup to reduce image size
apt-get clean -y
apt-get clean -y
apt-get autoclean -y

rm -rf /var/lib/apt/lists/*
RUN_CMD_EOF

# Install Dist and Cargo Release tools
RUN <<RUN_CMD_EOF
curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/axodotdev/cargo-dist/releases/latest/download/cargo-dist-installer.sh | sh
cargo install cargo-release
RUN_CMD_EOF
