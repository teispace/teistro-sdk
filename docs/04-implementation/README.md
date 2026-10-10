# Implementation

Status: `built`, revised 2026-10-11 against the tree. Rust is decided
(ADR-0001). The project is public and open source from the first commit,
structured as a large-scale project: governance files, RFC process,
generated artefacts, and one tool for the repository's tasks.

## Repository layout

`ls` is the authority for what is in a directory; this is the map.

```
teistro-sdk/
  LICENSE                      Apache-2.0
  NOTICE                       third-party notices (ERFA BSD-3 with its provenance table, VSOP87, ELP, CLDR, tzdb, Hipparcos)
  CLEAN_ROOM.md                the clean-room policy: sources by rank, what may be taken (ADR-0019)
  deny.toml                    the dependency policy checked by `cargo deny check` in the fast check (ADR-0019)
  README.md  CONTRIBUTING.md  CODE_OF_CONDUCT.md  SECURITY.md  GOVERNANCE.md  DCO
  CHANGELOG.md                 "Numbers" first
  CODEOWNERS
  .cargo/config.toml           the `cargo xtask` alias
  rust-toolchain.toml          the stable channel, with rustfmt and clippy
  .github/
    ISSUE_TEMPLATE/            bug, feature, accuracy report (with chart data), docs
    PULL_REQUEST_TEMPLATE.md   numbers, change, verification
    workflows/                 fast-check, verify, hash-matrix, benchmarks, docs, release
  rfcs/                        the RFC process for significant changes
  Cargo.toml                   the workspace and its lints
  crates/                      one crate per module; docs/STATUS.md's crate table names each one, held to
                               the directory by `check-lints`. Among them: core (the catalogue, settings and
                               profiles, the envelope), port-ephemeris and port-timezone (the ports), astro,
                               siddhanta, ephemeris-builtin (the built-in tiers), time, calendar, the chart
                               modules, intl (with the teistro-intl command line), serial, idl, ffi (the C ABI),
                               sdk (the Rust façade), mcp (the teistro-mcp server), ephemeris-kit (the provider
                               conformance kit), scenario, test-allocator and test-adapter
  catalogue/                   entity catalogue YAML, generated into crates/core by `cargo xtask gen catalogue`
                               and gated by `check-catalogue`
  i18n/                        the SDK's own messages per locale, gated by `check-intl`
  packs/                       the reading corpora (readings/, states/) per locale, with the README that tells a
                               consumer how to build one
  idl/                         api.json, the extracted API description (`cargo xtask gen ffi`, gated by `check-ffi`)
  bindings/
    c/                         the generated header include/teistro.h and the smoke test (`check-c`)
    node/                      the napi addon (native/), the generated and hand-written layer (lib/), tests, a
                               strict consumer (typecheck/), examples and parity.mjs (`check-node`)
    wasm/                      the wasm-bindgen module (native/), its loaders, the browser, bundler and workerd
                               checks (`check-wasm`)
    dart/                      dart:ffi declarations and layer (lib/), tests, wrong usages (typecheck/),
                               examples and bin/parity.dart (`check-dart`)
    python/                    the ctypes package (teistro/), tests, wrong usages (typecheck/), examples and
                               parity.py (`check-python`)
    java/                      the Panama FFM binding: generated/ and src/, tests, examples and
                               parity/ParityRunner.java (`check-java`)
  adapters/                    outside the workspace (ADR-0019), with their README
    ephemeris-teimeris/        the port over Teimeris, from Rust and as a package per binding; published
                               separately (Teimeris terms)
    ephemeris-sweph/rust/      the port over the Swiss Ephemeris C sources; published separately (Swiss terms)
  xtask/                       the repository's tasks in Rust, `cargo xtask <task>` (ADR-0014; see Build)
  fixtures/                    the conformance corpus, a submodule of teispace/teistro-conformance pinned to a
                               tag (ADR-0022); `check-fixtures` refuses a checkout without it
  spikes/                      the spikes, each with a result page; spike 2's crates are workspace members,
                               linted like the rest and never published
  docs/                        this documentation (source of the docs site's concept pages)
  site/                        the Fumadocs site; generated reference under site/content/reference (`check-site`)
```

Named in the plan and not in the tree: a consumer-facing `teistro` command
line beyond `teistro-intl` and the kit's binary, fuzz targets, oracle
crates for differential tests, a JPL DE file reader (`ephemeris-de`,
v1.x, ADR-0021), a C++ wrapper, a Flutter plugin and one full application
per binding.

## Coding standards

- Rust 2024 edition on the stable channel (`rust-toolchain.toml`), which
  is pinned to an exact version at the first release; `rustfmt` and
  `clippy -D warnings` with `all` and `pedantic`, in the fast check.
  `cargo doc` with `-D warnings` is the standard and no workflow runs it
  yet (`05-testing/01-quality-bar.md`, "documentation").
- `unsafe_code = "forbid"` for the workspace. A crate that downgrades it
  is on `check-lints`'s unsafe inventory (the boundary, the ephemeris
  port, the MCP server, the test allocator and adapter, the Node and wasm
  addons and the adapters), and every `unsafe` block carries a reviewed
  `SAFETY:` comment.
- No `panic!`, `unwrap`, `expect`, `todo`, printing or slice indexing in
  library crates (workspace lints; a tooling binary allows them with a
  comment); errors are values; iteration caps on every search
  (`Limits::max_iterations`).
- No unordered collection in an output-producing path; no reads of the
  clock, the environment or the locale in computation crates; no `f64`
  division in a classification path; transcendental functions from
  `teistro_core::math` alone (ADR-0016, ADR-0022; `cargo xtask
  check-lints`).
- `core` and `astro` were to be `no_std + alloc`; no crate is today, and
  nothing holds it.
- No bare primitive in a public signature; validated newtypes with
  unit-suffixed names at the C ABI (ADR-0023).
- Every public item documented (`missing_docs`, an error in the fast
  check's clippy), with an example that compiles; the examples that exist
  run as doctests, and an example on every item is not yet held.
- Comments explain why and name the measurement, the alternative rejected
  and the defect that motivated the code.
- One source of truth per fact, and no repetition: a second copy of any
  rule, table or template is a generator or a shared helper, never a paste;
  the moment a second binding, emitter or test needs the same rule, the
  rule moves to one place (maintainer mandate, 2026-09-05).
- Named constants with citations for every astrological or astronomical
  table; no magic numbers.
- One crate per module; features for optional pieces; no cyclic
  dependencies (cargo refuses one).
- Generated files carry a header naming their generator and are never
  hand-edited: each is regenerated in memory and compared by its `check-`
  gate.

## Open-source process

- Contributions under Apache-2.0 with DCO sign-off (Q18: DCO, no CLA),
  held on every pull request by `cargo xtask check-dco` (the fast check's
  `dco` job), under the clean-room policy in `CLEAN_ROOM.md`; a pull
  request that adds a dependency names its licence.
- Significant changes (a new module, an API-shape change, a default
  change, a new binding) start as an RFC in `rfcs/` and end as an ADR.
- Conventional Commits prefixes (`feat`, `fix`, `perf`, `docs`, `test`,
  `ci`, `build`, `refactor`, `chore`) with bodies that state what was wrong
  and how it was found (Q19).
- `main` is protected: a change lands by pull request with the
  `fast-check` status green, as a rebase merge onto a linear history. A
  release runs the full verify matrix itself (`release.yml` calls
  `verify.yml`), and nothing is published past a red one.
- Issue templates require the data that reproduces an accuracy report:
  birth data, settings, provider and tier, expected value and its source.

## Build

Every repository task is `cargo xtask <task>`, a Rust binary in `xtask/`
reached through the alias in `.cargo/config.toml` (ADR-0014).
`cargo xtask` with no task prints every command it takes.

- **Build and test**: plain `cargo build` and `cargo test --workspace`;
  the adapters under `adapters/` are outside the workspace and build on
  their own.
- **Generators**: `gen ffi` extracts the API description (`idl/api.json`)
  from the boundary crates and emits from it the C header and every
  binding's generated layer (the Node addon's glue and declarations, the
  wasm glue, the Dart, Python and Java declarations, decoders and
  catalogue tables). `gen catalogue`, `gen calendars`, `gen time` and
  `gen intl` regenerate the catalogue, the Bikram Sambat table, the
  Delta T and leap-second tables and the typed messages. Each has a
  `check-` gate that regenerates in memory and refuses a difference.
- **Measured pages**: each falsification pass writes its page under its
  own name (`cargo xtask vargas`) and is gated by `check-<pass>`;
  `accuracy`, `conformance` and `sizes` write `ACCURACY.md`,
  `CONFORMANCE.md` and `SIZES.md`.
- **Bindings**: `check-c`, `check-node`, `check-wasm`, `check-dart`,
  `check-python`, `check-java` and `check-rust` build each binding against
  the real library and run its suite and examples; `check-parity` compares
  them value by value. These need a toolchain beyond Rust, so they run in
  `verify.yml` rather than the fast check.
- **Packaging and release**: `package [TARGET]`, `package wasm` and
  `package stage` build and stage what a release ships; `check-package`
  installs the artefacts into throwaway projects and runs a consumer;
  `version`, `check-versions`, `check-tag` and `changelog-entry` hold one
  version and the release notes; `publish maven` uploads to Maven Central.
  A Linux glibc row links against its floor with `cargo-zigbuild`, a musl
  row builds natively in Alpine (`xtask/alpine.sh`), and every library and
  addon is built through the pinned `cargo-auditable`. The wasm module is
  produced by the `wasm-bindgen` command line at the lockfile's version;
  `wasm-opt` was measured and declined. Android and iOS targets are not
  built.
- **Measurement**: `hashes` and `compare-hashes` (the determinism matrix),
  `bench` and `compare-bench` (instruction counts under callgrind, Linux).
- **What a workflow runs**: each workflow step is either a `cargo xtask`
  task or the toolchain's own command (`cargo fmt`, `cargo clippy`,
  `cargo test`, the `cargo deny` action); beside them, the workflows
  install their pinned tools (`pip install` of the zigbuild requirements,
  the npm and Valgrind installs), and the release's publish job calls each
  registry's own client (`npm`, `dart pub publish`, the PyPI action, `gpg`
  for the Maven signature). `check-lints` holds that every gate is run by
  some workflow and that every workflow file parses.
