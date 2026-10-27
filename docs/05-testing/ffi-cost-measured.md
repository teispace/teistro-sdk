# What a binding pays to cross

Status: `generated` by `cargo xtask ffi-cost` from `docs/05-testing/ffi-cost/*.json`, gated by `check-ffi-cost`. Do not edit. Each record is written by `cargo xtask measure-ffi-cost`, which builds the library in release and runs each binding's harness in turn (`03-design/ffi-cost.md`).

Every binding times the same calls. `positions` is asked for 1 instant × 3 bodies and for 366 instants × 10 bodies, first over the library's own test provider and then over a **hosted** provider, one written in the binding's own language (in Rust, behind a vtable) that answers every cell with a fixed longitude, so its work is the columns and nothing it computes. A call includes reading the answer's grid size, so a lazy decode is paid as a consumer pays it, and `decode` is the decoder alone on the grid's bytes. Rust is the floor: **marshalling** is a binding's `call-native` over Rust's, what its request and its decoding add, and **callback** its `call-hosted` over Rust's, what hosting a provider in it adds; the grid's is shared over its 3,660 cells. A difference is printed only where the two figures' fastest-to-slowest ranges are apart, and `within spread` where they meet. A binding's `call-native` and `grid-native` past 1.5× Rust's (the performance page's round trip within 1.5x of the C path) fail the gate, as does a figure from fewer than three runs, or whose median lies outside its own range, fails the gate, and so does a record missing a key for a binding below or measured from a tree with changes.

- **rust**: `crates/ffi/examples/ffi_cost.rs`, the C ABI from Rust
- **node**: `bindings/node/bench/ffi-cost.mjs`, the Node layer
- **python**: `bindings/python/bench/ffi_cost.py`, the Python layer
- **dart**: `bindings/dart/bench/ffi_cost.dart`, the Dart layer
- **java**: `bindings/java/bench/FfiCost.java`, the Java layer

## `macos-aarch64`

macos aarch64, commit `436994439`; rust rustc 1.99.1 (58cf8f9c3 2026-10-13), node v26.12.0, python Python 3.15.0, dart Dart SDK version: 3.13.5 (stable) (Tue Sep 29 01:00:52 2026 -0700) on "macos_arm64", java java 25.0.1 2025-10-21 LTS.

| binding | `call-native` (1 × 3, test provider) | `call-hosted` (1 × 3, hosted provider) | `grid-native` (366 × 10, test provider) | `grid-hosted` (366 × 10, hosted provider) | `decode` (the 366 × 10 blob, decoded alone) | marshalling: `call-native` over Rust | callback: `call-hosted` over Rust | a grid's callback: `grid-hosted` over Rust |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| rust | 34.8 | 30.4 | 1801 | 1145 | 0.06 | — | — | — |
| node | 35.8 | 34.8 | 1757 | 1135 | 0.91 | within spread | 4.38 | within spread |
| python | 50.4 | 51.6 | 1814 | 1238 | 9.72 | 15.6 | 21.2 | within spread |
| dart | 29.4 | 28.4 | 1710 | 1136 | 1.10 | within spread | within spread | within spread |
| java | 31.7 | 34.0 | 1738 | 1189 | 1.37 | within spread | within spread | within spread |

Medians in microseconds a call. The spread of every figure:

| binding | key | runs | calls a run | fastest | slowest |
|---|---|---:|---:|---:|---:|
| rust | call-native | 5 | 2048 | 27.7 | 38.3 |
| rust | call-hosted | 5 | 2048 | 27.2 | 33.9 |
| rust | grid-native | 5 | 32 | 1783 | 2008 |
| rust | grid-hosted | 5 | 64 | 1137 | 1454 |
| rust | decode | 5 | 1048576 | 0.05 | 0.14 |
| node | call-native | 5 | 2048 | 35.4 | 49.5 |
| node | call-hosted | 5 | 2048 | 34.4 | 35.0 |
| node | grid-native | 5 | 32 | 1731 | 1812 |
| node | grid-hosted | 5 | 64 | 1125 | 1152 |
| node | decode | 5 | 65536 | 0.90 | 0.93 |
| python | call-native | 5 | 1024 | 50.2 | 51.5 |
| python | call-hosted | 5 | 1024 | 49.6 | 52.1 |
| python | grid-native | 5 | 32 | 1806 | 1827 |
| python | grid-hosted | 5 | 64 | 1220 | 1301 |
| python | decode | 5 | 8192 | 9.70 | 9.81 |
| dart | call-native | 5 | 2048 | 28.6 | 29.8 |
| dart | call-hosted | 5 | 2048 | 28.1 | 28.7 |
| dart | grid-native | 5 | 32 | 1692 | 2369 |
| dart | grid-hosted | 5 | 64 | 1117 | 1163 |
| dart | decode | 5 | 65536 | 1.08 | 1.14 |
| java | call-native | 5 | 2048 | 30.0 | 36.5 |
| java | call-hosted | 5 | 2048 | 31.6 | 63.9 |
| java | grid-native | 5 | 32 | 1724 | 1819 |
| java | grid-hosted | 5 | 64 | 1180 | 1206 |
| java | decode | 5 | 65536 | 1.33 | 1.37 |
