# Ephemeris abstraction

Status: `accepted`, revised 2026-09-04 after Q7, Q8 and Q15 were decided.
Feeds `02-architecture/02-ephemeris-port.md`, `13-astronomy-layer.md` and
`14-builtin-ephemeris.md`. Sources: the baseline engine's `AstronomicalBackend`
interface, Teimeris's public headers and `PLAN_MIGRATION.md`, Swiss
Ephemeris's API, jyotishganit's Skyfield backend, Astronomy Engine.

## The decision that reshaped this page

Q8: everything is in the SDK from v1. The SDK is the master key, not a
starter project. Consequently the ephemeris port's **required** surface
shrinks to the one thing only an ephemeris can supply, the positions of
bodies, and the SDK owns every astronomical computation above that: time
scales, precession, nutation, obliquity, sidereal time, coordinate
transforms, apparent-position corrections, topocentric parallax, the whole
ayanamsha catalogue, every house system, rise and set, crossings and
stations, and later eclipses and fixed stars (`13-astronomy-layer.md`).

Q7: the SDK also ships its own built-in ephemeris (`14-builtin-ephemeris.md`)
so that it works with no provider at all. It is a module that implements
the same port, removable by tree-shaking, and never a hidden default: the
context says which provider computed a result.

A provider may still supply any of the higher operations natively (Teimeris
does, exactly and fast), and the SDK prefers the native one when the
capability is declared and the profile allows it; conformance tests hold the
SDK's own implementation and the provider's to a stated agreement.

## What astrology needs from an ephemeris, exhaustively

| need | who consumes it | port status | who computes it by default |
|---|---|---|---|
| geometric or apparent positions of bodies at instants: longitude, latitude, distance and their speeds | everything | **required** | provider (built-in provider when none is registered) |
| which corrections the provider applied (geometric, light-time, aberration, deflection, nutation) and which frame (J2000 or of-date, geocentric, heliocentric, barycentric) | frame completion | required declaration | provider declares; SDK completes the rest |
| Delta T, timescales, leap seconds | time | optional override | SDK `astro` |
| obliquity, nutation, precession, frame bias, sidereal time | frames, houses, transforms | optional override | SDK `astro` |
| ayanamsha value for any of the 47 catalogue entries plus custom | sidereal charts | optional override | SDK `astro` (star-anchored ones use the SDK star table) |
| lunar nodes mean and true, apogee mean and osculating | Vedic and Western | required for mean node (or derivable from the provider's osculating elements); optional true node and apogees with an SDK derivation when absent | provider or SDK derivation |
| house cusps and angles for every system | charts | optional override | SDK `astro` (all systems) |
| topocentric correction | charts | optional override | SDK `astro` |
| rise, set, transit with disc, refraction and custom horizon | panchanga, muhurta, upagrahas | optional override | SDK `astro` |
| crossings (single body, composite angle, lattice), stations | limbs, ingresses, returns, transits | optional override | SDK `astro` |
| eclipses, occultations | eclipse charts, blackouts | optional override | SDK `astro` (v1.x) |
| fixed stars | Western, nakshatra yogataras, ayanamsha anchors | optional override | SDK star table (anchor stars in v1, full catalogue v1.x) |
| asteroids and minor bodies | Western | optional | provider only |
| identity and coverage: name, version, data version, date range, bodies, precision profile | validation, result stamps | required | provider |

## Design forces (unchanged in substance)

1. The port is the SDK's contract with Teimeris's signatures as template.
2. Required versus optional with capability negotiation; overrides preferred
   when declared; every result stamps which implementation ran.
3. Batch first: `positions(jds[], bodies[], frame)` returning columns.
4. Two adapter shapes with one API: native vtable (Teimeris, Swiss shim, the
   built-in provider) and host-language object (Node `sweph`, Python
   Skyfield); the built-in provider makes the SDK usable with zero setup.
5. Frames on the request, never provider state.
6. Determinism required and tested.

## Adapter inventory

| adapter | language | route | notes |
|---|---|---|---|
| built-in (`ephemeris-builtin`) | inside the SDK | native, same crate graph | analytic series; tiers `compact`, `standard`, `full`; the zero-setup default and the test oracle for the SDK's own astronomy layer against Teimeris |
| Teimeris | every binding | C vtable exported by Teimeris (Q15 decided: Teimeris will be updated as needed) | exact Swiss-compatible positions and, optionally, its native houses, events and crossings |
| Swiss Ephemeris (`sweph`, `pyswisseph`, `swisseph-rs`) | Node, Python, Rust | host-language adapter | for consumers already on Swiss; the baseline engine's bridge during migration |
| Skyfield, Astropy (JPL) | Python | host-language adapter | research users |
| test provider | all | fixed tables | for unit tests; no astronomy |

## Conformance for providers

Unchanged: a conformance kit with instants, bodies, expected values and
tolerance bands per precision profile, determinism and capability-honesty
checks. Additionally, every SDK-native override (houses, rise/set,
crossings, ayanamsha) is held to a published agreement with Teimeris, and
the built-in provider publishes its worst-case error per body, century and
tier against Teimeris.
