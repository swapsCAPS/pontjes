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
- [ ] Compare joins vs pre imported data performance
- [ ] Change "via" to "van" if "other" stop time is before selected stop time
- [ ] Set up ARM cross compilation docker build
- [ ] Add some usage metrics
- [ ] Minify css
- [ ] Add ad
