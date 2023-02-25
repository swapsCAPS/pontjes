# Heen en weer

## Running
Install rust if necessary
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install deps if necessary
```
sudo apt install gcc make sqlite3 unzip
cargo install cross --git https://github.com/cross-rs/cross
```

Rocket needs nightly
```
rustup override set nightly
```

Download and initialize data
```
make download-and-import
```

Blast off
```
cargo run
```

The docker container mounts the static files and the templates.  
Editing these without a re-build is therefore possible, but does require a `make docker-restart` afterwards.  
Also, be careful ; )

## TODO
- [x] Compare joins vs pre imported data performance
- [x] Use joins in combination with indexes
- [x] Add manifest
- [x] Add service worker
- [x] Support optional build steps
- [x] Prevent >24 hour times
- [x] Fix build
- [x] Add favicon!
- [x] Put feed info in static
- [ ] Fix nightly version in place
- [x] Fix cross compilation
- [x] Update to rocket 5
- [x] Add date of gtfs file
- [ ] Optimize grouping by doing group_concat on date and trip_id in sql query
- [x] Add more clear back button
- [ ] Use diesel table aliasing once it lands... sql query can be done in one go, however diesel does not support it.
- [ ] Change "via" to "van" if "other" stop time is before selected stop time
- [x] Set up ARM cross compilation docker build
- [ ] Add some usage metrics
- [ ] Minify css
- [ ] Add ad. Boooo!
