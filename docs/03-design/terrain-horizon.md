# The sunrise behind the terrain (SR-91)

Status: **built**, 2026-11-12. Tithipatra's addendum 7 asks: accept a horizon
profile, the terrain's angle by azimuth, for a visible rise and set beside the
astronomical one; a rise with a 2° horizon at its azimuth comes later by the
measured amount. It pairs with teigeodata's TG-09, which may produce such
profiles from an elevation model. This page records what the sources do, what
was decided, and what was measured on the built thing.

## 1. What the sources do

**The almanacs reckon a level horizon.** The US Naval Observatory's
definitions of rise and set ("Rise, Set, and Twilight Definitions", Astronomical
Applications Department) take the upper limb on the horizon under average
air, at a place in a level region of the Earth's surface; terrain and an
observer's height are outside the definition. The *Astronomical Almanac* (section A)
and Meeus (*Astronomical Algorithms*, 2nd ed., ch. 15) do the same: the event is
the centre at `h0 = −0°34′ − semidiameter` (`−0°50′` for the Sun), with the
34′ an average refraction at the horizon. The *Explanatory Supplement to the
Astronomical Almanac* (3rd ed., 2013, ch. 7) adds that the horizontal
refraction varies by many arcminutes with the air, which is why
`horizon-atmosphere.md` lets a convention name its air.

**A visible rise is a crossing of the skyline.** Where the terrain is wanted it
is described as a horizon profile, the skyline's elevation by azimuth, and the
body's apparent altitude is compared with it at the body's own azimuth:

- the European Commission's PVGIS takes a user horizon as heights at azimuths
  from north, read between the samples, and shades the Sun by it;
- Stellarium's polygonal landscapes are lists of (azimuth, altitude) pairs,
  joined by straight segments and closed through north;
- HeyWhatsThat and PeakFinder publish such profiles computed from an elevation
  model, with the Earth's curve and terrestrial refraction applied;
- the "Chai Tables" of visible sunrise for Jewish practice (netz hachama) time
  the first ray over the computed skyline of each place, the case the
  consumer's request describes.

**Refraction above the horizon is Bennett's.** Meeus's eq. 16.4 (G. G. Bennett,
*Journal of Navigation* 35, 1982) gives the refraction at an apparent
altitude `h`, `R = cot(h + 7.31° / (h + 4.4))` arcminutes, fitted to apparent
altitudes from 0° to 90° and good to 0.07′ there, at 1010 hPa and 10 °C; it is
the formula `horizon-atmosphere.md` already reads at `h = 0`. At 2° it is
18.2′, against 34.5′ at the horizon: a skyline two degrees up lifts the Sun by
about half as much as the level horizon does. Below the horizon the formula is
not fitted, and turns over: its argument is least at −1.7°, where it reads
57′, and below that it falls again.

## 2. The profile

A `HorizonProfile` (`teistro_astro::terrain`) is a strict, typed list of
`HorizonSample { azimuthDeg, elevationDeg }`:

- **at least one sample, at most 36,000** (one every hundredth of a degree,
  finer than an elevation model resolves at a skyline's distance);
- **azimuths finite, in `[0, 360)` from north through east, strictly
  increasing**, so sorted and without duplicates; a profile that is not is
  refused rather than sorted, since two samples at one azimuth have no
  reading;
- **elevations finite, above −5° and below 45°.** −5° clears the dip of the sea
  horizon from the highest summit (some 3°); a wall above 45° hides the sky,
  not the horizon, and no Sun stands behind it for long.

Between samples the elevation is read **linearly in azimuth, the last sample
joined to the first through north**, as PVGIS and Stellarium read theirs; one
sample is a level skyline. Linear reading keeps the skyline continuous, so the
altitude less the skyline is continuous in time and a sign change brackets a
crossing.

**The elevation is the skyline as seen**, terrestrial refraction and the
Earth's curve included, which is what a theodolite or a photograph measures and
what HeyWhatsThat and PeakFinder publish. A producer from an elevation model
(TG-09) applies both before handing the profile over.

The JSON request reads it strictly: an unknown key, a `null`, a value of the
wrong kind, or any rule above is `INVALID_ARG` on the field it broke,
`horizonProfile` for the list itself, `horizonProfile[2].azimuthDeg` for a
sample's.

## 3. The event

The visible rise is the first instant the convention's disc point, under its
own refraction, reaches the skyline at the body's current azimuth; the visible
set is the first instant after it that it sinks behind it. In the solver's
geocentric terms (`rise_set.rs`), the event altitude of the centre is

```text
target(t) = e(A(t)) − R · s(e(A(t))) ∓ semidiameter + parallax
```

where `A` is the azimuth at `t`, `e` the profile's elevation there, `R` the
convention's refraction at the horizon (the almanac's 34′, or an air's
Bennett), and `s(e) = Bennett(e) / Bennett(0)` for `e > 0`. So the convention
keeps its disc point and its air, and its refraction is carried to the
skyline's apparent altitude by Bennett's shape. **Below zero `s` is one**: the
ray to a skyline below the level horizon runs through air under the observer
that a profile does not describe, Bennett's formula is not fitted there, and
holding the horizon's value keeps the target continuous at zero. The skyline
replaces the convention's altitude: a `CUSTOM` altitude (a twilight's −6°) has
no meaning behind a terrain, so the terrain is the horizon.

**The azimuth moves while the body climbs**, so the target is a function of
time. The solver reads the azimuth at every sample from the same hour angle and
declination as the altitude, and **searches by its scan** (a ten-minute grid
bisected to 1e-7 d), never its iteration: the iteration settles on whichever
crossing lies nearest its estimate, and behind a notched skyline the first is
wanted. The absence proofs (`horizon-absence-measured.md`) bound a fixed
target's crossing and are not tried; the walk answers. A body that slips behind
a peak and out again within ten minutes may be missed; the first crossing the
grid brackets is the one reported.

**The observer's height** reaches the event only through the air, as it does on
a level horizon (SR-90, `geo-port.md` §6a): under `ATMOSPHERIC` the pressure is
the standard air's at the height, and the fixed 34′ reads none. The solver
applies no dip, since the profile's elevations are measured from the
observer's eye and already hold it.

**Polar days and nights** need nothing new. The day's search is
`Solver::day`'s: the rise inside the civil day, the set after it, and, when
there is neither, which side of the skyline the body stood at the day's
middle. A Sun that never clears the terrain answers no visible rise and set and
`visibleState: ALWAYS_BELOW`; a midnight Sun a northern ridge hides answers a
visible set and rise where the level horizon has none.

## 4. Where it shows up

`almanac.days` takes `horizonProfile` and answers a `visible` section, one entry
per day, sealed under the days' provenance with the applied convention
`visible.horizon` ("4 samples, linear in azimuth"):

```json
{"from": …,
 "sun":  {"rise": …, "set": …,
          "visibleRise": {"at": …, "azimuthDeg": …, "terrainDeg": 2.0},
          "visibleSet":  {"at": …, "azimuthDeg": …, "terrainDeg": 0.0},
          "visibleState": "CROSSES"},
 "moon": {…}}
```

`rise` and `set` are the level horizon's under the day's own sunrise convention,
the visible ones beside them, never in their place. The Moon is answered too:
it costs two more scans a day. The search opens at each day's local midnight
(the civil day), not at the panchanga day's sunrise, because a skyline below
zero brings the visible rise before the astronomical one.

In Rust it is `AlmanacRequest::with_visible(profile)` and
`AlmanacAnswer::visible`; the kernel is `Solver::with_terrain` and
`terrain::visible_day`. The typed C entry point `ts_panchanga_days` takes
coordinates in a fixed struct and carries no profile; bindings reach it through
`record("almanac.days", …)`, whose request type each binding generates.

## 5. The panchanga day stays put

A panchanga day opens at its astronomical sunrise, and the tithi, vara and every
span read from it are the day's. The profile changes none of them: the days and
their hashes with a profile are the days without one, to the bit
(`crates/sdk/tests/terrain_horizon.rs`). The consumer's almanac prints the
astronomical sunrise, which is what the printed panchangas print
(`nepal-day-measured.md`); a visible sunrise is a second line beside it. A day
anchored on the visible sunrise would be a new convention, asked by name, and no
consumer has asked.

## 6. Measured: the acceptance

At Kathmandu (27.7172° N, 85.324° E, 1400 m) on 12 to 14 November 2026 under
the default profile's sunrise (the upper limb, the almanac's 34′), a ridge 2°
high from 80° to 140° azimuth delays the sunrise by the amount derived
independently in the test: the Sun's declination and right ascension from the
*Astronomical Almanac*'s low-precision formulas, the hour angle at which the
centre stands at the level target and at the skyline's
(`2° − 34′ · Bennett(2°)/Bennett(0°)`), and their difference over the hour
angle's rate (360.985647° a day less the Sun's motion in right ascension).

| Day | Measured | Derived | Difference |
| --- | ---: | ---: | ---: |
| 2026-11-12 | 655.290 s | 655.274 s | 0.016 s |
| 2026-11-13 | 656.605 s | 656.587 s | 0.018 s |
| 2026-11-14 | 657.920 s | 657.900 s | 0.020 s |

The test holds them within 0.1 s. The 0.02 s left is the derivation's: its
Sun is good to 0.01° and its hour-angle rate a day's mean. A level skyline of zero gives the level
horizon's rise and set within 0.02 s (the scan and the iteration each settle to
1e-7 d), a skyline 2° high all round gives the ridge's rise to the bit, and the
western set, where the ridge is not, is the level horizon's.
