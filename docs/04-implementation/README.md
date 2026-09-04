# Implementation

Status: `planned`, revised 2026-09-04. Rust is decided (ADR-0001). The
project is public and open source from the first commit, structured as a
large-scale project: governance files, RFC process, generated artefacts,
and one tool for everything.

## Repository layout

```
teistro-sdk/
  LICENSE                      Apache-2.0
  NOTICE                       third-party notices (VSOP87, ELP, CLDR, tzdb, Hipparcos)
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
    astro/                     the astronomy layer
    ephemeris-builtin/         VSOP87, ELP/MPP02, fitted Pluto; tiers as features
    siddhanta/
    calendar/ time/
    chart/ houses/ vargas/ state/ aspect/ points/
    strength/ dasha/ rules/ jaimini/ kp/ tajika/
    panchanga/ muhurta/ gochar/ prashna/ matching/ rectification/ longevity/
    remedies/ numerology/ lalkitab/ pakshi/ namakarana/ rashifal/ research/
    interpret/ intl/ serial/
    ffi/                       the only unsafe crate; C ABI; cbindgen config
    test-provider/             fixed-table ephemeris for tests
  catalogue/                   entity catalogue YAML (keys, ids, attributes, citations)
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
  fixtures/                    golden vectors: baseline/, pyjhora/, texts/, teimeris/
  docs/                        this documentation (source of the docs site's concept pages)
  site/                        Fumadocs site; generated reference under site/content/reference
  examples/                    one full application per binding
```

## Coding standards

- Rust 2024 edition, stable toolchain pinned; `clippy -D warnings`;
  `rustfmt`; `cargo doc` with `-D warnings` on public items.
- `#![forbid(unsafe_code)]` in every crate except `ffi`.
- No `panic!` in library paths; errors are values; iteration caps on
  every search.
- Every public item documented with an example that compiles.
- Comments explain why and name the measurement, the alternative rejected
  and the defect that motivated the code.
- Named constants with citations for every astrological or astronomical
  table; no magic numbers.
- One crate per module; features for optional pieces; no cyclic
  dependencies (gate).
- Generated files carry a header naming their generator and are never
  hand-edited (gate).

## Open-source process

- Contributions under Apache-2.0 with DCO sign-off (Q18 to confirm DCO
  over a CLA).
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
