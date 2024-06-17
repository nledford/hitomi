
docker_build:
	docker build -t nledford/chidori:latest .

docker_run:
	docker run -h docker-dev-mbp\
		-e TZ="America/New_York"\
		-it -v "./data/profiles:/data/profiles"\
		--rm --name chidori nledford/chidori:latest run -l

install:
	cargo install --path .