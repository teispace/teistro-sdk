# A chart founded on a classical astronomy, measured

Status: `generated` by `cargo xtask classical-chart` over the Surya
Siddhanta provider. Do not edit: `check-classical-chart` regenerates
this page and fails on any difference.

The question `siddhanta.md` §5 leaves to the chart layer, and
`classical-chart.md` answers: when a chart is founded over the text's
provider, which of its parts are the text's?

## 1. What was founded

Every recorded birth of the corpus (55 of them) was founded **through
the SDK** over `Ephemeris::SuryaSiddhanta`, as a Rust consumer opens it,
under the root profile with the text's own ayanamsha named
(`SURYASIDDHANTA`); the root profile's sunrise is already the text's,
the centre on the geometric horizon. Each chart was then held against
what `crates/siddhanta` answers at the same instant and place: its Lagna
(III.46 to 49), its ayanamsha, its nine grahas and its own day arc,
through which the same hora reckoning was counted. The text refuses
`c028-troms-1988-06-21` and `c029-troms-1988-12-21`, because on the day
it has no sunrise to count the Lagna from; the SDK refuses
`c028-troms-1988-06-21` and `c029-troms-1988-12-21` under the root
profile's polar-day policy, which synthesises no day.

The steps the provenance stamps on each chart: `positions:NATIVE,
corrections:NATIVE, ayanamsha:NATIVE, zodiac-shift:SDK, zodiac:NATIVE,
angles:NATIVE, day:NATIVE`.

## 2. Measured: which parts are the text's

Over the 53 births both answer, each part of the chart the SDK founded
against the text's own, to within a rounding (1e-9° or 0.001 s):

| part | the text's | median | worst | note |
|---|---|---:|---:|---|
| the zodiac: the chart's ayanamsha against the text's | yes | 0.000° | 0.000° |  |
| the grahas, all nine, in the chart's zodiac | yes | 0.000° | 0.000° |  |
| the Lagna, in the chart's zodiac | yes | 0.000° | 0.000° | 0 of 53 in another sign |
| the midheaven, as `sdk.chart().angles` answers it | yes | 0.000° | 0.000° |  |
| the day's sunrise | yes | 0.000 s | 0.000 s | 0 of 53 in another hora |

**The obliquity is not the difference.** The spherical ascendant on the
text's own 24° stands a median 1.547° and at worst 8.255° from the
text's Lagna, in another sign on 5 of 53 births. The text reckons the
Lagna from the Sun at its own sunrise, carried through the signs' rising
times in proportion within each sign (Burgess's note under III.46 to 49
calls this the text's own approximation), on a clock with no equation of
time, so only the text reproduces it.

## 3. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| the zodiac is the text's | **holds** | worst 0.000° |
| the grahas are the text's | **holds** | worst 0.000° |
| the Lagna is the text's | **holds** | worst 0.000° |
| the midheaven is the text's | **holds** | worst 0.000° |
| the sunrise is the text's | **holds** | worst 0.000 s |
| the Lagna is in the text's sign | **holds** | 0 of 53 disagree; worst 0.000° |
| the hora is the text's | **holds** | 0 of 53 disagree |
| the text's obliquity alone would give the text's Lagna, in sign | falsified | 5 of 53 disagree; worst 8.255° |
| the SDK founds every recorded birth over the text | falsified | 2 of 55 disagree |
| the text answers a Lagna for every recorded birth | falsified | 2 of 55 disagree |

Three of the ten claims are falsified. The parts the text gives the
chart: the zodiac; the grahas; the Lagna; the midheaven; the sunrise.
The parts the chart still computes itself: none.

