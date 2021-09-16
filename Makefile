tag:= $(shell date +%F-%H%M)

download:
	scripts/download.sh

import:
	scripts/import.sh

build-app:
	cargo build --release

build-pi:
	cross build --target aarch64-unknown-linux-gnu --release
	cp target/aarch64-unknown-linux-gnu/release/pontjes bin/pontjes_aarch64-unknown-linux-gnu

build-docker:
	docker build . -t pontjes:$(tag) -t pontjes:latest

restart:
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
		pontjes:latest

download-and-import: download import

full: download-and-import build-docker run

update-db: download-and-import restart
