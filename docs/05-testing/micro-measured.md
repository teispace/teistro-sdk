# The kernels against their budgets

Status: `generated` by `cargo xtask micro` from `docs/05-testing/micro/*.json`, gated by `check-micro`. Do not edit. Each record is written by `cargo xtask measure-micro`, which runs the crates' own `criterion` benches in release, one crate at a time, and reads back the samples `criterion` saved (`03-design/micro-budgets.md`).

The **budget** is the performance page's (`docs/02-architecture/09-performance-architecture.md`), and the gate fails a median past 2× it, because wall time is noisy and a small regression is the instruction-count gate's to catch. A figure is in microseconds an iteration, and each of `criterion`'s samples is one run: a median from fewer than three, or outside its own range, fails, and so does a record missing an operation or measured from a tree with changes. The rules' budget is a rate, 2.22 µs a rule, times the rules the bench evaluated, since the shipped set grows with every chapter.

Every median is inside 2× its budget.

## `macos-aarch64`

macos aarch64, commit `c049423a8`; rustc 1.99.1 (58cf8f9c3 2026-10-13).

| operation | bench | budget | median | of its budget | samples | fastest | slowest |
|---|---|---:|---:|---:|---:|---:|---:|
| `dasha-at-vimshottari` | `vimshottari: at(t, 5), per instant` | 20.0 | 0.36 | 2% | 100 | 0.34 | 0.81 |
| `dasha-at-ashtottari` | `ashtottari: at(t, 5), per instant` | 20.0 | 0.35 | 2% | 100 | 0.33 | 0.94 |
| `dasha-at-registered` | `a registered row: at(t, 5), per instant` | 20.0 | 0.35 | 2% | 100 | 0.34 | 0.62 |
| `dasha-at-chara` | `chara: at(t, 5), per instant` | 20.0 | 0.22 | 1% | 100 | 0.22 | 0.32 |
| `dasha-at-kalachakra` | `kalachakra: at(t, 5), per instant` | 20.0 | 0.14 | 1% | 100 | 0.13 | 0.19 |
| `varga-one` | `varga: D9 chart` | 5.00 | 0.23 | 5% | 100 | 0.23 | 0.31 |
| `varga-all` | `varga: every catalogued chart` | 100 | 6.41 | 6% | 100 | 6.29 | 9.30 |
| `rules` | `rules/the shipped set over one chart` | 10707 (4818 × 2.22) | 234 | 2% | 100 | 224 | 396 |
