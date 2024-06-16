FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
LABEL authors="nledford"

WORKDIR /plex-playlists

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /plex-playlists/recipe.json recipe.json
# Install dependencies
RUN apt-get update && apt-get install -y llvm lld
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin plex-playlists

FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /plex-playlists
COPY --from=builder /plex-playlists/target/release/plex-playlists .
ENTRYPOINT ["./plex-playlists"]