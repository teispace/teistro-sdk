# The moonrise convention and the composed horizon

Status: decided 2026-11-16 (SR-102, and SR-113 as far as it shares this
design). Built: `day.moonrise` in `crates/core/src/settings/mod.rs`, the
solver's horizon in `crates/port-ephemeris/src/horizon.rs`, read by
`crates/panchanga/src/almanac.rs` and `crates/astro/src/day_sky.rs`.

Until this change the Moon rose and set by `day.sunrise`. That setting
answers which point of the **Sun's** disc opens the day
([`sunrise-convention.md`](sunrise-convention.md)), and Nepal's profiles
set it to the disc's centre with refraction, because that is Nepal's
printed sunrise. The Moon inherited it: a Nepal almanac's moonrise came
out at the centre, some eighty seconds after the upper limb that every
almanac prints for the Moon. This page separates the two and records
which convention each profile reads for the Moon, and how a horizon is
composed of its parts.

## 1. The knob

`day.moonrise` is a `SunriseConvention`, the same type as `day.sunrise`
and `panchanga.eclipse_horizon`: one type for every horizon, so a
convention, its validation and its description are written once
(`HORIZON_KNOBS`, and the coherence rules apply to all three). It takes:

- a named convention, `{"kind":"NAMED","which":…}`, where `Sunrise` now
  names all six pairs of disc point and refraction: `CENTRE_NO_REFRACTION`,
  `UPPER_LIMB_NO_REFRACTION`, `LOWER_LIMB_NO_REFRACTION` (new),
  `CENTRE_REFRACTION`, `UPPER_LIMB_REFRACTION`, `LOWER_LIMB_REFRACTION`;
- a custom altitude, `{"kind":"CUSTOM","altitude_deg":…}`;
- an atmospheric one, a named convention refracted through a stated air
  ([`horizon-atmosphere.md`](horizon-atmosphere.md));
- **a composed one** (§4), `{"kind":"COMPOSED","disc":…,"refraction":…,
  "height_m":…}`.

The default, at the root, is `UPPER_LIMB_REFRACTION`. The Moon's event
altitude is then the definition every almanac states: her centre at
−34′ − SD + π, the semidiameter and the horizontal parallax taken at the
instant from her distance (SD = asin(1737.4 km / d), π = asin(6378.137 km
/ d)). The solver had this already for any body; only the choice of
convention was the Sun's.

## 2. The evidence for the upper limb

| Source | The Moon's rise is | Where |
|---|---|---|
| USNO, "Rise, Set, and Twilight Definitions" | the upper limb on the horizon, refraction 34′, the zenith distance 90°34′ + SD − π, the parallax taken into account because it is large for the Moon | aa.usno.navy.mil, Astronomical Applications |
| The Explanatory Supplement to the Astronomical Almanac (3rd ed.) | the same definition, h₀ for the Moon varying with her distance | §9.33 (rising and setting) |
| Meeus, *Astronomical Algorithms* (2nd ed.) | h₀ = 0.7275π − 34′ for the Moon, the upper limb with standard refraction | ch. 15 |
| timeanddate.com, Kathmandu, January 2026 | 62 rises and sets; every one is the DE440 oracle's upper limb with 34′ (§6) truncated to the minute | its moonrise table for Kathmandu |
| Nepal's committee | timed the 2026-03-03 eclipse's vedha from a moonrise at 18:03: the upper limb with refraction rises at 18:03.7, the centre with refraction at 18:05.0, the centre without it at 18:07.7 | [`eclipses.md`](eclipses.md) §5, C194, C2000 |
| The baseline engine | the upper limb with 34′ for every moonrise, whatever its sunrise mode, on a geocentric centre | `conformance-baseline`'s recording |

Every source here that states or prints a moonrise reads the upper limb
with refraction; none reads the centre for the Moon.

## 3. The Surya Siddhanta's moonrise

The text computes the Moon's rising (ch. X, the Moon's rising and
setting, Burgess 1860) from her ascensional difference as it computes the
Sun's (III.34 to 35): the body's centre on the geometric horizon, with no
refraction and no semidiameter. `surya-siddhanta` reads
`CENTRE_NO_REFRACTION` for the Moon as for the Sun, and a context whose
sunrise is the text's (`siddhanta_sunrise: TEXT`) warns
(`siddhanta-text-moonrise-convention`) when its moonrise is anything
else, as it warns for the sunrise.

## 4. The composed horizon and the dip (SR-113)

SR-113 asks for the horizon as a setting of its parts: the disc's point,
the refraction, the observer's height and a horizon profile. The first
three compose here; the profile does not (§7).

`{"kind":"COMPOSED","disc":"UPPER_LIMB","refraction":"STANDARD",
"height_m":30}` reads:

- `disc`: `DiscPoint`, `CENTRE`, `UPPER_LIMB` or `LOWER_LIMB`, now a
  settings knob (it was the solver's private type);
- `refraction`: `NONE`, `STANDARD` (34′) or `ATMOSPHERE` with an air,
  now a settings type the solver re-exports;
- `height_m`: the eye's height above the visible horizon, 0 to 10 000 m
  (`OBSERVER_HEIGHT_M`), 0 by default. It is not the place's altitude:
  a city at 1400 m on a plateau sees a horizon at its own level.

The height lowers the horizon by the **dip**:

- with refraction, 1.76′ √h (h in metres), the Nautical Almanac's dip
  of the sea horizon, as Bowditch's *American Practical Navigator*
  (Appendix B) tabulates it, the terrestrial refraction folded in;
- without refraction, the geometric dip acos(R / (R + h)) with R the
  Earth's mean radius, 6 371 008.8 m (IUGG), computed as
  asin(√(h(2R + h)) / (R + h)) so that it stays exact for a small h.

At 30 m the dip is 9.6′ refracted and 10.6′ geometric; the Sun clears it
some forty seconds sooner at Kathmandu's latitude. A composed convention
with no height is the named one, to the bit, and a composed convention's
air is checked as an atmospheric one's (`sunrise-height`,
`sunrise-air`, `sunrise-air-refracts`). The dip crosses every binding as the day's `dipDeg`
(`dip_deg` in Python), so a reader can tell a lowered horizon from a
level one; across the C boundary a composed convention is named by its
disc and refraction and its dip is the convention value, negated.

The reference backend's four sunrise modes (its default the upper limb
with refraction) are each one point of this space, a disc point and a
refraction, and need no knob of their own.

## 5. The profiles

| Profile | `day.sunrise` | `day.moonrise` | Why |
|---|---|---|---|
| the root (`parashari-classical`, `kp-default`) | `UPPER_LIMB_REFRACTION` | `UPPER_LIMB_REFRACTION` | §2; inherited |
| `nepali-drik` (v7) | `CENTRE_REFRACTION` | `UPPER_LIMB_REFRACTION` | the committee's moonrise and timeanddate's (§2); the day keeps Nepal's printed sunrise |
| `nepali-committee` (v7) | `CENTRE_REFRACTION` | `UPPER_LIMB_REFRACTION` | the same evidence |
| `surya-siddhanta` (v4) | the text's | `CENTRE_NO_REFRACTION` | §3 |
| `tamil-vakya` (v3) | its own | `UPPER_LIMB_REFRACTION` | the root's, stated so the profile does not lean on it; no Tamil almanac's moonrise was measured |
| `conformance-baseline` (v15) | `UPPER_LIMB_REFRACTION` | `UPPER_LIMB_REFRACTION` | the recording engine's, stated so the profile does not lean on the root |

What would change it: a Nepal almanac printing moonrises that cluster on
the disc's centre (C2000).

## 6. Measured

`crates/sdk/tests/limb_oracle.rs` holds every moonrise and moonset of
1900, 1950, 2000, 2050 and 2100 at Kathmandu under `nepali-drik`,
`parashari-classical` and `kp-default` to DE440's, recorded by
`record_limbs.py --moon` (`oracle/moonrises.json`, schema
`teistro/moon-oracle/1`): the Moon's geocentric place from DE440 with the
Swiss reductions, her altitude reduced to the observer, and the root of
−34′ − SD + π found to a millisecond. Under every drik profile the worst moonrise is
0.030 s from DE440 and the worst moonset 0.044 s, against a bound of
0.1 s, the sunrise's (SR-102 asked a second). Under the old convention,
the centre with refraction, the median moonrise was 76 to 79 s late and
the median moonset as early, the worst 94 s ([`limb-oracle.md`](../05-testing/limb-oracle.md) §4).

`crates/sdk/tests/moonrise.rs` holds the published table: timeanddate's
January 2026 rises and sets for Kathmandu, each inside its printed
minute under every drik profile. Under the old convention (the centre,
as `nepali-drik`'s sunrise) half the rises fall outside it.

## 7. Left

- **A horizon profile as a setting.** The skyline already exists as a
  request (`terrain-horizon.md`, SR-91) and answers the visible rise
  beside the astronomical one. Making it a setting would put a
  360-sample profile into the settings document and its hash, and move
  the panchanga day off the astronomical sunrise every almanac prints.
  That is a different design and is left open (C2001).
- **Topocentric refraction by the place's air for the Moon** works
  through `ATMOSPHERIC` and `COMPOSED` with `ATMOSPHERE` as for the Sun;
  no profile sets it.
