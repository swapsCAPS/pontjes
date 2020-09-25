#!/bin/bash
mkdir -p data/gtfs
rm -rf data/gtfs/*
curl http://gtfs.ovapi.nl/nl/gtfs-nl.zip -o gtfs-nl.zip
cd data/gtfs
unzip ../../gtfs-nl.zip
cd ../..
./full-import.sh
