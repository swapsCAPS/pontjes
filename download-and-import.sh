#!/bin/bash
mkdir -p gtfs
rm -rf gtfs/*
curl http://gtfs.ovapi.nl/nl/gtfs-nl.zip -o gtfs-nl.zip
cd gtfs
unzip ../gtfs-nl.zip
cd ..
./full-import.sh
