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

Allocation
----------

Generatin multiple outputs (simulating a web server use case) show that we can avoid heap allocations entirely once the `flexbuffer::Builder` buffers have grown to size.

```shell
$ RUST_LOG=trace cargo run --release --features logging_allocator --bin embed -- --datadir db --frame Experiment --out /dev/null /experiments/ENCSR807BGP/ /experiments/ENCSR173USI/
...
[2020-04-26T22:18:11Z TRACE embed] begin_txn
[2020-04-26T22:18:11Z TRACE embed] begin_embed
[2020-04-26T22:18:11Z TRACE logging_allocator] alloc [address=0x7fb8ca403190, size=3, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403190, size=3, align=1] to [address=0x7fb8ca403190, size=6, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] alloc [address=0x7fb8ca4034d0, size=8, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] alloc [address=0x7fb8ca4034e0, size=16, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403190, size=6, align=1] to [address=0x7fb8ca4034f0, size=30, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca4034f0, size=30, align=1] to [address=0x7fb8ca4034f0, size=60, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca4034e0, size=16, align=8] to [address=0x7fb8ca403530, size=32, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca4034d0, size=8, align=8] to [address=0x7fb8ca4034d0, size=16, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403530, size=32, align=8] to [address=0x7fb8ca403530, size=64, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403530, size=64, align=8] to [address=0x7fb8ca403530, size=128, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca4034f0, size=60, align=1] to [address=0x7fb8ca403790, size=120, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca4034d0, size=16, align=8] to [address=0x7fb8ca4034d0, size=32, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403790, size=120, align=1] to [address=0x7fb8ca403790, size=240, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca4034d0, size=32, align=8] to [address=0x7fb8ca4034d0, size=64, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403530, size=128, align=8] to [address=0x7fb8ca403880, size=256, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403790, size=240, align=1] to [address=0x7fb8ca403980, size=480, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca4034d0, size=64, align=8] to [address=0x7fb8ca4034d0, size=128, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403880, size=256, align=8] to [address=0x7fb8ca403cd0, size=512, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403980, size=480, align=1] to [address=0x7fb8ca801000, size=1616, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca801000, size=1616, align=1] to [address=0x7fb8ca80a000, size=3232, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403cd0, size=512, align=8] to [address=0x7fb8ca800000, size=1024, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca4034d0, size=128, align=8] to [address=0x7fb8ca4034d0, size=256, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca800000, size=1024, align=8] to [address=0x7fb8ca801000, size=2048, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca4034d0, size=256, align=8] to [address=0x7fb8ca403880, size=512, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca80a000, size=3232, align=1] to [address=0x7fb8ca80a000, size=6464, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca403880, size=512, align=8] to [address=0x7fb8ca800000, size=1024, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca801000, size=2048, align=8] to [address=0x7fb8ca80ba00, size=4096, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca80a000, size=6464, align=1] to [address=0x7fb8ca80ca00, size=12928, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca80ca00, size=12928, align=1] to [address=0x7fb8ca80ca00, size=25856, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca80ca00, size=25856, align=1] to [address=0x7fb8d0308000, size=51712, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8ca800000, size=1024, align=8] to [address=0x7fb8ca801000, size=2048, align=8]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8d0308000, size=51712, align=1] to [address=0x7fb8d0308000, size=103424, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8d0308000, size=103424, align=1] to [address=0x7fb8d0308000, size=206848, align=1]
[2020-04-26T22:18:11Z TRACE logging_allocator] realloc [address=0x7fb8d0308000, size=206848, align=1] to [address=0x7fb8d0308000, size=413696, align=1]
[2020-04-26T22:18:11Z TRACE embed] end_embed
[2020-04-26T22:18:11Z TRACE embed] begin_output
[2020-04-26T22:18:11Z TRACE embed] end_output
[2020-04-26T22:18:11Z TRACE embed] end_txn
[2020-04-26T22:18:11Z INFO  embed] txn:6.641572ms embed:6.62625ms output:4.106µs root:/experiments/ENCSR807BGP/
[2020-04-26T22:18:11Z TRACE embed] begin_txn
[2020-04-26T22:18:11Z TRACE embed] begin_embed
[2020-04-26T22:18:11Z TRACE embed] end_embed
[2020-04-26T22:18:11Z TRACE embed] begin_output
[2020-04-26T22:18:11Z TRACE embed] end_output
[2020-04-26T22:18:11Z TRACE embed] end_txn
[2020-04-26T22:18:11Z INFO  embed] txn:2.53788ms embed:2.525623ms output:949ns root:/experiments/ENCSR173USI/
```

(The speed difference here is due to the second result being smaller.)
