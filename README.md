# mybee

<img src="https://raw.githubusercontent.com/elbaro/mybee/main/mybee.png" width="128" />

An eBPF profiler for MySQL 8.0.

mybee directly probes on mysqld, read queries and client information from the mysqld memory.

## Prerequisites

1. Install a rust nightly toolchain with the rust-src component: `rustup toolchain install nightly --component rust-src`
2. Install bpf-linker: `cargo install bpf-linker`

## Run

```bash
cargo xtask build-ebpf
cargo xtask run
# A Prometheus exporter is available at localhost:9000
```
