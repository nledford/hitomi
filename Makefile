
docker_build:
	docker build -t nledford/chidori:latest .

docker_run:
	docker run -h docker-dev-mbp -e TZ="America/New_York" -it --rm --name chidori nledford/chidori:latest -l

install:
	cargo install --path .