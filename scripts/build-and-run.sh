#!/bin/bash
TAG=latest
docker build . -t pontjes:$TAG --no-cache
scripts/docker-run.sh $TAG
echo $(date) > $HOME/.last-import
