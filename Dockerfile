# Cross-compiles caster-control to a Windows .exe, using cargo-chef so the
# dependency build is cached in its own Docker layer.

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
RUN apt-get update \
    && apt-get install -y --no-install-recommends gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-pc-windows-gnu
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
WORKDIR /app

# Work out the dependency "recipe" from Cargo.toml / Cargo.lock.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies only (cached until Cargo.toml/Cargo.lock change),
# then build the app itself.
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-pc-windows-gnu --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-pc-windows-gnu

# Export just the .exe (used with `docker build --output`).
FROM scratch AS export
COPY --from=builder /app/target/x86_64-pc-windows-gnu/release/caster-control.exe /
