# The annual chart (Varsha Pravesha)

Status: `built`, 2026-09-22 — written **after** the falsification pass,
corrected by it, and then built ([`annual-chart-measured.md`](annual-chart-measured.md),
`check-varshaphala`). Phase 7's Tajika module begins here, and the dashas
over the annual chart wait on it
([`dasha-coverage-measured.md`](dasha-coverage-measured.md)).

`crates/tajika` holds it and `sdk.chart().praveshas` and
`sdk.chart().annual` reach it. **The measured page moved to the shipped
module and its numbers did not change**, which is the only evidence that
the thing measured and the thing built are one thing — a pass that keeps
its own copy of what it measures measures itself.

## What it is

The annual chart is a chart cast for one instant: the moment the Sun
returns to the longitude it held at birth. Everything in Tajika hangs off
it — the Muntha, the lord of the year, the sixteen yogas, the Mudda and
Patyayini dashas — so the instant is not a detail of the technique but
its whole foundation, and an error in it is an error in every judgement
made from it.

## The rule, and why it had to be measured

**The Sun returns to its natal *sidereal* longitude, read as the chart
reads it.** Three words in that sentence are decisions, and the pass
measured each against its rival over every recorded birth — how many
that is, the measured page counts and this one does not.

**Sidereal, not tropical.** The tradition's return is to the sidereal
longitude; the Western return is to the tropical one. They are not the
same instant, because the ayanamsha moves between them — and forty years
on they are **14.2 hours** apart, which puts the lagna **177°** away.
That is not a different chart by a little; it is the opposite sign, a
different lord and a different reading of every house. The rival is
measured on the page rather than dismissed in a sentence, because a
consumer building a Western tool wants it and should be able to ask for
it by name.

**True, not mean.** "One sidereal year per year of life" is the older
arithmetic and is still taught. It is **14.5 minutes** out at forty and
moves the lagna **5.6°** — less dramatic than the tropical rival and
still a house boundary when the birth is near one. A setting, not a
default anyone may quietly pick.

**Read as the chart reads it.** This is the one the pass caught, and it
would not have been caught by reading the rule. A `Frame`'s own
`Zodiac::Sidereal` applies the **mean** ayanamsha; a founded chart under
this profile applies the **nutated** one, and the two are 18.46
arcseconds apart — about seven minutes of the Sun's time, and about two
degrees of lagna. The first shape of the pass searched the frame's
sidereal longitudes, and reading the answer back through a founded chart
falsified it by exactly that. It reads **0.000 arcseconds** now.

Two things follow, and both are now facts about the repository rather
than notes here:

- The read-back goes through a **founded chart** and never through the
  source the search already used. A solver asked whether it landed on its
  own lattice line will always say yes.
- The shift belongs in one place. `teistro_astro::sidereal` now holds the
  `Sidereal` source and the `Zodiac` it reads in — it was the panchanga
  limb search's private adapter, and the annual chart is the second
  consumer that needs exactly it. A second copy would have been a second
  thing to get 18 arcseconds wrong.

## What the search costs, and why that is settled too

A return is the crossing search this SDK already has: `Quantity::Longitude(Sun)`
over a lattice of one line at the natal longitude. Nothing new is solved.

The sampling step is the whole cost, and the default of one day samples
forty years of every birth. The Sun never turns and moves about a degree
a day, so a **ten-day** step cannot pass the one target twice between two
samples; the pass takes it and the page is byte-identical at one day and
at ten, which is the check rather than the claim.

Taking it found a second defect the slower step had hidden: the search
brackets a crossing *between* two samples, so it reads up to a step below
where it was told to start, and a birth at the very edge of the ephemeris
then fails on an instant the caller never asked about. The window starts
a step in.

## What is decided and what is not

| | |
|---|---|
| **decided** | the Sun's return to the natal sidereal longitude, read on the chart's own ayanamsha basis; the tropical and mean returns as named rivals; the crossing search and its ten-day step |
| **a setting** | which of the three readings; how many returns; the place the return is cast for |
| **not decided** | whether the annual chart is cast for the birthplace or the current residence — the schools differ and the corpus is silent. **The SDK does not decide it for anyone** ([`muntha.md`](muntha.md), "Where a year's chart is cast"): `varsha_json.place` is `"birth"` or a residence, written by the caller, and absent means no chart is founded. The one text read casts for the birthplace without stating a rule, so the crux stays open until a text gives one |
| **across the boundary** | `TsChartRequest.varsha_json` names the reading and the years; the instants come back in the charts blob's `praveshas` section, ragged by `cast.pravesha_count`. **The boundary answers the instant and not the chart**, which is the residence question above respected in the API rather than decided by it: a caller founds the year with the place they mean, and that second call is the same `found` they already know. Node, Dart and Python each take a `varsha` request and read `chart.praveshas`, each has a worked example, and all four parity runners print all three readings so a binding that crossed one as another would be caught |
| **built** | `teistro_tajika::praveshas` and `mean_praveshas`; `sdk.chart().praveshas(&document, reading, through)` for the instants and `sdk.chart().annual(&document, reading, year, &request)` for the chart a year opens. A year past what the provider covers comes back as **fewer instants**, not as the provider's own `OutOfRange` naming a day the caller never mentioned; a year outside one to two hundred is refused by `through` **before** any capping, because capping a nonsense year to zero would answer an empty list where the caller wanted to be told |
| **not built** | the Varshesha by Pancha Vargeeya Bala and the four office-bearers besides the Muntha; the Tajika aspects with their deeptamsha orbs; the sixteen yogas; the sahams (built since, [`tajika-sahams.md`](tajika-sahams.md)); the Mudda and Patyayini dashas. Each is a step above this one and none of them can be built before it. **The Muntha and its lord are built** ([`muntha.md`](muntha.md)), and they are the one office-bearer that can ride with the instant, because only they need nothing but the birth |

## What the corpus cannot settle

The conformance repository records **no annual chart of any kind**. So
nothing above is checked against a recording, and the page says so on its
first line rather than reading as though it were. What the corpus does
record is **births**, and a return is a property of a birth, so the
rule is measurable over them even where its answer was never written
down — which is the same move
[`horizon-absence-measured.md`](horizon-absence-measured.md) makes for a
different silence.

A recorded annual chart would settle two things this cannot: the
residence question above, and whether a school rounds the instant to the
minute before casting. Both are named here so that a later export knows
what to ask for.
