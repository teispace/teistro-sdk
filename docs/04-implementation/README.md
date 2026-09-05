# Implementation

Status: `planned`, revised 2026-09-04. Rust is decided (ADR-0001). The
project is public and open source from the first commit, structured as a
large-scale project: governance files, RFC process, generated artefacts,
and one tool for everything.

## Repository layout

```
teistro-sdk/
  LICENSE                      Apache-2.0
  NOTICE                       third-party notices (ERFA BSD-3 with its provenance table, VSOP87, ELP, CLDR, tzdb, Hipparcos)
  CLEAN_ROOM.md                the clean-room policy: sources by rank, what may be taken (ADR-0019)
  deny.toml                    the dependency allow list checked by `cargo deny` (ADR-0019)
  README.md  CONTRIBUTING.md  CODE_OF_CONDUCT.md  SECURITY.md  GOVERNANCE.md
  CHANGELOG.md                 "Numbers" first
  CODEOWNERS
  .github/
    ISSUE_TEMPLATE/            bug, feature, accuracy-report (with chart data), docs
    PULL_REQUEST_TEMPLATE.md   numbers, change, verification
    workflows/                 fast-check, nightly-verify, release, docs, scheduled
  rfcs/                        the RFC process for significant changes
  Cargo.toml                   workspace; rust-toolchain.toml
  crates/
    core/
    port-ephemeris/ port-calendar/ port-timezone/ port-geo/ port-intl-data/ port-log/
    astro/                     the astronomy layer; its IAU routines are a port of ERFA (ADR-0021)
    ephemeris-builtin/         VSOP87, ELP/MPP02, fitted Pluto; tiers as features, plus the DE-refit `reference` tier
    ephemeris-de/              the JPL DE file reader provider (v1.x, ADR-0021)
    siddhanta/
    calendar/ time/
    chart/ houses/ vargas/ state/ aspect/ points/
    strength/ dasha/ rules/ jaimini/ kp/ tajika/
    panchanga/ muhurta/ gochar/ prashna/ matching/ rectification/ longevity/
    remedies/ numerology/ lalkitab/ pakshi/ namakarana/ rashifal/ research/
    interpret/ intl/ serial/
    ffi/                       the only unsafe crate; C ABI; cbindgen config
    test-provider/             fixed-table ephemeris for tests
  catalogue/                   entity catalogue YAML (keys, ids, attributes, citations); present, with its README,
                               generated into crates/core by `cargo xtask gen catalogue` and gated by `check-catalogue`
  i18n/
    en-Latn/  ne-Deva-NP/  hi-Deva-IN/  sa-Deva/     Teistro Intl sources (SDK namespaces)
  packs/
    interpret/<locale>/        interpretation sources with citations
    rules/<pack>/              rule packs (yogas, doshas, muhurta, matching, remedies)
  profiles/*.yaml
  idl/                         extracted API description, generators, checkers (or the Diplomat bridge, per ADR-0007)
  bindings/
    c/ cpp/ node/ wasm/ python/ dart/ teistro_flutter/ rust/ java/
    shared/                    ergonomic code shared by node and wasm
  adapters/
    ephemeris-teimeris/<binding>/    published separately (Teimeris terms)
    ephemeris-sweph/<binding>/       published separately (Swiss terms)
  xtask/                       repository tasks in Rust, `cargo xtask <task>`: check-docs, check-dco,
                               verify, idl extract, gen <binding>, ephemgen, rulegen, bench,
                               conformance, size, release (ADR-0014)
  crates/cli/                  the consumer-facing `teistro` binary: intl (validate, build, gen,
                               extract, analyze, ...), provider conformance kit, pack tools
  fuzz/                        cargo-fuzz targets
  fixtures/                    golden vectors with their README and the central tolerances.json:
                               baseline/ (manifest, charts/, variants/), pyjhora/, texts/, teimeris/ (moves to the
                               separate CC0 conformance repository, mounted here as a submodule, ADR-0022)
  spikes/                      throw-away experiments with a README and a result page each, workspace
                               members that are linted like the rest and never published
                               (02-binding-toolchain/ decided ADR-0007 and is the model for idl/)
  oracles/                     dev-only crates (`publish = false`) wrapping licensed or copyleft
                               references for differential tests; never in a published graph
  docs/                        this documentation (source of the docs site's concept pages)
  site/                        Fumadocs site; generated reference under site/content/reference
  examples/                    one full application per binding
```

## Coding standards

- Rust 2024 edition, stable toolchain pinned; `clippy -D warnings`;
  `rustfmt`; `cargo doc` with `-D warnings` on public items.
- `#![forbid(unsafe_code)]` in every crate except `ffi` and the `mmap`
  path of `ephemeris-de`, each with reviewed `SAFETY:` comments.
- No `panic!`, `unwrap`, `expect`, `todo`, printing or slice indexing in
  library crates (workspace lints; a tooling binary allows them with a
  comment); errors are values; iteration caps on every search.
- No `HashMap` in output-producing paths; no reads of the clock, the
  environment or the locale in computation crates; no `f64` division in a
  classification path (ADR-0016, ADR-0022; `cargo xtask check-lints`).
- `core` and `astro` are `no_std + alloc` from the first commit; the
  domain crates may require `std`.
- No bare primitive in a public signature; validated newtypes with
  unit-suffixed names at the C ABI (ADR-0023).
- Every public item documented with an example that compiles.
- Comments explain why and name the measurement, the alternative rejected
  and the defect that motivated the code.
- One source of truth per fact, and no repetition: a second copy of any
  rule, table or template is a generator or a shared helper, never a paste;
  the moment a second binding, emitter or test needs the same rule, the
  rule moves to one place (maintainer mandate, 2026-09-05).
- Named constants with citations for every astrological or astronomical
  table; no magic numbers.
- One crate per module; features for optional pieces; no cyclic
  dependencies (gate).
- Generated files carry a header naming their generator and are never
  hand-edited (gate).

## Open-source process

- Contributions under Apache-2.0 with DCO sign-off (Q18: DCO, no CLA), under
  the clean-room policy in `CLEAN_ROOM.md`; a pull request that adds a
  dependency names its licence.
- Significant changes (a new module, an API-shape change, a default
  change, a new binding) start as an RFC in `rfcs/` and end as an ADR.
- Conventional Commits prefixes (`feat`, `fix`, `perf`, `docs`, `test`,
  `ci`, `build`, `refactor`, `chore`) with bodies that state what was wrong
  and how it was found (Q19 to confirm).
- Reviews required on `main`; the fast check must be green; a release tag
  needs the full verify green.
- Issue templates require the data that reproduces an accuracy report:
  birth data, settings, provider and tier, expected value and its source.

## Build

- `cargo build` for the core and the C ABI; `cbindgen` emits the header;
  `cargo xtask idl extract` produces the API description; `cargo xtask gen
  <binding>` emits the mechanical layers; `cargo xtask verify` runs the
  gates. Everything is Rust: no Python, no `just`, no shell scripts
  (ADR-0014). Workflow YAML calls `cargo` and nothing else.
- Cross builds via `cross` and `cargo-zigbuild`; wasm via `wasm32`
  targets with `wasm-opt`; Android via `cargo-ndk`; iOS and macOS
  frameworks via a scripted xcframework build.
