# Micro-operation budgets: the kernels held to the performance page

Status: `draft`, 2026-10-24. The quality bar's "micro-benchmarks per module
with budgets" row ([`05-testing/01-quality-bar.md`](../05-testing/01-quality-bar.md)):
each operation stays within its budget
(`02-architecture/09-performance-architecture.md`), with the baselines
checked in. Several crates carry `criterion` benches, no workflow runs
them, and no record says what they measured.

## 1. What is already held, and what is not

Three gates already time the SDK, and between them they leave the kernels
out:

| gate | holds | cannot hold |
|---|---|---|
| `check-app-speed` | the façade's requests against their budgets: a day, a month, the muhurta search, a chart | an operation no façade call isolates: one varga, a rule set, `dasha_at` alone |
| `compare-bench` (callgrind) | each scenario section against itself on the base commit, to 3% | an absolute budget: a section that was always too slow passes |
| `check-ffi-cost` | what a binding pays to cross | the work behind the crossing |

The rows of the performance page's budget table that none of them reaches:

| operation | budget | where it is timed now |
|---|---|---|
| `dasha_at(instant, depth 5)` | 20 µs, zero allocations | `crates/dasha/benches/dasha.rs`, each kernel, unrecorded |
| one varga chart | 5 µs | nowhere |
| all standard vargas | 100 µs | nowhere |
| 900 rules | 2 ms | `crates/rules/benches/rules.rs`, over 597 rules: the shipped set is now 4,396 |
| rectification step (Δt = 1 s) | 1 ms | nowhere |

The full parity chart's 10 ms is held by `check-app-speed`'s `chart`, which
asks the chart and its Vimshottari to depth 3 but no rules. Section 5 says
what closes that gap.

## 2. The harness: the crates' own `criterion` benches

The kernels are timed where they live, by the benches each crate already
has, because a kernel's input (a chart foundation, a recorded chart, a
timeline) is built in the crate and no façade call hands it over.

`criterion` 0.5 writes every bench's samples to
`target/criterion/<id>/new/sample.json` as two arrays, `iters` and `times`
(total nanoseconds for that many iterations). A sample's figure is
`times[i] / iters[i]`. The record keeps, per operation, the count of
samples, their median, fastest and slowest, which is the shared results
schema's `Figure` (`xtask/src/measured.rs`, `03-design/ffi-cost.md` §3)
with a sample as a run. Nothing is parsed from `criterion`'s printed text.
A bench is found by the `full_id` in the `benchmark.json` beside its
samples, never by its directory: `criterion` writes the directory from
the id with `?"/\*<>:|^` replaced and long names cut, so two ids can
share one, and every id this list names holds a colon.

- `cargo xtask measure-micro` runs each budgeted bench in release, one
  crate at a time, never two at once, with `--bench <name> -- <filter>`
  so only the budgeted ids run, and reads their samples back.
- It writes `docs/05-testing/micro/<os>-<arch>.json` with the operating
  system, the architecture, the commit (refused when `-dirty`, as
  `check-ffi-cost` refuses it) and `rustc`'s version.
- A bench id the record needs and `criterion` did not write fails the
  measurement: a renamed bench is never a silently missing row.

## 3. The budgets as a list, at their source

The budget of each operation is written once, in `xtask/src/micro.rs`, as
a list of `(key, crate, bench, id, budget_us, what)` rows. The performance
page's table is the prose of that list, and `check-micro` fails when the
two disagree on any budget, so the page cannot drift from the gate.

| key | bench id | budget |
|---|---|---|
| `dasha-at-<kernel>` | `<kernel>: at(t, 5), per instant`, every kernel the bench times | 20 µs |
| `varga-one` | `varga: D9 chart` | 5 µs |
| `varga-all` | `varga: every catalogued chart` | 100 µs |
| `rules` | `rules/the shipped set over one chart` | 2 ms for each 900 rules, scaled to the set's size |

The rules budget scales because the set grows with every chapter that
ships: the performance page's "900 rules under 2 ms" is a rate, 2.22 µs a
rule, and holding a fixed 2 ms against the shipped set would fail a kernel
that got faster. The bench reads every evaluable rule of `shipped::packs()`
(a new list of the crate's six packs, which the interpret crate's span test
reads too) and gives their count as its `criterion` group's elements, and
the budget is that count times the rate. The 597 rules the old bench read
were the recording engine's corpus, a test fixture, not what ships.

The rectification step is not on the list. Its cost is the sky's: one
candidate is a set of positions and a lagna at an instant, and the only sky
the crate holds is a test's straight line, so a figure timed there would
measure the toy. The step is held where the real sky is, by
`check-app-speed`: a `rectify` request over a full day's window under
Teimeris, against the page's 1 s, with its candidates counted so the
step's 1 ms is printed beside it (§5, step 3).

## 4. The gate

`check-micro` (fast check) renders `05-testing/micro-measured.md` from
the records and fails:

- a figure that breaks the results schema;
- a record missing any key the list names;
- a median past **twice** its budget, the rule `check-app-speed` applies
  and for the same reason: wall time on a runner is noisy, and the
  instruction-count gate is what catches a 3% regression on every pull
  request;
- a dirty commit.

No CI job measures. A record is taken on a maintainer's machine, as the
app-speed and FFI-cost records are, and the page names the machine. The
quality bar's "regression beyond the noise floor" half of the row is the
callgrind gate's, and the row will say so rather than claim it twice.

## 5. Order of work

1. `xtask/src/micro.rs`: the list, the sample reader, `measure-micro`, the
   page and `check-micro`, with a test reading a `sample.json` written by
   hand.
2. The benches the list names and the crates lack: `crates/vargas/benches`
   (one chart and every chart over one foundation), and the rules bench
   over the shipped set, giving its count as the group's elements.
3. A `rectify` request in the app-speed harness: a full day's window over
   Teimeris against 1 s, and its time over the candidates it judged
   against the step's 1 ms.
4. A record from this machine, the performance page's table checked
   against the list, and the quality bar's row.
5. The full parity chart: `check-app-speed`'s `chart` asks the shipped
   rules too, or a `parity-chart` request joins it, so the 10 ms budget
   is held over what it names.
