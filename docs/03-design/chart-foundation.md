# The chart foundation

Status: `draft`, written 2026-09-07 as the first design of Phase 4, after
the bhava chalit falsification pass
([`chart-bhava-chalit.md`](chart-bhava-chalit.md)) which decided what a
house placement has to carry. Derives from
[`02-architecture/01-module-catalog.md`](../02-architecture/01-module-catalog.md)
(the `chart` row and its `ChartFoundation`),
[`settings-and-profiles.md`](settings-and-profiles.md) (every knob it
reads), [`astro-house-systems.md`](astro-house-systems.md) (the cusps),
[`time-and-timezone.md`](time-and-timezone.md) (the day and its arcs) and
[`ephemeris-port-and-adapters.md`](ephemeris-port-and-adapters.md) (where
the positions come from). The rank-2 reference is the conformance
corpus's `foundation` section on all 55 charts.

## 1. Purpose and scope

Every module above this one — vargas, state, aspects, panchanga,
strengths, dashas, rules — starts from the same handful of facts about
one moment at one place: when it was, where the day it belongs to began
and ended, where the lagna is, where the cusps are, where the grahas are,
and under which settings all of that was decided. Computing those facts
once, stamping them, and handing them on is what the `chart` module is
for.

It settles: what a foundation holds; which day a chart belongs to, which
is not the civil date; what anchors the lagna; what a house placement
carries; what the foundation deliberately does *not* hold; and how a
batch of instants is founded together.

It is not: the vargas (`varga-kernel.md`), the panchanga day's limbs
(`panchanga`, a later page), the strengths or the dashas. Those consume a
foundation and none of them may recompute one.

## 2. Inputs, settings and ports

| input | from |
|---|---|
| the instant | resolved by `time` from civil input, or given as a Julian day |
| the place | latitude, longitude, altitude (`core::quantity::Place`) |
| the chart kind | `ChartKind`: natal, transit, event, prashna, return, relocated, composite |
| the settings | a resolved `Settings` with its hash (`core::settings`) |
| the ephemeris | the port, through the override policy the profile sets |

The knobs it reads, each already existing:

| knob | what it decides here |
|---|---|
| `frame.*` | the zodiac, the ayanamsha, the node, the centre, the positions |
| `houses.placement_system` | the cusps a graha is placed against |
| `houses.chalit_system` | the bhava the placement reports (§5) |
| `houses.polar_policy` | what stands in for cusps inside the polar circle |
| `day.day_boundary`, `day.sunrise`, `day.polar_day_policy` | which arc the chart belongs to (§3) |
| `day.ghati_reckoning`, `day.hora_reckoning` | the birth timing and the hora lord (§7) |
| `provider.*` | which ephemeris answers, and what the SDK completes |

Nothing here introduces a knob. A foundation that needed a knob of its
own would be a foundation that made a choice the profile could not see,
and every result carries the profile's hash.

## 3. The day a chart belongs to is not its civil date

This is the crux of the module, and the corpus was recorded to prove it.
Chart `c001` is a birth at 05:30 in Kathmandu on 14 April 1990 —
Nepali New Year — and sunrise that morning is 05:30:44. The recorded
foundation says `kind: pre-sunrise`, the day arc it gives runs from the
*previous* evening's sunset to that morning's sunrise, `is_day_birth` is
false, and the sunrise that anchors the lagna is the *previous* day's.

A panchanga day runs from sunrise to sunrise. An instant before the civil
date's sunrise belongs to the day that began the morning before, and with
it the vara, the hora sequence, the ishtakaal and the lagna's anchor all
move back one day. An implementation that takes the civil date and looks
up its sunrise gets a chart that is wrong by a day for every instant
between midnight and sunrise — a quarter of the clock, and every one of
those charts plausible-looking.

`time::local_day::local_day` answers "what is the arc of *this date*".
The foundation needs the inverse — "what arc holds *this instant*" — and
it is one step: take the civil date at the place, ask for its arc, and if
the instant precedes that arc's sunrise, take the previous date's arc
instead. The answer is a `DayArc`:

```rust
pub struct DayArc {
    /// The day the instant belongs to, which may be the civil date
    /// before it.
    pub day: LocalDay,
    /// Which part of the arc holds the instant.
    pub part: DayPart,
    /// How far through its part the instant is, 0 at the start and
    /// 1 at the end.
    pub elapsed: f64,
}

pub enum DayPart {
    /// Between sunrise and sunset of the day it belongs to.
    Daylight,
    /// Between sunset and the next sunrise.
    Night,
}
```

`DayPart` is two members and not three. "Pre-sunrise" is not a third part
of a day: it is the night of the previous day, and the corpus's own
`kind: pre-sunrise` is a label on that night, not a part beside it.
Calling it what it is stops the question "does pre-sunrise count as
night?" from being asked once per module.

The polar case is `day.polar_day_policy`, and the foundation does not
decide it: `local_day` already reports a synthesised arc, the nearest
event, or a refusal, and whatever it reports is what the foundation
carries, with the `DayState` that says which.

## 4. The lagna and its anchor

The lagna is the ascendant of the chart's instant, in the chart's zodiac:
`astro::houses` gives it with the cusps, and the sidereal offset comes
from the resolved ayanamsha. That is the whole of it, and it is not where
the difficulty is.

The difficulty is that several later modules do not want the lagna of the
*instant*; they want the lagna of the **sunrise that began the chart's
day** — the arudha reckonings, the birth timing's proportions and the
Lagna-based dasha variants all measure from it. The corpus records it as
a field of its own (`lagna_sunrise_jd`), and for `c001` it is the
previous day's sunrise, following §3.

The foundation therefore carries both, named so that neither can be
mistaken for the other: `lagna` (at the instant) and `day_lagna` (at the
sunrise that opened the arc). A caller who wants "the lagna" gets the
first; a module that wants the anchor asks for the second and says so.

## 5. A house placement carries the method that produced it

The falsification pass measured what happens when it does not. Over the
55 charts, the four named bhava chalit methods put a graha in a different
house between 10% and 51% of the time depending on the pair; the two a
Jyotisha application actually chooses between disagree on 21.8% of
placements, and 37.2% beyond 30° of latitude. Two of them — Sripati and
Porphyry — are the *same cusps* read two ways and disagree half the time.

So a placement is not a number:

```rust
pub struct Placement {
    /// The bhava, 1 to 12.
    pub bhava: u8,
    /// Which chalit put it there.
    pub method: HouseSystem,
    /// How far through its bhava the graha is, 0 at the sandhi that
    /// opens it and 1 at the one that closes it.
    pub through: f64,
    /// How far from its bhava's madhya, in degrees, signed.
    pub from_madhya: f64,
}
```

`through` and `from_madhya` are there because `astro::houses::Houses`
returns the cusps, the auxiliary angles and the outcome, and has no
notion of a madhya at all. Cusps are enough to say which bhava a graha is
in and not enough to say how near the middle of it the graha sits — which
is the whole of bhava bala, and the thing the strength page will need.
The pass named this gap; the foundation closes it by carrying the madhya
beside the sandhi:

```rust
pub struct Bhavas {
    /// The boundaries, bhava 1 first.
    pub sandhi: [f64; 12],
    /// The middles, bhava 1 first.
    pub madhya: [f64; 12],
    /// Which method these are.
    pub method: HouseSystem,
    /// Whether the method was computed as asked or stood in for.
    pub outcome: Outcome,
}
```

For a quadrant chalit the sandhi are the cusps and the madhya are the
midpoints; for Sripati the cusps are the madhya and the sandhi are the
midpoints between them; for Vehlow the madhya are the ascendant and every
30° from it. One type, four fillings, and the method is in the value.

The chart carries **two** sets: the placement system (`houses.placement_system`,
whole sign by default, which is what "in the 7th" means in most of the
tradition) and the chalit (`houses.chalit_system`). They are different
questions and a result that conflates them is the defect entry 14 of the
deliberate-difference registry records.

## 6. Positions

The port answers with the columns it is asked for; the foundation asks
for one grid — the chart's instant by the catalogue's grahas — and keeps:

| kept | why |
|---|---|
| sidereal and tropical longitude | the chart's zodiac and the frame it was completed from |
| latitude, distance | the phenomena and the topocentric step need them |
| speed in longitude | retrogression, and every rate the event kernel takes |
| declination and right ascension | the aspects that are not ecliptic, and the bhava bala |
| the completion steps applied | provenance: which corrections this position has had |

Both frames are kept and neither is recomputed, because a module that
needs the other one and converts it itself will use a different ayanamsha
than the chart did on the day someone changes the setting.

The nodes are what `frame.node` says (mean or true) and Ketu is Rahu plus
180° **in the same frame**, which is entry 6 of the registry. Nothing in
the foundation invents a latitude, speed or distance for a node that the
provider did not give.

## 7. Birth timing, the hora and the abda

All of it comes from the arc of §3 and `time`'s own reckonings, and none
of it is recomputed here:

| fact | from |
|---|---|
| ishtakaal, in ghati and pala | `time::ghati` over the arc, under `day.ghati_reckoning` |
| bhayat and bhabhoga | the elapsed and total of the part, in ghati-pala |
| the hora lord | `time::hora::hora_at` under `day.hora_reckoning` |
| the vara | the `LocalDay`'s, which is the arc's and not the civil date's |
| the abda lord | the year lord of the arc's own year |

The corpus records ishtakaal twice, `civil` and `proportional`, because
the recording engine computes both. The SDK computes the one
`day.ghati_reckoning` names and says which; a caller who wants both asks
twice under two settings, and gets two hashes, which is the point.

## 8. The upagrahas

Seven shadow points, each a stated function of the Sun's longitude or of
the day's arc, each with its own definition and its own disagreements
between texts. They are in the corpus's foundation and they belong in the
`points` module, not here: the foundation holds what every module needs,
and no module needs an upagraha to compute anything else. `points` takes
a foundation and returns them.

## 9. What the foundation does not hold

The corpus's `foundation` section carries `arudha_lagna_sign_index` and
`navamsha_lagna_sign_index`. The SDK's does not, and the difference is
deliberate rather than an omission: the navamsha lagna is a varga of the
lagna, the arudha is computed from a placement and its lord, and both
therefore depend on modules that depend on the foundation. Putting them
in it would make the dependency circular, and the way that circularity
usually gets broken — the foundation computing a little varga of its own
— is how two evaluators of the same rule come to exist.

The rule: **the foundation holds what is needed to compute, never what is
computed.** A field belongs in it when more than one module above needs
it and no module above can produce it.

## 10. The mixed-chart axis

`varga-kernel.md` settles that `(varga_for_planets, varga_for_lagna)` is
a chart context parameter and not a property of a varga. It rides on the
foundation as exactly that: a pair the caller sets, defaulting to `(D1,
D1)`, carried in the value and hashed with the settings. The foundation
computes nothing from it; the varga service reads it.

## 11. Batch

Principle 5: batch is the primary shape. A foundation for many instants
at one place shares the day arcs (a whole month of charts is a month of
arcs, not a month of arcs per chart), one settings resolution, one
provider request with the whole grid, and one provenance. The API is
therefore a batch with a single-instant convenience over it, never the
other way round.

## 12. Errors

Every failure names the field and what would fix it, as everywhere else:

| condition | outcome |
|---|---|
| the instant is outside the provider's span | `UNSUPPORTED`, naming the span and the provider |
| a polar day under `UNDEFINED` | `UNSUPPORTED`, naming the policies that synthesise one |
| the placement or chalit system is degenerate at this latitude | computed under `houses.polar_policy`, with `Outcome` saying which stood in — never an error, because a substituted house is still a house and the value says so |
| the provider refuses a body | the provider's own code and message, unchanged |
| a settings pair that cannot both hold | the resolver's, before any of this runs |

## 13. Provenance

One `Envelope<ChartFoundation>`, with the `Provenance` `core` already
defines: the SDK and module versions, the calculation and catalogue
versions, the profile and its settings hash, the input hash, the provider
stamp, and the time stamp. Every module above passes the foundation's
provenance on rather than making its own, so a chart and everything
derived from it carry one story.

## 14. The API

```rust
// The batch, which is the shape.
pub fn found(
    request: &FoundationRequest,
    provider: &dyn EphemerisProvider,
    settings: &Resolved,
) -> Result<Envelope<Vec<ChartFoundation>>, Error>;

// The convenience.
pub fn found_one(
    instant: JulianDay<Utc>,
    place: &Place,
    kind: ChartKind,
    provider: &dyn EphemerisProvider,
    settings: &Resolved,
) -> Result<Envelope<ChartFoundation>, Error>;
```

`ChartFoundation` is a plain value: no interior mutability, no handle, no
lazy field. It is `Clone` and cheap to pass, it serialises whole, and two
foundations of the same inputs under the same settings are equal field
for field — which is what the determinism contract asks of everything
that crosses a binding.

## 15. Tests

| what | against |
|---|---|
| the day arc, including every pre-sunrise birth | the corpus's `foundation.panchanga_day` and `sunrise` blocks on all 55 charts |
| the lagna and its anchor | `foundation.lagna` and `lagna_sunrise_jd`, all 55 |
| the placements under each chalit | the corpus's `houses.bhava_chalit.planet_houses`, and the falsification pass's own numbers |
| the madhya and sandhi | `bhava_madhya` and `bhava_sandhi`, all 55 |
| positions in both frames | `positions.bodies`, all 55, within the corpus's tolerance band |
| the birth timing | `foundation.birth_timing`, all 55, under the reckoning the fixture used |
| the hora and abda lords | `foundation.hora_lord`, `abda_lord`, all 55 |
| batch equals single | every chart founded both ways, field for field |
| a foundation is not recomputed | the module boundary: no module above `chart` calls the provider |

The registry's entries are expected differences and are asserted as
differences, not skipped: a test that reproduces entry 14 by computing
Vehlow and Sripati and finding them apart is worth more than one that
avoids the subject.

## 16. Open questions

- The abda (year) lord's own year: which reckoning bounds it — the solar
  year from Mesha sankranti, the civil year, or the samvatsara. The
  corpus records the lord and not the rule, so this needs a rank-1
  source before it is implemented rather than after.
- Whether `ChartKind::Composite` founds one value from two, or is a pair
  of foundations with a composite view over them. The second costs
  nothing until a composite is asked for and cannot lose the two charts
  it came from; it is the likely answer and is not settled here.
- Whether the placement system's `Bhavas` is worth carrying when it is
  whole sign, where the madhya are a formality. Carrying it uniformly is
  simpler and costs 24 doubles; that is the current answer.
