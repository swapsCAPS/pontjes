#!/bin/bash
cargo build --release
scripts/download-and-import.sh
docker build . -t pontjes:latest
scripts/docker-run.sh
