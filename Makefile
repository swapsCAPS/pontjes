tag:= latest

download:
	scripts/download.sh

import:
	scripts/import.sh

build-app:
	cargo build --release

docker-build:
	docker build . -t pontjes:$(tag)

docker-restart:
	docker restart pontjes

run:
	docker stop pontjes; true
	docker rm -f pontjes; true
	docker run \
		-d \
		-p 6376:6376 \
		-v $(PWD)/data/:/data \
		-v $(PWD)/templates:/templates \
		-v $(PWD)/public:/public \
		--name pontjes \
		--restart always \
		pontjes:$(tag)

download-and-import: download import

full: download-and-import docker-build run

update-db: download-and-import docker-restart
