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

The SDK is built and heading for its first release; nothing is published
yet. It is reached from Rust, C, Node, Dart, Python, Java and WebAssembly,
and the documentation in `docs/` is organised as a map. Start at
[`docs/README.md`](docs/README.md), and for installing,
[`site/content/docs/install.mdx`](site/content/docs/install.mdx).

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
| stage | before the first release: version `0.0.0`, every package marked unpublishable until a release is cut |
| built | the astronomy layer and the built-in ephemeris; charts, vargas, houses, balas and dashas; the panchanga, the Bikram Sambat calendar, festivals and muhurta; rules for yogas and doshas; matching; Tajika annual charts and prashna; KP; the Hellenistic and Western techniques; rectification; interpretation in English and Nepali through Teistro Intl |
| held by | generated pages a gate keeps true: [`ACCURACY.md`](docs/05-testing/ACCURACY.md), [`CONFORMANCE.md`](docs/05-testing/CONFORMANCE.md), [`SIZES.md`](docs/05-testing/SIZES.md), and one scenario compared value by value across every binding |
| next | the dated plan to the release candidate in [`docs/07-roadmap/`](docs/07-roadmap/00-roadmap.md), then review by the maintainer, astrologers and researchers |

## Licence

Apache License 2.0; see [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).
Contributions are accepted under the Developer Certificate of Origin; see
[`CONTRIBUTING.md`](CONTRIBUTING.md).
