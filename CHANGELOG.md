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
branded type, Dart an extension type and Python a `float` subclass, and
the constructor checks the range.

**The Python binding computes for the first time** (`bindings/python`,
generated `ctypes` declarations, value classes, catalogue enums and blob
decoders with a hand-written layer above them): the same calls the Node
and Dart bindings answer, from a package with **no runtime dependency**
and nothing to compile, because it loads the shared library the release
already builds. An ephemeris written in Python answers the SDK for the
first time, through `CFUNCTYPE` trampolines that catch everything a
provider raises, because an exception escaping a `ctypes` callback would
otherwise return zero and the port would read that as success. A decoded
positions column is a read-only `memoryview` over the blob's own bytes,
which `numpy.asarray` wraps without copying, so numpy interop costs
nothing and is not a dependency. The parity gate now walks its scenario
through **three** ergonomic layers and compares 103 values from each.

Every binding gained the eighteen units the boundary had been missing:
`ObliquityC`, `HorizonRequestC`, `CrossingRequestC`, `CrossingEventC`
and `CapabilitiesC` carried floating-point fields with no unit, range or
example, so the C header, the TypeScript surface, the Dart classes and
the documentation site all documented them as bare numbers. A
falsification pass over the API description (`cargo xtask surface`, held
by `check-surface`) found them, along with the parameter role no entry
point uses — which the Python emitter refuses by name rather than
guessing at — and the fact that each binding's reserved-word rule fires
in a different place, so the three word lists moved into one module
where they can be counted.

**Numbers:** reading a Bikram Sambat date no longer allocates. The date
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
- `knob-has-a-reader`, the fifth determinism lint, and the one settings
  knob it found that was really unread.

  **Numbers:** none moved. `Scheme::cyclic` behaves exactly as it did;
  what changed is that the convention is now asked for rather than
  assumed.

  A settings knob that ships, resolves and is read by nobody is a bug
  whether or not anything crashes, and three had been found by hand in
  as many modules: `state.combustion_orbs`, which made a chart founded
  on the SDK's own default profile fail (registry entry 23);
  `houses.module_overrides`, which quietly gave a KP reading whole-sign
  houses where every shipped profile says Placidus; and
  `output.precision`, which did nothing at all. Three in three is a
  pattern rather than an accident, so `cargo xtask check-lints` now
  finds them by machine.

  The rule enumerates the knobs from `core` itself —
  `Settings::knob_paths`, a new list held to the settings document by
  its own test, so a group added to the document is watched without a
  second list to remember — and counts readers outside the settings
  layer over the source with its whitespace collapsed, because a chain
  the formatter breaks across lines is still one access. It found
  fourteen.

  **One was real.** `crates/vargas`'s `Scheme::cyclic` named a
  convention it never asked for, and the crate's own design page claimed
  `vargas.unattested_dn` chose it. `Scheme::unattested(divisions,
  convention)` now takes the reading and refuses one it has not been
  taught by name, and `Scheme::for_settings` reads the knob — so a
  convention added to the catalogue forces a decision rather than being
  silently read as the cyclic one.

  The other thirteen are **deferred with a reason** written at the
  knob's own declaration and printed on every run: `dasha`, `jaimini`
  and `strength`'s belong to Phase 5 modules, `provider.tier` to Phase
  3's built-in ephemeris, `frame.siddhanta` and `frame.nakshatra_scheme`
  to work each names, and `calendars.civil_calendar` and `.eras` gain a
  reader when something builds a chart from a settings document alone.
  **An allowance that is no longer needed is itself a failure**, so the
  inventory cannot rot.

  All three failure modes were proven red before the rule was trusted,
  as the other four lints were: a knob with no reader and no marker, a
  stale allowance on a knob that is read, and — through the core test —
  a group renamed out of the list.

- `crates/serial`, one JSON document for a chart and one way of writing
  it, and the measurement that decided it first. 22 tests.

  **Numbers:** the canonical form's **bytes moved**, so a stored content
  hash from an earlier build will not match. The form now writes every
  number as a plain decimal to twelve places with trailing zeros
  trimmed, where before it wrote whatever Rust's JSON layer produced —
  `1e-6` where JavaScript's layer writes `0.000001`. The values are
  unchanged; only the way they are written is. Two doubles differing
  below the grammar's resolution now hash alike, deliberately: that is
  what a caller asking "is this the same answer" means.

  This was the first falsification pass to read the **source** rather
  than the corpus, because nothing recorded can say whether the SDK
  fills the fields it documents. Three things were wrong.

  **The content hash was the hash of nothing.** `Provenance` carries
  every field ADR-0020 asks for, and the one the envelope exists for is
  set to `Hash::of(&[])` by `Provenance::new` as a placeholder and was
  replaced by exactly one producer of three. A founded chart and a daily
  panchanga both went out claiming a hash of the empty string. That is a
  shape problem rather than a bug in a producer — the one field that
  cannot be filled until the value exists is the one everybody forgets —
  so `Sealed::new` is the only constructor and it computes the hash. A
  stale one is not representable.

  **The chart layer could not be serialised.** `ChartFoundation`, which
  every other Phase 4 value is computed from, along with `Bhavas`,
  `ChartDay`, `ChartZodiac`, `GrahaPosition`, `Placement`, `Chalit`,
  `Reading`, `DayPart` and `BirthTiming`, derived no `Serialize` at all.
  The SDK could not publish a chart. They do now.

  **Two bindings would have disagreed about a number**, which is the
  reason for the grammar above. A binding implementing it needs no float
  printer of its own.

  `output.precision` gets a reader — the third shipped, populated and
  unread knob found in as many modules, after `state.combustion_orbs`
  (registry entry 23) and `houses.module_overrides`. It governs the
  **rendering** and not the hash: a hash that moved with a display
  setting would be a worse cache key, and the settings hash already
  tells two precisions apart.

  The canonical grammar lives in `core` beside `content_hash`, because
  there is one canonical form in the SDK and not two;
  `core::envelope::canonical_json_at` is what a rendering uses.
  `Document` holds every section the chart layer produces — foundation,
  panchanga, vargas, state, aspects, points, houses — with everything
  but the foundation optional, and the whole sealed once.

- `crates/houses`, which house under which reading, and the measurement
  that decided it first. 26 tests.

  **Numbers:** nothing moved. The geometry was already there — `astro`
  computes twenty-two systems and `chart` makes bhavas of their cusps —
  and this is the service over them, not a second copy.

  The pass had to find its own subject. Most of the corpus's `houses`
  section already had a reader, so it measured the parts that did not,
  and those turned out to be the ones a service depends on.

  **A boolean cannot say what happened.** The engine records
  `is_degenerate` — one bit for "the chosen system had no solution
  here" — and the SDK's `astro::houses` returns an outcome with three
  cases. Nothing had ever compared them, and **they disagree in both
  directions**: the engine flags two charts under Placidus at 64.15° and
  64.84°, *below* the polar circle of 66.56° where the SDK computes
  Placidus without trouble, and leaves one clear at 69.65°, *above* it,
  where the SDK cannot compute the system at all. They are not the same
  quantity read to different precision — they disagree about which
  charts are the difficult ones. Registry entry 26, and the reason
  `Houses` reports an outcome and a policy rather than a flag, and why a
  degenerate chart is reported rather than refused.

  **The shift, counted the other way.** The engine lists the bodies the
  chalit moves out of their whole-sign house; `chart`'s test checks
  every body it lists, and nothing checked that the SDK lists no
  *others*. A rule that shifted one body too many would have passed.
  Both directions now hold on the same 135 bodies over 75 fixtures.

  **A knob worse than unread.** `houses.module_overrides` says which
  system a named module uses, and the root *populates* it — all five
  shipped profiles carry `kp → PLACIDUS` and nothing had ever asked.
  Under the default profile the rest of a chart is whole-sign, so a KP
  reading was quietly the wrong chart rather than an error: registry
  entry 23's failure in a softer form. `system_for` is now the one place
  that question is answered.

  Two fields simply hold: `cusp_sign_index` is the sign each cusp's own
  longitude falls in, exactly, on all 852, and the recorded midheaven is
  the SDK's within 0.0022°.

  The crate also ships the classifications `strength` and `rules` will
  both want — kendra, panapara and apoklima **partitioning** the twelve,
  with trikona, dusthana and upachaya cutting across them — and a
  house's lord taken from the sign its **middle** falls in, because
  under an unequal division a house can begin in one sign and be centred
  in another.

- `crates/points`, points that are not bodies but behave like them, and
  the measurement that decided them first. 25 tests.

  **Numbers:** the SDK computes an upagraha and a special lagna for the
  first time. Nothing moved.

  The corpus records these answers and every input they are made of, so
  the pass is the sharpest kind the project can run: a formula
  reproduces a recorded value or it does not. **Six of the eight rules
  reproduce it exactly**, to the last bit of a double, over all 71
  fixtures that carry them — the five upagrahas the Sun casts, and the
  Yogi point with its Avayogi. The project's research page marks every
  one of them "verify"; this is that verification.

  - **The five the Sun casts are a chain**, not five offsets, and two of
    its steps are reflections — so Dhuma, Indrachapa and Upaketu advance
    with the Sun while Vyatipata and Parivesha retreat. Written as five
    constants added to the Sun, two would carry the wrong sign and look
    right; `solar::advances` states each direction and a test measures
    it.
  - **The Sree lagna is a fraction of a circle**, not of a sign. The
    three rival readings are wrong by tens of degrees.
  - **Gulika begins Saturn's eighth and Mandi ends it.** Two "verify"
    marks stood over these — where inside the portion each is read, and
    whether the names differ at all — and the pass answered both by
    **deriving** rather than proposing: all twenty-four candidate
    instants tried against every recorded value. They are two readings
    of one portion, and it is start against end rather than the
    start-against-middle the sources are usually said to divide over.
    The eighth's index the catalogue already carried; a night birth
    walks it five weekdays on.
  - **The special lagnas start at the Sun *at birth***, not at sunrise
    as most statements of the rule say.

  One bracketed difference, and it identifies itself. The hora, ghati
  and pranapada lagnas are exact on 46 of 71 and out on the rest by an
  amount **proportional to each one's rate** — 0.82° at 30° an hour,
  1.63° at 60°, 2.04° at 75°. Three rules wrong in proportion to their
  rates are three *right* rules reading one wrong clock, and the pass
  confirmed it: the three imply the same elapsed time as each other on
  every fixture, and against that time all three are exact. The engine's
  clock is at most 1.633 minutes from the ishtakaal recorded beside it
  (registry entry 25); the SDK uses the ishtakaal it computes.

  **The Varnada does not ship.** Every recorded value is a whole sign,
  which is a fact about the field worth having; the received rule
  reproduces 40 of 71 and no variant built from the same parts does
  better. That is crux C22 — five published schools disagree — met in
  the data. The catalogue keeps the `VARNADA_LAGNA` key for whoever
  cites a school, as it keeps 38 other point rows that have no formula
  because the corpus records none of them.

  The two day-division points need the **ascendant at an instant that is
  not the birth**, which is a sidereal time and a latitude rather than
  an ephemeris — so the module takes it through an `Ascendant` trait and
  stays testable without a provider. `GhatiPala::to_hours` was added to
  `crates/time` for the elapsed time the lagnas read.

- `crates/aspect`, which bodies reach which, and the measurement that
  decided it first. 54 tests.

  **Numbers:** the SDK computes a drishti for the first time. Nothing
  moved. `crates/state` moved one reading in the *other* direction: the
  three lajjitadi it reported as undecided everywhere now answer
  `Holds::No` with certainty where the tradition's own necessary
  condition fails — 1301 of 1953 open questions closed, none of them a
  state the recording engine records.

  **The corpus records no aspect.** This is the first Phase 4 module it
  cannot check, and the pass established that rather than assuming it,
  by searching every key of all 115 fixture files for a name an aspect
  could have been recorded under. So `cargo xtask aspect` measured what
  it still could, and it was worth running:

  - **It refused a claim the design had made.** Two grahas aspecting
    each other fully need not be in the seventh from each other: 48 of
    the 1020 sign pairs where they reach each other fully are not, and
    they are two configurations. **Saturn three signs after Mars**
    stands in Mars's fourth and holds Mars in its own tenth, both full,
    on 7 of the 93 recorded charts. The other is Jupiter with itself
    across a trine, which is arithmetic and not astrology — and is why
    `Aspects::mutual` takes a pair of bodies rather than of positions.
  - **The two systems are not variants of one thing.** Over 6696 ordered
    pairs of bodies the graha drishti and the Jaimini rashi drishti
    agree on 4053, and each sees relations the other does not. A rashi
    drishti always looks back; a graha drishti almost never does.
  - **Every recorded placement is in its whole-sign house**, whatever
    bhava chalit its fixture's settings name (registry entry 24). So a
    drishti counted from the sign and one counted from the recorded
    house are the same relation over this corpus and part on any chart
    whose houses come from cusps. The module counts from the sign.
  - **A drishti is a step function**, so a relation carries the distance
    from both ends to the sign edge that decides it and ships no
    threshold: moving every body the engine flagged as near an edge
    across it changes 52 of the 6696 relations. `Boundaries` moved down
    into `teistro-core` for this, since `state` and `aspect` both want
    it; `state::boundary` re-exports it and nothing above changed.

  **The sphuta drishti does not ship.** The degree-based value Drik Bala
  weighs has no construction written down anywhere in this project: the
  research page records that one exists and never gives its terms, and
  the corpus has no drishti value to fit one to. `aspect.drishti_table`
  resolves `PARASHARA` and refuses anything else by name, the thirteen
  values any construction must reproduce are published, and the question
  is registered as crux C45. `Strength::virupas` is the *whole-sign*
  value in the same unit, which is those thirteen values and not an
  interpolation between them.

  **A node's aspect beyond the seventh stays `NONE`**, the root's
  reading, because nothing in the corpus prefers one. The knob's other
  two readings make a node a special graha with a chosen pair of houses,
  exactly as Mars, Jupiter and Saturn have fixed ones.

  What the pass gave back to `crates/state` is the larger half. It
  retried the three lajjitadi that module had refused, with a real
  drishti, over the same 651 readings. No rule is exact — and none
  misses a single recorded reading, while adding the aspect clause makes
  every one of them worse. So the tradition's condition is *necessary*,
  the recording engine applies something narrower, and whatever that is
  it is not a drishti. `Lajjitadi` now carries three lists rather than
  two and `Holds` names the three answers.

  Every shipped profile is tested for naming a drishti table that
  resolves, which is the `state` lesson (registry entry 23) turned into
  a test rather than waited for.

- `crates/state`, what a graha *is* as opposed to where it is, and the
  measurement that decided it first. 43 tests.

  **Numbers:** the SDK computes a planetary state for the first time.
  Nothing moved. One reading of the recording engine's changes by
  design, and it is the shallower one: **no body is ever deeply combust
  under the SDK's default profile.** Both shipped orb tables carry the
  Surya Siddhanta's degrees of time as the outer orb and only `BPHS`
  gives a deeper orb inside it; the text gives none, and the default
  profile is the texts as read (ADR-0024). Over the corpus the same 66
  of 837 readings burn, and the 36 the engine calls deeply combust come
  back merely combust. Set `state.combustion_orbs` to `BPHS` for the
  other reading; `conformance-baseline` already resolves to it.

  `cargo xtask state` proposed a rule for every recorded state field and
  measured it over **837 readings of nine grahas on 93 fixtures**,
  before a line of the crate existed, writing
  `03-design/state-tables-measured.md` (held by `check-state`). It
  settled two rules that a reading of the texts alone gets wrong:

  - **Moolatrikona is tried above exaltation.** Three grahas have a
    moolatrikona span inside their exaltation sign, so taking exaltation
    first is wrong on every reading in one.
  - **The five ages alternate.** Six-degree fifths run forward in an odd
    sign and backward in an even one; reading them forward everywhere is
    wrong on 359 of the 837.

  And it **refused** six avasthas rather than fitting them. The deeptadi
  below its top three, and three of the six lajjitadi, are not a
  function of anything a founded chart holds — combustion,
  retrogradation, the war, the house, the navamsha, a conjunction and
  the drishti were each tried and each falsified — because their
  definitions read "or aspected by". `deeptadi` returns an `Option` and
  `Lajjitadi::undecided` names the three on every reading. They wait on
  `aspect`.

  Two differences are asserted as differences and registered: the deeply
  debilitated body the engine records as dreaming rather than asleep
  (entry 21), and the lagna, which the engine gives a dignity and
  placeholder friendships that do not even compound to what it reports,
  and which this crate does not treat as a graha at all (entry 22).

  Building it found that the SDK's **default profile named a combustion
  table the SDK did not ship** — `parashari-classical` has set
  `state.combustion_orbs` to `SURYA_SIDDHANTA` since ADR-0024 and only
  `BPHS` was ever written, so a chart founded on the default profile
  failed with `UNSUPPORTED`. Both tables now ship, the citation read
  literally, and a test holds their six outer orbs to the same numbers
  `astro`'s heliacal visibility reads from the same verses, so the two
  copies cannot drift.

  `boundary` reports the **distance** to the nearest sign, nakshatra and
  pada edge and ships no threshold: the corpus brackets the engine's own
  between 21 and 43 arcseconds, and a constant compiled into the library
  could not answer "is this classification safe" for two providers of
  different accuracy. The tolerance is the caller's to state.

- `crates/vargas`, the divisional charts, and the measurement that
  decided them first. 41 tests.

  **Numbers:** the SDK computes a divisional chart for the first time.
  Nothing moved; what it computes was decided before it was written.

  A varga is the one thing in the SDK the corpus can settle **outright**.
  Everything else is a comparison within a tolerance against another
  implementation's numbers; a divisional chart is a function of one
  sidereal longitude, and the corpus records the longitude *and* the
  answer. So `cargo xtask vargas` derives each chart's table from the
  corpus, independently of this crate, and holds the design's proposed
  rule to it. It writes `03-design/varga-tables-measured.md`, which
  `check-vargas` holds.

  **The rule survived 19 530 recorded placements over 93 fixtures** — the
  55 charts and the 38 variants that carry divisional sections — with
  nothing left over and no cell pinned twice with different answers. Four
  of the fixtures are *tropical*, which moves every longitude
  twenty-four degrees and lands the bodies in different parts of
  different signs, so the rules are not fitted to one zodiac.

  One evaluator, twenty-one rows: sort the sign into a group, take the
  part of the sign the longitude falls in, and either step through the
  signs (`(a·rashi + step·part + offset) mod 12`, with `a` nought, one or
  the chart's own number) or read the answer off a list. Eighteen step
  and three list; there is no second family, only two ways to fill one
  table. D5 and D30 send their five parts to the **same five signs** and
  differ only in how wide the parts are.

  Three corrections the measurement made to the design page:

  - **The spans belong to the group, not to the chart.** D30's odd signs
    are cut 5, 5, 8, 7, 5 degrees and its even signs the same widths
    reversed, so one chart has two span rules. The page's schema had
    `spans` above both and could not say it. The boundaries were read off
    the corpus rather than assumed: each is bracketed by the last
    placement giving the sign before it and the first giving the sign
    after, and every one is settled by a single whole degree.
  - **`divisions` names the chart and is not always its part count.**
    D30 is called thirty and cuts a sign into five.
  - **Vargottama is a property of a body, not of a graha.** The engine
    computes it for the grahas alone — all 837 readings agree with the
    definition and no lagna is ever marked, though on two recorded charts
    the lagna's navamsha sign *is* its rashi sign. Entry 20 of the
    deliberate-difference registry; the SDK answers for whatever it is
    asked about.

  The crate carries the kernel (`scheme`, `evaluate`), a whole chart of a
  founded moment with the mixed `(grahas, lagna)` axis (`chart`), and the
  varga change search (`change`), which uses `astro::events`'s lattice
  where the parts are equal — twenty of the twenty-one charts and every
  arbitrary D-N — and bisects on the sign for the one chart where they
  are not. Arbitrary D-N takes the cyclic (parivritti) convention that
  `vargas.unattested_dn` names, accepts 1 to 300 and refuses the rest by
  name, and carries the convention in the value's own key.

  The part index is integer arithmetic on the canonical angle (ADR-0016).
  The engine computes it as `floor(deg / (30/N))` in floating point, and
  none of 30/7, 30/11, 30/27 or 0.2 is representable, so a body exactly
  on a part boundary can land either side of it depending on the
  platform. The exhaustive test asserts every boundary of every part of
  every chart in nanoarcseconds, along with the design's other seven
  invariants over all 16 728 cells.

  Also: `xtask`'s two falsification passes now share one `measure` module
  — a claim, a verdict and a page that stays readable however wide the
  numbers turn out to be — rather than each carrying a copy.

- `crates/panchanga`, the daily panchanga built to the design the
  falsification pass produced. 59 tests.

  **Numbers:** the SDK computes a daily panchanga for the first time, so
  nothing moved; what it computes is measured below.

  `limb` finds the four moving limbs in **three** crossing searches, not
  four: a karana is half a tithi, so the six-degree lattice's crossings
  contain the twelve-degree one's. The window is widened by a day and a
  half — longer than any tithi or nakshatra transit — so the first and
  last spans carry the member's **own** bounds and not the window's,
  which is the fact the corpus loses. One source serves all four: the
  provider is asked for tropical longitudes and shifted by the ayanamsha
  *at each instant*, because a fixed shift is two seconds of the Moon's
  time out over a search window, wider than the tolerance the corpus
  declares. The karana is named from its half-tithi of the lunar month
  rather than from the previous karana, so a list that starts mid-month
  is still right.

  `period` divides two arcs and nothing else. The three inauspicious
  eighths are a table; the sixteen choghadiya are the hora's weekday walk
  over a seven-row table of names, which is `time::hora::lord_of` and not
  a grid of fifty-six; the thirty muhurtas are fifteenths, with Abhijit
  named and void on Wednesdays and Brahma muhurta sized from the night
  that **ends** at sunrise. A period whose arc does not exist is absent:
  on a polar day the lists are shorter, where the recording engine writes
  twelve intervals of no length.

  `omen` reports panchaka and the muhurta yogas as **intervals**. A flag
  is what an interval reduces to — "some interval contains sunrise" — so
  the interval is strictly more information and costs nothing, the spans
  being already computed. `sky` reports the Moon's rises and sets as
  lists, because a 24-hour window holds none, one or two of each.
  `almanac` assembles and stamps, by date, by an instant's day, or by a
  range, which is the primary shape.

  New in `core`: `interval::Interval`, with the one divider every period
  is — `divided`, `part`, `part_at`, `clipped_to`, `fraction_at` — half
  open so that a sequence of parts is a partition and an instant belongs
  to exactly one hora. It is in `core` rather than in the module that
  needed it first because dashas, muhurta windows and transits are
  intervals too.

  New knobs: `panchanga.centre` (an almanac is geocentric wherever the
  chart is), `panchanga.moon_events` and `panchanga.muhurta_tables`.
  `conformance-baseline` becomes version 3, setting `moon_events =
  CIVIL_DAY`. New catalogue kinds: `choghadiya`, `kaala`, `panchaka` and
  `muhurta_yoga`, with names in all five locales. And `day.day_boundary`,
  declared in Phase 1 and read by nothing, gets its first reader: it
  moves the window the limbs are clipped to and moves no period, because
  a choghadiya divides the daylight whatever the window is.

  **Against the corpus**, computed from the arcs the corpus itself
  recorded: 159 inauspicious eighths, 848 choghadiya with their lords,
  1272 horas, 53 Abhijit muhurtas, 55 lunar months, 9 panchakas, the
  ayana and the disha shool — every one within 0.04 milliseconds, which
  is the last bit of a double near two and a half million. The
  classification of a real position is checked against the Sun and Moon
  recorded at all 55 birth instants, into all four limbs and their
  attributes. Brahma muhurta is asserted **different** by the registry's
  own amount: a median 10.02 seconds and at worst 27.56.

  What is not yet tested is the limb *instants*: a tithi's boundary is a
  position over time and no provider inside the workspace has real
  positions. Those, and the rank-1 comparison against the eight printed
  tithi ends of Nepal's national panchangam, wait on the conformance
  harness over an adapter.

- The daily panchanga falsified, then designed. `panchanga_day` is the
  largest section of the conformance corpus and the only one nothing had
  read: twenty-seven fields a day, none of which says how it was
  reckoned. `cargo xtask panchanga` proposes a rule for every one of them
  and measures it over all 55 recorded days, writing
  `03-design/panchanga-day-conventions.md`; `check-panchanga` holds the
  page, so its numbers are what this build produces.
  `03-design/panchanga-day.md` is the design written from the result.

  **Numbers:** none moved; this measures numbers that were already there
  and designs a module that does not yet exist.

  What holds, exactly, on every recorded day: the window is sunrise to
  the next sunrise (all four limb lists begin and end at one instant, to
  0 s); every period is an equal division of the daylight or of the night
  (the eighths to 7e-9 of an eighth, the choghadiya and the horas to
  0.04 ms); a choghadiya's lord is the hora's weekday walk, over 1320
  horas and 880 choghadiya with no exceptions; Abhijit is the eighth
  muhurta of the daylight and void on Wednesdays; the purnimanta month is
  the amanta month plus one through the dark fortnight; panchaka's kind
  is its nakshatra's; and the SDK's own catalogue reproduces the engine's
  attribute tables — the tithi's paksha and class, the nakshatra's
  muhurta nature, the yoga's auspiciousness, the karana's Vishti flag —
  member for member.

  What it falsified, each now a row of the deliberate-difference
  registry. **Brahma muhurta is sized from the wrong night** (entry 17):
  it ends before sunrise, so it belongs to the night that ends there, and
  the engine sizes it from the night that follows the day — a median
  10.0 s and at worst 27.6 s out. **The Moon's rise and set are the civil
  day's** (entry 18): every other field is bounded by sunrise and these
  two are the first at or after local midnight, so 24 of the 108 recorded
  events fall outside the window the limbs occupy. **The tithi is
  numbered through the month** (entry 19), one to thirty, where the SDK's
  catalogue numbers it within its paksha, one to fifteen — not a
  disagreement but two fields, and a harness that compares them without
  saying which is comparing nothing.

  And one limb the corpus cannot settle at all: the five muhurta yogas
  fire 13 times on 12 days, which cannot derive a seven-by-twenty-seven
  table, and the engine's positives match no published table under any
  rotation of the weekday or the nakshatra index. The design ships them
  as cited tables with confidence marks and reports them as intervals,
  since the corpus's flags reduce to "some interval contains sunrise".

- The foundation's birth timing and its stamp. The ishtakaal and the
  planetary hour are fields rather than calls, both counted over the arc
  of **the day the chart belongs to** — so a birth in the small hours is
  late in its own day rather than early in the next, which a test asserts
  in both directions. Bhayat and bhabhoga are still absent, and the type
  says why: they are the Moon's nakshatra transit and belong to `dasha`.

  A foundation now comes back inside an `Envelope`, as every result of
  the SDK does: the profile and its settings hash, the calculation and
  catalogue versions, the ephemeris that placed the grahas with the frame
  it answered in and the steps the SDK completed itself, the Delta T
  model and the leap-second table, and a hash of what was asked — which
  tells two questions apart and two askings of one question together. A
  batch carries one stamp, because everything in it was founded under the
  same settings by the same provider.

  **Numbers:** none.
- `ChartFoundation` and `Founder`: the value the three modules beneath
  them were built for. One moment at one place, with the day it belongs
  to, the zodiac it is measured in, the lagna and the sunrise that
  anchors it, **both** divisions of the sky — the placement system and the
  chalit, which are different questions — and every graha placed in each,
  carrying both readings of its longitude and the method that placed it.
  `Founder` is built once with a provider and a profile and founds as many
  charts as are asked for, because batch is the primary shape; Ketu is
  derived as Rahu's opposite point rather than requested, because no body
  is Ketu.

  **Numbers:** none. The test provider is analytic and nothing here is
  compared with an ephemeris; what the tests assert is what only the
  assembled value can show, and each is one of the design's own claims —
  every graha stands in the same zodiac as the cusps it is placed
  against, a placement's `through` and `from_madhya` describe the bhava
  it names, the lagna falls in the first bhava of its own division, the
  day holds the instant, Ketu is half a circle from Rahu with the same
  speed and no distance, and founding a moment twice or in a batch gives
  the same value field for field.

  `Body::graha()` moved into the port that owns `Body`, where the
  siddhanta provider had a private copy of the same mapping.
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
