# Teistro SDK

The computational foundation for astrology applications. A Teispace
product, open source under the Apache License 2.0.

Teistro SDK is a low-level astrology engine written in Rust with generated
bindings for the languages applications are written in, so that every
platform gets the same API, the same signatures and the same behaviour. It
owns the whole astronomy layer above raw planetary positions (time scales,
precession and nutation, sidereal time, the full ayanamsha catalogue, every
house system, sunrise and set, crossings), ships its own built-in
ephemeris so it works with nothing else installed, and accepts any other
ephemeris through a small port (Teimeris and Swiss Ephemeris adapters are
published separately, each under its own licence). It is modular and
tree-shakable, localised through one opinionated standard (Teistro Intl)
that anyone can add a language to without touching the core, and held to
measured claims: every accuracy and performance number in the
documentation is produced by a gate.

The project is in Phase 0, discovery and decisions. There is no code yet;
everything that exists is in `docs/`, which is organised as a map. Start at
[`docs/README.md`](docs/README.md).

| if you want to know | read |
|---|---|
| what we are building and why | [`docs/00-vision/`](docs/00-vision/01-vision.md) |
| what astrology software computes, everywhere, and what the baseline engine does today | [`docs/01-research/`](docs/01-research/README.md) |
| how it is shaped | [`docs/02-architecture/`](docs/02-architecture/00-overview.md) |
| what is decided and what is open | [`docs/08-decisions/`](docs/08-decisions/README.md), [`docs/QUESTIONS.md`](docs/QUESTIONS.md) |
| where the work stands right now | [`docs/STATUS.md`](docs/STATUS.md) |
| the plan | [`docs/07-roadmap/`](docs/07-roadmap/00-roadmap.md) |
| how to contribute | [`CONTRIBUTING.md`](CONTRIBUTING.md), [`GOVERNANCE.md`](GOVERNANCE.md), [`CLEAN_ROOM.md`](CLEAN_ROOM.md) |

## Status

| | |
|---|---|
| phase | 0, discovery and decisions |
| decided | Rust core with a C ABI and Rust-only tooling; generated, type-safe bindings with a parity gate; v1.0 is parity with the baseline engine; Apache-2.0 with a clean-room policy and a licence allow list; built-in ephemeris with a reference-accuracy path; SDK-owned astronomy layer; exact classification and dasha arithmetic; kernel-and-table designs for dashas, vargas, balas and rules; evidence ranks; a calculation version; a determinism contract with a CC0 conformance repository; Teistro Intl; a gated quality bar |
| next | the Phase 0 spikes: golden-vector export, binding toolchain (C ABI plus IDL against Diplomat), ephemeris port, Teistro Intl |

## Licence

Apache License 2.0; see [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).
Contributions are accepted under the Developer Certificate of Origin; see
[`CONTRIBUTING.md`](CONTRIBUTING.md).
