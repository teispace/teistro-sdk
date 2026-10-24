# What a binding pays to cross

Status: `generated` by `cargo xtask ffi-cost` from `docs/05-testing/ffi-cost/*.json`, gated by `check-ffi-cost`. Do not edit. Each record is written by `cargo xtask measure-ffi-cost`, which builds the library in release and runs each binding's harness in turn (`03-design/ffi-cost.md`).

Every binding times the same calls. `positions` is asked for 1 instant × 3 bodies and for 366 instants × 10 bodies, first over the library's own test provider and then over a **hosted** provider, one written in the binding's own language (in Rust, behind a vtable) that answers every cell with a fixed longitude, so its work is the columns and nothing it computes. A call includes reading the answer's grid size, so a lazy decode is paid as a consumer pays it, and `decode` is the decoder alone on the grid's bytes. Rust is the floor: **marshalling** is a binding's `call-native` over Rust's, what its request and its decoding add, and **callback** its `call-hosted` over Rust's, what hosting a provider in it adds; the grid's is shared over its 3,660 cells. A difference is printed only where the two figures' fastest-to-slowest ranges are apart, and `within spread` where they meet. No budget binds these figures yet. A figure from fewer than three runs, or whose median lies outside its own range, fails the gate, and so does a record missing a key for a binding below.

- **rust**: `crates/ffi/examples/ffi_cost.rs`, the C ABI from Rust
- **node**: `bindings/node/bench/ffi-cost.mjs`, the Node layer

## `macos-aarch64`

macos aarch64, commit `b63652cab`; rust rustc 1.99.1 (58cf8f9c3 2026-10-13), node v26.12.0.

| binding | `call-native` (1 × 3, test provider) | `call-hosted` (1 × 3, hosted provider) | `grid-native` (366 × 10, test provider) | `grid-hosted` (366 × 10, hosted provider) | `decode` (the 366 × 10 blob, decoded alone) | marshalling: `call-native` over Rust | callback: `call-hosted` over Rust | a grid's callback: `grid-hosted` over Rust |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| rust | 27.7 | 26.2 | 1706 | 1096 | 0.06 | — | — | — |
| node | 35.3 | 35.5 | 1763 | 1165 | 0.89 | 7.64 | 9.25 | 69.2 (0.02 a cell) |

Medians in microseconds a call. The spread of every figure:

| binding | key | runs | calls a run | fastest | slowest |
|---|---|---:|---:|---:|---:|
| rust | call-native | 5 | 2048 | 27.1 | 31.7 |
| rust | call-hosted | 5 | 2048 | 25.9 | 27.0 |
| rust | grid-native | 5 | 32 | 1688 | 1732 |
| rust | grid-hosted | 5 | 64 | 1074 | 1107 |
| rust | decode | 5 | 1048576 | 0.06 | 0.06 |
| node | call-native | 5 | 2048 | 35.1 | 36.1 |
| node | call-hosted | 5 | 2048 | 33.9 | 35.9 |
| node | grid-native | 5 | 32 | 1737 | 1787 |
| node | grid-hosted | 5 | 64 | 1140 | 1189 |
| node | decode | 5 | 65536 | 0.88 | 0.93 |
