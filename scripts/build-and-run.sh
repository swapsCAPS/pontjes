#!/bin/bash
scripts/download-and-import.sh
docker build . -t pontjes:1
docker stop pontjes
docker rm -f pontjes
docker run --rm -d --net host --name pontjes --restart always pontjes:1
docker image prune -a
