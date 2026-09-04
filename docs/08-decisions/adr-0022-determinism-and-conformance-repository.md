# ADR-0022: The determinism contract and the conformance repository

Status: accepted (maintainer, 2026-09-04); extends ADR-0015
Date: 2026-09-04
Question: Q32

## Context

Principle 10 promises byte-identical output across bindings; ADR-0011
tests cross-platform agreement "to the stated tolerance". A tolerance
hides a real divergence, and consumers who cache on the envelope need
identity, not closeness. The conformance corpus is also the project's
central claim, and a standard that can only be obtained by cloning one
engine is not a standard.

## Decision

**The contract.** For identical inputs, settings, provider identity and
data hashes, pack versions and calculation version, the serialised output
is byte-identical on x86-64 and aarch64 Linux, macOS and Windows, in
Node, and in wasm32 in Node and a browser. CI compares output hashes,
never tolerances, and any divergence fails with the first differing
field. Countermeasures, each a lint or a build flag: no fast-math;
contraction pinned, with `#[inline(never)]` boundaries on the
classification path, which is integer anyway (ADR-0016); no `HashMap` in
any output-producing path (`BTreeMap` or an insertion-ordered map); no
reads of the clock, the environment or the locale in computation crates;
one serialiser with explicit precision and never `to_string` on a float;
solvers with fixed tolerances and caps, never a time budget; parallel
reductions in a fixed order.

**The conformance repository.** The corpus lives in its own repository,
`teispace/teistro-conformance`, under CC0-1.0 with independent semantic
versioning: fixtures as JSON with a complete settings profile and the
settings hash asserted, every expected value cited to a text, a tool and
version, or an institution; tolerances in one central file keyed by field
and provider class, never per fixture; a JSON Schema; runners per binding
that emit a machine-readable report. This repository consumes it as a
version-pinned submodule, bumped deliberately. Fixtures start in
`fixtures/` here (spike 1) and move to the separate repository before
Phase 1 exits; the repository is created then. Composition target at
1.0: about 500 fixtures with about 120 edge cases (polar latitudes,
exact boundaries to the microarcsecond, leap seconds, the 1582 transition,
pre-standard-time births, Nepal's 1986 zone change, adhika and kshaya
months, stations at cusps, twins). The SDK publishes its own score and
never a score it computed for another engine.

**Quality-bar additions** (the page is updated): mutation testing on
kernel crates with at least 80% of mutants caught; instruction-count
benchmarks that fail a pull request above 3% and warn above 1%, beside
the wall-clock suite; every feature built alone and with no default
features; semantic-version checks on the public API; compile-fail tests
that prove a swapped newtype and a rule without a source do not compile;
boundary generators in property tests at plus and minus one
microarcsecond of every classification boundary; a fixture reproducing a
defect lands before the fix; rules that always fire together across the
corpus are flagged as probable duplicates.

## Consequences

- The determinism matrix runs from the first crate; wasm joins when the
  binding exists (Phase 5).
- `fixtures/` here becomes a submodule mount point; the golden-vector
  page documents the move.
- The quality bar rows are gated in the fast check where fast (lints,
  compile-fail tests, feature matrix on a sample) and nightly otherwise.

## Alternatives considered

Fixtures in-tree (simpler; not a standard); tolerance-based
cross-platform checks (hide divergence); Apache-2.0 on the fixtures
(attribution is friction for an implementation checking itself).

## Evidence

The baseline engine's corpus design: central tolerance bands, profile
hashes asserted, edge cases as first-class entries, failing entry as a
release blocker; the wasm and aarch64 floating-point differences recorded
in `01-research/platform/07-performance.md`.
