# Vision

Status: `draft`, 2026-09-04. Written from the maintainer's brief and the
research in `01-research/`.

## The one-paragraph version

Teistro SDK is the computational foundation for astrology software: a
low-level engine with generated bindings for every platform an application is
written on, exposing one API with one set of signatures and one behaviour
everywhere. It computes charts, strengths, periods, rules, timings, matches and
interpretations across traditions, starting with the Vedic surface that the baseline engine
has today and growing to the Western, Hellenistic and other traditions that
the world's astrology software covers. It is ephemeris-agnostic, modular to
the point that an application ships only what it uses, localised through data
packs that anyone can add to, and held to the same standard of measured
claims and generated artefacts that Teimeris established.

## Why

**The baseline engine proved the demand and exposed the cost.** It replaced the
traditional tools (Kala, Parashara's Light, Sky Vision) with a modern,
accurate, feature-rich product. Its engine, seven TypeScript packages, works,
but it is bloated, its internationalisation is ad hoc and expensive, its
separation of concerns is thin, and it can only ever be consumed from Node.
Every new Teispace product would have to carry that engine, and no outside
developer could.

**Teimeris proved the model.** One C core, one machine-readable description of
the API, every binding generated, every claim measured, every artefact gated.
The same discipline applied one layer up, to the astrology on top of the
ephemeris, is what this project is.

**Nothing like it exists.** The open-source astrology libraries are single
language monoliths (Python, C#) bound to one ephemeris; the commercial tools
are desktop applications; the JavaScript ecosystem is wrappers around Swiss
Ephemeris with astrology written per app. An SDK that any application, on any
platform, can build on, with its own ephemeris and its own languages, is an
open position.

## Who it is for

- **Teispace products**: the baseline engine's backend, which migrates onto the SDK and
  deletes its `packages/` folder; Flutter and web applications that need the
  same computations on device.
- **Developers building astrology applications** of any scale who want to
  call a function and get a correct, localised, documented result without
  becoming ephemeris and calendar experts.
- **Researchers and practitioners** who need batch computation, rule
  searches and reproducible results with the settings recorded.

## What it is

- A core library that computes, and nothing else: no HTTP, no database, no
  UI, no process-wide state.
- A set of ports the consumer fulfils: the ephemeris (required), and optional
  calendar, timezone, geo and locale-data providers, with default
  implementations shipped as separate packages.
- A module catalogue where each module is independent, versioned and
  removable, so a panchanga widget does not ship the rectification engine.
- A localisation system where the engine emits keys and packs turn keys into
  text, so a language is data that anyone can author and ship.
- Bindings generated from one description, with a parity gate that fails when
  a capability is reachable in one language and not another.
- Documentation that is derived from the sources of truth or gated against
  them, and a docs site that ships with the SDK.

## What it is not

- **Not an ephemeris.** It never links one. Teimeris is the default provider
  and is packaged separately; Swiss Ephemeris and others are adapters the
  consumer picks.
- **Not an application.** No screens, no accounts, no storage.
- **Not a single school.** Where traditions disagree the SDK offers the named
  variants and a profile picks; it never silently chooses one.
- **Not an oracle.** Interpretation text is data with citations, shipped as
  packs; the engine's outputs are computations, and the boundary is explicit.

## Success looks like

- the baseline engine runs on the SDK with its `packages/` folder deleted, at equal or
  better accuracy, measured against golden vectors from the old engine.
- A Flutter app and a Node service compute byte-identical results from the
  same inputs through their own bindings.
- A third party adds a language, a calendar and an ephemeris without touching
  the SDK's source.
- Every performance and accuracy claim in the docs is produced by a gate.
