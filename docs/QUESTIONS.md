# Open questions and decisions

Every question the research raised, each with options, trade-offs and a
recommendation. A decided question keeps its number, records the decision
and links its ADR. Status key: `open`, `decided`, `deferred`.

## Decided

| # | question | decision (2026-09-04) | record |
|---|---|---|---|
| Q1 | core language | Rust core, one audited `ffi` crate exposing a Teimeris-style C ABI | ADR-0001 |
| Q2 | binding generation | one description, generated bindings, parity gate; the generator chosen by the Phase 0 spike with numbers: option A (a designed C ABI, an extracted API description, our own generators in Rust), Diplomat rejected because its JavaScript and Dart backends cannot pass a host provider, marshal trees only through accessors, and serve Node only through wasm | ADR-0004, ADR-0007 |
| Q3 | binding order | Node native, wasm, Dart/Flutter, Python, Rust, then Java; Swift and Kotlin on demand; C and C++ headers from Phase 1 | roadmap |
| Q4 | v1 scope | v1.0 is baseline parity; Western and Hellenistic designed in Phase 0, shipped in v1.x | ADR-0005 |
| Q5 | licence | Apache-2.0 open core; packs and adapters under their own terms | ADR-0006 |
| Q6 | ownership of the baseline engine's content | Teispace owns the SDK and the baseline engine's interpretations, rules, names and corpora; the baseline engine will replace its packages with the SDK | this table |
| Q7 | fallback ephemeris | a built-in analytic ephemeris ships in v1 as its own module and phase | ADR-0008 |
| Q8 | who computes houses and the rest | everything is in the SDK from v1: the `astro` layer owns every computation above raw positions; providers may declare native overrides | ADR-0009 |
| Q9 | calendars in v1 | at least the baseline engine's set plus Julian, mixed, ISO week and the Indian lunisolar calendar; the rest as plug-ins later | calendar architecture page |
| Q10 | localisation | Teistro Intl, one opinionated standard in the manner of next-intl and slang | ADR-0010 |
| Q11 | docs site | Fumadocs | ADR-0012 |
| Q12 | prose conventions | British spelling; Teimeris comment discipline | guideline 01 |
| Q13 | numeric policy | precision-first `f64` with error-bounded algorithms | ADR-0011 |
| Q14 | team and infrastructure | the maintainer and this assistant; public repository | roadmap |
| Q15 | Teimeris relationship | Teimeris updated as needed | ADR-0002, ADR-0009 |
| Q16 | names | GitHub `teispace/teistro-sdk`; npm `@teistro/*`; crates `teistro-*`; PyPI `teistro`; pub.dev `teistro` and `teistro_flutter`; adapters published separately | implementation page |
| Q17 | built-in ephemeris tiers and data terms | all three tiers, `standard` default; published-series tables with citations in `NOTICE`; Pluto fitted from a public-domain JPL kernel | ADR-0013 |
| Q18 | contributor agreement | DCO sign-off, no CLA | guideline 07, `DCO` |
| Q19 | commit convention | Conventional Commits with bodies that say what was wrong | guideline 05 |
| Q20 | Teistro Intl source conventions | base locale `en-Latn`; JSON canonical with YAML accepted; `i18n/<locale>/<namespace>.json` | ADR-0010 |
| Q21 | Teistro Intl for consumer applications | yes, the same engine, CLI and typed accessors | ADR-0010 |
| Q22 | eclipses and the full star catalogue | v1.x; anchor stars and yogataras in v1.0 | ADR-0013 |
| Q23 | provider override policy default | `prefer-native`, with `sdk-only` selectable, both gated | ADR-0013 |
| Q25 | tooling language | everything we author is Rust: repository tasks as `cargo xtask`, consumer tools as Rust binaries, generators in Rust; no Python, `just` or shell scripts; only the docs site, the bindings' own-language layers and workflow YAML are not Rust | ADR-0014 |
| Q26 | exact classification and periods | `f64` astronomy stays; angles as canonical nanoarcsecond integers; classification by exact integer arithmetic with half-open boundaries; dasha spans as exact rationals; classify from what is serialised | ADR-0016 |
| Q27 | kernel and table | one kernel per family, systems as cited rows, falsified before code, whole-table invariants, a lazy dasha cursor; a system needing its own code is a kernel defect | ADR-0017, `03-design/` |
| Q28 | evidence and unsourced variants | texts outrank the baseline engine, which outranks third parties; rows carry V, T, S; unsourced variants are registered and refused, never silent defaults; discrepancies go to the cruxes register | ADR-0018 |
| Q29 | clean room and containment | `CLEAN_ROOM.md`; a licence allow list in `deny.toml` with copyleft and MPL denied; oracles unpublished; adapters outside; a test-provider-only build; Swiss adapter rules | ADR-0019 |
| Q30 | calculation version and provenance | a calculation version bumped on any numeric change; the cache key; the envelope extended with time, calendar, deviation, conventions and confidence fields | ADR-0020 |
| Q31 | reference-accuracy ephemeris | the IAU routines as an ERFA port; a JPL DE file reader provider (v1.x); a DE-refit `reference` tier; stage-isolated validation | ADR-0021 |
| Q32 | determinism and conformance | byte identity across architectures compared by hash; the corpus in a separate CC0 repository consumed as a pinned submodule; the quality bar extended (mutation, instruction counts, feature matrix, semver, compile-fail) | ADR-0022 |
| Q33 | type safety in every binding | validated newtypes, generated typed surfaces with documentation and examples in every binding, runtime schemas, coherence validation, typed states and errors, compile-fail and strictness gates | ADR-0023 |

Principle confirmed by the maintainer: the baseline engine's packages are the minimum
bar, not the model. The SDK is structured, designed and governed as a
large-scale, professional, open-source, market-ready product from the
first commit. Confirmed 2026-09-04: every API and signature in every
binding is type safe, offers suggestions, and is robust (Q33).

## Q24. Contact addresses for conduct and security reports: `open`

The code of conduct and the security policy currently direct reports to
GitHub's private channels (private vulnerability reporting; the
maintainers via the repository). A dedicated address (for example a
Teispace conduct or security mailbox) would be the usual practice.

Recommendation: create `security@` and `conduct@` mailboxes on the
Teispace domain and add them to `SECURITY.md` and `CODE_OF_CONDUCT.md`;
until then the GitHub channels stand.

## Decisions log

Decisions are recorded in the table above with the date; the reasoning is
in the linked ADR or page. A decision is reopened only by a new RFC with
new evidence.
