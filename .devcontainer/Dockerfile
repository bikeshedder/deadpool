FROM mcr.microsoft.com/vscode/devcontainers/base:1-bookworm

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install clang lld libsqlite3-dev libmariadb-compat-dev libmariadb-dev libpq-dev \
   && apt-get autoremove -y && apt-get clean -y

USER vscode

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
