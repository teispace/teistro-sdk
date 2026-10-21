# The committee's star planets

Status: `built`, 2026-10-22. §1 is measured and §3 built, both gated by
`crates/sdk/tests/printed_day.rs`.

Nepal's national panchanga committee prints, on every half-month page of
its *Rashtriya Panchangam*, a table of the nine grahas at each morning's
sunrise at Kathmandu (उदयकालिक सूर्यादि स्पष्ट ग्रह). Its Sun and Moon are
the Surya Siddhanta's with the committee's bija (C28). Its star planets
and node were read on four rows as modern positions in Lahiri's zodiac
(C38, `calendars/bikram-sambat.md`). Until version 4 `nepali-committee`
answered them from the text, so a chart under `nepal_official` placed
Venus up to 23 degrees from the planet the committee prints for that
morning.

## 1. Measured

The first row of every half-month page of VS 2082 and 2083, 49 mornings
from April 2025 to March 2027, was read off the page images at 250 dpi.
Each row was dated by its node, which stands a steady 1′ ahead of the
modern mean node, so a day's error shows as 3′. The root mean square
miss, in arcminutes:

| reading | Mars | Mercury | Jupiter | Venus | Saturn |
|---|--:|--:|--:|--:|--:|
| a modern sky, Lahiri's zodiac (built-in; pyswisseph agrees to 0.1′) | 18 | 33 | 9 | 31 | 9 |
| the Surya Siddhanta under `nepal_official` | 94 | 157 | 260 | 386 | 459 |
| the text with the best whole bija for that planet (±3000 revolutions an age) | 32 | 113 | 20 | 52 | 20 |
| modern planets seen from an Earth placed by the printed Sun | 20 | 39 | 9 | 51 | 10 |

Saturn runs a steady 8′ behind the modern place, Jupiter within 22′,
and Mars, Venus and Mercury part most when they pass nearest the Earth
(Venus by 74′ at its September 2026 approach, Mercury by up to 94′ near
its stations). An error that grows as the planet nears the Earth is a
heliocentric one seen close up. So the committee computes a modern
heliocentric sky to a few arcminutes, the precision of a fitted series
rather than an ephemeris. Its method is not named on the page and is
not identified here (C38 stays open on the method).

Two readings are refuted:

- **A bija on the text.** A whole bija per planet is a constant offset in
  that planet's mean motion over two years. The best one still misses by
  twice the modern sky's error, and Venus's mean miss of 33′ is one no
  bija on its conjunction can move, since its mean place is the Sun's.
- **A modern sky seen from the text's Sun.** Placing the Earth by the
  printed Sun makes Venus worse (51′ against 31′) and changes nothing
  else.

## 2. What the measurement settles

The committee's star planets are a modern sky in Lahiri's zodiac to
within its own arcminutes, and the text's are not its at all. A profile
named for the committee should give the reader the planets it prints.
The same was true of its lagna table (C680), and
`panchanga.lagna_table` answered it for one quantity. The planets are
the chart's, so the answer belongs in the sky the profile opens.

## 3. Built: the star planets from the modern entry

`Siddhanta::Surya` gains `planets`, beside `sunrise`:

- `TEXT` (the default): every graha from the text, as now.
- `MODERN { ayanamsha }`: Mars, Mercury, Jupiter, Venus and Saturn, and
  the node as the mean node, from the modern entry the chain's modern
  sunrise already comes from, in the named zodiac. The Sun and the Moon
  stay the text's, and so do the angles, the obliquity and the
  ayanamsha the text defines.

`nepali-committee` sets `MODERN { LAHIRI }`, citing this page, and is
version 4. The composition is `teistro_siddhanta::ModernParts`, which
also carries the modern sunrise (C39), so one modern entry answers
both. A sidereal place answered this way is the modern place less Lahiri's
ayanamsha, so a chart reads the committee's printed planet; a tropical
one adds the text's ayanamsha back, so the two frames stay one sky.

Acceptance, held: under `nepal_official` each of the five reads as the
modern row of §1's table (within half an arcminute of the default
context's), and the node is within 2′ of the print on all 49 rows.
Every chart under the profile changes its five planets and node, which
is the documented, gated reason the brief's §4 asks for.
