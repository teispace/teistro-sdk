# A body's returns: the lunar return (the `western` module)

Status: `built`, 2026-10-04, written from the source before any code.
`sdk.chart().returns` answers each graha's returns. `HitRequest::returns`
is the same question as a hit list, and every binding builds it with its
own constructor: `returnsRequest` in Node and wasm, `returns_request` in
Python, `HitRequest.returns` in Dart.

The roadmap's `western` list names the solar and the lunar return. The
solar return is built: it is the Tajika pravesha's `Tropical` reading
(`annual-chart.md`). This page decides the lunar return, and with it the
return of any body, since the doctrine does not change with the body.

A return needs no new astronomy. It is the instant a body's longitude
comes back to where it stood at birth, which the transit hit list
already finds as a conjunction of the body with its own natal place
(`transit-hit-list.md`). So the work is in deciding what the source
means by "its place", and in giving the question a name of its own.

## What the source decides

The source is Jean-Baptiste Morin, *Astrologia Gallica* (The Hague,
1661), Book XXIII, *De revolutionibus*, read on the Internet Archive's
open scan of the first edition (`bub_gb_auvbdtmpi0IC`, printed pp.
633–635 = leaves 706–708). It is rank 1: the book that set out the
revolution figure as later astrology uses it, with worked figures.

- **Two monthly returns, and he reads one** (ch. VIII, p. 633). The
  synodic, the Moon's conjunction with the Sun, is the same for everyone.
  The other is "reditus ☽ in illud Eclipticae punctum, quod ipsa occupavit
  tempore ortus": the Moon's return to the point of the ecliptic she held
  at birth. That one is the native's own.
- **Cardan's return is refused** (p. 633). Cardan counted the Moon's
  return to her place in the year's solar revolution. Morin rejects it:
  the radix is the native's, the year's figure is not.
- **Longitude only** (ch. IX, p. 634): the return is in the ecliptic,
  "non habita ratione ejus latitudinis".
- **The radix must be accurate** (p. 634). A degree of error in the
  radical Moon moves the return about two hours.
- **The time is corrected twice**, as in his chapter V. That is how a
  table's mean motion is brought to the true motion by hand. A root
  search on the true longitude does the same thing in one step.
- **The figure is erected where the native is** (p. 634), "ad locum in
  quo est natus initio revolutionis", as for his solar revolution.
- **The worked figures** (ch. X, p. 635). Gustavus Adolphus's revolution
  "prope Norimbergam" before his death, and Richelieu's, "Parisiis",
  before his. Both are dated in astronomical time from noon, *T.A.*
  (tempus apparens), and print the RAMC above the figure.

## The acceptance test

Richelieu's figure is printed for 29 November 1642, 9h 58m from noon
apparent time at Paris, with RAMC 35°28′. It shows the Moon at 18°49′
Pisces and Saturn at 18°17′ Pisces. Recast by pyswisseph (Moshier):

- Apparent 21h 58m, less the equation of time's 10.8 minutes, is 21h
  47m local mean time, or 21h 38m UT (JD 2321121.4012). There the recast
  RAMC is 35°35′ against his 35°28′. The twelve Regiomontanus cusps for
  Paris are his to the degree: Leo 20 rising, Taurus 8 culminating,
  Virgo 11, Libra 5, Scorpio 8, Sagittarius 20 and Capricorn 26, with
  their opposites.
- The recast Saturn stands at 18°18′ Pisces, against his 18°17′.
- The recast Moon stands at 19°09′ Pisces at his instant, 20′ past his
  printed place. She reaches 18°49′ at 21h 03m UT (JD 2321121.3768),
  35 minutes before his figure. A third of a degree is the size of a
  17th-century table's lunar error, and his own p. 634 warns of exactly
  that: a degree of the Moon is two hours of return.

So the figure is a return to the Moon's **tropical** radical longitude,
measured on the **geocentric** Moon (his tables have no parallax), and
erected for the place given. The built-in ephemeris begins in 1800, so
the SDK cannot found 1642 itself. Its test holds the definition the
recast confirmed, on the recast's own engine:

- a radix at Paris on 1 January 2000, 12h UT, whose three lunar returns
  are pyswisseph's (Moshier) to within ten seconds;
- every return of a year back on the radical longitude to the arcsecond,
  read from a chart founded at it.

Gustavus Adolphus's figure (24 October 1632, 0h 4m T.A., latitude 50°)
prints the Moon at 20°16′ Pisces and the Sun at 1°26′ Scorpio. That
agrees with the recast radix (Stockholm, 19 December 1594 N.S., in the
morning) and with a return about an hour before his time. His printed
RAMC, 287°27′, fits neither his time nor his own cusps, so the figure
holds a slip. It is not a test, and it is named here so that nobody
takes it for one.

## What is decided

- **A return is a conjunction with the natal place, at 0°** (C255). The
  target is the body's own radical longitude, never Cardan's place in
  the year's figure.
- **In the chart's own zodiac** (C256). Morin's is tropical, and a
  Western chart is founded tropical. A sidereal chart returns to its
  sidereal place, as the Tajika pravesha's own reading does. The two
  differ by the precession since birth, about 50″ a year. That moves a
  lunar return by about an hour at forty years, and a solar return by
  fourteen. A caller wanting the other founds the radix in it.
- **On the chart's own centre** (C257). Morin's Moon is geocentric. A
  topocentric frame moves the Moon by up to a degree, which is about two
  hours of return. The SDK reads the frame the context's settings name,
  as the hit list does, so a geocentric Western profile gets Morin's
  return.
- **Any body, the Moon by default.** The rule does not name a body: the
  solar return is the same search on the Sun, and a "Saturn return" on
  Saturn. A body that turns retrograde crosses its place up to three
  times, and each crossing is a row carrying its `motion`. Nothing is
  thrown away as the "wrong" crossing.
- **The answer is instants.** The figure is a chart at the instant, for
  the place the native is at (C258). Every binding already founds a
  chart at an instant and a place, so the return does not found one
  itself. The doctest shows the founding, at the birth place and at
  another.

## The surface

The question already crosses the boundary: a return is a hit list asked
for one graha, its aspect to its own natal place, at 0°. So the return
adds a name, not a second search or a second set of sections.

- `HitRequest::returns(graha, from, to)` is that hit list. It asks for
  that graha only, `ASPECT` only, its own natal place only, at the
  angle 0.
- `sdk.chart().returns(&birth, from, to, grahas)` asks each graha's
  return in one search. It answers `Vec<BodyReturn { graha, at, motion
  }>` in instant order. Several grahas are searched together, as the hit
  list does, and a crossing of one graha over another's natal place is
  not a return, so it is dropped.
- Every binding gets a constructor for the same record: Node's
  `returnsRequest(from, to, graha?)`, Python's `returns_request(...)`
  and Dart's `HitRequest.returns(...)`, the Moon by default. Its answer
  is the hit list each binding already reads, with parity. Each doc
  example founds the return chart at the instant, at the birth place and
  at another (C258).

## Read back elsewhere

The solar return is already built, so the Sun's row answers a cross-check
for free. On a sidereal chart, `returns` for the Sun must give the Tajika
pravesha's `Sidereal` instant, and on a tropical chart its `Tropical`
instant, each to the second. Two searches, written for two traditions,
must find one instant.

## What building it found

- **The question was already answerable.** The hit list crossed every
  binding before this page, and a return is one of its questions. So the
  return added a constructor in each binding and no ABI section. Its
  search sits under the hit list's parity, and each binding's test
  checks the constructor's record and a cast with it.
- **The built-in ephemeris begins in 1800**, after Morin's figures. The
  page's recast holds his definition. The SDK's test holds the SDK to
  the recast's engine at a date both reach: three lunar returns within
  ten seconds of Moshier's.
- **Two searches, one instant.** On a sidereal chart the Sun's return is
  the Tajika pravesha's `Sidereal` instant, and on a tropical chart its
  `Tropical` one, under a second over three years each. C46's questions,
  whether a return chart is precession-corrected and whether it is
  relocated, are C256 and C258: the chart's own zodiac, and the caller's
  place.

## The half- and quarter-returns

Built 2026-10-17. The demi-solar return is the instant the
Sun stands 180° from its natal place, half a year after the birthday.
The quarti-solar returns are the three instants it stands 90°, 180° and
270° past it, so with the return they divide the solar year into four.
Each is read as a figure for its quarter of the year.

### What the sources say

- **Morin does not have them.** Book XXIII sets out the solar and the
  lunar revolution and nothing between them. A search of the open
  scan's OCR text of the book, for *quadra-*, *dimidi-* and *opposit-*,
  finds only the aspects inside his figures and two other uses of
  *quadrante*. One is Stadius's complaint, which Morin reports: the
  tables erred in the Sun's place by 20 or 30 minutes, so the figures
  were erected "plusquam Coeli quadrante" wrong (OCR text, long s
  normalised). The other is how far a native may travel from his
  birthplace. That is a search of the OCR
  text, not a page-by-page reading of the images, so it is recorded as
  that.
- **No earlier text is known to this page.** The half- and
  quarter-returns are twentieth-century practice. They are usually
  credited to the sidereal school of Cyril Fagan and Donald Bradley,
  which read solar and lunar returns in the sidereal zodiac. This page
  has not read a printed definition, so it cites none, and the crux
  (C433) records the practice as rank 3.

So the page has to decide what the practice leaves open, and it does
that from the return's own rule. A partial return is the return with its
line moved, not a new kind of figure.

### What is decided

- **A body's own longitude at natal + offset** (C433). The demi-solar
  return is the Sun's crossing of the line 180° past its radical place,
  found by the same search as the return. It is not the hit list's
  aspect of the transiting Sun to its natal place. The two give the same
  instant at 180°, but the aspect is a different question.
- **The offset is directed, forward in the zodiac** (C434), from 0 to
  359. A quarter ahead of the natal place, about three months after the
  birthday, is 90. A quarter behind it, about nine months after, is 270.
  The hit list's aspect angle runs from 0 to 180 and means both sides,
  so its 90 is both squares. A quarter-return has to tell them apart, as
  the phase return does (C420).
- **Any body, as the return takes any body** (C435). The practice's
  names are the Sun's, but the rule names no body, any more than the
  return's does. So the Moon's half- and quarter-returns are the same
  search over a month, and a planet's over its own period. A retrograde
  planet crosses a line up to three times, each a row with its `motion`.
- **Whole degrees from 0 to 359, any of them** (C436). The quarters are
  constants (`OffsetReturn::QUARTERS`, `OffsetReturn::HALF`), but the
  request takes any whole degree, so an octile-return at 45° is the same
  call. 360 or more is refused on `offsets`, and the hint names the same
  line less the circle. Offset 0 is the return.
- **Read as the return is read** (C437): longitude only, in the chart's
  own zodiac, on the centre the settings name, and erected at the
  caller's place (C255 to C258). A sidereal chart's quarti-solar returns
  are the sidereal quarters, which is the zodiac the practice used.

### The surface

- `sdk.chart().returns_at(&birth, from, to, grahas, offsets)` answers
  `Vec<OffsetReturn { graha, offset_deg, at, motion }>`, ordered by
  instant, then graha, then offset. Each graha and offset is its own
  lattice, a single line one circle wide, scanned by the hit list's own
  search (`Founder::transit_events`) in one pass over the window.
- `sdk.chart().returns` and its `BodyReturn` are unchanged, and so are
  the hit list and every binding's `returnsRequest`. Offset 0 of
  `returns_at` gives the same instants as `returns`, bit for bit.
- **Bindings.** A binding asks for a half-return today as a hit list:
  `{ ...returnsRequest(from, to, ['SUN']), aspects: [180] }` in Node, and
  the same record in the others. The opposition has one side,
  so its row is the demi-solar return. A quarter-return asked as
  `aspects: [90]` gives both squares with `angle: 90`. The side is the
  chart founded at the row's instant, which the binding founds anyway to
  erect the figure (C258). Carrying `returns_at` across the boundary as
  its own section is left until a binding needs the side answered
  without founding a chart. That is a follow-up, and the founded chart
  already gives the answer.

### The tests

`crates/sdk/tests/returns.rs`, on the Paris radix of 1 January 2000,
12h UT, under the tropical Western profile:

- the Sun's returns at 0°, 90°, 180° and 270° over a year come in the
  order 90, 180, 270, 0. Each one, read from a chart founded at its
  instant, puts the Sun at natal + offset to within an arcsecond. Each
  falls 88 to 95 days after the last, the summer quarters the longest.
  The demi-solar asked alone is the 180° row;
- the Moon's quarters over a sidereal month come in the same order, 5.5
  to 8.5 days apart, each read back to the arcsecond;
- offset 0 gives `returns`' instants bit for bit;
- 360, no offset, or an offset named twice is refused on `offsets`, with
  a hint naming the line it meant. The window and the grahas are refused
  as the return's are.
