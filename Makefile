
docker_build:
	docker build -t nledford/chidori:latest .

docker_run:
	docker run -h docker-dev-mbp\
		-e TZ="America/New_York"\ -e PROFILES_DIRECTORY=/data/profiles
		-it -v "./data/profiles:/data/profiles" -v "./data/config:/data/config"\
		--rm --name chidori nledford/chidori:latest run -l

install:
	cargo install --path .