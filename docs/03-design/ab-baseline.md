# Interleaved A/B against the baseline engine

Status: `implemented`, 2026-10-25: the record is
[`05-testing/ab-measured.md`](../05-testing/ab-measured.md). The quality bar's row "interleaved A/B
against the baseline engine" ([`05-testing/01-quality-bar.md`](../05-testing/01-quality-bar.md)):
the SDK through Node is not slower than the engine it replaces on the same
inputs. The performance page asks the same
(`02-architecture/09-performance-architecture.md`, "Benchmark harness"),
through the Node binding and with a results schema that refuses claims
smaller than their spread.

The first record timed two requests, a chart and a day. This page now
specifies the in-depth benchmark: every request the two engines both
answer (§2), throughput on many threads and processes (§4), the tail of
a sustained run (§5), a cold start (§6) and memory (§7), each recorded,
rendered and gated (§8).

## 1. What runs, and where it lives

The baseline engine is a separate repository, and the SDK names it nowhere
(the clean-room rule, `CLEAN_ROOM.md`). The golden vectors already cross
that line one way: an exporter kept in the baseline's own repository
writes files the SDK reads (`05-testing/01-golden-vectors.md`, rank 2). The
A/B crosses it the same way:

- **In the SDK:** `bindings/node/bench/ab.mjs`, the interleaving runner
  and the SDK's side of every request; `bench/ab-inputs.mjs`, what both
  sides are given and how a side is opened in a fresh process or thread;
  `bench/ab-load.mjs`, the load, tail, cold-start and memory runs; and the
  record, the page and the gate.
- **Outside it:** an *adapter*, a module the runner imports from the path
  `TEISTRO_AB_BASELINE` names, kept in the baseline's repository beside
  its exporter. It is the only code that touches the baseline engine.

The adapter's contract is `version`, `sun`, and one export a request (§2),
all synchronous:

| export | what |
|---|---|
| `version` | the baseline's version string, recorded as it is |
| `sun(answer)` | the Sun's sidereal longitude in degrees from the adapter's own `chart` answer |
| one export a request | the request of §2, given one input and answering what the table says it holds |

The inputs are the golden corpus's (`fixtures/baseline/manifest.json`):
the same 55 births both engines were compared on for accuracy, read by
the runner and handed to both sides, so neither chooses its own case. An
input carries the birth's UT Julian day, the same instant as an ISO-8601
UTC string and its IANA zone (the baseline's birth record takes both),
the place, the clock's offset and the civil date. It also carries what a
request needs beyond the birth, chosen by the runner, so that both sides
are given the same:

| field | what | used by |
|---|---|---|
| `partner` | the next birth of the corpus in id order, the last one's the first | `milan` |
| `instants` | ten instants an hour apart from the birth, at its place | `batch` |
| `transitAt` | the birth's instant thirty Julian years on | `gochar` |
| `window` | from the birth to thirty Julian years on | `sadeSati` |

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
the performance page's budgets are written for, and `kp-default` over the
same for `kp`. The baseline reads the same Swiss Ephemeris files it was
exported with. Both run in one Node process, warmed, after their contexts
are open: opening is a separate figure (§6), since the baseline has no
context to open.

Each request asks the SDK for what the baseline's answer holds, since
a bare request would time less work. Where the two engines do not cut
their answers at the same seams, **the SDK is asked for the larger**, so
that any error in the match is against the SDK:

| request | the baseline's answer holds | the SDK is asked for |
|---|---|---|
| `chart` | its foundation (the grahas and their dignities, the lagna and its cusps, every divisional chart, drishti, upagrahas, arudhas, the birth's panchanga and three sunrises) and the Vimshottari to depth 3 | the founded chart with every varga, `aspects`, `points`, `houses`, `state` and `jaimini`, and the Vimshottari to depth 3 |
| `day` | the five limbs, the kaalas, choghadiya, horas, moonrise and moonset, and the muhurta yogas | the almanac day, which computes every one of those |
| `full` | `compute()`: the whole natal chart, every slice, 18 dashas to depth 3, yogas, doshas, Shadbala, Bhava bala, Ashtakavarga, Vimshopaka, Ayurdaya and the marakas | the performance page's parity chart (`app-speed`'s `parity-chart`: every section and varga, every shipped rule, the parity-tier dashas), with the baseline's 18 dashas added to its 16 (22 in all), the avakahada and the rules' longevity |
| `vargas` | its foundation alone, which carries every divisional chart | `chart` without the dasha |
| `yogas` | the foundation, its yogas and its doshas with the panchanga's | every shipped rule set but the readings (yogas, Nabhasa, arishtas, doshas, gandantas) |
| `strength` | the foundation, the Shadbala, the Bhava bala over it and the Ashtakavarga | `shadbala`, `bhavaBala` and `ashtakavarga` |
| `dashas` | the foundation and its 18 dashas, each to depth 3 | the same 18 systems |
| `month` | 30 almanac days from the birth's civil date, each as `day` | `almanac.of` over the same 30 days, each read as `day` reads one |
| `milan` | both births' foundations and doshas, the eight kootas, the nadi and bhakoot doshas, the extended considerations, the Mangal match and the marriage doshas | the birth with its partner as the bride: the Ashta Koota, the ten considerations, the Kuja dosha and the marriage doshas |
| `varsha` | the foundation, then the first year's solar return, its chart, the muntha, the year lord, the Tajika yogas, the sahams and the Mudda dasha | `varsha` through year 1 at the birthplace, with every matter, every saham and the Mudda dasha |
| `kp` | the foundation under Krishnamurti's ayanamsha, the Placidus cusps, every cusp's and planet's sub lord, the significators and the ruling planets | `kp` on `kp-default`, which reads the same ayanamsha and houses |
| `gochar` | the foundation and its Ashtakavarga, the grahas at `transitAt`, their houses from the Moon and lagna, their bindus and the gochar phala with vedha | `ashtakavarga` and `gochar` at `transitAt` with the Ashtakavarga's verdict |
| `sadeSati` | the foundation and Saturn's Sade Sati and dhaiyya periods over `window` | `sadeSati` over `window`, whose default spells are the 4th and the 8th |
| `batch` | `computeAll` over `instants`: a whole `compute()` chart at each | `foundMany` over the same instants, each chart asked for what `full` asks |

An SDK answer decodes on read and the baseline's comes back built, so the
SDK side reads every part it asked for inside the timing. The baseline
runs a polar day from civil midnight to civil midnight, so the SDK side
does too (`day.polar_day_policy: CIVIL_MIDNIGHT`, on both contexts), and
the corpus's polar births are timed under the same rule on both sides.

The baseline's public slices sometimes pay for one part twice: its houses
bundle computes the Shadbala again for the Bhava bala. The adapter reaches
the service that bundle calls and hands it the Shadbala it has, so
`strength` times the least work the baseline can do. It builds the two
engines a chart does not need (the predictive and the applied) on first
use, so that its cold start pays for a chart and no more.

**Dropped:** the muhurta search. The baseline's finder samples the Sun
and the Moon once a day at sunrise and scores four factors (the tithi's
group, the nakshatra, the weekday, whether noon falls in Rahu kaal); its
window assembler then cuts one day's choghadiya. The SDK's search finds
every window in a range where an activity's rules hold, limb by limb and
lagna by lagna, which is another computation, not more of the same one.
Neither can be asked for the other's answer, so no timing of the two is
the same work. The tithi calendar is dropped for the same reason: it
finds occurrences of one tithi, which the SDK answers only inside a
festival rule set.

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

## 4. Load: throughput on many threads and processes

One process answering one request at a time is not how either engine is
deployed. A server answers many at once, on worker threads or on a
process per core. So `chart`, `day` and `full` are driven on 1, 2, 4 and
N at once, N the machine's `os.availableParallelism()`, two ways:

- **threads**: one process, N worker threads, each with its own side (a
  context, or the adapter imported afresh);
- **processes**: N processes, each with its own side, which is how a
  Node server scales past one thread (the cluster module) and the only
  way an engine that cannot run in a worker can scale at all.

Each side runs in processes of its own, never beside the other. Every
worker or process opens its side, warms it on every input, and reports
ready; the parent then names a wall-clock instant a little ahead, and
each one asks the request over the inputs in turn from that instant for a
fixed span, counting the answers it finished inside it. A trial's figure
is all the answers over the span, in requests a second; a count's figure
is the median of at least 3 trials with the fastest and slowest. The
**scaling efficiency** is the count's median over the single one's times
the count, the share of linear throughput kept.

A side that cannot be opened or asked in a worker thread is a finding,
not a failure: the record keeps the side, the mode and the reason, the
page says so, and the processes mode still measures it. The baseline's
native ephemeris binding holds global state (the ephemeris path, the
topocentric observer), so this is expected of it.

## 5. The tail of a sustained run

A median says nothing of the slow answers a user waits for. Each side
answers `chart`, `day` and `full` over the inputs in turn for 30 seconds
on one thread, in a process of its own, timing every answer. The run is
cut into 10 windows of 3 seconds, and each window's 50th, 95th and 99th
percentile is taken; a percentile's figure is the median of the windows'
with the fastest and slowest, per answer in microseconds, so that the
results schema holds it like any other figure. The count of answers is
recorded with it.

## 6. Cold start

A serverless function or a command-line tool pays for its start on every
call. A fresh process opens one side and answers one `chart`: **open** is
from the process's start (`performance.now()` from its time origin) to
the side being ready, the import and the engine or context included, and
**first** is that first chart. The parent times the whole process from
spawn to exit as **process**. Each side is started 9 times, the two
alternating, and each phase's figure is the median with the fastest and
slowest, in milliseconds.

## 7. Memory

A process of its own for each side answers `chart` over the inputs in
turn: 200 to warm, then 10 000 more. Before and after the 10 000 the
process collects (`--expose-gc`) **until two reads agree**: a finaliser
runs between turns of the event loop, so a fixed wait or a single
collection reads a heap still holding what is about to be freed. A read
is `heapUsed` plus `external`, since an SDK answer's bytes live outside
the JavaScript heap. The record keeps the peak resident set over the
process's life, the settled heap after the workload, and the heap's
growth over the 10 000, each the median of 3 processes with the fastest
and slowest. The growth is the measure of a leak; the resident set
cannot be (on macOS the kernel compresses idle pages, so a process
holding 4 000 contexts read 1.0 GB where it had read 2.3), and is
recorded as what a host pays.

## 8. The record and the gate

`cargo xtask measure-ab` runs the harness when `TEISTRO_AB_BASELINE` is
set, and refuses with a sentence naming the variable when it is not. It
writes `docs/05-testing/ab/<os>-<arch>.json` (the commit, refused when
dirty, the Node version, the machine's parallelism and the baseline's
`version`), and `05-testing/ab-measured.md`. `check-ab` (fast check)
renders the page and fails:

- a figure that breaks the results schema, of any kind: a request's
  time, a throughput, a percentile, a cold-start phase, a memory measure;
- a request where the SDK is slower and the two ranges are apart: a
  claim of "not slower" that `within spread` meets but a measured ratio
  above one does not;
- the same of a throughput (the SDK answering measurably fewer a second
  at a count and mode both sides were measured in) and of a percentile;
- an input count below the corpus's less the births that request was
  refused for, so a record over a hand-picked few is no claim;
- a refusal of a birth that is not at a range edge;
- a request, a load count, a percentile, a cold-start phase or a memory
  measure that a side lacks, unless the record says why the side could
  not be measured there (§4);
- fewer rounds, trials, windows, starts or processes than this page says.

The cold start and the memory are recorded and compared, not claimed:
the SDK opens an ephemeris and a catalogue the baseline reads lazily, and
a larger resident set is a cost a host can weigh, not a regression of the
request it answers. The page prints their ratios.

A CI runner has no baseline, so the record is taken on a maintainer's
machine, as the FFI-cost and micro records are, with nothing else
running: the record is taken under the machine's heavy-job lock.

## 9. Order of work

1. `ab.mjs`, the adapter contract as a JSDoc type, and a stand-in adapter
   under `bindings/node/test/` that answers the SDK's own values, so the
   runner, the agreement check and the interleaving are tested without
   the baseline.
2. `xtask/src/ab.rs`: `measure-ab`, the page and `check-ab`.
3. The adapter in the baseline's repository (not committed there unless
   its maintainer asks, as the exporter is not), a record from this
   machine, and the quality bar's row.
4. The in-depth benchmark: this page's §2 requests, §4 to §7, the stand-in
   test of each (the load runner on the SDK side alone), the record's new
   fields and the gate on them, and a record from this machine.
