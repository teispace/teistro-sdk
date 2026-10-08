# Generated pages: `SIZES.md`, `CONFORMANCE.md`, and ACCURACY's JPL rows

Status: `draft`, 2026-10-08. Track A item 7 of the completion plan
(`07-roadmap/00-roadmap.md`). `05-testing/README.md` already lists
`SIZES.md` and `CONFORMANCE.md` as generated, and
`05-testing/01-golden-vectors.md` step 5 says `CONFORMANCE.md` summarises
the per-class reports. Neither page exists yet.

## 0. What already exists, and the one problem both pages share

- **The page machinery.** `xtask/src/generated.rs` gives `Output`,
  `write` and `check`. `check` regenerates a page in memory, fails on any
  difference and prints the first differing lines, so a number that moved
  on a platform the author does not run is visible in the log.
  `xtask/src/skip.rs` gives `skip` (counted, and a failure under
  `TEISTRO_STRICT`, which verify and release set) and `excused` (by design,
  with its reason).
- **The lint `generated-page-is-gated`** (`xtask/src/lints.rs`) reads
  only `docs/03-design/*-measured.md`. `ACCURACY.md` in `docs/05-testing`
  is outside it today, and the two new pages would be too. **First fix:**
  make `MEASURED` a list. Add `docs/05-testing` with an explicit list of
  its generated pages (`ACCURACY.md`, `CONFORMANCE.md`, `SIZES.md`), held
  both ways: a listed page that is missing fails, and an upper-case page
  there that is not listed fails. Prove it red by renaming one.
- **Sizes are already measured, twice.** `xtask/src/package.rs` writes a
  per-platform manifest (`teistro-<version>-<platform>.json`) with the size
  and SHA-256 of every file it stages: the raw library, its `.gz`, the C
  bundle, the npm platform package, the SBOMs. `package stage` merges them
  into `target/dist/manifest.json`. `check-package` (`consumer.rs`)
  runs `package::build` and `package::stage --partial` on every verify
  bindings row. `check-wasm` measures the staged module raw and
  gzip-best against `bindings/wasm/size.json`, both ways.
- **The corpus is already scored.** `crates/ephemeris-kit/src/corpus.rs`
  writes a `CorpusReport` in the corpus's report format, judged against
  the exhaustive `KNOWN` divergences (which fail both ways).
  `crates/ephemeris-builtin/tests/conformance.rs` runs it per tier in
  verify's `ephemeris tier` matrix and writes `target/kit/corpus-<tier>.json`.
  The Teimeris adapter's `kit` binary runs it `--class same-ephemeris`,
  and with `--native-frame-only`, by hand. Every other corpus section is
  decided by a falsification pass (`yogas.rs`, `shadbala.rs`,
  `dasha_systems.rs`, …) whose page is gated in fast-check.

**The shared problem: most numbers come from builds that do not happen
locally.** Every platform's library is built on its own runner. The wasm
profiles and the non-default ephemeris tiers are feature builds. Teimeris
is an adapter outside the workspace. CI's Linux wasm build is not even the
same size as macOS's (`measure-what-ships`). So a page "measured live"
from a laptop would be a page about the laptop.

**The rule both pages follow.** A number comes from one of two places,
and the page says which beside it:

1. **live**: computed in the gate from sources every machine has (the
   fixtures, the source tree, the default build). `check-*` recomputes it
   everywhere, exactly.
2. **recorded**: measured by a named CI run (or a dated run by hand) and
   committed as a structured record (JSON) carrying the run id, the
   commit, the date and the tool versions. The page is rendered from the
   record, so `check-*` holds the *page* everywhere. Wherever the
   artefact exists (verify, release), the gate *also* re-measures it and
   compares with the record. Where it does not, that half is `excused`
   locally ("built only in verify") and a `skip` under `TEISTRO_STRICT`,
   so verify cannot pass while measuring nothing.

A recorded number is never typed. The record is written only by the
`xtask` command that reads the run's artefacts, and the page never
restates a recorded figure in prose (`a-prose-claim-is-the-part-that-rots`).

## 1. `SIZES.md`

### 1.1 What it lists

Every artefact a release ships, with **raw** and **gzipped** bytes, and its
budget where one exists. Gzipped is what a consumer downloads, so it is
the column a decision reads (`measure-what-ships`).

| group | artefact | raw from | gzip from |
|---|---|---|---|
| per platform (every row of `platform::PLATFORMS`, musl included) | shared library | manifest `library.size` | the `.gz` archive's `size` (gzip-best, as the Dart installer downloads it) |
| | C bundle (`.tar.gz`) | — | manifest `archives[1].size` |
| | Node addon | manifest `addon.size` | gzipped in the record step, gzip-best |
| | npm platform package (`.tgz`) | — | manifest `npm.size` |
| | Python wheel | `wheel.rs` output | — (a wheel is a zip; its size *is* the download) |
| wasm, per profile (`full`, then `panchanga`, from `wasm-profiles.md`) | module `teistro_wasm_bg.wasm` | staged package | gzip-best, as `check-wasm` computes it |
| | glue JS | staged package | gzip-best |
| | the npm wasm package (`.tgz`) | `npm pack` in staging | — |
| bundler | one `/catalogue` member bundled by esbuild, Vite, webpack | `check-wasm`'s bundler step | gzip-best |
| bundler | the entry bundled | the same | the same |
| Dart, Node root packages | the published archive | staging | — |

A row exists for every platform in `PLATFORMS`, every wasm profile the
build declares and every bundler pinned in `bindings/wasm/bundlers`. The
lists are read from those sources, not repeated (`list-dont-infer`). A
platform or profile with no recorded figure is a failure, not a blank
cell.

Each row also shows the recorded figure's run and the headroom left
under its budget, as `check-wasm` already prints on a pass.

### 1.2 Where the numbers come from

A new **`xtask/src/sizes.rs`**:

- `cargo xtask sizes --from DIR` reads a directory of a run's artefacts:
  the per-platform manifests, the staged wasm package per profile, and a
  small `bundles.json` the bundler check writes. It writes
  **`docs/05-testing/sizes.json`** (the record) and **`SIZES.md`** (the
  page). The record carries `run` (the workflow run id), `commit`, `date`,
  the toolchain (`rustc -V`, wasm-bindgen, flate2's version, because gzip
  bytes depend on the encoder) and every figure.
- `cargo xtask check-sizes` renders `SIZES.md` from `sizes.json` and
  compares (everywhere, in fast-check). It then re-measures whatever
  artefacts exist under `target/dist`, so it is live in verify and release.

**Budgets move into the record.** `bindings/wasm/size.json` holds
today's budget and, in its `why`, every past re-measure as prose figures
(the very form that rots). The design folds it into `sizes.json`:

- `budget: {raw, gzip}` per budgeted artefact, under the file's existing
  rule (2% over, to the next ten kilobytes, held both ways);
- `why` as one sentence per re-baseline saying *what grew it*, with the
  figures in structured `history` entries (`run`, `date`, `raw`, `gzip`)
  that the page renders as a table.

`check-wasm` reads its budget from there. Budgets are added for the
shared library per platform and the wasm profiles. Native libraries are
loaded, not downloaded, so their budget is a regression alarm with a wide
band, and the page says so.

### 1.3 Which gate holds what, and how it stays honest

| check | where | holds |
|---|---|---|
| `check-sizes` render | fast-check, every machine | the page equals what the record renders |
| record completeness | fast-check | every platform, profile and bundler has a row; a row whose source no longer exists is stale and fails |
| live versus budget | verify bindings rows, verify wasm job, release | each built artefact inside its budget, both ways (as `check-wasm` today) |
| live versus record | the same | each built artefact's gzip within **2%** of the record (the budget file's own headroom unit); over that, the gate fails with the `cargo xtask sizes --from` command to re-record |
| release exactness | release `stage` | the release's own figures are written as `SIZES.md` *for that release* and attached to the GitHub release and the docs site. The repository's page names the run it came from, and a release is refused if the committed record is more than 2% from it |

Why 2% and not exact: almost every feature PR moves the wasm module.
Exact equality would mean a CI round-trip for every PR. That is the
friction which made `size.json` a prose log. With 2%, the page is never
more than one budget step stale, and the release's attached page is
exact.

**Getting the run's artefacts.** Verify's bindings rows upload their
`target/dist/teistro-*-<platform>.json` and the wasm job its staged
package as `sizes-<row>` artefacts. A last verify job, `sizes`, downloads
them all and runs `check-sizes --live target/sizes`, so one job compares
every platform at once. To re-record, a contributor runs `gh run download
<id> -p 'sizes-*'` (by run id, `measure-what-ships`) and then `cargo xtask
sizes --from`. This stays a manual step, because CI never commits.

## 2. `CONFORMANCE.md`

### 2.1 What it says

The SDK's score on the conformance corpus, **one section per corpus
directory** in `fixtures/corpus.json`'s `corpora`, with the corpus version
and tag read from that file. For each section:

| column | meaning |
|---|---|
| section | the corpus path (`baseline/yogas`, `pyjhora/vimshottari`, `official`, …) and its evidence rank |
| compared | how many recorded values the SDK compared, counted as **attempted**, so a section that silently compared nothing shows 0 and fails (`count-attempts-not-reports`) |
| agree | within the tolerance file's band for the class, or exactly for a classification |
| explained | misses each covered by a named entry: a `KNOWN` divergence, a crux (C-number), an engine finding (F-number) or a deliberate difference, **each listed with its count** |
| unexplained | misses no entry covers. **Must be zero for a release**; the gate prints each |
| idle | entries that applied to this run and explained nothing: stale, and they fail (as `KNOWN` does) |

There is no single percentage across sections. A yoga decision and a
lagna to 1e-6° are not one unit, and a total over them would answer no
question (`judge-a-claim-on-what-it-says`). The page's one summary line
is the count of unexplained misses, the figure that blocks a release.

### 2.2 The positions section and the reference attribution

Positions are scored per **provider class** (from `tolerances.json`), one
column each: `builtin-compact`, `builtin-standard`, `builtin-full`,
`same-ephemeris`, and later `reference` and `de`. Beside them are the
three attribution runs the project already makes
(`a-reference-run-splits-sdk-from-engine`):

1. **across tiers**: a miss that is the same in every built-in tier is
   not ephemeris accuracy, and the page marks it;
2. **same ephemeris** (Teimeris through its adapter): this splits "the
   SDK completes it wrongly" from "the engine computes it differently";
3. **native frame only** (`--native-frame-only`): isolates the SDK's
   completion.

Each `KNOWN` divergence is a row naming the classes it holds under and
the fixtures it covers, so a reader sees which run attributed it.

### 2.3 Where the numbers come from

- **Every section is live, scored by the crates' corpus tests.** Found
  while building: the xtask passes that read a corpus section weigh rival
  readings for a measured page; the comparison the SDK answers for is the
  crate's corpus test, which already fails on a miss. So each such test
  compares through one shared `Tally`
  (`crates/core/tests/support/conformance.rs`): every comparison counted
  as attempted, a miss failing unless a named entry explains it, and one
  score line written to `scores.jsonl` when `TEISTRO_CONFORMANCE_DIR` is
  set. `cargo xtask conformance` runs those tests and renders the page;
  nothing is computed twice and no score file is checked in. A failed
  test records nothing, so an unexplained miss fails the gate rather than
  reaching the page.
- **The default tier is live too.** `builtin-standard` is the default
  feature, so the builtin crate's corpus test scores positions through
  the same tally, naming each `KNOWN` divergence with the misses it
  explains.
- **The other tiers are recorded.** Verify's `ephemeris tier` jobs already
  write `target/kit/corpus-<tier>.json`. They upload it, and
  `cargo xtask conformance --from DIR` records it with the run id.
  Verify re-runs and compares exactly: a corpus report is deterministic,
  unlike a byte size, so here the comparison is exact, not 2%.
- **Same-ephemeris and native-frame runs are by hand**, dated. The
  adapter's `kit --corpus fixtures --class same-ephemeris --out DIR`
  writes the report; the record names the Teimeris version and commit and
  the date. The page shows the date beside the column, as ACCURACY's
  "by hand" column does.

**`check-conformance`** (fast-check):

- the page equals what the score files render;
- **every corpus directory in `corpus.json` has a score file or a declared
  reason** (an exhaustive `const` with a reason each, failing both ways).
  Today, `baseline/names.json` may be compared by `intl` and not scored
  here. That must be a declared reason, not an absence;
- every `KNOWN` entry, crux and finding the scores name exists;
- the corpus tag in the record equals the submodule's pinned tag. A bumped
  corpus with old scores fails.

### 2.4 Found while reading

`fixtures/README.md` says "Version 0.1.1" while `fixtures/corpus.json`
says `0.11.0` (released 2026-09-15). That prose figure has already gone
stale in the corpus repo. `CONFORMANCE.md` should read the version only
from `corpus.json`, and the corpus repo's own check could hold its README
to it.

## 3. ACCURACY's Horizons and CSPICE rows

ADR-0021 makes JPL Horizons and CSPICE against DE440 the final authority:
"agreement with Swiss Ephemeris proves consistency, not correctness".
Every `ACCURACY.md` row today measures against Teimeris or the baseline.
The truth rows need a JPL recording **in the corpus**, so that CI reads
it offline like `fixtures/teimeris/` and the measurement tests can put it
in the "measured in CI" column. "By hand" would mean a number nobody
re-checks.

### 3.1 What to record (a new `jpl/` corpus directory)

- **`jpl/horizons/vectors.json`**: geometric state vectors (no
  corrections) of the Sun, the Moon, the planets and Pluto. Centre
  `500@399` (geocentre) and `@0` (barycentre), `REF_SYSTEM=ICRF`,
  `TIME_TYPE=TDB`, so no ΔT enters. This rung isolates the ephemeris.
- **`jpl/horizons/observer.json`**: astrometric and apparent RA/Dec and
  ecliptic longitude/latitude, geocentric and at a few topocentric sites
  (the corpus's own places), at TT instants. This rung measures the SDK's
  reduction chain (light time, aberration, deflection, precession–
  nutation) against JPL's. Record the precession–nutation model the
  Horizons header states as a **convention**, so a model difference is
  attributed, not counted as error.
- **`jpl/cspice/states.json`**: the same geometric and corrected states
  from CSPICE (`spkezr` with `NONE`, `LT`, `LT+S`, frames `J2000` and
  `ECLIPJ2000`) over `de440.bsp`. This is the second reference, and
  Horizons against CSPICE is the reference checking itself. Their
  agreement is its own ACCURACY row, so a disagreement is the
  reference's, not the SDK's.
- **The instants**: a stratified grid across 1600–2400 plus adversarial
  instants (perigee, apogee, stations, conjunctions, segment boundaries).
  The SDK's own searches may *choose* the adversarial instants, but the
  values come only from JPL, so the choice cannot bias them. ADR-0021's
  six-hourly sweep over the whole range is too large for a corpus file.
  It runs by hand, and only its summary (worst per body and rung, with
  the instant) is recorded, in a dated `jpl/sweep-summary.json`.
- **`manifest.json` with provenance**, per the corpus contract: the
  recording date; every Horizons query exactly as sent (the API URL with
  its parameters); the API's version signature and the ephemeris it names
  (DE440 or DE441, stated, not assumed); the SHA-256 of every raw
  response, with the raw responses kept under `jpl/horizons/raw/`; the
  CSPICE toolkit version; each kernel's file name and checksum (`de440.bsp`,
  `naif0012.tls`, `pck00011.tpc`, and the Earth orientation kernel used for
  topocentric states); and the recorder script, committed in the corpus
  repo under `tools/` (CC0) so a reader can rerun it.
- **Evidence rank 1** (an institution's own publication), and a
  `tolerances.json` class for "against DE440", or the existing `de`
  class.
- **Licence check before recording.** JPL data and NAIF kernels are
  generally free to use, but whether recorded values can be dedicated
  under CC0 should be confirmed against JPL's and NAIF's terms. This is a
  risk, not an assumption.

### 3.2 How ACCURACY reads it

- New tests in `crates/astro/tests` (`jpl_vectors.rs`, `jpl_apparent.rs`,
  `jpl_cspice.rs`) read the recording and log measurements under new row
  ids, exactly as the Teimeris tests do via `TEISTRO_ACCURACY_DIR`.
  Measuring the built-in tiers against truth is a per-tier figure. The
  default tier is measured in fast-check, and the others in verify's tier
  jobs, recorded the same way as §2.3's tier reports.
- `accuracy-rows.yaml` gains rows: `ephemeris_truth` (geometric, per
  tier), `apparent_truth` (the reduction chain), and `reference_self`
  (Horizons against CSPICE).
- **The date is gated, not typed.** `accuracy.rs` reads the recording
  date from the corpus manifest (`corpus.json` `recorded`) for the JPL
  rows rather than from a `by_hand` string, so a re-recording changes the
  page through the generator.
- `completion-measured.md` and `pluto-measured.md` both say a refit
  against JPL is owed. Once the recording exists, their "proves
  consistency, not truth" paragraphs gain a truth figure from the same
  tests.

## 4. Order of work

1. **Widen `generated-page-is-gated`** to `docs/05-testing` with the
   explicit list, proved red. This is cheap and covers `ACCURACY.md` today.
   **Done:** a page anywhere under `docs` whose status line calls it
   generated is held, whatever its name, and a gate may be named bare or
   as the `cargo xtask` command; proved red on `ACCURACY.md` with its gate
   removed and on the panchanga conventions page with its gate misspelt.
2. **`CONFORMANCE.md`, live half**: the score-file `Output` in each corpus
   pass, the section list held to `corpus.json`, `check-conformance` in
   fast-check. Positions over the default tier.
   **Done** (2026-10-08), through the crates' tests rather than the
   passes (§2.3); `baseline/names.json` and `teimeris` are declared
   unscored with their reasons. Not built yet: the check that each named
   entry exists, and the corpus tag against the submodule's pin.
3. **`CONFORMANCE.md`, recorded half**: verify uploads the tier reports,
   `conformance --from`, the exact live comparison in verify. Then the
   dated same-ephemeris and native-frame columns from a run by hand.
4. **`SIZES.md`**: `sizes.json` absorbing `bindings/wasm/size.json`
   (`check-wasm` reading it), verify's `sizes-*` uploads and the `sizes`
   job, `sizes --from`, `check-sizes`. Then the release's attached exact
   page. Add wasm profile rows as each profile is staged
   (`wasm-profiles.md` step 3).
5. **The JPL recording** in the corpus repo (licence check, recorder
   script, a corpus release and tag), then the submodule bump, the astro
   tests and the ACCURACY rows. This is the largest piece and the only one
   that waits on another repository, so it can start in parallel with 2.
6. Run **every** `check-*` after each step (`sweep-every-generated-page`),
   and read the CI step durations before merging.
