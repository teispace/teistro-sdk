# Astronomy: eclipses

Status: `built`, written 2026-10-01 from two prototypes measured
against NASA's Five Millennium Canon before any module existed. Every
part of it is built: the global search and the local circumstances
(`astro::eclipse`), the almanac's `sdk.almanac().eclipses` and the
section in every binding, measured in `eclipses-measured.md`. C188
stays open (§10). It derives from:

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
is seen. Built as `Eclipses::solar_seen` and `Eclipses::lunar_seen`
(`crates/astro/src/eclipse/local.rs`).

- **A solar eclipse** is read from the place: the eclipse's own Sun and
  Moon (astrometric, as in §4.1) less the station's geocentric position
  (`sky::observer`, the place's height included), in the true equator of
  date. The maximum is the greatest magnitude, which is not the least
  separation of the centres: the discs' topocentric sizes change as the
  bodies climb, and for a shallow eclipse the two instants are seconds
  apart (measured: 14 s at the worst before the search was changed). The
  first and fourth contacts are where the separation equals the sum of
  the semidiameters (the Moon's at k = 0.2725076); the second and third,
  for a central eclipse, where it equals their difference (k = 0.272281).
  The magnitude is the fraction of the Sun's diameter covered, and inside
  a central phase the ratio of the diameters, as NASA prints it; the
  obscuration is the covered fraction of the disc's area.
- **A lunar eclipse**'s contacts are the same instants everywhere; the
  place adds the Moon's topocentric altitude at each.
- **Seen.** Every contact is geometric, the Earth taken as transparent,
  so a contact below the horizon is reported with its negative altitude
  and a place on the night side can have all four. `seen` is the stretch
  of the eclipse the body stands above a `Horizon` convention, read the
  way a sunrise is (`rise_set::centre_altitude_deg`, less the parallax
  since the altitude is already topocentric): `None`
  is the place not seeing the eclipse, which is the sutak's question.
  It is the hull from the first instant up to the last, which differs
  from the truth only for a body that sets and rises again inside one
  eclipse. A lunar eclipse also gives `umbral_seen`, the same stretch
  between the umbra's first and last touch. That is the part the eye
  sees, and the observances count by it: *Dharmasindhu* holds an
  eclipse's time to last only as long as it can be seen
  ("चाक्षुषदर्शनयोग्य"). A penumbral eclipse has none.

**Measured** (`eclipses-measured.md` §6): against the 214 cities of
NASA's bulletins for 2009 July 22 and 2010 January 15, Kathmandu among
them, every contact the bulletin prints is found and every one it omits
is below the horizon here; the contacts agree within 2.6 s, the maxima
within 6.9 s (the flat tops of shallow eclipses) and the magnitudes
within 0.0009.

## 5. The API

```rust
let eclipses = Eclipses::new(&sky, delta_t).with_shadow(ShadowRule::Chauvenet);
let lunar: Vec<LunarEclipse> = eclipses.lunar_between(from, to)?;
let solar: Vec<SolarEclipse> = eclipses.solar_between(from, to)?;
let next = eclipses.next_lunar(from)?;   // the next one, at most a year on
```

A place reads each one through `solar_seen` and `lunar_seen`, or a
window's worth at once through `here_between(from, to, place, &horizon)`,
which answers `EclipsesHere { lunar, solar }`: each eclipse beside its
`SolarView` or `LunarView` (§4.5).

The façade asks it of an almanac's days:

```rust
let found: Envelope<EclipsesHere> =
    sdk.almanac().eclipses(&from, &to, &place, offset)?;
let seen = found.value.any_seen();
```

- **The window** runs from the local midnight that opens the first day
  to the one that closes the last, on the almanac's clock, so an
  eclipse belongs to the civil day its greatest moment falls in. The
  provenance names it as the convention `eclipse.window`.
- **The horizon** is the setting `panchanga.eclipse_horizon`, by default
  the **eye's**: the upper limb, lifted by standard refraction. The first
  design took the almanac's sunrise convention, so that a body "up" for
  an eclipse would be up by the same rule as the day. Measurement
  overturned it (C194). *Dharmasindhu* counts an eclipse only while the
  eye can see it, and the eye sees the refracted limb. Nepal's committee
  timed the 2026-03-03 vedha from a moonrise at 18:03. The upper limb
  with refraction rises at 18:03.7 over the built-in sky, the disc's
  centre with refraction at 18:05.0, and the centre without it at
  18:07.7. A consumer who wants the day's convention sets the same value
  on both settings.
- **The shadow** is the setting `panchanga.eclipse_shadow`, `DANJON` by
  default, and it is part of the settings hash.
- **A classical sky** (the Surya Siddhanta's) refuses with `Unsupported`
  naming C188, rather than answering a modern eclipse over a text's day.

Every binding asks for it beside the days, as it asks for the lunar
years: `eclipses: true` in Node and Dart, `eclipses=True` in Python, the
`eclipses` argument of Java's `AlmanacArea.of`, the bit
`TS_PANCHANGA_ECLIPSES` across the boundary, and the panchanga
blob's section 24 carrying the envelope as JSON. The JSON is camelCase,
and every instant in it is a UT1 Julian day.

The kinds are catalogue members, `lunar_eclipse_kind` (68) and
`solar_eclipse_kind` (69), with Espenak and Meeus's types. The section
writes them as full keys (`lunar_eclipse_kind.TOTAL`) from one table,
`EclipsesHere::MEMBERS`, as the lunar years write theirs. A test holds
the table to serde both ways: every string leaf of a month with a view
of each kind is a listed member or the shadow rule, and every listed
path is reached. So each binding reads a kind as it reads any member: a
union of full keys in TypeScript, the generated enums in Python and
Dart. A place's view of a solar eclipse is never `HYBRID`, which
TypeScript says by excluding it. The shadow rule is a setting's value
and crosses bare (`DANJON`), as a lunar year's `count` does.

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

**Bindings:** each binding's suite reads September 2025 at Kathmandu: a
total lunar eclipse seen whole, a partial solar one not seen, and the
shadow knob moving the umbra. The parity gate prints every field of
both eclipses in every runner. The wasm runner carries the built-in
sky's compact tier, so its eclipse lines are held to the same *shape*,
every kind, count and absent contact, and their numbers are its tier's
own. That exception fails both ways (`xtask/src/parity.rs`,
`WASM_TIER`). The boundary's test holds the section to the façade's
answer, byte for byte.

## 9. Localisation

The kinds are catalogue members (§5), so `sdk.entity` names them. A
strict locale names every member the base locale names, so a member is
named in English only when its Nepali is vetted too
(`entity-names.md` §4b). The Nepal Panchanga Decision Committee prints
an eclipse seen in Nepal as खग्रास or खण्डग्रास, so `PARTIAL` and `TOTAL`
are named in both kinds. Its 2083 almanac prints no eclipse Nepal does
not see, so it gives no word for `PENUMBRAL`, `ANNULAR` or `HYBRID`,
and those three stay unnamed until a source does.

## 10. Open questions

- **C188:** whether a classical profile's eclipse should be the text's
  own (Surya Siddhanta IV to VI). The Nepali print's eclipse times have
  not yet been compared.
- **The sutak's hours**, answered by the muhurta page (§4.1.1, C192):
  `panchanga.eclipse_vedha` counts them *Dharmasindhu*'s way or Nepal's
  committee's fixed hours, and a lunar eclipse is seen by its umbral
  phase, so a penumbral one has none.
