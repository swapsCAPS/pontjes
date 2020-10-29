# Heen en weer

## Running
Install rust if necessary
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install deps if necessary
```
sudo apt install sqlite3
```

Rocket needs nightly
```
rustup override set nightly
```

Download and initialize data
```
./scripts/download-and-import.sh
```

Blast off
```
cargo run
```

## TODO
- [x] Compare joins vs pre imported data performance
- [x] Use joins in combination with indexes
- [ ] Optimize grouping by doing group_concat on date and trip_id in sql query
- [ ] Add more clear back button
- [ ] Use diesel table aliasing once it lands... sql query can be done in one go, however diesel does not support it.
- [ ] Add favicon!
- [ ] Change "via" to "van" if "other" stop time is before selected stop time
- [ ] Set up ARM cross compilation docker build
- [ ] Add some usage metrics
- [ ] Minify css
- [ ] Add ad
