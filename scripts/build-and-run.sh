#!/bin/bash
cargo build --release
scripts/download-and-import.sh
docker build . -t pontjes:1
scripts/docker-run.sh
