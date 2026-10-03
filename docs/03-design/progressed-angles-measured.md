# The progressed angles, measured

Status: `generated` by `cargo xtask progressed-angles` from the corpus's
recorded births, 2026-10-03. Do not edit: `check-progressed-angles`
regenerates this page and fails on any difference.

C237 (`western-progressions.md`) is how the progressed midheaven moves:
Leo's own map advances it by the mean Sun in right ascension, and the
twentieth century's habit is the solar arc. No text weighs one against
the other, so every method ships as a knob. This page states the
choice's size. It reads 54 of the corpus's births under
`western-tropical-default` through `ChartArea::progressed` at the
secondary rate and the tropical year, under each `AngleMethod`, at 8
ages from 10 to 80 years. c048-kathmandu-2399-12-30 is left out, as its
progressed sky runs past the built-in ephemeris's years.

| proposed rule | verdict | measured |
|---|---|---|
| every method returns the radical midheaven and ascendant for no sky at all | **holds** | 0 of 54 disagree |
| the solar arc and Naibod's along the ecliptic part by no more than twice the equation of the centre (3.83°) | **holds** | 0 of 432 disagree |
| the solar arc and Leo's mean Sun in right ascension part by no more than the equation of time's range (7.65°) | **holds** | 0 of 432 disagree |
| the quotidian meridian is Leo's at every birthday, within 0.05° | **holds** | 0 of 432 disagree |
| the quotidian meridian stands half the circle from Leo's half a year after a birthday, within 1° | **holds** | 0 of 432 disagree; a day of sky turns it a full circle, so between birthdays it reads any sign |
| under the conformance profile's NEAREST_EVENT polar day, a progressed chart is founded wherever the birth was | falsified | 5 of 432 refused, at 2 births; a polar day's nearest event can lie weeks from the sky a later age reads; CIVIL_MIDNIGHT holds them all |

## The midheaven, against Leo's map

Each cell is the median and the worst distance, over the births, between
a method's progressed midheaven and Leo's.

| age | `NaibodLongitude` | `SolarArcLongitude` | `SolarArcRightAscension` |
|---|---|---|---|
| 10 | 0.69° / 0.88° | 0.57° / 1.19° | 0.58° / 1.23° |
| 20 | 1.27° / 1.73° | 1.19° / 2.36° | 1.11° / 2.48° |
| 30 | 1.60° / 2.52° | 1.63° / 3.46° | 1.68° / 3.58° |
| 40 | 2.04° / 3.24° | 2.02° / 4.48° | 2.44° / 4.43° |
| 50 | 2.53° / 3.83° | 2.41° / 5.33° | 2.68° / 4.98° |
| 60 | 3.04° / 4.32° | 2.70° / 5.91° | 2.50° / 5.45° |
| 70 | 3.25° / 4.57° | 2.62° / 6.56° | 2.53° / 6.62° |
| 80 | 2.91° / 4.87° | 2.64° / 7.09° | 2.38° / 7.54° |

## The ascendant, against Leo's map

The ascendant is the one the turned meridian rises with at the
birthplace, so a small gap in the meridian is a larger one in the
ascendant far from the equator.

| age | `NaibodLongitude` | `SolarArcLongitude` | `SolarArcRightAscension` |
|---|---|---|---|
| 10 | 0.61° / 7.73° | 0.55° / 4.38° | 0.57° / 4.59° |
| 20 | 1.09° / 10.97° | 1.18° / 6.84° | 1.13° / 6.49° |
| 30 | 1.55° / 4.73° | 1.73° / 5.14° | 1.54° / 3.88° |
| 40 | 1.74° / 4.64° | 2.07° / 5.71° | 2.17° / 5.26° |
| 50 | 2.09° / 4.60° | 2.41° / 5.71° | 2.43° / 6.43° |
| 60 | 2.39° / 5.41° | 2.46° / 6.04° | 2.34° / 7.19° |
| 70 | 2.71° / 5.82° | 2.69° / 6.96° | 2.34° / 7.42° |
| 80 | 2.73° / 5.83° | 2.49° / 7.65° | 1.99° / 7.08° |

## What it means

The first row holds that the methods agree where they must. The next two
hold the bounds the arithmetic gives: a true arc and a mean one part by
the change in the equation of the centre along the ecliptic, and by the
change in the equation of time along the equator, whatever the age. The
next two say what the quotidian is: Leo's meridian at each birthday,
because the sidereal day's excess is what both add, and any meridian at
all between birthdays, so the tables leave it out. For the other three
the tables are C237's size: up to 80 a method's midheaven stands at most
7.54° from Leo's, and its ascendant at most 10.97°.
