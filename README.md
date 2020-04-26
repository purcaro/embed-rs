Embedding JSON with Rust, LMDB, and Flexbuffers
===============================================

Load some test data:
```shell
cargo run --release --bin load -- --datadir db --attr '@id' --in example/data.jsonlines
cargo run --release --bin load -- --datadir db --dbname frames --key Experiment --in example/experiment-frame.json
echo 'null' | cargo run --release --bin load -- --datadir db --dbname frames --key null
```

Construct hierarchical output:
```shell
cargo run --release --bin embed -- --datadir db --frame Experiment /experiments/ENCSR807BGP/ --json
```

Performance
-----------

After loading the full dataset (1.2M docs, 2.3GB) framing a doc takes 6.25ms to a flexbuffer:
```shell
$ RUST_LOG=info cargo run --release --bin embed -- --datadir db --frame Experiment /experiments/ENCSR807BGP/ --out /dev/null
[2020-04-26T04:09:23Z INFO  embed] txn:6.252747ms embed:6.238401ms output:13.846µs root:/experiments/ENCSR807BGP/
```

And 12.7ms to json:
```shell
$ RUST_LOG=info cargo run --release --bin embed -- --datadir db --frame Experiment /experiments/ENCSR807BGP/ --out /dev/null --json
[2020-04-26T04:09:44Z INFO  embed] txn:12.708103ms embed:6.249224ms output:6.458304ms root:/experiments/ENCSR807BGP/
```

For comparison, constructing such a document in Postgres from JSONB takes 180ms (prepared, specialized SQL query) or 200ms (prepare, recursive function.)
