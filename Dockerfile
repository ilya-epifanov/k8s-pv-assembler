FROM rust:1.60-bullseye AS chef
RUN cargo install cargo-chef --version 0.1.35
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/pv-assembler /usr/local/bin/pv-assembler

EXPOSE 8443
CMD ["/usr/local/bin/pv-assembler"]