# tests/fixtures/native/update-linux.Dockerfile

FROM ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl build-essential python3 pkexec polkitd dbus \
    desktop-file-utils shared-mime-info util-linux apt \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.sh \
    && sh /tmp/rustup.sh -y --profile minimal --default-toolchain 1.99.0 \
    && rm /tmp/rustup.sh
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /repo
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY tools ./tools
RUN cargo fetch --locked
# Actual source is mounted read-only; test trust is generated in /evidence only.
CMD ["python3", "scripts/check-linux-update.py", "--output", "/evidence/linux", "--offline"]
