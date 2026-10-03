# Synastry: the Western aspects between two charts (the `western` module)

Status: `built`, 2026-10-03, written from the sources before any code.
`western::synastry` reads on the engine a chart's own table reads, and
`sdk.chart().synastry` is held to George V and Queen Mary, measured over
every pair of the corpus's births in
[`synastry-measured.md`](synastry-measured.md), and crosses to every
binding as `chart.synastry`, with parity.

[`western-aspects.md`](western-aspects.md) answers *which aspects does
this chart hold?* Synastry asks the same of two charts: which of one
person's planets stand in aspect to the other's. It is the oldest form
of chart comparison in the Western texts, and it reads nothing a single
chart's table does not already measure. Only the pairs are new: every
point of the first chart against every point of the second.

## What the sources decide

**Alan Leo**, *How to Judge a Nativity* (1928 edition), is again the
rank 1 text. It was read on the page images of the Internet Archive scan
`howtojudgenativi00leoa`.

- **The luminaries interchanged** (p. 189, the seventh house): "The
  best testimony for a marriage … is the interchanging of the
  luminaries. The Sun of the native in the same place as the Moon of the
  partner or the Moon near the partner's Sun … or even a good aspect
  between the luminaries in the two horoscopes is very favourable."
- **The ascendants and the rulers** (p. 221, the eleventh house): "The
  luminaries in favourable accord in two nativities will produce
  sympathy, while if, moreover, the ascendants be in good aspect to one
  another a lasting and firm friendship will result." The same holds when
  "the ruling planets are in favourable aspect to each other".
- **One planet on another's place** (pp. 222–223): each planet "in one
  horoscope on the place of Venus in another", from Neptune to the Moon,
  and the benefics "in one horoscope" occupying "the place of the
  luminaries or ascendant in the other". A father's Sun "on the place of
  the Moon in his daughter's nativity" is his one worked case, without
  data.

So the comparison is an aspect table across the two charts. Its points
are the planets of each and the two ascendants. Leo gives the reading
no orbs of its own, so the orbs are his orbs for a single chart (p. 47),
under whichever model the caller chose.

**The fixture.** Leo gives a married pair's births on p. 130: King
George V, "born 1-18 a.m., 3rd June, 1865, London", and Queen Mary,
"born 11-59 p.m., 26th May, 1867, London". He reads neither synastry,
but he describes each chart: George's "Neptune rising in Aries in close
sextile with the Sun", and Mary's Jupiter "in Pisces intercepted in the
ascendant in dexter square to the Sun". Cast at Marlborough House and
Kensington Palace, both remarks hold, which is what makes the two
charts a fixture. A recast with the Moshier ephemeris (pyswisseph, rank
3) gives the cross contacts the test holds, among them his Mars
sextile her Sun (0.39° from exact), his Mars opposite her ascendant
(0.32°) and his Venus sextile her Moon (1.23°).

## Decisions

1. **The rows run across, never within.** Every point of the first
   chart is read against every point of the second, so the Sun of one
   against the Sun of the other is a pair, and the first chart's Moon
   against the second's Sun is a different pair from the first's Sun
   against the second's Moon. A chart's aspects to itself are its own
   table's business.
2. **The points are the planets and the lagna.** The seven, and the
   outer three when that chart placed them, as the single chart's table
   reads; and each chart's lagna, which Leo names twice. A row names
   each side as a `NatalPoint`, which the hit list already gives every
   binding: a graha, or the lagna. The lagna can be left out, since it
   is the point an uncertain birth time moves most.
3. **The lagna stands as a planet in the orbs (C242).** Leo gives the
   ascendant no orb. Under his model it takes a planet's place in his
   rule: 8° at the conjunction and opposition beside a planet or the
   other lagna, 10° beside a luminary, and never the luminaries' own
   12°. Under Lilly's moieties it has none, so a request that keeps the
   lagna and reads moieties is refused by field, as Uranus is.
4. **No applying.** Two births do not move against each other, so a
   synastry row has no `applying` and is a type of its own rather than a
   `WesternAspectRow` with a field that means nothing.
5. **The zodiac is tropical by default (C241).** A separation between
   two charts founded years apart depends on the zodiac: in a sidereal
   one, each chart's longitudes carry its own instant's ayanamsha, and
   the two differ by the precession between the births, about 50″ a
   year. Leo's frame is tropical, and every placed graha already carries
   its tropical longitude. `SynastryZodiac::Charts` reads each chart in
   its own zodiac instead, for a sidereal reader, and refuses two charts
   founded in different zodiacs.
6. **The table is one engine.** The single chart's `aspects` and the
   synastry share one loop over pairs and one orb rule. Only the pairs
   handed to it differ, so an orb is never measured twice.
7. **Across the boundary, a partner on the chart request.** A chart
   batch is founded at one place, and two people are not born at one,
   so the partner's birth travels whole in a `synastry` record beside
   the aspect table's fields: `{"partner": {"instant", "place",
   "utcOffsetSeconds"}, "aspects", "orbs", "lagna", "zodiac"}`. The
   SDK founds the partner once for the batch
   (`ChartArea::synastry_with`) and reads every chart against it, so
   one person read against many candidates, or a life's progressed
   charts against one partner, is a single call. Sections 84 and 85
   carry the rows, each side a `NatalPoint` in the hit list's two cells.
   A call of its own, taking two births, was the first plan; it would
   have repeated the chart request's whole founding surface for a
   second person and given up the batch.

## Order of work

1. This page, and the cruxes C241 and C242.
2. `western::synastry` beside `aspects` on one engine, and
   `sdk.chart().synastry`. George V and Queen Mary are the test: Leo's
   two remarks inside each chart, then the cross contacts against the
   recast.
3. A measured page over pairs of the corpus's births: how often the two
   zodiac readings part, and by how much.
4. The `synastry` record on the chart request and every binding, with
   parity.

## What building it found

- **The table agrees with an independent recast.** Every Ptolemaic
  contact within 2.5° of exact between George V and Queen Mary, nine of
  them, matches the Moshier recast to 0.01°. Three of them involve a
  side's lagna or outer planets: his Mars opposite her ascendant
  (0.32°), his Pluto conjunct hers (1.69°) and square her Mars (2.03°).
  Leo's two remarks inside the charts hold under his orbs.
- **One engine, by construction.** `aspects` and `synastry` now hand
  their pairs to one function that chooses the orb and measures the
  gap. A chart's table reads each pair once; a synastry reads every
  pair across, with the speeds left at zero since no row reports
  applying.
- **The zodiacs part by the ayanamsha's difference, exactly.** Under a
  sidereal profile the two births, two years apart, carry ayanamshas
  about 100″ apart, and every separation across them moves by that
  difference to 1e-9° when read in each chart's own zodiac. That is
  C241's whole size for this pair; it grows by about 50″ a year of age
  gap.
- **C241 is small for a couple and large across centuries.** Over the
  1485 pairs of the corpus's births, the two readings part on 1.2% of
  the contacts either finds when the births are under ten years apart,
  and on 42.6% when they are a century or more apart; the measured page
  gates that the share grows band by band. A synastry between
  contemporaries barely depends on C241; one between a person and a
  historical chart does.
- **`flatten` hid the field a refusal names.** The record lays the
  aspect table's fields beside the partner, and serde's `flatten`
  buffers whatever it reads, so an unknown zodiac was refused as
  `synastry` rather than `synastry.zodiac`. Both requests are now read
  through flat records of their own, and the ABI test and every
  binding hold the field by name.
- **The lagna beside a luminary.** The first test expected the lagna 9°
  from the other's Sun to hold nothing, reading "a planet's orb" as 8°.
  Leo's rule widens any luminary–planet pair to 10°, so it holds; the
  decision above now says where the lagna stands in the rule rather
  than which number it gets.

## What this does not decide

- **The composite charts.** A midpoint composite (each pair of planets
  read at its midpoint) and a relationship chart founded at the
  midpoint in time and place are later methods with no rank 1 text read
  yet. They get their own page.
- **Weighting.** Leo reads a contact by the planets' natures (Mars on
  Venus, Saturn on Venus) and gives no score. The rows carry the
  distance from exact and the orb, and leave any weighting to the
  reader, as the single chart's table does.
- **The parallels.** They are read on the declinations' page
  ([`western-declinations.md`](western-declinations.md), decision 8): a
  synastry's `parallels` asks for them beside the aspects.
