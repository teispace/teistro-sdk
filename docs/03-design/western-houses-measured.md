# A Western chart's houses, measured

Status: `generated` by `cargo xtask western-houses` from the corpus's
recorded births, 2026-10-03. Do not edit: `check-western-houses`
regenerates this page and fails on any difference.

Each of the corpus's 53 births Placidus is defined at is founded under
`conformance-baseline` with the outer planets and its 530 planets
counted in Placidus, the division Leo's figures are cast in
(`western-houses.md`, C249), and again in six other divisions and in the
tropical zodiac.

| proposed rule | verdict | measured |
|---|---|---|
| a planet the sidereal and the tropical founding count in different houses stands 0.01° or more apart in them: the zodiac moves no house | **holds** | 0 of 1 disagree |
| Placidus is defined at every recorded birthplace | falsified | 2 of 55 disagree |
| the degree that rose an hour before the birth stands in the twelfth house under Placidus | falsified | 2 of 53 disagree |

Placidus divides the semi-arcs, which do not exist inside the polar
circles, and the conformance profile refuses rather than falls back, so
these births are left out of everything below: c028-troms-1988-06-21,
c029-troms-1988-12-21. A profile that falls back, as
`western-tropical-default` does to Porphyry, reads them, and the answer
names the division used.

## The zodiac and the profile

Each birth is founded again under `western-tropical-default`, and its
houses read in that zodiac. The two profiles differ in more than the
zodiac, so a planet they place differently can change house where it
stands near a cusp; the zodiac alone turns every cusp and every planet
together and moves none.

| birth | planet | apart in the two foundings |
|---|---|---|
| c015-kabul-1999-09-09 | moon | 0.557° |

## The division a planet is counted in

How many planets each division counts in another house than Placidus,
over every birth. This is the size of C249: a reader who takes Lilly's
Regiomontanus, or a modern default, for Leo's houses moves this many
planets.

| division | planets | in another house | share | births with one moved | share |
|---|---|---|---|---|---|
| regiomontanus | 530 | 33 | 6.2% | 22 | 41.5% |
| koch | 530 | 40 | 7.5% | 21 | 39.6% |
| campanus | 530 | 61 | 11.5% | 30 | 56.6% |
| porphyry | 530 | 43 | 8.1% | 22 | 41.5% |
| equal | 530 | 131 | 24.7% | 39 | 73.6% |
| whole sign | 530 | 266 | 50.2% | 52 | 98.1% |

## The ascendant's reach

Leo counts with the ascendant everything up to the degree that rose one
sidereal hour before the birth (C250). Of the 30 planets in the twelfth
house, 17 (56.7%) stand within that reach; 48 more stand in the first
house itself. The hour is 15° of right ascension, and the table gives
how much longitude it raised, which Leo calls half a sign "in a great
many cases". It is widest where signs rise quickly, so it depends on the
sign rising as much as on the latitude.

| latitude | births | median | narrowest | widest |
|---|---|---|---|---|
| 0° to 20° | 10 | 16.0° | 14.2° | 18.0° |
| 20° to 40° | 30 | 15.4° | 12.2° | 22.3° |
| 40° to 55° | 11 | 12.2° | 10.9° | 34.9° |
| 55° to 90° | 2 | 16.2° | 15.9° | 16.4° |

Leo takes the degree to "be found in the twelfth house, of course" (p.
90). It is not at 2 of the 53 births: where the signs about the
ascendant rise quickly, Placidus's twelfth house is narrower than the
hour, and the reach runs past its cusp into the eleventh. The flag still
follows the degree, so a planet of the eleventh within it is read with
the ascendant.
