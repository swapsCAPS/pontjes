#!/bin/bash
TAG=latest
cargo build --release
scripts/download-and-import.sh
docker build . -t pontjes:$TAG
scripts/docker-run.sh $TAG
