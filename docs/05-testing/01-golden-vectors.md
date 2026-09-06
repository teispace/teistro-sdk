# Golden vectors

Status: `draft`, 2026-09-04; revised 2026-09-06 when the corpus moved to
its own repository. The result page of Phase 0 spike 1 and the plan for
the corpus the conformance harness (ADR-0022) consumes.

The corpus is [`teispace/teistro-conformance`](https://github.com/teispace/teistro-conformance),
mounted here as a pinned submodule at `fixtures/`. What each fixture
holds, what each chart is for and what every settings profile changes are
described there, in [`fixtures/README.md`](../../fixtures/README.md) and
[`fixtures/baseline/README.md`](../../fixtures/baseline/README.md). What
is on this page is the SDK's own reading of it: where the vectors come
from, how the harness will use them, and every convention of the
recording engine that the SDK deliberately does not copy.

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
| 2 | Teimeris | its own conformance corpus through its port adapter (spike 3 built the adapter; `spikes/03-ephemeris-port/`) | planned |
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

## The move (ADR-0022), done 2026-09-06

`teispace/teistro-conformance` exists under CC0-1.0 with a version of its
own, and `fixtures/` is a submodule of it pinned to a tag. Every file
moved byte for byte; the SDK's own gate proves it after the move as it
did before.

The corpus took its description and its checking with it: JSON Schemas
for a fixture, a manifest, the tolerance file and a conformance report,
and `validate.py`, which checks every fixture against its schema, every
settings hash against the profile it claims, every listed section against
what the file holds, and every file against the manifest that must list
it. It runs there on every push.

What stayed here is what is the SDK's rather than the corpus's: the
registry of conventions below, and `cargo xtask check-fixtures`, which
now also refuses to pass when the submodule is not checked out — a
corpus that is absent must not read as a corpus that agrees.

Bumping the pin is deliberate: `git -C fixtures fetch --tags && git -C
fixtures checkout vX.Y.Z`, then the SDK's gates, then a commit that says
which values moved. What is still to come is runners per binding that
emit the report schema.

## Open items

- A second baseline export for the seventeen other dasha systems, the
  aspects, the yogas and doshas, the strengths, Ashtakavarga, the
  Jaimini slice, KP and milan, once the corresponding design pages exist
  and say what the fixture must carry (the same script, more sections).
- Rank-1 vectors for the cruxes that block Phase 5 (C1, C2, C3, C6, C8).
- The tolerance bands are provisional until measured against Teimeris
  and the built-in tiers.

## The baseline's conventions, registered rather than copied

Observed while exporting. Each becomes a row of the deliberate-difference
registry the harness reads, so a divergence here is expected and
explained rather than a failure or a silent adoption.

1. The natal panchanga uses the topocentric Moon (c055 shows tithi 1 a
   second after the topocentric new moon); the daily panchanga is
   geocentric like every almanac. The SDK's day-boundary and frame knobs
   decide this explicitly (`02-architecture/05-data-model-identifiers.md`).
2. Local mean time is rounded to the whole minute (c023, c047: a
   longitude of -0.13° gives -1 minute, not -31 seconds), and
   `tz_offset_min` is whole minutes while the resolved instant keeps the
   seconds of historical offsets (c005, c011, c025, c038).
3. Polar day and polar night have no horizon crossing; the engine
   synthesises sunrise and sunset as the bounds of the civil day (c028:
   a 24-hour day) or collapses them (c029: a zero-length day and a
   post-sunset birth). The SDK's polar policy is a setting (ADR-0016's
   companion pages); these two fixtures are recorded for comparison only.
4. Sunrise ignores the observer's altitude; the altitude reaches only the
   topocentric positions.
5. The Placidus cusps at 69.6°N (c028, c029) are returned without a
   degeneracy flag; the library falls back internally above the polar
   circle and the engine's heuristic does not notice. The SDK's house
   module must flag them (`03-design/` house page, planned).
6. Ketu is Rahu plus 180° in the same frame; the nodes carry no
   latitude, speed or distance of their own beyond what the library
   reports for the node.
7. The dasha balance breakdown uses 365.25-day years and 30.4375-day
   months with integer floors and a rounded final minute.
8. Zone ids come from the coordinates through a lookup library and
   canonical tzdb names, so Tromsø resolves to `Europe/Berlin` (a tzdb
   link); the SDK reports the id its own provider gives.
9. The engine stores no tropical longitude; the exporter asked the
   library for it in the positions' frame, and for the Lagna added the
   nutated ayanamsha to the sidereal value (the two agree to a
   milliarcsecond for the planets).
10. The `outer` planets are geocentric even under a topocentric profile.
11. A zone resolution's era is labelled by comparing the applied offset
    with the offset in force when the export ran, a northern summer, so a
    seasonal offset the zone still applies every year reads `historical`
    (c018 Sydney and c019 Auckland in their summer time, c029 Berlin and
    c035 New York in standard time, c037 the later occurrence of the New
    York fold); the SDK compares with the offsets the zone applies in the
    database's own year and reads no clock, so it calls those `current`
    (cruxes register C33).

12. Sunrise and sunset. The baseline's library computed refraction from
    its standard atmosphere; the SDK's standard refraction is the
    almanac's 34 arcminutes with the semidiameter and the horizontal
    parallax from the distance, so the SDK's sunrise under the same
    convention differs from the fixtures' by up to 2.5 s below 60° of
    latitude and 9.8 s at Fairbanks on the solstice (cruxes C34); the
    band on `foundation.sunrise.*_jd` is set from that spread when the
    harness lands. The day's arc in the SDK is the civil date's sunrise
    and the sunset that follows it, as the baseline reckons for c043,
    whose sunset falls after the next civil midnight. For c022, c025 and
    c039 the baseline's `foundation.sunrise` block holds the previous
    day's events and its `next_day` block the civil date's own (C35): a
    comparison reads the latter there.

13. Planetary hours. The baseline reckons twelve horas over the day from
    sunrise to sunset and twelve over the night (`hora_reckoning:
    PROPORTIONAL`); `crates/time/tests/hora_fixtures.rs` reproduces the
    lord at the birth instant for every chart except c022 and c039, whose
    day-early sunrise block (convention twelve) places the birth in
    another day's horas, and c028, whose polar day the baseline
    synthesises. The equal reckoning (sixty-minute horas from sunrise)
    disagrees with the fixtures on many charts, so they decide the
    default.
