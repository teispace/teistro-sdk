# Astronomy: eclipses

Status: `draft`, written 2026-10-01 from two prototypes measured
against NASA's Five Millennium Canon before any module existed. It
derives from:

- `01-research/platform/13-astronomy-layer.md`, the eclipses row:
  global circumstances from the Sun and the Moon, lunar eclipses from
  shadow geometry, and one second of contact times;
- `07-roadmap/00-roadmap.md`, Phase 7, Nepal step 3: eclipses as
  blackouts;
- `muhurta.md` §4.1: the `ECLIPSE_STAR` blackout, which waits on this
  page.

The reference is rank 1. Fred Espenak and Jean Meeus's catalogues of
solar and lunar eclipses (NASA TP-2006-214141 and TP-2009-214172, NASA
GSFC, public domain) are read whole for 1900 to 2100. The measured page
is `eclipses-measured.md`.

## 1. Purpose and scope

There are three reasons the SDK needs eclipses:

- **The muhurta needs them.** The eclipse's star, *grahanotpatha*,
  closes that nakshatra for marriage for six months. The *sutak* closes
  the hours before an eclipse that is visible where the rite is held.
- **The daily panchanga prints them.** A Nepali or Indian almanac prints
  the day's eclipse with its contacts.
- **A Western chart reads them.** Both the eclipse before a birth and
  eclipse transits do.

What this page builds:

- every solar and lunar eclipse in a window, with its kind, gamma and
  magnitude, the instant of greatest eclipse, and the nakshatra and sign
  it falls in;
- for a lunar eclipse, its six contacts;
- for a solar eclipse, the place on the Earth where it is greatest.

The local circumstances at a place are §4.5. They are the second half
of this page and are needed for the sutak.

**Out of scope:**

- occultations of stars and planets;
- the Besselian elements as a published product;
- path maps;
- the Surya Siddhanta's own eclipse method (chapters 4 to 6). That is
  a classical astronomy's eclipse and a fork for C188.

## 2. Inputs, settings and ports

An eclipse is read from any `ApparentPositions` source: the frame
completion over a provider, or a classical model. It needs only the Sun
and the Moon, apparent and geocentric, with their distances. Two
choices are the caller's, and each is reported on the answer:

- **`ShadowRule`**: how the Earth's shadow is enlarged by its
  atmosphere.
  - `DANJON`, the default, adds 1/85 of the Earth's radius, which is
    1% of the Moon's parallax.
  - `CHAUVENET` adds 1/50 of the umbra and the penumbra.

  NASA's catalogue and the French almanac use Danjon. The Astronomical
  Almanac used Chauvenet until 2015. Measured against the catalogue,
  Chauvenet's magnitudes are 0.008 larger in the umbra and 0.028 larger
  in the penumbra.
- **The Delta T model.** A search runs in UT1 and its answer is in
  UT1. The TT the catalogue speaks is one conversion away, under the
  model the caller chose.

## 3. The data model

- `LunarKind`: `PENUMBRAL`, `PARTIAL` or `TOTAL`.
- `SolarKind`: `PARTIAL`, `ANNULAR`, `TOTAL` or `HYBRID`. A hybrid is
  annular at the path's ends and total in its middle.
- **`LunarEclipse`**:
  - `greatest`, a UT1 instant;
  - `kind` and `gamma`;
  - `umbral_magnitude` and `penumbral_magnitude`;
  - `contacts`, a `LunarContacts` record. P1 and P4 are always present.
    U1 and U4 exist when the eclipse is at least partial, and U2 and U3
    when it is total. Each is an `Option`, so a penumbral eclipse is not
    given invented umbral contacts;
  - `shadow`, the rule applied.
- **`SolarEclipse`**:
  - `greatest` and `kind`;
  - `gamma` and `magnitude`;
  - `point`: the geodetic latitude and longitude of greatest eclipse,
    where the axis meets the Earth or, for a non-central eclipse, the
    point of the limb nearest it.

**Gamma** is the least distance of the shadow axis from the Earth's
centre, in equatorial radii. It is positive when the axis passes north
of the centre.

- *Solar:* the Moon's shadow axis, measured in the fundamental plane.
- *Lunar:* the Earth's shadow axis, measured from the Moon's centre.

## 4. Algorithms

### 4.1 The geometry at an instant

Each body is placed as a geocentric equatorial vector of date, in Earth
radii. Which place is read is decided by the path the light took, and
the two kinds of eclipse differ (`ShadowSource`, `Seen`):

- **A solar eclipse reads both bodies astrometric**: corrected for light
  time and not for aberration. The shadow reaching the Earth now is
  sunlight that passed the Moon 1.3 seconds ago. That is the Moon's
  astrometric place, and the Sun's astrometric place is the Sun that
  light left. Aberration is an observer's, and the shadow has none.
- **A lunar eclipse reads both bodies apparent.** The shadow the Moon
  stands in was cast past the Earth where the Earth was 1.3 seconds
  before the Moon is seen. Taken from the Earth now, the shadow's axis
  is therefore displaced by the Earth's motion over those seconds. Seen
  from the Earth, that displacement is the Moon's aberration. The Sun's
  aberration is the same angle, v/c, so to the first order both apparent
  places stand where the geometry puts them.

Each placing was measured, not chosen. Each 20.5″ of aberration in the
wrong place moves a lunar eclipse's greatest moment by about 38
seconds:

| Lunar eclipses read | Greatest eclipse against the catalogue, median |
|---|---|
| both apparent | +0.2 s |
| the Moon apparent and the Sun astrometric | +38 s |
| both astrometric | +77 s |

Solar eclipses read astrometric fall within a second.

**What the prototype found first was a defect.** The prototype was
built on the frame completion's apparent place, and two defects were
under it, both found here and fixed with a test proved red first
(`ephemeris-builtin/tests/provider.rs`):

- The completion asked a provider for its own native frame. For the
  built-in ephemeris that is the J2000 equator, geometric. Every apparent
  place it gave was therefore 1.28° of right ascension from the equator
  of date in 1900 and 0.31° in 2024. That is a minute of every sunrise
  read over the built-in, and was the first thing the catalogue
  exposed: central eclipses' longitudes drifted 1.4° a century, centred
  on 2000.
- A request without speeds reached the light-time correction with every
  rate zeroed by the precession step. The Sun stood 20.7″ ahead of
  itself, and an apparent place over the built-in was its geometric
  one.

The prototype's empirical rule, "the Sun a light time earlier", undid
the second defect by accident and was not physics.

The constants are:

| Constant | Value |
|---|---|
| Earth's equatorial radius | 6378.137 km |
| Flattening | 1/298.257 |
| Solar radius | 696 000 km |
| Lunar radius, penumbra and lunar shadows | k = 0.272 507 6 |
| Lunar radius, solar umbra | k = 0.272 281 |

Both k values are the IAU's, and NASA's two catalogues use them the
same way. The umbral k is the mean radius under the valleys of the lunar
limb. Taking the larger k throughout turns four total eclipses into
partial ones and one annular eclipse into a partial one.

### 4.2 Lunar eclipses

The Moon's angular distance d from the antisolar point is measured with
these radii:

- π_m and π_s: the Moon's and the Sun's horizontal parallaxes;
- s_m and s_s: their semidiameters.

The shadow's radii under Danjon's rule are:

- umbra ρ_u = 1.01·π_m − s_s + π_s;
- penumbra ρ_p = 1.01·π_m + s_s + π_s.

Under Chauvenet's rule both are 1.02·(π_m ∓ s_s + π_s).

The two quantities read off them are:

- **Magnitude:** a shadow's magnitude is (ρ + s_m − d) / (2·s_m).
- **Kind:** the eclipse is total when the umbral magnitude reaches 1,
  partial when it is positive, and penumbral when only the penumbral
  magnitude is.

**Greatest eclipse** is the minimum of d. It is found by a
golden-section search over a day either side of a full moon, narrowed
to 10⁻⁷ days.

**Contacts:**

- P1 and P4 are where d = ρ_p + s_m.
- U1 and U4 are where d = ρ_u + s_m.
- U2 and U3 are where d = ρ_u − s_m.

Each contact is the shared boundary solver's root on its side of
greatest eclipse.

**The search:**

- The full moons are seeded from the mean lunation, and each minimum is
  found from its seed.
- A minimum is narrowed coarsely first, and a full moon whose coarse d
  exceeds the penumbral limit by more than the Moon's motion over the
  coarse bracket is dropped. Only a candidate is refined.
- The coarse bracket is 0.01 days, and the Moon moves 0.13 Earth radii
  in it.

### 4.3 Solar eclipses

These quantities are read in the fundamental plane, the plane through
the Earth's centre perpendicular to the Moon's shadow axis:

- **The axis:** the shadow axis runs through the Sun and the Moon.
- **(x, y):** where the axis meets the plane.
- **ρ₁ and the Earth's outline:** the Earth's outline is an ellipse
  with semi-axes 1 and ρ₁ = √(1 − e²·cos²d), where d is the axis's
  declination.
- **The two shadow radii in the plane:**
  - l_u, the umbra's, is positive for a total eclipse and negative for
    an annular one;
  - l_p is the penumbra's.

  Both come from the vertex distances of the two cones.

**Greatest eclipse** is the minimum of √(x² + y²).

**Gamma:**

- Gamma is that minimum, signed by y.
- It is read on the sphere: NASA's gamma does not flatten the Earth.
- It agrees with the catalogue within 0.000 13 on all 454 eclipses.

**Kind and magnitude when the axis meets the ellipsoid:**

- *Kind:* the umbra's radius is read twice, in the plane and at the
  surface point, where the cone is narrower or wider. The kind is
  total if both are positive, annular if both are negative, and hybrid
  if they differ.
- *Magnitude:* the ratio of the Moon's apparent diameter to the Sun's,
  from that point.

**Kind and magnitude when the axis misses the ellipsoid:**

- *Δ:* the distance from (x, y) to the outline, found by Newton's
  method on the ellipse's parameter.
- *Magnitude:* (l_p − Δ) / (l_p − l_u).
- *Kind:* the eclipse is a non-central total or annular when Δ < |l_u|,
  and partial otherwise.

The ellipse is not optional. On a sphere every partial magnitude was
0.005 low. With the Earth squashed along its pole and the distance taken
in the squashed space, July eclipses were up to 0.0028 low. Taken as the
true distance to the outline, they agree within 0.000 17.

### 4.4 Where an eclipse falls

An eclipse's nakshatra and sign are the Moon's at greatest eclipse, in
the caller's zodiac. They are read through the same longitude source a
chart uses, so a sidereal grahanotpatha follows the profile's
ayanamsha. The muhurta reads it from there.

### 4.5 Local circumstances

These are needed for the sutak, which is observed only where the eclipse
is seen.

- **A lunar eclipse** is seen wherever the Moon is above the horizon
  during any contact interval. The local answer is the contacts as
  above, each with the Moon's altitude at the place. A contact is
  visible when the altitude is above the horizon convention's.
- **A solar eclipse** needs the topocentric Moon. Its contacts at a
  place are where the topocentric separation of the Sun's and the
  Moon's centres equals the sum of their semidiameters, for C1 and C4,
  or their difference, for C2 and C3. The local magnitude is at the
  separation's minimum.

  Topocentric places come from `astro::topocentric`, which the rise and
  set solver already uses. The reference is NASA's local circumstances
  for named cities, which the GSFC eclipse pages print for each eclipse.

## 5. The API

```rust
let eclipses = Eclipses::new(&sky, delta_t).with_shadow(ShadowRule::Chauvenet);
let lunar: Vec<LunarEclipse> = eclipses.lunar_between(from, to)?;
let solar: Vec<SolarEclipse> = eclipses.solar_between(from, to)?;
let next = eclipses.next_lunar(from)?;   // the next one, at most a year on
```

The façade's `Context::eclipses()` builds this over the context's
completion and Delta T. The answers cross to every binding as records,
and the kinds are catalogue members: `eclipse_kind`.

## 6. Errors and degenerate states

- **A window that runs backward** is an `InvalidArg` on `to`.
- **A source that cannot answer an instant** passes its error through,
  so an ephemeris's range is the search's.
- **No eclipse in the window** is an empty list, not an error.
  `next_lunar` and `next_solar` look one year ahead and refuse with
  `NotConverged` past that. A year always holds at least two solar eclipses
  and is never without a lunar one under the penumbral definition, so a
  refusal means the source's range ended.
- **A grazing eclipse at the limit** belongs to whichever side the
  arithmetic puts it. The catalogue's 1935 January partial, magnitude
  0.0013, is the narrowest: the prototype read it as a miss (−0.0014),
  and the shipped search finds it. The measured page names any miss by date, so a regression at the
  limit is named rather than counted.

## 7. Performance budget

A coarse narrowing costs about 11 golden steps of three apparent
places. A refinement costs about 35 more, and a contact solve about 20.
Two centuries of syzygies are 4 950 seeds, so a scan of 1900 to 2100
is under three hundred thousand apparent places on the built-in
ephemeris. The measured pass runs the two scans and its three
comparisons on a thread each, about 34 seconds in all on the
maintainer's machine; a caller's year is well under a second.

## 8. Tests

**The measured page.** It reads every eclipse in both catalogues for
1900 to 2100 and gates:

| Quantity | Measured | Gate |
|---|---|---|
| Found, both ways | all 459 lunar and 454 solar | any miss or extra |
| Kind | all agree | any disagreement |
| Greatest eclipse | within 2.9 s | 5 s |
| Gamma | within 0.00014 | 0.0005 |
| Magnitudes | within 0.0003 | 0.001 |
| Lunar phase durations | within 0.61 min | 1 min |
| Point of greatest eclipse | within 0.31° | 0.5° |

The page's own tables are the authority; these figures are its run of
2026-10-01.

Two checks run both ways. Every catalogue eclipse is found by the
search, and every eclipse the search finds is in the catalogue. The
second is how a false positive, a full moon read as a penumbral
eclipse, would be caught.

**Unit tests** (`crates/ephemeris-builtin/tests/eclipses.rs`, and the
geometry's own in `crates/astro/src/eclipse.rs`):

- a known pair: the total lunar eclipse of 2025-09-07 and the total
  solar eclipse of 2024-04-08, each with its kind and its time to the
  second;
- the shadow rule's effect on one penumbral eclipse;
- a window with no eclipse;
- the error for a backward window;
- the next lunar eclipse after a known one.

**Bindings:** the parity runner over the 2025 to 2026 eclipses.

## 9. Localisation

The kinds are catalogue members and are named in each locale's pack:
*grahan*, *khagras*, *khandagras* and *kankanakriti* in Nepali and
Hindi.

## 10. Open questions

- **C188:** whether a classical profile's eclipse should be the text's
  own (Surya Siddhanta IV to VI). The Nepali print's eclipse times have
  not yet been compared.
- **The sutak's hours:** nine for a lunar eclipse and twelve for a
  solar one, and whether a penumbral eclipse has one. These are the
  muhurta page's question, and depend on its source.
