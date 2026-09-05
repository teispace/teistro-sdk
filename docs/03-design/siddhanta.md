# The Surya Siddhanta model

Status: `draft`, written 2026-09-05 when `crates/siddhanta` was built
for the Bikram Sambat engine; revised the same day when the crate gained
the sighra daily motion, the latitudes and the Lagna and became a
provider behind the ephemeris port. Derives from
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
computes them), the latitudes (II.56 to 58), the true daily motion of
the star planets (II.50 to 51), the Lagna from the oblique ascensions
(III.42 to 50), and nothing the text does not say. The crate is a model
first and a provider second: it answers for a graha at an instant, and
`SiddhantaProvider` presents it behind the ephemeris port so a classical
chart runs through the same trait, completion and solver as the engines.

## 2. Inputs, settings and ports

An instant as `JulianDay<Ut1>` (the text's day is the mean solar day, so
Universal Time is the right scale and no Delta T applies), a `Graha`
from the catalogue, a latitude for the day's arc. Two choices, both
plain values: the [`Parameters`] (the text's, or the text's with a
tradition's bija applied) and the [`Trig`] (the text's table, or exact
trigonometry for comparison). The settings knob `frame.siddhanta`
selects the model in a profile; `Surya { bija: true }` is refused as
unsourced until a bija set is cited (§10). The model uses no port; the
provider adapter answers one (§5).

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
    extreme_latitudes_arcmin: [u32; 6],    // Moon 270, Mars 90, Mercury 120, Jupiter 60, Venus 120, Saturn 120: I.68 to 70
    lanka_rising_asu: [u32; 3],            // 1670, 1795, 1935 for the first three signs: III.42 to 43
}
pub struct Motion { revolutions: u64, cycle: Cycle /* Yuga | Kalpa */, retrograde: bool }
pub struct Epicycle { even_arcmin: u32, odd_arcmin: u32 }
pub struct Bija { moon, moon_apsis, moon_node, mars, mercury, jupiter, venus, saturn: i64 }   // whole revolutions per age added to the counts
pub enum Trig { Table, Exact }
pub struct Ahargana { days: i64, fraction: f64 }   // whole days since the epoch, exact; the current day's fraction
pub struct Position { longitude: Degrees, latitude_deg: f64, speed_deg_per_day: f64 }
pub struct Trace { graha, ahargana, mean_deg, apsis_deg, node_deg: Option<f64>, conjunction_deg: Option<f64>,
                   manda_equation_deg, manda_corrected_deg, sighra_equation_deg: Option<f64>, karna: Option<f64>,
                   longitude_deg, latitude_deg, speed_deg_per_day }
pub struct DayArc { sunrise: JulianDay<Ut1>, sunset: JulianDay<Ut1>, ascensional_difference_deg, declination_deg }
pub struct RisingTimes { asu: [f64; 12] }   // the oblique rising time of each sign at a latitude, in respirations
pub struct Lagna { sidereal_deg, tropical_deg, meridian_sidereal_deg, sun_tropical_deg, elapsed_asu, rising: RisingTimes }
pub struct SiddhantaProvider { model: SuryaSiddhanta }   // the model behind the ephemeris port
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

**Daily motion** (II.47 to 51). The Sun's and the Moon's: the anomaly's
daily motion (the mean motion less the apsis's) times the table's
difference of sines at the reduced anomaly over the step, times the
corrected circumference over 360, taken from the mean motion in the half
from Makara through Mithuna and added in the other (II.47 to 49). The
star planets': the same manda correction of the mean motion, then the
sighra correction (II.50 to 51): the conjunction's motion less the
equated motion, times the hypotenuse less the radius over the
hypotenuse, added to the equated motion; a negative result is
retrograde. Mercury and Venus take the Sun's motion as the mean and
their own as the conjunction's, the others their own and the Sun's. This
is the text's rule and not the derivative of the text's places: the kit
measures the two up to 0.23° a day apart for Mars (cruxes C36), so the
provider declares its speeds as a rule and the kit publishes the gap.

**Latitudes** (I.68 to 70, II.56 to 58). The extreme latitudes are the
Moon's 270′, Mars's 90′, Mercury's and Venus's 120′, Jupiter's 60′ and
Saturn's 120′. The latitude is the sine of the planet's distance from
its node times the extreme latitude over the hypotenuse (over the radius
for the Moon), north when the reduced arc's sine is positive. The
distance is taken from the manda-corrected place for the superior
planets and from the conjunction with the manda equation applied for
the inferior ones (II.57 to 58); the Sun and the nodes have none.

**The Lagna** (III.42 to 50). The rising times of the first three signs
at Lanka are 1670, 1795 and 1935 respirations of the 21 600 in a day
(III.42 to 43), and the quadrants mirror them. At a latitude the
ascensional differences of 30°, 60° and 90° of tropical longitude give
three portions, taken from the first three signs and added to the next
three, the second half of the zodiac mirroring the first (III.44 to
45). The horoscope point after a time from sunrise: the Sun's tropical
place at sunrise, the part of its sign still to rise at that sign's
rate, the following signs in succession, and the remainder at the sign
reached (III.46 to 48); before sunrise the same walk backward; the
meridian point from the hour angle at Lanka's rates (III.49); the time
between two points by the walk inverted (III.50). The sunrise is the
text's own day arc at the place's local mean time, so the Lagna the
tradition computes by hand comes out of the same numbers.

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
`planet_node_deg`, `moon_apogee(at)`, `ayanamsha_deg(at)`,
`declination_deg(tropical)`, `ascensional_difference_deg(latitude,
declination)`, `day_arc(local_mean_midnight, latitude)`,
`local_mean_midnight(at, longitude)`, `rising_times(latitude)`,
`lagna(at, latitude, longitude)`, `describe()`; `RisingTimes::lanka`,
`RisingTimes::at`, `of_sign`, `point_after(from, asu)` and
`asu_between(from, to)`; `Parameters::TEXT`,
`Parameters::with_bija(&Bija)`. The calendar crate implements its
`SolarModel` over this (`calendar-bikram-sambat.md`).
`SiddhantaProvider::text()` (or `new(model)`) implements
`EphemerisProvider`: nine bodies (the seven grahas, the mean node and
the mean apogee) in the text's own frame (geocentric, of date, ecliptic,
sidereal by the text's ayanamsha, geometric), distances as the hypotenuse
on the radius (`DistanceUnit::MeanDistances`), speeds by the text's rule
(`SpeedModel::Rule`), `Astronomy::Classical`; its overrides are the
text's obliquity, its ayanamsha and its sunrise and sunset (the centre
on the geometric horizon; other conventions refused as unsupported). C
ABI and bindings reach the model through the port's vtable the way they
reach an engine.

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
| the Sun's longitude, exact trigonometry | 300 ns | 89 ns |
| the Moon with its latitude, table | 300 ns | 94 ns |
| a star planet (four steps, the motion by rule, the latitude), table | 2 µs | 298 ns |
| all nine grahas, table | 5 µs | 1.65 µs |
| the day's arc at Kathmandu | 1 µs | 754 ns |
| the Lagna at Kathmandu (the day's arc, the rising times, the walk) | 3 µs | 1.10 µs |

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
- Burgess's worked computation for midnight of 1 January 1860 at
  Washington (his notes to I.48 to 53, II.47 to 51, II.56 to 58, III.9 to
  12 and III.42 to 49) reproduces: the day count 1 811 945 (714 404 108
  572 from the creation), the mean Sun 8s 17°48′7″ within 2″, the
  precession 20°24′39″ within 5″, the Moon's mean place, apsis and node of
  his table within 5″, the Moon's anomaly 10s 18°46′15″ (his print reads
  18°4′15″, which his own table contradicts; §10), the Moon's true motion
  737′4″, the Sun's 61′26″, Mars's motion 32′3″ from an equated 27′45″ and
  a hypotenuse of 3984, the Moon's latitude 3°36′ north at 53°14′ from
  the node and Mercury's 2°4′ north; the ascensional differences at
  Washington 578′, 1061′ and 1263′ and the oblique rising times 1312½,
  1733½, 2137½ and 2278½ respirations within 2; the horoscope point 4s
  25° at 6555 respirations after a sunrise with the Sun at 1s 12° within
  0.15°.
- The kit over `SiddhantaProvider` (`tests/kit.rs`) passes: determinism,
  batches, continuity by the second difference, the range and body
  refusals, the text's ayanamsha against Burgess's figure; the
  informational rows measure the text against modern astronomy (§10).
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
   (cruxes register C28). Measured on 2026-09-05 from the national
   panchanga committee's printed places for BS 2082 and 2083
   (`fixtures/official/`): its Sun is the text's without bija within 3″,
   its Moon the text's with `Bija { moon_apsis: -4 }` within 0.5′ at ten
   printed points (`tests/official.rs`); its star planets are modern
   positions, not the text's (C38).
2. **The epicycle convention.** The text places the even-quadrant value
   at the anomaly's 0° and 180° and the odd-quadrant value at 90° and
   270° (II.34 to 38); the baseline engine had them swapped, which moves
   the Sun's greatest equation by 0.05°; the text wins (C27).
3. **The sign of the libration.** III.9 to 12 give a triangular
   libration; the tradition applies it with the sign that makes the
   ayanamsha positive today; the crate follows the tradition and says so.
4. Closed: **II.50 to 51**, **II.56 to 58** and **III.42 to 50** are
   implemented and tested against Burgess's worked example; the star
   planets' speeds are the text's rule, and its distance from the
   derivative is published (C36).
6. **The text against modern astronomy**, measured by the kit over the
   provider and published rather than gated: the obliquity 24° against
   IAU's 23.44° (2065″); the text's sunrise, six hours less the
   ascensional difference in local mean time, against hour-angle
   geometry over the text's own places up to 250 s, the text having no
   equation of time (C37); the speed rule against the derivative up to
   0.23° a day. A chart from this provider is the tradition's, and the
   provenance names it.
7. **Burgess's Moon anomaly.** His worked example under II.47 to 49
   prints the mean anomaly as 10s 18°4′15″; his own table of mean places
   for the same instant gives the apsis 327°50′24″ and the Moon 9°4′9″,
   whose difference is 18°46′15″, which the crate reproduces and the
   tests assert; the printed figure is taken as a misprint.
5. **The tradition's day count** is one more than the text's midnight
   count for the same civil day (the worked example); the Bikram Sambat
   measurement shows the difference is immaterial to the calendar under
   the punya-kala rule.
8. **Chapter IX's own arithmetic.** The heliacal risings and settings
   compute over this provider through `astro::visibility` under the
   text's thresholds (IX.6 to 8, X.1) and its measure in degrees of time,
   but with exact horizon geometry in place of the text's rising-time
   tables (III.42 to 50, which `lagna` has) and its correction of the
   planet's place for latitude at the horizon (VII.8, not built); the
   difference is the text's own approximation, to be measured when VII.8
   is built.
