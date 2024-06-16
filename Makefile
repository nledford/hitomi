
docker_build:
	docker build -t nledford/plex-playlists:latest .

docker_run:
	docker run -h docker-dev-mbp -e TZ="America/New_York" -it --rm --name plexfm nledford/plex-playlists:latest -l

install:
	cargo install --path .