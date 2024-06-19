FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
LABEL authors="nledford"

WORKDIR /hitomi

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /hitomi/recipe.json recipe.json
# Install dependencies
RUN apt-get update && apt-get install -y llvm lld
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin hitomi

FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /hitomi
COPY --from=builder /hitomi/target/release/hitomi .
ENTRYPOINT ["./hitomi"]