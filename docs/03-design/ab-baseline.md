# Interleaved A/B against the baseline engine

Status: `implemented`, 2026-10-25: the record is
[`05-testing/ab-measured.md`](../05-testing/ab-measured.md). The quality bar's row "interleaved A/B
against the baseline engine" ([`05-testing/01-quality-bar.md`](../05-testing/01-quality-bar.md)):
the SDK through Node is not slower than the engine it replaces on the same
inputs. The performance page asks the same
(`02-architecture/09-performance-architecture.md`, "Benchmark harness"),
through the Node binding and with a results schema that refuses claims
smaller than their spread.

## 1. What runs, and where it lives

The baseline engine is a separate repository, and the SDK names it nowhere
(the clean-room rule, `CLEAN_ROOM.md`). The golden vectors already cross
that line one way: an exporter kept in the baseline's own repository
writes files the SDK reads (`05-testing/01-golden-vectors.md`, rank 2). The
A/B crosses it the same way:

- **In the SDK:** `bindings/node/bench/ab.mjs`, the interleaving runner, the
  SDK's side of every request, the record, the page and the gate.
- **Outside it:** an *adapter*, a module the runner imports from the path
  `TEISTRO_AB_BASELINE` names, kept in the baseline's repository beside
  its exporter. It is the only code that touches the baseline engine.

The adapter's contract is four exports, all synchronous:

| export | what |
|---|---|
| `version` | the baseline's version string, recorded as it is |
| `chart(input)` | one birth chart of a golden input, with its Vimshottari to depth 3 |
| `day(input)` | the almanac day of a golden input's date at its place |
| `sun(answer)` | the Sun's sidereal longitude in degrees from the adapter's own answer |

The inputs are the golden corpus's (`fixtures/baseline/manifest.json`):
the same 55 births both engines were compared on for accuracy, read by
the runner and handed to both sides, so neither chooses its own case. An
input carries the birth's UT Julian day, the same instant as an ISO-8601
UTC string and its IANA zone (the baseline's birth record takes both),
the place, the clock's offset and the civil date.

## 2. The same work on both sides

A timing is the same work only if both sides computed the same thing.
Before any figure is taken, the runner asks each side once for every
input and holds the two Suns' sidereal longitudes within an arcminute. An
input where they disagree is reported and leaves the timing, so a side
that answered fast because it answered something else (another instant,
another place) is never timed. An arcminute and not the golden tolerance:
measured on c001, `parashari-classical` over Teimeris stands 5.8″ from the
recorded Sun, `nepali-default` 11.4″, the same over the built-in, so the
gap is an ayanamsha model's and not an ephemeris's. Matching that model
is the golden vectors' work (`05-testing/01-golden-vectors.md`); a wrong
instant moves the Sun by 2.5′ an hour, which the arcminute catches.

The SDK side is `parashari-classical` over Teimeris, the modern engine
the performance page's budgets are written for. The baseline reads the
same Swiss Ephemeris files it was exported with. Both run in one Node
process, warmed, after their contexts are open: opening is a separate
figure and is not compared, since the baseline has no context to open.

Each request asks the SDK for what the baseline's answer holds, since
a bare request would time less work:

| request | the baseline's answer holds | the SDK is asked for |
|---|---|---|
| chart | the grahas, the lagna and its houses, drishti, upagrahas, dignities, arudhas, the birth's panchanga and sunrises, and the Vimshottari to depth 3 | the founded chart with `aspects`, `points`, `houses`, `state` and `jaimini`, and the Vimshottari to depth 3 |
| day | the five limbs, the kaalas, choghadiya, horas, moonrise and moonset, and the muhurta yogas | the almanac day, which computes every one of those |

An SDK answer decodes on read and the baseline's comes back built, so the
SDK side reads every part it asked for inside the timing. The baseline
runs a polar day from civil midnight to civil midnight, so the SDK side
does too (`day.polar_day_policy: CIVIL_MIDNIGHT`), and the corpus's polar
birth is timed under the same rule on both sides.

A side may refuse a request, and the runner reports it with the side and
the reason and keeps that input out of that request alone. The gate
accepts that only for a birth the corpus tags `range-start` or
`range-end`. There an ephemeris's data ends: the day of 2 January 1800
needs the Moon and Sun before 0h on 1 January 1800, where the planet
file starts, and the SDK refuses rather than answer from an analytic
fallback, as the golden tests already accept at those births.

## 3. Interleaving

Wall time drifts on a laptop: thermals, other processes, the clock. A
figure taken all of A then all of B carries that drift into the
comparison. So a round runs every input once on A and then once on B,
then the next round B first. Each side's figure per request is the median
over at least 15 rounds, with its fastest and slowest round, per input in
microseconds, the shared results schema (`xtask/src/measured.rs`).

## 4. The record and the gate

`cargo xtask measure-ab` runs the harness when `TEISTRO_AB_BASELINE` is
set, and refuses with a sentence naming the variable when it is not. It
writes `docs/05-testing/ab/<os>-<arch>.json` (the commit, refused when
dirty, the Node version and the baseline's `version`), and
`05-testing/ab-measured.md`. `check-ab` (fast check) renders the page and
fails:

- a figure that breaks the results schema;
- a request where the SDK is slower and the two ranges are apart: a
  claim of "not slower" that `within spread` meets but a measured ratio
  above one does not;
- an input count below the corpus's less the births that request was
  refused for, so a record over a hand-picked few is no claim;
- a refusal of a birth that is not at a range edge.

A CI runner has no baseline, so the record is taken on a maintainer's
machine, as the FFI-cost and micro records are.

## 5. Order of work

1. `ab.mjs`, the adapter contract as a JSDoc type, and a stand-in adapter
   under `bindings/node/test/` that answers the SDK's own values, so the
   runner, the agreement check and the interleaving are tested without
   the baseline.
2. `xtask/src/ab.rs`: `measure-ab`, the page and `check-ab`.
3. The adapter in the baseline's repository (not committed there unless
   its maintainer asks, as the exporter is not), a record from this
   machine, and the quality bar's row.
