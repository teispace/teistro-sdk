# The panchanga day

Status: `built`, written 2026-09-07 from the falsification pass in
[`panchanga-day-conventions.md`](panchanga-day-conventions.md), which
measured every convention this page relies on against the 55 recorded
days of the conformance corpus. Derives from
[`chart-foundation.md`](chart-foundation.md) (the day an instant belongs
to), [`time-and-timezone.md`](time-and-timezone.md) (the local day, the
horas, the ghati), [`astro-events-and-crossings.md`](astro-events-and-crossings.md)
(the boundary solver and the rise and set solver),
[`core-types-and-catalogue.md`](core-types-and-catalogue.md) (the kinds
and their attributes) and
[`settings-and-profiles.md`](settings-and-profiles.md) (the knobs).
`02-architecture/01-module-catalog.md` gives the module its row. Built as
`crates/panchanga`; what §14 describes is what it exposes.

## 1. Purpose and scope

A panchanga is an almanac of one day at one place: which tithi, vara,
nakshatra, yoga and karana run and when each gives way to the next, which
parts of the day are auspicious and which are not, when the Moon rises
and sets, which lunar month it is. It is the single most-read output of a
Jyotisha library — a chart is consulted once, a panchanga every morning —
and it is almost entirely convention, which is why it was measured before
it was designed.

It settles: what a panchanga day *is*, and that it is the window
`crates/time` already computes; how the four moving limbs are found and
what a span carries; that every period in the day is one arithmetic
operation on one of two arcs; which of those periods the SDK reports as
absent rather than empty; where the daily reckoning may differ from the
chart's and how the difference is made visible; and what the corpus can
and cannot hold the module to.

It is not: the natal panchanga, which is the same five limbs read at a
chart's instant and belongs to `chart` (§10); the muhurta service, which
scores and searches windows and takes this module's output as an input
(`muhurta`, Phase 5); the Indian lunisolar calendar, which decides the
intercalary month (`calendar-indian-lunisolar.md`, Phase 2); or the
festival rules, which are a rule pack over this.

## 2. Inputs, settings and ports

| input | from |
|---|---|
| the date | a `CalendarDate` in `calendars.civil_calendar`, or a `FixedDay` |
| the place | latitude, longitude, altitude (`core::quantity::Place`) |
| the settings | a resolved `Settings` with its hash |
| the ephemeris | the port, through the profile's override policy |
| the zone | the timezone port, to turn a civil date into an instant |

An instant may be given instead of a date, and then the day is the one
that holds it — the inversion `crates/chart`'s `day` module already
performs. A panchanga *of a date* and a panchanga *of an instant* are the
same value reached two ways, and the API offers both (§14).

The knobs it reads, all but three of which exist:

| knob | what it decides |
|---|---|
| `day.sunrise` | the horizon convention the arc is found under |
| `day.day_boundary` | where the window opens: sunrise, midnight, sunset or noon |
| `day.polar_day_policy` | what stands in for an arc the sky did not give |
| `day.hora_reckoning` | proportional or equal horas |
| `calendars.lunar_month` | amanta or purnimanta |
| `calendars.civil_calendar` | which calendar the date is in |
| `frame.*` | the ayanamsha, the node, the corrections |
| `provider.*` | which ephemeris answers |
| **`panchanga.centre`** | which centre the day's limbs are computed from (§10) |
| **`panchanga.moon_events`** | which window the Moon's rise and set are found in (§11) |
| **`panchanga.muhurta_tables`** | which tradition's muhurta yoga tables (§12) |

`day.day_boundary` has existed since Phase 1 and nothing has read it.
This module is its first reader, and reading it draws a distinction the
page returns to in §5: **the window bounds the limb spans; the arc bounds
the periods.** Under `MIDNIGHT` the spans run midnight to midnight and
the choghadiya still divide the daylight and the night, because a
choghadiya is defined on the arc and not on the window.

## 3. The day is the window, and the window is a `LocalDay`

Measured first, because everything else is bounded by it: over all 55
recorded days the four limb lists begin at one instant and end at one
instant, exactly, and that first instant is the day's sunrise
([conventions §1](panchanga-day-conventions.md)). The window is 23.97 to
24.02 hours long — a day plus or minus the change in the length of the
daylight.

`time::local_day::local_day` already returns that window and everything
about it:

```rust
pub struct LocalDay {
    pub place: Place,
    pub date: CalendarDate,
    pub vara: Vara,
    pub sunrise: JulianDay<Utc>,
    pub sunset: JulianDay<Utc>,
    pub next_sunrise: JulianDay<Utc>,
    pub state: DayState,          // normal, or polar with the policy applied
    pub convention: SunriseConvention,
    pub model: String,            // which solar model, for the stamp
}
```

So the panchanga day introduces **no day model of its own**. It takes a
`LocalDay`, and the vara it reports is that day's vara — which is the
vara of the civil date the arc opens on, and which does not change at
midnight. An application asking for "today's panchanga" at 02:00 gets
yesterday's date's day, and the page's §14 API makes that impossible to
get wrong by accident.

## 4. A limb is a span, and a span carries both pairs of bounds

The corpus records each limb as the members that touch the window, in
order, each clipped to the window: a tithi that began at 21:05 yesterday
is written as beginning at sunrise, and **its real start is not
recoverable from the fixture** ([conventions §2](panchanga-day-conventions.md)).
That is a loss, and an almanac prints both facts — "Krishna Chaturdashi
until 14:32" and "Krishna Chaturdashi, from 21:05 yesterday" — so the SDK
carries both:

```rust
pub struct Span<T> {
    /// Which member.
    pub member: T,
    /// The member's own bounds, whether or not they lie in the window.
    pub whole: Interval,
    /// `whole` clipped to the window: what an almanac row prints.
    pub inside: Interval,
}
```

with `began_before()`, `ends_after()` and `fraction_at(instant)` over it.
`inside` is never empty — a span that does not touch the window is not in
the list.

### The kernel

`astro::events` already has the lattices (`Lattice::TITHIS` at 12°,
`KARANAS` at 6°, `NAKSHATRAS` and `YOGAS` at 360/27) and the quantities
(`Quantity::ELONGATION`, `MOON_PLUS_SUN`, `Longitude(Moon)`), and its
`Search::between` returns every crossing in a window. So a limb list is:
search the widened window, take the crossings as boundaries, and classify
the interval between two boundaries by the integer path (ADR-0016).

Three details make it one pass rather than four:

1. **The window is widened by the longest span**, 1.5 days each side. The
   longest tithi is about 26 hours and the longest nakshatra transit about
   28, so a widened search finds the true `whole` bounds of the first and
   the last member without a second search backwards.
2. **The karana search gives the tithi for free.** A karana is half a
   tithi, so the 6° lattice's crossings are a superset of the 12°
   lattice's — every second one is a tithi boundary. One search, two
   limbs.
3. **The four limbs need three searches**, not four: elongation at 6°,
   Moon plus Sun at 360/27, and the Moon at 360/27.

The classification is the SDK's integer path over the same longitudes, not
a second reading of the crossing instants, so a span's member and its
bounds cannot disagree.

The karana is the limb that is not a cycle: sixty karanas make a lunar
month, Kimstughna opens it, Shakuni, Chatushpada and Naga close it, and
the seven movable ones repeat through the rest. The conventions page
measured the successor relation over all 109 consecutive karana pairs in
the corpus and it holds; the module computes the member from the
half-tithi's number in the month rather than from the previous member, so
a list that starts mid-month is still right.

## 5. Every period is one arithmetic operation on one of two arcs

Six families of period divide an arc into equal parts, and the corpus
reproduces every one of them exactly ([conventions §3](panchanga-day-conventions.md)):

| period | arc | parts | which |
|---|---|---|---|
| rahu kaal, yamaghanda, gulika | the daylight | 8 | one per vara, by table |
| choghadiya | the daylight, then the night | 8 each | all of them |
| hora | the daylight, then the night | 12 each | all 24 |
| muhurta | the daylight, then the night | 15 each | all 30 |
| Abhijit | the daylight | 15 | the 8th |
| Brahma muhurta | the night before sunrise | 15 | the 14th |

So there is one divider, and it belongs on the interval type rather than
in six places:

```rust
impl Interval {
    /// The interval divided into equal parts, in order.
    pub fn divided(self, parts: NonZeroU8) -> impl Iterator<Item = Interval>;
    /// One part of such a division, counted from zero.
    pub fn part(self, index: u8, of: NonZeroU8) -> Option<Interval>;
}
```

`Interval` itself goes in `core` — as `core::interval::Interval`, with
`from`, `to`, `days()`, `contains()`, `fraction_at()` and `clipped_to()`
— because `panchanga`, `muhurta`, `dasha` and `gochar` all return
intervals and none of them should define its own. The panchanga is simply
the first module that needs one.

The inauspicious eighths are a table of three rows by seven columns, and
the conventions page prints the one the corpus holds. Two of the three
rows are arithmetic and one is not, so all three ship as data.

## 6. The choghadiya is the hora's walk

A choghadiya is normally printed as a grid of fifty-six names. It is not a
grid. Measured over every recorded day ([conventions §4](panchanga-day-conventions.md)),
with no exceptions in 1320 horas and 880 choghadiya:

- the lord of the *k*th eighth of the daylight is the lord of the *k*th
  hora — the vara's lord walked five weekdays on per part, which is
  `time::hora::lord_of`;
- the night's walk starts four weekdays on from the vara and steps four;
- the name follows from the lord alone: Moon → Amrit, Jupiter → Shubha,
  Mercury → Laabh, Venus → Char, Sun → Udveg, Mars → Rog, Saturn → Kaal.

So the whole of the choghadiya is a seven-row table over a function the
SDK already has. Sixteen intervals a day come out of it and none of it is
a new algorithm. The horas themselves are `time::hora::horas(&day,
reckoning)` unchanged — the module does not divide that arc twice.

## 7. The muhurtas, and one difference worth registering

Abhijit is the eighth of the daylight's fifteen muhurtas and is void on
Wednesdays: measured on all 55 days, with the four Wednesdays void and
the other 51 effective.

Brahma muhurta is the fourteenth muhurta of a night, placed before
sunrise. Which night? The one that *ends* at that sunrise — and the
recording engine sizes it from the night that *follows* the day, the same
night its choghadiya divide. The two differ by the change in the length
of the daylight, which moves the start by a median of 10.0 seconds and at
worst 27.6 over the corpus. Small, systematic, and wrong. **The SDK sizes
it from the night it is in**, and this becomes a row of the
deliberate-difference registry rather than a defect either implementation
has to carry.

The thirty named muhurtas of a day are a catalogue addition this page
asks for but does not ship until a rank-1 citation exists for the names
and their order (§16). The *divisions* ship now, numbered, with Abhijit
and Brahma muhurta named, because those two have rules and the rules
measure.

## 8. The lunar month, and what belongs elsewhere

The purnimanta month is the amanta month, plus one through the dark
fortnight — measured on all 55 days. `calendars.lunar_month` chooses
which one the value leads with and the other is carried beside it, since
an application that shows one usually has a reader who wants the other.

The intercalation is not settled here. The corpus marks two adhika months
and no kshaya month, which shows the field is computed and does not test
the rule that computes it. Adhika and kshaya belong to
[`calendar-indian-lunisolar.md`](calendar-indian-lunisolar.md); this
module reads the month from that calendar and does not decide it.

The rule that calendar will apply is now measured
([`calendar-indian-lunisolar-measured.md`](calendar-indian-lunisolar-measured.md)):
**the count of sankrantis in the lunar month** — none is adhika, one is
ordinary, two is kshaya — which reproduces the marking on all fifty-five
recorded days. Two things the measurement settles for this module in
particular. The naming here needs **no** change: an adhika month has no
sankranti to be named by, and yet the Sun stands in the same sign at its
new moon as at the following nija month's, so `limb::amanta_month` gives
both the same name without a rule of its own. And the classification is
robust where the *month of an instant* is not — a boundary that moves by
half an hour almost never changes whether a sankranti falls inside a
29.5-day window, but it can change which month a moment minutes from a
new moon belongs to.

## 9. Panchaka, the ayana and the disha shool

- **Panchaka** runs while the Moon is in the last five nakshatras, and its
  kind is a function of the nakshatra alone: Dhanishtha → Mrityu,
  Shatabhisha → Agni, Purva Bhadrapada → Raja, Uttara Bhadrapada → Chora,
  Revati → Roga. The corpus reproduces the table on all nine days that
  carry one. It does **not** separate the nakshatra rule from the
  sign rule — the Moon is never in Dhanishtha's first half on a recorded
  day — so the SDK ships the nakshatra rule, which is what the texts
  state, and the difference stays an open item rather than a silent one.
- **The ayana** is uttarayana while the sidereal Sun is in Capricorn to
  Gemini: measured on all 55.
- **The disha shool** is the vara's, one direction each, and is a
  catalogue attribute rather than code.

Each of these is an interval and not a flag, for the same reason the
yogas are (§12): panchaka begins when the Moon enters Dhanishtha and the
ayana turns at a sankranti, and a reader wants the instant.

## 10. The frame is a knob

The corpus records the limbs twice — at the birth instant in `panchanga`,
and as the day's spans — and where the birth falls inside the window the
two can be compared. They disagree on 5 of 136 comparisons, all within
minutes of a boundary ([conventions §10](panchanga-day-conventions.md)):
the natal reading uses the topocentric Moon and the daily one is
geocentric, as every almanac is. That is entry 1 of the
deliberate-difference registry, now measured rather than observed.

It matters more than five in a hundred and thirty-six suggests. The lunar
parallax reaches about a degree, and the elongation advances about
twelve degrees a day, so a topocentric tithi boundary can be **two hours**
from the geocentric one. An almanac and a chart that disagree by two
hours about when a tithi ended are not two readings of one convention.

So `panchanga.centre` is a knob of its own, reusing the existing `Centre`
enum, defaulting to `GEOCENTRIC`. A profile that computes charts
topocentrically still gets a geocentric almanac unless it says otherwise,
and either way the choice is in the settings hash. The natal five limbs
stay on the chart, under `frame.centre`, where they already are.

## 11. The Moon's rise and set

Everything else in the recorded section is bounded by sunrise; the Moon's
rise and set are not. Twenty-four of the 108 recorded ones fall outside
the window the limbs occupy, and none falls before the local civil
midnight: they are the first rise and the first set at or after midnight
([conventions §9](panchanga-day-conventions.md)).

Two windows in one almanac is a defect, not a convention — a reader
cannot tell which one a row belongs to. The SDK reports the Moon's events
over the window the rest of the day uses, and `panchanga.moon_events`
carries the other reading for the profile that needs it:

```rust
pub enum MoonEvents {
    /// The rises and sets inside the panchanga day. The default.
    Window,
    /// The first rise and set at or after the local civil midnight,
    /// which is what most published almanacs print.
    CivilDay,
}
```

Under `Window` there may be **none, one or two** of each — a rise and a
set are about 24 h 50 m apart, so a 24-hour window sometimes holds two
moonrises and sometimes none. `Vec<Interval>`-shaped fields say so
honestly; a single nullable field would not.

## 12. The muhurta yogas are intervals, and they need a rank-1 source

This is the one limb the corpus cannot settle, and the page says so
rather than guessing. Over 55 days five flags fire 13 times on 12 days.
A vara-and-nakshatra yoga is a table of seven rows by twenty-seven
columns; twelve positive days cannot derive one, and the engine's
positives do not match the published tables — the standard Sarvartha
Siddhi table reproduces one of the engine's five and adds nine of its
own, and no rotation of the weekday or the nakshatra index does better
than four wrong.

Two things the corpus does settle, and both are design decisions:

1. **A vara-and-nakshatra yoga is keyed on the nakshatra at sunrise.**
   Reading any nakshatra of the day into the Amrit Siddhi pairs fires it
   on four days the engine leaves clear; reading only the sunrise one
   agrees on all 55.
2. **Tripushkar's classical rule holds** — a Bhadra tithi on a Sunday,
   Tuesday or Saturday in a three-footed nakshatra — on the corpus's one
   instance, with no false positives. Dwipushkar never fires, so its rule
   ships untested and says so.

So the yogas ship as **cited tables with the catalogue's confidence
marks** (`V` where a rank-1 or rank-2 source states the row, `T` where
only tradition does), selected by `panchanga.muhurta_tables` the way
`aspect.drishti_table` selects a drishti table, and they are reported as
intervals:

```rust
pub struct MuhurtaYoga {
    pub yoga: MuhurtaYogaKind,
    /// While it holds: the span of the nakshatra or tithi that makes it,
    /// clipped to the window.
    pub at: Interval,
    /// What made it, so a reader can see why.
    pub because: YogaCause,
}
```

The corpus's flags reduce to this exactly: a flag is "some interval
contains sunrise". Reporting the interval is strictly more information
and costs nothing, because the spans of §4 are already computed.

## 13. Tarabalam and chandrabalam

Both take a *second* input — the native's janma nakshatra, or the natal
Moon's sign — so neither is a property of the day. They are functions
over a panchanga:

```rust
pub fn tarabalam(day: &Panchanga, janma: Nakshatra) -> Vec<Span<Tara>>;
pub fn chandrabalam(day: &Panchanga, natal_moon: Rashi) -> Vec<Span<Bala>>;
```

They return spans because the day's nakshatra changes and the answer
changes with it. Neither is in the corpus, so both are held to their
texts and to property tests (the nine taras cycle, the twelfth and the
eighth are the weak ones) rather than to a fixture.

## 14. The API

```rust
/// The almanac over a provider, a solar model for the day's arcs, a
/// calendar and a clock: the same collaborators the chart foundation
/// takes, because it is the same day model underneath.
pub struct Almanac<'a, P: EphemerisProvider + ?Sized> { /* ... */ }

impl<'a, P: EphemerisProvider + ?Sized> Almanac<'a, P> {
    /// The almanac of one date at one place.
    pub fn day(&self, date: &CalendarDate, place: &Place)
        -> Result<Envelope<Panchanga>, Error>;

    /// The almanac of the day an instant belongs to, which before sunrise
    /// is not the day of its civil date.
    pub fn at(&self, instant: JulianDay<Utc>, place: &Place)
        -> Result<Envelope<Panchanga>, Error>;

    /// Every day in a range, both ends included: the primary shape.
    pub fn between(&self, from: &CalendarDate, to: &CalendarDate, place: &Place)
        -> Result<Envelope<Vec<Panchanga>>, Error>;
}
```

A month of days is the shape an application actually asks for, and it is
much cheaper than thirty days computed separately: consecutive windows
share a boundary, so day *n*'s next sunrise is day *n+1*'s sunrise. The
single-day call is the convenience over the range, never the other way
round, and `MOST_DAYS` is a year and a day — a caller asking for more is
asking for a different shape and the refusal names the limit.

`Panchanga` is a plain value — no interior mutability, no handle, no lazy
field — `Clone`, serialisable whole, and equal field for field between two
runs of the same inputs under the same settings. A date a caller writes
carries no era numbers and one a calendar renders does, so the value's
date is taken through the calendar and back: the same day asked for by
date and reached from an instant are *equal*, not merely the same day.

```rust
pub struct Panchanga {
    /// The day, with its arc, its vara and its polar state.
    pub day: LocalDay,
    /// What the spans are clipped to: the arc under `SUNRISE`, the civil
    /// day under `MIDNIGHT`.
    pub window: Interval,
    /// The four moving limbs, each a `Vec<Span<T>>`.
    pub limbs: Limbs,
    /// The inauspicious eighths the day has.
    pub kaalas: Vec<Kaalas>,
    /// Eight of the daylight and eight of the night, when it has both.
    pub choghadiya: Vec<Part>,
    /// Twenty-four, from `time::hora`.
    pub horas: Vec<Hora>,
    /// Thirty divisions, with Abhijit and Brahma muhurta named.
    pub muhurtas: Muhurtas,
    pub month: LunarMonth,
    pub moon: MoonDay,
    pub sun: SunDay,
    /// Panchaka, the muhurta yogas and the disha shool.
    pub omens: Omens,
}
```

with the accessors a reader actually wants — `tithi_at(instant)`,
`nakshatra_at`, `hora_at`, `choghadiya_at`, `is_inauspicious` — so that
"what is running now" is one call and not a linear scan the caller writes
five times.

## 15. Polar days, and absence reported as absence

The recording engine writes a period that cannot exist as a period of no
length: under polar day its twelve night horas are twelve intervals from
an instant to itself, and under polar night its day horas are
([conventions §11](panchanga-day-conventions.md)). An empty interval is
not the absence of a period. It is a period claiming to begin and end at
one instant, and code asking "which hora is it" gets an answer out of it.

The SDK reports absence as absence. `LocalDay` already carries
`DayState::Polar` with the policy that synthesised the bounds, and every
period whose arc does not exist is simply not in its list — which is why
those fields are vectors and `panchaka` is an option. `is_daytime()` on a
polar day is a question with no answer and the value says so, rather than
answering it wrongly.

| condition | outcome |
|---|---|
| a polar day under `polar_day_policy = UNDEFINED` | `UNSUPPORTED`, naming the policies that synthesise one |
| a polar day under any other policy | computed, with the periods of the missing arc absent and `DayState` saying why |
| the date is outside the provider's span | `UNSUPPORTED`, naming the span and the provider |
| a limb boundary the solver cannot bracket | the solver's own error, naming the quantity and the window |
| a date the civil calendar does not have | the calendar's `NONEXISTENT_DATE`, before anything runs |
| a settings pair that cannot both hold | the resolver's, before anything runs |

## 16. What the catalogue gains

Every attribute the limbs need already exists and already matches the
recording engine. The pass compares them: the tithi's paksha and class,
the nakshatra's muhurta nature, the yoga's auspiciousness and the
karana's Vishti flag agree member for member over every span of every day
([conventions §12](panchanga-day-conventions.md)), which is two
independently sourced tables agreeing rather than one copied. So the
module names members and lets the catalogue say what a member is, in
whatever language the caller asked for.

One numbering does differ and the harness has to know it: the corpus
numbers a tithi one to thirty through the lunar month, and the SDK's
catalogue numbers it one to fifteen within its paksha, which is how a
tithi is named. Both are right; they are not the same field.

Four kinds were missing and are now shipped, each the usual way — a YAML
file under `catalogue/`, `cargo xtask gen catalogue`, and
`check-catalogue` holding the generated code to it — with names in all
five locales:

| kind | number | members | mark |
|---|---|---|---|
| `choghadiya` | 58 | 7, each with its lord and whether it is auspicious | V, measured (conventions §4) |
| `kaala` | 59 | 3, each with the eighth it takes per vara | V, measured (conventions §3) |
| `panchaka` | 60 | 5, each with its nakshatra | V, measured (conventions §7) |
| `muhurta_yoga` | 61 | 5 | T, except Tripushkar, whose classical rule the corpus bears out |

The marks are the point: a caller can see that a choghadiya's name is
measured and a muhurta yoga's table is not. The thirty named muhurtas
wait on a citation for the names and their order; the divisions ship
now, numbered.

## 17. Localisation

No new namespace. The kinds above take entries in `sdk.entity`, the way
every catalogue kind does, and the instants are formatted by
`sdk.calendar`'s existing `time`, `datetime` and `duration` messages. A
sentence like "Amrit Choghadiya until 09:14" is the entity name and a
time, composed by the caller; the SDK does not own the sentence.

The four locales the SDK ships (`en-Latn`, `hi-Deva-IN`, `ne-Deva-NP`,
`sa-Deva` with its `sa-Latn` transliteration) each gain the new kinds'
names, and `check-intl` refuses a locale that is missing one.

## 18. Performance

Per day, over a provider: three crossing searches for the four limbs, one
for the Sun's sign, one for the Moon's, and two rise-and-set solves. The
periods are arithmetic — sixteen choghadiya, twenty-four horas, thirty
muhurtas and three kaalas cost thirty additions between them — and the
classification is integer.

A month asked for as a range costs a little more than a single day, not
thirty times more: one search over the month's span yields every limb
boundary in it, and the arcs are the thirty `LocalDay`s, which share
their boundaries. The budget the benchmark holds is the ratio: **a
thirty-day range under 3× a single day**, measured in instructions by
`cargo xtask bench` over a `crates/scenario` case added with the module.

## 19. Tests

The crate has 59 of them, in four files. What each can and cannot decide
is the design's own division: the periods and the tables are arithmetic
and are compared against the corpus exactly; the limb *instants* are
positions over time and no provider inside this workspace has real
positions, so they wait on the conformance harness over an adapter.

**`tests/baseline.rs`** — the corpus, computed from the arcs the corpus
itself recorded:

| what | against | measured |
|---|---|---|
| the three kaalas | `rahu_kaal`, `yamaghanda`, `gulika_kaal` | 159, worst 0.04 ms |
| the choghadiya and their lords | both recorded sequences | 848, worst 0.04 ms |
| the horas and their lords | `hora` | 1272, worst 0.04 ms |
| Abhijit, and its Wednesday void | `abhijit` | 53, worst 0.04 ms |
| Brahma muhurta | asserted **different** by the registry's amount | 52, median 10.02 s, worst 27.56 s |
| the month relation | `lunar_month`, both conventions | 55 |
| the classification of a real position | the recorded Sun and Moon at each birth instant, into all four limbs | 55 |
| panchaka, the ayana, the disha shool | `panchaka`, `sun_sign.ayana`, `disha_shool` | 55, 9 panchakas |
| the limbs' recorded attributes | the SDK's catalogue | 336 |
| a lord names one choghadiya | both recorded sequences | 880 |

Worst 0.04 ms is the last bit of a double near two and a half million,
which is what arithmetic against arithmetic leaves.

**`tests/kernel.rs`** — the limb kernel's own properties over the
analytic test provider, which is the wrong sky at the right speed: the
spans cover the window and are in order, consecutive members share one
instant, every span carries both pairs of bounds, a tithi boundary is a
karana boundary, a day inside a week is the same day to the solver's own
tolerance, and every member is the one the integer path names.

**`tests/almanac.rs`** — the assembled value: every period divides the
arc of the day it is reported with, the window moves with
`day.day_boundary` and the periods do not, `panchanga.moon_events` moves
two fields and nothing else, a day reached from an instant inside it is
the same value, a range is its days and shares their boundaries, and the
stamp carries the profile and both hashes.

**The modules' own tests** — the karana's chain over a whole month, the
weekday walk against the Chaldean order, the eighths table on a Saturday
and a Sunday, a half with no arc contributing no periods, Tripushkar
needing all three of its conditions, a table the SDK does not ship
refused by name.

Still to come, with the harness: every limb span's instants against
`panchanga_day.{tithi,nakshatra,yoga,karana}` at the corpus's declared
tolerance (1e-5 day), the Moon's rise and set under `moon_events =
CIVIL_DAY`, the Sun's and Moon's signs and their transitions, and the
**rank-1** rows — eight printed tithi end instants and 22 printed day
arcs from Nepal's national panchanga committee, which are the only
evidence in the project that is not another implementation.

## 20. Open questions

- **Panchaka's window.** The nakshatra rule and the sign rule differ only
  in Dhanishtha's first half and the corpus has no such day. The SDK
  ships the nakshatra rule; a rank-1 panchangam with a Dhanishtha morning
  would settle it in one row.
- **The muhurta yoga tables.** Which tradition's Sarvartha Siddhi and
  Siddha tables to ship as the default, and whether the recording
  engine's are a published variant or its own. Needs a text, not a
  measurement.
- **The thirty muhurta names**, which the catalogue wants and no source in
  the project yet states in order.
- **Whether `panchanga.centre` should default to the profile's frame in
  a profile that says nothing.** The default here is `GEOCENTRIC`
  unconditionally, which is what almanacs do; the alternative — follow
  `frame.centre` — makes a chart and its almanac agree at the cost of an
  almanac that no published one matches. The current answer is the one
  the corpus supports.
- **Whether a synthesised polar arc should carry sixty-two tithis.**
  Under `polar_day_policy = NEAREST_EVENT` a day at 69.65°N in midsummer
  is bounded by the nearest real sunrise and sunset, which can be a
  fortnight apart. Everything then follows correctly and absurdly: the
  divisions stay fixed (twenty-four horas, each fourteen hours long) and
  the crossings multiply — sixty-two tithis, a hundred and twenty-two
  karanas, forty moonrises in one "day"
  ([`panchanga-at-the-boundary-measured.md`](panchanga-at-the-boundary-measured.md)
  §1). Nothing is wrong with any single number; what is doubtful is that
  an almanac row is the right shape for the answer. The alternative is
  that a polar day is a different question from the one `Almanac::day`
  answers, and should be refused or reported as a span rather than a
  day. The measurement makes the choice visible; a source, or a rank-1
  polar panchangam, would settle it.
- **The vara under `day_boundary = MIDNIGHT`.** The window moves; the
  vara of a day is still the civil date's, and the horas still start at
  sunrise. Nothing measures this, because the recording engine has no
  such setting.
