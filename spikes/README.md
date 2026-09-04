# Spikes

Throw-away experiments that answer a question with a measurement. A spike
is not shipped, not published and not the start of a module: when it has
answered its question the answer goes into a decision or a design page and
the code stays here as the evidence, until the page that cites it no
longer needs it.

Rules:

- Every spike has a `README.md` stating the question, the slice it builds,
  how to run it, and the measurements, filled in when it ends.
- Spike crates are workspace members with `publish = false`, so they
  build, format and lint with the rest of the repository; they may relax
  the library lints (unsafe code in an FFI crate, generated code exempt
  from pedantic lints) with a comment saying so. The one exception is an
  adapter for a licensed engine, which ADR-0019 keeps outside the
  workspace (`exclude` in the root `Cargo.toml`) as a standalone crate
  depending on the spike, never the reverse.
- Nothing in `crates/` may depend on a spike.
- Dependencies obey `deny.toml` like everything else.

| spike | question | result |
|---|---|---|
| [`02-binding-toolchain/`](02-binding-toolchain/README.md) | which binding toolchain, A (C ABI plus API description plus our generators) or B (Diplomat), for the same slice in Node and Dart, with numbers | option A; ADR-0007, 2026-09-05 |
| [`03-ephemeris-port/`](03-ephemeris-port/README.md) | does one port shape (positions required, overrides declared, one C vtable, frame completion, one conformance kit) carry Teimeris, the Swiss Ephemeris and a test provider at no cost and under the containment rules | yes; `docs/03-design/ephemeris-port-and-adapters.md`, 2026-09-05; Delta T becomes a table plus a model |
| [`04-teistro-intl/`](04-teistro-intl/README.md) | does one localisation standard (JSON sources, `MessageFormat 2` with SDK functions, a base locale, validation, packs, typed accessors) hold up on real English and Nepali content, and what does each piece cost | yes; `docs/03-design/intl-engine-and-packs.md`, 2026-09-05; four conventions changed (stable syntax, no sidecar, entity gender selection, source text in packs) |

Spike 1 (the golden-vector export) produced data, not code, and lives in
`fixtures/` with its result page in `docs/05-testing/01-golden-vectors.md`.
