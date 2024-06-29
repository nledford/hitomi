run: test
    cargo run -- run

docker-build:
    docker build -t nledford/hitomi:latest .

docker-run: docker-build
    docker run -h docker-dev-mbp
    -e TZ="America/New_York" -e CONFIG_DIR="/config" -e PROFILES_DIRECTORY="/profiles" -it -v "./data/profiles:/profiles" -v "./data/config:/config" --rm --name hitomi nledford/hitomi:latest run

clippy:
    cargo clippy

format:
    cargo fmt --all -- --check

build:
    cargo build

test: build
    cargo test

run-loop: test
    cargo run -- run -l

install: build test
    cargo install --path .