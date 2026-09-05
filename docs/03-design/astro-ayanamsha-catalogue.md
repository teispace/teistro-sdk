# Astronomy: the ayanamsha catalogue

Status: `draft`, written 2026-09-05 when `astro::ayanamsha` and
`astro::precession` were built and the frame completion began to
complete the sidereal zodiac from the SDK's own catalogue. Derives from
`01-research/platform/13-astronomy-layer.md` (the ayanamsha and star
table rows and their conformance targets), `03-design/core-types-and-catalogue.md`
(the forty-seven catalogued members and their categories),
`03-design/settings-and-profiles.md` (the `frame.ayanamsha` and
`frame.ayanamsha_basis` knobs), `03-design/ephemeris-port-and-adapters.md`
(the `AYANAMSHA` override and the completion's zodiac step) and
`03-design/astro-timescales-and-frames.md` (the precession models the
values are carried by). The Swiss Ephemeris documentation describes
each definition and Teimeris carries the same table (rank 2,
`CLEAN_ROOM.md`); the authors' own publications are the rank-1 sources
named per row, and Teimeris's recorded values are the oracle.

## 1. Purpose and scope

A sidereal longitude is a tropical one less the ayanamsha, and there is
no single ayanamsha: forty-seven are catalogued, each an author's
answer to where the sidereal zodiac begins. This page settles how the
SDK computes them itself, so that a sidereal chart needs no provider
override and every provider gives the same sidereal longitudes: the
definition of each member (an epoch and the value there, a frame, or an
anchor in the sky), the construction that carries a value to any date,
the correction for the precession model a constant was fitted with,
mean against nutated values, custom definitions, and the measurement
against Teimeris. The twelve members anchored to a star or the galactic
centre are defined here and refused until the star table exists
(§10).

## 2. Inputs, settings and ports

An instant in TT (a UT1 instant goes through the Delta T model first);
the `frame.ayanamsha` knob (`AyanamshaChoice`: a catalogued member or a
custom epoch, value and rate); the `frame.ayanamsha_basis` knob (`MEAN`,
the value a sidereal longitude subtracts, or `TRUE`, with the nutation
in longitude added); the precession model (`PrecessionModel`, Vondrák
2011 by default) and the Delta T model for epochs stated in Universal
Time. No port: the computation is the SDK's own. Under the ephemeris
port the completion asks a provider that declares the `AYANAMSHA`
override for its value when the policy allows, and computes the SDK's
otherwise (`ephemeris-port-and-adapters.md`, §5).

## 3. The data model

```rust
pub enum Definition {
    Epoch(Epoch),                                   // a value at an epoch carried by precession
    Frame(Epoch),                                   // J2000, J1900, B1950, Mardyks: a frame, whose value is the precession since
    Object { anchor: &'static str, fixed_deg: f64 },  // a star or the galactic centre held at a sidereal longitude
    Unsourced,                                      // a member this build has no definition for
}
pub struct Epoch { jd: f64, scale: EpochScale /* Tt | Ut */, value_deg: f64, fitted: Fitted /* Current | Iau1976 | Newcomb */ }
pub use teistro_core::settings::AyanamshaBasis as Basis;   // Mean | True
```

The forty-seven members and their definitions, degrees at the epoch, the
epoch as a Julian day in the scale the author stated it in (TT unless
marked UT, where the SDK applies its Delta T of the epoch):

| member | definition | fitted with | source |
|---|---|---|---|
| `FAGAN_BRADLEY` | 24.042044444° at B1950.0 (JD 2433282.42345905): the synetic vernal point | Newcomb | Fagan and Bradley, 1950 |
| `LAHIRI` | 23.250182778° − 0.004658035° at JD 2435553.5 (1956-03-21): the Indian Astronomical Ephemeris's 23°15′00″.658 less the nutation of the epoch, so the definition is a mean value | IAU 1976 | Calendar Reform Committee, 1955; the Indian Astronomical Ephemeris |
| `DELUCE` | 0° at JD 1721057.5 UT (the epoch of the Christian era) | | De Luce |
| `RAMAN` | 360° − 338.98556° at J1900 (21°00′52″) | Newcomb | B. V. Raman |
| `USHASHASHI` | 360° − 341.33904° at J1900 (18°39′39″) | | Usha and Shashi |
| `KRISHNAMURTI` | 360° − 337.636111° at J1900 (22°21′50″) | Newcomb | K. S. Krishnamurti |
| `DJWHAL_KHUL` | 360° − 333.0369024° at J1900 | | |
| `YUKTESHWAR` | 360° − 338.917778° at J1900 | | Sri Yukteshwar |
| `JN_BHASIN` | 360° − 338.634444° at J1900 | | J. N. Bhasin |
| `BABYL_KUGLER1`, `2`, `3` | −5.66667°, −4.26667°, −3.41667° at JD 1684532.5 UT | | Kugler's three Babylonian zodiacs |
| `BABYL_HUBER` | −4.46667° at JD 1684532.5 UT | | Huber, 1958 |
| `BABYL_ETPSC` | −5.079167° at JD 1673941 UT (η Piscium at 0° Aries) | | |
| `ALDEBARAN_15TAU` | −4.44138598° at JD 1684532.5 UT (Aldebaran at 15° Taurus) | | |
| `HIPPARCHOS` | −9.33333° at JD 1674484.0 UT | | Hipparchos |
| `SASSANIAN` | 0° at JD 1927135.8747793 UT | | the Sassanian zodiac |
| `J2000`, `J1900`, `B1950` | frames: 0° at J2000.0, J1900 (JD 2415020), B1950.0 | | the standard equinoxes |
| `SURYASIDDHANTA` | 0° at JD 1903396.8128654 UT: the instant the text's true Sun was at 0° Mesha in 499 CE | | the Surya Siddhanta (`siddhanta.md`) |
| `SURYASIDDHANTA_MSUN` | −0.21463395° at the same epoch (the mean Sun) | | |
| `ARYABHATA` | 0° at JD 1903396.7895321 UT (Aryabhata's true Sun) | | the Aryabhatiya |
| `ARYABHATA_MSUN` | −0.23763238° at the same epoch | | |
| `SS_REVATI` | −0.79167046° at JD 1903396.8128654 UT (ζ Piscium at 359°50′ by the text's star table) | | the Surya Siddhanta, chapter VIII |
| `SS_CITRA` | 2.11070444° at the same epoch (Spica at 180°) | | |
| `TRUE_CHITRA` | Spica held at 180° | | |
| `TRUE_REVATI` | ζ Piscium held at 359°50′ | | |
| `TRUE_PUSHYA` | δ Cancri held at 106° | | P. V. R. Narasimha Rao |
| `GALCENT_0SAG` | the galactic centre (Sgr A*) held at 240° | | |
| `GALCENT_RGILBRAND` | Sgr A* held at 210° + 90° × 0.3819660113 (the golden section) | | R. Gil Brand |
| `GALEQU_IAU1958`, `GALEQU_TRUE` | the galactic pole (the IAU 1958 definition, or the true pole) projected to 150° | | |
| `GALEQU_MULA` | the galactic pole at 150° + 6°40′ (mid-Mula) | | |
| `GALALIGN_MARDYKS` | a frame: 30° at JD 2451079.734892 | | Mardyks |
| `TRUE_MULA` | λ Scorpii held at 240° | | Chandra Hari |
| `GALCENT_MULA_WILHELM` | Sgr A* held at 246°40′, by right ascension | | Wilhelm |
| `ARYABHATA_522` | 0° at JD 1911797.740782065 UT (522 CE) | | |
| `BABYL_BRITTON` | −3.2° at JD 1721057.5 UT | | Britton, 2010 |
| `TRUE_SHEORAN` | δ Cancri held at 103.49264221625° | | Sheoran |
| `GALCENT_COCHRANE` | Sgr A* held at 270° | | Cochrane |
| `GALEQU_FIORENZA` | 25° at JD 2451544.5 UT | | Fiorenza |
| `VALENS_MOON` | −2.9422° at JD 1775845.5 UT | | Vettius Valens |
| `LAHIRI_1940` | 22.44597222° at J1900 | Newcomb | Lahiri's 1940 value |
| `LAHIRI_VP285` | 0° at JD 1825235.2458513028 (the vernal point of 285 CE) | | |
| `KRISHNAMURTI_VP291` | 0° at JD 1827424.752255678 (291 CE) | | Krishnamurti and Senthilathiban |
| `LAHIRI_ICRC` | 23.25° − 0.00464207° at JD 2435553.5 | Newcomb | the Indian Calendar Reform Committee's value |

A `Frame` member's value is the general precession since its epoch;
positions referred to the ecliptic of the epoch itself are a completion
step (`astro-timescales-and-frames.md`), not an angle, and the SDK's
sidereal zodiac is always the ecliptic of date. The provenance of a
sidereal result names the ayanamsha, its basis and the precession model
(`Frame` bits 16 to 31 carry the catalogue id; the settings hash carries
the rest).

## 4. Algorithms

**An epoch-defined value at a date** (`mean_deg`). The construction every
published table of these values was computed from: the vernal point of
the date, a unit vector along x in the mean equatorial frame of date, is
carried back to J2000.0 by the precession model in use and on to the
definition's epoch, rotated into the ecliptic of the epoch by the
model's own mean obliquity there, and read as a longitude; the
ayanamsha is minus that longitude plus the value at the epoch. The
angle is measured on the ecliptic of the epoch and subtracted from a
longitude measured on the ecliptic of date, which the Swiss
documentation itself calls illogical and which is nevertheless what the
tables were built from, so it is the definition. An epoch stated in
Universal Time is moved to TT by the SDK's Delta T of the epoch first.

**The fitted-model correction.** Fagan and Bradley's constant was fitted
with Newcomb's precession, Lahiri's with IAU 1976's, and several others
likewise; using them under another model would shift every sidereal
longitude. The correction is the residual longitude of the epoch's
vernal point carried to J2000.0 by the model in use and back by the
fitted model, read on the epoch's ecliptic, folded to a signed
quantity, and subtracted. Under the fitted model itself it vanishes.

**Mean and true** (`value_deg`). The mean value is what a sidereal
longitude subtracts from a tropical longitude of the mean equinox; the
true value adds the IAU 2000B nutation in longitude of the date, about
±17″. The completion subtracts the mean value from apparent (true
equinox) longitudes, as the engines do, and the true basis is for
reporting.

**A custom definition.** `AyanamshaChoice::Custom { epoch_jd_tt,
value_deg, rate_deg_per_year }` is linear: the value plus the rate times
the Julian years since the epoch. A custom value carried by precession
instead is an `Epoch` definition through the Rust API.

**The rate** (`speed_deg_per_day`). A central difference of the mean
value over a day: the general precession, about 50.29″ a year.

**Anchored definitions.** The anchor's mean longitude of date (its
apparent place without nutation, so the mean ayanamsha is the mean one)
less its fixed sidereal longitude, the galactic pole by right ascension
projected to the ecliptic; refused until the star table exists.

## 5. The API

Rust: `ayanamsha::definition(id) -> Definition`, `is_computable(id)`,
`mean_deg(&choice, tt, model, delta_t)`, `value_deg(&choice, tt, basis,
model, delta_t)`, `speed_deg_per_day(&choice, tt, model, delta_t)`;
`precession::{PrecessionModel, matrix, to_date, to_j2000, between,
mean_obliquity_rad, mean_obliquity_deg}`; the completion takes the model
through `Completion::with_precession` and completes a
`Zodiac::Sidereal { ayanamsha }` frame from the catalogue under
`sdk-only`, or when the provider declares no override under
`prefer-native`. C ABI and bindings arrive with the chart layer:
`ts_ayanamsha(choice, jd_tt, basis, model)` returning the value and its
rate, and the definition table as data.

## 6. Errors and degenerate states

| situation | outcome |
|---|---|
| a member anchored to a star or the galactic centre | `UNSUPPORTED`, naming the anchor and suggesting an epoch-defined member or a provider with the override, field `frame.ayanamsha` |
| a member the catalogue registers without a definition here | `UNSUPPORTED (unsourced)`, field `frame.ayanamsha` |
| a custom definition with a non-finite epoch, value or rate | `INVALID_ARG`, field `frame.ayanamsha` (the settings coherence rule `custom-ayanamsha-finite` catches it first) |
| an epoch in Universal Time the Delta T model cannot answer for | the Delta T error |
| the value passing through 0° (every definition does, a few centuries CE) | normalised into `[0, 360)`; the rate is a difference of normalised values folded to ±180°, so no definition reports a rate of 360 000° a day at the crossing |

## 7. Performance budget

| operation | budget | measured (`cargo bench -p teistro-astro`, Apple M-series, 2026-09-05) |
|---|---:|---:|
| the precession matrix, Vondrák 2011 (the two pole series, twenty-two periodic terms) | 2 µs | 142 ns |
| the precession matrix, IAU 2006 | 1 µs | 36 ns |
| a mean ayanamsha (Lahiri), Vondrák 2011: two matrices and the obliquity series | 5 µs | 0.58 µs |
| a mean ayanamsha with the fitted-model correction (Fagan and Bradley): four matrices | 8 µs | 0.59 µs |

No allocation. A chart computes the ayanamsha once per instant; the
completion computes it once per requested instant.

## 8. Tests

- The ERFA ports reproduce ERFA's own reference values: `p06e`, `pfw06`,
  `fw2m`, `pmat06`, `bp06`, `bi00`, `ltpecl`, `ltpequ`, `ltp`, `ltpb`
  and the vector primitives, to the tolerances the reference program
  uses; Vondrák's obliquity series gives the IAU 2006 obliquity at
  J2000.0 to a microarcsecond and tracks it within 0.05″ a millennium
  either side; every precession model is the identity at J2000.0,
  orthogonal elsewhere, and gives 1.396° of general precession over a
  century.
- The published values at J2000.0 reproduce (Lahiri 23.857°, Raman
  22.41°, Krishnamurti 23.76°, Fagan and Bradley 24.74°); a definition's
  value at its own epoch is the published constant; the frames give zero
  at their epoch and the precession since; the nutated value differs
  from the mean by −13.9″ at J2000.0; the rate is 50.29″ a year; a
  custom definition is linear; the refusals are named.
- **Against Teimeris** (`tests/teimeris_ayanamsha.rs` over
  `fixtures/teimeris/ayanamsha.json`, 1044 rows: every epoch-defined and
  frame member the engine offers at 24 Julian epochs from −700 to 2500):
  the definitions stated in TT agree within 1e-7″ (the rounding of the
  same construction, bit-identical in most rows), and those stated in
  Universal Time within 2.1e-4″ (the SDK's Delta T against the engine's
  at epochs in antiquity, worst at 499 CE); bounds 1e-5″ and 5e-4″.
- The completion: a sidereal request over the test provider, which
  declares no override, is completed by the catalogue with the step
  stamped `SDK`; a star-anchored request is refused by name.

## 9. Localisation

None: the members are catalogue keys; their names are the locale packs'.

## 10. Open questions

1. **The star table.** The twelve anchored members need the anchors'
   ICRS positions and proper motions (Hipparcos and Gaia, open data), the
   IAU 2000A or 2000B nutation and the aberration from the Earth's
   velocity (ERFA's `epv00`, or the provider's Sun); Phase 2's star
   table brings them, and their target is 0.001″ against Teimeris.
2. **The frame members' projection.** Positions referred to the
   ecliptic of J2000, J1900, B1950 or Mardyks's epoch are a completion
   step (the `equinox` step of `astro-timescales-and-frames.md`); until
   it exists the SDK gives these members' values on the ecliptic of date.
3. **A precessed custom definition** is expressible in Rust as an
   `Epoch`; whether the settings knob should carry one beside the linear
   form waits for a consumer who needs it.
4. **Delta T at ancient epochs.** The Universal Time epochs differ from
   Teimeris's by the two Delta T models' disagreement in antiquity, a
   few seconds and 2.1e-4″ of ayanamsha; both are inside every stated
   tolerance and the SDK's Delta T carries its uncertainty.
