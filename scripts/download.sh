#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
echo $DIR
DATA_DIR="$DIR/../data"
echo $DATA_DIR
mkdir -p $DATA_DIR/gtfs
rm -rf $DATA_DIR/gtfs/*
curl http://gtfs.ovapi.nl/nl/gtfs-nl.zip -o $DATA_DIR/gtfs/gtfs-nl.zip || exit 1
cd $DATA_DIR/gtfs
unzip gtfs-nl.zip || exit 1
cd $DIR
