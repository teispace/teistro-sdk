# Architecture decision records

Status: living index, 2026-09-04. One record per decision, numbered, never
edited after acceptance (a change is a new record that supersedes the old;
a record still `proposed` may be revised). A record moves from `proposed`
to `accepted` when the maintainer confirms it in `QUESTIONS.md`.

| ADR | title | status | question |
|---|---|---|---|
| [0001](adr-0001-core-language.md) | Rust core with a stable C ABI | accepted 2026-09-04 | Q1 |
| [0002](adr-0002-ephemeris-agnostic-port.md) | Ephemeris-agnostic through a capability-negotiated port; positions required, overrides optional | accepted 2026-09-04 (revised) | Q7, Q8, Q15 |
| [0003](adr-0003-keys-not-strings.md) | Engine emits keys; localisation is data packs | accepted 2026-09-04 | Q10 |
| [0004](adr-0004-binding-generation.md) | One API description, generated bindings, parity gate; generator chosen by the Phase 0 spike | accepted 2026-09-04 | Q2, Q3 |
| [0005](adr-0005-modularity.md) | Module families as crates and packages; profiles; size gates; v1.0 is baseline parity | accepted 2026-09-04 | Q4 |
| [0006](adr-0006-licence.md) | Apache-2.0 open core; packs and adapters under their own terms | accepted 2026-09-04 | Q5 |
| 0007 | The binding generator (A or B), from the spike measurements | pending the spike | Q2 |
| [0008](adr-0008-builtin-ephemeris.md) | A built-in analytic ephemeris ships in v1 | accepted 2026-09-04 | Q7 |
| [0009](adr-0009-sdk-native-astronomy.md) | The SDK owns the astronomy layer above raw positions | accepted 2026-09-04 | Q8, Q15 |
| [0010](adr-0010-teistro-intl.md) | Teistro Intl, one opinionated localisation standard | accepted 2026-09-04 | Q10 |
| [0011](adr-0011-precision-policy.md) | Precision policy | accepted 2026-09-04 | Q13 |
| [0012](adr-0012-docs-site-fumadocs.md) | Documentation site on Fumadocs | accepted 2026-09-04 | Q11 |
| [0013](adr-0013-override-policy-and-tiers.md) | Provider override policy default; built-in ephemeris tiers; eclipses and stars in v1.x | accepted 2026-09-04 | Q17, Q22, Q23 |
| [0014](adr-0014-rust-only-tooling.md) | Everything we author is Rust, including the tooling (`cargo xtask`) | accepted 2026-09-04 | Q25 |
| [0015](adr-0015-quality-bar.md) | The quality bar is gated: tests, benchmarks, memory and leak checks are part of done | accepted 2026-09-04 | maintainer instruction |

## Template

```
# ADR-NNNN: title

Status: proposed | accepted | superseded by ADR-MMMM
Date:
Question: Qn

## Context
## Decision
## Consequences
## Alternatives considered
## Evidence
```
