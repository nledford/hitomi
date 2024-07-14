export DATABASE_URL := "sqlite:./data/hitomi.db"

# Default recipe to display help information
default:
    @just --list

# Build a docker image
[group('docker')]
docker-build:
    docker build -t nledford/hitomi:latest .

# Build and run a docker image
[group('docker')]
docker-run: docker-build
    docker run -e TZ="America/New_York" -e CONFIG_DIR="/config" -e PROFILES_DIRECTORY="/profiles" -it -v "./data/profiles:/profiles" -v "./data/config:/config" --rm --name hitomi nledford/hitomi:latest run

# Run clippy. Fails if clippy finds issues.
[group('rust')]
clippy:
    cargo clippy -- -Dwarnings

# Format rust files
[group('rust')]
format:
    cargo fmt --all

# Build rust files
[group('rust')]
build: format clippy
    cargo build

# Build and run tests
[group('rust')]
test: build
    cargo nextest run

# Run the application once
[group('rust')]
run: test
    cargo run -- run

# Run the application in loop mode
[group('rust')]
run-loop $RUST_BACKTRACE="full": test
    cargo run -- run -l

# Install the application to local machine
[group('rust')]
install: build test
    cargo install --path .

# Update rust crates
[group('rust')]
update:
    cargo upgrade; cargo update;

# Create a database using sqlx
[group('sqlx')]
create-db:
    sqlx database create

# Delete a database using sqlx
[group('sqlx')]
drop-db:
    sqlx database drop

# Recreates the database from scratch
[group('sqlx')]
rebuild-db: drop-db create-db

# Create a sqlx migration
[group('sqlx')]
add-migration migration:
    sqlx migrate add {{migration}}

# Run sqlx migrations
[group('sqlx')]
run-migrations:
    sqlx migrate run