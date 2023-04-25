# mybee

<img src="https://raw.githubusercontent.com/elbaro/mybee/main/mybee.png" width="128" />

An eBPF profiler for MySQL 8.0.

mybee directly probes on mysqld, read queries and client information from the mysqld memory.  
mybee does not read and parse tcp packets, a work already done by mysqld.

[Example](https://github.com/elbaro/mybee/wiki/Demo)

## Prerequisites

1. rust nightly
2. `rustup toolchain install nightly --component rust-src`
3. `cargo install bpf-linker`

## Run

```bash
cargo xtask build-ebpf
cargo xtask run
# A Prometheus exporter is available at localhost:9000
```
