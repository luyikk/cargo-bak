# cargo backup
backup and restore $CARGO_HOME cache to cargo_bak.zip
cargo restore for custom docker ci

list of backup directories:
```shell
   $CARGO_HOME/registry/index/
   $CARGO_HOME/registry/cache/
   $CARGO_HOME/git/db/
```

usage:

* backup
```shell
cargo-bak bak
cargo-bak bak -s [filename]
```

* backup zip compression level default 0
* change compression level use -c
```shell
cargo-bak bak -c 6
```

* restore
```shell
  cargo-bak restore ./cargo_bak.zip 
```
