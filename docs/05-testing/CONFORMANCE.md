# Conformance

Status: `generated`, by `cargo xtask conformance` from the crates' corpus tests
and `fixtures/corpus.json`, held by `cargo xtask check-conformance`; do not edit.

The SDK against the conformance corpus 0.11.0 (released 2026-09-15,
https://github.com/teispace/teistro-conformance), one section per corpus directory. Each row is one test's comparisons under one reading:
**compared** counts every comparison the test attempted, **explained** the
misses a named entry accounts for, listed below with its count, and
**agree** the rest. A miss no entry explains fails its test, so the page
cannot be regenerated with one: **unexplained misses: 0**. There is no total
across sections, since a yoga decision and a longitude are not one unit
(`03-design/generated-pages.md` §2.1).

| section | rank | reading | compared | agree | explained |
|---|---|---|---|---|---|
| `baseline` | 2 | Vimshottari under the engine's reading | 73573 | 73573 | 0 |
| `baseline` | 2 | positions over the built-in standard tier | 2420 | 2359 | 61 |
| `baseline/dasha-systems` | 2 | the engine's reading | 212688 | 212688 | 0 |
| `baseline/rashi-dashas` | 2 | the engine's reading | 300486 | 300486 | 0 |
| `baseline/kalachakra` | 2 | the engine's reading | 121352 | 121352 | 0 |
| `baseline/ashtakavarga` | 2 | the engine's reading | 1848 | 1848 | 0 |
| `baseline/vimshopaka` | 2 | the engine's reading | 2604 | 2604 | 0 |
| `baseline/shadbala` | 2 | the engine's reading | 11431 | 11431 | 0 |
| `baseline/bhava-bala` | 2 | the engine's reading | 3408 | 3408 | 0 |
| `baseline/yogas` | 2 | the SDK's rules for the eight the engine computes in code | 1148 | 1148 | 0 |
| `baseline/yogas` | 2 | the engine's reading | 76921 | 76921 | 0 |
| `baseline/doshas` | 2 | the SDK's rules for the seventeen the engine computes in code | 2272 | 2268 | 4 |
| `baseline/doshas` | 2 | the engine's reading | 8565 | 8565 | 0 |
| `pyjhora/vimshottari` | 3 | four year lengths, each start within its year's drift from the kernel's | 16444 | 16307 | 137 |
| `official` | 1 | the Surya Siddhanta under punya-kala, against the committee's printed sankrantis and months | 50 | 50 | 0 |
| `official` | 1 | the Surya Siddhanta's Moon with the committee's bija, against its printed Moon and tithi ends | 12 | 12 | 0 |
| `official` | 1 | the Surya Siddhanta's Saturn and node, against the committee's printed ones | 8 | 0 | 8 |
| `official` | 1 | the Surya Siddhanta's Sun, against the committee's printed Sun | 2 | 2 | 0 |

## Explained misses

- `baseline`, positions over the built-in standard tier: delta-t-beyond-the-table — 40
- `baseline`, positions over the built-in standard tier: moon-topocentric-speed — 21
- `baseline/doshas`, the SDK's rules for the seventeen the engine computes in code: the seven grahas the nodes caught, named (doshas-measured.md) — 4
- `pyjhora/vimshottari`, four year lengths, each start within its year's drift from the kernel's: the written balance's day, begun or whole (dasha-kernels.md) — 137
- `official`, the Surya Siddhanta's Saturn and node, against the committee's printed ones: the committee's star planets are modern (bikram-sambat.md, R2) — 8

## Not scored here

- `baseline/names.json` (the same engine's entity name tables): vocabulary, not answers: the source `sdk.entity`'s names were migrated from (`crates/intl/src/migrate.rs`, whose tests read it), and a name is vetted by a speaker rather than scored.
- `teimeris` (Teimeris 0.1.0 in its corrected profile): an engine's own outputs, measured rather than scored: the worst difference of each is on `ACCURACY.md`, held by `cargo xtask check-accuracy`.
