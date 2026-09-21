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

**Every binding ships six worked examples** (`bindings/*/example/`), the
same six scenarios in JavaScript, Dart and Python, each a program
`cargo xtask check-{node,dart,python}` runs — every file in the
directory, so a scenario added is a scenario gated. They are the
quickstart, a Nepali birth record placed by sign, nakshatra and pada, the
five limbs of a panchanga, a Bikram Sambat year as a calendar page, a
year of the sky in one call, and an ephemeris of your own. Writing them
was a falsification pass over the three ergonomic layers and it found
nine gaps the parity gate could not: the parity gate compares values, and
these were shapes. Node had no examples at all, no `dispose`, no date or
zone constructors, no `jdCount`, and named the provider's coverage
differently from the other two — and the `dispose` it gained then
reported a call on a freed context as the boundary's bare `invalid
argument`, where Dart and Python both name the context; all three now
name it, and all three now gate it. `whenUnknown` promised in all three
that a resolution "reports rather than guesses" a missing time of day,
where in fact it **refuses** unless the profile sets
`time.unknown_time`; the three docs now say what happens, the birth
chart example shows all three policies, and Python's `context` takes a
`settings` mapping like the other two rather than only a JSON string a
caller had to serialise by hand. The Dart layer wrapped a provider's own
exception in the library's; the Python layer's weekday docstring named
the wrong day as one. Every enum in the TypeScript surface now carries
its id table, not just the two that appear in blob columns.

**A provider's refusal now reads the same in all three bindings.** The
port's `validate` moved to the SDK's side of the boundary
(`VtableProvider::positions`), which is where it had to be: only a code
crosses back, so a refusal raised out in the binding arrived as a number
and the sentence naming the body was lost, and each of the three
bindings had grown its own copy of the policy to get the words back.
Checked on this side, the words survive into every binding at once and
no binding keeps a copy — a body the provider never declared is now
`the provider does not support MARS; it answers SUN, MOON` in
JavaScript, Dart and Python alike, with the status `unsupported` to match
on. What a provider raises on its own side is now given back to its
caller as itself, in all three: the `StateError`, the `RangeError` or the
`FileNotFoundError` the provider wrote, with the library's own refusal
kept as its cause where the language has one, rather than a summary of
it. Coverage stays what the port always said it was — a per-cell
`CellStatus::OutOfRange`, not a reason to refuse a batch — so a year
whose last day runs past the ephemeris keeps the days it can compute; a
binding that would rather refuse early does it in its own adapter and
says so in its own language. Every column a provider supplies is now held
to the cell count, not only the three it must supply, because a speed
column of the wrong length silently padded with zeroes is a wrong answer.

**The mark crosses the boundary.** The panchanga blob's `days` section
gains a `month_kind` column, the three ergonomic layers expose it as
`month.kind` on a day, and `check-parity` compares it: **597** values
across three bindings where it compared 594.

`TsMonthKind` converts **exhaustively**, unlike the three settings knobs
this module also carries. A knob is `#[non_exhaustive]` on purpose, so a
match on one needs a wildcard and a member added later would cross as
whatever came first — which is why `TsSunrise` and its kin convert
fallibly and are guarded by a test over the knob's own `ALL`. `MonthKind`
is the calendar's own and closed, so a kind added there stops this crate
compiling instead. The stronger guard where it is available.

**A lunar month now carries its mark.** `crates/calendar` gains a
`lunisolar` module — a `LunarModel` trait beside `SolarModel`, a
three-way `MonthKind`, `kind_of` for a span whose bounds a caller already
knows, and `month_at` for one that does not — and `panchanga`'s
`LunarMonth` carries the kind beside the name. The rule is the one
`03-design/calendar-indian-lunisolar.md` designs and the measurement
holds: none, one or two sankrantis in the month.

A `LunarModel` of its own rather than a method on `SolarModel`, because a
solar calendar needs no Moon and five implementations of that trait —
three of them test doubles — would have to grow one for nothing. The two
must answer in the same sky, which no type can enforce, so the functions
take them together and say so.

**`panchanga` finds the month's bounds once** where it used to find its
opening new moon and throw the search away. `limb::lunar_month_span`
returns both new moons, `limb::masa_at` names the month from the Sun at
the first, and the kind comes from the sankrantis between them — one
crossing search serving the name and the mark instead of two serving one
each. It applies the rule over **its own** sky rather than the text's,
because the month it marks is the one it named.

**The rule that decides adhika and kshaya is measured** (`cargo xtask
lunisolar` → `03-design/calendar-indian-lunisolar-measured.md`, gated by
`check-lunisolar`). `panchanga` names the lunar month and cannot mark it,
and no other module could either; this is the falsification pass the
calendar that will mark it is designed from.

**The corpus cannot settle it alone**, which is the first thing the pass
had to establish. It records `is_adhika` on every day — the *answer* —
and none of the inputs: not the new moon that opened the month, nor the
sankranti that named it. So unlike the panchanga's conventions, which are
arithmetic over recorded numbers, this has to compute the sky. It
computes it from the **Surya Siddhanta**, as the Bikram Sambat engine
does, so the calendar needs no ephemeris — which is also what the
tradition itself did.

Over **12 368 lunar months of a millennium**: 388 hold no sankranti
(adhika, one every 2.58 years against the classical seven in nineteen),
11 961 hold one, and 19 hold two (kshaya, one in some fifty years). The
rule is that count, and it reproduces the corpus's marking on **all
fifty-five** recorded days, including the two marked adhika — Delhi in
August 1947 and Fairbanks in June 2015.

Two things the measurement settled that reasoning had got wrong.
**An adhika month needs no naming rule of its own.** The usual
formulation is that it takes the following month's name, because it has
no sankranti to be named by; in fact the Sun stands in the same sign at
its new moon as at the following nija month's, so the existing rule gives
both the same name unaided — August 1947 is Shravana twice over. And
**the classification is robust where the month of an instant is not**:
whether a sankranti falls inside a 29.5-day window barely moves when a
boundary shifts by half an hour, but which month a moment belongs to
does. The one recorded day where the text and the recording disagree is
nineteen minutes from a new moon — the eclipse conjunction of 8 April
2024, with the text's instant and the recording's on either side of it.

**And the pass answers the design's central question before the design
asks it.** A lunisolar date's *day* is the tithi running at sunrise, and
a tithi runs twenty-three to twenty-six hours — so it can catch two
sunrises or none. Over 365 234 sunrises at Ujjain the day repeats **one
day in forty-four** and is skipped **one in twenty-six**, so
`(year, month, day)` is not a key: it names two days sometimes and none
at others. A Hindu lunisolar date needs **two** flags — which of a
repeated pair a day is, and whether its month is the adhika one — and
`CalendarDate` carries neither. The calendar therefore cannot be another
`CalendarSystem` without either extending a value that crosses the
boundary and reaches three bindings, or giving `day` a different meaning
and saying plainly that the result is not the date a panchangam prints.
The measurement does not decide which; it establishes that the question
cannot be dodged.

**Kshaya is measured and not tested**, and the page says so rather than
implying otherwise: the corpus records none, so nothing holds the rule to
an authority. What can be said is that its frequency is what the
astronomy predicts and that every one of the nineteen falls between
Vrishchika and Kumbha — the perihelion window where the Sun moves fast
enough to cross two sign boundaries inside one lunar month. Nothing in
the pass looks for that; it falls out of the counts, and a rule producing
a kshaya month in Karka would fail it.

**The parity gate now compares 594 values across the three bindings,
where it compared 103.** All 103 came from `positions`; the chart and the
almanac add 491, and until they did, "the bindings agree" meant something
for one entry point and the chart and almanac examples agreed only
because a person had compared their output by eye.

The scenario needed a second context to say it. The gate runs under
`nepali-default`, whose frame is **topocentric**, and a chart cannot be
founded under it at all — the completion's centre step is Phase 3's. That
refusal is now a compared value in its own right (`chart-under-topocentric`),
so the three bindings must fail the same way and not merely succeed the
same way; everything after it runs on a second context under the SDK's
own geocentric default.

Two instants and three days, deliberately. A per-chart section laid out
charts-outermost the wrong way round shows up as the second chart's
values in the first's place rather than as nothing at all, and a day's
lists are ragged — two consecutive days with the same counts would not
exercise the offsets at all.

**The gate found a gap on its first run.** When `part` and `elapsed` moved
out of the shared day section into the chart's `cast` — they belong to an
instant, not to a day — the columns were added and **no layer surfaced
them**. Node had nothing at all; Dart and Python could only reach them by
indexing the raw column. All three now expose `dayPart` and `dayElapsed`
on a chart, which is where they belong, and the runners read the accessor
rather than the column.

**A daily panchanga crosses the boundary, as a batch of days.**
`ts_panchanga_days` takes a **range** — not a list of dates, because
consecutive days share a boundary, so day *n*'s next sunrise is day
*n+1*'s sunrise and a month costs much less than thirty days computed
separately — and answers with one blob holding the four moving limbs, the
periods, the lunar month under both conventions, what the Moon and the
Sun did, and what each day is said to be. Every per-day list is
concatenated across the batch with a `counts` section saying how many
rows are each day's, which is the layout the measurement settled: one
rule for all thirteen, including the five whose length never moved.

Three things it decides that a chart blob never had to. **A value a day
may not have crosses as a presence flag beside it** — an absent Abhijit
and an Abhijit at Julian day zero are both nought, so no sentinel can
serve. **Two lists answering one question in two halves become one
section with a discriminant**: the muhurtas of the daylight and of the
night, the moonrises and the moonsets. And **the seven span lists do not
share a shape**, which is the first place that rule needed a boundary: a
shape makes two sections decode to one type, which is right for the same
section in two blobs and wrong for a span of tithis and a span of
nakshatras — same structure, different meanings, and one type for both
would let a caller pass either where the other is wanted. A shape is for
sameness of meaning, not similarity of structure.

**The shared day section carried two fields that were never a day's.**
`day_section` held `part` — which arc of the day an instant falls in —
and `elapsed` — how far through that arc it is. Both belong to an
**instant**: a chart has one and a panchanga day has none, so the
panchanga could only have filled them with a lie. The design page had
said all along that a chart's day and a panchanga's are the same
*eighteen* fields while the section held twenty, and nothing noticed
because a chart was the only blob carrying a day — a field that is wrong
for a reader who does not exist reads as right. They now sit in the
chart's `cast` section with the other things an instant decides.

All three bindings gained `almanac(range)` and `almanacDay(one)` over the
one crossing, with each day a view over its batch and its ragged lists
resolved by a prefix sum done **once** when the batch is decoded, rather
than per access — the alternative is quadratic over the year of days an
almanac is actually asked for. Each ships a week's panchangam as a worked
example, and the three print byte-identical pages.

Two gaps the examples found and the SDK now records rather than papers
over: a binding's `has(key)` answers for a *message* and there is no
non-throwing way to ask the same of an **entity**, so all three examples
catch a refusal to do it; and **`masa` and `direction` have no name in
any of the five entity packs**, so an almanac cannot print the lunar
month or the disha shool in the reader's language — the two things a
panchanga page leads with.

**The shape of a batch of almanacs is measured** (`cargo xtask almanac`
→ `03-design/panchanga-at-the-boundary-measured.md`, gated by
`check-almanac`), which is the falsification pass the panchanga blob's
layout is designed from. A chart needed no such pass: its per-chart
sections have a stride the blob states once. A day's do not, and the 450
days the pass founds at three latitudes split its fifteen lists cleanly
in two — a split the names do not suggest. **Every fixed list is a
division of an arc** (twenty-four horas, fifteen muhurtas of the daylight
and fifteen of the night, sixteen choghadiya, three kaalas) and **every
ragged one a crossing inside the window** (a tithi boundary, a moonrise,
the Moon entering a sign). A division's count is a convention, so a polar
day whose synthesised arc runs for a fortnight still has twenty-four
horas — each of them fourteen hours long — while its crossings multiply
to sixty-two tithis and a hundred and twenty-two karanas. That decides
the layout: a rectangular one wastes 10.4% of its rows over ordinary days
and **78.1%** once a single polar day joins the batch, because one such
day sets the stride for every other.

**A Moon that does not rise is an answer; a step budget is not.** The
horizon scan's bracket cap was a constant four hundred, described in its
own comment as "a day of ten-minute steps" though four hundred of them is
two and three-quarter days. A polar day's synthesised arc is longer, so
the panchanga's moonrise search over one met the constant rather than the
horizon and refused with `NOT_CONVERGED` — at both solstices at 69.65°N,
which is every polar range the pass asks for. The cap is now sized from
the span the scan is asked to search, with four hundred as a floor. It
changes only calls that previously failed, so no number moved; until it
was fixed the SDK could not produce a polar day at that latitude at all,
and the five fixed lists looked fixed because nothing had asked them a
hard question.

**A founded chart crosses the boundary, and it crosses as a batch.**
`ts_chart_found` takes a grid of instants and answers with one blob of
charts founded at one place — the grahas placed under both readings, the
twelve bhavas and the chart's chalit, the zodiac, the day and the birth
timing. Every per-chart section runs **charts outermost**, as the
positions blob puts instants outermost, and a batch of one is the
ordinary case. The founder already had the batch form and it was sitting
unused behind a single-instant entry point: the settings resolve once,
the solar model is built once, and the day each instant belongs to is
reckoned against the same sunrise, so a hundred candidate birth times
cost one setup rather than a hundred. A rectification pass is what that
is for, and each binding now ships one as a worked example — the same
program in Node, Dart and Python, printing the same eighteen lagnas.

Each binding's layer offers both shapes over the one crossing:
`found(one)` answers a chart and `foundMany(list)` a batch, with the
charts in it views over the blob's bytes rather than copies. A batch of
none is an empty result rather than a refusal, so a caller who filtered a
list to nothing does not special-case it. The Node binding's TypeScript
surface had **no chart in it at all** — neither `found` nor the class it
returned was declared, so the ergonomic layer's chart path was invisible
to a type checker and its two byte-section accessors had never run; both
double-decoded text the decoder had already decoded.

**A section can name its shape when it is a column section, not only a
fixed one**, which is what a chart's day becoming one row per chart
required: the day a chart carries and the day a panchanga will carry stay
**one** decoded type in each binding rather than two identical ones.
`check_shapes` now refuses two sections that name one shape and disagree
about its kind, its fields or their scalars, because the emitters render
a shape once and would otherwise decode the second through the first
one's type — silently, since both are valid blobs. `Writer::rows` writes
a column section from rows of values, the shape a fixed section takes
repeated, so a per-chart section reuses the one function that knows the
field order; it narrows each value to the scalar the schema declares and
**refuses** one too wide for its column rather than truncating it, which a
fixed section's eight-byte slots cannot catch.

**The chart document's shape is measured** (`cargo xtask schema` →
`03-design/schema-measured.md`, gated by `check-schema`), which is the
falsification pass a JSON Schema for it is designed from. The sample is
built rather than recorded — three document shapes founded over the
analytic provider — because a recorded one goes stale the first time a
section gains a field. All five proposed rules were falsified, and three
of them matter beyond the schema.

**A stored chart reads back.** 60 of the chart layer's 65 types now
derive `Deserialize`, and the five that do not are the five that cannot:
a value whose identity is a shipped constant, holding a `&'static` no
document can produce — a divisional scheme's group table, its listed
signs, an aspect angle's key, the drishti table a chart was read under.
Each has a reader written by hand that reads the value back **by its
identity**, looking the constant up in this build's own table and
checking what the document says about that table against it. A document
that names D9 and describes something else is refused by name, which is
stricter than a derive would have been and is what a stored chart wants.
`canonical::from_hash_form` reads one and `canonical::reads_back` asks
whether a value survives the trip; every sample document now validates
and reads back equal, in bytes, in value and in hash.

**The generated catalogue readers had never worked, and nothing had
tried them.** All sixty used `<&str>::deserialize`, which needs a string
borrowed from the input buffer, so a catalogue enum could be read from
text and never through a `serde_json::Value`. They take a `Cow` now and
are `DeserializeOwned`.

**Numbers: none moved, and three comparisons got sharper.**
`teistro-core` now asks for `serde_json`'s `float_roundtrip` feature.
The default float parser is a fast path that is not correctly rounded —
it read 84 of a chart document's 1518 numbers a unit in the last place
low — and a reader on it cannot reproduce the hash it exists to check.
Because the conformance corpus is JSON too, it had been read the same
way, and several comparisons against it were measuring the parser rather
than the SDK. Nothing the SDK computes changed; what moved is how
closely it is now seen to agree:

| claim | before | now |
|---|---|---|
| the choghadiya are eight equal parts of the daylight and eight of the night | worst 0.040 ms | **0 s, exactly** |
| the horas are twelve over the daylight and twelve over the night | worst 0.040 ms | **0 s, exactly** |
| Brahma muhurta is sized from the night after the day | worst 0.040 ms | **0 s, exactly** |
| rahu kaal, yamaghanda and gulika are each one eighth of the daylight | worst 6.66e-9 | 4.55e-9 |
| the three clock-driven lagnas against the recorded ishtakaal | 25 of 71 disagree | 24 of 71 |

Three claims that had held to within a twenty-fifth of a millisecond
hold **exactly**. The SDK's arithmetic was right all along and the
fixture it was compared against had been read a unit in the last place
low.

`arbitrary_precision` fixes the same numbers and is the wrong tool.
Serde buffers an internally tagged enum before writing it, and that
buffer writes a number as `{"$serde_json::private::Number": …}`, which
breaks `DeltaTModel`, `CalendarResolution`, `Outcome` and every other
`#[serde(tag = …)]` the SDK has; it also leaves `from_str` wrong, so a
reader would have had to go through a `Value` to be correct.
`float_roundtrip` has neither cost. The feature is global to a build, as
`preserve_order` already is, so the defence is a test rather than a
declaration: one that fails if it is ever off.

**Numbers: every content hash moves. No computed value does.** The
canonical form now writes each number as the **shortest** decimal that
reads back as the same double, where it wrote a fixed twelve decimals
before. Nothing the SDK computes has changed — the same longitudes, the
same instants, to the last bit — but the bytes they are written as have,
and `content_hash` is taken over those bytes. A hash recorded from an
earlier build will not match one taken now. **The settings hash does not
move**: no setting holds a number that twelve decimals could not write.

The old form was not a fixed point, which is the one property a content
hash rests on. Twelve decimals was chosen on the stated grounds that
"the SDK's own quantities are degrees, days and scores whose magnitudes
are under 10⁶", and that premise is false: a Julian day is 2.46 × 10⁶,
and a chart document carries the instant it was cast for. At that
magnitude one unit in an `f64`'s last place is about 5 × 10⁻¹⁰, so three
of the twelve digits were the decimal expansion of a binary value rather
than information, and a parse did not return them —
`2460483.108666389249` was written, read, and written again as
`2460483.108666389715`. A consumer that stored a document and hashed it
did not get the producer's hash.

The measurement was already there and had not been read: the canonical
form's own pass had recorded "every number of the corpus round-trips
through the form" as **falsified**, at 22 188 of 193 366, and the module
shipped. It now holds at 0 of 193 366. What made it impossible to leave
was the consequence rather than the count — the schema pass asked what a
consumer would validate and found the bytes were not stable.

The form is exact rather than lossy now. Two doubles that differ at all
are written differently, where the old form rounded them together below
its resolution. That lossiness was deliberate, and is not missed:
rounding never delivered it reliably (two values one unit apart straddle
a rounding boundary some of the time), and "would these compute the
same" is the **settings** hash's question. The content hash asks whether
the bytes are the same bytes.

**A Rust consumer must not read a document through `serde_json`'s
default number path.** It is not correctly rounded — it reads
`218.91170673806658` as the double one unit in the last place below, and
does that to about one number in fifteen — so a reader built on it
cannot reproduce the hash it is meant to check, however correct the
grammar is. `str::parse` is correct, as are JavaScript's `JSON.parse`
and Python's `json`.

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

- `crates/strength`, the strength measures, beginning with the Ashtakavarga:
  each graha's bindus, the sarvashtakavarga, and their reductions and pindas
  under BPHS's reading (the default) or the conformance corpus's engine's
  (`strength.shodhana`, `strength.ekadhipatya`).

  **Numbers:** new. Every settings hash moved with the new knob and the
  reshaped `strength.ekadhipatya`, which nothing read before; no number the
  SDK computed before changes.

  The Vimshopaka followed: each graha's strength out of 20 across the
  sixteen vargas under the shadvarga, saptavarga, dashavarga and
  shodashavarga, with BPHS ch. 7's weights, scored by the text's points
  (the default) or the corpus's engine's Saptavargaja virupas
  (`strength.vimshopaka`, crux C63), through `ChartRequest::with_vimshopaka`,
  the document's `vimshopaka` section, the boundary's section 28 and
  `chart.vimshopaka` in every binding.

  **Numbers:** new. Every settings hash moved with `strength.vimshopaka`,
  and `conformance-baseline` is version 5; no number the SDK computed
  before changes.

  The Shadbala followed: each graha's six strengths in virupas, the Sthana
  and Kaala by component, their sum in rupas and whether it reaches the
  requirement, under BPHS ch. 27's reading (the default) or the corpus's
  engine's at eleven forks, each a setting (`strength.saptavargaja`,
  `nathonnatha`, `pre_dawn_night`, `sun_ayana`, `moon_cheshta`, `kranti`,
  `kaala_lords`, `dig`, `drik`, `naisargika`, `required_rupas`; cruxes
  C64–C71), through `ChartRequest::with_shadbala`, the document's `shadbala`
  section, the boundary's section 29 and `chart.shadbala` in every binding.
  `strength.bala_scheme` is now read, and `PARASHARA_EXTENDED` is refused as
  unsupported. `Founder::angles_at` gives a chart's ascendant, midheaven and
  true obliquity at an instant, and the façade re-exports the strength crate
  as `teistro::strength`.

  **Numbers:** new. Every settings hash moved with the eleven knobs, and
  `conformance-baseline` is version 6; no number the SDK computed before
  changes.

  Then Sripati's reading, from B.V. Raman's worked Standard Horoscope:
  `teistro_aspect::sphuta`, the sphuta drishti with the special aspects;
  `ShadbalaRules::SRIPATI`, which reproduces Raman's worked Shadbala; four
  more settings (`strength.drekkana`, `benefics`, `cheshta`, `yuddha`) and
  new values for `kranti` (`HINDU_TABLE`), `drik` (`QUARTER`, the old
  `QUARTER` now `QUARTER_WITH_JUPITER_MERCURY`) and `luminary_cheshta` (which
  replaces `moon_cheshta`; `sun_ayana`'s `CHESHTA_ONLY` is `NOT_IN_KAALA`,
  `required_rupas`'s `RECORDING_ENGINE` is `SRIPATI`); a Yuddha component in
  `KaalaBala` and the boundary's `shadbala` section; and the ahargana's lords
  divided on the count including the day of birth.

  **Numbers:** the default Shadbala moves: the Drik reads the sphuta
  drishti, the benefics are conditional, the Cheshta reads Kedarnath Dutt's
  elements, grahas at war exchange a Yuddha bala, and the Saptavargaja gives
  the moolatrikona's 45 in the rasi alone. `conformance-baseline` is version 7
  and reproduces the corpus exactly as before.

  The Bhava bala followed: each bhava's lord's Shadbala, Dig, drishti and
  special rules under BPHS ch. 27's reading (the default), Sripati's or the
  corpus's engine's (`strength.bhava_dig`, `bhava_drishti`,
  `bhava_special_rules`; cruxes C73–C75), through
  `ChartRequest::with_bhava_bala`, the document's `bhava_bala` section, the
  boundary's section 30 and `chart.bhavaBala` in every binding.

  **Numbers:** new. Every settings hash moved with the three knobs, and
  `conformance-baseline` is version 8.

  The Ishta and Kashta phalas followed, on each graha of the Shadbala reading
  and its boundary section: BPHS ch. 28's rays (the default), Sripati's
  square roots, or the engine's roots of its own Cheshta
  (`strength.ishta_kashta`, crux C76).

  **Numbers:** new; `conformance-baseline` is version 9.

  The Vaiseshikamsa followed: each graha's count of good vargas and the name
  it earns in the shadvarga, saptavarga, dashavarga and shodashavarga, from
  BPHS ch. 6 vv. 42 to 53 (crux C77), through `ChartRequest::with_vaiseshikamsa`,
  the document's `vaiseshikamsa` section, the boundary's section 31 and
  `chart.vaiseshikamsa` in every binding; the thirty names are a new
  catalogue kind, `vaiseshikamsa`.

  **Numbers:** new. No settings hash moved.

  The Sayanadi avasthas followed, from BPHS ch. 45 vv. 30 to 37: every
  graha's state carries `sayanadi`, one of the twelve states and its
  sub-state under each of the five ankas a name's first syllable can have
  (`Sayanadi::cheshta(Anka)`), through the boundary's `states` section and
  every binding. The sub-states are a new catalogue kind, `avastha_cheshta`.
  Two knobs, `state.sayanadi_ghatis` and `state.sayanadi_nodes`, carry
  what the verses leave open (crux C78). A graha in Shayana is now also
  impaired in the Vaiseshikamsa (crux C77).

  **Numbers:** new, and the Vaiseshikamsa's `impaired` moves for a graha in
  Shayana. Every settings hash moved: the `state` group gained two knobs.

  The dasha phala followed, from BPHS ch. 28 vv. 5 and 7 to 10 and ch. 47
  vv. 3 to 6: the Shadbala carries each graha's Subha and Ashubha rays, and
  `ChartRequest::with_dasha_phala` reads each of the nine grahas' Subhankas
  in the seven vargas and their totals, its rasi place's nature, where in
  its dasha its effects come (`DashaPhase`), and whether its placement makes
  the dasha favourable or unfavourable, under a new knob,
  `dasha.shanta_sign` (crux C79). It crosses as the document's
  `dasha_phala` section, boundary section 32 and `chart.dashaPhala` in
  every binding. `teistro_state::dignity::varga_dignity` reads a divisional
  sign's dignity, and the Saptavargaja now shares its temporary friendship.

  **Numbers:** new. Every settings hash moved: the `dasha` group gained a
  knob.

  A consumer's own nakshatra-seeded dasha system now registers, the Phase 5
  exit's consumer-row clause. A `UduDefinition` goes on
  `ContextBuilder::dasha_system` or `TsContextOptions.dashas_json`, and
  every binding's context option takes one. Each is checked by the rules a
  shipped row passes and refused by its place and field. A request asks for
  it by key (`dasha_system.ACME_SAPTAKA`, an id from `0x8000`), and the
  answer names it so. The reading carries its `definition`, so a stored
  document rebuilds the cursor in a context that never registered it.
  `DashaReading.system` is now a `DashaName`, a catalogued member or a
  registered key, serialised as the bare key as before.
  `ChartRequest::with_dashas` takes ids, and the boundary request's
  `dashas` array is plain ids rather than an enum.

  **Numbers:** unchanged for every catalogued system; a registered twin of
  Vimshottari reproduces it to the bit.

- Phase 6 begins with the measurement: the corpus moves to 0.10.0, which
  records the recording engine's 605 yoga rules in its condition language
  and their presences on 93 charts. `cargo xtask yogas` (held by
  `check-yogas`) evaluates every rule under each reading of that language.
  The engine's reading reproduces all 55 521 decisions, 5350 presences'
  planets and every cancellation. The Moon's and Mercury's natures and
  sign-against-orb conjunction are decided; five forks the corpus cannot
  see are named; 116 rules with no positive case are listed. The page is
  what the `rules` kernel will be built from.

- `crates/rules`, the rules kernel's first slice: the recording engine's
  condition language typed (three combinators and 22 predicates over the
  grahas and the lagna) and read strictly, so an unknown field, predicate,
  house or body is refused with its path; a `RuleChart`; the seven places
  the language leaves open as `Readings`, the engine's choices the default;
  and an `Evaluator` returning each rule's presence, participants, houses
  and held cancellations. It reproduces every recorded yoga, and
  `cargo xtask yogas` now measures the built kernel rather than a private
  copy of its rules. The engine's 597 written rules over one chart take
  about 10 microseconds.

  **Numbers:** none; nothing in a chart document or the boundary reads the
  kernel yet.

- `teistro-rules` references: a condition can be about the lord of a sign,
  the holder of a chara karaka, an arudha pada, the upapada, a body's
  navamsha, or a sign counted from any of these, as BPHS chs. 29, 30, 33, 34
  and 40 write their rules. A `BodyRef` is a body and a `SignRef` a sign, so a
  rule asking for the dignity of a pada is refused when it is read. The
  engine's rules read unchanged. `Placement` gains `navamsha` and `Readings`
  gains `upapada` (crux C80). `teistro_points::arudha::pada` counts the pada
  of any sign.

  **Numbers:** none; the corpus's yogas reproduce as before.

- `teistro-rules` traces: `Evaluator::explain` returns the rule's answer with
  each condition checked, whether it held, the bodies it added and each
  reference resolved, as a tree that serialises to JSON and prints as prose.
  One evaluator runs both calls, so `evaluate` allocates no trace, costs the
  same, and cannot disagree with an explanation.

  **Numbers:** none.

- `teistro-rules` tables: a rule can look a body's degree up in a
  degrees-by-sign table (`planet-at-table-degree`) or a sign up in a
  signs-by-tithi table (`planet-in-table-sign`), each table data with its
  source. `Tables::classical` ships Jataka Parijata's Mrityu Bhagas (ch. 1
  v. 57 and its translator's table) and Pushkara bhagas (v. 58), Brihat
  Prajapatya's Moon row, and the Dagdha rashis. `Tables::check` refuses a rule
  naming a missing table or the wrong kind. `RuleChart` gains `tithi`,
  `Readings` gains `bhaga` (cruxes C82 to C84), and `Readings::default()` is
  now `Readings::TEXTS`, which differs from `RECORDING_ENGINE` only there.

  **Numbers:** none; no shipped rule reads a table yet.

- The doshas: the corpus moves to 0.11.0, which records the recording engine's
  natal dosha evaluator: 52 rules and their presences on 93 charts, with
  severity, cancellations and net status. `teistro-rules` models what a dosha
  adds once, for yogas too. A `Rule` gains reference groups, labelled
  cancellations, a `Severity` rule, a cancellation threshold, remedies and
  scope, and a `RuleResult` gains where it was found from, its severity and its
  `NetStatus`. The language gains the lord, lagna, gandanta and panchanga
  predicates, and `RuleChart.tithi` becomes `RuleChart.panchanga`. Under
  `Readings::RECORDING_ENGINE_DOSHAS` the kernel reproduces every recorded
  dosha of the 35 rules the language can say (crux C85).

  **Numbers:** none.

- The seventeen doshas the recording engine computes in code are rules:
  `teistro_rules::shipped::computed_doshas` ships Kalsarpa, its twelve named
  forms, Kala Amrita, Mrityu Bhaga, Dagdha Rashi and Badhaka, each with its
  citation and the engine's severity. Each says present exactly where the
  engine's code did on every recorded chart, and the last three reproduce every
  recorded field. `all-planets-between-nodes` takes a `side`, a `SignRef` can
  be `{"badhakaOf": …}` (crux C86), and a group can carry a `weight` a
  count-based severity reads. `cargo xtask doshas` writes
  `03-design/doshas-measured.md`, held by `check-doshas`.

  **Numbers:** none.

- The Neecha Bhanga family is sayable: `teistro_rules::shipped::computed_yogas`
  ships the eight the recording engine computes in its yoga service, each one
  condition over any debilitated graha, and all eight say present where that
  code did. The language gains `for-any`, which binds `SELF` for the condition
  inside it and takes every body that meets it as a participant;
  `{"exaltationOf": …}`, `{"debilitationOf": …}` and `{"exaltedIn": …}`; and
  `same-sign` and `same-body`. `SELF` outside a `for-any` is refused when a
  rule is read. Their citation is unsettled (crux C87).

  **Numbers:** none.

- `in-varga` reads a condition inside a divisional chart: every body in its
  sign there, its houses whole-sign from that division's lagna, its dignity
  from that sign. An evaluator is given the divisions it may step into, and a
  rule that reads a longitude inside one is refused when it is read.

  **Numbers:** none.

- A citation says how good its evidence is: `Source.rank`, 1 for a text, 2 for
  an implementation, 3 for a secondary source, 4 for nothing found. Every rule
  and table the SDK ships sets one, so a consumer can ask for only what a text
  supports. The survey behind it is in
  `01-research/feature-universe/04-yogas-doshas.md`: the "800 yogas" figure is
  a program's screen count, not a classical number; seven of the recording
  engine's doshas have no verse in seven full translations, and four more are
  widened from what their chapters say (cruxes C88 to C91).

  **Numbers:** none.

- A class of bodies can aspect and can be counted: the aspect predicates take
  `any-benefic` or `any-malefic` as well as a body reference, and
  `count-in-houses` asks for at least so many of a class in houses counted from
  any reference. Together with what the language already had, these say the
  arishta verses of BPHS ch. 9 and the papa and shubha kartari of Phaladeepika
  ch. 6 sl. 8.

  **Numbers:** none; every recorded yoga and dosha reproduces as before.

- A rule can name a point: `{"point": "GULIKA"}` resolves any point the
  catalogue names — the upagrahas, the special lagnas, the sphutas — from the
  set an evaluator is given (`with_points`), and resolves to nothing when the
  chart does not carry it.

  **Numbers:** none.

- A chart's panchanga can carry a limb's ghatikas — how far the birth stood
  into the tithi, the Moon's nakshatra and the rising sign, and how much was
  left — and `at-limb-edge` reads them. On that,
  `teistro_rules::shipped::gandantas` ships BPHS ch. 92's tithi, nakshatra and
  lagna gandantas and its Abhukta Moola, at rank 1 with chapter and verse: the
  SDK's first rules written from a text rather than mirrored from an
  implementation. They measure a different quantity from the engine's
  `planet-at-gandanta`, which reads degrees from a sign junction (crux C92).

  **Numbers:** none.

- `teistro_rules::shipped::arishtas` ships eleven of BPHS ch. 9's evils at
  birth and four of ch. 10's antidotes, each read from the verse at rank 1.
  `birth-by-day` reads whether the birth fell between sunrise and sunset, which
  ch. 10 v. 5 asks beside the paksha. The verses that turn on a graha being
  "strong" are deliberately not in the pack: the kernel has no strength
  measure, and a cancellation that fires too often is worse than one that is
  missing.

  **Numbers:** none; these are new rules, and no recorded answer moves.

- A rule can name another: `{"type": "rule", "key": …}` holds when that rule
  holds, read from the set an evaluator is given, and `check_references`
  refuses a set with a dangling key or a circle, naming it. With it, BPHS
  ch. 9's evils carry ch. 10's antidotes as their cancellations while the
  antidotes stay rules of their own, and the pack grew to the evils to the
  mother and to the father: 32 evils and 4 antidotes. `count-aspecting` counts
  how many bodies of a class aspect a reference, which ch. 9 v. 24's three
  malefics on the Moon needed.

  **Numbers:** none.

- Varahamihira's Balarishta: eleven rules from Brihat Jataka ch. 6, read in
  Chidambaram Iyer's 1885 translation, ship beside BPHS's. `planet-in-degrees`
  reads a body's place within its sign, which v. 8's last navamsa needs, and is
  refused inside an `in-varga` as every longitude is. The verses whose escape
  turns on a *powerful* benefic carry that in their notes rather than in a
  cancellation the kernel cannot measure.

  **Numbers:** none.

- A rule can say what happens when it holds: `Rule::outcome`, whose one kind is
  a `life-span` — a count and the unit its verse uses — which a `RuleResult`
  carries. It is what the texts actually grade an affliction by, and it
  replaces an invented score. Ten rules of Saravali ch. 10 ship with the spans
  Kalyana Varma gives them, from sixteen days to nine years.

  **Numbers:** none.

- Saravali chs. 11 and 12, the antidotes to those evils: eleven more rules,
  one of them graded at a hundred years. Each evil of ch. 10 names ch. 12's
  six, which counter the evils at birth generally, and those that name the
  Moon name ch. 11's five beside them, ch. 11 being her chapter. The arishta
  pack is now 53 evils and 15 antidotes over three texts.

  **Numbers:** none.

- The dwigraha generator: Brihat Jataka ch. 14's twenty-one pairs of grahas
  sharing a sign and Phaladeepika ch. 18's seventy-two readings of the Moon in
  each sign under each of six aspects, built from a table of what changes
  rather than written out. Neither text grades anything, so `Outcome` gained a
  second kind, `effect`, carrying what the verse says in words; `days()` now
  answers only where a text counts a span. `shipped::readings()` is the set.

  **Numbers:** none.

- Jataka Parijata's lists beside them, as the appendix to Iyer's 1885 Brihat
  Jataka prints them: every combination of the seven grahas sharing one sign,
  from two to six — 21, 35, 35, 21 and 7. They run in combinatorial order, so
  a rule's grahas are generated and only its reading is data. `readings()` is
  now 212 rules, and the SDK ships 309.

  **Numbers:** none.

- Saravali chs. 22 to 29 beside them: each of the seven grahas in each of the
  twelve signs, eighty-four readings, a chapter to a graha. `readings()` is now
  296 rules, and the SDK ships 393. The chapters' further readings of a graha
  under another's aspect, and ch. 23 v. 88's making the whole of them wait on
  strength, are not here.

  **Numbers:** none.

- Saravali ch. 30 after them: each of the seven grahas in each of the twelve
  bhavas, eighty-four more readings. `readings()` is now 380 rules, and the SDK
  ships 477. The chapter's closing verses — malefics harming the bhava they
  occupy except in the sixth, eighth and twelfth, and all of these readings
  varying with strength — are not here.

  **Numbers:** none.

- The thirty-two Nabhasa yogas of BPHS ch. 35, read from the text, each with
  the effect its verse gives: `shipped::nabhasas()`. The seven sankhya yogas
  name the other twenty-five as cancellations, as v. 17 requires. Twenty-seven
  of the thirty figures the recording engine also carries answer exactly as it
  recorded over the 93 charts; the three that differ are readings, recorded as
  crux C93 and pinned so that a divergence which moves — or disappears — fails
  the build. The SDK ships 509 rules.

  **Numbers:** 27 of 30 Nabhasa figures reproduce the engine's answers exactly;
  every chart answers one sankhya yoga and 46 of the 93 are cancelled.

- BPHS ch. 37's lunar yogas and ch. 38's solar ones beside them: the Moon
  counted from the Sun, Adhi yoga, the three grades of benefics in the
  upachayas from her, Sunapha, Anapha, Duradhara, Kemadruma, Vesi, Vosi and
  Ubhayachari — fourteen rules, each with its verse's effect. The SDK ships
  523 rules.

  **Numbers:** 34 of the 38 figures the engine also carries reproduce its
  answers exactly. Kemadruma is the fourth divergence (crux C94): read whole,
  the verse answers 1 of the 93 charts where the engine answers 57.

- **Breaking:** `Rule::outcome` and `RuleResult::outcome` become `outcomes`, a
  list, because a verse may say more than one thing — Saravali ch. 37's Pancha
  Mahapurusha verses describe the native *and* count his years. `life_span()`
  and `effect()` on a rule and on a result reach either kind without matching.

- The five Pancha Mahapurusha yogas as Saravali ch. 37 gives them, and the
  named yogas of BPHS ch. 36 the language can say: Shubha, Ashubha, Gaja
  Kesari, Amala, Parvata, Chamara, Srinatha, Matsya, Koorma, Khadga, Kalanidhi,
  Kalpadruma, Lagnadhi and the three Trimurthi yogas. The SDK ships 544 rules.

  **Numbers:** every Pancha Mahapurusha yoga reproduces the engine exactly, on
  49 answers over the 93 charts; 41 of the 49 figures both carry now agree
  exactly, and the four new divergences are crux C95.

- Two predicates for everything Jaimini reads by. `rashi-aspects` is BPHS
  ch. 26's aspect of the signs, held to the table the chapter prints over all
  144 pairs; `argala` and `vipareeta-argala` are ch. 31's intervention, with
  `ArgalaPlace` pairing each intervening house to the one that obstructs it so
  a rule cannot pair them wrongly, and counted backwards from a node as the
  verse directs. Seven rules use them: ch. 29 vv. 13 to 15's graded gains of
  the eleventh from the pada of the ascendant, and ch. 39's two associations.
  The SDK ships 551 rules.

  **Numbers:** the graded gains nest as the verse says — 18 charts of the 93
  answer the first grade, 14 the intervention, 12 a benefic's, 1 an exalted
  benefic's.

- Saravali chs. 49, 50 and 51: the part of a sign that rises — two halves,
  three thirds and nine ninths of each of the twelve, a hundred and sixty-eight
  readings. They need no division chart, and the number of parts is data, so
  the chapter that does cut a sign nine ways needed no code at all.
  `readings()` is now 548 rules and the SDK ships 719.

  **Numbers:** one hora, one decanate and one navamsa rise in every chart, so
  each division answers 93 times over the 93 — which says the degree bands tile
  a sign with no gap and no overlap.

- **Strength, which is what every omitted verse wanted.** A chart may carry
  `Strengths` — a measure, each body's number and what it must reach — and
  `planet-strong`, `planet-weak` and `planet-stronger-than` read them. The
  kernel compares; it does not compute, so the measure and the reading of the
  requirement (crux C71) stay with whoever builds the chart. Strong and weak
  are two questions, not one and its negation: a chart that says nothing
  answers false to both. `Rule::reads_strength` says which rules want a chart
  that can answer.

  BPHS ch. 31's intervention is now whole — v. 4 gives two tests and either
  serves — and seven more yogas of ch. 36 ship (Kahala, Sankha, Bheri,
  Mridanga, Lakshmi, Sarada, Kusuma), together with the three Brihat Jataka
  arishtas whose escape is a *powerful* benefic, now carried as cancellations.
  The SDK ships 726 rules.

  **Numbers:** the corpus records the engine's Shadbala for 71 of the 93
  charts. Completing the intervention moved two charts into the graded gains;
  the new cancellations fire on 12 chart-rules where nothing fired before;
  Mridanga answers none of the 93.

- A nakshatra for any body: `planet-in-nakshatra` divides a body's own sidereal
  longitude, and `same-nakshatra` asks whether two stand in one. Four more of
  Saravali ch. 10's evils ship with it — the birth star identical with Ketu's,
  the Sun in a tenth of Mars or Saturn under a strong malefic's aspect, Rahu in
  an angle aspected by malefics, and the three lords combust. The SDK ships
  730 rules.

  **Numbers:** the computed nakshatra and pada agree with the corpus's recorded
  ones over all 27 nakshatras and 4 padas on each of the 77 charts that record
  one, and exactly one pair holds on each.

- **What a house says when several grahas share it** (crux C96). Reading a
  placement one graha at a time cannot say what several grahas in one house say
  together. The texts answer in four shapes and refuse a fifth, and the SDK now
  ships all four: Saravali ch. 31's 84 readings of each pair of the seven in
  each of the four angles — the only family in the corpus keyed to a set in a
  *named* house — and ch. 34's counts, three grahas in the ascendant and the
  enemies answering the number in the sixth. A pair in a named house needed no
  new predicate: a conjunction says one sign and `planet-in-house` says which.
  `readings()` is now 632 rules and the SDK ships 817.

  **Fixed:** the dwigraha pack attributed the pairwise-composition rule to
  Brihat Jataka ch. 14 v. 5. That verse says only "in the case of other
  planetary yogas the effects described shall be determined and applied"; the
  pairwise split is N. Chidambaram Iyer's note (a) on it. Phaladeepika ch. 18
  v. 5 is the verse that carries the rule, and the pack and the design page now
  say so.

  **Numbers:** Saravali ch. 31 v. 87 declines to give the same readings for
  three, four, five or six grahas in an angle, so nothing of that shape ships.
  A pair conjunct in an angle is a pair conjunct, so the four angle readings of
  a pair can never answer more charts than Brihat Jataka's one reading of it —
  which the test holds for all 21.

- **A house read whole.** `Evaluator::house_reading` gathers everything bearing
  on a house — its sign, the grahas standing there, every rule that held whose
  participants stand there — and the `Composition`s the texts give for reading
  them together. A composition is not a reading: it says nothing of the native,
  only how what held is to be taken, and it carries its verse. Six ship, in
  four kinds: `Compose` (Phaladeepika ch. 18 v. 5), `Arbitrate` (ch. 22 v. 19,
  BPHS ch. 79 vv. 2 to 3, Saravali ch. 34 v. 66), `Modulate` (ch. 34 v. 65) and
  `Refuse` (Saravali ch. 31 v. 87). Saravali ch. 19 v. 8's five-or-six-together
  reading ships beside them. The SDK ships 818 rules.

  **Numbers:** 48 of the 1116 houses of the corpus hold three grahas or more —
  35 with three, 10 with four, 3 with five — which is exactly the case a
  reading of one graha at a time cannot answer and the case the texts decline
  to write. Over the 93 charts a consumer receives 4913 results gathered under
  a house and 455 statements of how to read them together.

- **The ascetic yogas of BPHS ch. 79**, the first rules strength unlocked and
  the sharpest measurement the corpus has given. The yoga forms when four or
  more grahas *possessed of strength* share a house, and the native takes the
  order of *the strongest of them alone*; vv. 6 to 8's three further figures
  ship beside them, and v. 4's cancellation with them. Saying "the strongest of
  these" needed no predicate: it is `not` of a `for-any` over `same-sign` and
  `planet-stronger-than`. The SDK ships 829 rules.

  **Numbers:** the recording engine writes the same seven as "this graha shares
  a sign with three others" and answers 34 chart-rules over the 93 where the
  verse answers 1 — and on the one chart that satisfies the verse it fires four
  at once, giving the native four holy orders where Parashara gives him one.
  Crux C97.

- `Evaluator::house_readings` evaluates each rule **once** and hands its result
  to every house its participants stand in, where it had evaluated every rule
  twelve times. A test holds the whole-chart path to the one-house path
  reading for reading.

- Saravali ch. 10 v. 14, the ascendant in a Nigala, Sarpa, Pakshi or Pasa
  decanate, which had shipped as a refusal for want of a catalogue.
  Phaladeepika ch. 3 vv. 13 and 14 give the catalogue in the verse, so the rule
  ships — and with it the finding that **no two texts agree on which decanates
  are the serpent's** (crux C98). It needed no divisional chart: a decanate is
  a sign and a third of it. The SDK ships 830 rules.

- **The rules kernel is reachable.** Until now the only crate that depended on
  `teistro-rules` was `xtask`: every test passed, every gate was green, and 830
  shipped rules could not be called by any consumer. The façade now depends on
  it, re-exports it as `teistro::rules`, and adds `teistro::rule_chart` — the
  join from a chart the SDK computed to the chart the rules read. It leaves
  empty what it cannot fill (the SDK computes no chara karakas) and refuses a
  graha the foundation or the states do not carry rather than defaulting it.

  **Numbers:** `crates/sdk/tests/rules.rs` founds the corpus's first chart with
  the built-in ephemeris, joins it and reads it; the navamsha the bridge
  computes is the D9 the conformance corpus recorded for that chart, body for
  body.

- Saravali ch. 34's readings of a **named set of grahas in a named house** —
  the rarest shape the texts carry and the one a reading of one graha at a time
  cannot say. Fourteen ship: the named triple in the second (Mars, Saturn and
  the Sun), the same under a weak Moon's aspect, named pairs in the second and
  the seventh, Saturn standing *alone* in the second, and Saravali's Lagnadhi
  from the sixth against Parashara's from the seventh. `HouseReading`, `Held`,
  `Composition` and `Kind` now serialise, as a rule result already did. The SDK
  ships 844 rules.

  **Numbers:** over the 93 charts the named triple in the second never happens
  and the named pair in the seventh happens seven times — thirteen answers
  between every named-set-in-a-named-house rule the texts carry, which is how
  rare the shape is.

- **`Readings::TEXTS` now says what the texts say.** It claimed to be the
  texts' reading wherever a text settles one, and was a single field. Two more
  are settled: the nodes' motion, which BPHS ch. 31 v. 6 states plainly and
  which the intervention's backward count already rests on, and which house a
  body is in, which the verses count as whole signs from the lagna where
  `Recorded` follows whatever the caller's chart carries.

  **Numbers:** neither moves an answer over the 93 charts — no rule the SDK
  writes asks whether a node is retrograde, and every house the corpus records
  is already whole-sign. `crates/rules/tests/readings.rs` measures both flips
  at zero and pins the fields, so a silent reversion fails rather than passing
  on a corpus that cannot see it.

- Four invariants over the whole shipped set, which no pack can check for
  itself: a key names one rule, every rule reads back as itself through the
  language, every rule is evaluable, and the categories are a closed vocabulary
  of twenty-seven. Key uniqueness is the load-bearing one — a cancellation
  names another rule by key.

- BPHS ch. 44 vv. 38 and 39, the fate of the corpse, read from the
  twenty-second decanate: a benefic's decanate burns the body, a malefic's
  throws it in water, a mixed planet's lets it dry, a serpent's gives it to the
  animals. It needed no new reference — twenty-one decanates are seven signs,
  so the twenty-second is the eighth house's sign at the lagna's own third. The
  SDK ships 848 rules.

  **Numbers:** exactly one of the three lord-kinds answers each of the 93
  charts, so the benefic, malefic and mixed split is exhaustive and disjoint;
  seven charts take the serpent reading beside it. The serpent list is BPHS's
  own (v. 40), not Phaladeepika's, so crux C98's anticipated table is *not*
  extracted: the two rules cite different texts, and a shared table would have
  coupled them into one answer the sources do not give.

- The composers' first step (`03-design/interpret-composers.md`, built):
  `crates/interpret` gives a **narrative plan** — `Plan`, an ordered list of
  `Item`s, each a message key and its slots — and `placements`, which says
  where each of the nine grahas stands and who shares a sign, from the rules
  kernel's own `RuleChart`. A plan holds no words, reads and writes as JSON,
  and is the same bytes for the same chart, so `sdk.intl` renders one plan in
  every locale. `teistro_intl::Value` reads and writes JSON, because a plan
  that cannot be written down cannot be stored, sent or held by a golden
  file — and it does so in the shape **the C boundary has always taken**:
  text, a number and a list as themselves, and everything else as one
  `$`-tagged object, `{"$entity": "graha.SUN"}` and the four beside it. One
  shape, written once in `teistro_intl::wire`, so a binding can hand a plan's
  slots straight back to `ts_intl_render` without converting them first; the
  boundary's own hand-written reader, ninety lines that knew the same five
  tags, is gone, and its test passes unchanged against the shared one.
  `cargo xtask interpret` measures it and `check-interpret` holds the page.

  A second composer, `readings`, says what each rule a chart held says: its
  verse's statement, who took part with a verb that agrees, whether a
  cancellation moved it and how grave it is. It brings `sdk.reading`, six
  messages in English and Nepali, built on the **typed** accessors the intl
  generator already writes — so a message that gains or loses a slot stops
  the composer compiling, and `KEYS` is the messages' own constants rather
  than a list of strings beside them. The verse's own statement is not
  translated: it crosses as a slot in the words the rule cites, because a
  machine translation of a cited text would be worse than the visible seam.
  `LifeClass::key` and `NetStatus::key` join the kernel, each held to what
  serde writes by a test, so a message selects on one list and not two.

  The façade carries it: `sdk.interpret().placements(&document)` is a ninth
  area — the first that is **Rust only**, since a plan has no crossing of its
  own yet — `teistro::{Plan, Item}` and `teistro::interpret` are re-exported,
  and a tenth worked example says one birth record in English and in Nepali,
  run by `check-rust` like the rest.

  **Numbers:** none move. The placements composer adds no message:
  `sdk.reason` already carried `grahaInRashi`, `grahaInBhava` and `occupants`
  in both strict locales. Over 93 recorded charts and five shipped rule packs
  the two composers write 8347 items, and all 16 694 renderings — every item
  in each strict locale — answer from that locale's own message with nothing
  to warn about. 2449 of those items carry the verse's own words untranslated,
  and the measured page says so rather than letting a reader mistake the seam
  for a defect. What the placements composer cannot say is counted: the lagna
  stands in every chart and in none of its items, one a chart, because those
  messages read a graha and the lagna is `point.LAGNA` — it does take part in
  a reading, where the message names no kind. One measured number moves
  the other way: `ts_intl_render` reaches two crates where it reached three,
  because reading a parameter no longer needs the calendar.

- **A library built at an ephemeris tier could not be asked for it.**
  `teistro-ffi`'s tier features forwarded to the façade without turning on
  the boundary's own `builtin-ephemeris` — `builtin-standard =
  ["teistro/builtin-standard"]` — and that feature is what its `#[cfg]`
  guards read. So `--features builtin-compact`, `builtin-standard` and
  `builtin-full` each linked a library with a built-in ephemeris underneath
  and refused `TsEphemeris::Builtin` as `UNSUPPORTED`: `ts_context_new`
  answered "this build of the library has no built-in ephemeris" to a build
  that had one. The façade's own four features were written the right way
  next door. Each tier now names `builtin-ephemeris` as the façade's does,
  and `check-lints`' `a-tier-turns-on-its-base` reads the sources and the
  manifest together, so only a crate that gates on the feature is held to
  it and one that starts gating cannot forward without it.

- **Eighteen of the twenty-five untouched members are untouched no
  longer**, and the tests that reached them are real coverage rather than
  a number being chased.

  Python had never read an **almanac day** beyond the limbs its example
  prints: its ayana, the direction not to travel in, the signs the
  luminaries stood in, Brahma muhurta, panchaka, the muhurta yogas, the
  window a day's spans are clipped to and the range a per-day list
  occupies were decoded by nothing. It had never read a **founded chart's
  own day** either — which arc of it the instant fell in, how far through,
  the lagna at the sunrise that opened it, the ayanamsha applied — nor the
  twelve bhavas of the **chalit** beside the ones under the placement
  system. Two tests now do, and Python drops from 22 untouched to 4.

  Node and Python's `FrameArea` had never been used: both tests
  round-tripped a frame through the free functions and never through the
  area a consumer with a context reaches for. Both assert it now, and that
  the two paths agree.

  Four more followed: Node's batch never answered a **registered dasha
  system's key by its id**, nor did an almanac ever say where one day's
  rows of a per-day list begin and end; Python had never read a **varga
  placement's** own question — whether the division left a body in the
  sign it was already in, which the D1 always does and a D9 rarely — nor
  the **last error** a refusal leaves on the context.

  **Three of 216 are left, and all three are the same thing**: `callJson`,
  `call_json` and `manifest_json`, the engine passthrough's JSON forms,
  which need a real engine the workspace does not carry. Node is now
  exercised member for member.

  The page reads a **fourth** surface: the Rust areas the bindings' layer
  classes mirror. All 45 of them were already exercised, which is the
  contrast worth having on the same page — the surface the SDK's own tests
  use was whole, and the three that mirror it were not, which is exactly
  why nothing noticed.

- **A page measuring what each binding's own surface has ever been used
  for.** `entry-point-is-reachable` holds that every boundary function is
  *placed* in a binding — exposed, declared, callable — and says nothing
  about whether anyone has called it. That difference cost a whole corpus,
  so `cargo xtask exercised` asks the other question of the layer no
  generator owns: of the members each hand-written binding declares, how
  many does anything in that binding's own tests or examples name?

  **25 of 171 are named by nothing**, and they are listed rather than
  counted: `dashaName` and `range` in Node, `callJson` in Dart, and
  twenty-two in Python. A member that stops being exercised changes the
  page, and one that starts changes it too — the Node frame area's `pack`
  and `unpack` came off the list in this change, because the test that
  round-trips a frame through the free functions now does it through the
  area a consumer with a context actually reaches for.

  A member counts as exercised when its name appears after a dot anywhere
  in the tests or examples, so a property read counts as much as a call
  and the count errs towards *exercised* — a member the page names is
  therefore one nothing touches.

  Getting the three extractors honest took five passes, and each correction
  is in the code: a Dart getter's body (`Foo get x => jsonDecode(y)`), a
  local helper inside a method, and a multi-line call all read as
  declarations until the rule required a **member's own indent** and a
  **return type**; a local function inside a *top-level* function needed
  the enclosing **class** tracked; and tracking it by column zero shut
  every class at its first blank line, which the page's own claim caught by
  reporting a layer with no members at all. A measurement with noise in it
  is worse than none, because the list is the thing a reader acts on.

- **Every binding can read a reading now. None could before.** No binding
  test or example had ever called `loadPack` — the function was generated
  into all three and executed from none — so nothing had shown that the
  path only half worked. The bytes crossed and the record landed, and then
  each binding's generated entity decoded the six forms `i18n/` declares
  and **dropped every other one**. A `phala`, a `timing`, a `namakarana`
  arrived at the boundary and was thrown away on the way out: the whole
  999-record corpus was unreachable from Node, Dart and Python.

  A record's forms are an **open** set, so each binding's entity now
  carries a `forms` map of every form beside its named fields:
  `entity.forms.phala`, `entity.forms['phala']`, `entity.forms["phala"]`.
  The named fields stay, because they are the ones `i18n/` guarantees and
  they type-check. `forms` is a **reserved form name** — a record with a
  form called that would shadow the map in three languages at once.

  Each binding's tests now load a fixture pack that lays one form over
  `graha.SUN` and read it back, asserting the counts the record returns
  (`entries`, `replaced`, `merged`) and that the shipped name survived
  beside the new form. The fixture is written by the same
  `blob_fixtures` example the three gates already run, so no gate gained
  plumbing.

- **A lint holds every composer to every binding.** A `PlanRequest`
  crosses the boundary as JSON rather than as a struct, so none of the
  three bindings generates its plan surface from the API description: each
  spells the record itself, which is three copies of one list that nothing
  held together. Two composers in a row had to be remembered into three
  files by hand.

  `composer-reaches-every-binding` reads `PlanRequest::MEMBERS` and
  requires each name in **both** declarations of each binding — the request
  a caller fills in and the record of plans it gets back — because one
  without the other is a composer that can be asked for and never read, or
  read and never asked for. It reads the declarations by their anchors
  rather than counting words, because `chalit` and `houses` are also names
  of chart sections and a substring count says nothing.

  No other gate sees this failure: `check-parity` compares the values a
  scenario answers, and a composer nobody can ask for answers nothing. The
  lint was proved red by removing one member from one binding before it was
  believed.

- **The reading corpora tell a consumer how to use them.** Two roots of
  649 and 350 records a locale sat in `packs/` with nothing but the design
  pages to explain them: no README, and the top-level one never mentioned
  them. `packs/README.md` says what they are, why they are not in `i18n/`,
  the one command that builds their packs, how each of the four bindings
  loads the bytes, and that a record **merges** rather than replaces.

  The documentation gate could not have seen such a file: `markdown_files`
  walked `docs`, `rfcs`, `.github`, `fixtures`, `spikes`, `adapters` and
  `crates`, and `packs` was not among them — so a README there would have
  rotted unread, which is the trap this project has hit before. The gate
  walks `packs` now, and 326 files pass instead of 325.

- **A fifth locale for both reading corpora, derived rather than
  translated — and the defect deriving it found.** `sa-Latn` is `sa-Deva`
  in Latin script, as it already is in `i18n/`. Transliterating a corpus of
  **prose** through the same path showed that it **title-cased every
  word**: right for a name (`Aśvinī Kumāra`, which is how the sources' own
  `iast` forms are written) and wrong for a passage, which came out
  `Gururlagne Rājayogakārakaḥ. Prajñāvān, Dhārmikaḥ` where the text says a
  sentence.

  Casing is now **declared by the caller** — `derive::Casing::{Names,
  Sentences}`, and `teistro-intl derive --prose` — because the source
  script carries no case and nothing in the text could tell a two-word
  passage from a two-word name. A rule that counted words would be the
  inference this project refuses elsewhere. Sentence casing capitalises
  the first letter of the text and of each sentence after it, treating the
  danda as the stop it is, and touches nothing else.

  Both corpora now carry `packs/readings/sa-Latn` and
  `packs/states/sa-Latn`, and `check-state-readings` re-derives each and
  holds the checked-in files against it, exactly as `check-intl` does for
  `i18n/sa-Latn`. Every claim on both measured pages holds with the fifth
  locale: 1 750 records, 4 850 forms answering from loaded packs, and the
  readings of a rule still standing under the state readings laid over
  them.

- **A tenth composer, `chalit`: where the two house readings disagree.** A
  chart places every graha twice — under the placement system, which is
  what most of the tradition means by "in the seventh", and under the
  chalit, which reads the cusps — and keeps both without recomputing
  either. They differ for **135 of 675** placings in the recorded corpus
  and nothing said so. `sdk.reason.chalitShift` says it: *Sun in house 2 by
  sign and house 1 by chalit*, *सूर्य राशिले २ भावमा, चलितले १ भावमा*.

  **It says only the grahas that differ.** Agreement is the ordinary case
  and an item a graha would bury the disagreement in eight repetitions of
  it; a chart whose readings agree composes to nothing here, which is the
  answer and not a failure. The measured page holds the composer's item
  count against the corpus's own `shifted` list in both directions, so a
  composer that said one shift too many or too few changes that page.

  It reads the chart's own grahas rather than the rules' chart, which
  carries one house a graha — the façade is held by a test to adapt and
  never re-derive — so it is a composer of its own, needs **no section**,
  and is a tenth member of `PlanRequest`, off by default and present in all
  three bindings.

- **The houses say the sign each bhava falls in, beside its lord.** A
  `Bhava` carries the sign its **middle** falls in and the graha that rules
  that sign; for three composers the plan said only the second.
  `sdk.reason.bhavaInRashi` says the first, so `houses` emits two items a
  bhava — *the 1st house in Pisces*, *पहिलो भाव मीनमा*.

  It cost **no new vocabulary**: a `Rashi` is catalogued and named in every
  locale, and the ordinal shape is the one `grahaInBhava` already had
  translated in both strict locales. The composer repeats the record rather
  than choosing between a house's middle and its cusp — they differ only
  under an unequal division, and every division the corpus records is
  whole-sign, so the page says it cannot tell them apart rather than
  claiming a branch it has not exercised.

  What a bhava knows besides — its quadrant, and whether it is a trine, a
  house of difficulty or one that grows better with time — stays unsaid for
  a reason now named rather than assumed: `Quadrant` is a Rust enum and the
  rest are predicates, so none is a catalogue member and a message would
  need words no locale here has been given.

- **The strengths say whether a graha is strong enough, and never say
  "strong".** The Shadbala carries two facts about each graha — the rupas
  it scores and the rupas its text requires, beside whether it reaches them
  — and for four composers the second crossed in the document and was
  absent from the plan, counted at 341 of 497 grahas.
  `sdk.reason.strength.meets` says it, so `strength` now emits two items a
  graha: *Mercury falls short of the 7.00 rupas its text requires*, *बुधले
  आवश्यक ७.०० रूपा पुग्दैन*.

  It names the **requirement** and not a verdict. "Strong" is a word the
  tradition spends carefully and no locale here has been given it; what the
  Shadbala computes is a number and a comparison, so that is what the
  message says. The two items sit together, score then sufficiency, so a
  consumer filtering to `score` still reads the ranking in the items'
  order.

- **The lagna is said.** It stands in every chart and was in none of the
  placement items, because `grahaInRashi`, `grahaInBhava` and `grahaAt`
  read a `Graha` and the lagna is `point.LAGNA` — 93 items a corpus, one a
  chart, unsaid for six composers and counted on the measured page the
  whole time. Two new messages read a **point** instead
  (`sdk.reason.pointInRashi`, `sdk.reason.pointAt`), and `placements` and
  `positions` say them.

  It cost two sentences a locale rather than a translation project,
  because both strict locales already name eight members of the `point`
  kind — the ascendant, the five upagrahas, Gulika and Mandi — so the
  vocabulary was bought before the frame was written. The messages are
  general, so a composer that one day places Gulika says it through the
  same key.

  The lagna is said **first**, because it is what the rest of a chart is
  read against, and by its **sign alone**: its bhava is the first by
  definition. A plan of placements is one item longer, and a plan of
  positions too; 93 charts now compose to 20 406 items and every one of
  the 40 812 renderings answers from its own locale with nothing to warn
  about.

- **The three inauspicious kaalas, and one alias table instead of two.**
  The previous entry's own prose said the `-kaal` families key onto
  subjects the SDK has not modelled. Checking rather than believing it
  found `kaala` is a catalogue kind whose three members —
  `RAHU_KAALA`, `GULIKA_KAALA`, `YAMAGHANDA` — the SDK computes on every
  day it builds an almanac for. The whole gap was spelling: no
  normalisation turns `yamaganda` into `YAMAGHANDA`. Three written aliases
  closed it, and `inauspicious-kaal` is mapped.

  Its other two keys are refused by name for reasons that had to be
  checked too: `dur-muhurta`, because the SDK divides the day into
  muhurtas without naming any of them, and `varjyam`, because it computes
  no such window. Its sibling `auspicious-kaal` stays unmapped for the
  corrected reason — Abhijit and Brahma muhurta **are** computed
  (`panchanga::Muhurtas`), but as fields rather than catalogue members, so
  there is no key space to migrate into and a kind naming them is a
  catalogue decision with its own sourcing.

  The twelve lagna signs and the three kaalas are now one
  `migrate::STATE_KEY_ALIASES` keyed by category, where there had been a
  table for the lagnas and a branch in the resolver for them. 26
  categories map, 425 readings into 350 records a locale, 12 left.

- **A state category may name more than one catalogue kind, and a key with
  no subject is refused by name.** `planet-condition` is the case that
  asked for it: a graha's condition is a `dignity` where the sign gives it
  (`EXALTED`, `OWN_SIGN`) and a `state` where the sky does (`COMBUST`,
  `RETROGRADE`), and the engine keeps both in one table. A category now
  carries the kinds its keys may name and a key must name **exactly one**
  member of them — none means the SDK has no subject for the reading, and
  two would be an ambiguity the table has to settle rather than something
  to pick between. `shadbala-strength` and `muhurta-factor` will want the
  same widening when something says them.

  Its eighth key, `COMBUST_CANCELLED`, names nothing: the SDK computes
  combustion but not its cancellation. `migrate::STATE_REFUSALS` carries it
  with that reason and the gate holds the list **both ways** — a key there
  that names a member now fails, and a key that names none and is not there
  fails. Without it a corpus with one known gap would fail its migration on
  every run, and the pressure would be to widen the check rather than to
  record the gap. 25 categories now map, 422 readings into 347 records a
  locale.

  **Two shortfalls this corpus creates rather than closes are now
  measured.** 26 readings land on a record the base locale does not
  **name** — five special lagnas, three `state` members and the 18 rules
  that had no reading — because `entity-names.md` §4 refuses a translated
  stub and those kinds have no vetted table: the reading answers and the
  subject's own name does not. And 211 of the 422 readings have no composer
  that says them. Both can only shrink, and both are counted on the page
  rather than described.

- **A ninth composer, `phala`: the first that says what a *corpus* carries
  rather than what the SDK computed.** It says the reading a loaded pack
  holds for this chart's subjects — a graha in a bhava, the lagna's sign,
  and each limb of the panchanga — through six messages under `sdk.phala`
  in English and Nepali. It is a member of `PlanRequest` off by default and
  is **silent until a pack of state readings is loaded**, so a chart
  composes to exactly the plan it did before until a consumer asks for the
  words. Over the recorded corpus it says 930 items no composer could say
  before.

  `Vocabulary`, the trait a composer asks before choosing what to say, was
  spelled for one subject (`has_reading(rule)`). A second subject made it
  the general question it always was: `has_form(key, form)` over a
  catalogue key, with `has_reading` the spelling that names a rule's
  record. Existing implementations keep working through the provided
  method.

  `examples/phala.rs` runs the whole path: load both corpora, compose, say
  it in English and Nepali, then read a record that two corpora describe —
  `nakshatra.ASHWINI` with its `name`, its `iast`, its `phala` and its
  `namakarana` together, which is the merge on load seen from the outside.

- **The state readings: a reading for what a chart *is*, not only for what
  it triggers.** The baseline engine exports one `STATE_INTERPRETATIONS` of
  38 categories and 554 records — a graha in a bhava, a nakshatra, a tithi,
  a lagna, a dosha's timing — in the same four languages and the same
  shape the rule readings use. **24 of the categories key onto subjects
  this SDK already has**, and 216 of their 217 keys are the catalogue's own
  spelling character for character; the one exception is an alias the
  catalogue already carried. They are migrated into `packs/states/`,
  **loaded rather than embedded** for the reason the rule readings are, and
  measured by `check-state-readings`. Only two mappings are written by
  hand, and each is a list rather than a rule: twelve lagna signs, because
  stripping a prefix would work for all twelve and mis-file the thirteenth
  silently, and nine graha abbreviations for the one composite key.

  Every shipped rule now carries text in all four languages: 639 a reading,
  and the 18 that had none a `timing`. The measured page prints those apart
  because a timing says when a dosha acts and not what it means.

  **`Intl::load_pack` lays an entity record over the one standing rather
  than replacing it.** This is a behaviour change and a fix: a pack
  carrying one form meant to add that form, and replacing left the record
  with that form **and nothing else** — no name, no transliteration, no
  glyph. A form the file carries is now the file's, a form only the
  standing record carries is kept, and the gender and glyph are the file's
  where it has one; a message entry still replaces, because a message is
  one string and has nothing to merge. It matters because a consumer's own
  pack is the supported way to change what the SDK says, and because two
  corpora now describe one subject: 48 of the states corpus's subjects are
  described by more than one category. `IntlLoaded` reports `merged` beside
  `replaced` in Rust and in all three bindings; the field took the
  `reserved` word `ts_intl_loaded` already held, so the ABI did not grow.

  An **overlay** root — one whose records add a form to records another
  root names — also relaxed three rules that had only ever seen complete
  roots: an entity is recognised by its shape (an object whose values are
  all text) rather than by carrying a `name`, a `name` is owed by a locale
  that declares `strict` completeness rather than by every record, and a
  form name is a `camelCase` word rather than lowercase letters, so
  `phalaProse` and `ishtaDevata` are names a form may have.

  The catalogue gained its **second open kind**, `graha_bhava`, for the one
  genuinely composite key, and open kinds moved from a `const` inside the
  generator into `catalogue/*.yaml` beside every closed kind's file. A kind
  costs the bindings nothing: only a closed kind's members become an enum
  at the boundary.

- **A seventh and an eighth composer, `conditions` and `karakas`: every
  fact a placement carries is now said.** A `Placement` is nine facts.
  `placements` said the sign and the house and `positions` the longitude;
  the other six had no message in any locale, and the measured page had
  counted them for three composers running. `conditions` says four — the
  dignity, the navamsha sign, the vargottama that sign may make, the
  retrogression and the combustion — and `karakas` the two chara karakas.
  Neither costs a request a section: both read the same graha states
  `placements` reads, so one knob now serves four composers.

  **The second time translation debt was spent, and it cost much less than
  the first**, for a reason worth stating as a rule: **where a composer says
  a catalogued value, the vocabulary is already bought.** Four of the seven
  new messages carry a dignity, a rashi or a chara karaka as an `:entity`
  slot, and `sdk.entity` names those in all five shipped locales — so only
  a condition with no value to name had to be written: वक्री, अस्तंगत and
  वर्गोत्तम, the tradition's own terms, flagged for the native review `ne`
  and `hi` already wait on.

  An entity slot is also the **typed** one, and that is not only ergonomics.
  The generated `sdk.condition.dignity` takes a `Dignity`, not a string, and
  renders each member's own word. Eleven `.match` arms with a catch-all —
  the shape `aspects` uses for `Strength`, which is not catalogued — would
  have read a twelfth member as the eleventh, and `Dignity` is
  `#[non_exhaustive]` with members appended by the catalogue. The message
  cannot go wrong by standing still.

  **A dignity is said of every graha, `NEUTRAL` included**, where `aspects`
  skips `Strength::None`. The two look alike and are not: no aspect is an
  absence, and its catch-all would have said "fully" of it, while *sama* is
  a dignity the texts name. The three conditions that **are** absences —
  not retrograde, not burnt, not vargottama — are said only where they hold.
  Vargottama stands beside the navamsha rather than instead of it, for the
  reason `mutual` stands beside its two casts: the fact is the sign, and the
  name is what the texts read.

  **`karakas` emits both schemes because they disagree.** Over the corpus's
  93 charts the seven-karaka scheme and the eight give a graha the same
  karaka 326 times and a different one 325, and the eight rank 93 grahas the
  seven do not rank at all. Emitting one would have chosen for the consumer
  in half of all cases, so both are emitted and **the key** says which —
  `ofSeven`, `ofEight` — so filtering by key gives one scheme whole rather
  than reading a slot to find out. Which order the eight are ranked in stays
  the chart's: `rule_chart` follows BPHS ch. 32 and
  `RuleChart::with_chara_karakas` switches to the recording engine's.

  **The corpus settled a question the design page would otherwise have
  guessed.** Three quarters of the retrogressions these charts hold are the
  nodes' — 182 of 243 — which looked like a tautology worth skipping. It is
  not: 2 of the 93 charts record Rahu and Ketu **direct**, and both are
  `--true-node` variants, of the 6 the corpus holds. The true node turns and
  the mean node does not, so the condition carries information and is said.

  `PlanRequest` gained the two members, and with them a `MEMBERS` list the
  refusal quotes and a test holds against the record's own serialisation,
  both ways. The hint it printed had gone stale one composer earlier — it
  still named five when there were six — which is exactly the shape a list
  written by hand beside a struct takes.

  **Numbers:** none move. 93 charts now compose to 19 061 items, up from
  15 577, and all 38 122 renderings still answer from the strict locale's
  own message with no fallback and nothing to warn about. 21 of 28 messages
  are emitted, across `sdk.aspect`, `sdk.condition`, `sdk.karaka`,
  `sdk.reading` and `sdk.reason`.

- **A sixth composer, `aspects`, and the first whose messages were written
  for it.** Which graha looks at which and how strongly: a `cast` item for
  every drishti a chart holds and a `mutual` item for every pair that looks
  back. Four composers shipped free because the packs held a message nobody
  read and the fifth exhausted them, so **this is where the translation debt
  is first spent** — `sdk.aspect` is a namespace of two messages added in
  English and in Nepali, because no locale carried a word for a drishti: not
  "looks at", not a grading, not the relation.

  **The terms are the tradition's, not a translator's guess.** पाद, अर्ध,
  त्रिपाद and पूर्ण दृष्टि for the quarters a drishti is counted in, and
  परस्पर दृष्टि for a mutual one, so the native review the roadmap already
  requires for `ne` and `hi` is checking grammar and register rather than
  vocabulary. The precedent is `sdk.reading`'s six, written the same way and
  flagged the same way.

  The grading belongs to the message: `Strength` is not a catalogued entity,
  so `cast` selects on the slot as `sdk.reading`'s `lifeClass` does, with
  arms spelled exactly as `Strength::key()` writes them. `Strength::None` is
  never stored by `Aspects` and the composer skips it anyway, because the
  catch-all arm is `FULL` and a catch-all that would say "fully" of no
  aspect at all is worth one branch to make unreachable.

  **`mutual` is a typed pair rather than a list.** It began as
  `{$grahas :list}`, which generated `Vec<crate::Value>` and rendered "Sun
  and Saturn" beside `cast`'s "the Sun casts…"; two entity slots generate
  two `Graha`s, read better and match `Mutual { first, second }` exactly.

  `Aspects::mutual()` took each pair once by the foundation's order. That
  rule is a property of a **set of relations** and not of the container, so
  it moved to `aspect::mutual_pairs` over a slice and `Aspects::mutual()`
  delegates to it — the composer, the façade and the measured pass now share
  one implementation instead of three.

  **Numbers:** none move. 93 charts now compose to 15 577 items, up from
  10 581, and all 31 154 renderings still answer from the strict locale's
  own message with no fallback and nothing to warn about.

  The coverage table that was supposed to catch a message left unread almost
  missed this one: it named `sdk.reason` and `sdk.reading` rather than
  deriving them, so a whole new namespace fell outside its own scope. It now
  takes the namespaces from `KEYS`, so a composer over a new one widens the
  check by itself — 14 of 21 messages emitted, across `sdk.aspect`,
  `sdk.reading` and `sdk.reason`.

- **A consumer's own composer is settled for v1.0 — by testing the promise
  rather than by building a registry.** `03-design/interpret-composers.md`
  §8 said a registry cost nothing to add once a second composer existed to
  prove the interface. Five now exist, and what they proved is that the
  registry was the wrong thing to reach for.

  The extensibility table's promise is *a narrative plan function (Rust)*,
  and it was **already kept**: `Plan`, `Item`, `TypedMessage`, `params` and
  the generated `messages` tree are published through `teistro`, and a
  composer is a function returning a `Plan`. `crates/sdk/tests/plans.rs`
  now writes one with the published surface alone — the grahas sharing the
  Moon's sign, which no shipped composer says — concatenates it with the
  SDK's own and renders the whole plan, so the table's row is a gated fact
  rather than a claim.

  A registry would buy one thing only: naming a consumer's composer **at the
  boundary**, which is the v1.x declarative plan. That trades against
  `interpret_json` refusing an unknown member by design, so it is a decision
  and not a detail. And if it is built, **the subject is not `&Document`**:
  four of the five take a document at the façade, and `readings` takes a
  `RulesReading` because a rule's answers are not in the document and never
  will be. A registry keyed on the document would exclude exactly the
  composer a consumer most wants to extend — the one that says what its own
  rule pack found.

  **The test falsified the documentation on its first run.** Two pages said
  an unknown key surfaces as `Rendered::is_fallback`. It does not:
  `is_fallback` means a *fallback locale* answered, while a key no locale
  carries at all is not a fallback because nothing fell back — it leaves
  `resolved_from` empty, and `Intl::has` answers before rendering. The
  measurement pass had always checked both; only the prose conflated them.
  Both pages now name the right field, and the test names both failures.

- **A fifth composer, `positions`**, and the last the shipped packs can
  carry for free: where each graha stands **to the degree**
  (`sdk.reason.grahaAt`, "Mars at 12°35′ Scorpio" and
  "मंगल १२°३५′ वृश्चिकमा"). It reads what `placements` reads, so it costs a
  request no section of its own, and `grahaAt` was already carried by both
  strict locales, hand-translated and tested, and read by nothing.

  **It is a composer rather than a line inside `placements`** because
  `grahaInRashi` says the sign and `grahaAt` says the sign *and* the degree:
  one subsumes the other, and a composer emitting both would repeat itself
  once a graha. Apart, the precision is a knob — a narrative report asks for
  `placements`, a position table asks for `positions`, and a consumer asking
  for both can see that it pays for the sign twice. It does not emit
  `sdk.reason.exactLongitude`, which renders a longitude alone
  ("222°34′35″"): that is a fragment a consumer formats with and not a
  sentence a plan says, the line `strength` drew at `strength.rank`. The
  rule has now held twice and is stated as one.

  **The measured page now says where the composers stop.** A placement is
  nine facts; three are said, and the six that remain have no message in any
  locale. Over the 837 grahas the corpus places: 243 stand retrograde, 66
  are burnt by the Sun, 534 hold a dignity that is not neutral, 106 are
  vargottama, 651 carry a chara karaka — and the plan says none of it. That
  is not a gap in what the SDK computes but in what a locale can say.

  **Numbers:** none move. 93 charts now compose to 10 581 items, up from
  9744, and all 21 162 renderings still answer from the strict locale's own
  message with no fallback and nothing to warn about.

  With it the free pool is empty, and the page **decides** that rather than
  the prose claiming it: it now lists every message the base locale carries
  under `sdk.reason` and `sdk.reading`, says which composer emits it, and
  prints any that is neither emitted nor given a reason as unaccounted.
  Twelve of nineteen are emitted; the seven left are two fragments
  (`exactLongitude`, `strength.rank`), a fact about the zodiac rather than
  a chart (`rashiNature`), a count of what `occupants` already names
  (`conjunction`), and the packs' three example messages. **The next
  composer must therefore be given a key that does not exist yet**, which
  is a translator's decision and the maintainer's call — the largest gap
  being the drishti, a whole computed section (`Document.aspects`) that no
  locale has a word for.

- **A fourth composer, `houses`**, and the first to say what a graha
  *rules* rather than where it stands: the lord of each of the twelve
  bhavas, the first house first (`03-design/interpret-composers.md` §4).
  Lordship is the relation the rest of the tradition is read through, and no
  other composer said it — `placements` says where a graha stands, and the
  two are different facts about the same graha. Like `placements` and
  `strength` it adds **no message**: `sdk.reason.lordship` was already
  carried by both strict locales, translated by hand, and read by nothing.
  It reads `Document.houses`, which the boundary computes for it, so
  `interpret_json` gains `"houses": true` and `sdk.interpret().houses(…)`
  answers in Rust.

  **It carries the largest silence of the four, and the page counts it.** A
  bhava also knows the sign it falls in, which third of the wheel it stands
  in and whether it is a trine, a house of difficulty or one that grows
  better with time; a chart knows which bodies fall in a different house
  under the chalit, and whether the division came back degenerate. No locale
  carries a message for any of it, so the plan claims none of it: the corpus
  records a division for 75 of the 93 charts and, for those, 135 of 675
  placings shift under the chalit — and the plan says not one of them.

  It also corrected this design's own page. The page claimed the corpus had
  no unequal division to try the "the bhava's sign is its *middle*'s sign"
  branch on; the conformance repository records twenty systems' cusps per
  chart and eight charts selected under Placidus, two of them degenerate.
  What is true is narrower: none of those eight is in the yogas corpus the
  composer is measured over, so all 75 charts it reaches are whole-sign,
  where cusp and middle coincide. The page now says that instead.

  **Numbers:** none move. 93 charts now compose to 9744 items, up from 8844,
  and all 19 488 renderings still answer from the strict locale's own
  message with no fallback and nothing to warn about.

- **A third composer, `strength`**, and the first over a *section*: each
  graha's Shadbala in rupas, the strongest first
  (`03-design/interpret-composers.md` §4). Like `placements` it adds no
  message — `sdk.reason.strength.score` was already carried by both strict
  locales and used by nothing — so it ships with no translation debt, which
  is why it is the composer that follows the crossing. It reads
  `Document.shadbala`, and the boundary computes that section for it exactly
  as it computes what a rule set reads, so `interpret_json` gains
  `"strength": true` and asking for a plan never means also knowing which
  knob it needs.

  **What it does not say is the point.** The Shadbala carries
  `required_rupas` and `strong`, and no locale carries a message for either,
  so the plan claims neither and the measured page counts the silence: the
  corpus records a Shadbala for 71 of the 93 charts, 497 grahas, of which
  341 reach the rupas their text requires. What the plan says instead is the
  **ordering**, which needs no word at all. It also does not emit
  `sdk.reason.strength.rank`, which renders an ordinal alone — `1st`,
  `१लो` — because that is a fragment a consumer formats with and not a
  sentence a plan says; a message in the pack is not automatically a plan
  item.

  **Numbers:** none move. 93 charts now compose to 8844 items, up from 8347,
  and all 17 688 renderings still answer from the strict locale's own
  message with no fallback and nothing to warn about.

- A narrative plan crosses the C boundary
  (`03-design/plans-at-the-boundary.md`, building), so what a chart has to
  say is no longer Rust's alone. It rides on `ts_chart_found` as the rules
  and the SVGs do: a nullable `interpret_json` naming the composers to run,
  `{"placements": true, "readings": true}`, and section 34 of the charts
  blob carrying one entry a chart. The composers run over the charts that
  crossing founded and the rules it just answered, so a plan is a rename of
  work already done and never an evaluation repeated.

  **An item's slots are the renderer's own**, which is the whole point and
  was not true until the params shape was unified: a binding says an item by
  handing `item.params` straight to `ts_intl_render`, with nothing in
  between, in any locale and in as many as it likes — a plan carries no
  locale at all, so two readers of one blob can read it in two languages.
  The ABI test holds exactly that, saying every item of both plans of two
  charts through the renderer rather than merely parsing the section.

  `sdk.interpret().readings(&RulesReading)` joins the façade beside
  `placements`, and `PlanRequest` is the record both the boundary and Rust
  read. A reading asked for without rules is refused as
  `interpret_json.readings` rather than answered with an empty plan, because
  an empty plan and an unasked question look alike and only one of them is
  the consumer's mistake; a composer that is not one is refused beside the
  composers there are.

  Each binding gains a worked example beside its nine, the same record said
  in English and in Nepali, and each is run by that binding's gate.

  **Fixed on the way**: the Dart message accessors did not parse.
  `sdk.reading.lifeClass` selects on a slot called `class`, and the emitter
  wrote `required String class`; Python renamed it to `class_` and
  JavaScript needed no rename, so one generator was right in two targets and
  wrong in the third. The Dart parameter is `class$` now, the slot keeps its
  own name, and `check-intl` reads the identifiers back out of the Dart it
  generates and fails on a keyword among them — because a generated file
  being **up to date** says nothing about its compiling, and the gate that
  compiles Dart runs in the verify matrix rather than in fast-check.

  **Numbers:** none move, and one is new. A plan written down costs 12 573
  bytes a chart over the corpus's 93, 17 425 for the widest and 140 an item
  — and the verses' own cited words, which were expected to be the weight,
  are 9% of it. What a plan costs is its items, each naming its message and
  its rule again. Small enough to cross whole, so nothing is packed or
  paged.

- Every rule renders to prose (`03-design/rule-doc.md`, built). A condition is
  one sentence and a rule a short passage carrying what it holds beside its
  conditions — its groups and their labels, its cancellations and their
  threshold, its severity, its outcomes, its timing, its remedies, its
  citation — from one vocabulary in `teistro_rules::prose`, which the trace
  reads too: a step prints "holds: the lord of house 10 stands in a kendra"
  where it printed the schema's own `planet-in-kendra`. `cargo xtask rule-doc
  <pack|category|key>` prints the passages; `check-rule-doc` holds the
  measurement. `language::KINDS` lists the 64 predicates for whoever walks the
  language, held against `Condition::kind` both ways by `check-lints`'
  `every-predicate-is-listed`.

  **Numbers:** none move. 1654 rules — the 997 shipped and the corpus's own
  beside them — hold 5415 conditions written 2600 ways, which say 2596 things
  and read as 2596 sentences: no two meanings share a sentence. Four
  renderings are shared by two spellings of one meaning, and three of those
  were single-armed combinators in `classical-marana.json`, simplified when
  the pass first found them; `Or` of one condition is that condition, so no
  answer moves. Two of the 64 kinds occur in no pack at all and are held by
  the golden test alone.

- Rules in every binding: Node's `rules` option and `chart.rules`, Python's
  `rules=` and `chart.rules`, Dart's `RuleRequest` and `chart.rules` — shipped
  sets and a consumer's own rules, answered in the chart's own crossing
  (`03-design/rules-at-the-boundary.md`, built).

  **Numbers:** the four parity runners ask for the Nabhasa set with longevity
  and agree on all 6 758 values, each chart's present rules and Pindayu among
  them.

- Rules at the C boundary: `TsChartRequest` gains a nullable `rules_json`, and
  section 33 `rules` of the charts blob carries each chart's answers as
  canonical JSON, rules by key. A held rule in a house reading is written by
  key too (`teistro_rules::key_of`). Refusals are named from the request's
  root, `rules_json.rules[0]` (`03-design/rules-at-the-boundary.md`, step 2 of
  4). No existing section moves.

- A chart reading that answers rules: `RuleRequest` names shipped sets and a
  consumer's own rules, validates into a `RuleSet` (each key once, every
  reference resolved), and `sdk.chart().readings_with_rules` returns each
  document with a `RulesReading` — the present results, and the house and
  longevity readings when asked. A birth with no sunrise is read without the
  points a rule named and says so in `unreadable`
  (`03-design/rules-at-the-boundary.md`, step 1 of 4).

  **Numbers:** over the corpus's 53 readable charts, the text-written and
  generated sets (895 rules) answer 3 145 times, about 59 a chart, each chart
  exactly as the same set evaluated through `RuleInputs`.

- `natural-relation`: how one graha regards another by natural relationship,
  and `Relation::between` for code; BPHS ch. 43 v. 67 and vv. 71 to 73's
  second figure ship with it. The SDK ships 997 rules.

  **Numbers:** the lagna lord's regard for the Sun partitions every chart but
  the seven with Leo rising, 60 friend and 26 enemy; no lagna lord regards the
  Sun as neutral.

- BPHS ch. 44's manner, place and awareness of death (vv. 25 to 37) and the
  worlds before birth and after death (vv. 41 to 45): 34 rules in the new
  `marana` and `loka` categories. The SDK ships 993 rules.

  **Numbers:** over the 93 charts the verses' partitions hold on every chart —
  one place of death on each of the 59 with the third occupied, one by the
  third's modality on all 93, one prenatal world on each of the 71 that compare
  the luminaries' strength.

- The marakas: `Evaluator::marakas` gives every graha's reasons from BPHS ch. 44
  vv. 2 to 24, graded death or difficulty; `Evaluator::vulnerability` reads a
  running chain with v. 8's malefic major period in a malefic sub-period; and
  `teistro::maraka_windows` gives a dasha's windows over the ages a class of
  life runs to. Every result carries `Presentation::Vulnerability` (crux C105).

  **Numbers:** the prime reasons are the kernel's maraka class on all 93
  charts; seven grahas of nine carry a death-bringing reason. The corpus's
  first chart, medium life by its three pairs, has 18 windows from 32 to 64
  years, 7 of them v. 8's fatal kind.

- Pindayu, Nisargayu and Amsayu: `Evaluator::ayurdaya` sums BPHS ch. 43 vv. 4
  to 32's three spans with each giver's basic years, reductions and net, and
  `Ayurdaya::chosen` picks by strength with ties averaged (crux C104).

  **Numbers:** the translator's seven basic years reproduce from his
  longitudes and, to a hundredth, from the SDK's own cast of his chart, with
  his Sun's and Moon's reductions. Over the 93 charts Pindayu averages 85
  years, Nisargayu 79 and Amsayu 39.

- The three pairs: `Evaluator::three_pairs` reads BPHS ch. 43 vv. 33 to 50 —
  each pair's class, how the chart's was decided, Saturn's and Jupiter's
  shifts, the years and their rectification under `ThreePairsRules` (crux
  C103). `PointAt` carries a point's longitude.

  **Numbers:** the translator's worked chart, cast by the SDK, matches his
  longitudes to 0.1° and his three pair classes; the verses then lower his
  short life to Yogarishta, which he does not apply. Over the corpus's 51
  charts with a hora lagna, the pairs agree all three on 9 and differ on 14.

- A class of life is an outcome: `Outcome::LifeClass` with `LifeClass`'s
  seven classes and their spans (BPHS ch. 43 vv. 52 to 54), raised and lowered
  a step as Jupiter and Saturn do (vv. 47 to 50). BPHS ch. 43's 21
  combinations for the class of life ship in `ayur`. The SDK ships 959 rules.

  **Numbers:** over the 93 charts, 97 long, 18 medium and 101 short; vv. 71 to
  73 place the stronger lord in exactly one of the three house groups on every
  chart that compares strength (crux C102).

- BPHS ch. 39 v. 15's and v. 24's figures on the special lagnas, and the
  bhava lagna they name (BPHS ch. 4 vv. 2 to 3, one sign in five ghatis),
  now computed beside the hora and ghatika lagnas. The SDK ships 938 rules.

  **Numbers:** a chart's points gain the bhava lagna; nothing else in the
  points moves. Neither figure answers any of the 51 corpus charts the SDK
  reads with points. V. 12 is not shipped: its two readings answer 6 charts
  and 52 (crux C100).

- **Fixed: a birth or a day near the date line.** Where a civil clock keeps
  more than half a day from the place's mean time — Samoa, Tonga, Tokelau,
  Kiribati's Line Islands — the local day's sunrise was taken from the wrong
  date, and a birth before dawn was refused outright ("not in the local day").
  A civil date now reaches a solar model as the mean-time date at its own noon
  (`calendar::solar::civil_day_light`), in the chart's day, the daily
  panchanga and the solar month-start rules.

  **Numbers:** sunrise, sunset and everything counted from them move by a day
  at those places, and only there; no chart elsewhere moves.

- From a chart reading to rules in one step: `ChartRequest::with_rule_inputs`
  asks for exactly what a rule set reads (derived by `Rule::vargas`,
  `Rule::reads_strength` and `Rule::reads_points`), and
  `teistro::RuleInputs::of(&document).evaluator(readings)` gives an evaluator
  with the chart, its points and its divisions.

  **Numbers:** the SDK's own reading of each of the corpus's charts matches the
  corpus body by body except at one chart cast on a sign boundary, where the
  Sun stands 0.2″ apart.

- A rule result says which dasha periods deliver it. `Rule::timing` is
  `concerned` (BPHS ch. 31: "the Rashi or planet concerned") or `throughout`
  (ch. 35 v. 50, now on all 32 Nabhasa yogas); `Evaluator::delivery` answers
  for a running chain given as `Running` lords and signs, and
  `teistro::rule_periods` gives it the SDK's own. Two timed verses ship: BPHS
  ch. 41 v. 16's wealth-givers and ch. 42 v. 13's harm to finances. The SDK
  ships 936 rules.

  **Numbers:** delivery is held against the verse read directly off each of
  the 93 charts, 327 wealth-giving periods, and every Vimshottari mahadasha of
  a chart the SDK founded agrees.

- Chara karakas and divisions reach a consumer. `RuleChart::with_chara_karakas`
  computes both karaka schemes from the longitudes (BPHS ch. 32, to the
  arc-second), `RuleChart::varga_signs` places a chart in a division, and
  `teistro::rule_vargas` does so under the SDK's classical schemes;
  `teistro::rule_chart` now fills the karakas, so the twenty raja and Jaimini
  rules that read one or a division answer for a chart the SDK computed.

  **Numbers:** the seven-karaka scheme reproduces the corpus on all 93 charts.
  The eight-karaka scheme reproduces on all 93 only in the recording engine's
  order, which puts the Pitrikaraka last where BPHS puts the Darakaraka last;
  `EightKarakas` chooses, defaulting to the verse (crux C101).

- BPHS ch. 39's raja yogas and ch. 40's yogas for royal association: 56
  rules in a new `raja` category, and `count-of`, a counting quantifier that
  shares `for-any`'s evaluation. The SDK ships 934 rules.

  **Numbers:** v. 37's angular lord joining a trinal lord agrees with the
  recording engine on all 42 answers over the 93 charts; the fifth and ninth
  lords answer five wider, the verse adding mutual aspect. "Benefics in
  angles" is not shipped: some benefic answers 67 charts and every benefic 5
  (crux C100).

- BPHS ch. 42's combinations for penury: fifteen rules in a new `daridra`
  category, with v. 17's other half (the Sun in the second unaspected by
  Saturn) as a `dhana` rule. The marakas of BPHS ch. 44 vv. 3 to 5 are a
  class: `any-maraka` beside `any-benefic` and `any-malefic`, a public
  `Class`, a `planet-is` condition, and `except` on `count-in-houses` so a
  body is not counted as joining itself. The SDK ships 878 rules.

  **Numbers:** the sixteen answer between 0 and 40 of the 93 charts; the two
  frequent ones were traced case by case and are the verse's arithmetic. The
  Sun-in-the-second pair partitions the twelve charts with the Sun there.
  Verse 12 is not shipped: its readings answer 57 charts or none (crux C99).

- BPHS ch. 41's combinations for wealth: fourteen rules, the most particular
  figures the SDK ships. Verses 2 to 8 each want the lord of the fifth in the
  fifth and the lord of the eleventh in the eleventh, which holds for one or
  two ascendants apiece, so a chart can answer at most one of the seven — a
  test holds that. The SDK ships 862 rules.

  **Numbers:** one of the fourteen answers once over the 93 charts. A figure
  that pins a graha to a sign, a house and two companions is rare by
  construction, and the SDK reports what the verse says rather than loosening
  it until it fires.

  **Also measured:** 823 rules over the 93 charts answer in 15 ms in release —
  0.16 ms a chart, 0.2 µs a rule. The evaluator resolves a rule reference by
  walking the set, which looked like a scaling defect; at that cost it is not
  worth an index, and an index would have cost the evaluator its `Copy`. The
  measurement stopped the work rather than starting it.

- Rashi bala from BPHS ch. 46: `teistro_dasha::rashi::stronger_sign`
  compares two signs as vv. 158 to 166 do, and two knobs read it.
  `dasha.dual_lord` (`BPHS` by default, `KENDRA` the corpus engine's) finds
  the stronger lord of Scorpio and Aquarius (crux C51). `dasha.rashi_start`
  (`STRONGER` by default, `LAGNA` the engine's) starts Mandooka, Shoola and
  Trikona from the stronger of their signs (crux C53). A rashi reading
  records both in `DashaReading.rashi`. `conformance-baseline` keeps the
  engine's readings as version 10.

  **Numbers:** under the default settings, the Chara family's lords and
  years move wherever the two dual lords' rules differ, 360 of the corpus's
  616 answers, and the three systems' starts 98 of 231. Every settings hash
  moved. The corpus's own readings are unchanged under
  `conformance-baseline`.

- PyJHora cross-checks: the corpus moves to 0.9.0, which records PyJHora
  4.8.7's Vimshottari for 53 charts at evidence rank 3, and
  `crates/dasha/tests/pyjhora.rs` gives the kernel the tool's Moon and each
  of its years and holds every antardasha start to the year constant's
  difference: a tenth of a millisecond where the constants agree. The written
  balance differs as a convention (75 of 212 agree), counted.

- The dasha cursor's budgets are measured: `crates/dasha/benches/dasha.rs`
  times `at(t, 5)` for every kernel and a registered row (119 to 354 ns
  against 20 µs) and a materialised depth-3 tree (15.8 µs against 500 µs).
  `teistro-scenario` gains a `dashas` section, so the instruction-count gate
  and the cross-architecture hash matrix now watch the cursor too.

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
