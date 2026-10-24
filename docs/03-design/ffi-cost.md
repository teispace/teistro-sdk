# FFI cost: what a binding pays to cross

Status: `implemented`, 2026-10-24. The quality bar's "FFI cost" row
([`05-testing/01-quality-bar.md`](../05-testing/01-quality-bar.md)): the
cost of a callback-based provider and of result marshalling per binding is
measured and published, never guessed. The performance page's budgets are
for a native provider (`02-architecture/09-performance-architecture.md`),
and nothing yet says what a provider written in a binding's own language,
or a binding's decoding of the answer, adds to them.

## 1. What is measured

Every figure comes from one scenario, run the same way through the C
ABI from Rust and through each binding: Node, Python, Dart and Java.
Each context uses the test provider or a provider written in the binding's
own language. Neither depends on a real engine's work, so the measure is
the crossing and not the sky.

| key | what is timed | what it isolates |
|---|---|---|
| `call-native` | `positions`, 1 instant × 3 bodies, test provider | a call's fixed cost: the entry, the request copy, the blob, the decode |
| `call-hosted` | the same, through a provider written in the binding's language answering fixed columns | the same plus one callback crossing out and its columns crossing back |
| `grid-native` | `positions`, 366 instants × 10 bodies, test provider | a call whose answer is 3,660 cells |
| `grid-hosted` | the same through the binding's provider | a callback whose columns are 3,660 cells |
| `decode` | the binding's decoder on the `grid-native` blob, no call | marshalling alone: what reading an answer costs |

A call's callback is `call-hosted − call-native`, and a grid's is
`grid-hosted − grid-native`, printed with its share of the 3,660 cells.
The page prints a difference only where the two figures' ranges do not
overlap. Otherwise it prints `within spread`, as `check-app-speed` already
does for a ratio.

Every hosted provider answers the same fixed columns: a longitude of
123.25° and a speed of 1 in every cell. In Rust it sits behind a
`ProviderVtable` (`Exported`), so its figure is the crossing alone and the
floor every binding is read against. In a binding it is written in the
binding's own language, so its figure is what a consumer pays to host a
provider there, including building the columns. The page prints each
binding's figure less Rust's for the same key. The native provider
computes positions and the hosted one does not, which is why a hosted
grid can be faster than a native one: the two are never subtracted across
providers. Each call reads the answer's grid size, so a decoder that
decodes lazily is paid for as a consumer pays it.

## 2. How a figure is taken

- A run is `N` calls timed together, and `N` is chosen so a run lasts at
  least 50 ms on the machine. A figure is the median over at least five
  runs, with its fastest and slowest run, per call in microseconds.
- Each binding's harness lives in its package's `bench/` directory
  (`ffi-cost.mjs`, `ffi_cost.py`, `ffi_cost.dart`, `FfiCost.java`; the
  Rust one is `crates/ffi/examples/ffi_cost.rs`) and prints one JSON line
  per key. None of them ships in a package. Each is checked where its
  binding's code is: `check-python`'s strict mypy and `check-dart`'s
  analyzer read `bench/`, and `check-java` compiles `FfiCost.java` as the
  shared examples are compiled.
- Java's `decode` reads the grid's blob from a file, because the binding
  hands a consumer no call's raw bytes. The Rust floor writes that blob
  (`target/ffi-cost/grid.tsrb`) from the same request and the same test
  provider, and the harness refuses one that is not a 366 × 10 grid. `cargo xtask measure-ffi-cost` builds the library in release,
  runs every harness one after another, never two at once, and writes
  `docs/05-testing/ffi-cost/<os>-<arch>.json` with the operating system,
  the architecture, the commit and each binding's runtime version.
- Every answer is checked before it is timed: the hosted provider's
  longitudes come back as it sent them, so a figure is never taken over a
  call that failed fast.

## 3. The gate and the shared schema

`check-ffi-cost` (fast check) renders `05-testing/ffi-cost-measured.md`
from the records and fails on a figure that breaks the results schema.
That schema is the quality bar's "results schema" row, and this page is
where it is first shared rather than repeated. A new `xtask/src/measured.rs`
holds the type every timed figure is recorded as and the rules it is held
to. `app_speed.rs` moves onto it in the same change:

- a figure has its runs, its median, its fastest and its slowest run;
- it needs at least `MIN_RUNS` runs, and the median must lie inside its
  own range;
- a difference or a ratio between two figures is a number only when their
  ranges do not overlap, and `within spread` otherwise;
- a record names its machine, operating system, architecture and commit.

No budget binds these figures yet. They are published so that a budget
can later be set from them rather than guessed. The page says so.

## 4. Order of work

1. `measured.rs`, with `app_speed.rs` moved onto it, unchanged in output.
2. The Rust floor and the Node harness, `--measure`, the page and its
   gate, with a record from this machine.
3. Python, Dart and Java harnesses, recorded the same way.
4. The quality bar's row and the results-schema row updated with what is
   enforced.
