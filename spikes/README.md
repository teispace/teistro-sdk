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
  from pedantic lints) with a comment saying so.
- Nothing in `crates/` may depend on a spike.
- Dependencies obey `deny.toml` like everything else.

| spike | question | result |
|---|---|---|
| [`02-binding-toolchain/`](02-binding-toolchain/README.md) | which binding toolchain, A (C ABI plus API description plus our generators) or B (Diplomat), for the same slice in Node and Dart, with numbers | option A; ADR-0007, 2026-09-05 |

Spike 1 (the golden-vector export) produced data, not code, and lives in
`fixtures/` with its result page in `docs/05-testing/01-golden-vectors.md`.
