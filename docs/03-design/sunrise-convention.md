# The sunrise convention

Status: decided 2026-10-25.

Every panchanga answer hangs on sunrise: the day's limbs are the ones
standing at it, a vriddhi or kshaya is decided by it, and the hora, the
muhurtas and the kaalas are counted from it. Which point of the Sun's disc
counts, and whether the air's refraction lifts it, moves the instant by up
to about four minutes. This page records which convention each profile
reads and why.

## 1. The conventions

`day.sunrise` names one of four (`Sunrise`), or a custom altitude:

| Key | The Sun's position at the rise | Against the centre without refraction |
|---|---|---|
| `CENTRE_NO_REFRACTION` | the disc's centre on the geometric horizon | 0 |
| `CENTRE_REFRACTION` | the centre, with standard refraction (34′) | about 2½ minutes earlier |
| `UPPER_LIMB_REFRACTION` | the upper limb, with refraction (50′ in all) | about 4 minutes earlier |
| `LOWER_LIMB_REFRACTION` | the lower limb, with refraction | about 1 minute earlier |

The minutes are at mid-latitudes and grow toward the poles.

## 2. The evidence

| Source | What it reads | Where |
|---|---|---|
| The Surya Siddhanta | the centre on the geometric horizon, no refraction, in local mean time | III.34 to 35, III.42 to 43; crux C37 |
| Swiss Ephemeris | the upper limb with refraction by default; `SE_BIT_HINDU_RISING` is the centre without refraction, named for Hindu astrology | the programmer's manual, §8.12 |
| Drik Panchang | the upper limb with refraction, "currently used and suggested"; the middle limb as an option | its sunrise settings page |
| Drik Panchang, an older FAQ | the disc's centre, refraction ignored, for festivals | its FAQ; the default has moved since |
| Nepal's daily print | the centre with refraction: a median 0.4 minutes off over 333 days, 320 within 1.5 minutes; the upper limb 1.7 minutes early, the centre without refraction 2.5 late | [`nepal-day-measured.md`](nepal-day-measured.md) §4, C39 |
| The Calendar Reform Committee (1955) | names the centre on the horizon the "calculated" sunrise and the observed upper limb about 4½ minutes earlier; fixes no convention | its report, pp. 225 to 226 |

The Indian Astronomical Ephemeris and the Rashtriya Panchang state none
that could be found; Kalnirnay and the Sringeri and Ujjain panchangas
neither.

## 3. The decision

- **Modern profiles read the upper limb with refraction**
  (`UPPER_LIMB_REFRACTION`): the root, and so `parashari-classical` and
  `kp-default`. It is the sunrise a user sees and checks the answer
  against in a newspaper or an almanac, the default of Drik Panchang and
  of Swiss Ephemeris, and the convention the root already read for the
  eclipse horizon, so the root no longer reads two.
- **`nepali-drik` reads the centre with refraction**
  (`CENTRE_REFRACTION`), which is Nepal's print to a median 0.4 minutes;
  the centre without refraction it read before stood 2.5 minutes late.
- **`nepali-committee` keeps the centre with refraction**, measured.
- **`surya-siddhanta` keeps the text's own sunrise**, the centre without
  refraction in mean time.
- **`conformance-baseline` keeps the upper limb**, its recording engine's.
- `CENTRE_NO_REFRACTION` stays a value any caller may set: Swiss
  Ephemeris's "Hindu rising", which also takes the Sun geocentric
  without its latitude.

What would change it: printed sunrises from a widely used Indian
panchanga, measured as Nepal's were, clustering on the disc's centre.
