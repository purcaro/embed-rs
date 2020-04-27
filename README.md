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
[2020-04-27T07:48:54Z INFO  embed] txn:6.230217ms embed:6.211199ms output:15.975µs size:291366 root:/experiments/ENCSR807BGP/
```

And 8.5ms to json:
```shell
$ RUST_LOG=info cargo run --release --bin embed -- --datadir db --frame Experiment /experiments/ENCSR807BGP/ --out /dev/null --json
[2020-04-27T07:48:29Z INFO  embed] txn:8.509165ms embed:6.118059ms output:2.388125ms size:291366 root:/experiments/ENCSR807BGP/
```

For comparison, constructing such a document in Postgres from JSONB takes 180ms (prepared, specialized SQL query) or 200ms (prepare, recursive function.)

Allocation
----------

Generatin multiple outputs (simulating a web server use case) show that we can avoid heap allocations entirely once the `flexbuffer::Builder` buffers have grown to size.

```shell
$ RUST_LOG=trace cargo run --release --features logging_allocator --bin embed -- --datadir db --frame Experiment --out /dev/null /experiments/ENCSR807BGP/ /experiments/ENCSR173USI/ /experiments/ENCSR807BGP/ /experiments/ENCSR173USI/ --json
...
2020-04-27T07:46:51Z TRACE embed] begin_txn
[2020-04-27T07:46:51Z TRACE embed] begin_embed
[2020-04-27T07:46:51Z TRACE logging_allocator] alloc [address=0x7fe639c03100, size=3, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c03100, size=3, align=1] to [address=0x7fe639c03100, size=6, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] alloc [address=0x7fe639c02c30, size=8, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] alloc [address=0x7fe639c03750, size=16, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c03100, size=6, align=1] to [address=0x7fe639c03760, size=30, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c03760, size=30, align=1] to [address=0x7fe639c03760, size=60, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c03750, size=16, align=8] to [address=0x7fe639d00100, size=32, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c02c30, size=8, align=8] to [address=0x7fe639c02c30, size=16, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639d00100, size=32, align=8] to [address=0x7fe639d00100, size=64, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639d00100, size=64, align=8] to [address=0x7fe639d00100, size=128, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c03760, size=60, align=1] to [address=0x7fe639c03760, size=120, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c02c30, size=16, align=8] to [address=0x7fe639d005e0, size=32, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c03760, size=120, align=1] to [address=0x7fe639c03760, size=240, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639d005e0, size=32, align=8] to [address=0x7fe639e00360, size=64, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639d00100, size=128, align=8] to [address=0x7fe639d00100, size=256, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c03760, size=240, align=1] to [address=0x7fe639c03760, size=480, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639e00360, size=64, align=8] to [address=0x7fe639e00360, size=128, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639d00100, size=256, align=8] to [address=0x7fe639d00100, size=512, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639c03760, size=480, align=1] to [address=0x7fe63a004000, size=1616, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe63a004000, size=1616, align=1] to [address=0x7fe63a004000, size=3232, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639d00100, size=512, align=8] to [address=0x7fe63a004e00, size=1024, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639e00360, size=128, align=8] to [address=0x7fe639e00360, size=256, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe63a004e00, size=1024, align=8] to [address=0x7fe63a004e00, size=2048, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639e00360, size=256, align=8] to [address=0x7fe639e00360, size=512, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe63a004000, size=3232, align=1] to [address=0x7fe63a005600, size=6464, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe639e00360, size=512, align=8] to [address=0x7fe63a000000, size=1024, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe63a004e00, size=2048, align=8] to [address=0x7fe63a007000, size=4096, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe63a005600, size=6464, align=1] to [address=0x7fe63a008000, size=12928, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe63a008000, size=12928, align=1] to [address=0x7fe63a008000, size=25856, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe63a008000, size=25856, align=1] to [address=0x7fe640308000, size=51712, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe63a000000, size=1024, align=8] to [address=0x7fe63a004000, size=2048, align=8]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe640308000, size=51712, align=1] to [address=0x7fe640308000, size=103424, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe640308000, size=103424, align=1] to [address=0x7fe640308000, size=206848, align=1]
[2020-04-27T07:46:51Z TRACE logging_allocator] realloc [address=0x7fe640308000, size=206848, align=1] to [address=0x7fe640308000, size=413696, align=1]
[2020-04-27T07:46:51Z TRACE embed] end_embed
[2020-04-27T07:46:51Z TRACE embed] begin_output
[2020-04-27T07:46:51Z TRACE embed] end_output
[2020-04-27T07:46:51Z TRACE embed] end_txn
[2020-04-27T07:46:51Z INFO  embed] txn:9.06603ms embed:6.709545ms output:2.336506ms size:291366 root:/experiments/ENCSR807BGP/
[2020-04-27T07:46:51Z TRACE embed] begin_txn
[2020-04-27T07:46:51Z TRACE embed] begin_embed
[2020-04-27T07:46:51Z TRACE embed] end_embed
[2020-04-27T07:46:51Z TRACE embed] begin_output
[2020-04-27T07:46:51Z TRACE embed] end_output
[2020-04-27T07:46:51Z TRACE embed] end_txn
[2020-04-27T07:46:51Z INFO  embed] txn:3.834515ms embed:2.930335ms output:877.049µs size:103666 root:/experiments/ENCSR173USI/
[2020-04-27T07:46:51Z TRACE embed] begin_txn
[2020-04-27T07:46:51Z TRACE embed] begin_embed
[2020-04-27T07:46:51Z TRACE embed] end_embed
[2020-04-27T07:46:51Z TRACE embed] begin_output
[2020-04-27T07:46:51Z TRACE embed] end_output
[2020-04-27T07:46:51Z TRACE embed] end_txn
[2020-04-27T07:46:51Z INFO  embed] txn:7.797486ms embed:5.473958ms output:2.308811ms size:291366 root:/experiments/ENCSR807BGP/
[2020-04-27T07:46:51Z TRACE embed] begin_txn
[2020-04-27T07:46:51Z TRACE embed] begin_embed
[2020-04-27T07:46:51Z TRACE embed] end_embed
[2020-04-27T07:46:51Z TRACE embed] begin_output
[2020-04-27T07:46:51Z TRACE embed] end_output
[2020-04-27T07:46:51Z TRACE embed] end_txn
[2020-04-27T07:46:51Z INFO  embed] txn:3.073523ms embed:2.254249ms output:807.334µs size:103666 root:/experiments/ENCSR173USI/
```
