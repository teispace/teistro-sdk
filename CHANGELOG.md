# Changelog

Every release answers one question first, because it is the only one an
astrology engine's consumer actually needs:

> **Does this move any number, and by how much?**

A chart computed with the previous version and stored somewhere is a fact
someone may still be looking at. So each entry begins with **Numbers**, and
"none" is an answer that has to be earned by the conformance run against
the previous release, not by nobody having looked.

## Unreleased

**Numbers:** the reference engine's own corrections moved what the SDK is
measured against, not what it computes. All six findings the SDK filed
against Teimeris were fixed upstream and the recorded tables were taken
again under the engine's corrected profile: its sidereal time no longer
steps by 1.9″ at 2050, its Moon's parallax and disc now come from the
same distance, its Horizon co-ascendant at the equator agrees with every
other system, its star table's five wrong rows are right, and its IAU
2000B nutation carries the model's fixed offsets. The SDK's own numbers
are unchanged: it already computed all six the corrected way, which is
why the comparisons tightened rather than moved (the Moon's parallax from
0.5″ to 0.0002″, the equation of time from 0.2 s to 0.0075 s beyond 2050,
the galactic-centre ayanamshas from 0.68″ to 0.05″).
The Bikram Sambat table's computed rows moved. Every year
outside the official span (BS 1970 to 2095) is now computed by the SDK's
own engine (the Surya Siddhanta as the text prints it, Nepal's clock,
Kathmandu, the punya-kala rule) and the table runs from 1700 to 2500 BS;
the earlier rows for 1856 to 1969 and 2096 to 2457 were the baseline
engine's projections and differ from these by a day at some month
boundaries, and 2096 to 2100 are computed, no longer marked official.
Inside the official span no date moved; eleven boundaries there now
report `Divergent`. Sunrise and sunset from modern positions compute for
the first time (the rise and set solver); Delta T's values are
unchanged by their move from `time` to `astro`. The Surya Siddhanta's
star planets now report the text's daily motion (II.50 to 51) instead of
a central difference of the text's places, up to 0.23° a day apart for
Mars; their longitudes did not move. The text's latitudes, its Lagna and
the planetary hours compute for the first time. Crossings and stations
compute for the first time: sign ingresses with their retrograde
re-entries, the tithi, nakshatra, yoga and karana boundaries, composite
angles and single targets, and the stations, over any source of
longitudes. The boundary solver's narrowing moved from bisection to the
ITP method: every searched instant (a sunrise, a sankranti, a boundary)
is still the middle of a bracket no wider than its tolerance, so an
instant may differ from the previous release's by up to that tolerance
(under a hundredth of a second for the sankranti and the sunrise), in
about a quarter of the evaluations. The twelve star-anchored ayanamshas
(True Chitra, True Revati, True Pushya, True Mula, Sheoran, the four
Galactic Centre and the three Galactic Equator members) compute for the
first time, over the star table's SIMBAD astrometry (Hipparcos, the new
reduction, and Gaia DR3) and the SDK's own Earth ephemeris, so every
provider gives the same sidereal longitudes under them; the star table
itself (128 catalogued members) has places for the first time. The
planetary phenomena compute for the first time: elongation, phase angle
and illuminated fraction, apparent disc and horizontal parallax, and the
visual magnitude under the Astronomical Almanac's models (Mallama and
Hilton 2018 for the planets); so does the equation of time. A provider
may now answer a crossing search itself (the `CROSSINGS` override, with
its vtable slot and two kit checks): under `PREFER_NATIVE` Teimeris's own
search answers within 0.004 s of the SDK's kernel, so no instant moves
beyond that. The kit's Surya Siddhanta ayanamsha expectation for a modern
engine is now the catalogued epoch definition (18.94° at Burgess's 1860
instant), the text's own value being the classical astronomy's alone.
The Greenwich apparent sidereal time is now the IAU 2006 expression with
the IAU 2000B nutation (`gst06b`) instead of the IAU 2000 one (`gst00b`):
the meridian, every house cusp and the equation of time move by under
0.002″ (6e-7°) between 1950 and 2050 and by 0.01″ at 1850, and
Teimeris's sidereal time inside its window is matched within 0.0012″.
Visibility and the heliacal phenomena compute for the first time
(`astro::visibility`): the state of a body near the Sun on a day and the
days it appears and disappears, under the Surya Siddhanta's degrees of
time, the tradition's combustion orb or Ptolemy's arcus visionis, each
named in the call. The local mean midnight helper moved from the
classical crate to `astro::sky` unchanged. A composite quantity's two
bodies (the tithi, the yoga, an aspect) and a visibility reading's body
and Sun are now read in one position request: no instant moves, and the
tithi search costs 45 % less measured against its previous form in the
same run. Teistro Intl computes for the first time (`teistro-intl`): the
stable `MessageFormat 2` grammar with the SDK's functions, validation
with the catalogue as the authority for entity keys, `.tpack` packs and
`.tbundle` locale bundles, typed accessors for TypeScript, Dart and Rust,
and the `teistro-intl` command line; the SDK's `i18n/` ships `en-Latn`
and `ne-Deva-NP` with the entity records of the grahas, the signs, the
nakshatras and the Lagna. The engine's runtime API: a pack or bundle
loaded after construction, in-memory overrides, and the report of what is
loaded and covered. The date functions: `:date`, `:time`, `:datetime`,
`:ghati` and `:duration`, calendar-aware over the calendar crate, with
the patterns and names a locale declares in `sdk.calendar` (shipped for
`en-Latn` and `ne-Deva-NP`), era records for the nine eras, and the
`useGrouping` and `minimumIntegerDigits` options on numbers. The
baseline engine's entity name tables are imported (`teistro-intl migrate
baseline`): 274 records in each of four languages, `hi-Deva-IN` and
`sa-Deva` joining the shipped locales at `base` completeness. The C ABI
computes for the first time (`teistro-ffi`, `bindings/c/include/teistro.h`):
contexts from a profile, a JSON settings patch, a locale and the port's
vtable; the last error with its message, field and hint; keys and ids;
dates in every shipped calendar; civil times to instants with the zone
metadata and the scale conversions; the locale engine over the embedded
bundles; the frame a request asks for by name (centre, equinox,
coordinates, zodiac, corrections) rather than by its packed bits; and
positions through the port completed into that frame, as a result blob
(`TSRB`) with the completion steps and the provenance envelope. The C
binding's own test compiles against the header with warnings as errors
and runs (`cargo xtask check-c`). The Node binding's generated layers
come from the same description: the TypeScript surface (every enum a
string union with a frozen table, every boundary struct a readonly
interface with its units, ranges and examples), the catalogue's tables,
and one decoder per result blob reading columns as views over the blob's
own bytes; `cargo xtask check-node` runs the decoders against blobs the
library produced and type-checks a consumer at maximum strictness. The
Node addon computes for the first time (`bindings/node`, generated napi
glue over the C ABI with a hand-written layer above it): a context from a
profile, a settings patch and a locale; dates in every shipped calendar;
civil times to instants with their zone metadata and the scale
conversions; messages in any loaded locale; and positions through the
ephemeris port, decoded from the result blob on first use. A failed call
is a `TeistroError` carrying the status, the code, the field and the hint
the library gave. An ephemeris written in JavaScript answers the SDK for
the first time: an object with a `positions` callback is bound into the
port's vtable, asked once for a whole grid, and may refuse a frame by
answering with nothing, in which case the astronomy layer completes the
rest from the provider's own frame. The Dart binding computes for the
first time (`bindings/dart`, generated `dart:ffi` declarations, value
classes, catalogue enums and blob decoders with a hand-written layer
above them): the same calls the Node binding answers, with a context
freed by a native finaliser, a bitset field read as a `Set`, and a call
that hands back two things returning a named record whose fields are the
Node object's. An ephemeris written in Dart answers the SDK for the first
time: a class with a `positions` call is bound into the port's vtable
through an isolate-local callback, asked once for a whole grid, and may
refuse a frame by answering with nothing, in which case the astronomy
layer completes the rest from the provider's own frame. The two bindings
are held to each other by a parity gate: one scenario walked through both
ergonomic layers, ninety-six values reported by each, and the two
compared value by value. Both refuse a library that is not the build they
were generated from: `ts_build_info` says what a build is (version, ABI,
commit, profile, target, sanitizer, compiler), and a mismatched version,
a sanitizer build, or an unoptimised one the loader searched out is
refused rather than loaded. Both carry the SDK's typed message accessors,
so an application spells a message key once, in the generator: every
message of the SDK's locale is a function of its typed parameters, and
every catalogued entity is its forms in the locale
(`ctx.messages.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 })`,
`ctx.entity('graha.SUN').name`), over the new `ts_intl_entity` entry
point. A time renders on a twelve-hour clock where a locale reads one:
`:time hour12=true` gives every pattern the hour on both clocks, the
locale's word for the part of the day and its am or pm, so English reads
`6:15 am` and Nepali `बिहान ६:१५`. A Sanskrit or Nepali term written in
Devanagari reads in Latin: the transliteration is a table
(`teistro_intl::translit`, `ts_intl_transliterate` at the boundary), and
`sa-Latn` is derived from `sa-Deva` by it, so a Latin-script reader gets
all 274 entities without anyone writing them twice. The sources go to a
translator's own tools and come back: `teistro-intl export xliff` and
`import xliff` round-trip every message and entity form as XLIFF 2.1. A
latitude can no longer be passed where a longitude is wanted: the
description says which quantity a number carries, TypeScript gets a
branded type and Dart an extension type, and the constructor checks the
range. **Numbers:** reading a Bikram Sambat date no longer allocates. The date
itself is unchanged; what moved is that `CalendarResolution` borrows the
authority and the edition of the table it came from rather than copying
them, so the path every chart takes for every date it shows allocates
nothing. A counting allocator now holds that and the other hot paths to
their measured budgets.

Four determinism rules no compiler checks are now a gate on every push
(`cargo xtask check-lints`): no unordered iteration in a computation
crate unless the file says why, no reads of the clock or the environment
in one, only the port and the boundary may hold unsafe code, and the
classification functions stay integer arithmetic. Whether the same source
computes the same numbers on another machine is now measured rather than
assumed: `cargo xtask hashes` hashes
a hundred thousand computed values per build, and the nightly matrix
compares Linux x86-64, Linux aarch64 and macOS aarch64. The first run
says the two architectures agree bit for bit, and that macOS differs in
the astronomy and the house systems because its maths library rounds
differently in the last place; the calendars and the classical model
agree everywhere. The API description (`idl/api.json`, `teistro-idl`) is extracted
from the boundary crates' source and gated. **The settings hash moved
for any build that enabled the JSON layer's `preserve_order` feature**:
the canonical document's keys are now sorted by the SDK itself, so the
hash of a settings document is the same in every build (a crate compiled
alone and the workspace hashed the same settings differently before);
the astronomical numbers do not move. Nothing else computes yet.

- Project founded: research, architecture, decisions, roadmap and the
  open-source scaffolding. See `docs/STATUS.md`.
- `chart::zodiac`, and the ayanamsha basis it found. A chart holds **one**
  ayanamsha value and measures the grahas and the cusps from it, so it
  asks a provider for a tropical frame and shifts what comes back rather
  than letting the provider apply an ayanamsha of its own to the grahas
  while the SDK applies one to the cusps — which would compare two
  zodiacs at every bhava boundary. It also lets a custom ayanamsha work
  at all, where the port's `Zodiac` has nowhere to put an epoch and a
  value.

  The corpus settles the design: over its 55 charts and 550 bodies,
  `sidereal = tropical - ayanamsha` closes to 1.1e-13° with one value per
  chart, and the recorded lagna is the recorded ascendant on every one.

  **Numbers:** `conformance-baseline` moves, and nothing else. The
  recording engine applies the **nutated** ayanamsha and has no knob for
  it: the SDK's mean Lahiri is up to 18.46 arcseconds from the value the
  engine recorded and its true Lahiri within 0.0086, two thousand times
  closer, the difference being the nutation in longitude. The profile
  whose job is to reproduce those charts now sets
  `frame.ayanamsha_basis = TRUE` (version 2); the SDK's own default keeps
  the mean value, which is what the Lahiri definition states. Entry 16 of
  the deliberate-difference registry.
- The chart day against the corpus, and two things it corrected. The day
  selection, the lagna's anchor, day-or-night and the ishtakaal now read
  the corpus's `foundation` section, which nothing had read before.

  **Numbers:** none moved. Over the 50 charts comparable without a
  registered exception, the arc holding the birth, the sunrise anchoring
  the lagna and `is_day_birth` are the recorded ones on every chart, 20
  of them births before sunrise; the ishtakaal agrees over 100 readings
  in both reckonings.

  **Bhayat and bhabhoga are not the day's part**, which the chart
  foundation's design page said they were. They are the duration of the
  Moon's traversal of its nakshatra and the elapsed part of it at birth:
  they reproduce `dashas.methods.temporal.nakshatra_span` over all 55
  charts to within 0.39 minutes, the ghati-pala rounding, and they are
  nowhere near the length of the night. They belong to `dasha`; the page
  is corrected and a test asserts the fact so it cannot drift back.

  **The engine's night is `24h` minus the daylight**, not the interval
  from sunset to the next sunrise. True of all 110 nights the corpus
  records, and up to 1.80 minutes from the real interval; the two agree
  only when consecutive days have equal daylight. It reaches the
  proportional ishtakaal, which spreads thirty ghatis over the night, and
  the SDK — which divides the night it actually has — is up to 3 palas
  from the engine over the corpus. Entry 15 of the deliberate-difference
  registry.
- `crates/chart`, the first code of Phase 4: the two parts the design
  named as easy to get wrong, both against all 55 recorded charts.

  `chart::day` answers which day an instant belongs to. A panchanga day
  runs sunrise to sunrise, so an instant before the civil date's sunrise
  belongs to the day that began the morning before, and the vara, the
  hora, the ishtakaal and the sunrise that anchors the lagna move back
  with it. `time::local_day` answers "the arc of this date"; this is the
  inverse. `DayPart` has two members and not three, because pre-sunrise
  is the previous day's night. The type is `ChartDay` rather than the
  design's `DayArc`, which `calendar::solar` already has for a different
  thing.

  `chart::bhava` turns cusps into bhavas. It is the only place that does,
  and it keeps the madhya beside the sandhi because under an unequal
  division the middles are not midway between the boundaries — Sripati's
  madhya are Porphyry's cusps and cannot be recovered from Sripati's own.
  A `Placement` carries the chalit that made it, how far through its
  bhava the graha is and how far from the madhya.

  **Numbers:** none moved; the corpus's `houses.bhava_chalit` section had
  nothing to compare against until now, and now it does. The SDK's madhya
  and sandhi reproduce the recorded ones over all 55 charts — 1320
  compared, worst 1.7e-13° — every one of the 495 graha placements is the
  recorded one, and the engine's own list of the 107 grahas its chalit
  moves out of their whole-sign house is right in both readings. Sripati
  against Vehlow comes to 21.8% of placements, which is what `cargo xtask
  chalit` measures independently from the SDK's own cusps; the test
  asserts that difference rather than avoiding it, because it is entry 14
  of the deliberate-difference registry.
- The chart foundation's design page
  (`03-design/chart-foundation.md`), Phase 4's first. It settles what
  every module above the chart starts from, and three things in it are
  not obvious.

  **The day a chart belongs to is not its civil date.** A panchanga day
  runs sunrise to sunrise, so an instant before the civil date's sunrise
  belongs to the day that began the morning before — and with it the
  vara, the hora sequence, the ishtakaal and the lagna's anchor all move
  back a day. `c001` was recorded to prove exactly this: a birth at 05:30
  in Kathmandu where sunrise is 05:30:44. An implementation that looks up
  the civil date's sunrise is wrong by a day for every instant between
  midnight and sunrise. `time::local_day` answers "what is the arc of
  this date"; the foundation needs the inverse and carries a `DayArc`
  with two parts, not three: "pre-sunrise" is the previous day's night
  and calling it that stops the question being asked once per module.

  **A house placement is not a number.** It carries the chalit that
  produced it, how far through its bhava the graha is, and how far from
  the madhya — because the falsification pass measured the four methods
  disagreeing on 10% to 51% of placements, and because
  `astro::houses::Houses` has no notion of a madhya at all, which is what
  bhava bala is built on.

  **The foundation holds what is needed to compute, never what is
  computed.** The corpus's own `foundation` carries the arudha and
  navamsha lagnas; the SDK's does not, because both depend on modules
  that depend on the foundation, and the usual way that circle gets
  broken is a second evaluator of the same rule.
- The bhava chalit falsification pass, which the roadmap asks for before
  the chart layer is written. `cargo xtask chalit` computes each of the
  four named methods with the SDK's own house systems over the 55
  recorded charts, places the recorded grahas in them, and writes
  `03-design/chart-bhava-chalit.md`; `check-chalit` holds the page, so
  every number on it is what this build produces.

  **Numbers:** none moved; this measures numbers that were already there.
  What it found: the recording engine's `bhava_chalit` calls itself
  `equal-house` on all 55 charts and is Vehlow, which reproduces every
  recorded placement on 55 of 55 where Sripati manages 20 and Porphyry
  none. And the four methods are not variants of one thing — they put a
  graha in a different house between 10% and 51% of the time depending on
  the pair. The sharpest pair is Sripati against Porphyry at 50.5%: the
  *same cusps*, agreeing to the last decimal, read once as house middles
  and once as house starts. Sripati against Vehlow, the two a Jyotisha
  application actually chooses between, is 21.8% — better than one
  placement in five, and 37.2% beyond 30° of latitude. So a result that
  names a house without naming the method is not reproducible, which is
  what `houses.chalit_system` in the settings hash is for, and the chart
  layer must report the madhya as well as the sandhi: `astro::houses`
  returns cusps alone, which places a graha and cannot say how near the
  middle of its house it sits. Entry 14 of the deliberate-difference
  registry (`05-testing/01-golden-vectors.md`), and principle 4 now cites
  the measurement rather than asserting it.
- The default profile is the texts as read, and inherits nothing else
  (Q34, ADR-0024). `parashari-classical` stays the profile a context gets
  when its options name none, and it now patches the root rather than
  `nepali-default`. It had been declared over that profile, so it
  inherited the recording engine's topocentric centre, Nepal's civil
  calendar and eras, and its synthesised polar days — none of which is in
  any text, while the profile's own documentation said the opposite. It
  is now geocentric, Gregorian, the three pan-Indic eras and an undefined
  polar day, with the four knobs that define it cited: Sripati bhava
  (BPHS), proportional ghatis, eight chara karakas (Jaimini 1.1.10-18)
  and the Surya Siddhanta's combustion orbs. `nepali-default` is
  unchanged and keeps all four, each cited to the engine whose charts it
  reproduces.

  **Numbers:** none today, and a great many later. The centre is the knob
  that moves them, and the SDK's own frame completion does not apply the
  topocentric step yet (Phase 3); what changes today is what a capable
  provider is asked for under `prefer-native`, and the settings hash of
  every result computed under the default, which is how a change of
  defaults is meant to be visible. How much it will move is recorded
  rather than estimated: the corpus holds the same six charts both ways
  (`baseline/variants/*--geocentric.json`), and across them the Moon
  differs by up to 39.1 arcminutes — a fifth of a pada — against 0.135
  for the Sun, 0.246 for Venus and 0.060 for Mars. Two of the six change
  a classification: c049's Moon moves from nakshatra 20 to 21, which
  changes the Vimshottari mahadasha lord and the whole tree under it, and
  c050's from pada 2 to pada 3. A caller at a polar latitude who names no
  profile now gets a reported absence rather than a synthesised day.
- Phase 1, Foundation, met its exit criteria on 2026-09-06. Every one is
  held by a gate rather than by a claim: one scenario through both
  bindings value by value (`check-parity`, 103 values), 100,236 values
  identical across x86-64 and aarch64 (the `hash-matrix` workflow), the
  conformance kit against the Teimeris adapter, the classification
  property tests over every divisor, a swapped latitude and longitude
  refused in Rust, TypeScript and Dart, and all four packages installed
  into throwaway projects and run before they can be published
  (`check-package`). The roadmap records what was deferred by decision
  and what was built beyond the list (`07-roadmap/00-roadmap.md`).
- The conformance corpus left this repository.
  [`teispace/teistro-conformance`](https://github.com/teispace/teistro-conformance)
  v0.1.1 holds it under CC0-1.0 with a version of its own, and `fixtures/`
  is a submodule of it pinned to a tag (ADR-0022). Every file moved byte
  for byte and no recorded value changed: the SDK's gate proves the corpus
  after the move exactly as it did before. A standard obtainable only by
  cloning one implementation is not a standard, which is why it went.
  It took its description and its checking with it — what each of the 55
  charts is for, what each of the 13 profiles changes, what every section
  of a fixture holds, JSON Schemas for a fixture, a manifest, the
  tolerance file and a conformance report, and a validator that runs on
  every push there. What stayed here is what is the SDK's rather than the
  corpus's: the thirteen conventions of the recording engine that the SDK
  deliberately does not copy, now in `05-testing/01-golden-vectors.md`
  where the pages that cite them can reach them. `cargo xtask
  check-fixtures` refuses a checkout with no corpus rather than passing
  over an empty directory, every workflow checks the submodule out, and
  `CONTRIBUTING.md` says to clone with `--recurse-submodules`.
- Teistro Intl: the parts of the day became a locale's own, and
  `:duration` learnt to break a count into several units. A locale states
  `dayPeriods` in its `_meta.json` — the hour each part begins at and the
  key that names it, in order, the last wrapping past midnight — and one
  that states none takes the division English and Nepali share, which is
  what every locale had before. The validator holds a stated division to
  being in order, inside a day and named once, because an order that
  slips silently swallows a part and a key with no message renders as
  itself in the wrong language. `{$v :duration unit=second
  into=|hour,minute,second|}` over 3725 now reads "1 hour, 2 minutes and
  5 seconds", each part through the unit's own plural message and joined
  by the locale's `and` list pattern, in the locale's digits; the units
  may be named in any order, a zero part is dropped unless every part is,
  the shortest unit keeps the remainder rather than rounding it away, and
  a negative duration is negative once rather than on each part. No
  number moves and no rendering changes for a locale that states nothing
  new (`03-design/intl-engine-and-packs.md` §5).
- Instruction-count benchmarks. `crates/scenario` holds the fixed
  scenario the SDK measures itself with — the calendars, the astronomy,
  the house systems and the classical model over the same fixed days and
  instants — lifted out of the hash gate so that the determinism matrix
  and the benchmarks walk the same code and neither measures a path the
  other never checked; every digest is unchanged by the move. `cargo
  xtask bench` runs it under callgrind, once per section and once doing
  nothing at all, and reports the difference as what that section costs;
  `compare-bench` compares two such runs, failing above 3% and reporting
  above 1% (ADR-0022's quality bar). Nothing is timed: wall-clock time on
  a shared runner moves further with a neighbouring job than with most
  changes, while an instruction count is exact. The `benchmarks` workflow
  runs it on every pull request against the base commit measured in the
  same job on the same machine, because an instruction count belongs to a
  compiler and a target as much as to the source.
- The documentation site, `site/`: Fumadocs on Next.js, exported as
  static files. Its reference is generated from the same description every
  binding is generated from (`crates/idl/src/emit/mdx.rs`): one page per
  entry point, grouped by the boundary's own source files, each carrying
  the doc comment, the C declaration, tabs naming what the Node addon and
  the Dart library call it, a parameter table with every role, unit, range
  and example the description knows, what the call hands back, the blob
  schema it fills and its safety contract; with the structs and the enums
  beside them. `cargo xtask gen ffi` writes the tree and `check-ffi` holds
  it — including files the generator does not produce, so a page for a
  removed entry point cannot linger. `cargo xtask check-site` builds the
  site and checks that every generated page was rendered, which is what
  proves the emitter's MDX escaping: a doc comment is prose written for
  Rust, and MDX reads a `{` at the start of a line as an expression even
  inside what a Markdown reader would call code. The `docs` workflow
  publishes to GitHub Pages on a tag, so the site a reader lands on is the
  site of the version they are reading about
  (`docs/06-cicd/05-docs-deploy.md`).
- Packaging and the release matrix. The SDK has one version, declared in
  `[workspace.package]` and held across both package manifests, the five
  platform packages and the API description by `cargo xtask
  check-versions`; `cargo xtask version X` moves it. `cargo xtask package`
  builds what a platform ships — the shared library gzipped, a C bundle
  with the header and both libraries, and the npm package carrying that
  platform's addon — each recorded in a manifest with its size and its
  SHA-256, the library's taken uncompressed. `cargo xtask package stage`
  merges the five and writes the two packages published once, the Node
  package that depends on the platform packages and the Dart package whose
  installer fetches from the release. `cargo xtask check-package` installs
  all of it into throwaway projects and runs a consumer against each: the
  C bundle unpacked and linked both ways, the npm packages packed and
  installed into an empty project, and a Dart project that fetched its
  library through `dart run teistro:install`. Node finds its addon in the
  platform package npm chose (`@teistro/sdk-<platform>`); Dart fetches a
  library and refuses one whose digest is not the one recorded when it was
  built, with SHA-256 the package implements itself rather than take a
  dependency for. Two workflows: `verify`, the bindings' gates and the
  packaging on all five platforms nightly, and `release`, which builds,
  merges, stages and publishes from a tag that must be the version the
  repository carries (`docs/06-cicd/`).
- `crates/core`, `crates/calendar` (the arithmetic calendars and Bikram
  Sambat), `crates/siddhanta` (the Surya Siddhanta model), the seed of
  `crates/astro` (the boundary solver), and the Bikram Sambat engine with
  its measurement (`docs/calendars/bikram-sambat.md`).
- `crates/time` and `crates/port-timezone`: time scales with Delta T as
  the IERS table (1956 to the present) then Espenak and Meeus (2006)
  with Morrison and Stephenson's uncertainties, the IANA leap-second
  table, civil time, zone resolution over the embedded tzdb with the
  metadata a stored chart replays, local mean time, the sunrise-anchored
  day with the polar policies, ghati-pala. Every zone resolution of the
  55 fixture charts reproduces the baseline's instant and metadata.
- `crates/port-ephemeris` (spike 3's port promoted, with the rise and
  set override), `crates/astro` (Delta T moved here from `time`; the
  IAU routines ported from ERFA with a provenance table; sidereal time
  and the obliquity; frame completion over the port; the rise and set
  solver under the sunrise conventions, with polar days reported),
  `crates/ephemeris-kit` (the conformance kit: fifteen checks, both
  engines passing), the drik solar model for the calendars, the local
  day's convention and the `SUNRISE` unknown-time fallback in `time`,
  and the adapters under `adapters/`. Measured: the geometric sunrise
  agrees with Teimeris's own search within 0.13 s; the refracted one
  within 2.5 s of the baseline's fixtures below 60° of latitude (the
  refraction convention, cruxes C34). The committee's stated method for
  Bikram Sambat (the Surya Siddhanta) recorded in the memo, and modern
  positions measured at 65 % of the official months against the text's
  98.5 %.
- `crates/siddhanta` completed to the text: the sighra daily motion
  (II.50 to 51), the latitudes (II.56 to 58) and the Lagna from the
  oblique ascensions (III.42 to 50), each reproduced against Burgess's
  worked computation for 1 January 1860; `SiddhantaProvider` presents the
  model behind the ephemeris port as a classical astronomy and passes
  the kit, whose report publishes the text's distance from modern
  astronomy (the obliquity, the sunrise, the speed rule) instead of
  gating it. The port gained `Astronomy`, `SpeedModel`, `DistanceUnit`
  and the `DUT1` override; the completion orders the zodiac shift and
  the rotation so a sidereal ecliptic provider completes to equatorial
  tropical coordinates. `crates/time` gained the planetary hours under
  the `hora_reckoning` knob (proportional by default, as the baseline's
  fixtures decide) and UT1 from a provider's DUT1.
- `fixtures/official/npns-2082-2083.json`: the national panchanga
  committee's published panchangas for BS 2082 and 2083 read into data
  (24 sankranti instants, printed places, sunrise and sunset, tithi
  ends), with tests that the SDK's engine reproduces every instant
  within 1.6 minutes and every month start, that the committee's Sun is
  the text's within 3″ and its Moon the text's with a bija of four
  revolutions fewer on the apsis, and that its star planets are modern
  positions (`docs/calendars/bikram-sambat.md`, R2; cruxes C38, C39). No
  computed number moved.
- `crates/astro`: precession as a catalogue of models (Vondrák 2011 the
  default, IAU 2006, IAU 1976, Newcomb) over new ERFA ports (the IAU 2006
  angles and matrices, the long-term poles and matrices, the vector
  primitives, each against ERFA's reference values) and Vondrák's own
  obliquity series; the ayanamsha catalogue, every epoch-defined and
  frame member computed from its published definition with the
  fitted-model correction, mean or nutated, custom definitions linear;
  the frame completion now completes a sidereal zodiac from the SDK's
  catalogue when the provider declares no override, so `sdk-only`
  sidereal charts compute for the first time. Against Teimeris's
  recorded values (`fixtures/teimeris/ayanamsha.json`, 1044 rows) the
  definitions stated in TT agree within 1e-7″ and those in Universal
  Time within 2.1e-4″. The twelve star-anchored members are refused by
  name until the star table (`docs/03-design/astro-ayanamsha-catalogue.md`).
- `crates/astro`: the twenty-two catalogued house systems (`houses`) as
  one construction with the circles each system picks, the auxiliary
  points (vertex, equatorial ascendant, the co-ascendants, the polar
  ascendant), the sign-based systems in the zodiac in use, and the four
  polar policies with the outcome reported. Measured within 5e-6° of
  Teimeris over 25 194 cusps and angles at ten latitudes
  (`fixtures/teimeris/houses.json`) and within 0.0002° of the baseline's
  55 charts between 1800 and 2200 (0.0033° beyond, the engines' long-term
  sidereal time). Houses compute for the first time
  (`docs/03-design/astro-house-systems.md`).
