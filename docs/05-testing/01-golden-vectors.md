# Golden vectors

Status: `draft`, 2026-09-04. The result page of Phase 0 spike 1 and the
plan for the corpus the conformance harness (ADR-0022) will consume. The
files themselves and their schema are described in
[`../../fixtures/README.md`](../../fixtures/README.md).

## What a golden vector is here

A recorded output of another implementation that the SDK must reproduce
within a stated tolerance, carrying the complete settings profile it was
computed under, the hash of that profile, every version that influenced
the numbers, and a citation of where it came from. The tolerance is never
in the fixture: it lives in one central file keyed by field and provider
class (`fixtures/tolerances.json`), so loosening a band is a visible,
reviewed change to one line rather than a quiet edit in one fixture.

## Sources, by evidence rank (ADR-0018)

| rank | source | how the vectors are made | state |
|---|---|---|---:|
| 1 | classical texts and printed almanacs | hand-entered, with the verse or page cited on every value | planned, per crux in `01-research/feature-universe/19-verification-cruxes.md` |
| 2 | the baseline engine | an export script in the baseline engine's own repository, run against its built packages and its ephemeris files | 115 fixtures over 55 charts (spike 1) |
| 2 | Teimeris | its own conformance corpus, once its vtable adapter exists (spike 3) | planned |
| 3 | PyJHora, JHora and Parashara's Light printouts | scripted exports and hand-entered printouts, marked rank 3 | planned |

## Spike 1: the baseline export

The spike asked for a script that dumps the foundation, positions,
houses, vargas, dignities, a panchanga day and a Vimshottari tree for
fifty charts with settings and versions. It delivered:

| item | result |
|---|---|
| charts | 55: 48 chosen for time-zone, latitude, altitude and data-range hostility, 7 placed by search to the second at a classification boundary |
| fixtures | 115: every chart under the default profile, and 60 variants under twelve alternative profiles (balance method, node, frame, houses, four ayanamshas, the tropical zodiac, the Surya Siddhanta engine with and without the node correction, the sunrise mode) |
| sections | foundation, positions (with avasthas and the outer planets), houses (the selected system, all 22 systems, Bhava Chalit, special lagnas, arudhas, chara karakas), vargas (21), the natal panchanga, the daily panchanga with every transition, Vimshottari under both balance methods with the tree to depth 3, children at chosen paths to level 5 and the active chain at two instants |
| size | 8.9 MB pretty-printed; dasha trees as rows keep a chart under 130 KB |
| run time | four seconds for the whole set |
| failures | none; the polar-day and polar-night charts computed under the engine's fallback conventions |
| gate | `cargo xtask check-fixtures` in the fast check |

The script is not part of this repository and is written in the baseline
engine's language; the maintainer keeps it beside that engine. Nothing in
the fixtures names the engine, its company or its packages, and the
fixtures gate enforces that alongside the docs gate.

### Design choices worth keeping

- **Searched instants.** Seven charts were placed by bisection in the
  topocentric frame the natal chart uses: the Moon within an arcsecond
  past a nakshatra edge and past a pada edge, chosen so that the
  geocentric Moon is still on the other side (the dasha lord and the pada
  then depend on the frame, and the `geocentric` variant of the same
  chart proves it); the last whole second before and the first after the
  Sun's sidereal Aries ingress; the first second after a Mercury station,
  where the retrograde flag flips; the ascendant within a second past a
  sign edge; the first second after a new moon, where tithi 30 becomes
  tithi 1. These are the cases the exact-classification design
  (ADR-0016) exists for.
- **Variants export only what the knob changes.** A house-system variant
  carries the houses section alone; an ayanamsha variant carries
  everything. The manifest records which sections each profile carries.
- **Trees as rows.** A dasha tree is a list of `[path, lord, start_jd,
  end_jd]` rows in pre-order; the level is the path's length. An order of
  magnitude smaller than nested objects, and a diff shows the one row
  that moved.
- **The settings hash is reproducible**, not just recorded: each fixture
  carries the canonical `key=value` string the engine hashed and the
  truncated SHA-256 of it. The SDK's own settings hash (ADR-0020) differs
  in recipe; the harness maps the fixture's profile to an SDK profile,
  asserts the SDK hash it computes, and records the source hash beside it.

### What the export revealed

Ten baseline conventions are listed in the fixtures README as rows for the
deliberate-difference registry, not as behaviour to copy. The two that
matter most for design:

1. The natal panchanga uses the topocentric Moon while the daily
   panchanga is geocentric. The SDK makes the frame a setting and says
   which it applied in the envelope (`applied_conventions`, ADR-0020).
2. Local mean time is rounded to the whole minute, and Placidus cusps
   above the polar circle come back without a degeneracy flag. Both are
   defects to fix in the SDK, not to reproduce; the fixtures pin the
   baseline's answer so the difference is explained rather than
   discovered.

## How the harness will use the corpus (Phase 1)

1. Load the manifest; for each fixture, map its profile to an SDK settings
   profile and compute the SDK settings hash; refuse a fixture whose
   profile cannot be expressed.
2. Resolve the input the SDK's own way (zone from the coordinates, LMT
   from the longitude to the second) and compare the instant to
   `input.resolved.jd_ut` under the `same-ephemeris` band for instants;
   record the known minute-rounding difference for the LMT charts.
3. Compute the listed sections and compare field by field under the
   band for the provider class in use; classification fields exactly,
   with the edge policy in the tolerance file for values within a band
   of a boundary.
4. Classify the fixture's own `f64` longitudes with the SDK's integer
   path (ADR-0016) and expect the fixture's sign, nakshatra and pada: this
   tests the classification independently of the ephemeris.
5. Emit a machine-readable report per provider class; the generated
   `CONFORMANCE.md` summarises it; a failing fixture blocks the release
   unless a registry row explains it.

## The move (ADR-0022)

Before Phase 1 exits the maintainer creates `teispace/teistro-conformance`
under CC0-1.0, `fixtures/` moves there with its README, tolerance file
and manifest, gains a JSON Schema and runners per binding, and is mounted
here as a version-pinned submodule; `cargo xtask check-fixtures` runs
there as the corpus's own gate.

## Open items

- A second baseline export for the seventeen other dasha systems, the
  aspects, the yogas and doshas, the strengths, Ashtakavarga, the
  Jaimini slice, KP and milan, once the corresponding design pages exist
  and say what the fixture must carry (the same script, more sections).
- Rank-1 vectors for the cruxes that block Phase 5 (C1, C2, C3, C6, C8).
- The tolerance bands are provisional until measured against Teimeris
  and the built-in tiers.
