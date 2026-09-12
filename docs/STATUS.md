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
sunrise for a Nepali place without a provider override. Of the
deliverables it deferred, **the completion's centre step was built on
2026-09-09** — it turned out to need nothing of Phase 3 — and the
corrections and equinox steps, and a heliocentric or barycentric centre,
still wait for the built-in ephemeris; eclipses wait for v1.x. Phase 1, Foundation, remains open:
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
**Last updated:** 2026-09-09, end of the sixty-fourth session (the
completion's topocentric centre, measured and then built. The design
page's own description of the step — "the observer's geocentric position
(WGS84) and the parallax" — is falsified by a residual that does not
shrink with distance: a third of an arcsecond on Saturn, whose whole
parallax is under an arcsecond, because the station's own motion
aberrates the light it receives. Two further terms are each worth
another third on the Moon and nothing on anything else, and only
together. The lunar nodes take none of it, because a direction is not a
place. The two shipped profiles that could not found a chart now do, and
`check-parity` compares 635 values where it compared 594); before that
the sixty-third session (the Indian
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
3. **Phase 3, the built-in ephemeris — milestone M3 — is all but
   closed** ([`07-roadmap/00-roadmap.md`](07-roadmap/00-roadmap.md); the
   tier ladder is
   [ADR-0021](08-decisions/adr-0021-reference-ephemeris-path.md)). Every
   body is built — VSOP87's planets, ELP2000-82B's Moon with its bija,
   Pluto fitted, the nodes and the mean apogee — at three tiers, each
   inside its size budget by an assertion that fails the build; the
   conformance kit passes at all three; the accuracy document is
   generated and gated per body, per span and per tier
   (`03-design/completion-measured.md`, `03-design/pluto-measured.md`);
   and the ephemeris now crosses the boundary, so a chart computes with
   nothing but the SDK installed **in every binding** rather than in Rust
   alone (ADR-0028).

   **Nothing is left of it.** The `reference` tier was the last open
   item and it was a choice rather than a task: ADR-0021 made it
   conditional on the fitter meeting 0.005 arcsecond in about 1 MB, the
   condition was evaluated on 2026-09-11, and the tier **moved to v1.x**
   because a fitted Moon alone measures 3.76 MB and would spend the whole
   budget on one body. The ladder, the target and the machinery all
   stand; Pluto already proved the machinery. Everything else that was
   outstanding is done — the tier
   matrix gates the kit at all three tiers in CI, the nakshatra boundary
   timing is published beside the tithi's, and the osculating apogee is
   built, which took the one constant neither theory carries
   (`GM_EARTH_MOON`, from IERS 2010 and DE440, checked against the mass
   ratio it implies). **The provider now computes every one of the port's
   fourteen bodies**, so the test that asked it to refuse one could not be
   written any more and asserts completeness instead.

   Two measured figures are recorded and **not** explained, which is the
   honest state rather than a gap to paper over: the true node disagrees
   by 118 arcseconds, traced to the two theories' orbital planes and not
   to truncation, the bija or the precession model; and the mean apogee
   by 417, where the mean node beside it is inside one. Both accounts are
   in `03-design/completion-measured.md` with the candidates named.

   **The next task is Phase 4's plugin and surface work** (ADR-0029,
   ADR-0030, researched in
   [`01-research/platform/15-provider-plugins-and-targets.md`](01-research/platform/15-provider-plugins-and-targets.md)).
   The maintainer's brief of 2026-09-11: an ephemeris is **plugged in**
   the way a transport is plugged into nodemailer, in some 98% of cases a
   real engine rather than the built-in, with the consumer reading
   `sdk.<area>.<operation>` and reaching the engine's own functions at
   `sdk.engine.*` where the SDK has not ported one.

   The survey found two things that reorder the work. **Outside Rust no
   consumer can reach an engine at all** — both adapters are `rlib`s with
   no C entry point and no package — so the 98% path does not exist in
   Node, Python or Dart. And **`sdk.engine.*` is built at the boundary and
   attached to nothing**: the Teimeris adapter has always described 161
   functions in `tools/idl/teimeris.idl` and has never answered the port's
   `native_manifest`, which its own source says is its remaining work.
   The second is smaller than it looks, because that IDL is typed to the
   parameter's role and the SDK's generators are role-driven, so a typed
   `sdk.engine.*` is generated rather than written — into the **adapter**,
   so the port stays agnostic.

   In order: the adapters gain `cdylib` and a vtable factory; the SDK
   gains one `ts_provider_load` rather than three; the Teimeris adapter
   answers the manifest; the façade generator runs; the surface is
   namespaced while it is still cheap. wasm's ephemeris is the built-in
   `compact` tier, which exists for it — a wasm module cannot load a
   shared library, and an engine compiled to wasm is a project rather
   than a packaging step.

   **The first four of those are built.** The adapters export a plugin
   (ADR-0029), the boundary loads one (`ts_provider_load`), and the
   Teimeris adapter answers `native_manifest` and `native_call` from
   marshalling generated out of its own IDL by `cargo xtask engine`,
   gated by `check-engine`. The coverage is a **measurement**, not a
   target: `03-design/engine-passthrough-measured.md` classifies all 161
   functions into what is callable today (62), what the adapter will
   never hand over whatever its shape (12, each of which opens, closes or
   rebinds the context every chart is cast on), and what is queued behind
   one more shape (87). Strings closed the way the page predicted they
   would: three shapes, one helper each, and the queue's largest
   non-struct group went with them.

   The page then falsified its own next sentence. It had said arrays were
   the next tranche and nearly free, on the strength of a table that
   grouped a function by the **first** unlearned role in parameter order.
   Grouped by the **hardest** one instead, 81 of the 87 are behind
   structs and an array of numbers would release three: 31 of the arrays
   are arrays *of structs*, which is the struct job wearing a count. So
   there is one tranche left and it is the large one, and the cheap thing
   the page recommended would have bought almost nothing.

   Growing that IDL was part of it. Three of the engine's public types —
   `tm_body`, `tm_flags`, `tm_ayanamsha` — are integers a caller passes
   like any other, and the description said nothing about them, so every
   generator that read it carried its own copy of the list and one copy
   had already gone wrong. They are described now, which is what makes
   the five functions that take one callable here.

   **What is left of Phase 4's surface work:** the namespacing, then the
   typed façade. Both were asked for explicitly, and the order is the
   other way round from how they were asked because the façade attaches
   to the `sdk.engine` namespace the other one creates.

   The namespacing's measurement is done and it falsified the rule ADR-0030
   words it by (`03-design/surface-areas-measured.md`, gated by
   `check-areas`). The ADR says the areas are "derived from the boundary
   modules the reference site already groups by"; measured, **5 of the 14
   boundary modules are never reached by anything a consumer calls** —
   they are the C caller's memory and the context's own life — and one
   member reaches two modules. So the grouping is real and the derivation
   is not: the areas have to be *chosen* with the measurement as
   evidence, which is what the design page is for. Two further findings:
   entry points do not measure an area's size to a consumer (`chart`,
   `positions` and `panchanga` have one each and are what the SDK exists
   for), and **6 of the members already spell their own area inside their
   own name** — `convertTime` because `convert` was taken by the
   calendar — so namespacing mostly gives those names back rather than
   inventing any.

   Running that pass found a hole beside it: the gate-coverage lint read
   `xtask`'s hand-written arms and so could not see the nineteen
   generated-page gates at all, four of which no workflow ran. It reads
   the pass table now, and `check-agreement`, `check-pluto`,
   `check-engine` and `check-areas` are wired.

   **The design is written and the first two steps of it are built**
   (`03-design/surface-areas.md`). Seven areas and a root, with one rule
   the measurement could not supply: *an operation whose name is its own
   area's name is a root operation and not an area of one*, which puts
   `positions` at the root and keeps `frame` as an area. `sdk.ephemeris`
   becomes `sdk.engine` (ADR-0030) and the area over `ts_panchanga_days`
   is `almanac`, taking the consumer's word over the module's.

   Built: the boundary's `keys.rs`/`strings.rs` renamed to match the
   functions they hold — the **files**, because a file is not an ABI
   symbol — and **the whole Node layer**, with its 39 tests, its eight
   examples, its typecheck at maximum strictness and its parity runner.
   `check-parity` proved that behaviour-preserving: all 635 values still
   agree with Dart and Python, which had not changed. Node's dynamic
   index signature came out, which ADR-0030 had considered and rejected
   under ADR-0023.

   The measured page turned over as Node landed, which was the point of
   gating it: it had measured a flat surface, and now holds five
   properties of the built grouping — a module reached from one area, no
   empty area, no operation spelling its own area, every module reached
   or accounted for as plumbing, and the boundary's own naming. Four
   hold; the fifth is the boundary naming, whose seven exceptions are six
   `lib` functions the rule does not cover and one declared in the design
   page.

   **Dart and Python followed**, each in its own idiom — a `late final`
   field and a `functools.cached_property`, both of which make an area a
   value a consumer can hold — and each green on its own gate. Python's
   `messages` stopped being eager along the way: it was built in
   `__init__` for every context whether anything rendered or not.

   And **`check-parity` gained the grouping**, which was the gap the
   measurement named: it held three bindings to the same *values* and to
   no *shape*, so a namespaced surface could have drifted into three
   groupings without the gate telling a deliberate difference from a
   mistake. Each runner now prints a `surface.<area>.<operation>` line
   per operation, keyed by a canonical path and referencing its own
   binding's spelling. 673 values agree across all three; proven red by
   moving `calendar.convert` to `time.convert_date` in one runner, which
   reported the extra key and the missing one in both comparisons.

   **Next, and it is the 98% path rather than a tidy-up:** a consumer
   outside Rust still cannot plug a real engine. The boundary has
   `ts_provider_load` and `ts_context_new_with_provider`; the Node layer
   even has a generated `Provider` class that loads an adapter. What is
   missing is the **other half** — no generated layer wraps
   `ts_context_new_with_provider`, so a loaded provider cannot be handed
   to a context, and no ergonomic layer offers a plugin path at all. So
   `sdk.engine.*` works in Node, Dart and Python against the *test*
   provider and against nothing else, which is the reverse of the
   maintainer's brief.

   It is a generator gap rather than a design one: the entry point takes
   a `handle` of an opaque type that is not `TsContext` beside a
   `handle_out`, and the three emitters have not been shown that shape.
   Teaching them, and then giving each ergonomic layer a plugin option
   (`new Context({ ephemeris: { plugin: …, config: … } })`), is what
   makes the 98% path exist outside Rust.

   **`check-lints` holds the class of gap rather than the instance.** Its
   ninth rule, `entry-point-is-reachable`, applies the emitters' own
   grouping rules to every function in the description and reports any
   the rules place nowhere — born red on exactly this one.

   **The generated half is now built.** `rules::factories` is the rule
   that was missing: a class has one constructor and may have more than
   one way in, and every other `handle_out` for a type is a **factory**.
   `build_call` in each emitter learned whose handle is `self`, so any
   other opaque's becomes a parameter of the class that holds it. All
   three now have it — `Context.newWithProvider(options, provider)` in
   Node, `TeistroContext.newWithProvider(lib, …)` in Dart,
   `TeistroContext._new_with_provider(lib, …)` in Python — and all four
   binding gates pass on the generated code. The lint's inventory is
   empty and the list is kept, so the next unplaced entry point has
   somewhere to be declared.

   **And the ergonomic half is built, so the 98% path exists in every
   binding.** Two options in each layer, spelled the same in all three:
   `plugin`, the adapter's platform binary, and `pluginConfig` /
   `plugin_config`, that adapter's own options — handed over as JSON and
   read by the SDK not at all. `plugin`, `provider` and `ephemeris` each
   answer the same question, so two together is a refusal rather than one
   silently winning, which is the rule the settings patch already had.
   The loaded handle is freed at once in each layer: the context takes
   its own reference, so what keeps the library loaded is the context and
   a consumer holds neither.

   Proven end to end, three times, against a real Teimeris built as a
   `cdylib`: the Sun at J2000 at **280.3689°** from Node, Dart and
   Python alike, `sdk.engine` reporting 62 operations, and
   `tm_body_name(0)` answering `"Sun"` — a string crossing the plugin
   boundary, through the generated marshalling, into three languages.
   Each binding has a test for it that skips with a printed reason where
   the adapter is not built, which is what `crates/ffi/tests/abi.rs`
   already did with the same `TEISTRO_TEIMERIS_ADAPTER`.

   **The typed façade is generated too**, from the same reading that
   writes the dispatch and the page, so it cannot type an argument the
   dispatch would refuse by name. Four files under
   `adapters/ephemeris-teimeris/` — `node/engine.js` with its `.d.ts`, a
   Dart extension, a Python class — each with a method per callable
   operation, named the way its language names things while the keys that
   cross stay the engine's own.

   Proven against the real engine in all three: `tmBodyName({ body: 0 })`
   → `"Sun"`, `tmDeltaT({ jdUt1: 2451545 })` → `63.8289…`,
   `tmVersion()` → a record, `tmAngleFormat(…)` → `5 Tau 30'00"`.

   Two things it corrected. **ADR-0030's façade shape was wrong** and is
   amended: a TypeScript declaration merge and a Python `Protocol` would
   each have promised methods nothing installs — with the index signature
   gone, `sdk.engine.tmBodyName` is `undefined` however well it
   type-checks — so the façade is a **value a consumer takes**
   (`teimeris(sdk.engine)`), except in Dart where an extension method has
   a body and is therefore both typed and installed. And the shape of an
   answer is a **measurement**: 46 of the 62 answer with exactly one
   value, so a method hands that value back as itself and only the five
   that answer with more get a record — five types per target rather than
   sixty-two. The same count showed `return` never appears beside another
   key, so no target has to rename a keyword.

   **And the packages are built, so the façade is gated by a compiler
   rather than proven by hand.** `@teistro/ephemeris-teimeris`,
   `teistro_ephemeris_teimeris` for Dart and pub, and the Python
   distribution of the same name: each exports `teimeris(…)` — the
   descriptor, with the platform binary filled in — and carries the
   generated façade. Each resolves its binary the way the SDK resolves
   its own: a named path, `TEISTRO_TEIMERIS_ADAPTER`, the per-platform
   package, then this repository's release and debug builds, with a
   refusal naming every place it looked.

   Each binding's gate now checks its adapter package too, because what
   an adapter package needs is that ecosystem's checker at that
   ecosystem's strictness: `tsc` at maximum strictness in `check-node`,
   `dart analyze --fatal-infos` in `check-dart`, `mypy --strict` in
   `check-python`. **Proven red**: a string where the engine declares
   `tm_body` is a type error, which traces back through the façade and
   the manifest to the integer aliases the engine's IDL gained earlier in
   this session.

   **The licence, as the maintainer decided.** Each package declares
   `AGPL-3.0-only` and carries the licence text, because the artefact it
   ships links Teimeris and an Apache-2.0 library that links AGPL code is
   an AGPL work. Both adapter crates keep `license = "Apache-2.0"` for
   their own source, and each says so in its manifest where a reader
   meets the apparent contradiction.

   The binary resolver paid for itself on its first outing: it found a
   stale local *release* build ahead of the debug one, and the boundary
   refused it by name — `vtable size 72 version 2; this port is version
   3` — which is the ABI check ADR-0029 asked for, working.

   **And the plugin surface is ADR-0029's**, after one commit where it
   was not. `plugin` and `pluginConfig` — a path, which that ADR rejected
   for four stated reasons, and no chain — are folded into one option:
   `ephemeris` takes an entry or an **ordered chain** of them, tried in
   order. An entry is a name of the SDK's own or an adapter's descriptor,
   spelled each language's way: a plain object in Node, a `Plugin`
   dataclass in Python, a **sealed** `EphemerisChoice` in Dart so the
   switch that opens one is exhaustive.

   **A chain of one is not a chain**, and that was a defect before it was
   a rule. The first version caught every refusal to try the next entry,
   so a bad *profile* — not an ephemeris failure, and identical on every
   entry — came back as "nothing could be opened" instead of a refusal
   carrying its status, its field and its hint. **An existing Dart test
   caught it**, which is the memory's own rule earning itself again: a
   refactor that reddens a test has found either a bug or a contract.
   With one entry nothing is caught; with more, every refusal is kept and
   reported together.

   **`check-package` covers the adapter now**, which is the half of
   ADR-0029's packaging cost that does not need a publishing pipeline.
   The npm package is packed, installed into a throwaway project beside
   the SDK's own tarballs, and a consumer that knows nothing but the
   published names is run against it — which is the only thing that tests
   the *package* rather than the code in it. It skips, saying which it
   wanted, without the adapter's library or the engine's data.

   It earned itself on its first run: the descriptor sent `data_dir`
   where the adapter's own `Config` is `#[serde(rename_all =
   "camelCase", deny_unknown_fields)]` and wants `dataDir`. All three
   packages had it wrong, and nothing else could have caught it — the SDK
   passes that object through as JSON and reads none of it, and every
   earlier probe had called `teimeris()` with no options at all.

   **A red nightly that predates all of this, and took four attempts.**
   The verify matrix was dispatched on the branch to validate the three
   new adapter gates on clean runners, and showed the C binding failing
   on win32 and `check-package`'s C consumer on both Linuxes — on `main`
   too, since at least 2026-09-10. `undefined reference to `sin``: the
   astronomy calls it, Linux and MinGW keep the maths functions in a
   separate `libm`, and `bindings/c/README.md` told a consumer to link
   without it. macOS has them in libSystem, so every local run passed.
   One constant, `-lm`, read by both gates and stated on that page —
   and **three of the four failing platforms went green**.

   Then two more attempts chased win32 with flags, and both broke
   platforms that had been passing. The log said why, once it was read
   rather than reasoned about.

   **It was never a flag.** `check-c` links `-L target/release
   -lteistro_ffi`, and on Windows a searching linker finds
   `teistro_ffi.lib` — the **static** library — beside
   `teistro_ffi.dll.lib`, and takes it. So the gate was statically
   linking Rust's whole standard library, built for the MSVC ABI, with
   the runner's MinGW gcc: `__chkstk`, `__imp_NtReadFile`, and
   `??_7type_info@@6B@`, which lives in the MSVC C++ runtime MinGW does
   not have. No `-l` closes that; the two ABIs do not meet.

   And `rustc --print native-static-libs` — the third attempt, which
   looked principled — **cannot be pasted into an arbitrary C driver**.
   Its dialect is the Rust target's linker's: on win32 it answers
   `kernel32.lib` and `/defaultlib:msvcrt`, which MinGW's ld looks for
   as file names and does not find; on Linux its `-lc` gives `cannot
   find -lc`; on darwin its `-lc -lm -liconv -lSystem` gives `library
   'm' not found` where `-lm` alone had linked. It is information for a
   consumer who knows their own toolchain, not a link line, and
   `bindings/c/README.md` now says to read it rather than paste it.

   So the two questions are answered apart. `binding::shared_link` says
   how to point a C compiler at the **shared** library — `-L <dir>
   -lteistro_ffi` on Unix, the import library **by path** on Windows,
   and nothing else on either, because a shared library resolves its own
   imports. `binding::static_link` says what to link beside the
   **static** one (`-lm`), or refuses with the reason this platform's
   `cc` cannot link it at all. `check-package` prints that reason for
   the Windows static library instead of failing on it, and proves the
   import library there, so the gap narrows from "the C step skips on
   win32" to "the win32 static library wants `cl`".

   Two tests over the platform table hold both, and the first is the one
   that would have caught this on the first attempt: a platform that
   ships an import library must link through it, because `-l` searches
   and searching is the defect. Proved red by putting `-l` back.

   **And win32 had a second defect hiding behind the first.**
   `check-python` had never run there, because `check-c` failed in the
   same job before it; with the link fixed it ran, and
   `UnicodeEncodeError: 'charmap' codec can't encode characters in
   position 0-5` — those six characters being `सोमबार`, the weekday
   `almanac.py` prints first. The Windows console's default encoding is
   cp1252. PEP 540's UTF-8 mode is the answer and becomes Python's
   default in 3.15, so the gate sets `PYTHONUTF8` and the Python README
   tells a Windows reader to set it too.

   **Fixed in one gate when four needed it**, and the next run said so:
   `check-parity`'s Python runner failed with the same error on `\u2609`,
   the Sun. Four gates start a Python process and each answered "which
   interpreter" for itself; `binding::python_command` answers both
   questions once, and `check-lints`'s tenth rule,
   `python-runs-in-utf8-mode`, holds the class — the interpreter is named
   in one place and no module builds a Python command of its own. Proved
   red by putting the parity runner's own `Command::new` back.

   **The docs had the same class of defect as the code: they described a
   surface nobody had run.** The site's install page gave a Node
   quickstart that refuses — `the context has no ephemeris` — because
   `ephemeris` has no default and the page named none; it had no Python
   section at all, though that binding has been gated since the 8th; and
   its C link line was the line that cannot link. All three are fixed,
   and the page gained a **Choosing an ephemeris** section: the three
   things an entry can be, the ordered chain, all three languages, and
   why an adapter is its own package rather than a build flag.

   And twelve of the fifteen example programs still selected the
   **test** provider, three of them claiming in prose that it "selects
   the analytic ephemeris the SDK carries". It does not: `builtin` is
   that, and `test` is one periodic term per body. Every example names
   the built-in now, and `ephemeris.{mjs,dart,py}` gained the retrograde
   scan it had carried as a hypothetical comment — `stations`, the same
   shape over `lonSpeed` that `ingresses` is over `lon`. Measured, and
   the three bindings agree: Mars retrograde at -0.3281°/day, one
   station, day 55 from 2025-01-01, which is when it turned.

   The refusal a consumer meets when they forget the option was written
   for a C caller and duplicated three times — *pass a provider vtable
   to `ts_context_new`, or the `TS_CONTEXT_TEST_PROVIDER` flag for
   tests*, which a Node, Dart or Python consumer can act on in neither
   half. One `support::no_ephemeris` now, naming the `ephemeris` option
   and hinting at the three kinds of answer it takes.

   **And one more hole, in the gate that holds the three bindings to one
   shape.** `check-parity` compares what the three runners *print*, and
   the list of canonical `surface.<area>.<operation>` paths is written
   out once per runner in three languages — so three runners that all
   miss the same new operation agree perfectly and the gate is silent.
   `check-areas` reads all three lists against the layer's own
   declarations now, as a sixth property, and it was **born red**:
   `(root).engine`, the accessor a consumer reads to reach the engine at
   all, was listed by none of them. 117 pairs, 39 operations by three
   runners, and it holds.

   **win32 kept giving up one defect per run, and each was real.** With
   the link fixed, `check-c`, `check-node`, `check-dart`, `check-python`
   and `check-parity` all pass there. `check-package` then found two
   more, both Windows facts hard-coded as Unix ones: a Python virtual
   environment puts its executables in `Scripts` and not `bin`, so
   `pip` was never where the gate looked — now a row of the platform
   table like every other name an operating system decides; and **npm,
   npx and tsc are `.cmd` shims** there, which `Command::new` cannot
   find because `CreateProcess` appends `.exe` and nothing else. So a
   runner with npm installed answered "no `npm` on this machine" and the
   gate skipped the Node packages on the platform whose packaging is
   least like the others'. `binding::tool` resolves a tool's spelling
   once; a skip that says the machine lacks a tool it has is worse than
   a failure.

   **And the TypeScript type-check had never run on Windows**, which the
   npm fix revealed rather than caused: with `npx.cmd` resolvable the
   gate stopped skipping and failed, on `The system cannot find the path
   specified.` — a message naming no tool, in a step that printed only
   what it wanted. Two changes, and the second is the one that matters:
   `step` now names the program on its failure line, and the compiler is
   run as what it is. `tsc` is a JavaScript program; `node
   typescript/bin/tsc` is the same file on every platform, so that path
   has no `.cmd` shim in it at all.

   And it is **pinned**: `bindings/node/typecheck/package.json` names the
   version, the gate installs it from the lock file on first run the way
   `check-site` does with its own, and `TSC` still overrides. Before
   this, every machine type-checked with whatever compiler it happened
   to have and every runner with whatever its image carried — which is
   why nobody had noticed that on four of the five platforms the gate
   was skipping. Adding the manifest reddened the gate at once, and
   correctly: it became the nearest `package.json` to those files, so
   they stopped being ES modules until it said `"type": "module"`.

   **And the matrix could never have gone green anyway.** Every
   dispatch of `verify.yml` today — eleven of them — was still in
   `queued`, and the one run from the 11th "completed" only because it
   had been cancelled. One row did it: `bindings (darwin-x64)` asked for
   `macos-13`, GitHub retired that image, and **a retired label does not
   fail, it queues**. So the run never reached a conclusion, and every
   judgement of "green" today was made from the four rows that do run.
   It is the same defect as a gate that skips while claiming the machine
   lacks a tool it has: silence read as the absence of a problem.

   `macos-15-intel` is the image that replaced it, and the fix was the
   experiment — a wrong label fails a job at once rather than hanging,
   so a dispatch answers either way. It answered, and then the whole
   matrix did: **run 34688535483 is the first `verify` run to reach a
   conclusion, and the conclusion is `success`** — all five binding
   platforms, including the Intel macOS row that had never run and the
   win32 row that had never passed, and all three ephemeris tiers.

   Three places had to change, and `xtask/src/platform.rs` opens by
   saying it is "the only place any of that is written". It was not: both
   workflows kept their own copy of which runner builds which platform.
   `check-lints`'s **eleventh** rule,
   `runner-matches-the-platform-table`, holds every platform row of
   every workflow matrix to that field — born red on the one row this
   correction had reached, which is how a rule should arrive.

   The Python type checker was the same story one step behind: `mypy`
   installed by a workflow step, unpinned, so a release of it could
   redden CI on a day nothing here changed and a machine with an older
   one would quietly check less. It is pinned in
   `bindings/python/typecheck/requirements.txt`, the gate installs it
   into `bindings/python/.venv` when it finds none — proved by moving
   that directory away and watching the gate rebuild it — and the
   workflow step is gone, because a gate that needs a tool should get
   it rather than rely on the caller having read a list.

   **And the site now says what the API looks like**, which it did not.
   Between the install page and a generated reference of C entry points
   there was nothing telling a Node, Dart or Python consumer the shape of
   the thing they had installed — the seven areas and the root, built
   earlier today, appeared only in the binding READMEs.
   [`site/content/docs/surface.mdx`](../site/content/docs/surface.mdx) is
   that page: what each area answers, why `ctx.calendar.convert` and
   `ctx.time.convert` can both be `convert`, why `positions` is at the
   root, that an area is a value a consumer can hold, and what
   `sdk.engine` is for with the adapter's façade beside it.

   Prose by hand and facts gated, which is the split that matters: the
   measured page's **seventh** property reads the guide's table of areas
   against the areas the layer actually wires, in both directions — an
   area the layer wires and the page does not name is a surface nobody
   can find, and an area the page names and the layer does not wire is a
   page describing something that is not there. Proved red by renaming
   `almanac` to `panchanga` on the page, which reported both halves.
   `check-site` requires the page to render, as it already did for the
   install page.

   **Next: the publishing half of packaging**, and it was surveyed
   rather than started, because the survey moved the cost twice — once
   down, once onto a decision that is not this assistant's to take.

   The deliverable is ADR-0029's: a package per adapter per target
   triple, each shipping the platform binary its host needs, so a
   consumer installs rather than builds. Everything above resolves a
   binary a contributor built on their own machine.

   **What the survey found, in the order it matters:**

   - **Nothing outside this machine can build the adapter at all.** Its
     manifest reads `teimeris = { path = "../../../../teimeris/..." }`
     — a sibling checkout, four levels above the repository root. No CI
     runner and no contributor has it, which is why
     `check-package`'s adapter step and every plugin test skip
     everywhere but here. The 98% path is *built*, and it is proven on
     one machine.
   - **The engine does not need cross-compiled prebuilts**, which was
     the feared cost. `teimeris-sys`'s `build.rs` resolves an archive
     from `TEIMERIS_LIB_DIR`, then `vendor/<target>/`, then a
     development checkout — **and compiles the C core from source when
     none of them has one**, with `core/` and `data/` travelling in the
     `.crate` for exactly that case. So every target builds itself.
   - **And the default tier is self-contained.** `teimeris-sys` embeds
     one tier's ephemeris data into the library at build time — about
     2.05 MB, the default — so an adapter package built that way needs
     no `dataDir` and no licensed data directory at install time. Any
     other tier needs `TM_EPHE_DIR` against a full ephemeris.

   So the whole of it reduces to **one question that is the
   maintainer's**: how the SDK's CI obtains the Teimeris source. A git
   dependency pinned to a tag, a submodule beside `fixtures/`, or
   vendored `.crate` files — the first two keep AGPL out of this
   repository's tree and out of the workspace `cargo deny` reads, and
   the third does not. Nothing about acquisition changes what the
   artefact is licensed as: it links Teimeris either way, which is why
   the packages already declare `AGPL-3.0-only`.

   **And it is blocked on two commits.** `teispace/teimeris` has
   `51b4345` and `6df3867` unpushed — the public integer typedefs the
   engine façade is generated from, and the context reachability the
   adapter's passthrough needs. Whatever mechanism CI uses, it can only
   fetch what has been pushed.

   Then wasm, whose ephemeris is the built-in `compact` tier.

   **And Rust's own consumer surface has had its measurement taken**,
   which is the step the project's own order asks for before the design
   page ADR-0030 §9 defers it to (`cargo xtask rust-surface` →
   `03-design/rust-consumer-surface-measured.md`, gated by
   `check-rust-surface`). The obvious proposal is that Rust mirrors the
   other three — eight areas and a root over one context — and what it
   is worth depends on a question nobody had asked: how far is a Rust
   consumer from it today?

   Measured by composing two readings the repository already had: the
   boundary's description says which module each entry point came from,
   the Node layer says which entry points each area reaches, and each
   boundary module names the SDK crates it calls.

   - **A context and its areas need 9 of the SDK's crates**, and the
     proposal that an area's operations come from one crate is
     **falsified 7 of 8 times**: `chart` needs six, `almanac` five,
     `(root)`'s `positions` four.
   - **Two of those nine are held by `crates/ffi` and by nothing
     else** — `teistro-intl` and `teistro-ephemeris-builtin` — so the
     composition that makes a context is written once, inside the crate
     whose whole purpose is the C ABI. Precisely, because the claim is
     load-bearing and was checked rather than assumed: `TsContext::build`
     is `pub`, and so are `settings`, `profile`, `provider` and `intl`,
     so a Rust consumer *can* reach the composition. What they cannot
     reach is an **operation** — converting a date is
     `ts_calendar_convert` with three raw pointers. **Rust today has a
     context it can build and cannot use**, which is the sharper
     statement of the gap and points at the same façade.
   - **23 of the 46 entry points reach two or more crates**, so a façade
     over them would be composition rather than a rename; 8 reach none
     at all and are the C caller's memory. The widest are
     `ts_panchanga_days` at seven and `ts_chart_found` at six; the six
     calendar operations are a clean two apiece.
   - And it would be **smaller** than the boundary, not larger: the
     eight that reach nothing are its memory and its handshake, which a
     Rust consumer does not have.

   The per-entry-point figures are the second reading. The first
   attributed a whole module's imports to each of its entry points,
   which is honest but coarse, and sharpening it found its own two
   defects — checked against the source rather than believed. Resolving
   calls by **bare name** across every module made a date conversion
   reach the chart and `intl` reach `teistro-panchanga`; keeping a path
   like `Place::new` whole as one word made every associated function
   invisible, so `ts_chart_found` came out not reaching `teistro-chart`
   at all. Calls are resolved through `use crate::<module>::…` and
   `crate::<module>::<name>` now, and methods are followed because
   `TsContext::new` *is* the assembly. The two readings agree where they
   overlap, which is the cross-check that makes either believable.

   Leaving `context` and `provider` out understated it — the areas are
   operations *on* a context, and building one is where the settings,
   the locale and the ephemeris are composed — so the page counts those
   two modules too and says why.

   **And the design page is written**
   (`03-design/rust-consumer-surface.md`), which settles ADR-0030 §9.
   Four decisions, each from the measurement rather than from taste:

   - **The façade composes the crates; it does not call the SDK's own C
     ABI.** The cheap alternative — wrap `teistro-ffi` as the other
     three bindings do — is refused on the third result: a Rust consumer
     would write a request struct so the boundary could decode it, and
     decode a blob to read numbers the crates already returned as
     `JulianDay` and `Longitude`. Rust would be the only binding whose
     implementation language is its consumption language and which still
     crossed a C ABI to reach itself.
   - **So the composition moves into the façade and `teistro-ffi`
     depends on it**, keeping only what is its own: the handles, the
     `struct_size` handshake, the blob writer, the panic guard, the
     `last_error` slot. The composition gets one home, and the home is
     the one both a Rust consumer and the C boundary can reach.
   - **That is checkable, which is why it is written this way round.**
     The measured page's *no crate a context needs is brought in by the
     boundary alone* is falsified 2 of 9 today and **holds** when the
     façade holds those two crates. The pass that measured the gap is
     the acceptance test for closing it — a pass whose subject you are
     changing turning over when the change lands.
   - **One `Context`, a builder, and an area as a borrowing view.**
     `sdk.calendar().convert(…)`: an area is a value in every other
     binding, and the Rust equivalent of a value you can hold is
     `Calendar<'a>(&'a Context)` — allocating nothing, unable to outlive
     its context. A builder rather than an options struct because
     `build()` can report the settings it resolved, which is the "no dead
     ends" brief's *what was applied is reported*. No `dispose`: `Drop`
     is the whole of it, and that is the first place the Rust surface is
     smaller than the C one.

   **And step 1 is begun, and the building corrected the page twice.**
   `crates/sdk`, the crate `teistro`, carries the context, the builder,
   the ephemeris chain and the **calendar** area — six operations, the
   ones `surface-areas.md` puts there and no others, because
   `check-areas` holds every binding's list to the same canonical paths.
   Eleven tests, and the ones that matter assert what
   `bindings/c/tests/smoke.c` asserts in the same words: 14 April 2015
   is 1 Baisakh 2072 BS, in the Vikrama era, inside the official table.

   **Clippy found a design gap before a test could.** It reported that
   the function opening a chain entry returned a `Result` that could not
   be an error — and it could not, because in Rust an entry as designed
   was an already-built `Box<dyn EphemerisProvider>`, which cannot fail
   to open. So every Rust chain succeeded on its first entry and the
   ordering was decoration, where in the other three an entry is a
   *description* whose opening can fail. `Ephemeris::opening(name, f)`
   is the fourth kind: a **recipe**, which is what a chain can fall back
   from. `Provider(p)` stays for the case a consumer already has one,
   and the two are not a second spelling — they answer *here is an
   ephemeris* and *here is how to get one, which may fail*, which is
   `rules::factories` again: one constructor, more than one way in.

   **And the acceptance test had to be sharpened, because it went green
   for the wrong reason.** *No crate a context needs is brought in by
   the boundary alone* flipped to holding the moment the façade declared
   a dependency on `teistro-intl` — before a line of composition had
   moved. It is two properties now, and both are honest: *the façade
   owns the composition* (falsified 3 of 9 — `teistro-chart`,
   `teistro-panchanga` and `teistro-time`, which are exactly the three
   areas not yet built) and *the boundary is inverted onto it*
   (falsified, 1 of 1). Both flip as the work lands, and neither can
   flip without it.

   **And a third correction, from a test that would not compile.** A
   `Context` is neither `Send` nor `Sync`, and the part that decides it
   is not the one a reader would guess: the port requires `Send + Sync`
   of an ephemeris, and it is the **locale engine** whose plural rules
   hold an `icu_plurals::PluralRules` with an `Rc`-backed payload. Every
   binding already says *one context serves one thread, and a worker
   builds its own*, so this is the stated rule enforced rather than a new
   limit — but it lands differently in Rust, where the obvious shape for
   a server is one context in an `axum` app state behind a `&`. The test
   builds one per worker, which is the pattern it leaves, and the design
   page's §9 records what lifting it would take: `icu_provider` has a
   `sync` feature (checked, not guessed) that moves those payloads to
   `Arc`, at the cost of atomic reference counts on every render in every
   binding — so that decision wants a falsification pass in front of it,
   over the message set `check-intl` already walks.

   **And `time` followed**, which is the first area that is not only a
   rename: `resolve`, `civil_of`, `convert` and `delta_t`, with the C
   smoke test's own facts asserted here too — 00:20 on 1 January 1986 in
   Kathmandu is +05:45 under the zone's current rules with no warning,
   and ΔT at J2000 is about 64 seconds. Reading the boundary for it
   found a rule the design page had not stated: **the surface owns a
   type exactly where an operation is dynamic and the crates are
   static.** `time.convert(jd, from, to)` names its scales at run time,
   and the crates keep a scale in the *type system* —
   `JulianDay<Ut1>`, `<Tt>`, `<Utc>` — so there is nothing to
   re-export; `crates/ffi` invented a `TsScale` and a private `Applied`
   for the same reason. Both are the façade's now, so the boundary can
   convert *from* them rather than there being a third copy. And
   `Conversion` answers with what was applied as well as the number,
   because 63.8 seconds from one ΔT model is not the same answer as 63.8
   from another. The acceptance test moved with it: 2 of 9 rather than 3,
   `teistro-chart` and `teistro-panchanga` left.

   **Five of the eight areas are built now**: `calendar`, `time`,
   `intl`, `keys` and `frame`, over nineteen tests. `intl` is where the
   second design note earned itself — its `messages` is a **module**
   tree in Rust rather than a method, `messages::sdk::reason::GrahaInBhava
   { graha, bhava }` handed to `render_typed`, which is what a namespace
   is in this language and which `teistro-intl` already generates. The
   test renders it and gets `गुरु`. What is left is the three that need
   the ephemeris — `chart`, `almanac` and the root's `positions` — which
   is where this surface stops composing calendars and starts computing,
   and which is why the acceptance test still names `teistro-chart` and
   `teistro-panchanga`.

   **And `positions` at the root**, which is the first operation on this
   surface that computes rather than composes:
   `Completion::new(provider, overrides, delta_t).positions(&request)`,
   answering with the astronomy crate's own `Completed` — a Rust
   consumer reads a longitude off `sky.columns.at(0, 0)` where every
   other binding decodes a result blob for the same number. The test
   gets the Sun at J2000 near 280° from the built-in, and a context
   without an ephemeris refuses with the same field and hint the C
   boundary gives. Twenty-one tests.

   It also found a gap by failing to be convenient: the test reached
   past the crate for `Body`, `Frame` and `PositionRequest`, which is
   exactly the five-dependency problem this crate exists to stop, so
   they are re-exported. One dependency is the point of a façade.

   **`engine` too**, which makes six of the eight areas and the root's
   `positions`: one reading of the provider's manifest behind two
   refusals — no ephemeris at all, and an ephemeris that describes no
   operations of its own, which is exactly what the built-in is, so a
   consumer on the fallback is told which they have rather than handed
   an empty manifest. Twenty-two tests.

   **And the order of work reverses for the last two, because of an
   oracle.** Every area so far could be checked against a fact the C
   smoke test already asserts — a Bikram Sambat date, a Kathmandu
   offset, the Sun near 280°. A chart's lagna, day lagna, ayanamsha
   offset and day part are asserted by no smoke test; what holds them is
   the **parity report**, where the three bindings agree on them value
   by value. So lifting that composition without the fourth runner would
   be writing numbers with nothing to check them against.

   And the runner cannot join `check-parity` as it stands, for **two**
   reasons rather than one. The gate compares key sets, so a report
   missing `chart.found` fails rather than saying "not yet" — and a
   *complete* Rust report still will not match, because among the ~674
   keys the three print are `abi`, `build-sdk`, `build-commit`,
   `build-target` and the result blobs' sections and hashes. A Rust
   consumer has none of those, by design: Cargo resolved the versions,
   `Drop` freed the memory, and the crates handed back their own types
   instead of a blob. **The absences are the design working**, so the
   gate has to hold the Rust report to every key the others have *except
   a declared list* — the `knob-has-a-reader` shape, an inventory
   printed every run so an absence that stops being deliberate becomes a
   stale allowance and therefore a failure. The list is the design's §6,
   already written.

   So: write the runner, run it by hand against the other three for the
   operations that exist, build `chart` and `almanac` against it, teach
   `check-parity` the declared absences, and wire it in.

   **The runner is written, and the first step of that is done.**
   `crates/sdk/examples/parity.rs` prints 84 keys; **every one of them is
   a key the Node runner prints, and every value is identical.** That is
   what proves this composition equal to the one at the C boundary
   rather than merely compiling — the settings hash, the Bikram Sambat
   date with its era and resolution, the Kathmandu offset and tzdb
   version, ΔT and its source and model, the key round trip, the refusal
   with its detail and hint, six position cells with their longitudes,
   latitudes, distances, speeds and statuses, the steps applied, the
   Nepali render's hash and length, the Sun's four forms, and the frame's
   bits.

   Getting there took three formatting agreements the diff found rather
   than a reading did: a resolution's kind is its JSON tag (`tabular`), a
   cell's status is the **id** the boundary carries and not its name, and
   a step's implementation is `PASS_THROUGH` where the astronomy crate's
   own `step_keys` gives `PassThrough`. Each was one line, and each would
   have been a false disagreement in the gate.

   **All eight areas and the root are built, and the parity runner is
   what says so.** `chart` and `almanac` came last and after the runner,
   which reversed the design's steps 1 and 2 for them — and that was the
   right order: with the oracle in place both compositions were
   checkable line by line, and both agreed on their first run. **125
   keys, every one of them a key the Node runner prints, every value
   identical**: the settings hash, the Bikram Sambat date, the Kathmandu
   offset, ΔT, the key round trip, six position cells, two charts' lagna
   and day lagna and ayanamsha offset and day part, three almanac days'
   varas and sunrises and windows, the Nepali render, the Sun's forms
   and the frame's bits.

   **So the acceptance test's first half has flipped.** *The façade owns
   the composition: every crate a context needs is one it depends on* is
   **holds**, 0 of 9 — where it was falsified 3 of 9 when the crate was
   a context and a calendar. The second half stands: `teistro-ffi` does
   not depend on `teistro` yet, so the composition is written twice,
   knowingly, until step 3.

   Twenty-four tests beside the runner, and the two newest assert the
   properties the numbers cannot: a batch of one takes the same path as
   the batch (bit for bit), consecutive almanac days share a boundary —
   day n's next sunrise is day n+1's sunrise, which is why a run costs
   less than the days apart — and the envelope carries a content hash
   rather than the founder's placeholder, which is what the C boundary
   fills and the first place `serial-and-the-envelope.md` §8's open
   question showed.

   **And the inversion landed, so the design is built.** `teistro-ffi`
   depends on `teistro`; `TsContext` wraps a `teistro::Context`; and
   `TsContext::build` — which resolved the profile, parsed the patch,
   loaded the embedded bundles, started the locale engine, read the ΔT
   knob and wrapped the provider in its cache — is a call into the
   builder. **Both of the measured page's last two properties hold**:
   *the façade owns the composition* (0 of 8) and *the boundary is
   inverted onto it* (0 of 1).

   It was third in the order for a reason and the reason held: with the
   façade built and the parity runner agreeing, the inversion was
   checkable rather than hopeful. **Every one of the boundary's 30 unit
   tests and 9 ABI tests passed unchanged**, as did `check-c`,
   `check-node`, `check-dart`, `check-python`, `check-ffi`,
   `check-lints`, `cargo deny`, and `check-parity`'s 674 values across
   three bindings.

   Four things moved out of the boundary with the composition:
   `own_provider` became `own_ephemeris`, answering the façade's
   `Ephemeris` rather than a boxed provider; `remembering` went with the
   `provider.cache_cells` knob it reads, and its three tests with it;
   `builtin()` became a variant selector, because the façade gates the
   *variant* on the feature while a C caller passing a number still
   needs the run-time refusal; and the locale bundles' build script, so
   the boundary's build script is `build_info` alone and has no build
   dependency at all.

   What stayed is what only a C caller needs — the handles, the
   `struct_size` handshake, the blob writer, the panic guard, the
   `last_error` slot — and `teistro-intl`, because the boundary still
   **marshals** the engine's types even though it no longer composes it.
   The composition moved; the types are shared. One accidental deletion
   along the way took `ts_context_new` with it, caught by four
   "unused import" warnings naming things only that function used.

   **Confirmed on every platform**: the verify matrix passed on the
   inversion — all five binding platforms and all three ephemeris
   tiers — which is the C boundary, all four bindings, the parity gate
   and the packaging gate all working through a composition that now
   lives in the façade. The one thing that did fail was `check-ffi`, and
   it was right to: the note I put on `TsContext` explaining *why* the
   composition had moved was a `///` comment, and `cargo xtask gen ffi`
   extracts those into `idl/api.json`, `teistro.h` and three bindings'
   declarations — so a C consumer opening the header met this
   repository's changelog, with a date and a design page's section
   number in it. In `crates/ffi`, `///` is a public document in five
   languages and `//` is a note to us.

   **And `check-parity` runs four runners now**, so the 125-key
   agreement is a gate rather than a snapshot: *4 bindings agree, value
   for value*, with `Rust does not print 549 of Node's keys` printed
   beside it on every run. Proved red by adding one to a day of the
   month.

   The 549 are a **count and not a declared list**, deliberately: a list
   would have to name all of them, and most are not the boundary-only
   keys §6 accounts for — they are detail the Rust runner does not print
   yet, a graha at a time and a muhurta at a time. Completing it is
   mechanical, and the count is what says how much is left. The number
   only going down is the property a reader can watch.

   **And it went down: 549 → 155, in four passes, with every added key
   agreeing on its first run.** The runner prints 519 keys where it
   printed 125 — every graha's longitude, latitude, speed, retrograde
   flag, house and placement in both charts; all twelve bhava madhyas
   and sandhis of each; every tithi, nakshatra, yoga and karana span of
   three days with its member and both bounds; each day's lunar month
   and convention and ayana and disha shool; and the values a day may
   *not* have, printed `none` in all four bindings rather than nought in
   one. That last is the class of disagreement a count of absences
   cannot see, and it is now compared.

   Growing it found one real gap, which is what a gate is for.
   `jd_of_fixed` and `fixed_of_jd` were operations the other three
   bindings have and the façade did not: the runner could not convert
   the day it was standing on. They are **free functions** on the crate
   root rather than members of `calendar` or `time`, because a fixed day
   and a Julian day are two spellings of one integer and no profile,
   locale or setting changes the arithmetic — an area is for what a
   context's state bears on, and this bears on none of it.

   What is left of the 155 is named by kind rather than counted: 54
   per-item day rows (kaalas, choghadiya, horas, muhurtas, moon events),
   36 for a topocentric scenario the Rust runner does not walk, and
   about ten that are genuinely §6's — `abi`, the `build-*` keys, the
   blobs' hashes.

   The order of work was deliberately duplication-first: the façade beside
   the boundary, then the fourth parity runner that proves it equal to
   the other three, and only then the dependency inversion — because the
   two steps before it are what make it safe. Left unsettled and said so:
   the crate's name and whether `teistro` is held on crates.io (the
   maintainer's), how much of each area is re-export (step 1 answers it
   by construction), and `no_std`.

   Its first measurements are done and three of them falsified the plan
   they were measuring, which is what the passes are for. The truncation
   curve holds every size claim; the theory floor does not — VSOP87's
   Uranus and Neptune drift 4.7″ and 6.6″ against a modern ephemeris
   because they were fitted to DE200 in 1981, and no truncation mends
   that. The Moon's chosen theory turned out to be unobtainable, and the
   one that can be had misses by 19.4″ over the span `standard` claims.
   ADR-0027 settles it: a quadratic bija of twenty-four bytes takes the
   Moon to 3.0″ over six centuries and 0.26″ over the two it is fitted
   to, where a fitted table would have cost 3.76 MB. The pages are
   `03-design/builtin-ephemeris-measured.md` and
   `03-design/lunar-accuracy-measured.md`.

   [`07-roadmap/02-plan-performance-and-passthrough.md`](07-roadmap/02-plan-performance-and-passthrough.md)
   is finished: every step is built, or closed by a measurement that
   refused it (A1c, A3), or costed and deliberately unbuilt (B3). The
   rest of this item is that plan's record, and stays because it is the
   reason the numbers are what they are.

   The maintainer set a standing brief on 2026-09-09 with two halves,
   and a falsification pass measured where the SDK stands against both
   (`03-design/batch-and-parallelism-measured.md`, gated by
   `check-batching`).

   **Efficiency.** `positions` is a true batch — one ephemeris call
   whether it holds one instant or fifty — and **everything built on it
   is a loop**: 69 calls per chart, 1 213 per almanac day, at a mean call
   width of 1.1 cells through a port whose one required operation takes a
   grid. The repeat share rises with the batch, 22.5% within one chart to
   64.7% across fifty, which can only come from sharing between the
   items. The order of the fixes is the finding: **fix the arithmetic
   first, spend hardware last** — threads first would parallelise
   redundant work. `Capabilities::deterministic` is declared by both
   adapters and read by nothing; it is the memo's correctness gate and
   now has its reader.

   Attributing every one of an almanac day's 1228 calls to the code that
   makes them then **corrected A1 before a line of it was written**
   (the plan, §A1). A sunrise costs 4 calls, not 18, and all four are
   Meeus's iteration, which is serial and has no grid to ask for; the
   scan the plan meant to grid does not run at all at a temperate
   latitude. What does run is a **window**: to report which signs the
   Moon stood in during one day, the SDK searches eighty-one days of
   sign crossings — 44% of the day's calls — because the constant that
   sizes the reach was chosen for the Sun, which takes a month to cross
   a sign where the Moon takes 2.3 days. So A1 is now three parts in
   order: the reach follows the body, the uniform scan is gridded, and a
   range hoists what its days share.

   **A1a is built.** `events::least_rate` is `greatest_rate`'s
   companion, `longest_dwell_days` turns the pair into the longest a
   value can stand between two lattice lines, and the sign search asks
   for that instead of a constant: 31.579 days for the Sun, 2.564 for
   the Moon, and the old constant kept as a cap for bodies that can
   retrograde, whose dwell nothing bounds. An almanac day fell from
   **1228 calls to 841** and fifty days from 60 631 to 41 492. Nothing
   published moved: every generated page regenerates identically and the
   determinism digest is the same to the bit, with the span bounds
   themselves moving at most 2.8 ms (the Sun) and 0.04 ms (the Moon)
   against a tolerance of 8.6. `limb::signs_within` lets a caller name
   another reach.

   **A1b is built.** A scan knows every instant it will visit before it
   visits the first, so `events::Search::between` asks for them a grid
   at a time; only the ITP refinement stays serial, since each of its
   steps is chosen from the answer to the last. `Longitudes` grew
   `longitudes_and_speeds` and `longitudes_and_speeds_pair` beside the
   pair method it already had, defaulting to the walk and overridden by
   `FrameLongitudes` with one `positions` request; the chunk is
   `Search::with_chunk`, default 512, a memory bound rather than a waste
   bound. An almanac day fell from **841 calls to 665** and fifty days
   from 41 492 to 32 692; a four-hundred-day ingress search's 401 round
   trips became **1**. This one is **bit-identical**, held by
   `astro/tests/events.rs` over a retrograde planet at five chunk sizes
   comparing `to_bits()`.

   **A1b′ is built, and it was a defect.** Starting A1c asked whether a
   crossing found in a range-wide search is the one a per-day search
   would have found. It was not: the scan stepped from the caller's own
   `from`, so the same sign ingress came back **up to 2.2 ms apart**
   from windows offset by a fraction of a day, and only four of fifteen
   comparisons agreed to the bit. A crossing's instant was a property of
   the question as much as of the sky. Samples are now aligned to
   `SCAN_ANCHOR_JD` (J2000.0) and computed as `anchor + k × step` by
   multiplication, so a narrow window's samples are a subset of a wide
   one's; a bracket may reach outside the window, and a crossing found
   there is dropped rather than reported. A day rose 665 → **681** calls
   (the extra end samples) and in exchange a fifty-day range's distinct
   cells fell 39 675 → **21 446**, its repeat share 24.7% → **60.2%**,
   because consecutive days now ask for *the same instants*.

   Cumulatively an almanac day is 1228 calls → 681 and fifty days
   60 631 → 33 270, with the determinism digest unchanged to the bit
   throughout.

   **A2's memo is built.** `port_ephemeris::CachingProvider<P>` wraps any
   provider, including a borrowed one, and answers a cell it already
   holds from memory; a request is not all-or-nothing, so it asks for the
   instants that lack a cell by the bodies missing at any of them — one
   grid however scattered the gaps. It is sound because it reads
   `Capabilities::deterministic`, which the port has always asked
   providers to declare and nothing read: a provider that does not
   declare it is wrapped but **not cached**, and `caching()` says which.
   The capabilities it reports are the inner provider's unchanged,
   because they reach every provenance stamp and a cache must be
   invisible. A fifty-day range falls from 33 270 calls to **21 663** and
   from 53 935 cells to **23 888**, 55.7% answered from memory.

   Cumulatively an almanac day is 1228 calls → 616 and fifty days
   60 631 → 21 663.

   **The knob is built too.** A binding consumer hands the SDK a vtable
   and cannot wrap it, so `provider.cache_cells` is read in
   `TsContext::build`, where the SDK owns the box. One knob rather than
   two — nought is off, any other number is the capacity — so no pair of
   settings can disagree about whether the memo exists.
   `DEFAULT_CACHE_CELLS` lives in `core::settings` beside the knob and
   the port's default *is* that constant, so there is one number in one
   place. `EphemerisProvider` is now implemented for `Box<P>` as it
   already was for `&P`, without which nothing could wrap what a binding
   consumer supplies.

   **A1d is built, and the attribution found it rather than the plan.**
   Re-attributing a fifty-day range's remaining 21 663 calls before
   designing A1c showed that **85% of them were the horizon solver** and
   two thirds were its *fallback scan* — which A1's first draft had
   recorded as not running at a temperate latitude. It runs because
   `almanac::events` collects every rise in a window by searching on from
   the last, and the search that ends the loop has no event to find:
   proving that walks the whole remaining window at ten-minute steps,
   twice a day, every day. `solve::first_zero_gridded` now asks for the
   scan's instants a chunk at a time (`Solver::with_chunk`,
   `SCAN_CHUNK` = 32) while the narrowing stays serial, and
   `ApparentPositions::apparent_many` is `Longitudes`' grid one axis
   over. An almanac day fell 681 → **395** calls and fifty days
   33 270 → **19 632**; with the memo, **333** a day and **8 174** for
   fifty. Bit-identical, held by `to_bits()` over the grazing star at
   69.6°N across four event kinds and five chunk sizes.

   Cumulatively an almanac day is **1228 calls → 333** and fifty days
   **60 631 → 8 174**, with every generated page unchanged throughout.

   **A1c is closed, measured away three times.** The anchoring and the
   memo took its ephemeris ground — a fifty-day range fetches 23 888
   cells against a union of 21 446, within 11% of the least possible.
   Its arithmetic case was then measured directly: fifty days as a range
   against fifty asked one at a time shares **51% of the calls and 8.5%
   of the time**, so the memo does the sharing and the per-day
   arithmetic does not. Attributing the range's remaining 8 174 calls
   closed it — **over half is the horizon solver's Meeus iteration**,
   which is not shared work because every day genuinely has a different
   sunrise and each iteration is serial by construction, and the scans a
   hoist would share are 8% of the calls.

   **What that attribution points at instead**: 543 calls carrying
   **14 032 cells** are the horizon scan walking a whole window at
   ten-minute steps to prove an event is *absent* — `almanac::events`
   ends its loop by searching for a rise that is not there, twice a day.
   A body whose altitude cannot reach the target anywhere in the window
   can be shown so from the declination and the latitude in constant
   time.

   **That page is written and it falsified the rule twice before it
   held.** `03-design/horizon-absence-measured.md`, generated by
   `cargo xtask absence` and held by `check-absence`: 1768 searches over
   thirteen latitudes and thirty-four days, both bodies, rise and set.
   The rule is the two transits — `h ≤ 90° − |φ − δ|` and
   `h ≥ |φ + δ| − 90°`. **Absences cost 40 303 cells against 8 044 for
   the events that exist**, and the bound proves 60% of them, worth
   **59.4% of the cells**.

   The claim that matters is the one whose failure is a defect rather
   than a missed saving: the bound must never say absent where an event
   exists. It says so of **none of 1768** — but only after two wrong
   versions. With no margin it called eleven events absent, every one the
   Moon, whose declination moves **5.705° in a day** where the Sun's
   moves 0.405. Sizing the margin from that made it **worse**, fifty-three,
   and that is what said the lower bound was *inverted* rather than
   tight: absence needs the declination that lets the body dip deepest,
   the smallest `|φ + δ|`, and it was taking the largest. Building on
   either version would have turned sunrises into silence.

   **It is built**, and building it corrected the rule twice more.

   The margin was wrong: sized from the day's *motion* — six degrees for
   the Moon — it put every temperate latitude inside the range, proved
   nothing, and cost two readings to say so, taking an almanac from
   19 632 calls to 20 032. What a margin has to cover is the **stray from
   the chord** between the two readings, which the page now measures at
   seven interior points of every window: **0.168° for the Moon, 0.001°
   for the Sun**, thirty-six times tighter. With that the sweep proves
   216 of 274 absences and **78.2% of the cells they cost**.

   The placement was wrong twice. Before the iteration it charged every
   event that iterates cleanly — the common case anywhere temperate — for
   a proof it does not need. And with no window guard it ran on the
   partial windows an almanac's event loop ends with, where a
   whole-rotation bound can never fire: **the falsified claim on the page
   was the guard, and I had written it without connecting it to the
   workload**. Both fixed, an almanac now pays **0.08%** for it.

   Held two ways, which are not the same thing: the page holds the
   **rule**, swept with `Solver::with_absence_check(false)` so it is
   never measured against itself; `astro/tests/absence.rs` holds the
   **code**, 5 304 searches where the fast path and the walk agree on
   found or absent and on the instant to the bit, at **37.6% fewer
   cells**.
   **B1's route is built.** The port gained two optional methods —
   `native_manifest()` for the manifest the engine ships and
   `native_call(function, arguments_json)` to relay a call by the name
   that manifest gives — with `Capabilities::native` declaring them, in
   one of the two bytes `CapabilitiesC` had reserved, so the struct's
   size and offsets are unchanged and an adapter built against the old
   header still binds. `port_ephemeris::Native` reads the manifest into
   typed values. **The SDK holds no list of an engine's operations**, so
   a function the engine gains after the SDK ships is callable the day it
   appears; `Role::Other` keeps an unrecognised role rather than failing
   the whole manifest, and treats it as a caller's to supply so the
   function stays callable. The test provider ships a two-function
   manifest so the path is held by tests with no engine present, and the
   cache, the counter, a borrow and a box each forward the methods with a
   test that says so.

   The measured page's oldest falsified claim — *a consumer can reach
   what their engine offers beyond the port* — now **holds**.

   **B2's boundary is built.** `ts_ephemeris_manifest` and
   `ts_ephemeris_call` cross the C ABI; adding one module to the boundary
   put both into the C header, the Dart and Python declarations, the Node
   napi glue and the generated reference **without any of them being
   edited**. On the way it found a third instance of the same class of
   hole: a module of the `ffi` crate that holds entry points and is not
   in `teistro_idl::sdk::SOURCES` compiles, exports its symbols, and no
   binding has heard of it — the first version of this change was exactly
   that. `boundary-is-described` is the rule that now catches it, and it
   was proved by unregistering the module and watching it fail.

   **B2 is done: all three bindings have the proxy.**
   `context.ephemeris` answers with an engine in every language —
   `engine.tp_echo(value=6.0)` in Python (`__getattr__`, with `__dir__`
   so a REPL completes the names), `engine.tp_echo({ value: 6 })` in
   Node (a `Proxy`, with `has` and `ownKeys` so `in` and `Object.keys`
   see them), and `engine('tp_echo', {...})` in Dart **by name on
   purpose**: Dart spells members in camel case and an engine spells its
   functions as C does, so a member proxy would have to guess the mapping
   and a wrong guess would make that operation unreachable — the very
   dead end this route closes. Uniformity would have cost reach in one
   language, and reach is the point.

   **None of the three holds a list of operations**; Node's TypeScript
   declaration is an index signature rather than a set of methods for the
   same reason. `check-python` (80 tests, mypy strict), `check-node`
   (tests, maximum strictness) and `check-dart` (45 tests, analyse,
   format) all pass.

   **C1 is done.** `Body::key` and the catalogue's `Graha::key` both
   spell `"SUN"`, and nothing checked they matched. They are not merged —
   a `Body` is what an ephemeris computes and a `Graha` what a chart
   reads, and the port has two nodes where the catalogue has one Rahu —
   so the **overlap** is gated, using the pairing `Body::graha` already
   owns rather than a third list. The exceptions are pinned too: the two
   nodes, the two apogees, and Ketu as the one graha no body is.

   The audit that went with it found the port and the catalogue each
   hold a `Direction`, one a crossing's and one a compass point. Every
   binding emits one declaration per item into one namespace, so a
   collision is a compile error in Dart and TypeScript and a silent
   shadow in Python. The extractor already refused that for enums,
   structs, opaques, callbacks and functions — unsaid, so two tests now
   say it — but not for blobs, which is one word in the chain that
   existed. The first version of this change added a *second* checker
   beside it, which is the fault the step is about; reading the existing
   check first is the lesson.

   **A3 is closed by measurement, not merely blocked.** A3
   was to spend threads on a batch. The engine says otherwise about
   itself: a `teimeris::Context` is `Send` and deliberately **not**
   `Sync` — *"N contexts from N threads with no locking, which is a
   statement about N contexts, not about one shared between them"* — so
   the shipped adapter holds `Mutex<Context>` and every ephemeris call
   takes one lock. Measured over a year of the Moon's sign ingresses
   through the real engine (`parallelism_probe.rs`): **46–48% of the
   work is inside that lock**, 52–54% is the SDK's own arithmetic.
   Threads over one provider overlap only the second, which is **2.2×
   however many cores are given** and 1.7× at four. So the shape looked
   like a **provider per thread** — until that was measured too:
   **opening an engine context costs 43.8 ms, which is 19 491
   `positions` calls, and the largest batch this project measures is
   8 174**. A thread would pay more for its context than the whole batch
   costs. So the shape that wins is a pool of providers outliving many
   batches, which is the **consumer's** and needs nothing from the SDK —
   `Send + Sync` throughout, N providers on N threads works today.
   Threading one batch internally would buy at most 2.2× and cost a
   knob, a contended memo and the determinism of the gated counts.
   **Closed, not built**, with the threshold asserted in
   `tests/context_cost.rs` so it fails if a context ever becomes cheap
   enough to be worth revisiting.

   **The determinism matrix now covers the panchanga.** It had sections
   for the calendar, the astronomy, the houses and the classical model
   and none for the layer the searches feed, so the instants a crossing
   search *produces* were watched by nothing — three changes in a row
   moved them and the digest was identical to the bit each time.
   `teistro-scenario` has a `panchanga` section now: ten almanac days
   over the analytic provider, hashing every limb boundary, sunrise and
   moonrise, 374 values for ten milliseconds. Proved by moving
   `SCAN_ANCHOR_JD` half a step — the `astro` digest does not move and
   this one does. The module's own cost claim was stale by the same
   reading and is corrected to the measured forty milliseconds.

   **The engine's side is researched and costed, not yet built.** The
   maintainer handed me Teimeris on 2026-09-10
   (`~/Projects/Teispace/teistro/teimeris`, `teispace/teimeris`), so its
   conventions apply: `tools/ci/verify.sh all` is the arbiter, generated
   files are generated, and *benchmark before adding*. A dispatcher
   cannot live in the C core, which gates its size and speed against
   upstream, nor in `teimeris` or `teimeris-sys`, which have **no
   dependencies on purpose** so that they audit to nothing and build
   `--offline`. It wants a separate generated crate from the same IDL.
   The register hazard is moot there: `teimeris-sys` declares every
   function with its real C types, so the dispatcher writes typed Rust
   and never a machine word.

   Costed rather than assumed (`adapters/…/tests/dispatch_cost.rs`): the
   relay's floor is **18.70 µs against a 2.29 µs call, 8.15×** — but
   `positions` is the cheapest thing the engine does, and against an
   eclipse search the relay is under a per cent. It confirms the shape:
   this path is for what the port does not cover and is never the hot
   loop. Teimeris's roadmap is untouched, because their convention opens
   an item when work starts and this is a costed design.

   **Reach.** The port names eight operations; Teimeris's public header
   names 161 functions, 57 structs and 40 enums, so a consumer wanting an
   eclipse or a star must open a second handle to the same engine. The
   requirement is stricter than a passthrough: an operation the engine
   gains *after* the SDK ships must be callable without an SDK release.
   That is achievable because Teimeris already describes itself —
   `tools/idl/teimeris.idl`, 13 472 lines extracted from its own headers,
   carrying every signature **and a role for every parameter**, the same
   role vocabulary this SDK's own IDL uses. The SDK will hold no list of
   engine operations, only a dispatcher over the manifest the engine
   ships. The plan's §B3 records the one hard part so it is not
   rediscovered: doubles do not travel in the same registers as integers,
   and 82 Teimeris parameters are doubles passed by value across 54 of
   its functions, with eight more returning one, so a uniform
   word-sized dispatcher is wrong silently.

   Everything the maintainer and I settle mid-flight goes into that plan
   before it goes into code, so a compaction or a new session loses
   nothing.

   **The conformance harness is built and Phase 4's remaining exit work
   is done.** `adapters/ephemeris-teimeris/rust/tests/baseline_positions.rs`
   takes the instant and the place each of the 55 charts records, asks
   the SDK for every graha through the Teimeris adapter in the frame the
   fixture was recorded in, and compares longitude, latitude, speed and
   distance against what was recorded. **605 positions, every one
   exact**: the worst sidereal longitude, tropical longitude, latitude
   and distance are all 0.000000, and the worst speed is 0.000026″/day
   on Pluto. So the topocentric step's 0.084″ on the Moon does **not**
   matter to a recorded chart — the corpus cannot see it.

   Two things the harness had to get right, both of which read as
   agreement if got wrong. The `outer` block records
   `"frame": "geocentric"` while the chart is topocentric, so comparing
   those three in the chart's frame measures the parallax and calls it a
   disagreement — 0.44″ to 0.56″, which is 8.79″ over the distance in
   astronomical units, exactly what a parallax is; the fixture says which
   centre and the harness reads it. And a worst that compared *nothing*
   prints exactly like perfect agreement: the first version reported
   `0.000000″` for the tropical longitude and the distance and neither
   was ever compared, so every count is now printed and asserted.

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
   ships eight worked examples, and **`check-parity` compares 635 values
   across the three** where it compared 103 three sessions ago.

   The cross-phase dependency the roadmap recorded — **Phase 4 cannot
   exit before Phase 3 delivers the completion's centre step** — is
   discharged: the step turned out to be separable from the rest of
   Phase 3, because it needs the observer's position and the body's
   geocentric one and nothing of the built-in ephemeris, so it was built
   where it was needed rather than where it was scheduled.

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
chosen by spike. v1.0 is the whole feature universe: Western and Hellenistic
moved out of v1.x into Phase 7 on 2026-09-10 (ADR-0025, revising Q4),
the maintainer accepting the longer road; chart geometry moved out of
Phase 9 into Phase 4 with an optional first-party SVG renderer above the
core (ADR-0026). Apache-2.0 open core. Teispace owns all baseline engine content and
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
| 2026-09-12 | **The nightly matrix, read properly, and what it had been hiding.** The C link took four attempts and the log said it was never a flag: `-l<stem>` was finding `teistro_ffi.lib` beside `teistro_ffi.dll.lib` and turning a shared link into a static one, across two ABIs that do not meet (`__chkstk`, `__imp_NtReadFile`, `??_7type_info@@6B@`). `rustc --print native-static-libs` cannot be pasted into an arbitrary C driver either -- its dialect is the Rust target's linker's, and it broke all three platforms three different ways. `shared_link` and `static_link` answer the two questions apart, the second able to **refuse** with a reason, and two tests over the platform table hold them; the first is the one that would have caught it on attempt one. With the link fixed win32 gave up one defect per run, and every one was a Unix assumption: a virtual environment's executables are in `Scripts`; npm, npx and tsc are `.cmd` shims that `CreateProcess` cannot find, so a runner **with** npm said it had none and the gate skipped the Node packages; `check-python` had never run there and failed on `सोमबार` through cp1252, and then `check-parity` failed on `☉` because the fix went into one gate when four spawn Python. **And the matrix could never have finished at all**: `bindings (darwin-x64)` asked for `macos-13`, GitHub retired that image, and a retired label does not fail -- it queues, so eleven dispatches today never reached a conclusion and every judgement of green was made from the rows that ran. `macos-15-intel` picks it up. Four rules over classes rather than instances: `python-runs-in-utf8-mode`, `runner-matches-the-platform-table`, and two new properties of the measured surface -- every operation the layer declares listed by every parity runner (**born red** on `(root).engine`), and every area the layer wires named by the site's guide. The TypeScript compiler and mypy are both **pinned** and installed by their gates, because each was whatever a machine happened to have, which is why the type-check skipping on four of five platforms went unnoticed. `step` names the program on its failure line. **The docs had the same disease as the code: they described a surface nobody had run.** The install page's Node quickstart refused when run (`the context has no ephemeris`), its C line could not link, and Python was missing from the page though the binding has been gated since the 8th; the site had **no page at all** about `sdk.<area>.<operation>`, and now has one. Twelve of fifteen examples still selected the **test** provider while three of them called it "the analytic ephemeris the SDK carries" -- it is not, `builtin` is -- so every example now names the built-in, and `ephemeris.{mjs,dart,py}` gained the retrograde scan it had carried as a hypothetical comment (Mars stationing direct on day 55 of 2025, the same in all three). The no-ephemeris refusal, written for a C caller and duplicated three times, is one `support::no_ephemeris` naming the option every binding spells the same. Next: the publishing half of packaging, surveyed -- the engine compiles from source on any target and embeds its own data tier, so what is left is one maintainer's decision about how CI obtains the Teimeris source, and two unpushed commits. |
| 2026-09-09 (sixty-fourth session) | The completion's **topocentric centre**, measured and then built, and the cross-phase dependency that blocked Phase 4's exit discharged along the way. The corpus records six charts **twice** — from the centre of the Earth and from the place they were cast for, with every other setting equal — so for once a completion step had a recorded *before* as well as an after, and a proposed reading of it was right or wrong rather than close. `cargo xtask topocentric` (held by `check-topocentric`) tried nine readings against those pairs. **The one this project's own design page had carried since Phase 2 is the one that failed.** "The observer's geocentric position (WGS84) and the parallax" leaves a third of an arcsecond on **every** body — on Saturn, whose whole parallax is under an arcsecond, as much as on the Moon, whose parallax is forty arcminutes. A residual that does not shrink with distance is not a displacement gone wrong; the station is moving four hundred metres a second and the light it receives arrives aberrated by its own velocity, one and a half parts in a million whatever the distance. With that in, every planet comes inside a thousandth of an arcsecond and the Moon becomes the only body that can decide anything further. It says two more terms are needed, each worth another third of an arcsecond and **only together** — put in one at a time they make the answer worse: the displacement belongs on the direction the light actually came from rather than on the apparent one the Earth's motion has already turned by twenty arcseconds, and the body must be carried over the light time the station saves by standing an Earth radius nearer. Two simplifications a reader would call harmless are falsified by the same six pairs: a sphere costs 3.9″ and sea level 0.75″ against a bound of 0.0036″. **The lunar nodes take none of it**: in the pairs they are identical under both centres to the last bit of a double, and in all 174 recorded node rows — the true node included — their latitude is zero to 1e-15°, where a displaced point at that distance would sit 43′ off the ecliptic. A point defined as a direction on the Moon's orbit is not anywhere, which is now `Body::is_placed` on the port. What the pass leaves open it states rather than rounds away: 0.084″ on the Moon that nothing it could construct accounts for, and a velocity transform this corpus cannot decide at all, because it records a longitude speed and neither of the other two rates while both enter the answer. Built: five ERFA routines for the Earth an observer stands on (`eform`, `gd2gc`, `gd2gce`, `sp00`, `pom00`, `pvtob`) with the reference values of ERFA's own test program; `sky::observer` and `sky::earth_at`; `astro::topocentric::Station`, which transforms a cell's position, distance and all three rates analytically, because the step changes the Moon's longitude speed by 5.5°/day and a step that moved positions and left speeds alone would be wrong by more than it corrected. Held to the corpus at **0.00035″ on every body but the Moon** over 54 comparisons (`baseline_topocentric.rs`). Two things fell out of touching that path: **`sdk-only` now means what it says** — a provider is no longer asked for the whole frame first, because a provider that answers a whole frame has done several of the completion's steps itself — and a topocentric request with no observer is refused before any provider is asked, so the message names the field under every policy. The consequences: the shipped profiles that could not found a chart now do, the rectification examples run under `nepali-default` as a Nepali birth record should, and `check-parity` compares the chart that profile founds — **635 values across three bindings**, where it compared 594. Next: the conformance harness that compares a *computed* longitude against the 55 charts, which is what will decide whether the 0.084″ matters. |
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
