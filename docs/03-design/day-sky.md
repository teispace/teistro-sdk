# The day's sky

SR-63 (`consumers/addendum-3.md` §11). What a Today screen draws: the paths of
the Sun and the Moon over the day at the place, their rises, transits and sets
on those paths, and the Moon's lit shape as it stands in that sky. Asked by a
flag beside `lagnas` and `eclipses`; a request that does not ask pays nothing.

Built in `crates/astro/src/day_sky.rs`, with the angles in
`crates/astro/src/phenomena.rs`. A request names it with
`AlmanacRequest::with_sky`, and at the C boundary with `TS_PANCHANGA_SKY` and the
request's `sky_step_minutes`.

## 1. The answer

One `DaySky` a day, in the days' order, in the `sky` section:

| Field | Meaning |
|---|---|
| `from`, `to` | the window: the day's window, or see §5 |
| `stepMinutes` | the step asked |
| `sun`, `moon` | a `BodySky` each |

A `BodySky` is its `state` and its `track`. The track is one list in time
order: a `SAMPLE` point every step from `from`, one at `to`, and a `RISE`,
`TRANSIT` or `SET` point at each exact event, so a polyline and its marks come
from one list. An event belongs to a window when `from <= at < to`, so that
neighbouring days share none.

Each point has `at`, `kind`, `altitudeDeg`, `refractedAltitudeDeg`,
`azimuthDeg`, and on the Moon's track `moon`, its shape at that instant:
`illuminatedFraction`, `phaseAngleDeg`, `waxing`, `brightLimbDeg` and
`parallacticAngleDeg`. To draw the disc as it stands in the sky, turn the
bright limb from the zenith by `brightLimbDeg - parallacticAngleDeg`,
measured from the top of the disc through the left, which is Meeus's `χ − q`.

`at` is UT1 read as UTC, as every other instant of a day is.

## 2. Formulas and their sources

Every angle is degrees. α, δ are the apparent right ascension and declination of
date, H the local hour angle, φ the geodetic latitude.

| Quantity | Formula | Source |
|---|---|---|
| Topocentric place | `ρ' = ρ − s`, where `s` is the station's geocentric vector in the true equator of date, on the WGS 84 ellipsoid (`sky::observer`) | Meeus, *Astronomical Algorithms* 2nd ed., ch. 40 in vector form; SOFA `pvtob` |
| Altitude | `sin h = sin φ sin δ' + cos φ cos δ' cos H'` | Meeus ch. 13, eq. 13.6 |
| Azimuth, from north through east | `A = atan2(−cos δ' sin H', cos φ sin δ' − sin φ cos δ' cos H')` | Meeus eq. 13.5 turned to the north origin |
| Refraction | `R = 1.02′ / tan(h + 10.3 / (h + 5.11))` | Sæmundsson 1986, Meeus eq. 16.4 |
| Phase angle | the angle at the Moon between the observer and the Sun, from the topocentric vectors; `k = (1 + cos i) / 2` | Meeus ch. 48 in vector form; the Moon's `Phase` |
| Bright limb, χ | `atan2(cos δ☉ sin(α☉ − α), sin δ☉ cos δ − cos δ☉ sin δ cos(α☉ − α))`, from the north point of the disc through the east | Meeus eq. 48.5 |
| Parallactic angle, q | `atan2(cos φ sin H, sin φ cos δ − cos φ sin δ cos H)`, the east-of-north position angle of the zenith | Meeus eq. 14.1 with `tan φ` multiplied through, so a pole needs no case |

**A worked example holds each.** Meeus's example 48.a (the Moon of 1992 April 12)
gives `k = 0.6786` and `χ = 285.0°`, example 13.b's altitude and azimuth hold
the altitude's arithmetic, and the page's tests take their numbers from those.

## 3. Conventions (cruxes C970 to C973)

**C970: a transit is the hour angle's zero.** The body crosses the meridian;
its greatest altitude can fall a few seconds or, for the Moon at high latitude,
minutes off that. The almanacs print the meridian passage, so the point is the
meridian passage, and `Solver`'s existing search finds it. The geocentric search
is the topocentric answer: the station stands in the meridian plane at `H = 0`,
so parallax has no component in right ascension there. Horizons agrees to 0.15 s
for both bodies, with the Moon 85° up at Kathmandu on 2026-10-31 (§6).

**C971: the phase is seen from the place.** The fraction and both angles use the
topocentric Moon. At Kathmandu the parallax moves `k` by up to 0.008, which is
visible in a crescent's width.

**C972: `waxing` is geocentric.** It is the sign of the Moon's ecliptic
longitude less the Sun's, as the Earth's centre has it, so it changes at the
moment of new or full Moon, once a lunation, and agrees with the day's paksha. A
topocentric sign would flip hours off that at different places. The shape to
draw is `brightLimbDeg`, which is topocentric.

**C973: below the horizon nothing is refracted.** Sæmundsson's fit is for a body
seen in the air, and it turns over near −3°. `refractedAltitudeDeg` equals
`altitudeDeg` wherever the refracted value would stay below the horizon, which
is what Swiss-Ephemeris-style engines do. JPL Horizons instead keeps adding the
horizon's value below it. Neither is a measurement of anything; only the part
above the horizon carries meaning.

## 4. Rises and sets

Each window's events come from `rise_set::Solver` under the day's own horizon
convention (`day.sunrise`), the one the day's sunrise and moonrises use, so a
day's `moon.rises` and the Moon's `RISE` points are the same events (§7 gives
the agreement). The point's airless altitude shows what the convention means:
−0.57° under `nepali-default`'s centre with refraction, −0.83° under an
upper-limb convention.

## 5. Polar days, and the window

No rise or set is an answer. `state` is:

| `state` | Meaning |
|---|---|
| `CROSSES` | the window holds a rise or a set |
| `ALWAYS_ABOVE` | it holds neither, and the body is above the convention's altitude |
| `ALWAYS_BELOW` | it holds neither, and the body is below it |

The Sun still transits on a polar day and the point carries its altitude, so a
midnight Sun's path and its highest mark are both there. The transit is a list
for the same reason a moonrise is: a window of 24 hours can hold no moon
transit, about once a lunation, and sometimes two.

**The window.** It is the day's `window`. On a polar day that window is the
policy's: under `NEAREST_EVENT` it is the nearest day with a sunrise, five weeks
from midsummer at Tromsø, so it says nothing of the date asked
(`settings-and-profiles.md`, the polar note). A day whose state is polar takes
the civil day instead, midnight to midnight on the request's offset, and the
section's provenance names the rule under the convention `sky.window`.

## 6. Reference

The table in `crates/sdk/tests/day_sky.rs` was recorded from JPL Horizons on
2026-10-31: the Sun and the Moon at Kathmandu, Sydney, Tromsø and Quito, on
three dates, five instants each, with `QUANTITIES='4,10,24,27,42'`. Horizons
reads DE441 and is independent of the SDK's built-in ephemeris and of Teimeris.
`PsAng` is the position angle of the extended Sun-to-target radius, so the
bright limb is `PsAng + 180°`.

`PsAng` is measured from the ICRF's north and the bright limb from the true
equator's, so the test turns it by the pole's 20.04″ a year (Meeus ch. 21).

Worst of the 120 rows, held by the test:

| Quantity | Worst difference |
|---|---|
| Altitude, airless | 0.0003° |
| Azimuth, in the sky's own arc | 0.0004° |
| Refracted altitude, above 2° | 0.0004° |
| Fraction lit | 0.00002 |
| Phase angle | 0.0003° |
| Bright limb, phase 20° to 160° | 0.004° |
| Parallactic angle, below 80°, from Horizons' hour angle | 0.0008° |

## 7. Cost and agreement

The ephemeris is read only at Chebyshev nodes. Both bodies' α, δ and distance
are smooth over days, so a fit through nodes across the window and past it
answers every point and every search of the window, and the step does not change
what the ephemeris is asked. `day_sky.rs` names the node count, and its test
holds the fit against the source. A search that leaves the fit reads the source
itself. There is no extrapolation.

The flag is one bool on the off path: the `sky` section is `None`, the days are
untouched, and their hashes are unchanged, since nothing is added to `Panchanga`.

**The price when asked.** 22 readings of the ephemeris a day whatever the step, a
test holding that. Measured in release over 31 days at Kathmandu on the built-in
ephemeris at a ten-minute step (`price` in the test file, best of five): 24 ms
without the flag and 67 ms with it, 1.4 ms more a day. That is +177%, so the 3%
bound on a day's path is met only by the path that does not ask: the flag adds a
step's worth of readings, not a percentage of a day, and a track of 145 points
in each of two bodies has that cost. The nutation is read once a window, which
took 0.3 ms a day off it. A longer step costs less. `cargo xtask bench` runs on
CI and has not priced this.

Over October 2026 at Kathmandu a day's rises and sets on the fit lie within
0.0001 s of the day's own, held by `crates/sdk/tests/day_sky.rs` under 0.01 s.
