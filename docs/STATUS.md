# Status

The living tracker. Read this first in any session; update it before ending
one. It answers four questions: what is done, what is being done now, what
comes next, and what happened in each session.

**Project phase:** Phase 1, Foundation, met its exit criteria on
2026-09-06, and Phase 2, the astronomy layer, met its own on 2026-09-05.
Phase 1's every criterion is held by a gate rather than by a claim
(`07-roadmap/00-roadmap.md`): one scenario through every binding value by
value, 100,236 values identical across two architectures, the conformance
kit against the Teimeris adapter, a swapped latitude and longitude
refused in all three languages, and all four packages installed into
throwaway projects and run before they can be published. The next phases
are 3, the built-in ephemeris, and 4, the chart layer, which may run
beside each other.

Phase 2 met its exit criteria
on 2026-09-05: the accuracy document (`05-testing/ACCURACY.md`,
generated and gated) shows every built `astro` row within its target
against Teimeris, and houses compute for all twenty-two systems and
sunrise for a Nepali place without a provider override. Two of its
deliverables are deferred by decision: the completion's centre,
corrections and equinox steps to Phase 3, where the built-in ephemeris
needs them, and eclipses to v1.x. Phase 1, Foundation, remains open:
`core`, the ports, `time`, `calendar`, the test provider, the Teimeris
adapter, the conformance kit, the `intl` engine and CLI, and the `ffi`
crate with the API description and the C header (its first generator)
exist, as do the Node and Dart generators and bindings with the parity
gate, the determinism lints and the counting allocator, the
cross-architecture hash matrix, the packaging with its release matrix, and
the documentation site with its generated reference, the
instruction-count benchmarks and the conformance repository. Every
deliverable of Phase 1 is built and every exit criterion is met; the
phase's exit review is the next step (`07-roadmap/00-roadmap.md`). Phase 0 exited on 2026-09-05 (decisions
made, four spikes measured, repository live).
**Repository:** https://github.com/teispace/teistro-sdk (public,
Apache-2.0, created 2026-09-04). `main` is protected: pull requests with
the `fast-check` status, linear history. Changes land by branch, pull
request (the `dco` and `fast-check` jobs), rebase merge.
**Last updated:** 2026-09-09, end of the sixty-third session (the Indian
lunisolar calendar designed and its month built. The page Phase 2 named
and nothing wrote is written from the measurement, and its scope is the
finding: **the mark and not dates**, because a lunisolar date's day
repeats one day in forty-four and needs two flags `CalendarDate` does not
carry. The module gives a `LunarModel` beside `SolarModel` and a
three-way `MonthKind`; `panchanga` carries the kind beside the name, and
wiring it made the existing code shorter — one crossing search now serves
the name and the mark where two served one each); before that the sixty-second session (the rule
that decides adhika and kshaya, measured over 12 368 lunar months of a
millennium computed from the Surya Siddhanta's own Sun and Moon. The
corpus records the answer and none of the inputs, so this could not be
arithmetic over recorded numbers as the panchanga's conventions were. The
count of sankrantis in the month is the rule — none adhika, one ordinary,
two kshaya — and it reproduces all fifty-five recorded days. An adhika
month needs no naming rule of its own, which the usual formulation gets
wrong; and kshaya is measured and **not** tested, because the corpus
records none); before that the sixty-first session (the parity
gate over the two new blobs: 594 values compared across three bindings
where it compared 103, all of which had come from `positions`. The
scenario needed a second context, because the gate's own profile is
topocentric and a chart cannot be founded under it — that refusal is now
a compared value of its own, so the three must fail the same way and not
merely succeed the same way. The gate found a gap on its first run: when
`part` and `elapsed` moved into the chart's `cast` last session, no layer
surfaced them, and all three now expose `dayPart`/`dayElapsed`); before
that the sixtieth session (the daily
panchanga at the boundary, built on the layout the pass before it
measured: `ts_panchanga_days` takes a range of days and answers with one
blob whose per-day lists are concatenated across the batch, a `counts`
section saying how many rows are each day's. Three decisions a chart blob
never had to make — a value a day may not have crosses as a presence flag
beside it, two lists answering one question in two halves become one
section with a discriminant, and the seven span lists do **not** share a
shape, because a shape is for sameness of meaning rather than similarity
of structure. Building it found that the shared day section carried two
fields that were never a day's: `part` and `elapsed` belong to an
instant, and the page had said all along that the shared section is
eighteen fields while it held twenty); before that the fifty-ninth session (the shape
of a batch of almanacs, measured, and the step budget it found. The pass
founds 450 days at three latitudes and splits a day's fifteen lists in
two: every fixed one is a **division of an arc** and every ragged one a
**crossing inside the window**, so a polar day whose synthesised arc runs
a fortnight still has twenty-four horas and gains a hundred and
twenty-two karanas. A rectangular blob wastes 78.1% of its rows once one
polar day joins the batch, which settles the layout as ragged. It could
not reach a polar day at all until the horizon scan's bracket cap — a
constant four hundred, described as "a day of ten-minute steps" though
four hundred of them is two and three-quarter days — was sized from the
span it searches); before that the fifty-eighth session (a founded
chart crosses the boundary, as a **batch**, and the ergonomic layers of
three bindings meet it. `ts_chart_found` takes a grid of instants and
answers with one blob of charts founded at one place, every per-chart
section charts outermost as the positions blob puts instants outermost.
The first version took a single instant while `Founder::found` sat unused
beside `found_one` — the dead end the design page's own §3a warns
against. Building the batch corrected that page in four places, and
writing the same rectification example in Node, Dart and Python found
that the Node binding's TypeScript surface declared no chart at all, so
its two byte-section accessors had never run and both double-decoded text
the decoder had already decoded); the fifty-second to fifty-seventh
sessions are in the log below, and before them the fifty-first session (the Python
binding, and the falsification pass that designed it. The pass is the
first to measure the **API description** rather than the corpus, because
a binding was what was being designed and the corpus records charts, not
calling conventions. It counted what each emitter's renaming rule
actually catches and found the three targets are caught in *different
places* — Dart on a member the description calls `Return`, Python on
`from`, a struct field and two parameters, TypeScript on nothing — so one
shared word list would have found neither; it measured the struct sizes
on both targets, because the C header asserts them at compile time and a
`ctypes` declaration is simply trusted; and it found **eighteen
floating-point boundary fields with no unit**, which every binding had
been documenting as bare numbers and which are now named. The binding
itself is `ctypes` over the same shared library the release already
builds, so the package has no runtime dependency and needs no compiler:
branded `float` subclasses instead of `NewType` stubs, `IntEnum`
catalogues whose members are all truthy because `IntEnum` would otherwise
make `Status.OK` falsy, `memoryview` columns numpy wraps without copying,
and `CFUNCTYPE` trampolines that catch everything, because an exception
escaping a `ctypes` callback returns zero and the port reads that as
success. Two gates caught the same defect from opposite directions —
`convert_time` took `TimeScale` where the boundary wants `Scale`, which
mypy reported as a type error and the parity gate as three disagreeing
values — and the parity gate now compares three reports rather than two);
before that the fiftieth session (the
`knob-has-a-reader` determinism lint: three settings knobs that shipped,
resolved and were read by nobody had been found by hand in as many
modules, so the fifth rule of `check-lints` finds them by machine — it
enumerates the knobs from `core` itself, counts readers outside the
settings layer, and treats a stale allowance as a failure so the
inventory of deferred knobs cannot rot; it found fourteen, one of which
`crates/vargas` now reads for real); before that the forty-ninth session
(the canonical form falsified, designed and built: the pass read the source
as well as the corpus and found that the field the whole envelope exists
for — the hash of the value — was the hash of *nothing* on all but one
producer, that `ChartFoundation` and the rest of the chart layer's
values derived no `Serialize` at all so the SDK could not publish a
chart, and that Rust's JSON layer writes `1e-6` where JavaScript's
writes `0.000001`, which is two hashes for one number; all three are
fixed, and `output.precision` — the third shipped, populated, unread
knob in as many modules — has a reader); before that the forty-eighth
session (the houses service falsified, designed and built: the pass found its subject
by looking for the parts of the recorded houses **nothing had read**,
and the answer was the ones that matter for a service — the degeneracy
flag, which disagrees with the SDK's own outcome in *both* directions
and is registry entry 26; the chalit's shift, counted the other way as
well; and `houses.module_overrides`, a knob the root populates on every
shipped profile and nothing had ever asked for); before that the
forty-seventh session (the derived points falsified, designed and
built: the corpus records these
answers, so the pass is the sharpest kind — six of the eight rules
reproduce it **exactly**, the Sree lagna's fraction is settled against
three rivals that are wrong by tens of degrees, Gulika and Mandi were
**derived** rather than proposed by trying all twenty-four candidate
instants (Gulika begins Saturn's eighth and Mandi ends it, which two
"verify" marks in the research page had left open), the three
clock-driven lagnas turned out to be three right rules reading one
wrong clock, bracketed at 1.633 minutes as registry entry 25, and the
Varnada is refused outright, which is crux C22 met in the data); before
that the forty-sixth session (the drishti falsified, designed and
built: the first Phase 4 module the
corpus **cannot check** — it records no aspect at all, which
`cargo xtask aspect` established by searching every key of all 115
fixtures — so the pass measured each system's own invariants, the
systems against each other over 6696 ordered pairs, and the avasthas
the state pass refused; it refused a claim of its own (a mutual full
aspect is not only the seventh: Mars and Saturn make it across the
fourth and tenth), gave `crates/state` a necessary condition that turns
1301 open questions into certain answers, and refused the sphuta
drishti for want of a source, which is crux C45); before that the
forty-fifth session (the planetary state falsified, designed and built: `cargo xtask state`
proposed a rule for every recorded state field and measured it over 837
readings — settling the order of the dignity ladder and the ages'
alternation, and refusing six avasthas outright — `crates/state` is the
page that came out of it, 43 tests, and the default profile turned out
to name a combustion table the SDK had never shipped, which is now two
cited tables and registry entry 23); before that the forty-fourth
session (the varga kernel falsified and built: a divisional chart is a function of one
longitude and the corpus records both, so `cargo xtask vargas` derives
each chart's table from the corpus and holds the design to it — 19 530
placements over two zodiacs, with nothing left over — and `crates/vargas`
is the design built, 41 tests, with the three corrections the measurement
found); before that the forty-third session (the
panchanga day falsified, designed and built: `cargo xtask panchanga`
proposed a rule for each of the recorded daily panchanga's twenty-seven
fields and measured it over all 55 days, three of the rules it falsified
became registry entries 17 to 19 before a line of the module existed,
and `crates/panchanga` was written to the page that came out of it —
59 tests, `core::interval::Interval`, three knobs and four catalogue
kinds); before that the forty-second session (Phase 4
opened: the bhava chalit falsification pass the roadmap asks for first —
the four methods measured against each other over the 55 recorded charts,
and the answer that they are not variants of one thing — the chart
foundation's design page, whose crux is that the day a chart belongs to
is not its civil date, and `crates/chart` with the two parts that page
named as easy to get wrong, measured against all 55 recorded charts);
before that the forty-first session (Phase 1
closed and its "Now" rewritten for the two phases that follow; GitHub
Pages enabled, the registries deferred to the release; Q34
decided and the defect it exposed fixed: the default profile patches the
root rather than `nepali-default`, so it is the texts as read rather than
those plus one engine's centre and one country's calendar; ADR-0024);
before that the fortieth session (the
conformance corpus left this repository for
`teispace/teistro-conformance` v0.1.1 under CC0-1.0, mounted back as a
pinned submodule, and Phase 1 met its exit criteria); before that the
thirty-ninth session (the parts
of the day became a locale's own and `:duration` learnt to break a count
into several units); before that the thirty-eighth session (the
instruction-count benchmarks over a fixed scenario shared with the
determinism matrix, compared against the pull request's own base commit);
before that the thirty-seventh session (the
documentation site: Fumadocs in `site/`, with the API reference generated
from the same description every binding is generated from, held by
`check-ffi` and built by `check-site`); before that the thirty-sixth
session (the
packaging and the release matrix: one version across the repository, five
platforms built into four packages, and every one of them installed into
a throwaway project and run before it can be published); before that the
thirty-fifth session (the Dart binding, the parity gate, the typed intl
accessors in both bindings, the hash matrix's first run, the determinism
lints and the counting allocator); before that the thirty-fourth session (an
ephemeris written in JavaScript answers the SDK through the port's
vtable, refuses a frame so the astronomy layer completes it, and reports
its failures in its own words); before that the thirty-third session (the Node
addon generated from the same description and the ergonomic layer over
it: a context, calendars, time, the locale engine and positions all
answer from JavaScript, with fourteen tests and a strict type-check);
before that the thirty-second session (the Node
binding's generated layers: the TypeScript surface, the catalogue's
tables and the result-blob decoders, with the decoders tested against
blobs the library produced and the types against a consumer at maximum
strictness); before that the thirty-first session (the C ABI
built as `crates/ffi` and the API description toolchain as `crates/idl`
from spike 2: thirty-six entry points, `idl/api.json` and the C header
gated by `check-ffi`, the settings hash made build-independent); before
that the thirtieth to twenty-seventh sessions (Teistro Intl built as
`crates/intl` from spike 4 with its runtime API, date functions and the
baseline migration, the SDK's `i18n/` sources at the root gated by
`check-intl`); before that the twenty-sixth session
(Phase 2's exit review recorded here and in the roadmap), before that the
twenty-fifth session (one request for both bodies of a composite
quantity), the twenty-fourth (visibility and the heliacal phenomena
under three named criteria), the twenty-third session (the
sidereal time moved to the IAU 2006 expression `gst06b`, held to
Teimeris within 0.0012″ strictly inside its 1850 to 2050 window by
`tests/teimeris_sidereal.rs`; F1 measured beyond the window, F6 filed;
`astro-events-and-crossings.md` §4); before that the twenty-second
session (the `CROSSINGS` override with its vtable slot and kit checks,
18 checks against Teimeris); before that the sixteenth session (the
twenty-two house systems in `astro::houses` with the auxiliary points
and the polar policies, within 5e-6° of Teimeris at ten latitudes and
0.0002° of the baseline's 55 charts; `astro-house-systems.md`); before
that the fifteenth session (Phase 2's
astronomy begun: `astro::precession` as a catalogue of four models over
new ERFA ports, `astro::ayanamsha` computing every epoch-defined member
within 1e-7″ of Teimeris over 1044 recorded rows, the completion
completing the sidereal zodiac from the SDK's catalogue, the design
pages `astro-timescales-and-frames.md` and
`astro-ayanamsha-catalogue.md`); before that the fourteenth session (the
national panchanga committee's 2082 and 2083 panchangas obtained from
`npns.gov.np` through the browser and read into
`fixtures/official/npns-2082-2083.json`: the SDK's engine reproduces all
24 sankranti instants within 1.6 minutes and every month start; the
committee's Sun is the text's within 3″, its Moon the text's with four
revolutions fewer on the apsis, its star planets modern positions in
the Lahiri frame, its sunrise modern under a convention of its own; R2
closed for the method, C30 explained, C38 and C39 opened); before that
the thirteenth session (the
`siddhanta` crate completed to the text with the sighra daily motion,
the latitudes and the Lagna, each reproduced against Burgess's worked
1860 computation, and presented behind the ephemeris port as a classical
astronomy that passes the kit; the port's `Astronomy`, `SpeedModel`,
`DistanceUnit` and `DUT1` override; the completion's ordering of the
zodiac shift and the rotation; the planetary hours in `time` under the
`hora_reckoning` knob, decided by the baseline's fixtures; UT1 from a
provider's DUT1).

## How to resume

1. Read this file, then `QUESTIONS.md` (every decision; none is open).
2. The local checkout is the repository root; `cargo xtask check-docs`
   and `cargo deny check` must pass before any commit; commits are
   signed off (`git commit -s`) with Conventional Commits subjects; the
   clean-room policy (`CLEAN_ROOM.md`) is binding. A discrepancy traced
   to the reference engine is measured, filed as an issue in
   `teispace/teimeris` with its reproduction and assigned to the engine's
   maintainer, and entered in `05-testing/02-engine-findings.md` with
   the bound the SDK holds it at meanwhile (the maintainer's rule,
   2026-09-05).
3. **The next task is the mark crossing the boundary.** The calendar
   decides it and `panchanga`'s `LunarMonth` carries it; what it does
   not yet do is reach a consumer. The panchanga blob's `days` section
   gains a `month_kind` column, the three ergonomic layers expose it on
   a day, and `check-parity` compares it — the same path the rest of
   that blob took.

   After it, the remaining Phase 4 work is what the roadmap lists;
   [`calendar-indian-lunisolar.md`](03-design/calendar-indian-lunisolar.md)
   §6 records what a full lunisolar `CalendarSystem` would need and why
   nothing needs it yet.

   What the measurement settles before a line is written: the rule is
   the **count of sankrantis in the lunar month** (none adhika, one
   ordinary, two kshaya); it needs only the Surya Siddhanta's own Sun
   and Moon, so the calendar takes no ephemeris; an adhika month needs
   no naming rule of its own; and `panchanga`'s existing
   `limb::amanta_month` is already right and wants only the mark. What
   it cannot settle is kshaya against an authority, because the corpus
   records none.

   **The page also poses the question the design has to answer.** A
   lunisolar date's day is the tithi at sunrise, which repeats one day
   in forty-four and is skipped one in twenty-six, so
   `(year, month, day)` is not a key and a date needs **two** flags —
   the repeated day and the adhika month. `CalendarDate` carries
   neither. Either it gains them, which crosses the boundary as
   `ts_calendar_date` and reaches three bindings, or `day` means
   something other than the tithi and the calendar says plainly that its
   date is not the one a panchangam prints.

   Phase 4's boundary work is done and gated: `ts_positions`,
   `ts_chart_found` and `ts_panchanga_days` all cross, three bindings
   offer `found`/`foundMany` and `almanac`/`almanacDay` over them, each
   ships eight worked examples, and **`check-parity` compares 594 values
   across the three** where it compared 103 a session ago.

   Note the cross-phase dependency the roadmap records: **Phase 4 cannot
   exit before Phase 3 delivers the completion's centre step**, because
   all 55 recorded charts are topocentric and the examples had to change
   profile to run.

   The JSON Schema emitter waits behind it, and deliberately. The schema
   comes from the API description (`schema-measured.md` settles that:
   not a sample, which cannot tell a count from a whole double nor give
   a string field its member list), and the description holds 27 structs
   — all C-boundary types, none of the chart document's. A schema
   emitted now would describe a fragment. It is worth writing when the
   description holds a whole document. The two blobs now built do not
   change that: they describe wire layouts, not the Rust value tree a
   stored document is.

   After both, the **Indian lunisolar calendar** `panchanga` still needs
   for adhika and kshaya months.

   Every binding now ships **eight worked examples**
   (`bindings/*/example/`, the same eight scenarios in three languages,
   every file run by that binding's gate — the six the fifty-second
   session wrote, plus a rectification pass and a week's panchangam). Writing them was a falsification pass over the three
   ergonomic layers; the nine gaps it found are in `CHANGELOG.md` and in
   this file's session row, and the rule it earned is in
   `02-architecture/07-binding-architecture.md` under "Examples". Two
   things it left behind, both small and both worth doing before the next
   binding:

   - **A body's key is spelled two ways.** `teistro_port_ephemeris::Body::key`
     returns `SUN`, the catalogue's case, while the generated binding
     enums spell the same key `sun` (the emitter lower-cases a variant
     name, and the description carries no key for `Body` at all). A
     refusal therefore says `does not support MARS` to a caller who wrote
     `Body.mars`. `Body::from_key` accepts either case, so nothing is
     broken; what is missing is one spelling in the description.
   - **Refusing early on coverage is each binding's own choice.** The
     port holds that an instant outside the declared span is a per-cell
     `CellStatus::OutOfRange` and not a reason to refuse a batch, and the
     test provider relies on it. All three adapters nonetheless refuse
     the whole call, which is friendlier for a hand-written provider and
     wrong for a mixed grid. Deciding it properly wants the design page
     the port never had for per-cell status.

   The Python binding is **built** (`bindings/python`, 69 tests,
   `cargo xtask check-python`; the parity gate now compares three
   bindings and they agree on all 103 values). Its falsification pass is
   `cargo xtask surface`, held by `check-surface`, and it is the first to
   measure the **API description** rather than the corpus, because a
   binding is what was being designed. What it left behind: per-platform
   wheels, which are a release change and not a binding change, and a
   `numpy` extra, which wants a real workload to decide
   (`03-design/python-binding.md` §14).

   Two of `serial`'s open questions are worth a look too: whether
   `chart` and `panchanga` should seal their own envelopes rather than
   leaving `content_hash` a placeholder, and the dossier, blob and
   layout rows the module catalogue lists.

   Thirteen settings knobs are **deferred with a reason** and printed by
   `cargo xtask check-lints` on every run — `dasha`, `jaimini` and
   `strength`'s belong to Phase 5 modules, `provider.tier` to Phase 3's
   built-in ephemeris, and `calendars.civil_calendar` and `.eras` gain a
   reader when something builds a chart from a settings document alone.
   Each is a small piece of work waiting for its module rather than a
   thing to fix now.

   Phases 1 and 2 are closed; Phase 4 is open and under way, and Phase 3
   may run beside it.

   Two things earlier modules left behind, in the order they are wanted:

   - **The Indian lunisolar calendar** (`calendar-indian-lunisolar.md`,
     a Phase 2 page nothing has written). `panchanga` names the amanta
     month from the solar sign the new moon fell in, which is right; what
     it cannot say is whether the month is adhika or kshaya, and no other
     module can either.
   - **The conformance harness over an adapter**, which Phase 1 deferred.
     Every arithmetic claim the corpus can decide is tested; anything
     needing positions *over time* — a tithi's boundary instant, a
     varga change, the Moon's rise — needs real positions, and so does
     the rank-1 comparison against the eight printed tithi ends of
     Nepal's national panchangam, the only evidence in the project that
     is not another implementation.

   The pattern to keep, earned eight times: **falsify, then design, then
   build.** `cargo xtask chalit`, `panchanga`, `vargas`, `state`,
   `aspect`, `points`, `houses` and `serial` each proposed rules and
   measured them before a line of the module existed, and between them
   they found fourteen differences (registry entries 14 to 26, and the
   bhayat correction) that would otherwise have been written into code
   as facts — and three defects in the SDK's own shape that no
   comparison with the corpus could have shown. The vargas pass is
   the sharpest, because a divisional chart is a function of one
   longitude and the corpus records both: 19 530 placements decided the
   whole kernel outright. The state pass shows the other half of the
   pattern — it **refused** six avasthas rather than guessing.

   The serial pass showed a sixth: **read the source, not only the
   corpus.** Nothing recorded can say whether the SDK fills the fields it
   documents, whether its own values serialise, or whether two bindings
   would write a number the same way. All three were wrong, and all three
   were found by measuring the SDK against itself.

   The houses pass showed a fifth: **when most of a section already has
   a reader, the pass's subject is what does not.** Three of the four
   things it measured had never been compared with anything, and one of
   the three — a boolean the engine records and the SDK answers with
   three cases — turned out to disagree in both directions.

   The points pass showed a fourth: **derive rather than propose, where
   the space of readings is small enough to enumerate.** Two "verify"
   marks in the research page had stood over Gulika for a year; trying
   all twenty-four candidate instants against every recorded value
   settled both in one measurement.

   The aspect pass showed a third thing: **a pass is worth running even
   where the corpus is silent.** It had no recorded answer at all, and
   it still refused a claim the design had made (a mutual full aspect is
   not only the seventh), measured how far apart two systems a reader
   might conflate actually are, and turned 1301 of `state`'s 1953 open
   questions into certain answers by finding a necessary condition where
   no sufficient one exists.

4. What is built, and what runs it:

   | crate | what it is |
   |---|---|
   | `core` | settings and profiles, the catalogue, the exact angle, the envelope |
   | `port-ephemeris` | the provider port, the test provider, the vtable |
   | `astro` | the ERFA ports, precession, the ayanamsha catalogue, 22 house systems, rise and set, crossings, the phenomena, the star table |
   | `siddhanta` | the Surya Siddhanta as a provider |
   | `time`, `port-timezone` | the scales, Delta T, the zones, the ghati and the hora, the local day |
   | `panchanga` | the almanac of one day at one place: the limbs as spans, the periods as divisions of the arcs, the month, the omens and the Moon's and Sun's day |
   | `vargas` | the divisional charts: one evaluator, twenty-one rows, arbitrary D-N, the mixed axis, vargottama and the change search |
   | `state` | what a graha is: the dignity ladder, the three friendships, combustion under two cited orb tables, the ages, the war, the avasthas it can decide and the six it will not |
   | `aspect` | which bodies reach which: the graha drishti, the Jaimini rashi drishti, the conjunction, and the orb engine `tajika` will take |
   | `points` | points that behave like bodies: the five the Sun casts, Gulika and Mandi off the day's eighths, the special lagnas and the Yogi points |
   | `houses` | which house under which reading: the system a module uses, the twelve bhavas with their lords and kinds, and the degeneracy outcome |
   | `serial` | the canonical form: the seal that computes its own hash, a number grammar with no exponent, and the chart document |
   | `calendar` | Gregorian, Julian, mixed, ISO week, Bikram Sambat, the drik and classical solar models |
   | `chart` | **the chart foundation** (`day`, `bhava`, `zodiac`, `foundation`), 37 tests |
   | `intl` | the locale engine, the CLI, the packs |
   | `idl`, `ffi` | the API description and the one C ABI |
   | `ephemeris-kit` | the provider conformance kit |
   | `scenario` | the fixed scenario the hash matrix and the benchmarks share |
   | `test-allocator` | the counting allocator |

   Gates on every push (`fast-check`): `check-docs`, `check-fixtures`,
   `check-catalogue`, `check-calendars`, `check-time`, `check-accuracy`,
   `check-intl`, `check-ffi`, `check-chalit`, `check-panchanga`,
   `check-vargas`, `check-state`, `check-aspect`, `check-points`,
   `check-houses`, `check-serial`, `check-lints`, `check-versions`.
   Needing another toolchain, run by
   hand and in `verify`: `check-c`, `check-node`, `check-dart`,
   `check-parity`, `check-package`, `check-site`. Also: `cargo xtask
   hashes` and `compare-hashes` (the determinism matrix), `bench` and
   `compare-bench` (instruction counts, Linux), `package` and `package
   stage` (what a release ships), `version X` (the one version), `chalit`,
   `panchanga`, `vargas`, `state`, `aspect`, `points`, `houses` and
   `serial` (the eight falsification pages), `accuracy`,
   `calendars bs-fit`, `gen ffi|intl|catalogue|calendars|time`.

   The corpus is a **submodule** at `fixtures/`
   (`teispace/teistro-conformance`, pinned to `v0.1.1`): clone with
   `--recurse-submodules`, or `git submodule update --init`.
   `check-fixtures` refuses a checkout without it.

   The adapters under `adapters/` are outside the workspace and need the
   engines locally (`TEIMERIS_LIB_DIR`, `SWEPH_SRC_DIR`, and
   `TEIMERIS_PROFILE=max`, which is what the recorded tables are taken
   under; `adapters/README.md`).

5. Deferred by the maintainer to the very end: the npm organisation, the
   pub.dev account and the publishing credential. Nothing needs them
   before a release, and all three names were free on 2026-09-07.
   GitHub Pages is enabled with Actions as its source, so `docs`
   publishes on a tag.

6. The spikes are all done and their results are in the pages they
   inform: spike 1 in `05-testing/01-golden-vectors.md`, spike 2 as
   ADR-0007 (`spikes/02-binding-toolchain/`), spike 3 in
   `03-design/ephemeris-port-and-adapters.md`
   (`spikes/03-ephemeris-port/`), spike 4 in
   `03-design/intl-engine-and-packs.md` (`spikes/04-teistro-intl/`).

## Done

- 2026-09-04: In-depth analysis of the baseline engine's backend (the reference and
  minimum bar). Recorded in `01-research/baseline-engine/`.
- 2026-09-04: Surface read of Teimeris (the default native ephemeris and
  the model for rigour and generated bindings).
- 2026-09-04: Web research on competitor feature sets and platform
  technology (Diplomat, UniFFI, ICU4X, MessageFormat 2, docs frameworks,
  WebAssembly, next-intl, slang, VSOP87 and ELP theories, Astronomy
  Engine, Swiss sidereal modes).
- 2026-09-04: Documentation written and revised: 86 pages (vision,
  research, architecture, roadmap, decisions, guidelines, living files).
- 2026-09-04: Twenty-five questions put to the maintainer and all
  decided; fifteen ADRs, fourteen accepted (0007, the binding generator,
  waits for the spike).
- 2026-09-04: The quality bar made binding (`05-testing/01-quality-bar.md`,
  ADR-0015) and landed through pull request #1, which proved the `dco` and
  `fast-check` path; `main` is at two commits.
- 2026-09-04: Open-source scaffolding: Apache-2.0 licence, `NOTICE`,
  `DCO`, Contributor Covenant 2.1, security policy, governance,
  `CODEOWNERS`, issue and pull request templates, RFC process, the Rust
  workspace with the `xtask` crate holding the documentation gate
  (`cargo xtask check-docs`) and the DCO check (`cargo xtask check-dco`),
  and the `fast-check` workflow (format, lint, tests, gates). Repository
  created and configured.
- 2026-09-04 (second session): eight further decisions recorded and
  accepted (ADR-0016 exact classification and periods, 0017 kernel and
  table, 0018 evidence ranks and mark-and-continue, 0019 clean room and
  licence containment, 0020 calculation version and provenance, 0021 the
  reference-accuracy ephemeris path, 0022 the determinism contract and
  the conformance repository, 0023 type safety in every binding); five
  design pages written in Phase 0 because they are retrofit-hostile
  (`03-design/`: exact arithmetic, dasha kernels with 56 systems as rows,
  the varga kernel, strength schemes, the rules engine v2); the
  verification-cruxes register (26 open items); `CLEAN_ROOM.md`,
  `deny.toml` with `cargo deny` in the fast check, library lints, the
  forbidden-terms gate extended; principles 16 to 18; the roadmap,
  quality bar, data model, API conventions, binding, performance,
  calendar, localisation, extensibility, guideline and glossary pages
  revised accordingly; `08-adding-a-dasha-system.md` written.
- 2026-09-04 (third session): spike 1 done. The golden-vector export
  script written in the baseline engine's repository and run: 55 charts
  (48 chosen for zone, latitude, altitude and data-range hostility, 7
  placed by search to the second at classification boundaries in the
  topocentric frame), 115 fixtures under 13 settings profiles, 8.9 MB,
  in `fixtures/baseline/` with a manifest; the corpus's README (layout,
  provenance, the chart set, the profiles, the schema, ten baseline
  conventions for the deliberate-difference registry);
  `fixtures/tolerances.json` (provisional, keyed by field and provider
  class); `cargo xtask check-fixtures` in the fast check;
  `05-testing/01-golden-vectors.md` as the spike's result page; the
  roadmap, testing map, glossary, layout and the varga and dasha design
  pages updated.
- 2026-09-05 (fourth session): spike 2 done. The same slice (a context,
  settings, the ephemeris port as a host callback, one batch call
  returning a tree) built both ways under `spikes/02-binding-toolchain/`:
  option A as a designed C ABI, an extractor over its Rust source, and
  Rust emitters for the C header, the napi glue, TypeScript, the blob
  decoder and Dart, with hand-written ergonomic layers; option B as a
  Diplomat bridge generated into JavaScript (wasm) and Dart. Measured
  in Node and Dart: callbacks, marshalling of trees to depth 5, code
  volume, the typed surface. Decided as option A (ADR-0007, Q2): Diplomat
  0.16 cannot pass a host provider from JavaScript or Dart, marshals
  trees only through per-node accessors (an order of magnitude slower
  than the blob), and serves Node only through wasm. The maintainer's
  mandate of the day (type safe, DRY, clean, no repetition) recorded in
  the coding standards; the spike's emitters share one `common` module
  and the benchmarks one harness per language.
- 2026-09-05 (fifth session): spike 3 done. The ephemeris port built
  under `spikes/03-ephemeris-port/`: the model (frames packed to 32
  bits, columns instants outermost, a status and a source per cell,
  capabilities with content hashes, reserved error codes), the trait
  with positions required and three overrides, the `#[repr(C)]` vtable
  with `struct_size` handshakes and a bit-identical round trip, the IAU
  2006 obliquity and IAU 2000B nutation ported from ERFA (`NOTICE`
  updated), frame completion by policy with stamped steps, a
  thirteen-check conformance kit under one published set of bounds, a
  shared kit runner and timing helper, and two adapters outside the
  workspace: Teimeris (one context behind a mutex, the body-major grid
  transposed) and the Swiss Ephemeris (compiled from `SWEPH_SRC_DIR`,
  one process-wide lock, explicit flags, fallback reported as missing
  data, hashed data, a cross-thread stress test). All three providers
  pass the kit; the two engines give the same numbers on every check;
  the port costs 0.2 % over Teimeris's own call, the vtable 1.8 %, and
  completion 0.16 to 0.32 µs per cell. Finding: the SDK's Delta T fit is
  5 s high by 2025 against the engines' tables, so Phase 1's Delta T is
  a table plus a model. The design page
  `03-design/ephemeris-port-and-adapters.md` written; the architecture,
  roadmap, spikes index and testing pages updated.
- 2026-09-05 (sixth session): spike 4 done. Teistro Intl built under
  `spikes/04-teistro-intl/`: the stable `MessageFormat 2` grammar in
  full (data model, parser with offsets and the data-model checks,
  serialiser with a property-tested round trip), the `i18n/`
  conventions (metadata, namespaces, keys, entity records, source
  order), the engine with the SDK's functions (`:string`, `:integer`,
  `:number`, `:dms`, `:zodiac`, `:entity`, `:list`, `:msg`), ICU4X plural
  rules, numbering systems, fallback chains with provenance and a parse
  cache, the validator with twelve proven gates, the `.tpack` container
  (zero-copy, checksummed, hashed, carrying the locale metadata), typed
  accessor generators for TypeScript and Dart over one model, the CLI
  (`validate`, `build`, `gen`, `render`, `report`), 49 entities and 13
  messages in `en-Latn` and `ne-Deva-NP`, and harnesses proving both
  surfaces compile and reject wrong usages. Findings: the stable syntax
  differs from the draft the architecture quoted; no parameter sidecar
  is needed; entities select on their own gender; Nepali ordinals need
  exact keys; packs keep source text and the Phase 1 container bundles a
  locale's namespaces. The design page `03-design/intl-engine-and-packs.md`
  written; the localisation architecture, design index, spikes index,
  roadmap, adding-a-language guide and glossary updated. Phase 0's exit
  criteria met.
- 2026-09-05 (seventh session): the five Phase 1 foundation design
  pages written in `03-design/`: core types and the catalogue (forty
  kinds, keys and ids with their C packing, the catalogue sources and
  generator, the three rules separating facts from school choices and
  presentation, the quantity newtypes, closed unions per binding, the
  envelope and status codes, registries and limits); settings and
  profiles (the typed knob inventory, profiles as patches over a cited
  root, coherence rules, the canonical form and hash, five shipped
  profiles including `conformance-baseline` for the fixtures); time
  and time zones (scales, Delta T as a table then a model, zone
  resolution with replay-safe metadata, local mean time, the local
  day, ghati-pala as exact integer arithmetic); the arithmetic
  calendars (the fixed day, Reingold and Dershowitz, the mixed
  transition, ISO weeks, exhaustive and differential tests); Bikram
  Sambat (table plus computed extension with the month-start rule
  chosen by measurement, the divergence set as a fixture, eras by
  new-year rules, the source memo). The calendar and data-model
  architecture pages and the design index updated.
- 2026-09-05 (eighth session): `crates/core` built. `catalogue/`: 53
  kinds, 629 members, attributes and citations in YAML (the baseline
  engine's entity data at rank 2 plus the standard facts, each row
  marked), with a README; `cargo xtask gen catalogue` generates one
  enum per kind with stable ids, typed attribute tables, aliases, marks,
  sources, serde forms and resolvers, plus `catalogue.json` and the
  entity skeleton, and `check-catalogue` gates the output in CI. The
  crate: keys and packed ids with suggestions for wrong keys, validated
  quantities with compile-fail proofs (swapped place, wrong time scale,
  bare number, private constructor), `Nas` with property-tested exact
  classification, bounded `Ratio`, the status codes and a small `Error`,
  the provenance envelope, registries for open kinds, limits, and the
  settings module (twenty-six knob sets in thirteen groups, patches,
  five shipped profiles over a cited root, coherence rules, canonical
  JSON and hash). 29 unit tests, 9 doctests, 4 compile-fail tests, a
  criterion benchmark; every budget met except the settings hash (15 µs
  for a 2 KB document against 10 µs per KB, recorded).
- 2026-09-05 (ninth session): `crates/calendar` built from its two
  design pages: the fixed day with its Julian-day relation and weekdays;
  proleptic Gregorian and Julian in astronomical years; the mixed
  calendar with the 1582, 1752 and 1918 transitions and the gap refused;
  the ISO week date; the `CalendarSystem` trait (`date_of`, `fixed_of`,
  month lengths, leap years, conversion) and the shipped calendars by
  key; Bikram Sambat over the baseline engine's table (BS 1856 to 2457;
  official 1970 to 2100 stamped `Tabular`, the rest `Computed`),
  anchored on 1 Baisakh 1970 = 13 April 1913. Tests: every day of −9999
  to 9999 round-trips in each arithmetic calendar and agrees with the
  `calendrical_calculations` oracle; every day of the Bikram Sambat span
  round-trips; the anchors of 2072 and 2081 hold. The catalogue's `era`
  kind gained `COMMON_ERA` and `BEFORE_COMMON_ERA`. The source memo
  `docs/calendars/bikram-sambat.md` opened with what the baseline
  engine's generator established (Surya Siddhanta longitudes at
  Kathmandu with Nepal's offset history and a fitted 0.705 day cutoff
  reproduce 87 % of official month splits and never drift beyond a
  day) and the SDK's engine plan. The maintainer's mandate recorded: the
  SDK must compute Bikram Sambat from first principles for any year the
  way the Nepali panchanga does, so Nepal's panchanga makers can use it.
- 2026-09-05 (tenth session): the Bikram Sambat computation engine.
  `crates/siddhanta`: the Surya Siddhanta as the text prints it
  (Burgess, 1860), every number cited by verse; mean places in exact
  integer arithmetic from the text's own epoch (midnight at Lanka at the
  start of the Kali age); the sine table with the text's interpolation;
  the manda and sighra equations, the four steps, the true daily motion,
  the text's precession, declination and ascensional difference; a bija
  overlay with no shipped set (unsourced); the classical path uses no
  platform mathematics and is bit-identical everywhere (the Sun in 54 ns).
  `crates/astro` seeded with the shared boundary solver; `crates/time`
  seeded with a zone's offset history as a local clock and Nepal's rows
  from tzdb; `core::time` with `UtcOffset`, the `LocalClock` trait and
  local mean time; `core`'s `Divergent` resolution now carries both
  labels. `crates/calendar`: the `SolarModel` trait, the sankranti finder,
  the month-start rules as cited rows (Orissa, Bengal, Tamil, Malabar,
  the almanac day, the shift family, the Dharmasindhu's punya-kala), the
  engine (a year from a model, a clock, a place and a rule), the fit
  report, and the table regenerated for BS 1700 to 2500 by
  `cargo xtask gen calendars` from the official rows
  (`crates/calendar/data/bikram-sambat.json`) and the engine, held by
  `check-calendars` in CI; dates inside the official span report
  `Divergent` where the engine differs. The measurement
  (`cargo xtask calendars bs-fit`): under the text's Sun and Nepal's
  clock the civil-day rule reproduces every official New Year and year
  total and 90.1 % of month lengths, and the per-sankranti analysis
  showed the two ayana sankrantis follow the Dharmasindhu's punya-kala
  convention (Karka by the sunrise-to-sunrise day, Makara by sunset),
  which reproduces 1490 of 1512 month lengths (98.5 %), 116 of 126 years
  exactly, with no drift; the eleven residual boundaries lie within 25
  minutes of the rule's boundary. The source memo, the Bikram Sambat and
  time design pages, the new `siddhanta.md`, the module catalogue, the
  cruxes register (C27 to C31), the changelog and the glossary updated.
- 2026-09-05 (eleventh session): `crates/time` built from
  `03-design/time-and-timezone.md`, with `crates/port-timezone` as the
  zone database's contract. Scales: TT from UT1 and back through Delta
  T, UTC read as UT1 with DUT1 zero and stamped, UTC before 1972 as
  proleptic, TT from UTC through the leap-second table exactly; the
  envelope's time stamp filled from what was applied. Delta T: the IERS
  EOP C01 series (UT1 from 1956 to August 2026, 708 rows at a tenth of a
  year, fetched from the IERS with its provenance) interpolated where
  measured, Espenak and Meeus (2006) either side with the seam offset
  tapered and the end slope trusted for a decade, Morrison and
  Stephenson's (2004) standard errors as the uncertainty before the
  atomic era and a growing one after the table; Stephenson, Morrison and
  Hohenkerk (2016) registered and refused as unsourced (C32). Leap
  seconds: the IANA list (28 rows, expiring 2027-06-28) with a warning
  beyond its word; a civil 23:59:60 accepted only where the table has
  one, folded onto the following midnight. Zones: `ZoneSpec` (IANA, local
  mean time, a fixed offset), the gap and overlap policies from the
  settings, `ZoneResolution` with offset, source, era (current, earlier
  rules, before the zone's first rule, decided against the offsets the
  zone applies in the database's own year, never the clock), the
  database version, the abbreviation, what the policy did and the
  warnings; a stored resolution is itself a clock for replay; the
  embedded database is `jiff`'s bundled tzdb (2026c), never the host's,
  with suggestions for a misspelt zone. The local day from any
  `SolarModel` with the three polar policies; ghati-pala exact on a
  tenth-of-a-millisecond grid in both reckonings, every vipala of a day
  round-tripping. Tests: 55 fixture charts reproduce the baseline's
  instant (to the second; to the minute for its rounded local mean
  time), offset, source, era and warnings, with the five era labels the
  baseline took from its export-time offset named as deliberate
  difference eleven (C33). `cargo xtask gen time` and `check-time`;
  `core::settings` now exports its knob enums; the calendar's solar
  model reports polar days; the Bikram Sambat table regenerated under
  the tzdb-backed clock (same rows, the frame stamp naming the version).
- 2026-09-05 (twelfth session): spike 3's port promoted:
  `crates/port-ephemeris` (the model renamed into the catalogue, the
  rise and set override with the horizon convention as port vocabulary,
  the C vtable, the `.se1` scanner, the analytic test provider);
  `crates/astro` (Delta T and the UT1/TT scales moved in from `time`,
  the IAU routines ported from ERFA 2.0.1 with a provenance table and
  tested against ERFA's reference values, sidereal time and the
  obliquity, frame completion by policy, the boundary solver's
  `first_zero`, the rise and set solver by Meeus's iteration with a scan
  as the safety net); `crates/ephemeris-kit` (fifteen checks; the test
  provider in CI; Teimeris and the Swiss Ephemeris passing by hand);
  `DrikSun` as the calendars' second solar model; the local day's
  convention and `resolve_at_place` for the `SUNRISE` fallback; the
  adapters moved to `adapters/` with a Teimeris rise and set override
  and a `bs-fit` binary. Measured: the geometric sunrise within 0.13 s
  of Teimeris's search, the refracted one within 7.3 s at 64°N (the
  refraction convention, C34) and within 2.5 s of the baseline's
  fixtures below 60° (C35 names three fixtures a day early); modern
  positions reproduce 65 % of the official Bikram Sambat months under
  the shipped rule against the text's 98.5 %; the committee's own
  announcement names the Surya Siddhanta as its method (R1, in part).
  Design pages: `astro-events-and-crossings.md` written; the port,
  time and Bikram Sambat pages and the memo revised.
- 2026-09-05 (thirteenth session): `crates/siddhanta` completed to the
  text: the sighra daily motion (II.50 to 51), the latitudes (I.68 to
  70, II.56 to 58) and the Lagna from the oblique ascensions (III.42 to
  50), with Burgess's worked computation for midnight of 1 January 1860
  at Washington as rank-1 test vectors (the day count, the mean places,
  the precession, the true motions, the latitudes, the rising times and
  the horoscope point all reproduce; his printed Moon anomaly is a
  misprint his own table corrects); `SiddhantaProvider` behind the
  ephemeris port, declaring `Astronomy::Classical`, `SpeedModel::Rule`
  and `DistanceUnit::MeanDistances`, with the text's obliquity,
  ayanamsha and sunrise as overrides, passing the kit (`tests/kit.rs`).
  The port gained those three capability fields and the `DUT1`
  override; the kit gained `override_dut1` (sixteen checks), a
  second-difference continuity check for speeds by rule, informational
  rows for a classical astronomy (the obliquity 2065″ from IAU, the
  sunrise 250 s from hour-angle geometry, the speed rule 0.23° a day
  from the derivative; C36, C37), a skip when a provider refuses a
  horizon convention, and the text's ayanamsha against Burgess's
  20°24′39″; the completion applies the zodiac shift while the columns
  are ecliptic and asks the provider's own frame for apparent
  positions, so the classical provider runs the SDK's rise and set
  solver. `crates/time` gained the planetary hours (`horas`, `hora_at`,
  the `hora_reckoning` knob, proportional by default: the fixtures
  reproduce the baseline's lord for every chart but the three its
  day-early or polar blocks decide) and `ut1_from_utc_with` over a
  provider's DUT1, bounded at 0.9 s. R2 stayed open that session (the
  committee's site serves its yearly panchanga through scripts) and was
  closed the next through the browser. Design pages revised: siddhanta (§3 to §5, §7, §8,
  §10), time (DUT1, horas), the port (capabilities, completion, the
  kit's table with the text's column), settings, the module catalogue,
  the glossary, cruxes C36 and C37, fixtures convention thirteen.
- 2026-09-05 (fourteenth session): the committee's publications read
  as data. The 2082 and 2083 Rashtriya Panchangam PDFs (no text layer;
  read from page images) gave 24 sankranti instants, four rows of
  printed places at sunrise, 22 days of sunrise and sunset and eight
  tithi ends (`fixtures/official/npns-2082-2083.json`, with provenance).
  Against the SDK: every instant within 1.6 minutes and every month
  start by the shipped rule (including a Makara at 03:23 kept on its
  civil day); the Sun the text's within 3″ (a modern Sun is 5.5′ off);
  the Moon the text's with `Bija { moon_apsis: -4 }` within 0.5′ at ten
  printed points, no other Moon bija or the swapped epicycle convention
  fitting as well; the star planets and node modern positions in the
  Lahiri frame (Saturn, Jupiter and the node within 1′ to 11′ of
  Teimeris, Mars, Mercury and Venus within 7′ to 94′; the text's places
  2° to 11° away); sunrise and sunset modern, the committee's 1.8 to
  2.8 minutes later than the almanac's upper-limb convention at rising;
  the printed velantara is the text's equation of time (reproduced
  within 4 seconds). The eleven residual boundaries (C30) are thereby
  the earlier makers' decisions inside their tolerance, not a different
  rule: the text's arc in mean time removes one, the drik arc adds two.
  Tests: `crates/calendar/tests/official.rs`,
  `crates/siddhanta/tests/official.rs`. Memo R1 to R3 revised, cruxes
  C28 and C30 updated, C38 and C39 added, fixtures README `official/`.
- 2026-09-05 (fifteenth session): Phase 2's astronomy begun. ERFA ports
  added with their reference values: the IAU 2006 precession angles and
  matrices (`p06e`, `pfw06`, `fw2m`, `pmat06`, `bp06`, `bi00`), the
  long-term poles and matrices of Vondrák 2011 (`ltpecl`, `ltpequ`,
  `ltp`, `ltpb`) with the paper's own obliquity series (`ltpeps`, a
  microarcsecond at J2000), and the vector primitives.
  `astro::precession`: four models (Vondrák 2011 the default, IAU 2006,
  IAU 1976, Newcomb) with the obliquity each is consistent with; 142 ns
  a matrix. `astro::ayanamsha`: the forty-seven members as definitions
  (epoch and value, frame, anchor), the published construction with
  the fitted-model correction, mean and nutated values, custom linear
  definitions, the twelve anchored members refused by name; 0.58 µs a
  value. The completion completes a sidereal zodiac from the catalogue
  when the provider declares no override. Measured against Teimeris's
  recorded values (`fixtures/teimeris/ayanamsha.json`, 1044 rows from
  the adapter's new `ayanamsha-table` binary): TT-epoch definitions
  within 1e-7″ (bit-identical in most rows), UT-epoch definitions within
  2.1e-4″ (the two Delta T models in antiquity). Design pages written:
  `astro-timescales-and-frames.md` (the models, the completion steps
  built and designed), `astro-ayanamsha-catalogue.md` (every member with
  its source); the port page, module catalogue, fixtures README and
  astro README revised.
- 2026-09-05 (sixteenth session): `astro::houses`, the twenty-two
  catalogued systems as one construction (the ecliptic point of a great
  circle of a pole height meeting the equator at a right ascension) with
  the circles each picks: whole sign, equal (three forms), Vehlow,
  Porphyry, Sripati, Regiomontanus, Campanus, Topocentric, Alcabitius,
  Koch, Placidus (iterated), Meridian, Morinus, Carter, Horizon,
  Krusinski, APC, Pullen's two, Sunshine (Treindl's construction); the
  auxiliary points; the sign-based systems in the zodiac in use; the four
  polar policies with `Outcome::{Defined, Substituted, Clamped}`.
  Measured: within 4.8e-6° of Teimeris over 25 194 cusps and angles at
  ten latitudes from −66° to 80° (`fixtures/teimeris/houses.json`, the
  adapter's new `houses-table` binary; the polar substitutions agree
  row for row), and within 0.00021° of the baseline's 55 charts for all
  twenty-two systems between 1800 and 2200 (0.0033° beyond 2200, where
  the engine behind the baseline uses a long-term sidereal time;
  Sunshine 0.05° where the Sun barely rises). Design page
  `astro-house-systems.md`; the design index, module catalogue, settings
  page, fixtures README, astro README and CHANGELOG revised.

## Decided (all on 2026-09-04 unless dated)

Rust core with a C ABI. Generated bindings with a parity gate, generator
chosen by spike. v1.0 is baseline parity with Western and Hellenistic
designed in. Apache-2.0 open core. Teispace owns all baseline engine content and
the baseline engine will migrate onto the SDK. A built-in analytic ephemeris in three
tiers (`standard` default) ships in v1 as its own phase. The SDK owns the
entire astronomy layer above raw positions; provider overrides
`prefer-native` by default. Calendars at least at the baseline engine's level in v1.
Teistro Intl as the single localisation standard (base locale `en-Latn`,
JSON, `i18n/<locale>/<namespace>.json`), offered to consumers too.
Fumadocs. British spelling. Precision-first `f64`. Two-person team,
public repository. Teimeris updated as needed. Names as recommended.
Binding order Node native, wasm, Dart/Flutter, Python, Rust, Java. DCO and
Conventional Commits. Eclipses and the full star catalogue in v1.x.
Everything we author is Rust, tooling included (`cargo xtask`). The
quality bar (tests, benchmarks, memory and leak checks, size and coverage
gates) is part of "done" for every module (ADR-0015). Second session:
`f64` astronomy with canonical nanoarcsecond angles, exact integer
classification and exact rational dasha spans (ADR-0016); one kernel per
family with systems as cited rows, falsified before code, and a lazy dasha
cursor (ADR-0017); evidence ranks with V/T/S marks and refusal of
unsourced variants (ADR-0018); a clean-room policy, a licence allow list
and adapter containment (ADR-0019); a calculation version and the
extended envelope (ADR-0020); the IAU routines as an ERFA port, a DE file
reader and a DE-refit `reference` tier (ADR-0021); byte identity across
architectures by hash and a separate CC0 conformance repository
(ADR-0022); type safety with generated, documented surfaces in every
binding (ADR-0023).

## Now

Phases 1 and 2 have both met their exit criteria; the phases open are 3,
the built-in ephemeris, and 4, the chart layer, which the roadmap says
may run beside each other. Nothing that was deliverable in a closed phase
is outstanding; what each left behind is listed where it was deferred,
and the session log below is the record of how each was built.

**Phase 4 has nothing waiting on it.** `chart` is the root the rest of
the layer hangs from (`02-architecture/01-module-catalog.md`), and every
module it depends on is built: `core`, `astro`, `time`, `calendar`,
`siddhanta` and the ephemeris port with the test provider and the
Teimeris adapter. It is also the layer the corpus was recorded for: the
115 fixtures carry `foundation`, `positions`, `houses`, `vargas`,
`panchanga`, `panchanga_day` and `dashas`, and none of those sections has
anything to compare against yet. The roadmap asked for one thing before
the code, a falsification pass over the four Bhava-Chalit methods, and it
is done (`03-design/chart-bhava-chalit.md`, by `cargo xtask chalit`, held
by `check-chalit`): the four are not variants of one thing, so a house
carries the method that produced it as a position carries its frame, and
the house service returns the madhya as well as the sandhi. The
foundation's design page is written
(`03-design/chart-foundation.md`) and the two parts it named as easy to
get wrong are built and measured: `crates/chart`'s `day` (the day an
instant belongs to) and `bhava` (the twelve bhavas with their madhya, and
a placement that carries its chalit), against all 55 recorded charts —
1320 boundaries and middles to 1.7e-13°, every one of 495 placements, the
107 shifted grahas right in both readings, and the day, the lagna's
anchor, day-or-night and the ishtakaal on every comparable chart. `chart::zodiac` holds the chart's one
ayanamsha value, which the corpus settles: `sidereal = tropical -
ayanamsha` closes to 1.1e-13° over 550 bodies with one value per chart.
Three findings came out of building it: bhayat and bhabhoga are the
Moon's nakshatra transit and not the day's part, which the design page had
wrong and a test now asserts; the engine's night is `24h` minus the
daylight rather than sunset to sunrise, up to 1.80 minutes out (entry 15);
and the engine applies the nutated ayanamsha, 18.46 arcseconds from the
mean one and 0.0086 from the true, so `conformance-baseline` sets
`ayanamsha_basis = TRUE` (entry 16). `ChartFoundation` and `Founder`
assemble them into the value every module above a chart starts from, with
the birth timing as a field and the whole stamped in an `Envelope`, so
the foundation is complete (`crates/chart`, 37 tests).

The panchanga day is built, and it was falsified before it was designed.
`panchanga_day` was the corpus's largest unread section — twenty-seven
fields a day, none of which says how it was reckoned — so `cargo xtask
panchanga` proposed a rule for every one of them, measured it over all 55
recorded days and wrote `03-design/panchanga-day-conventions.md`, which
`check-panchanga` holds. What held: the window is sunrise to the next
sunrise and it is `time`'s own `LocalDay`; every period is an equal
division of the daylight or the night; the choghadiya is the hora's
weekday walk, over 1320 horas and 880 choghadiya without exception;
Abhijit is the eighth muhurta of the daylight and void on Wednesdays; and
the SDK's catalogue reproduces the engine's own attribute tables member
for member. What it falsified became registry entries 17 to 19 (Brahma
muhurta sized from the following night, a median 10 s; the Moon's rise
and set taken over the civil day, 24 of 108 outside the day's window; the
tithi numbered through the month rather than within its paksha), and one
limb the corpus cannot settle at all — the muhurta yogas, whose tables
match no published one under any rotation.

`crates/panchanga` is the design built (59 tests): `limb` (three crossing
searches for four limbs, the karana's six-degree lattice giving the
tithi's twelve-degree one, over a window widened so the first and last
spans carry their own bounds), `span` (both pairs of bounds, because the
corpus keeps only the clipped one), `period` (one divider on two arcs
serving the eighths, the choghadiya, the horas and the muhurtas),
`month`, `omen` (panchaka and the yogas as intervals, not flags), `sky`
and `almanac` (by date, by instant, or by range, which is the primary
shape). It brought `core::interval::Interval` with the divider every
module above will want, three new knobs (`panchanga.centre`,
`panchanga.moon_events`, `panchanga.muhurta_tables`), four new catalogue
kinds with names in all five locales, and the first reader of
`day.day_boundary`, declared in Phase 1 and unread until now.

`crates/vargas` is the third module of the phase and the sharpest test
the corpus can put to anything. A divisional chart is a function of one
sidereal longitude, and the corpus records the longitude *and* the
answer, so `cargo xtask vargas` does not compare within a tolerance — it
**derives each chart's table from the corpus** and holds the design's
proposed rule to it. The rule survived: 19 530 recorded placements over
93 fixtures and two zodiacs, with nothing left over
(`03-design/varga-tables-measured.md`, held by `check-vargas`). Four of
the fixtures are tropical, which moves every longitude twenty-four
degrees, so the rules are not fitted to one zodiac.

It corrected three things. The spans belong to the **group** and not to
the chart, because D30's odd signs are cut 5, 5, 8, 7, 5 degrees and its
even signs the same widths reversed; `divisions` names the chart and is
not always its part count, D30 being called thirty and cutting a sign
into five; and vargottama is a property of a body rather than of a graha
— the engine never marks a lagna, and on two recorded charts the lagna's
navamsha sign is its rashi sign (registry entry 20). The crate (41 tests)
carries the kernel, a whole chart of a founded moment with the mixed
axis, arbitrary D-N under the cyclic convention, and the varga change
search, which uses a lattice where the parts are equal and bisection for
the one chart where they are not.

`crates/state` is the fourth, and the one where the falsification pass
earned its keep by **refusing**. `cargo xtask state` measured a proposed
rule for every recorded state field over 837 readings of nine grahas on
93 fixtures, and settled two things a reading of the texts would have
got wrong: moolatrikona has to be tried *above* exaltation, because
three grahas have a moolatrikona span inside their exaltation sign; and
the five ages alternate, running forward in an odd sign and backward in
an even one, reading them forward everywhere being wrong on 359 of the
837. It also found what the corpus cannot decide, by exhausting the
hypotheses rather than guessing: the deeptadi below its top three, and
three of the six lajjitadi, are not a function of anything a founded
chart holds — their definitions read "or aspected by". The crate
returns them as undecided and names them on every reading, which is the
honest answer and the one a module above can act on.

Building it turned up something the settings layer had been carrying
unnoticed: the SDK's own **default profile named a combustion table the
SDK did not ship**. `parashari-classical` sets `state.combustion_orbs`
to `SURYA_SIDDHANTA`, cited to the text's own orbs "where it gives
them", and the only table written was `BPHS`. The fix is the citation
read literally — the Surya Siddhanta gives six orbs and nothing inside
them, so `SURYA_SIDDHANTA` ships as those six alone and `BPHS` as the
same six with the deeper orb the corpus brackets. Under the default
profile no body is ever deeply combust, which is registry entry 23 and
is stated in the crate's own documentation rather than hidden. The six
outer orbs are the same numbers `astro`'s heliacal visibility reads, and
a test now holds the two copies together.

`crates/aspect` is the fifth, and the first the corpus cannot check at
all. It records no aspect of any kind, which `cargo xtask aspect`
established rather than assumed by searching every key of all 115
fixture files for a name one could have been recorded under. That
changed what a falsification pass could be, and it turned out to be
worth running anyway.

It **refused a claim the design had made**. The page proposed that two
grahas aspecting each other fully must be in the seventh from each
other; the measurement found 48 of the 1020 sign pairs where they reach
each other fully are not, and named the two configurations. One is
Jupiter with itself across a trine, which is arithmetic rather than
astrology and is why the module's `mutual` takes a pair of *bodies*; the
other is real and occurs on 7 of the 93 charts — **Saturn three signs
after Mars** stands in Mars's fourth and holds Mars in its own tenth,
and both aspects are full.

It also measured how far apart the two drishti systems are, which is the
bhava-chalit finding in another place: over 6696 ordered pairs of bodies
the graha and rashi readings agree on 4053 and each sees relations the
other does not, so a module cannot quietly offer one for the other. And
it found that every one of the corpus's 837 recorded placements is in
its **whole-sign** house whatever chalit its fixture names (registry
entry 24), so a harness comparing "aspects the seventh house" against
this corpus is comparing against a whole-sign reading.

Two things it would not do. The **sphuta drishti** does not ship: the
degree-based value Drik Bala weighs has no construction written down
anywhere in this project, so the module refuses the table by name,
publishes the thirteen values any construction must reproduce, and the
question is registered as crux C45. And a **node's aspect** stays the
root's `NONE`, because nothing in the corpus prefers a reading.

What it gave back was larger than expected. `crates/state` had shipped
three lajjitadi as undecided, saying they waited on `aspect`; the pass
retried every rule the tradition states, with a real drishti, and none
is exact — but **none misses a single recorded reading**, and adding the
aspect clause makes each rule worse rather than better. So the
condition is necessary, the engine applies something narrower, and it is
not a drishti. `state` now answers `Holds::No` with certainty where the
condition fails: **1301 of 1953 open questions closed**, with no aspect
model needed to do it. `Boundaries` moved down into `teistro-core` on
the way, because `aspect` wants the same fact about the same angle.

`crates/points` is the sixth, and the corpus records its answers, which
makes the pass the sharpest kind the project can run. Every input to a
derived point — the Sun, the Moon, the lagna, the sunrise and the time
since it — is recorded beside the answer, so a formula reproduces a
value or it does not.

**Six of the eight rules reproduce it exactly**, to the last bit of a
double, over all 71 fixtures that carry them: the five upagrahas the Sun
casts, and the Yogi point with its Avayogi. The project's own research
page marks every one of those "verify"; that is now verification and not
a hope. The Sree lagna is the lagna advanced by the Moon's nakshatra
fraction **of a circle**, which the pass settled against three rival
readings that are wrong by tens of degrees.

**Gulika and Mandi were derived rather than proposed.** Two "verify"
marks stood over them — where inside Saturn's eighth of the arc the
point is read, and whether the two names are one thing — and the pass
answered both in one measurement by trying all twenty-four candidate
instants against every recorded value: they are two readings of one
portion, and it is **start against end** rather than the start-against-
middle the sources are usually said to divide over. The eighth's own
index the catalogue already carried, measured on all 55 days when
`panchanga` was built; a night birth walks it five weekdays on, which is
the walk the choghadiya take. The rule needs the ascendant at an instant
that is not the birth, and because an ascendant is a sidereal time and a
latitude rather than an ephemeris, the module takes it through a trait
and stays testable without a provider.

The one bracketed difference is a small, sharp finding. The hora, ghati
and pranapada lagnas are one rule at three rates and each is exact on 46
of the 71 — and out on the rest by an amount **proportional to its own
rate**, 0.82° at 30° an hour against 2.04° at 75°. Three rules wrong in
proportion to their rates are three right rules reading one wrong clock,
and the pass confirmed it directly: the three imply the same elapsed
time as each other on every fixture, and against that time all three are
exact. The engine's clock is at most 1.633 minutes from the ishtakaal
recorded beside it (registry entry 25).

**The Varnada is refused.** Every recorded value is a whole sign, which
is worth knowing; the received rule reproduces 40 of 71 and no variant
the pass could build from the same parts does better. That is crux C22 —
five published schools disagree — met in the data rather than argued
about, and the module ships none. So do the catalogue's other 38 point
rows, which have keys and no formulas because the corpus records none of
them.

`crates/houses` is the seventh, and the one where the pass had to find
its own subject. Most of `houses.*` already had a reader: `astro`
compares all twenty-two systems' cusps, `chart` compares the chalit's
madhya, sandhi and every placement, and `cargo xtask chalit` measured
how far the four chalit methods stand apart. So the pass looked for what
**nothing had read**, and the answer turned out to be the parts a
service depends on.

**A boolean cannot say what happened.** The engine records
`is_degenerate`, one bit for "the chosen system had no solution here";
the SDK's `astro::houses` returns an outcome with three cases. Nothing
had ever compared them, and they disagree in *both* directions: the
engine flags two charts under Placidus at 64.15° and 64.84°, **below**
the polar circle of 66.56° where the SDK computes Placidus without
trouble, and leaves one clear at 69.65°, **above** it, where the SDK
cannot compute the system at all. They are not the same quantity read to
different precision — they disagree about which charts are the difficult
ones, and the engine's criterion is not a latitude threshold. Registry
entry 26, and the reason the module reports an outcome and a policy
rather than a flag.

**The shift, counted the other way.** The engine lists the bodies the
chalit moves out of their whole-sign house, and `chart`'s test checks
every body it lists; nothing checked that the SDK lists no *others*. A
rule that shifted one body too many would have passed. Both directions
now hold, on the same 135 bodies over 75 fixtures.

**A knob worse than unread.** `houses.module_overrides` says which
system a named module uses — and the root *populates* it, so all five
shipped profiles carry `kp → PLACIDUS` and nothing had ever asked. Under
the default profile the rest of the chart is whole-sign, so a KP reading
was quietly the wrong chart rather than an error. It is the same shape
of gap as registry entry 23, failing more softly.

The module is therefore a **service and not a second copy** of the
geometry: one place to ask which system a module uses, both readings
with the bodies that differ named, the outcome, and the classifications
`strength` and `rules` will both want — kendra, panapara and apoklima
partitioning the twelve with trikona, dusthana and upachaya cutting
across them, and a house's lord taken from the sign its **middle** falls
in, because under an unequal division a house can begin in one sign and
be centred in another.

The **`knob-has-a-reader` lint** closes the class of defect the last
three modules kept turning up by hand. A settings knob that ships,
resolves and is read by nobody is a bug whether or not anything crashes,
and each of the three failed differently: `state.combustion_orbs`
loudly, with a chart founded on the SDK's own default profile returning
`UNSUPPORTED` (entry 23); `houses.module_overrides` quietly, giving a KP
reading whole-sign houses where every shipped profile says Placidus; and
`output.precision` silently, doing nothing at all. Three in three is a
pattern.

The rule enumerates the knobs from `core` itself — `Settings::knob_paths`,
held to the settings document by its own test, so a group added to the
document is watched without a second list to remember — and counts
readers outside the settings layer, over the source with its whitespace
collapsed, because a chain the formatter breaks across lines is still
one access. It found **fourteen**.

One was real and is fixed: `vargas.unattested_dn`. `Scheme::cyclic`
named a convention it never asked for, and its own design page claimed
the knob chose it. `Scheme::unattested` now matches on the reading and
`Scheme::for_settings` reads the knob, so a convention added to the
catalogue forces a decision here rather than being silently read as the
cyclic one. The other thirteen are **deferred with a reason** at their
own declaration, printed on every run, and an allowance that is no
longer needed is itself a failure — so the inventory cannot rot. Every
one of the three failure modes was proven red before the rule was
trusted, as the other four were.

`crates/serial` is the eighth, and the first whose pass had to read the
**source** rather than the corpus. Nothing recorded can say whether the
SDK fills the fields it documents, whether its own values serialise, or
whether two bindings would write a number the same way — and all three
were wrong.

**The content hash was the hash of nothing.** `Provenance` carries every
field ADR-0020 asks for, and the one the envelope exists for — the hash
of the value — is set to `Hash::of(&[])` by `Provenance::new` as a
placeholder and was replaced by exactly one producer of three. A founded
chart and a daily panchanga both went out claiming a hash of the empty
string. That is a shape problem rather than a bug in a producer: the one
field that cannot be filled until the value exists is the one everybody
forgets, so `Sealed::new` is now the only constructor and it computes
the hash.

**The chart layer could not be serialised.** `ChartFoundation` — which
every other Phase 4 value is computed from — along with `Bhavas`,
`ChartDay`, `ChartZodiac`, `GrahaPosition` and `Placement` derived no
`Serialize` at all. The SDK could not publish a chart. They do now, and
`crates/serial/tests/document.rs` is the first thing that puts every
Phase 4 value in one place.

**Two bindings would have disagreed about a number.** Rust's JSON layer
writes `1e-6` where JavaScript's writes `0.000001`: the same number,
two byte strings, two hashes. So the canonical form's number grammar is
now explicit and has **no exponent at all** — a decimal to twelve
places, trailing zeros trimmed, no negative nought — and lives in `core`
beside `content_hash`, because there is one canonical form in the SDK
and not two. A binding implementing that grammar needs no float printer
of its own.

And `output.precision` has a reader. It governs the **rendering** and
not the hash — a hash that moved with a display setting would be a
worse cache key, and the settings hash already tells two precisions
apart. That is the third shipped, populated, unread knob found in as
many modules, which is why the next task is a gate for them.

**Phase 3 has nothing waiting on it either**, and is the larger piece of
numerical work: `tools/ephemgen`, VSOP87, ELP/MPP02, the fitted Pluto,
three analytic tiers with size budgets, and the completion's centre,
corrections and equinox steps, which Phase 2 deferred to it. Its exit —
"a full chart computes with nothing but the SDK installed" — needs
Phase 4 to have happened for "a full chart" to mean anything.

Carried forward, each from the phase that deferred it:

- Phase 2's visibility follow-ups: a photometric criterion, the stars'
  heliacal search, the Moon's first crescent
  (`03-design/astro-planetary-phenomena.md` §10); a settable atmosphere
  for the rise and set solver (C34); cusp speeds and house positions,
  which Phase 4 is what needs them.
- Phase 1's deferrals: a Flutter plugin carrying the library into an
  Android or iOS build (with the mobile targets, v1.x); a musl row in the
  platform table; runners per binding emitting the conformance report
  schema; versioned and Nepali documentation
  (`06-cicd/05-docs-deploy.md`).
- Teistro Intl's remainder: the composite provider's precedence with the
  bindings, abbreviated month names for Nepali, the `zone` option, rich
  renderers per binding (`03-design/intl-engine-and-packs.md` §13).
- The memo's R3: a third source for the calendar, the committee's earlier
  years not being online (`calendars/bikram-sambat.md`).

**The registries are deliberately last.** GitHub Pages is enabled for the
repository with Actions as its source (2026-09-07), so `docs` can publish
on a tag. The npm organisation and the pub.dev account are not created
and the publishing credential is not configured; the maintainer defers
them to the release itself, and nothing needs them before it. The names
are still free: the `@teistro` scope and `@teistro/sdk` on npm, `teistro`
on pub.dev (checked 2026-09-07).

## Next

1. Phase 2's astronomy as above (it is "Now"); a settable atmosphere
   for the rise and set solver (C34); the classical provider's Lagna and
   planetary hours exposed through the chart layer when it exists.
2. The bindings' remaining work. Built: the Dart binding from the same
   description with its own provider and finaliser, the typed intl
   accessors in both, the build handshake, the parity gate over 103
   values, and the packaging — five platforms, four packages, one
   version, each installed into a throwaway project and run by
   `check-package` before it can be published. Left: rich renderers per
   binding, which wait on a serialisation of a rendered message's parts
   that both bindings can read; a musl row in the platform table; a
   Flutter plugin that carries the library into an Android or iOS build,
   which belongs with the mobile targets; the wasm binding from the same
   description. The **Python binding is built** (`bindings/python`), so
   the parity gate now compares three reports rather than two and the
   packaging gate installs four packages rather than three; per-platform
   wheels remain, and are a change to the release matrix rather than to
   the binding.
3. Spike 3's remaining consequences: the kit's corpus checks (positions
   against fixtures per tier) and the `sdk-only` cross-provider
   byte-identity check; the Teimeris adapter as the Teimeris package's
   own crate.
4. Spike 4's consequences in Phase 1 are built but for three: day-period
   ranges a locale states for itself (the four parts are the same ranges
   everywhere today), the `zone` option on the date functions (it waits
   on zoned instants crossing the port), and the composite provider's
   precedence once a binding loads packs from several places.
5. Q24: conduct and security mailboxes on the Teispace domain.
6. Before Phase 1 exits: create `teispace/teistro-conformance` (CC0-1.0)
   and move `fixtures/` into it as a submodule (ADR-0022); the
   maintainer creates the repository.
7. Close the cruxes that block Phase 5 (C6 year length per system, C1,
   C2, C3, C8) by reading the texts; tradition reviewers as they appear.
8. The rest of Phase 1's test-only infrastructure: instruction-count
   benchmarks (`iai-callgrind`, which needs Linux, so they belong to the
   nightly matrix rather than a laptop), and the docs site skeleton with
   the generated reference.
9. A second baseline export (the same script, more sections) for the
   seventeen other dasha systems, aspects, yogas and doshas, strengths,
   Ashtakavarga, the Jaimini slice, KP and milan, once the design pages
   say what each fixture must carry; and the harness itself in Phase 1
   (`05-testing/01-golden-vectors.md`).

## Session log

| date | what happened |
|---|---|
| 2026-09-04 | The baseline engine analysis, Teimeris survey, competitive and platform research, docs written. Twenty-three questions compiled and decided by the maintainer the same day. Architecture revised for the astronomy layer, the built-in ephemeris and Teistro Intl; roadmap restructured into ten phases; governance and scaffolding written; the tooling made Rust-only (`xtask`) before the founding commit was finalised; repository `teispace/teistro-sdk` created public with the docs as the first commit and `main` protected. Next: spike 1, the golden-vector export from the baseline engine. |
| 2026-09-04 (second session) | A review of the team's earlier internal planning notes for the same SDK, now retired; everything worth keeping was absorbed into this repository in its own words and the notes are not referenced. Eight decisions (Q26 to Q33, ADR-0016 to ADR-0023), five falsified kernel and arithmetic designs, the cruxes register, the clean-room policy and dependency allow list, library lints, and the corresponding revisions across the architecture, quality bar, roadmap and guidelines. The maintainer added the type-safety mandate (Q33). Next: spike 1 unchanged. |
| 2026-09-04 (third session) | Spike 1 done: the export script written beside the baseline engine and run; 55 charts (48 chosen adversarially, 7 placed by search at classification boundaries in the topocentric frame), 115 fixtures under 13 settings profiles in `fixtures/baseline/`; the fixtures README with the schema and ten baseline conventions for the deliberate-difference registry; the provisional central tolerance file; `cargo xtask check-fixtures` in the fast check; `05-testing/01-golden-vectors.md` as the result page. Findings: the natal panchanga is topocentric while the daily one is geocentric; local mean time is rounded to the minute; Placidus above the polar circle is not flagged degenerate. Next: spike 2, the binding toolchain. |
| 2026-09-05 (fourth session) | Spike 2 done and decided: option A (ADR-0007). The slice, a designed C ABI with a result blob, a Rust extractor and five emitters sharing one rules module, generated and hand-written Node and Dart layers, a Diplomat bridge with its JavaScript (wasm) and Dart outputs, one benchmark harness per language, and four result files under `spikes/02-binding-toolchain/`. Findings: Diplomat 0.16 refuses host callbacks in JavaScript and Dart; a C-ABI callback costs 0.5 µs into JavaScript and 0.1 µs into Dart; a depth-3 tree marshals in 6 µs as a blob against 179 µs as accessors; Diplomat's Dart output emitted the keyword `true` as an enum member. The maintainer's mandate (type safe, DRY, clean, no repetition) recorded. Next: spike 3, the ephemeris port. |
| 2026-09-05 (fifth session) | Spike 3 done: the ephemeris port under `spikes/03-ephemeris-port/`, a port crate in the workspace (model, trait, C vtable, ERFA-ported obliquity and nutation, frame completion by policy, a thirteen-check kit, runner, timing helper, `.se1` scanner, the spike-2 test provider behind the port) and two standalone adapters outside it (Teimeris; the Swiss Ephemeris compiled from `SWEPH_SRC_DIR` under the containment rules, with the cross-thread stress test). Three providers pass the same kit; the engines agree to every printed digit; the port costs 0.2 % over Teimeris's own call and 1.8 % through the vtable. Findings: the SDK's Delta T fit is 5 s stale by 2025 (Phase 1 uses a table plus a model); Teimeris's grid is body-major and its ayanamsha call takes only the no-nutation switch; the mean ayanamsha is the override to expose. The design page written; `NOTICE` gains ERFA. Next: spike 4, Teistro Intl. |
| 2026-09-05 (sixth session) | Spike 4 done: Teistro Intl under `spikes/04-teistro-intl/`, the stable `MessageFormat 2` grammar with the SDK's functions and ICU4X plurals, the `i18n/` conventions on 49 entities and 13 messages in English and Nepali, a validator with twelve proven gates, the `.tpack` container, typed accessors for TypeScript and Dart with harnesses that reject wrong usages, and the CLI. Measured: renders 0.5 to 2.7 µs, a 6 KB pack verified in 1.4 µs, a lookup in 0.46 µs. Findings: stable syntax over the draft, no parameter sidecar, entities select on their own gender, exact ordinal keys for Nepali, source text in packs with a locale bundle to come. The design page written; Phase 0 exited. Next: the Phase 1 design pages (core types and catalogue first). |
| 2026-09-05 (seventh session) | The five Phase 1 foundation design pages written in `03-design/` (core types and the catalogue, settings and profiles, time and time zones, the arithmetic calendars, Bikram Sambat), each following the ten-section template with a data model, algorithms, an API, errors, a budget, tests, localisation and open questions; the architecture pages they settle and the design index updated. Decisions recorded: the lagna is a point, not a graha; school-dependent values are kernel rows, never catalogue attributes; only the resolved settings are hashed; Delta T is a table then a model; `Resolution` gains `Defined`; Bikram Sambat's month-start rule is chosen by measurement against the official table. Next: `crates/core`. |
| 2026-09-05 (eighth session) | `crates/core` built from its design page: the catalogue as YAML sources (53 kinds, 629 members, cited and marked) with a generator and a CI gate; keys, ids and suggestions; validated quantities with compile-fail proofs; the exact angle with a property-tested partition of the circle; bounded rationals; status codes and a small error; the provenance envelope; registries and limits; settings with thirteen knob groups, patches, five shipped profiles over a cited root, coherence rules and a canonical hash. Benchmarked: key resolution 40 ns, classification 1.7 ns, profile resolution 1.9 µs, settings hash 15 µs. Next: `crates/time` and `crates/calendar`. |
| 2026-09-05 (eleventh session) | `crates/time` and `crates/port-timezone` built: scales and their stamps, Delta T as the IERS series (1956 to August 2026, fetched with provenance) then Espenak and Meeus with Morrison and Stephenson's uncertainties, the IANA leap seconds with folding and expiry, civil time, zone resolution over the embedded tzdb (2026c) with replay-safe metadata under the daylight-saving policies and an era decided without a clock, local mean time, the local day with the polar policies, ghati-pala exact on a tenth-of-a-millisecond grid; `gen time` and `check-time`. The 55 fixture charts reproduce the baseline's instant and metadata; five era labels differ by design (C33). Findings: the IERS carries UT1 only from 1956; Stephenson, Morrison and Hohenkerk's coefficients are not in hand (C32); an `f64` Julian day resolves fifty microseconds, so ghati-pala snaps to a hundred. Next: the port promotion with the drik solar model and the rise and set solver. |
| 2026-09-05 (twelfth session) | Spike 3's port promoted into `crates/port-ephemeris` (with the rise and set override), `crates/astro` (Delta T moved in from `time`; the ERFA ports with a provenance table; sidereal time and the obliquity; frame completion; the rise and set solver) and `crates/ephemeris-kit` (fifteen checks); `DrikSun` for the calendars; the local day's convention and the `SUNRISE` fallback; the adapters under `adapters/` with a Teimeris rise and set override and a `bs-fit` binary. Measured: 0.13 s against Teimeris's geometric sunrise, 7.3 s refracted at 64°N (C34), 2.5 s against the fixtures below 60° (C35); modern positions 65 % of the official Bikram Sambat months against the text's 98.5 %; the committee names the Surya Siddhanta as its method. Next: the `siddhanta` verses and provider adapter, planetary hours, DUT1. |
| 2026-09-05 (thirteenth session) | `crates/siddhanta` completed to the text (the sighra daily motion, the latitudes, the Lagna) against Burgess's 1860 worked computation, and presented behind the ephemeris port as a classical astronomy (`SiddhantaProvider`) that passes the kit; the port's `Astronomy`, `SpeedModel`, `DistanceUnit` and `DUT1` override; the kit's sixteenth check, its second-difference continuity and its informational rows for a classical provider (C36, C37); the completion's zodiac-then-rotation order; the planetary hours in `time` under `hora_reckoning` (proportional, as 52 of 55 fixtures decide and the other three cannot) and UT1 from a provider's DUT1. Next: Phase 2's astronomy pages, the ayanamsha catalogue, houses, crossings and stations. |
| 2026-09-05 (fourteenth session) | The national panchanga committee's 2082 and 2083 panchangas fetched from `npns.gov.np` through the browser and read into `fixtures/official/npns-2082-2083.json` (24 sankranti instants, printed places, sunrise and sunset, tithi ends). The SDK's engine reproduces every instant within 1.6 minutes and every month start; the committee's Sun is the text's within 3″, its Moon the text's with four revolutions fewer on the apsis, its star planets modern positions in the Lahiri frame, its sunrise modern under its own convention. R2 closed for the method; C30 explained as the earlier makers' decisions; C38 and C39 opened. Next: Phase 2's astronomy pages. |
| 2026-09-05 (fifteenth session) | Phase 2's astronomy begun: new ERFA ports (IAU 2006 precession, Vondrák 2011 long-term poles and matrices, the vector primitives) with their reference values; `astro::precession` as four models with consistent obliquities; `astro::ayanamsha` computing every epoch-defined member from its published definition with the fitted-model correction, mean or nutated, custom definitions, the anchored members refused by name; the completion completing the sidereal zodiac from the SDK's catalogue. Measured: TT-epoch definitions within 1e-7″ of Teimeris and UT-epoch ones within 2.1e-4″ over 1044 recorded rows; 142 ns a precession matrix, 0.58 µs an ayanamsha. Design pages `astro-timescales-and-frames.md` and `astro-ayanamsha-catalogue.md`. Next: houses, crossings and stations, the star table. |
| 2026-09-06 (thirty-fourth session) | An ephemeris written in JavaScript answers the SDK. `bindings/node/native/src/provider.rs` is the port adapter, hand-written as the architecture says a port adapter is (every binding wraps its own callback mechanism), and small because the port carries the machinery: a Rust `EphemerisProvider` becomes a vtable through `Exported`, so the adapter only has to be that provider. A provider is an object with a `name`, the `bodies` it answers and one `positions` callback, asked once for a whole grid and never in a loop; everything else has a default. Four things it settles: answering with nothing means "not in that frame", so the SDK asks again in the provider's native frame and completes the rest (an equatorial engine gets `positions:NATIVE, delta-t:SDK, obliquity:SDK, rotate-equatorial-to-ecliptic:SDK` and the ecliptic longitude for free); the port's own `validate` runs before the callback, so a body it did not declare is refused by name; only a code crosses the C boundary, so the adapter keeps the sentence and the layer reports it (`the ephemeris provider threw: no data for that instant`); and the environment is lent for the length of a call and taken back after, so a callback that escaped finds nothing to call into. The generated class now holds the host for as long as the handle lives and brackets every call, which the emitter renders when a constructor takes a vtable; the seam it delegates to is `crate::provider::{ProviderInfo, Host, parts}`, one hand-written module per binding. Four more Node tests (thirteen in that file); the strict type-check gained the provider's shape, where a nameless provider, a body by its id and a column of strings are compile errors. |
| 2026-09-06 (thirty-fifth session) | The counting allocator, the second of Phase 1's test-only infrastructure: `crates/test-allocator` is a global allocator that counts what the calling thread allocates while a measurement runs, per thread so tests in parallel do not see each other's, with `const` thread-local counters so the allocator never allocates. What it pins, on the fast check: a completion allocates **13 times for a grid of 10 cells and 13 for 1000** — the columns are allocated once each, whatever the grid — while the bytes go from 642 to 58,062; Delta T, the obliquity and the nutation allocate nothing; a date in any shipped calendar allocates nothing to read or write; a render parses once and the second render of the same key allocates 58 times against the first's 126. It found something on its first run: reading a Bikram Sambat date allocated twice per call, for the authority and the edition of the table it came from, which are static text. `CalendarResolution` now borrows them (`Cow<'static, str>`, so a consumer's own calendar can still own its), and the path every chart takes for every date it shows allocates nothing. |
| 2026-09-06 (thirty-fifth session) | The determinism lints, the first of Phase 1's test-only infrastructure: `cargo xtask check-lints`, on the fast check. Four rules the compiler does not check, each read off the source. **Unordered iteration**: no `HashMap` or `HashSet` in a computation crate unless the file's own header says why it is safe there, which two do (the locale engine's parse cache, read by key and never iterated; the grammar checker's membership sets, where `insert` returning false is a duplicate). **Ambient input**: no read of the clock, the environment or the process in a computation crate outside its tests, and there is none. **The unsafe inventory**: exactly three crates may downgrade the workspace's `forbid` on unsafe code — the port, the boundary and the addon — so a manifest quietly changing its mind is caught where the compiler would say nothing. **Exact classification**: the six classification functions of `core::angle` are `const fn`, which in stable Rust cannot compute in floating point, which is ADR-0016's rule enforced rather than trusted. Every rule was proven red before it was trusted. The gate prints its allowances rather than hiding them, so what is permitted is an inventory of ten lines that a reader can argue with. |
| 2026-09-06 (thirty-fifth session) | **Every engine finding is closed.** The maintainer fixed all six upstream in `eba52e6`: four behind the engine's `MAX` profile and two unconditionally. The SDK's adapter now takes the profile from `$TEIMERIS_PROFILE`, every recorded table says which profile it was taken under, and the five Teimeris fixtures were re-recorded under `max`. Every comparison then tightened: the engine's sidereal-time branch **meets the expression at its window's bounds within 0.0014″** where it stepped by −1.909″ at 2050, and the test now holds that as the regression test for the fix; the Moon's horizontal parallax agrees to **0.0002″** where the bound was 0.5″, two thousand times tighter; the equation of time is held everywhere, 0.00007 s inside the window and 0.0075 s beyond, where it was 0.2 s from 2050; the **six equatorial Horizon rows** are compared like every other, 25,200 house values with no exception; **every star row compares** and the test asserts that none is left out, where five were reported rather than compared; and the galactic-centre ayanamshas' bound went from 0.68″ at 700 CE to 0.05″ flat while the IAU 1958 pole's went from 0.3″ into the general 0.005″. What remains is Gaia against Hipparcos (Rigil Kentaurus 6.3″ at 2100), the two readings of the 2000B arguments (C43, 0.0058″), the engine's older solar radius (0.84″ of disc) and its long-term sidereal-time construction beyond its window — every one a documented convention rather than a defect. The register records what the round trip taught: a profile is part of a fixture's provenance, a bound that a finding widened should say so, and an exception is better counted than skipped. |
| 2026-09-06 (thirty-fifth session) | What the hash matrix measured on its first run: **Linux x86-64 and Linux aarch64 compute every one of the 100,236 values bit for bit the same**, which is Phase 1's exit criterion met; **macOS aarch64 agrees on the calendars and the classical model and differs on the astronomy and the house systems**, which is a C library difference rather than an architecture one (the two Linux runners share glibc; the functions the astronomy layer calls round differently on Apple's). Measured: of the 2,022 values that differ, 1,534 differ by one place and 488 by up to a thousand, none by more, and the worst relative difference is 4.8e-14 in the house cusps, under a nanodegree. That measurement took a fix of its own: the value file wrote each double's little-endian bytes and read them back big-endian, so the first report said every difference was a whole turn; equality held either way, so the counts were right and only the distances were nonsense. The workflow now fails on the architecture comparison and reports the C library one, because a nightly that is always red teaches people to ignore it, and `cargo xtask compare-hashes` says how many values moved and by how many places rather than only that they did. Making a chart bit-identical across operating systems would mean the astronomy layer carrying its own maths functions rather than the platform's, which is a decision for its own ADR (`05-testing/01-quality-bar.md`). |
| 2026-09-06 (thirty-fifth session) | The cross-architecture hash matrix. `cargo xtask hashes` walks a fixed scenario and hashes what this build computes, per section: 2,576 calendar values (every shipped calendar over 160 000 fixed days, converted and converted back), 16,060 astronomy values (the obliquity both ways, the Earth rotation angle, the mean and apparent sidereal times, the IAU 2000B nutation, Delta T and three ayanamshas at every hundredth day of four centuries), 58,240 house values (ten systems at eight latitudes and every seventh meridian, with their ascendants and midheavens) and 23,360 classical values (the text's longitude and daily motion for eight grahas over the same span). Every value is hashed as its bits, because a difference of one unit in the last place is a difference: the point is to find out whether the same source computes the same numbers on another machine, not to decide how close is close enough, and the per-section report says which layer moved rather than only that one did. The `hash-matrix` workflow runs it on Linux x86-64, Linux aarch64 and macOS aarch64 nightly and on demand, and compares the three; the fast check stays fast, because a difference there is a property of the toolchain and the platform's maths library rather than of a pull request. It runs in a second. |
| 2026-09-06 (thirty-fifth session) | A latitude can no longer be passed where a longitude is wanted, which is half of Phase 1's exit criterion. The description says which quantity a number carries (`api: brand=latitude`), and each binding gives it a type of its own: TypeScript a branded number (`type Latitude = number & { readonly __brand: 'latitude' }`) whose constructor is the only way to make one, Dart an extension type (`extension type const Latitude._(double value) implements double`) that costs nothing at run time. Both constructors check the range the description states, so a latitude beyond ±90 is refused where it is written rather than at the boundary. The gates prove it rather than assert it: the TypeScript consumer marks a swapped pair and a bare number `@ts-expect-error`, so the check fails if the surface ever stops refusing them, and `bindings/dart/typecheck/wrong.dart` is analysed on its own and must report every error it expects, three today. The observer's fields gained their units and ranges along the way, so the C header states them too. |
| 2026-09-06 (thirty-fifth session) | The sources go to a translator's own tools and come back. `teistro-intl export xliff --locale <tag>` writes XLIFF 2.1 with the base locale as the source and the named locale as the target, one `<file>` per namespace and one `<unit>` per message and per translated entity form (`sdk.entity.graha.SUN#name`), each carrying a note that names the message's parameters so a translator keeps every one of them; a message crosses as its `MessageFormat 2` source, because XLIFF has no notion of one, and the glyph and the gender do not cross at all, a symbol being the same in every language and a gender a fact the locale states. `import xliff` is the inverse and no more: a unit with an empty target is one nobody has translated yet and is left alone, a unit whose id the base locale does not have is reported, and a form the file leaves out is kept. `sa-Deva` exports as 1056 units with 99 untranslated, and what is exported imports back unchanged, which is the test. The parser (`quick-xml`) is behind the crate's `cli` feature, so the engine the bindings embed carries no XML. |
| 2026-09-06 (thirty-fifth session) | A Sanskrit or Nepali term written in Devanagari reads in Latin, and `sa-Latn` is derived rather than written. `teistro_intl::translit` is the transliteration as a table: the vowels, the vowel signs, the consonants with the inherent `a` a sign or a virama replaces, the marks, the nukta letters and the digits, with an anusvara taking the nasal of whatever follows it (`maṅgala`, not `maṃgala`). `teistro_intl::derive` writes a locale from another through it, keeping the `iast` form because it is already in the target's script, giving each word the capital a Latin-script name carries, and applying the target's `_overrides.json` last so regenerating never loses a correction; an override that matches nothing is reported rather than left to rot. `cargo xtask gen intl` writes `i18n/sa-Latn/` and `check-intl` holds it to `sa-Deva`, which makes it the first generated locale. Its measure is the sources themselves: 241 of the 274 entities' Devanagari names transliterate letter for letter into the hand-written `iast` form beside them, and the 33 that differ are the sources' own variants (an `iast` form that adds the category word, `Deva Gaṇa` for देव), not the table's mistakes. `ts_intl_transliterate` puts the same function at the boundary and both bindings wrap it (`ctx.transliterate('सूर्य')`), with `teistro-intl derive` and `teistro-intl translit` on the command line. The Node emitter's lent-string helper was renamed along the way, because an entry point with a `text` parameter shadowed it. |
| 2026-09-06 (thirty-fifth session) | A time renders on a twelve-hour clock where a locale reads one. `:time hour12=true` (and `:datetime`) chooses the locale's `sdk.calendar.time.<style>12` pattern, and every time pattern is now given six parameters whatever the clock: the hour on both clocks, the minute, the second, the locale's word for the part of the day and its am or pm. English reads `6:15 am` and `6:05 pm`, Nepali reads `बिहान ६:१५`, `साँझ ६:०५`, `दिउँसो २:३०` and `राति १२:००`, and midnight and noon are twelve rather than zero. The parts are morning from 4 to 11, afternoon from 12 to 15, evening from 16 to 19 and night from 20 to 3, the same ranges in every locale until the sources can carry ranges of their own. Validation learnt that the engine's own patterns are rendered with a parameter set it supplies (`engine_params`), so a locale may use one the base locale's pattern does not: a Nepali clock reads by the part of the day and an English one by am and pm, and neither is inventing a parameter. Both locales gained the two patterns and the six day-period words; eight assertions in the date tests. |
| 2026-09-06 (thirty-fifth session) | The typed message accessors reached the bindings, so an application spells a message key once, in the generator. `cargo xtask gen intl` now writes three surfaces from the one model and `check-intl` holds all three to `i18n/`: the Rust one it already wrote, `bindings/node/lib/messages.js` with its declarations `messages.d.ts` (a `.js` and a `.d.ts` rather than a `.ts`, because the package ships no compiler), and `bindings/dart/lib/src/messages.dart`. Every accessor wraps its parameters as the engine's JSON takes them (`{"$entity": "graha.JUPITER"}`, `{"$date": {...}}`), so a caller passes a key or a value and never a tagged object, and the value classes gained the JSON they carry. An entity's forms needed a boundary entry point of their own: `ts_intl_entity` hands back every form the locale gives, the glyph and the gender as a JSON object lent until the next call, and a key the locale chain does not carry is `UNSUPPORTED` naming the locale that was asked. The layers wire them: `ctx.messages.sdk.reason.grahaInBhava({graha: 'graha.JUPITER', bhava: 7})` and `ctx.entity('graha.SUN').name` in both languages, with the Dart accessors their own entry point (`package:teistro/messages.dart`) because the locale's `Gender` is a word the catalogue uses too. Nine tests across the two bindings and six more values in the parity report; the strict TypeScript consumer gained six proofs of its own, where a graha that is not one, a key that is not a key, a parameter of the wrong type, a missing parameter, a write to the tree and an unchecked optional form are compile errors. |
| 2026-09-06 (thirty-fifth session) | The build handshake: the two halves of a binding must be one build, and now neither loads the other unless they are. `ts_build_info` returns a static JSON document written when the library is compiled (`crates/ffi/build.rs` records the commit and whether its tree was clean, the profile, the target, whether debug assertions and optimisation are on, the sanitizer if any, and the compiler), so asking costs nothing and the answer cannot disagree with the library it describes. Each loader applies the same three rules: another ABI or another SDK version than the generated files carry is refused; a sanitizer build is refused however it was found, because it answers differently and slowly and is never chosen by accident; an unoptimised build is refused only when the loader searched it out, because naming a path is a deliberate act and a development build is what a developer means by it. The report is on the surface too (`buildInfo` in Node, `Teistro.build` in Dart), because an application that stores a chart should be able to store what computed it, and the generated catalogues gained the SDK version they were generated from beside the ABI. Eleven tests, and the parity report grew six keys (the version, the ABI, the catalogue version, the commit, the dirty flag and the target), so a run where the addon and the shared library came from different trees fails rather than passing quietly. |
| 2026-09-06 (thirty-fifth session) | The parity gate: `cargo xtask check-parity` walks one scenario through both bindings' own ergonomic layers, each printing `key<TAB>value` lines, and compares the two value by value. Ninety values: the versions, a context's profile, locale, settings hash and the hash of the settings document as the library wrote it, 14 April 2015 in Bikram Sambat with its era and resolution, the fixed day and the weekday, a Kathmandu birth time with its offset, era, source and tzdb version and the civil time back, the scales and Delta T, a key packed and named and a refusal's status, detail and hint, a rendered message hashed, the frame and its round trip, every cell of a two-by-three grid with its longitude, latitude, distance, speed and status, the completion's steps, and the provenance hashed. Nothing in the gate says what a value should be, because the point is that the two bindings agree with each other. Two numbers are compared within 2e-9, one in the last place both reports print, so a tenth of a second in a Julian day fails; the gate was checked against itself with a moved value, an extra key and a wrong integer. Writing it found three real differences: the Node runner's FNV lost its low bits to double multiplication (`Math.imul` now), and both layers were hashing a re-encoding of the settings and the provenance rather than the text the library wrote, so both gained a `settingsJson` accessor, which is what a stored chart keeps. |
| 2026-09-06 (thirty-fifth session) | An ephemeris written in Dart answers the SDK. `bindings/dart/lib/src/host.dart` is the port adapter, hand-written as the architecture says a port adapter is: a class extending `EphemerisProvider` is bound into the vtable through `NativeCallable.isolateLocal`, whose function pointer is callable only from the isolate that made it, which is the boundary's contract exactly. It settles the same four things the Node adapter does (one call per grid; nothing means "not in that frame", so the SDK asks again in the provider's own and completes the rest; the port's checks run before the callback, so a body, an observer or an instant it never declared is refused by name; only a code crosses, so the sentence the provider threw is what the caller sees), and two of its own: the vtable, the capability strings and the callbacks are allocated for the life of the binding rather than in an arena, and a `Finalizer` closes them if nobody disposes the context. Three things the boundary gained so no binding writes a number: `ProviderCode` (the codes a vtable function returns) and the three capability enums (`DistanceUnit`, `SpeedModel`, `Astronomy`) are described and reach the C header, the TypeScript surface and the Dart enums; and every emitter now drops the Rust examples out of a doc comment, because a Rust doctest shown to a C or Dart reader as if it were theirs is worse than no example. Six provider tests (twenty-one in the package). |
| 2026-09-06 (thirty-fifth session) | The Dart binding, from the same description as the C header and the Node binding: `crates/idl/src/emit/dart.rs` renders `bindings/dart/lib/src/` (`catalogue.dart`, every enum a Dart enum carrying the boundary's id and the key the packs spell, a catalogued member gaining `fullKey` and an `unknown` member, and the boundary's constants beside them; `ffi.dart`, the `dart:ffi` declarations matching the header name for name, the library class that looks every symbol up once, a value class per struct that marshals itself into an arena with a bitset field as a `Set`, the context with a `NativeFinalizer` over `ts_context_free`, and the exception with its detail, field and hint; `blob.dart`, one decoder per result blob with columns as typed-data views). `bindings/dart/lib/teistro.dart` is the hand-written layer: it finds the shared library and checks its ABI, opens contexts with the defaults, carries the JSON both ways, and adds what a generator cannot know is wanted (`Calendar.gregorian.date(2015, 4, 14).at(hour: 0, minute: 20)`, a grid cell by its two indices, a rendered message's warnings). `cargo xtask check-dart` builds the library, writes the blob fixtures, analyses with `--fatal-infos`, format-checks and runs fifteen tests, which found three real defects on their first run: text sections read as code units rather than UTF-8, a blob at an offset a typed list cannot start on, and enum members keyed unlike the TypeScript surface's. Three things the description learnt along the way, all shared: what a call hands back (the returned scalar and every out parameter) is now one rule in `rules.rs` that both emitters read, so a call that hands back two things is a named record in Dart and the object of the same field names in Node; the boundary's constants reach both bindings rather than being written as literals; and the three binding gates share their steps in `xtask/src/binding.rs`. The generated Dart is 7,600 lines and carries `// dart format off`, so `check-ffi` compares it byte for byte on a machine with no Dart toolchain. |
| 2026-09-06 (thirty-third session) | The Node addon, generated from the same description as the C header and the TypeScript surface: `crates/idl/src/emit/node.rs` renders `bindings/node/native/src/generated.rs` (a `#[napi]` class over the context handle with a method per entry point, an object per boundary struct with a `Held*` value owning whatever the C struct points at so a borrowed buffer never outlives its call, the enums as the strings the tables name, the calls with their `unsafe` blocks, and `lastError` for the layer to rethrow). To render it the description learnt two things it lacked: **struct field roles** (`api: flag`, `bitset=`, `len=`, `present_if=`, and the fixed-byte and column shapes), so `ts_position_request` reaches JavaScript as `{ speeds: boolean, observer?, jds: number[], bodies: Body[] }` with no counts and no presence flag, and **per-parameter metadata** (an `api:` line opening with a parameter's name), so an integer parameter standing for an enum crosses as a member. Both improve the C header and the TypeScript surface as well. `bindings/node/lib/index.js` is the hand-written layer (380 lines): it finds the addon and checks its ABI, normalises what JavaScript spells `null` and napi spells absent, validates at the door, decodes a result on first use, names a positions row's bodies by their keys, and rethrows a failure as a `TeistroError` with the status, code, detail, field, hint and message key. `cargo xtask check-node` builds the addon, copies it where the loader looks, writes the blob fixtures and runs fourteen tests; nine of them drive the whole surface (a context and its settings, a patch that moves the hash, the refusals, 14 April 2015 as 1 Baisakh 2072, a Kathmandu birth time at +05:45 under tzdb 2026c and back, the scales through the leap-second table, positions with the frame round-tripped and the provenance, a Nepali message, a key packed and named). The strict type-check gained the layer's own declarations. |
| 2026-09-06 (thirty-second session) | The Node binding's generated layers, from the same description as the C header: `crates/idl/src/emit/ts.rs` renders five files into `bindings/node/lib/` (`catalogue.d.ts` and `catalogue.js`, every enum a string union with a frozen table beside it, a catalogued member spelled as its full key `'graha.SUN'` with the `'unknown'` arm §3.6 requires and a kind spelled as a key's first segment `'avastha_baladi'`; `types.d.ts`, every boundary struct a readonly interface importing exactly the enums it names, each member carrying its documentation, unit, range and example as JSDoc, plus the `TeistroError` class; `blob.d.ts` and `blob.js`, one decoder per result blob over the `TSRB` layout, columns as views over the blob's own bytes, a buffer at an odd offset copied once rather than misread, and a wrong magic, version, length or schema id a `TypeError`). `cargo xtask check-node`: `cargo run -p teistro-ffi --example blob_fixtures` writes a positions blob (two instants, three bodies) and a Nepali render blob through the C ABI, five Node tests decode them (the grid, the cells' statuses and longitudes, a column proved to be a view, the steps and the provenance as the JSON the library wrote, five refusals, an odd byte offset, the tables' keys), and `bindings/node/typecheck/consumer.ts` type-checks at maximum strictness with every wrong usage marked `@ts-expect-error`, so a surface that stops refusing one fails the check (a misspelt key answers "Did you mean '\"graha.PLUTO\"'?"). The declarations are 6,000 lines from 71 enums and 25 structs. Next: the napi addon and the ergonomic layer, then Dart. |
| 2026-09-06 (thirty-first session) | The C ABI and the API description toolchain built from spike 2: `crates/idl` (`teistro-idl`: the description model `teistro-api/1`, the naming rules, the C layout rules on both pointer widths, the `TSRB` result blob encoder and decoder, the extractor over Rust source with roles inferred from types and names and the `api:` metadata line, the SDK's sources and catalogue kinds put through it, the C header emitter with `_Static_assert`s of every struct's size) and `crates/ffi` (`teistro-ffi`, the workspace's only `unsafe`: contexts from a profile, a JSON settings patch, a locale and the port's vtable, with the size handshake refused as `SCHEMA_VERSION`, the last error kept on the context, owned strings and blobs freed by the library, lent strings valid until the next call, a panic guard into `INTERNAL`; keys and ids; dates in every shipped calendar with eras and resolutions; civil times to instants with the zone metadata, the scale conversions, Delta T; the locale engine over the SDK's bundles embedded at build time; positions through the port completed into the requested frame as a blob with the steps and the provenance). Thirty-six entry points, twenty-five structs, seventy-one enums (the catalogue's kinds among them), eight callbacks; `idl/api.json` and `bindings/c/include/teistro.h` by `cargo xtask gen ffi`, gated by `check-ffi`. The C binding's own test (`bindings/c/tests/smoke.c`, `cargo xtask check-c`) is a consumer of the header alone, compiled with warnings as errors and run: it converts a date into Bikram Sambat, resolves a Nepali birth time, renders a Nepali message and reads the Sun's longitude out of the positions blob. Writing it found the gap it closed: a C caller could not build a position request's frame, so `ts_frame` names the centre, equinox, coordinates, zodiac and corrections by field and `ts_frame_canonical`, `ts_frame_pack` and `ts_frame_unpack` move between them and the bits; it also found the kind members rendering lower-case in the header (`TS_KIND_graha`), now upper-cased with the key. The port's vtable callbacks became named aliases, `Body` and `TimeScale` gained `#[repr]` discriminants, the port's request decoding is one method. Found and fixed: the settings hash depended on whether any crate in the build enabled the JSON layer's `preserve_order` feature; `canonical_json` sorts keys itself now (ADR-0022). `DEFAULT_PROFILE` (`parashari-classical`, Q34 for the maintainer), `CALCULATION_VERSION` and the catalogue schema version live in `core`. The design page `03-design/ffi-abi-and-api-description.md`; the binding architecture, API conventions, module catalogue, settings, core, time and calendar pages, the glossary and the module guideline revised. `teistro-idl` 17 tests, `teistro-ffi` 16 tests, both with doctests. Next: the Node and Dart emitters and bindings with the parity gate. |
| 2026-09-06 (thirtieth session) | `teistro-intl migrate baseline` (`crates/intl/src/migrate.rs`): the baseline engine's entity name tables, exported by a names exporter beside its golden-vector exporter into `fixtures/baseline/names.json` (40 types, 383 entities, four languages), mapped onto the catalogue's kinds with the spelling aliases the two do not share and written as `sdk.entity` records (`name`, `prose`, `short` where the engine abbreviates, `iast` from the language or the Sanskrit transliteration, glyph and gender from the engine or the catalogue's skeleton); twenty types mapped (274 records per language, 0 unknown keys), twenty without a catalogue kind reported for the catalogue's growth; `en-Latn` and `ne-Deva-NP` keep their 49 hand-shaped records and gain 225, `hi-Deva-IN` and `sa-Deva` are created at `base` completeness. A catalogue kind's name (`avastha_baladi`) is a key segment now; the intl gate prints the summary when the sources pass. 47 tests. |
| 2026-09-06 (twenty-ninth session) | The intl date functions: `:date`, `:time`, `:datetime`, `:ghati` and `:duration` over the calendar crate (`Value::Date(CalendarDate)`, `Time(ClockTime)`, `DateTime`, `Ghati`), rendering through the patterns and names a locale declares in `sdk.calendar` (month and weekday names as `.match` messages, `date.numeric|long|full` per calendar, `time`, `datetime.join`, `ghati`, `duration.<unit>` plurals; a calendar sharing another's names links them with `:msg`), `calendar=` converting through the shipped calendars, `pattern=` naming any message, built-in defaults with a warning when a locale declares none; `useGrouping=false` and `minimumIntegerDigits` on numbers; the analysis types the new functions and links `pattern=` targets; the validator refuses a date selector and lets a linked message forward the base's parameters. `sdk.calendar` shipped for `en-Latn` and `ne-Deva-NP`; era records for the nine eras in both. The architecture page's Bikram Sambat example `२०८१/०५/१९ गते` renders as written. 44 tests. |
| 2026-09-06 (twenty-eighth session) | The intl engine's runtime API (`crates/intl/src/runtime.rs`): `Intl::load_pack` takes a `.tpack` or `.tbundle` after construction (a new locale with its metadata and plural rules, a new namespace, or entries replacing loaded ones, the record with the file's SHA-256 kept for the envelope), `set_override`/`set_overrides`/`clear_override`/`clear_overrides` patch messages in memory, checked as they are set and standing before the locale's own entry and any fallback, `report` lists every locale's coverage of the base keys, the files loaded and the overrides in force; `Rendered::is_override` and `Intl::resolution_from` say when an override answered; the parse cache forgets what the runtime API replaces. 39 tests. |
| 2026-09-06 (twenty-seventh session) | Teistro Intl built from spike 4 as `crates/intl` (`teistro-intl`): the engine, sources, analysis, validation, packs and generators promoted with the SDK's catalogue as the authority for entity keys (`teistro_core::key::resolve` at validation and render; `kind=` must name a catalogue kind; coverage per catalogue kind reported), `:zodiac` over the catalogue's sign keys, `Value::catalogued(Graha::Sun)`, the `TypedMessage` trait with `Intl::render_typed`; the locale bundle (`.tbundle`, format 2: a pack may carry no metadata when its bundle does, `Bundle::parse`, `locales_from_packs` reading both); the Rust accessor emitter (`generate::rust`, `RustPaths`) and the SDK's own typed messages in `src/messages.rs` by `cargo xtask gen intl`, gated by `check-intl` in CI; the `teistro-intl` command line as library functions (`validate`, `build --bundle`, `gen --target ts,dart,rs`, `render`, `extract`, `report`); `i18n/` at the repository root with `en-Latn` and `ne-Deva-NP` (the Lagna record moved from the grahas to the points, where the catalogue has it); a criterion bench. 37 tests. |
| 2026-09-05 (twenty-sixth session) | Phase 2's exit review against `07-roadmap/00-roadmap.md`: the exit criteria are met (the accuracy document generated and gated with every built row within its target against Teimeris; houses for all systems and sunrise for a Nepali place without an override); the deliverables deferred by decision are named (the completion's centre, corrections and equinox steps to Phase 3; eclipses to v1.x); Phase 1's remaining deliverables are listed as the next block of work. Recorded in the project phase line, the roadmap's Phase 2 section and "Now". |
| 2026-09-05 (twenty-fifth session) | One request for both bodies of a composite quantity: `Longitudes::longitude_and_speed_pair` (provided as the two single readings; the completion answers one position request, sharing the instant's obliquity, nutation and precession), used by every composite crossing and by the visibility reading of the body and the Sun; a counting source in the tests proves the tithi search reads pairs alone and the ingress search singles alone. Measured A/B in one run: the tithi search 336 µs to 185 µs over the test provider, the ingress search unchanged. The events and phenomena pages' performance tables re-measured in one machine state, with the note that rows compare within a table (the machine's state moves every row by tens of per cent between sessions). Next: the visibility follow-ups; cusp speeds; Phase 2's exit review. |
| 2026-09-05 (twenty-fourth session) | Visibility and the heliacal phenomena (`astro::visibility`): three named criteria, the Surya Siddhanta's degrees of time (IX.2 to 11, X.1, read from Burgess's 1860 translation: Jupiter 11, Saturn 15, Mars 17, Venus 10/8, Mercury 14/12, the Moon 12; the star classes of IX.12 to 15 and the six stars of IX.18 as data), the tradition's combustion orb over the same numbers in longitude, and Ptolemy's arcus visionis (Almagest XIII.7 to 9 as Burgess quotes them) read at the deepest twilight the body is up in; the state of a local mean day and the day-by-day scan for the four heliacal events over the rise and set solver and the completion, so any provider answers. `Solver::altitude_deg`, `sky::local_mean_midnight` (moved from the classical crate). By hand against Teimeris's photometric model at Kathmandu: Venus's rising of June 2020 and setting of May 2020 within two days under every criterion, Jupiter's rising of February 2021 within a week (bound ten days). Research: the tradition's combustion orbs verified as the text's own numbers (C17 closed at rank 1; C44 opened on the unit). Next: one request for both bodies of a composite crossing; the visibility follow-ups. |
| 2026-09-05 (twenty-third session) | The sidereal time expression question closed by measurement: the engine's default sidereal time strictly inside its 1850 to 2050 window is the IERS 2010 expression and agrees with the SDK's IAU 2006 form to 0.0012″ (1850) and 0.0004″ (from 1875); the +0.088″ once read "inside the window" was the boundary instant, which the engine gives to its long-term branch. The SDK's GAST moved from `gst00b` to `gst06b` (IAU 2006 mean sidereal time, IAU 2006 mean obliquity, IAU 2000B nutation): cusps move under 0.002″ between 1950 and 2050. New: `iau::ee06b`, `iau::gst06b` (against ERFA's `ee06a`/`gst06a` within the 2000B truncation), the adapter's `sidereal-table` binary, `fixtures/teimeris/sidereal.json` (49 instants), `tests/teimeris_sidereal.rs` (three accuracy rows in CI), crux C43 (the 2000B nutation read two ways). Engine findings: F1 measured beyond the window (−0.50″ at 1700 to +2.46″ at 2300, commented on teimeris#1); F6 filed (teimeris#6: the 2000B fixed offsets −0.135/+0.388 mas omitted, inherited from upstream). Next: the heliacal phenomena. |
| 2026-09-05 (twenty-second session) | The `CROSSINGS` override: the crossing vocabulary moved into the port (`crossing.rs`: `Quantity`, `Lattice`, `Direction`, `Event`, `CrossingRequest`; the events module re-exports it), `EphemerisProvider::crossings`, a vtable slot with a caller-owned buffer that grows to the count reported (ABI version 2), `Completion::crossings` choosing by the override policy and falling to the kernel for a request the provider refuses, the Teimeris adapter's implementation over its crossing search (the direction from the quantity's rate), and the kit's two crossings checks. Against Teimeris: Mercury's sign crossings within 0.0034 s and the tithis within 0.0039 s of the kernel, 18 checks all passed. A latent kit defect fixed on the way: the Surya Siddhanta ayanamsha expectation was the text's own value for every provider; for a modern engine it is now the catalogued epoch definition. Next: the heliacal phenomena, the sidereal-time expression question. |
| 2026-09-05 (twenty-first session) | The accuracy document, the Phase 2 exit artefact: `cargo xtask accuracy` runs the astronomy layer's measurement tests with `TEISTRO_ACCURACY_DIR` set, each recording its worst difference against its recorded engine or baseline table (`crates/astro/tests/common/mod.rs`), and renders `05-testing/ACCURACY.md` from those measurements and `accuracy-rows.yaml` (the seventeen areas of the astronomy layer with their targets, evidence and by-hand measurements); `check-accuracy` regenerates and compares in CI. Next: the `CROSSINGS` override, the heliacal phenomena. |
| 2026-09-05 (twentieth session) | The engine findings register (`05-testing/02-engine-findings.md`) and the rule behind it: five discrepancies the measurements traced to Teimeris measured, filed as `teispace/teimeris` issues #1 to #5 with reproductions and suggested fixes, assigned to its maintainer, and entered with the SDK's handling: the sidereal-time steps of 1.9″ at 2050 and 0.1″ at 1850 (which explain the equation-of-time gap; the earlier Delta T reading was wrong and is corrected), the Moon's disc and parallax from distances 40 km apart, a point's magnitude as 0.0, the Horizon system's Munkasey co-ascendant at the equator, and five star-catalogue rows. Next: the accuracy document, the `CROSSINGS` override. |
| 2026-09-05 (nineteenth session) | `astro::phenomena` and the equation of time: elongation, phase angle and illuminated fraction, the apparent disc and horizontal parallax (the rise and set solver's `Disc`), and the visual magnitude under the Almanac's models (Mallama and Hilton 2018 for Mercury to Uranus, Neptune's calendar step, Pluto's IAU 1986 polynomial, Allen with Samaha's crescent for the Moon, the inverse-square Sun), over the completion (the provider's heliocentric position at the retarded instant when it answers one, the geocentric difference otherwise) or a supplied geometry; `sky::equation_of_time_seconds` from the SDK's sidereal time and the Sun's apparent right ascension. Measured against Teimeris over its own geometry: angles within 1e-9°, magnitudes within 0.001 (the Sun's disc radius), the equation of time within a millisecond. Design page `astro-planetary-phenomena.md`; C19 updated. Next: the accuracy document, the `CROSSINGS` override. |
| 2026-09-05 (eighteenth session) | The star table: `catalogue/star.yaml` (kind 56, 128 members: the 27 yogataras with Vega, the ayanamsha anchors, the bright fixed stars, Sagittarius A* and the two galactic poles, each with SIMBAD's ICRS astrometry and the bibcode of every value) and `star_class`; new ERFA ports (`epv00` with its 1951 rows, `pmpx`, `ld`, `ldsun`, `ab`, `numat`) against the reference values; `astro::stars` placing a direction on the equator and ecliptic of date (proper motion, parallax, deflection, aberration over the SDK's own Earth ephemeris, frame bias, precession, nutation); the twelve anchored ayanamshas computing through it. Measured against Teimeris: the mean places over the engine's own astrometry bit-identical, the apparent within 0.0005″ (the nutation models), the parallax kept under its true-position flag; the SDK's astrometry against the engine's within 0.71″ (Gaia DR3 against Hipparcos); the anchored ayanamshas within 0.003″ where the rows are the same and by the data where they are not (C40 to C42). 12.6 µs the Earth's state, 13.9 µs a star's place. Design page `astro-star-table.md`. Next: a `CROSSINGS` override, cusp speeds, the equation of time. |
| 2026-09-05 (seventeenth session) | `astro::events`: the crossings and stations kernel over the boundary solver: a body's longitude, a composite angle of two bodies or a speed over a lattice of boundaries (the signs, the nakshatras, the tithis, the karanas, the yogas) or a single target, sampled at half the spacing over the greatest rate and never more than a day, unwrapped, each line narrowed by the shared solver; stations as the speed's sign changes; a synthetic looping planet as the retrograde test. The solver's narrowing moved from bisection to the ITP method with a floating-point guard: at most nine evaluations an event where bisection took twenty-seven, never more than a bisection and one. Measured against Teimeris's own searches: ingresses and tithi boundaries within 0.004 s, stations within 0.3 s; against the baseline's 280 geocentric panchanga transitions within 7.8 s (median 3.3 s, the baseline's own search). Design page revised. Next: the star table. |
| 2026-09-05 (sixteenth session) | `astro::houses`: the twenty-two catalogued house systems as one construction with the circles each picks, the auxiliary points, the sign-based systems in the zodiac in use, the four polar policies with the outcome reported. Within 4.8e-6° of Teimeris over 25 194 cusps and angles at ten latitudes (the adapter's `houses-table` binary), within 0.00021° of the baseline's 55 charts between 1800 and 2200. Design page `astro-house-systems.md`. Next: crossings and stations, the star table. |
| 2026-09-05 (tenth session) | The Bikram Sambat computation engine: `crates/siddhanta` (the text by verse, exact mean places, the sine table, both equations, the four steps, motion, precession, declination, the day's arc; 54 ns for the Sun, bit-identical), the `astro` seed (the boundary solver), the `time` seed (offset histories, Nepal's rows) with `core::time`, and in `crates/calendar` the `SolarModel`, the sankranti finder, the month-start rules as cited rows, the engine, the fit report and the table regenerated for 1700 to 2500 BS with a CI gate. Measured: the text's Sun at Kathmandu under Nepal's clock with the Dharmasindhu's punya-kala rule reproduces 1490 of 1512 official month lengths (98.5 %), 116 of 126 years exactly, every year total and every New Year, no drift; the eleven residual boundaries lie within 25 minutes of the rule's boundary. Findings: the baseline's seven-hour epoch shift and 0.705 cutoff nearly cancel to the civil day; the two ayana sankrantis are the whole difference; exact trigonometry changes one boundary; the tradition's day count changes none. Next: `crates/time` proper, then the port promotion with the drik model. |
| 2026-09-05 (ninth session) | `crates/calendar` built: the fixed day, Gregorian, Julian, mixed (1582, 1752, 1918) and ISO week with every day of −9999 to 9999 round-tripped and agreed with the `calendrical_calculations` oracle; Bikram Sambat over the baseline's table (1856 to 2457, official span stamped `Tabular`, the rest `Computed`) anchored on 13 April 1913; the source memo opened with the generator's findings (Surya Siddhanta at Kathmandu, Nepal's offset history, a fitted 0.705 cutoff, 87 % of month splits, drift within a day). Maintainer's mandate: compute Bikram Sambat from first principles for any year so Nepal's panchanga can use the SDK. Next: the Bikram Sambat engine (siddhanta Sun, drik through the port, rule rows, fit harness), then `crates/time`. |
| 2026-09-09 (sixty-third session) | The Indian lunisolar calendar designed and its month built. The design page Phase 2 named and nothing wrote is now written, from the measurement rather than from a textbook, and its scope is the finding: it is **the mark and not dates**. `panchanga` needs to know whether a month is adhika; a full `CalendarSystem` needs a date shape the SDK does not have, because a lunisolar date's day is the tithi at sunrise, which repeats one day in forty-four and is skipped one in twenty-six — so `(year, month, day)` is not a key and a date wants two flags `CalendarDate` does not carry. Separating them is what let `panchanga` be finished without a boundary change nobody has argued for. The module: a `LunarModel` trait beside `SolarModel` (its own, because a solar calendar needs no Moon and three of that trait's five implementations are test doubles), a three-way `MonthKind`, `kind_of` for a span whose bounds a caller has and `month_at` for one it does not. `panchanga`'s `LunarMonth` carries the kind beside the name, and finding it made the existing code better rather than longer: `limb` used to find the month's opening new moon and throw the search away, and now `lunar_month_span` returns both bounds while `masa_at` names from the first — one crossing search serving the name and the mark where there were two serving one each. Five module tests hold the rule at the two recorded adhika months, the shared name of an adhika month and the nija one after it, a contiguous year, a kshaya month where the measurement says one is, and that a month boundary really is a new moon. Next: the mark crosses the boundary with the panchanga blob's `days` section, then the three bindings and parity. |
| 2026-09-09 (sixty-second session) | The rule that decides **adhika** and **kshaya**, measured (`cargo xtask lunisolar` → `03-design/calendar-indian-lunisolar-measured.md`, gated by `check-lunisolar`). `panchanga` names the lunar month and cannot mark it; this is the pass the calendar that will mark it is designed from. The first thing it had to establish is that **the corpus cannot settle it alone**: it records `is_adhika` on every day — the *answer* — and none of the inputs, neither the new moon that opened the month nor the sankranti that named it, so unlike the panchanga's conventions this cannot be arithmetic over recorded numbers. It computes the sky from the **Surya Siddhanta**, as the Bikram Sambat engine does, so the calendar needs no ephemeris — which is what the tradition did. Over **12 368 lunar months of a millennium**: 388 hold no sankranti (adhika, one every 2.58 years against the classical seven in nineteen), 11 961 hold one, 19 hold two (kshaya, one in some fifty years). That count is the rule, and it reproduces the corpus's marking on **all fifty-five** days including the two marked adhika. Two things the measurement corrected. **An adhika month needs no naming rule of its own** — the usual formulation is that it takes the following month's name, but the Sun stands in the same sign at both new moons, so the existing rule gives both the same name unaided; August 1947 is Shravana twice over. And **the classification is robust where the month of an instant is not**: the one recorded day where text and recording disagree is nineteen minutes from the eclipse new moon of 8 April 2024, with the two conjunctions on either side of it. **Kshaya is measured and not tested** and the page says so: the corpus records none, so nothing holds the rule to an authority — what can be said is that its frequency matches the astronomy and that all nineteen fall between Vrishchika and Kumbha, the perihelion window, which the pass did not look for. Next: the design page and the calendar module the rule goes into, then wiring it through `panchanga`. |
| 2026-09-09 (sixty-first session) | The parity gate over the two new blobs: **594 values compared across three bindings**, where it compared 103. All 103 came from `positions`, so until now "the bindings agree" meant something for one entry point, and the chart and almanac examples agreed only because a person had compared their output by eye. The scenario needed a second context to say it at all: the gate runs under `nepali-default`, whose frame is **topocentric**, and a chart cannot be founded under it — the completion's centre step is Phase 3's. That refusal is now a compared value of its own (`chart-under-topocentric`), so the three must fail the same way and not merely succeed the same way, and everything after it runs on a second context under the geocentric default. Two instants and three days, deliberately: a per-chart section laid out charts-outermost the wrong way round shows as the second chart's values in the first's place rather than as nothing, and two consecutive days with the same list counts would not exercise the ragged offsets. **The gate found a gap on its first run.** When `part` and `elapsed` moved out of the shared day section into the chart's `cast` last session — they belong to an instant, not a day — the columns were added and *no layer surfaced them*: Node had nothing, Dart and Python could only reach them by indexing the raw column. All three now expose `dayPart`/`dayElapsed` on a chart and the runners read the accessor. mypy caught a second thing the eye would not: the new loops shadowed names bound earlier in `main`, which strict mode refuses. Next: the JSON Schema emitter is still blocked on the description holding no document type; after it, adhika and kshaya months. |
| 2026-09-09 (sixtieth session) | The daily panchanga at the boundary, built on the layout the previous session measured. `ts_panchanga_days` takes a **range** — consecutive days share a boundary, so a month costs much less than thirty days computed separately — and answers with one blob: the four moving limbs, the periods, the lunar month under both conventions, what the Moon and the Sun did, and what each day is said to be. Every per-day list is concatenated across the batch with a `counts` section saying how many rows are each day's, one rule for all thirteen. Three decisions a chart blob never had to make: **a value a day may not have crosses as a presence flag beside it** (an absent Abhijit and an Abhijit at Julian day zero are both nought, so no sentinel serves); **two lists answering one question in two halves become one section with a discriminant** (the muhurtas of the daylight and of the night, the moonrises and the moonsets); and **the seven span lists do not share a shape** — the first place that rule needed a boundary, since a shape makes two sections decode to one type, which is right for the same section in two blobs and wrong for a span of tithis beside a span of nakshatras. A shape is for sameness of **meaning**, not similarity of structure; the declaration's repetition goes to a helper instead. Building it found that **the shared day section carried two fields that were never a day's**: `part` and `elapsed` belong to an *instant*, a chart has one and a panchanga day has none, and the page had said all along the shared section is *eighteen* fields while it held twenty — nothing noticed because a chart was the only blob carrying a day, and a field wrong for a reader who does not exist reads as right. All three bindings gained `almanac(range)` and `almanacDay(one)`, each day a view over its batch with the ragged offsets prefix-summed **once** at decode rather than per access, and each ships a week's panchangam that prints byte-identical pages in all three. Two gaps the examples found and the SDK records rather than papers over: `has(key)` answers for a message and there is no non-throwing way to ask it of an **entity**; and **`masa` and `direction` have no name in any of the five entity packs**, so an almanac cannot print the lunar month or the disha shool in the reader's language — content needing a source rather than a guess. Next: `check-parity` over both blobs (103 values today, ~224 with them). |
| 2026-09-09 (fifty-ninth session) | The shape of a batch of almanacs, measured, and the step budget it found (`cargo xtask almanac` → `03-design/panchanga-at-the-boundary-measured.md`, gated by `check-almanac`). The chart blob needed no shape pass — its per-chart sections have a stride the blob states once — and the panchanga's do not, so this founds **450 days at three latitudes** and counts what each of a day's fifteen lists would put in a blob. The fifteen split cleanly in two, and not the way the names suggest: **every fixed list is a division of an arc** (24 horas, 15+15 muhurtas, 16 choghadiya, 3 kaalas) and **every ragged one a crossing inside the window** (a tithi boundary, a moonrise, the Moon entering a sign). A division's count is a convention, so a polar day whose synthesised arc runs a fortnight still has 24 horas, each fourteen hours long, while its crossings multiply to 62 tithis and 122 karanas. That decides the layout by two orders of magnitude rather than by argument: a rectangular blob wastes 10.4% of its rows over ordinary days and **78.1%** once one polar day joins the batch, since that day sets the stride for every other — 50 484 empty rows in `limbs.karana` alone. The pass could not reach a polar day at all on its first run: both Tromsø ranges refused with `NOT_CONVERGED` at **both** solstices, and the cause was not the horizon but the horizon scan's bracket cap — a constant 400, described in its own comment as "a day of ten-minute steps" though 400 of them is two and three-quarter days, so a caller searching a longer window met the constant. A Moon that does not rise at 69.65°N is an answer; a step budget is not, and the caller cannot tell `Ok(None)` from `Err`. The cap is now sized from the span, with 400 as a floor: only calls that previously failed change, so no number moved (73 astro tests, the panchanga suite and `check-accuracy` all unmoved). Two things the pass declined to settle and recorded instead: whether a synthesised polar arc should carry 62 tithis at all, or whether an almanac that long is a different question from the one `Almanac::day` answers; and that the default profile refuses a polar day outright under `UNDEFINED`, which is `panchanga-day.md` §15's first row met in practice rather than read. Next: the panchanga blob itself, ragged sections with an offset column each. |
| 2026-09-08 (fifty-eighth session) | A founded chart crosses the boundary, **as a batch**, and the ergonomic layers of three bindings meet it. `ts_chart_found` takes a grid of instants and answers with one blob of charts founded at one place: the grahas placed under both readings, the twelve bhavas and the chalit, the zodiac, the day and the birth timing, every per-chart section **charts outermost** as the positions blob puts instants outermost. The first version took a single instant while `Founder::found(instants, place, kind)` sat unused beside `found_one` — the exact dead end the design page's own §3a warns against, written by the page that warns about it. Building the batch corrected the design in four places: the ayanamsha offset is per **chart**, not per request, because it precesses; the per-request sections are written **from the request** rather than scraped from the first chart, so a batch of none still says under what it founded none; `day` and `timing` became column sections, which forced shaping to work for column sections and not only fixed ones; and a column section written from rows needed `Writer::rows`, which transposes rows of `FixedValue` into typed columns and **refuses** a value too wide for its column, where a fixed section's eight-byte slots cannot notice. `check_shapes` now refuses two sections that name one shape and disagree about it, since the emitters render a shape once and would decode the second through the first one's type. Each layer offers `found(one)` and `foundMany(list)` over the one crossing, with a chart a **view** over its batch rather than a copy, and each ships a rectification example — a birth time known only to the hour, eighteen candidates ten minutes apart in one crossing, the lagna changing sign at 01:20 — the same program three times, printing the same numbers. Writing it three times found what no gate had: the Node binding's TypeScript surface declared **no chart at all**, neither `found` nor the class it returned, so its two byte-section accessors had never run and both double-decoded text the decoder had already decoded. Recorded rather than fixed: the two blobs spell a completion step two ways (`{"name","implementation"}` with `PASS_THROUGH` against `"positions:PassThrough"`), because `ChartFoundation` derives `Deserialize` and `Step` borrows a `&'static str`; an empty grid is refused for positions and accepted for charts; and the day's seventeen other catalogued columns cross as bare ids because naming them by hand in three bindings is a table that drifts. Next: the panchanga blob over the shared day section, then parity over the chart (103 values today, ~224 with both blobs). |
| 2026-09-08 (fifty-seventh session) | The first two steps of `03-design/chart-at-the-boundary.md`, and three things reading found that the design had not. `SectionSchema` gains an optional **`shape`**: two blobs carrying the same section — a chart's day and a panchanga's day are the same eighteen fields with the same values — declare it once and name the shape, so a binding decodes both into one type rather than two identical ones; a section without one behaves exactly as before and none has one yet, so nothing regenerated. Then, from reading the emitters before writing a section for them: **a fixed section could not carry a double.** Every fixed field the SDK had was an integer, so the Dart emitter declared `final int` for all of them while its reader already dispatched on the scalar and would have emitted `getFloat64` — a double would have been read correctly and then failed to compile on assignment. TypeScript says `number` and Python dispatches, so Dart alone was wrong. The fix regenerates every binding byte for byte, so a synthetic schema carrying one of each is what holds it. Two design questions closed by reading the types rather than reasoning about them: the day's **date is already at the boundary** (`TsCalendarDate` with `TsResolution`, because a date crosses for `ts_calendar_convert`), so four missing enums were really two; and **a tagged enum crosses as `<name>_kind` and `<name>_value`** — `SunriseConvention::Custom { altitude_deg: f64 }` exists today, so `kind` beside `which` is wrong now rather than later and a `u16` over the pairs is wrong too, the pairs not being enumerable. A fixed section already gives every field an eight-byte slot, and `frame_bits` is the precedent for a scalar the ergonomic layers unpack, so this needs no new machinery. Next: the two blob schemas and their entry points, every question they need now answered. |
| 2026-09-08 (fifty-sixth session) | The chart foundation and the panchanga day at the boundary, designed. Asking what was next found that the JSON Schema emitter is blocked on something the schema pass had not looked for: the schema comes from the API description, and the description holds 25 structs — all C-boundary types, **none of the chart document's**. It describes the C ABI, and the document is a Rust value tree that does not cross it. The work that unblocks the schema is therefore the work that makes a chart crossable at all, which is **Phase 4's own exit condition** (the golden vectors reproduced in three bindings) and the reason each binding's examples stop where they do. `03-design/chart-at-the-boundary.md` designs `ts_chart_found` and `ts_panchanga_day` and the two blobs they answer with, written from the shape `schema-measured.md` had already measured: a foundation is 72 distinct paths in twelve groups, and only `grahas` repeats per row, which is what makes a columnar section right for it and wrong for the rest. The part worth arguing is §3, **what is reused rather than described again** — `zodiac.request` is a `Frame`, so it is the packed `frame_bits` the positions blob already writes and every binding already unpacks (eight fields to one); the day's place is the chart's place (three to none); and the two shapes that genuinely appear twice, a graha's `house`/`placement` and the `houses`/`chalit` reading, are named once. A blob that wrote every leaf it was given would be correct and would enshrine the repetition in four generated decoders and three ergonomic layers. §7 argues two sections and not seven: no blob but `positions` and `intl_render` exists and neither holds a nested value tree, so a mistake in the reuse would be repeated seven times before anything caught it. Next: build it. |
| 2026-09-08 (fifty-fifth session) | The chart document reads back, and the parser feature that made it possible. 60 of the layer's 65 types now derive `Deserialize`; the five that do not are the five that **cannot**, and they are one shape — a value whose identity is a shipped constant holding a `&'static` no document can produce (a divisional scheme's group table, its `Map::Listed` of signs, an aspect angle's key, the drishti table a chart was read under). Each has a hand-written reader that reads the value back **by its identity** and checks what the document says about the table against this build's, so a document naming D9 and describing something else is refused by name — stricter than a derive, and what a stored chart wants. `canonical::from_hash_form` is the reader, `canonical::reads_back` the question, and `tests/document.rs` holds the gate the schema pass said could not be written: every sample validates and reads back equal, in bytes, in value and in hash. Two leaf types (`Hora`, `Frame`) and their enums gained the derive, and `Input`/`BatchInput` deliberately did not — they are hashed, never read. **The generated catalogue readers were broken and nothing had tried them**: all 60 used `<&str>::deserialize`, which needs a string borrowed from the input buffer, so a catalogue enum could never be read through a `Value`; the generator now emits `Cow<'_, str>` and they are `DeserializeOwned`. The parser was the real work. `serde_json`'s default float path is a fast one that is not correctly rounded — it read 84 of a chart document's 1518 numbers a unit in the last place low — so `teistro-core` asks for **`float_roundtrip`**. `arbitrary_precision` was tried first and is the wrong tool: serde buffers an internally tagged enum before writing it and the buffer emits `{"$serde_json::private::Number": ...}`, which broke `DeltaTModel` and every other `#[serde(tag = ...)]` the SDK has, and it leaves `from_str` wrong so a reader would have had to go through a `Value`. The feature is global to a build, as `preserve_order` already is, and the defence is a test rather than a declaration. **Numbers: none moved, and the measured pages got sharper** — the corpus is JSON too and had been read a unit in the last place low, so several comparisons were measuring the parser rather than the SDK: the choghadiya, the horas and Brahma muhurta held to within 0.040 ms and now hold **exactly**, rahu kaal's worst went from 6.66e-9 to 4.55e-9 of an eighth, and the three clock-driven lagnas disagree on 24 of 71 fixtures where they had on 25. Next: the schema emitter, which is now the only thing left waiting. |
| 2026-09-08 (fifty-fourth session) | The canonical form's number grammar, fixed, and the parser finding that comes with it. **Numbers: every content hash moves; no computed value does.** The form now writes each number as the shortest decimal that reads back as the same double, where it wrote a fixed twelve. Twelve had been chosen on the stated grounds that "the SDK's own quantities are degrees, days and scores whose magnitudes are under 10⁶" — a premise the schema pass falsified, because a Julian day is 2.46 × 10⁶ and a chart document carries the instant it was cast for. At that magnitude an `f64` resolves about nine decimals, so three of the twelve were a binary value's decimal expansion rather than information and a parse did not return them, and a consumer that stored a document and hashed it did not get the producer's hash. The measurement had already been taken and not read: `serial-measured.md` recorded "every number of the corpus round-trips through the form" as falsified at 22 188 of 193 366, and the module shipped; it now holds at 0 of 193 366. What made it impossible to leave was the consequence rather than the count. `Digits::{Shortest,Rounded}` now says which of the two forms a caller wants — the hash form asks for no count at all, the rendering keeps `output.precision` — and the form is **exact** rather than lossy, which the design page argues is right: rounding never delivered the tolerance it promised (two values one ulp apart straddle a boundary some of the time), and "would these compute the same" is the settings hash's question, which does **not** move because no setting holds a number twelve decimals could not write. The three bindings still agree value for value on all 103. **A second finding, for the reader that comes next:** `serde_json`'s default number path is not correctly rounded — it reads `218.91170673806658` as the double one ulp below, and does that to about one number in fifteen — so a Rust reader built on it cannot reproduce the hash it exists to check, however correct the grammar is; `str::parse` is correct, as are JavaScript's `JSON.parse` and Python's `json`. The schema pass now measures both parsers over every number of every sample (0 moved by a correct parser, 84 of 1518 by serde_json's). Next: the reader, then the schema emitter it gates. |
| 2026-09-08 (fifty-third session) | The falsification pass for the chart document's JSON Schema (`cargo xtask schema` → `03-design/schema-measured.md`, gated by `check-schema`), and the three things it found before an emitter existed. The sample is **built rather than recorded** — `cargo run -p teistro-serial --example documents` founds a chart over the analytic provider and writes three shapes, widest to smallest, because a recorded sample goes stale the first time a section gains a field and the pass would not notice; `tests/document.rs` includes the example as a module rather than founding the same chart twice. All five proposed rules were falsified. **Where the schema comes from** is settled: the API description, beside the C header, the TypeScript surface, the Dart classes and the Python declarations. A sample cannot decide it — the canonical grammar removes a trailing zero on purpose, so a whole double is written as a JSON integer and 37 of 128 numeric paths are an integer in every document while one, `grahas[].latitude_deg`, is written both ways (a decimal for the Sun, `0` for the nodes), which proves the ambiguity rather than supposing it; and 93 of 98 string paths are catalogue members whose full lists only the description holds, because a sample proves a member exists and never that one does not. **Nothing in the layer reads back**: 65 types derive `Serialize` and none derives `Deserialize`, the mirror of the first pass's finding that `ChartFoundation` could not be serialised at all, and what makes the schema's natural gate — every sample validates and reads back equal — half unwritable. **The canonical form is not a fixed point once a document carries an instant**: the grammar writes twelve decimals, three past what an `f64` resolves at a Julian day's magnitude, so `2460483.108666389249` is written, read and written again as `2460483.108666389715`, and a consumer that stores a document and hashes it does not get the producer's hash — the one thing the form exists to guarantee. Survival is a coin toss per value, so the smallest sample holding (6 numbers past the resolution) does not make it safe and the widest (213) loses; `serial-measured.md` asserted the invariant over the corpus's recorded documents, whose numbers are all under 360. The fix moves the hash of every document, so it is a decision and not a patch, and it is recorded in `serial-and-the-envelope.md` §8 with the other two. Next: the round trip, then the number grammar, then the schema they gate. |
| 2026-09-08 (fifty-second session) | Six worked examples in each binding, and what writing them falsified. `bindings/{node,dart,python}/example/` hold the same six scenarios — the quickstart, a Nepali birth record placed by sign, nakshatra and pada, the five limbs of a panchanga, a Bikram Sambat year as a calendar page, a year of the sky in one call, an ephemeris of your own — and each binding's gate now runs **every** file in its example directory, so a scenario added is a scenario gated. Writing a real program three times is a falsification technique the project had not used, and it found nine gaps no gate could: the parity gate compares **values**, and these were **shapes**. Node had no examples at all, no `dispose`, no date or zone constructors, no `jdCount` or `bodyCount`, and spelled a provider's coverage `jdRange` where the other two say `jdMin`/`jdMax`; the TypeScript surface emitted an id table for two enums out of eighty-one; the intl accessor tree left its segments in snake case in a language that cases them camel; the Dart layer wrapped a provider's own exception in the library's; the Python layer's `weekday_of` docstring named the wrong day as one. The provider error contract was the deepest of them, and the three bindings disagreed three ways. `validate` moved to the SDK's side of the boundary (`VtableProvider::positions`), which is where it always had to be: only a code crosses back, so a refusal raised out in a binding arrived as a number with its sentence lost, and each binding had grown its own copy of the policy to get the words back. Checked on this side the words survive into every binding at once and no binding keeps a copy — `the provider does not support MARS; it answers SUN, MOON`, status `unsupported`, the same in all three. What a provider raises on its own side is now handed back to its caller as itself, with the library's refusal kept as its cause where the language has one. Coverage stays a per-cell `CellStatus::OutOfRange` as the port always said, because putting it in `validate` broke the test provider's own test and that test was right: a year whose last day runs past the ephemeris keeps the days it can compute. Every column a provider supplies is now held to the cell count, not only the three it must. Left behind and recorded under "How to resume": a body's key is spelled `SUN` by the port and `sun` by every generated enum, and refusing early on coverage is still each adapter's own choice. Next: a JSON Schema for the chart document. |
| 2026-09-08 (fifty-first session) | The Python binding, falsified, designed and built. The pass (`cargo xtask surface`, held by `check-surface`) is the first to measure the **API description** rather than the corpus: a binding was the thing being designed, and the corpus records charts and not calling conventions. It found `array_in` has no instance, so the emitter refuses that role rather than guessing at it; that the three targets' renaming rules fire in **different places** — Dart on the member `ChartKind::Return`, Python on `from` as a struct field and two parameters, TypeScript on nothing — so the word lists moved into one module (`teistro_idl::emit::reserved`) where they can be counted; that twelve of the twenty-five structs change size between targets and every one holds a pointer, a callback or a `size_t`, which is why the generated Python carries a table of both and a test asserts `ctypes.sizeof` against it; and **eighteen floating-point boundary fields with no unit**, now named, which reached the C header, the TypeScript surface, the Dart classes and the site's reference as bare numbers. The binding: `ctypes` over the same shared library (PyO3 was rejected with the other per-ecosystem tools in ADR-0004), no runtime dependency and no compiler; branded `float` subclasses rather than `NewType` stubs, because a subclass is both the distinct type a checker wants and the validating constructor `NewType` cannot have; `IntEnum` catalogues with `UNKNOWN` and a truthful `__bool__`, because `IntEnum` would otherwise make `Status.OK`, `Graha.SUN` and `Era.VIKRAMA` falsy; one copy of a result blob with zero-copy `memoryview` columns over it, which numpy wraps without copying; and `CFUNCTYPE` trampolines that catch everything, because an exception escaping a `ctypes` callback returns zero and the port reads that as success. 69 tests, the README's example run by the gate, six wrong usages a type checker must refuse, and the package strict-clean under mypy. Two gates caught one defect from opposite directions: `convert_time` took `TimeScale` where the boundary wants `Scale`, which mypy called a type error and the parity gate called three disagreeing values. `check-parity` now compares every binding present against the first, and the three agree on all 103. Next: a JSON Schema for the chart document. |
