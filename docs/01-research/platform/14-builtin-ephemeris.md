# The built-in ephemeris

Status: `research`, 2026-09-04. Follows from Q7 (a built-in fallback
ephemeris from v1, with its own phase). Feeds the `ephemeris-builtin`
module and Phase 3 of the roadmap. Sources: the VSOP87 and ELP theories
(Bretagnon and Francou 1988; Chapront-Touzé and Chapront 1983 for
ELP2000-82B; Chapront and Francou 2003 for ELP/MPP02), Meeus's
*Astronomical Algorithms*, Astronomy Engine (Don Cross, MIT, VSOP87
truncated to ±1 arcminute, tested against NOVAS C 3.1 on DE405), Teimeris
as the oracle.

## Purpose

The SDK must compute a correct chart with nothing but the SDK installed.
That means an ephemeris that lives inside the SDK, needs no files, no
network and no licence beyond the SDK's own, and is honest about its
accuracy. It also gives the SDK an independent second source for its own
astronomy layer's tests, and a "works anywhere" default for wasm and mobile
where shipping ephemeris data is a size problem.

It implements the ephemeris port like any other provider. It is a module:
a consumer who registers Teimeris does not ship it.

## Theories and their accuracy

| body or quantity | theory | full-series accuracy | data size (full) | notes |
|---|---|---|---|---|
| Sun (Earth) and Mercury to Neptune | VSOP87 (variant D: heliocentric ecliptic spherical of date, or A/B: J2000 rectangular; the SDK uses the J2000 variant and lets `astro` do precession so the frame chain is uniform) | about 1 arcsec for Mercury to Mars over 2000 BCE to 6000 CE, 1 arcsec for Jupiter and Saturn over 1000 years around 2000, up to 0.1 arcsec for the outer planets in their validity spans (published claims) | about 1 MB of coefficients for all planets | truncation by amplitude threshold gives tiers; Astronomy Engine truncates to 1 arcminute with a tiny table |
| Moon | ELP/MPP02 (fitted to LLR or to DE405/DE406) replacing ELP2000-82B; ELP2000-82B as the smaller variant | a few arcseconds over centuries around 2000 for MPP02; ELP2000-82B is the classic used by Meeus (about 10 arcsec truncated) | about 1 to 3 MB full | the Moon is the sensitive body for panchanga and nakshatra boundaries, so the tiers are chosen by Moon accuracy first |
| Pluto | no VSOP theory; options: Meeus's periodic series (1885 to 2099, about 1 arcsec), or Chebyshev coefficients fitted by the SDK's own tool from a JPL DE kernel (public domain) for 1800 to 2400 | fitted: arcsecond or better; Meeus: limited span | small | the fitted approach is preferred because it has no span cliff inside the SDK's default range |
| lunar mean node and mean apogee | mean elements (ELP or Meeus polynomials) | exact by definition | none | |
| lunar true node and osculating apogee | from ELP osculating elements or from the velocity vector of the series | arcsecond-class | none | needs the series derivatives, which are analytic |
| Chiron, asteroids, Uranian points | not covered; the port reports the bodies as unsupported | | | a consumer needs Teimeris or Swiss for these |
| speeds | analytic derivatives of the series, not finite differences | exact for the series | none | The baseline engine's finite-difference retrograde detection was a defect; this avoids it by construction |

Everything above the geometric positions (light-time, aberration,
nutation, topocentric, sidereal frames) is done by the `astro` layer, so
the provider is small and the corrections are shared with every other
provider.

## Tiers

| tier | target accuracy (Moon, planets) | approximate size | intended use |
|---|---|---|---|
| `compact` | 1 arcminute, 1 arcminute (Astronomy Engine class) | tens of KB | wasm and mobile where every KB counts; quick previews |
| `standard` (default) | 2 arcsec Moon, 1 arcsec planets over 1800 to 2400 | a few hundred KB | production charts where a Teimeris or Swiss provider is not wanted |
| `full` | the theories' full accuracy | a few MB | research and the test oracle for the astronomy layer |

Tiers are cargo features and separate packages in the bindings; the tier
is stamped into every position's `source` field so a cached chart says
which tier computed it. The truncation thresholds and the resulting
worst-case errors per body and century are produced by the ingestion tool
and published in the generated accuracy document, never hand-written.

## Licensing

The series are published scientific results distributed by the IMCCE and
CDS for use with citation, and they are re-implemented in many permissive
open-source libraries. The SDK's code is its own (Apache-2.0), the
coefficient tables are generated from the published files by the SDK's
tool with citations embedded, and JPL DE kernels are US government public
domain. Swiss Ephemeris's Moshier code is AGPL and is not reused. Q17 asks
the maintainer to confirm the redistribution terms are acceptable before
Phase 3 ships.

## The ingestion tool

`tools/ephemgen`: downloads or reads the VSOP87 and ELP files and the DE
kernel, truncates by amplitude threshold per tier, fits Pluto's
Chebyshev coefficients, emits Rust tables with citations and a manifest
(theory, version, tier, thresholds, term counts, fitted range), and
produces the accuracy report by comparing against Teimeris on a dense grid
of instants per century. Generated tables are checked in and diffed by the
generated-artefact gate.

## Plan (Phase 3 in the roadmap)

1. `ephemgen` with VSOP87 ingestion and truncation; planets and Sun;
   conformance against Teimeris at all three tiers; accuracy report.
2. ELP/MPP02 ingestion; the Moon at all tiers; nakshatra and tithi
   boundary timing error published (the figure consumers care about).
3. Pluto fitted from DE; mean and true nodes; apogees.
4. Analytic speeds; stations against Teimeris.
5. Packaging: the provider as a module in every binding, tier selection,
   size gates per tier.
6. The provider conformance kit run against it; the SDK test suite gains
   it as a second oracle for the `astro` layer.
