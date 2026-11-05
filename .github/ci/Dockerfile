FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
ENV PATH=/opt/cargo/bin:/opt/rustup/toolchains/nightly-2026-04-01-x86_64-unknown-linux-gnu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    cmake \
    ninja-build \
    python3 \
    python3-pip \
    libclang-dev \
    libudev-dev \
    build-essential \
    pkg-config \
  && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --profile minimal --default-toolchain nightly-2026-04-01 && \
    rustup component add rust-src && \
    rustup target add riscv32imc-esp-espidf

RUN cargo install --locked ldproxy espflash && \
    rm -rf /opt/cargo/registry /opt/cargo/git
