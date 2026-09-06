# Changelog

Every release answers one question first, because it is the only one an
astrology engine's consumer actually needs:

> **Does this move any number, and by how much?**

A chart computed with the previous version and stored somewhere is a fact
someone may still be looking at. So each entry begins with **Numbers**, and
"none" is an answer that has to be earned by the conformance run against
the previous release, not by nobody having looked.

## Unreleased

**Numbers:** the Bikram Sambat table's computed rows moved. Every year
outside the official span (BS 1970 to 2095) is now computed by the SDK's
own engine (the Surya Siddhanta as the text prints it, Nepal's clock,
Kathmandu, the punya-kala rule) and the table runs from 1700 to 2500 BS;
the earlier rows for 1856 to 1969 and 2096 to 2457 were the baseline
engine's projections and differ from these by a day at some month
boundaries, and 2096 to 2100 are computed, no longer marked official.
Inside the official span no date moved; eleven boundaries there now
report `Divergent`. Sunrise and sunset from modern positions compute for
the first time (the rise and set solver); Delta T's values are
unchanged by their move from `time` to `astro`. The Surya Siddhanta's
star planets now report the text's daily motion (II.50 to 51) instead of
a central difference of the text's places, up to 0.23° a day apart for
Mars; their longitudes did not move. The text's latitudes, its Lagna and
the planetary hours compute for the first time. Crossings and stations
compute for the first time: sign ingresses with their retrograde
re-entries, the tithi, nakshatra, yoga and karana boundaries, composite
angles and single targets, and the stations, over any source of
longitudes. The boundary solver's narrowing moved from bisection to the
ITP method: every searched instant (a sunrise, a sankranti, a boundary)
is still the middle of a bracket no wider than its tolerance, so an
instant may differ from the previous release's by up to that tolerance
(under a hundredth of a second for the sankranti and the sunrise), in
about a quarter of the evaluations. The twelve star-anchored ayanamshas
(True Chitra, True Revati, True Pushya, True Mula, Sheoran, the four
Galactic Centre and the three Galactic Equator members) compute for the
first time, over the star table's SIMBAD astrometry (Hipparcos, the new
reduction, and Gaia DR3) and the SDK's own Earth ephemeris, so every
provider gives the same sidereal longitudes under them; the star table
itself (128 catalogued members) has places for the first time. The
planetary phenomena compute for the first time: elongation, phase angle
and illuminated fraction, apparent disc and horizontal parallax, and the
visual magnitude under the Astronomical Almanac's models (Mallama and
Hilton 2018 for the planets); so does the equation of time. A provider
may now answer a crossing search itself (the `CROSSINGS` override, with
its vtable slot and two kit checks): under `PREFER_NATIVE` Teimeris's own
search answers within 0.004 s of the SDK's kernel, so no instant moves
beyond that. The kit's Surya Siddhanta ayanamsha expectation for a modern
engine is now the catalogued epoch definition (18.94° at Burgess's 1860
instant), the text's own value being the classical astronomy's alone.
The Greenwich apparent sidereal time is now the IAU 2006 expression with
the IAU 2000B nutation (`gst06b`) instead of the IAU 2000 one (`gst00b`):
the meridian, every house cusp and the equation of time move by under
0.002″ (6e-7°) between 1950 and 2050 and by 0.01″ at 1850, and
Teimeris's sidereal time inside its window is matched within 0.0012″.
Visibility and the heliacal phenomena compute for the first time
(`astro::visibility`): the state of a body near the Sun on a day and the
days it appears and disappears, under the Surya Siddhanta's degrees of
time, the tradition's combustion orb or Ptolemy's arcus visionis, each
named in the call. The local mean midnight helper moved from the
classical crate to `astro::sky` unchanged. A composite quantity's two
bodies (the tithi, the yoga, an aspect) and a visibility reading's body
and Sun are now read in one position request: no instant moves, and the
tithi search costs 45 % less measured against its previous form in the
same run. Teistro Intl computes for the first time (`teistro-intl`): the
stable `MessageFormat 2` grammar with the SDK's functions, validation
with the catalogue as the authority for entity keys, `.tpack` packs and
`.tbundle` locale bundles, typed accessors for TypeScript, Dart and Rust,
and the `teistro-intl` command line; the SDK's `i18n/` ships `en-Latn`
and `ne-Deva-NP` with the entity records of the grahas, the signs, the
nakshatras and the Lagna. The engine's runtime API: a pack or bundle
loaded after construction, in-memory overrides, and the report of what is
loaded and covered. The date functions: `:date`, `:time`, `:datetime`,
`:ghati` and `:duration`, calendar-aware over the calendar crate, with
the patterns and names a locale declares in `sdk.calendar` (shipped for
`en-Latn` and `ne-Deva-NP`), era records for the nine eras, and the
`useGrouping` and `minimumIntegerDigits` options on numbers. The
baseline engine's entity name tables are imported (`teistro-intl migrate
baseline`): 274 records in each of four languages, `hi-Deva-IN` and
`sa-Deva` joining the shipped locales at `base` completeness. The C ABI
computes for the first time (`teistro-ffi`, `bindings/c/include/teistro.h`):
contexts from a profile, a JSON settings patch, a locale and the port's
vtable; the last error with its message, field and hint; keys and ids;
dates in every shipped calendar; civil times to instants with the zone
metadata and the scale conversions; the locale engine over the embedded
bundles; the frame a request asks for by name (centre, equinox,
coordinates, zodiac, corrections) rather than by its packed bits; and
positions through the port completed into that frame, as a result blob
(`TSRB`) with the completion steps and the provenance envelope. The C
binding's own test compiles against the header with warnings as errors
and runs (`cargo xtask check-c`). The API description (`idl/api.json`, `teistro-idl`) is extracted
from the boundary crates' source and gated. **The settings hash moved
for any build that enabled the JSON layer's `preserve_order` feature**:
the canonical document's keys are now sorted by the SDK itself, so the
hash of a settings document is the same in every build (a crate compiled
alone and the workspace hashed the same settings differently before);
the astronomical numbers do not move. Nothing else computes yet.

- Project founded: research, architecture, decisions, roadmap and the
  open-source scaffolding. See `docs/STATUS.md`.
- `crates/core`, `crates/calendar` (the arithmetic calendars and Bikram
  Sambat), `crates/siddhanta` (the Surya Siddhanta model), the seed of
  `crates/astro` (the boundary solver), and the Bikram Sambat engine with
  its measurement (`docs/calendars/bikram-sambat.md`).
- `crates/time` and `crates/port-timezone`: time scales with Delta T as
  the IERS table (1956 to the present) then Espenak and Meeus (2006)
  with Morrison and Stephenson's uncertainties, the IANA leap-second
  table, civil time, zone resolution over the embedded tzdb with the
  metadata a stored chart replays, local mean time, the sunrise-anchored
  day with the polar policies, ghati-pala. Every zone resolution of the
  55 fixture charts reproduces the baseline's instant and metadata.
- `crates/port-ephemeris` (spike 3's port promoted, with the rise and
  set override), `crates/astro` (Delta T moved here from `time`; the
  IAU routines ported from ERFA with a provenance table; sidereal time
  and the obliquity; frame completion over the port; the rise and set
  solver under the sunrise conventions, with polar days reported),
  `crates/ephemeris-kit` (the conformance kit: fifteen checks, both
  engines passing), the drik solar model for the calendars, the local
  day's convention and the `SUNRISE` unknown-time fallback in `time`,
  and the adapters under `adapters/`. Measured: the geometric sunrise
  agrees with Teimeris's own search within 0.13 s; the refracted one
  within 2.5 s of the baseline's fixtures below 60° of latitude (the
  refraction convention, cruxes C34). The committee's stated method for
  Bikram Sambat (the Surya Siddhanta) recorded in the memo, and modern
  positions measured at 65 % of the official months against the text's
  98.5 %.
- `crates/siddhanta` completed to the text: the sighra daily motion
  (II.50 to 51), the latitudes (II.56 to 58) and the Lagna from the
  oblique ascensions (III.42 to 50), each reproduced against Burgess's
  worked computation for 1 January 1860; `SiddhantaProvider` presents the
  model behind the ephemeris port as a classical astronomy and passes
  the kit, whose report publishes the text's distance from modern
  astronomy (the obliquity, the sunrise, the speed rule) instead of
  gating it. The port gained `Astronomy`, `SpeedModel`, `DistanceUnit`
  and the `DUT1` override; the completion orders the zodiac shift and
  the rotation so a sidereal ecliptic provider completes to equatorial
  tropical coordinates. `crates/time` gained the planetary hours under
  the `hora_reckoning` knob (proportional by default, as the baseline's
  fixtures decide) and UT1 from a provider's DUT1.
- `fixtures/official/npns-2082-2083.json`: the national panchanga
  committee's published panchangas for BS 2082 and 2083 read into data
  (24 sankranti instants, printed places, sunrise and sunset, tithi
  ends), with tests that the SDK's engine reproduces every instant
  within 1.6 minutes and every month start, that the committee's Sun is
  the text's within 3″ and its Moon the text's with a bija of four
  revolutions fewer on the apsis, and that its star planets are modern
  positions (`docs/calendars/bikram-sambat.md`, R2; cruxes C38, C39). No
  computed number moved.
- `crates/astro`: precession as a catalogue of models (Vondrák 2011 the
  default, IAU 2006, IAU 1976, Newcomb) over new ERFA ports (the IAU 2006
  angles and matrices, the long-term poles and matrices, the vector
  primitives, each against ERFA's reference values) and Vondrák's own
  obliquity series; the ayanamsha catalogue, every epoch-defined and
  frame member computed from its published definition with the
  fitted-model correction, mean or nutated, custom definitions linear;
  the frame completion now completes a sidereal zodiac from the SDK's
  catalogue when the provider declares no override, so `sdk-only`
  sidereal charts compute for the first time. Against Teimeris's
  recorded values (`fixtures/teimeris/ayanamsha.json`, 1044 rows) the
  definitions stated in TT agree within 1e-7″ and those in Universal
  Time within 2.1e-4″. The twelve star-anchored members are refused by
  name until the star table (`docs/03-design/astro-ayanamsha-catalogue.md`).
- `crates/astro`: the twenty-two catalogued house systems (`houses`) as
  one construction with the circles each system picks, the auxiliary
  points (vertex, equatorial ascendant, the co-ascendants, the polar
  ascendant), the sign-based systems in the zodiac in use, and the four
  polar policies with the outcome reported. Measured within 5e-6° of
  Teimeris over 25 194 cusps and angles at ten latitudes
  (`fixtures/teimeris/houses.json`) and within 0.0002° of the baseline's
  55 charts between 1800 and 2200 (0.0033° beyond, the engines' long-term
  sidereal time). Houses compute for the first time
  (`docs/03-design/astro-house-systems.md`).
