#!/bin/bash
TAG=latest
docker build . -t pontjes:$TAG
scripts/docker-run.sh $TAG
