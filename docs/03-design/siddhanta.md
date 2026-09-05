# The Surya Siddhanta model

Status: `draft`, written 2026-09-05 when `crates/siddhanta` was built
for the Bikram Sambat engine; revised when the crate becomes a provider
behind the ephemeris port. Derives from
`02-architecture/01-module-catalog.md` (the `siddhanta` module),
`03-design/calendar-bikram-sambat.md` (the first consumer) and the text
itself in the translation of Ebenezer Burgess (1860), which is the
rank-1 source for every number here (`CLEAN_ROOM.md`); the baseline
engine's implementation is rank 2 and was read for its structure only.

## 1. Purpose and scope

The Surya Siddhanta is the text the classical panchangas of Nepal and
much of India compute from, and an official calendar computed from it
cannot be reproduced by modern astronomy (the source memo
`docs/calendars/bikram-sambat.md` measures the gap). This page settles
the model as a computation: the text's mean motions, apsides and nodes,
its sine table and interpolation rule, the manda and sighra equations
with the four-step procedure for the star planets, the true daily motion
of the Sun and the Moon, the text's precession, declination and
ascensional difference (which give sunrise and sunset the way an almanac
computes them), and nothing the text does not say. Latitudes (II.56 to
58), the sighra daily motion (II.50 to 51) and the Lagna (III.42 to 50)
are later additions to the same crate. The crate is a model first and a
provider second: it answers for a graha at an instant; the adapter that
presents it behind the ephemeris port arrives with the port's promotion.

## 2. Inputs, settings and ports

An instant as `JulianDay<Ut1>` (the text's day is the mean solar day, so
Universal Time is the right scale and no Delta T applies), a `Graha`
from the catalogue, a latitude for the day's arc. Two choices, both
plain values: the [`Parameters`] (the text's, or the text's with a
tradition's bija applied) and the [`Trig`] (the text's table, or exact
trigonometry for comparison). The settings knob `frame.siddhanta`
selects the model in a profile; `Surya { bija: true }` is refused as
unsourced until a bija set is cited (§10). No ports.

## 3. The data model

```rust
pub struct Parameters {                    // every field cites its verse (params.rs)
    yuga_civil_days: u64,                  // 1 577 917 828, I.37
    yuga_years: u32, kalpa_yugas: u32,     // 4 320 000 (I.15 to 17), 1000 (I.19 to 21)
    elapsed_days_at_kali: u64,             // 714 402 296 627: I.22 to 24 turned into days by I.37
    epoch_jd_ut: f64,                      // Kali start at midnight on the meridian of Lanka (I.48 to 53, I.62), in UT
    meridian_deg: f64,                     // Ujjain, 75°47′ E
    motions: [Motion; 7],                  // Sun, Moon, Mars, Mercury's conjunction, Jupiter, Venus's conjunction, Saturn: I.29 to 34
    moon_apsis: Motion, moon_node: Motion, // I.34; the node retrograde
    apsides: [Motion; 6], nodes: [Motion; 5],   // per aeon: I.41 to 44
    manda: [Epicycle; 7], sighra: [Epicycle; 5],   // circumferences at the even and odd quadrant ends: II.34 to 37
    obliquity_sine: u32,                   // 1397 on a radius of 3438: II.28
    ayana_revolutions_per_yuga: u32, ayana_extent_deg: u32,   // 600 and 27: III.9 to 12
}
pub struct Motion { revolutions: u64, cycle: Cycle /* Yuga | Kalpa */, retrograde: bool }
pub struct Epicycle { even_arcmin: u32, odd_arcmin: u32 }
pub struct Bija { moon, moon_apsis, moon_node, mars, mercury, jupiter, venus, saturn: i64 }   // whole revolutions per age added to the counts
pub enum Trig { Table, Exact }
pub struct Ahargana { days: i64, fraction: f64 }   // whole days since the epoch, exact; the current day's fraction
pub struct Position { longitude: Degrees, speed_deg_per_day: f64 }
pub struct Trace { graha, ahargana, mean_deg, apsis_deg, conjunction_deg: Option<f64>, manda_equation_deg,
                   manda_corrected_deg, sighra_equation_deg: Option<f64>, karna: Option<f64>, longitude_deg, speed_deg_per_day }
pub struct DayArc { sunrise: JulianDay<Ut1>, sunset: JulianDay<Ut1>, ascensional_difference_deg, declination_deg }
```

The sidereal frame is the text's own: mean places counted from the
first of Mesha at the start of the Kali age (I.57). It is not Lahiri's;
the text's precession (III.9 to 12) relates it to the tropical frame,
and a consumer wanting Lahiri applies the `astro` ayanamsha to the
tropical longitude instead. The provenance of a result computed from this
model carries `Deviation { model: "SURYA_SIDDHANTA", .. }`.

## 4. Algorithms

**The day count.** `Ahargana::at(jd_ut)` is the instant less the epoch,
split into whole days and a fraction. The epoch is the start of the Kali
age at midnight on the meridian of Lanka, JD 588 465.5 less Ujjain's
longitude in days; the text's days are reckoned to that midnight (I.48
to 53). The tradition's hand computations count the current day as
elapsed, so their count is one more than this one for the same civil
day; the worked example in the tests shows both.

**Mean places** (I.29 to 34, I.41 to 44). The days since the aeon began
(the text's elapsed days at Kali plus the count) times the revolutions,
modulo the cycle's days: the whole days in 128-bit integer arithmetic,
only the current day's fraction in floating point, so every mean place is
exact to the fraction and bit-identical everywhere. At the epoch every
planet is at the first of Mesha, the Sun's apsis at 77°7′48″, the Moon's
apsis at 90° and its node at 180°, all from the text's numbers and none
adjusted (Burgess's notes on I.41 to 44 agree). Retrograde motions count
down.

**The sine table** (II.15 to 27, II.31 to 33). Twenty-four sines on a
radius of 3438 in steps of 225′; the interpolation rule of the text
(the quotient counts entries, the remainder times the next difference
over 225 is added); the inverse by the same rule. [`Trig::Exact`]
substitutes the platform's sine on the same radius for comparison; the
two agree within 2 units of the radius, and the classical path uses no
platform mathematics.

**The corrected epicycle and the equations** (II.29 to 45). The anomaly
is the apsis less the mean place (or the conjunction less the planet),
reduced to the first quadrant with the signs its sine and cosine carry.
The epicycle's circumference at the reduced anomaly is the even-quadrant
value moved toward the odd-quadrant value by the sine over the radius
(II.38). The manda equation is the arc of the sine of the reduced anomaly
times that circumference over 360 (II.39 to 40); the sighra equation goes
through the hypotenuse: the cosine's share applied to the radius, added
in the half from Makara through Mithuna and taken away in the other, the
hypotenuse by Pythagoras, the sine of the equation as the sine's share
times the radius over the hypotenuse (II.39 to 42). Each equation is
additive when the anomaly is in the half of the circle that begins with
Mesha and subtractive in the other (II.45). The Sun and the Moon take
the manda equation alone; the star planets take the four steps of
II.43 to 44: half the sighra equation to the mean place, half the manda
equation of the result to the result, the whole manda equation of that
applied to the original mean place, the whole sighra equation of that
applied to it. Mercury's and Venus's mean place is the Sun's and their
own motion is their conjunction; the others' conjunction is the Sun's
mean place (I.29).

**Daily motion** (II.47 to 49). The anomaly's daily motion (the mean
motion less the apsis's) times the cosine of the reduced anomaly (the
table's difference of sines over the step) times the corrected
circumference over 360, taken from the mean motion in the half from
Makara through Mithuna and added in the other. The star planets' motion
is the change over the day centred on the instant until II.50 to 51 is
implemented.

**Precession** (III.9 to 12). The equinoxes librate 600 times an age;
the reduced arc of the libration's argument, three tenths of it, is the
ayanamsha, at most 27°. The sign is taken as the tradition applies the
text today: zero at the start of the Kali age and again 3600 years later
(Shaka 421), positive since, 54″ a year, so the tropical longitude is
the sidereal one plus this (22.875° in 2024).

**Declination and the day's arc** (II.28, III.14 to 17, III.34 to 35,
III.42 to 43). The sine of the declination is the sine of the tropical
longitude times the sine of the greatest declination over the radius.
The equinoctial shadow of a twelve-digit gnomon is twelve times the sine
of the latitude over its cosine; the earth-sine is the sine of the
declination times the shadow over twelve; the earth-sine times the
radius over the day-radius is the sine of the ascensional difference.
Half the day is a quarter of the circle plus the difference, so the Sun
rises the difference before six and sets it after eighteen in local mean
time; the declination is taken at the rise and set instants themselves,
each found in two passes. Where the sine of the difference exceeds the
radius the Sun neither rises nor sets and the arc is `None`.

## 5. The API

Rust: `SuryaSiddhanta::text()` and `SuryaSiddhanta::new(params, trig)`;
`sun(at)`, `moon(at)`, `graha(graha, at)`, `all(at)` (the nine in the
catalogue's order), `trace(graha, at)` (the tradition's figures),
`sun_longitude_deg(jd)` (the form a root finder calls), `ahargana(at)`,
`mean_deg`, `apsis_deg`, `node_deg`, `ayanamsha_deg(at)`,
`declination_deg(tropical)`, `ascensional_difference_deg(latitude,
declination)`, `day_arc(local_mean_midnight, latitude)`, `describe()`;
`Parameters::TEXT`, `Parameters::with_bija(&Bija)`. The calendar crate
implements its `SolarModel` over this (`calendar-bikram-sambat.md`). C ABI
and bindings: through the ephemeris port once the crate is a provider;
until then the model is Rust-only.

## 6. Errors and degenerate states

| situation | outcome |
|---|---|
| a graha the text does not model (Uranus, Neptune, Pluto) | `UNSUPPORTED` naming the graha, field `graha`, with the hint of what the text knows |
| `Surya { bija: true }` in a profile | `UNSUPPORTED (unsourced)` until a bija set is cited; a consumer's own `Bija` through `with_bija` is accepted |
| a latitude and declination where the Sun neither rises nor sets | `day_arc` and `ascensional_difference_deg` return `None`; no error |
| an instant before the Kali age | a negative count; every place still wraps correctly (tested) |

## 7. Performance budget

| operation | budget | measured (`cargo bench -p teistro-siddhanta`, Apple M-series, 2026-09-05) |
|---|---:|---:|
| the Sun's longitude, table | 200 ns | 54 ns |
| the Sun's longitude, exact trigonometry | 300 ns | 107 ns |
| the Moon, table | 300 ns | 71 ns |
| a star planet (four steps, a central difference), table | 2 µs | 712 ns |
| all nine grahas, table | 5 µs | 3.8 µs |
| the day's arc at Kathmandu | 1 µs | 618 ns |

No allocation anywhere in the crate.

## 8. Tests

- The elapsed days at Kali are the text's years turned into days (I.22
  to 24 with I.37); at the epoch every planet is at the first of Mesha,
  the Sun's apsis at 77°7′48″, the Moon's apsis at 90°, its node at 180°;
  a whole age later everything returns.
- The sine table is the text's, exact at its nodes, within a unit of the
  true sine; the interpolation inverts; the worked figure of II.31 to 33
  (61°31′9″ gives 3020.94); the slope is the cosine.
- The Sun's greatest equation is the arc of 13°40′/360 of the radius,
  2.175°; the tradition's worked figure (a circumference of 13°43′ at a
  sine of 3020.93 gives 115.1′) reproduces; exact and tabular
  trigonometry agree within 0.002°.
- The sighra equation vanishes at conjunction and opposition with the
  hypotenuse at the radius plus or less the circumference's share; Mars's
  greatest equation is between 40° and 46°, Mercury's greatest elongation
  between 20° and 24°.
- The four steps compose as the text says; the true motion is slowest at
  the apsis and equals the mean motion at quadrature.
- A hand computation of the tradition for 31 October 1994 at Kathmandu
  (mean Sun 6 signs 15°46′30″, apsis 77°17′39″, equation 1°55′6″, true Sun
  6 signs 13°51′) reproduces at the tradition's count, one day beyond the
  text's.
- The Sun crosses Mesha in mid-April (1 Baisakh 2081 BS is 13 April
  2024), moves about a degree a day, fastest in January and slowest in
  July; every graha answers, Ketu is opposite Rahu, Mercury stays within
  30° of the Sun, the unknown grahas are refused.
- Precession is zero at Shaka 421, 22.875° in 2024 and negative before.
- The declination is 24° at the solstice; the ascensional difference is
  zero at the equator, asin(tan φ tan δ) at Kathmandu, mirrored south of
  the equator, absent above the polar circle; Kathmandu's day is 13.8
  hours in June and 10.2 in December.
- Property tests: every longitude in range and continuous over a
  hundredth of a day across three centuries; the table inverts and
  tracks the true sine.
- A golden bit pattern of the Sun at J2000.0 through the table: a change
  there is a change in every number and needs a calculation-version
  entry.

## 9. Localisation

None: the crate emits catalogue keys (`graha.SUN`) and numbers. The
planets' Sanskrit names in `Planet::name` are for messages and stamps;
the locale packs carry the presentation forms.

## 10. Open questions

1. **The bija sets.** The later commentators (Ranganatha, 1603; the
   Makaranda tables) apply corrections to the revolution counts that
   differ between them; none is cited here yet, so `Surya { bija: true }`
   is unsourced and refused, and a consumer supplies its own `Bija`
   (cruxes register C28).
2. **The epicycle convention.** The text places the even-quadrant value
   at the anomaly's 0° and 180° and the odd-quadrant value at 90° and
   270° (II.34 to 38); the baseline engine had them swapped, which moves
   the Sun's greatest equation by 0.05°; the text wins (C27).
3. **The sign of the libration.** III.9 to 12 give a triangular
   libration; the tradition applies it with the sign that makes the
   ayanamsha positive today; the crate follows the tradition and says so.
4. **II.50 to 51** (the sighra daily motion), **II.56 to 58** (latitudes)
   and **III.42 to 50** (the Lagna from the oblique ascensions) are not
   yet implemented; the star planets' motion is a central difference.
5. **The tradition's day count** is one more than the text's midnight
   count for the same civil day (the worked example); the Bikram Sambat
   measurement shows the difference is immaterial to the calendar under
   the punya-kala rule.
