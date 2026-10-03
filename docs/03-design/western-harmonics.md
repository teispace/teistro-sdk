# A Western chart's harmonics (the `western` module)

Status: `designed`, 2026-10-04, written from the source before any code.

The roadmap's `western` list ends with harmonics, the one item no other
page reaches. A harmonic chart is not a new sky: it is the chart's own
points carried round the circle a whole number of times, so every pair
standing a fraction of the circle apart meets in it. This page decides
what the chart is, which points it carries, what counts as a meeting in
it, and where its houses are.

## What the source decides

The source is John Addey, *Harmonics in Astrology* (1976; second
printing 1977, read on the Internet Archive's open scan,
`addey-1977-harmonics-in-astrology`), the book the technique is named
from. It is rank 1 for this page: there is no older text, and every
later program cites it.

- **The chart is the longitude multiplied.** "Multiplying the absolute
  longitude (i.e. from 0° Aries) of the radical position by the number
  of the harmonic desired", and subtracting "the nearest multiple of
  360" below it (p. 100). His worked figure: a Moon at 23°33′ Scorpio,
  233°33′, in the 5th harmonic is 1167°45′ less 1080°, so 87°45′,
  27°45′ Gemini. His other two ways of reckoning it (tables, and the
  circle drawn in n small zodiacs) are the same map.
- **The angles go with the planets.** "When all the radical positions
  including the Ascendant and M.C. have been recalculated" they are put
  in one map. The harmonic Ascendant "no longer represents an actual
  Ascendant" and the M.C. "can, of course, fall anywhere in the
  circle" (p. 102).
- **Its houses are equal from the harmonic Ascendant**, as he expects
  "most students" to draw it (p. 102), and as his own reading of
  Churchill's 9th harmonic counts them: the Moon and Saturn "in the
  3rd", Pluto in the tenth, Venus rising (pp. 97–98). That is his
  suggestion, not a rule: C254.
- **The 9th harmonic is the navamsa.** Addey builds the chapter on it
  ("as the fruit to the tree", p. 96): the navamsa of a longitude is the
  sign of nine times it, which is the Parashari D9 exactly. The 5th he
  calls the Panchamsa chart. Under a sidereal chart the identity holds
  in the sidereal zodiac; Addey reckons from 0° Aries of the tropical
  one (C253).
- **The orb is one circle's orb in every harmonic.** "The orb must
  diminish in direct proportion to the number of the harmonic" (p. 130):
  an aspect of the n-th harmonic, 360°/n, takes the full circle's orb
  divided by n. Multiplied by n, that is the full circle's orb again,
  so a meeting in any harmonic chart is a conjunction within one orb.
  He offers 12° and 15° for the full circle, and prefers the smaller
  ("it is better to stick to the smaller orbs"): C252.
- **The chart answers for any whole number.** He reads the 5th, 7th and
  9th, speaks of the 120th as an aspect of 3°, and suggests the n-th
  harmonic for the n-th year of life (p. 161), which he calls an
  experiment with "mixed but promising results".

Recast by pyswisseph (Moshier, Placidus for the radix only), Churchill
at Blenheim, 30 November 1874, the time Addey adopts "about 2 minutes
before" 1.30 a.m. local, gives every 9th-harmonic statement on pp. 97–98:
the Moon (♐26.8) on Saturn (♐26.3) in the third equal house from the
harmonic Ascendant (♎4.8), "opposite this conjunction at 26° Gemini";
the harmonic Mercury (♐8.4) on the radical Sun (♐7.7); the harmonic Mars
(♒29.0) opposite the radical Moon (♌29.7); Venus (♎18.3) rising in
Libra; Pluto (♋12.8) in the tenth.

## Decisions

1. **The harmonic is a whole number from 1 to 360.** A fraction is
   refused: n·λ is continuous across 0° Aries only for whole n, so a
   planet at 359.9° and one at 0.1° would part by far more than their
   0.2°. The 1st harmonic is the chart itself, which keeps a loop over
   harmonics free of a special case. 360 is the circle in whole
   degrees, the harmonic whose aspect is 1°; above it a birth time
   known to the minute moves the Ascendant by more than the aspect, so
   the chart would be read off noise. `number` is the request's one
   required field: `{"number": 9}`, refused as `harmonic.number`.
2. **The points are the planets the chart places, the Ascendant and the
   midheaven.** The nodes are left out, as every Western page leaves
   them. A point of the harmonic chart is a `HarmonicPoint`: a graha,
   the Ascendant or the midheaven. The hit list's `NatalPoint` names
   no midheaven, and widening it would move every reader of the hits
   and the synastry for a point only this chart carries.
3. **In the chart's own zodiac (C253).** The positions are multiplied
   as the chart holds them, as the Western houses are read: under the
   tropical profile Addey's chart to the letter, under a sidereal one
   the navamsa's signs in the 9th. A meeting does not depend on the
   zodiac, since both points move by n times the same ayanamsha; only
   the signs and the houses' signs do.
4. **A meeting is a conjunction in the harmonic chart within the orb,
   12° by default (C252)**, at most 30°: two points whose harmonic
   longitudes stand within it. Each row names the two in a fixed order
   (the grahas by id, then the Ascendant, then the midheaven), the arc
   between them in the harmonic chart, and `aspect`, the whole number k
   for which the radical arc is nearest k·360°/n, so a row says it is
   the square (k = 1 of the 4th) rather than leaving the reader to work
   it out. The rows run closest first. The pair of the Ascendant and
   midheaven is a row like any other, as Addey's "72° or 144° apart,
   they will, of course, be conjunct" says it is.
5. **Each point's equal house from the harmonic Ascendant (C254)**, 1 to
   12, the Ascendant itself in the first. The answer carries it beside
   each point rather than leaving it to the caller, because the
   reading counts houses (pp. 97–98).
6. **One harmonic a request, every chart of a batch.** A batch of charts
   at one harmonic is the boundary's shape; many harmonics of one chart
   is a loop over a cheap kernel (multiplication), which every binding
   writes as a loop over `found` or the Rust caller over
   `harmonic(chart, request)`. If a consumer asks for many at once, a
   list on the request is an addition, not a change.

## Order of work

1. This page, and C252 to C254 in the cruxes register.
2. `western::harmonic`: the kernel, pure on longitudes, its tests
   Addey's p. 100 arithmetic, Churchill's recast and the navamsa
   identity.
3. `sdk.chart().harmonic(chart, &HarmonicRequest)`, held to Churchill's
   chart on the built-in ephemeris, and every Parashari navamsa of the
   corpus's births read back as the 9th harmonic's sign under the
   default sidereal profile.
4. The boundary and the bindings, with parity.
5. A measured page if building it finds something to measure.

## What this does not decide

- **The age harmonic.** Addey's "2nd harmonic chart to the second year
  of life" and "the 40th to the fortieth year" (p. 161) would read the
  n-th harmonic through the year that ends at age n, where his
  recurrences "at ages which are multiples of the age at which they
  first occurred" read it at age n. A caller asks for the harmonic it
  means; the SDK does not name either.
- **Contacts between the harmonic chart and the radix**, which Addey
  reads (the harmonic Mercury on the radical Sun). Both sets of
  positions are in the answer and the chart's own; a cross reading
  would be the synastry's engine over two point sets, and waits for a
  consumer who asks.
- **Harmonic analysis of many charts**, the wave amplitudes Addey
  computes over collections (chs. 2–9). That is research over a
  corpus, not a chart's answer.
