# cdbd

cdbd ("constant database daemon") is a small constant database server: given a
key-value file in a format it understands (like [CDB][]), it will serve that
file via a protocol it speaks (like memcached).

In the past I have used a company-internal tool like cdbd (called "SSTable
Service") to provide precomputed data to production services. The idea is that
it's super easy to start a cdbd instance and get the data where you need. In a
service-oriented architecture, cdbd makes your static data accessible.

## Example

```sh
echo '+3,5:one->Hello\n' | cdbmake f.cdb f.cdb.tmp
cdbd --cdb f.cdb --memcached 11211 &
memccat --servers=localhost:11211 one
```

## Installation

Install with [Cargo][], the Rust package manager, like so:

```sh
cargo install cdbd
```

## Supported constant databases

* [CDB][] (with flag `--cdb FILE`)
* [MTBL][] (with flag `--mtbl FILE`)

## Supported protocols

* [memcached][] (with flag `--memcached [HOST:]PORT`; supports memcached read operations only)

## Work to be done

* Loadtests and benchmarks
* Use Tokio
* Support other databases
  * LMDB
  * SQLite?
  * Berkeley DB?
* Support other protocols
  * Redis (get and mget)
  * HTTP?
* Pull protocols out into their own crates? It would allow others to
  write memcached etc. servers a little more easily, maybe.

## License

Copyright 2017 Leon Barrett

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

[Cargo]: http://doc.crates.io/
[CDB]: http://www.corpit.ru/mjt/tinycdb.html
[MTBL]: https://github.com/farsightsec/mtbl
[memcached]: https://memcached.org/
