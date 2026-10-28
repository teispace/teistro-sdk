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
rendered and gated (§8). It then compares what the two answered, field
by field (§10), and holds each side's answer to a reference neither side
computed, where one exists (§11), so the page says which side is right
where they differ.

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
| `facts(request, answer, input)` | the answer as facts in the shared vocabulary of §10 |
| `conventions` | the sky each of `chart`, `kp` and `gochar` is cast in, as §11's Swiss Ephemeris rung reads it |

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
fallback, as the golden tests already accept at those births. It
accepts one more: at a birth tagged `polar-day` or `polar-night`, a
refusal whose reason is a day on which the Sun does not rise. The
profile's solar month-start rule places a sankranti against the day's
arc, so a `month` that crosses a sankranti on a sunless day has no
answer under that rule and says so, with the hint to use another rule.
The baseline answers that month; the SDK's refusal is a finding the
page lists, not a fault.

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

An answer under load must be the answer alone: each `chart` is held to
the Sun the same side answered for that birth while warming, to the bit,
and a trial with one that differs is refused. A side that shares state
between threads answers another sky, or none, when two ask at once, and
throughput over wrong answers is no figure.

A side that cannot be opened or asked in a worker thread is a finding,
not a failure, **for the baseline**: the record keeps the side, the mode
and the reason, the page says so, and the processes mode still measures
it. Its native ephemeris binding holds the Swiss Ephemeris's global state
(the ephemeris path, the sidereal mode, the topocentric observer), so
this is expected of it. The SDK is held to answering in threads, which
its contexts are built for (`02-architecture/09-performance-architecture.md`),
and neither side may go unmeasured in processes.

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
  above one does not. A request the SDK is slower at for a reason the
  page gives (`SLOWER` in `xtask/src/ab.rs`, which the page prints) is
  held to a bound on the ratio instead, and the gate fails an entry that
  is no longer slower. The first in-depth record found one: `yogas`, at
  2.13×, since the shipped Nabhasa set carries about 2 500 of the SDK's
  3 130 classical rules and is not asked for in part;
- the same of a throughput (the SDK answering measurably fewer a second
  at a count and mode both sides were measured in) and of a percentile;
- an input count below the corpus's less the births that request was
  refused for, so a record over a hand-picked few is no claim;
- a refusal the corpus does not excuse (§2): any at a range edge, at a
  polar birth only one for a day on which the Sun does not rise;
- a request, a load count, a percentile, a cold-start phase or a memory
  measure that a side lacks; only the baseline may go unmeasured, only in
  worker threads, and only with the reason recorded (§4);
- fewer rounds, trials, windows, starts or processes than this page says;
- a request without agreement lines under both conventions (§10), a
  rung missing from a request it judges or present on one it does not
  (§11), and an SDK answer missing under `same` at a birth the corpus
  does not excuse;
- an agreement or reference share below its floor, the floors taken from
  the record and held in `xtask/src/ab.rs` as the timings' claim is;
- a cause the page names for a request that the record does not bear
  out (§11).

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
5. Agreement and accuracy: §10 and §11, the adapter's `facts` and
   `conventions`, the record's agreement, reference and unanswered lines,
   the page's accuracy section, the floors and the causes, and a record
   from this machine.

## 10. Agreement: the two answers, field by field

Over the inputs a request was timed on, each side's answer is read into
*facts*, `[family/what, value]` pairs, by `bench/ab-agree.mjs` for the SDK
and the adapter's `facts` for the baseline, and a fact both hold is
compared. The families, their units and their tolerances:

| family | what | compared as | tolerance |
|---|---|---|---|
| `longitude` | a graha, the lagna, a cusp, a transit, a saham | arcseconds, the shorter way round | 1″ |
| `sign`, `nakshatra`, `pada`, `retrograde`, `varga` | a placement and every divisional sign | exact | — |
| `dashaLord`, `dashaStart` | each system's periods to depth 2 (`full`, `dashas`, `batch`), the Vimshottari's (`chart`) and the Mudda's (`varsha`) | lords exact; starts in days | 1 day |
| `event` | a sunrise, a sunset, a limb's end, a solar return, a Sade Sati edge | seconds | 60 s |
| `limb` | the tithi, nakshatra, yoga, karana and vara at sunrise and each that follows | exact | — |
| `shadbala`, `bhavaBala` | the six components and the total, each bhava's | virupas | 1 |
| `bindus` | every Ashtakavarga bindu, and each transit sign's in its graha's own and in the sarva | exact | — |
| `yoga` | a yoga or dosha present or absent, by the pairs of `crates/rules/tests/same-figures.json` (the golden tests' own, which `baseline.rs` reads) | exact | — |
| `koota`, `porutham` | each koota's points and the total; Rajju, Vedha, Mahendra and Stree-deergha | exact | — |
| `office` | the year lord, the five office-bearers, the muntha's sign | exact | — |
| `kpLord` | every cusp's and graha's sign, star, sub and sub-sub lord | exact | — |
| `kpSignificator` | each house's four levels (planets in the occupants' stars, the occupants, planets in the lord's star, the lord), each as a set of the nine grahas; the baseline counts an intercepted sign's lord into the last two levels, the SDK reports the sign and not its lord (crux C154) | exact | — |
| `kpRuler` | the lagna's and the Moon's sign and star lords and the day's lord, each by its slot; the sub lords are `kpLord`'s | exact | — |
| `house` | a transit's house from the Moon | exact | — |
| `drishti` | the grahas each graha aspects in full, counted by sign, as a set of the nine (`full`, `batch`); the baseline's answer holds a graha's fully aspected houses, read here through the grahas in them. The graded aspects, the sphuta drishti and the rashi drishti are in no answer of the SDK's chart | exact | — |
| `jaiminiPair` | the class of life each of the three pairs gives and the class they decide, before Saturn's or Jupiter's shift (BPHS ch. 43 vv. 33 to 40); the SDK's second pair is Saturn and the Moon, the baseline's the lagna and the Moon, so those two are unpaired | exact | — |
| `ayurdaya` | Pindayu, Amsayu and Nisargayu: each graha's years before and after the reductions, the lagna's, and the total | years | 0.01 |
| `maraka` | the marakas by lordship, the lords of the second and the seventh, each a set | exact | — |
| `balarishta` | whether the Moon in the sixth, eighth or twelfth is afflicted, and whether that stands uncancelled; the SDK's figure is `ARISHTA_MOON_IN_DUSTHANA_MALEFIC_ASPECT` (BPHS ch. 9 vv. 3 to 6) | exact | — |
| `gochar` | whether a transit stands in a good house from the Moon, whether a good one is obstructed (Phaladeepika ch. 26 vv. 2 to 8; the SDK obstructs a good transit only, so a bad one's vedha is not compared), and whether its own Ashtakavarga makes it strong | exact | — |
| `kuja` | whether the bride and the groom each carry the Kuja dosha, after any lift the rules asked count | exact | — |
| `marriageDosha` | whether a match carries the Ashta Koota's nadi, bhakoot and gana doshas, lifted or not; the considerations' are `porutham`'s | exact | — |

A continuous family reports the median and the largest difference and the
share within its tolerance; a categorical one the share of exact matches.
Each line names the first three mismatching inputs and its worst, and the
facts one side holds and the other does not (`unpaired`), which are
counted and not compared.

A porutham is compared as the rule's own verdict, before the SDK lifts a
failed Rajju, Vedha, Gana or Rasi by *Kalaprakasika*'s p. 76 exception:
the baseline lifts none, and the printed lists of §11 are the rule's.

Every request is compared twice. **`timed`** is the SDK as it was timed,
`parashari-classical` (`kp-default` for `kp`): geocentric, the mean
ayanamsha basis, the centre of the disc unrefracted for sunrise and the
classical Shadbala readings. **`same`** is the SDK as the baseline casts:
`conformance-baseline`, whose every knob is the recording engine's
measured default (topocentric, the nutated basis, the refracted upper
limb, the engine's Ashtakavarga reductions), with `kp-default` given the
same frame, the baseline's mean node and its ruling planets (the sub
lords counted, the civil weekday's lord: cruxes C150, C151), and the
transit requests (`gochar`, `sadeSati`) cast from the Earth's centre as
the baseline casts a transit. Where a request names readings of its own,
`same` asks the baseline's: `milan` the Kuja dosha from the lagna, the
Moon and Venus in the six houses with the 2nd (C285, C286), and `full`
and `batch` the three spans with Amsayu multiplied, compound enmity,
each reduction taken in turn and Nisargayu's listed years (C104, C303).
What still differs under `same` is method, not convention.
Under `same` a polar day is reckoned by the conformance profile's own
policy, from the nearest sunrise and sunset, given to `kp-default` too,
where `timed` reckons it from civil midnight; at the corpus's polar
births the SDK may then leave a rule that needs a sunrise within a day
and a half unanswered, which the gate accepts there and the page lists.

## 11. Accuracy: a reference neither side computed

Where the two sides differ, a *rung* says which is right. Each has the
corpus's evidence rank (1 a text or an institution's own print, 2 an
established implementation or a source's worked example, 3 another
implementation), and is in `bench/ab-reference.mjs`:

| rung | rank | judges | what |
|---|---:|---|---|
| Swiss Ephemeris | 2 | `chart`, `full`, `batch`, `kp`, `gochar` | every longitude, from pyswisseph 2.10.03 at the side's own declared convention (the adapter's `conventions`, the SDK's settings): a difference is the side's arithmetic, not a convention |
| PyJHora | 3 | `chart`, `full`, `dashas` | the Vimshottari's lords and starts, `fixtures/pyjhora/vimshottari`, its Gregorian-year periods rescaled about the birth to the 365.25-day year both sides count |
| the committee | 1 | `day` | Nepal's *Rashtriya Panchangam* 2082–2083 (`fixtures/official/npns-2082-2083.json`): 22 printed sunrises and sunsets and 8 tithi ends at Kathmandu |
| Raman | 2 | `strength` | *Graha and Bhava Balas*, Example 56: the seven printed Shadbala totals, asked of each side at the example's birth and Raman's ayanamsha |
| Charak | 2 | `varsha` | *A Textbook of Varshaphala*'s worked year: the solar return's printed time and the year lord, the muntha and the five office-bearers |
| *Kalaprakasika* | 1 | `milan` | XIII's Rajju list (p. 75), Mahendra's counts and Stree-deergha's, read from each side's own Moons |

The Swiss Ephemeris rung needs a Python with pyswisseph, named by
`TEISTRO_AB_PYTHON`, and the files, named by `SE_EPHE_PATH`; the runner
refuses without them rather than record no accuracy.

The conformance corpus is not a rung: the baseline engine recorded it.
Four requests have no rung and are **agreement only**, and the page
claims no accuracy for them: `vargas` (a divisional sign is arithmetic on
a longitude the Swiss Ephemeris rung already judges, and no independent
tabulation of the corpus's vargas exists), `yogas` (the rules are read
from texts whose worked examples are the golden tests' and the rule
corpus's own, not an independent answer for these births), `month` (the
committee's days are `day`'s; a month is thirty of them) and `sadeSati`
(no printed edges exist for these births).

Each side is held to a rung under each convention it has, and the SDK
under one more where a rung's source fixes a reading: Raman works
Example 56 by Sripati's readings, and the SDK's `source` row is
`parashari-classical` given them, so that a miss is told from a choice of
reading. A miss is attributed by tier: **the SDK**, when it is farther
than the baseline from the reference under its own convention; **the
engine** underneath, when the Swiss Ephemeris rung finds a longitude off;
or **a convention**, named: the ayanamsha basis, the centre, the node, the
sunrise's limb and refraction, a reading of a rule, or the reference's
own (the committee's tithis are Surya Siddhanta's).

The page has one row a request: its agreement under each convention, the
rung, which side is closer to it, and the dominant cause of what differs.
A cause is held in `xtask/src/ab.rs` with the claim it makes, and the
gate checks the claim on the record: a **convention** cause, that `same`
agrees more than `timed`; a **method** cause, that `same` still differs;
a **baseline** cause, that the SDK is the closer to the rung. Where the
SDK is the farther from a rung, the page's closer column says so for
that rung, whatever the request's dominant cause.
