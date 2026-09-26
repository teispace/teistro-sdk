# The air at the horizon: a settable atmosphere for rise and set

Status: **built**, 2026-09-26. Written from measurements before any
code; §6 is the acceptance, measured on the built thing.

Cruxes C34. `astro-events-and-crossings.md` §10 left one question open:
a settable atmosphere (pressure, temperature, a refraction model) beside
the almanac's 34 arcminutes, so a chart can ask for the engines'
convention by name. This page answers it: what the engines do, what the
sources give, what was measured, and what is built.

## 1. What the SDK did

`Refraction` was two-valued. `NONE` is the geometric horizon. `STANDARD`
is the almanac's 34′ at the horizon, whatever the place and the weather
(*Astronomical Almanac*; Meeus, ch. 15). A native search given
`STANDARD` used the engine's own standard atmosphere instead, and the two
were documented as different.

## 2. What the engines do

Teimeris, like the Swiss Ephemeris it descends from, refracts through an
**atmosphere at the observer**: its standard is 15 °C at a pressure taken
from the observer's height, and its default model is Sinclair's, which is
continuous through the horizon. A caller may give the pressure and the
temperature instead (`tm_atmosphere`). Skyfield and PyEphem likewise take
a pressure and a temperature, and scale Bennett's formula by them.

## 3. Measured: height is the term that matters

Teimeris's own sunrise search, under its default air, for the same date
at each place's height against the same place at sea level:

| Place | Height | Sunrise later than at sea level |
| --- | ---: | ---: |
| Delhi | 216 m | 4.3 s |
| Fairbanks | 136 m | 5.9 s |
| Kathmandu | 1,400 m | 26.0 s |
| Denver | 1,609 m | 34.3 s |
| La Paz | 3,640 m | 56.0 s |

The fixed 34′ ignores height, so at Kathmandu it is half a minute from
every engine before any model question arises. Thin air refracts less:
at 856 hPa the horizon refraction is about 28′, not 34′.

## 4. Measured: the models agree at the reference air and part with the temperature

The refraction at an apparent altitude of zero, arcminutes. Teimeris's
two models are found by bisecting the true altitude whose apparent
altitude is zero, which is what its rise search does. "Bennett, scaled"
is Meeus's equation 16.4 multiplied by `(P / 1010) · (283 / (273 + T))`
(Meeus, ch. 16), the form Skyfield's `refraction()` uses.

| Pressure | Temperature | Sinclair (Teimeris) | Bennett (Teimeris) | Bennett, scaled |
| ---: | ---: | ---: | ---: | ---: |
| 1010 hPa | 10 °C | 34.460 | 34.433 | 34.478 |
| 1013.25 hPa | 15 °C | 33.593 | 33.846 | 33.988 |
| 1013.25 hPa | 0 °C | 36.740 | 36.103 | 35.856 |
| 1000 hPa | 25 °C | 31.328 | 31.988 | 32.418 |
| 950 hPa | 10 °C | 32.237 | 32.002 | 32.429 |
| 850 hPa | −10 °C | 32.332 | 30.598 | 31.222 |
| 1030 hPa | −20 °C | 42.736 | 40.445 | 39.330 |

At the reference air all three agree within 0.05′. Elsewhere Sinclair's
correction depends on the temperature about half again as strongly as the
density scaling does. That correction has no public source this project
could cite: the formula is known through the Swiss Ephemeris's AGPL
source, which the SDK does not copy, so the SDK does not reproduce it.
The ray-traced standard atmosphere gives 34.5′ at the horizon (Thomas and
Joseph, *Johns Hopkins APL Technical Digest* 17(3), 1996, Table 1, after
the *Nautical Almanac*), which the three agree with at the reference air.

## 5. Decided

**A convention names its air.** `day.sunrise` gains a third form beside
`NAMED` and `CUSTOM`:

```json
{ "kind": "ATMOSPHERIC", "which": "UPPER_LIMB_REFRACTION",
  "air": { "pressure_hpa": null, "temperature_c": null } }
```

It is a refracted named convention whose refraction comes from the given
air rather than the almanac's 34′. `which` must be a convention that
refracts (`UPPER_LIMB_REFRACTION` or `LOWER_LIMB_REFRACTION`); the
settings refuse one that does not, since there is nothing for the air to
act on. A variant rather than a knob, so every existing settings document
hashes as it did.

**Each part of the air may be left out**, and what fills it is the
engines' standard, resolved at the place:

- the **pressure**, from the observer's height by the ICAO standard
  atmosphere, `1013.25 · (1 − 0.0065 h / 288.15)^5.25588` hPa;
- the **temperature**, 15 °C, the standard atmosphere's at sea level and
  the engines' default at every height.

The pressure is within 0.15 hPa of Teimeris's own standard at every
height to 8,848 m (0.05 hPa at Kathmandu), which is 0.006′ of
refraction. So `"air": {}` asks for the engines' own convention by name, and a
consumer with a weather report gives both. The settings refuse a pressure
outside 100 to 1100 hPa and a temperature outside −90 to 60 °C.

**The SDK's solver refracts by Bennett, scaled by the air** (§4's last
column): `R = cot(7.31° / 4.4) · (P / 1010) · (283 / (273 + T))`
arcminutes at the horizon. It is the published form, it agrees with the
ray-traced atmosphere at the reference air, and it is what the Python
astronomy libraries use. Its difference from Teimeris's Sinclair model is
a model difference, measured and reported, not an unnamed convention.

**A native search is given the same air, resolved.** The horizon request
carries the pressure and the temperature the SDK resolved, so an engine
and the SDK's solver are asked about the same air and differ only by
their models. Teimeris receives them as its `tm_atmosphere`.

**What was applied is reported.** The chart's day section gains the air
the arc was reckoned under, `air_pressure_hpa` and `air_temperature_c`,
zero unless the convention is atmospheric; `convention_kind` stays the
named convention the air was given to.

## 6. Measured: the acceptance

The adapter's fixture test runs the 55 conformance charts at their own
recorded heights, 1 m to 3,640 m, and times the day's sunrise and sunset
from the SDK's solver and from Teimeris's own search under each
convention (`adapters/ephemeris-teimeris/rust/tests/fixtures.rs`,
`naming_the_air_brings_the_solver_to_the_engine_at_every_height`). 106
events, SDK against engine, seconds:

| Charts | Events | Fixed 34′, median | Fixed 34′, worst | Air named, median | Air named, worst |
| --- | ---: | ---: | ---: | ---: | ---: |
| below 200 m | 60 | 2.7 | 43.5 | 2.0 | 23.2 |
| 200 to 1,000 m | 6 | 16.0 | 19.1 | 2.7 | 3.1 |
| 1,000 m and above | 40 | 29.9 | 61.8 | 3.7 | 5.8 |

At La Paz the gap falls from 61.8 s to 5.8 s. What remains grows with
the height, as Bennett's density scaling and Sinclair's correction part
where the air thins, and the worst below 200 m is Fairbanks at the
solstice, where the Sun grazes the horizon and any model difference is
stretched into seconds. The test holds both sides: the named air within
8 s of the engine at every chart at or above 1,000 m, and the fixed 34′
more than 20 s away in the median there, so neither bound can pass by the
other standing still.

The baseline's own sunrise is reckoned at sea level; placed there, the
SDK's solver under the fixed 34′ stays within 9.8 s of it, as before.

## 7. The boundary

`Refraction` gains `ATMOSPHERE` (id 2), carrying the air. The C horizon
request appends two fields, the resolved `pressure_hpa` and
`temperature_c` (zero unless the refraction is `ATMOSPHERE`), and the
vtable's ABI version becomes 4. Appending is the path `struct_size`
exists for; the package is unreleased, so no plugin compiled against
version 3 exists outside this repository.

## 8. Not decided here

- **The dip of a raised horizon.** An observer above a sea horizon sees
  below the geometric one (Teimeris's `TM_EVENT_HORIZON_SEA`). That is a
  horizon altitude, not an air, and waits for a consumer who asks.
- **Sinclair's correction.** If a citable source for it is found, it
  becomes a model the air names; until then the difference is measured.
- **The standard convention in a native search.** `STANDARD` still means
  the engine's own standard there; a chart that wants the SDK and the
  engine to agree names the air.
