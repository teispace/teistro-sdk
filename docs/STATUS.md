# Status

The living tracker. Read this first in any session; update it before ending
one. It answers four questions: what is done, what is being done now, what
comes next, and what happened in each session.

**Project phase:** Phase 0, Discovery and decisions; the decisions are
made, the spikes are next.
**Repository:** https://github.com/teispace/teistro-sdk (public,
Apache-2.0, created 2026-09-04). `main` is protected: pull requests with
the `fast-check` status, linear history. Changes land by branch, pull
request (the `dco` and `fast-check` jobs), rebase merge.
**Last updated:** 2026-09-04, end of the founding session.

## How to resume

1. Read this file, then `QUESTIONS.md` (every decision, one open question).
2. The local checkout is the repository root; `cargo xtask check-docs`
   must pass before any commit; commits are signed off (`git commit -s`)
   with Conventional Commits subjects.
3. The next task is spike 1 below. Its output goes to `fixtures/baseline/`
   in this repository; the export script itself is written inside the
   baseline engine's own repository, in that repository's language, and
   is not committed there unless the maintainer asks.

## Done

- 2026-09-04: In-depth analysis of the baseline engine's backend (the reference and
  minimum bar). Recorded in `01-research/baseline-engine/`.
- 2026-09-04: Surface read of Teimeris (the default native ephemeris and
  the model for rigour and generated bindings).
- 2026-09-04: Web research on competitor feature sets and platform
  technology (Diplomat, UniFFI, ICU4X, MessageFormat 2, docs frameworks,
  WebAssembly, next-intl, slang, VSOP87 and ELP theories, Astronomy
  Engine, Swiss sidereal modes).
- 2026-09-04: Documentation written and revised: 86 pages (vision,
  research, architecture, roadmap, decisions, guidelines, living files).
- 2026-09-04: Twenty-five questions put to the maintainer and all
  decided; fifteen ADRs, fourteen accepted (0007, the binding generator,
  waits for the spike).
- 2026-09-04: The quality bar made binding (`05-testing/01-quality-bar.md`,
  ADR-0015) and landed through pull request #1, which proved the `dco` and
  `fast-check` path; `main` is at two commits.
- 2026-09-04: Open-source scaffolding: Apache-2.0 licence, `NOTICE`,
  `DCO`, Contributor Covenant 2.1, security policy, governance,
  `CODEOWNERS`, issue and pull request templates, RFC process, the Rust
  workspace with the `xtask` crate holding the documentation gate
  (`cargo xtask check-docs`) and the DCO check (`cargo xtask check-dco`),
  and the `fast-check` workflow (format, lint, tests, gates). Repository
  created and configured.

## Decided (all on 2026-09-04)

Rust core with a C ABI. Generated bindings with a parity gate, generator
chosen by spike. v1.0 is baseline parity with Western and Hellenistic
designed in. Apache-2.0 open core. Teispace owns all baseline engine content and
the baseline engine will migrate onto the SDK. A built-in analytic ephemeris in three
tiers (`standard` default) ships in v1 as its own phase. The SDK owns the
entire astronomy layer above raw positions; provider overrides
`prefer-native` by default. Calendars at least at the baseline engine's level in v1.
Teistro Intl as the single localisation standard (base locale `en-Latn`,
JSON, `i18n/<locale>/<namespace>.json`), offered to consumers too.
Fumadocs. British spelling. Precision-first `f64`. Two-person team,
public repository. Teimeris updated as needed. Names as recommended.
Binding order Node native, wasm, Dart/Flutter, Python, Rust, Java. DCO and
Conventional Commits. Eclipses and the full star catalogue in v1.x.
Everything we author is Rust, tooling included (`cargo xtask`). The
quality bar (tests, benchmarks, memory and leak checks, size and coverage
gates) is part of "done" for every module (ADR-0015).

## Now

- Phase 0 spike 1: the golden-vector export script in the baseline engine
  repository (50 charts: foundation, positions, houses, vargas, dignities,
  a panchanga day, a Vimshottari tree, with settings and versions).

## Next

1. Spike 2, binding toolchain: A (C ABI plus IDL plus generators, reusing
   Teimeris's extractor) against B (Diplomat), one slice with the positions
   callback into Node and Dart, measured; result as ADR-0007.
2. Spike 3, ephemeris port: the port trait with positions-only
   requirement, the Teimeris vtable adapter, a `sweph` host adapter, frame
   completion for one body, the conformance kit prototype.
3. Spike 4, Teistro Intl: `_meta.json`, two namespaces in `en-Latn` and
   `ne-Deva-NP`, the MF2 subset parser, `validate`, `build`,
   `gen --target ts,dart`, a sliced pack's size.
4. Phase 1 design pages in `03-design/`: core catalogue, settings and
   profiles, ephemeris port, time and calendar, Teistro Intl engine.
5. Q24: conduct and security mailboxes on the Teispace domain.

## Session log

| date | what happened |
|---|---|
| 2026-09-04 | The baseline engine analysis, Teimeris survey, competitive and platform research, docs written. Twenty-three questions compiled and decided by the maintainer the same day. Architecture revised for the astronomy layer, the built-in ephemeris and Teistro Intl; roadmap restructured into ten phases; governance and scaffolding written; the tooling made Rust-only (`xtask`) before the founding commit was finalised; repository `teispace/teistro-sdk` created public with the docs as the first commit and `main` protected. Next: spike 1, the golden-vector export from the baseline engine. |
