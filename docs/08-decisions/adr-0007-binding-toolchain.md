# ADR-0007: The binding toolchain

Status: accepted on the spike's measurements, 2026-09-05 (Phase 0 spike 2,
`spikes/02-binding-toolchain/README.md`); the maintainer may overturn it
before Phase 1 starts
Date: 2026-09-05
Question: Q2

## Context

ADR-0004 decided that every binding is generated from one description
and held to parity by a gate, and left the generator to a spike: A, a
designed C ABI with an API description and our own generators, the way
Teimeris is bound; or B, Diplomat, a Rust bridge crate whose tool emits
the bindings. The spike built the same slice both ways (one context, one
settings struct, one batch call returning a tree, the ephemeris port as a
host callback) into Node and Dart and measured code volume, callback and
marshalling cost, the typed surface each can emit, and how the output
reads.

## Decision

**Option A: a designed C ABI, an API description extracted from the Rust
boundary crate, and generators of our own, in Rust.** The description
carries units, ranges, examples and enum links per member, so every
binding's types and documentation come from one place (ADR-0023). The
Node binding is a napi module generated from the description; the Dart
binding is a generated `dart:ffi` layer with a generated typed class over
it; the C header is generated from the same description. Tree-shaped
results cross the boundary as a length-prefixed blob with a table of
contents and columnar sections, decoded by generated decoders as
zero-copy typed-array views.

Diplomat's output stays the bar for idiomatic quality, and three of its
mechanisms are adopted into the generators: opaque handles registered
with a `FinalizationRegistry` in JavaScript and a `NativeFinalizer` in
Dart so a dropped result frees its native memory without a call;
`@Native` leaf calls in Dart for zero-overhead entry points; one file per
type in the generated JavaScript so the package tree-shakes.

## Why

1. **The ephemeris port cannot be fulfilled from JavaScript or Dart with
   Diplomat 0.16.** Its JavaScript and Dart backends set `callbacks =
   false` and `traits = false`; the tool refuses a method taking a
   provider with "Traits are not supported by this backend". Only the
   Kotlin, C, C++ and Python (nanobind) backends accept callbacks. The
   first three bindings in the order (Node native, wasm, Dart) are exactly
   the ones that cannot pass a host provider, and a host provider is the
   port's reason to exist (ADR-0002).
2. **A callback through the C ABI is cheap in both languages.** One
   provider call from the core into JavaScript costs about 0.5 µs through
   napi; into Dart about 0.05 µs through `NativeCallable.isolateLocal`.
   Nine callbacks add 4 µs to an 11 µs chart in Node and under 1 µs in
   Dart.
3. **Trees marshal an order of magnitude faster as a blob than as
   accessor calls.** A depth-3 tree (819 nodes) becomes JavaScript objects
   in 6 µs from the blob against 179 µs through Diplomat's per-row
   accessors; the same in Dart is 14 µs against 22 µs. At depth 5
   (66 429 nodes) the blob decodes in under a microsecond as views, while
   the accessor path costs 20 ms in JavaScript and 7 ms in Dart. Diplomat
   has no spelling for a recursive struct or, in these backends, for an
   output slice, so a large result is accessor calls or nothing.
4. **Node native is not a Diplomat target.** Its JavaScript backend
   generates a wasm loader; the same slice computed 3.5 times slower
   under wasm than through the addon (37 µs against 11 µs for a depth-3
   chart), and the depth-5 chart ran out of wasm memory after 300
   charts because opaque results are freed only when the garbage
   collector fires the registry. Option A serves Node natively and wasm
   from the same description.
5. **The typed surface is ours to shape.** From one `api:` line per field
   the generators emitted documentation with units, ranges and examples in
   the C header, the TypeScript declarations and the Dart classes, string
   enums, validated value classes and, in the hand layers, branded
   scalars and extension types (ADR-0023). Diplomat's output documents
   what the Rust docs say and no more.
6. **The cost of A is a generator we own, and it is bounded.** The
   extractor and four emitters are 2 900 lines of Rust for this slice and
   grow with the number of type kinds, not with the number of entry
   points; adding an entry point to the ABI crate reached both bindings
   by re-running the generator. The hand-written layers are 304 lines for
   Node and 161 for Dart against Diplomat's bridge of 308 lines, so the
   per-binding hand cost is the same order and the mechanical layer is
   free in both.

## Consequences

- Phase 1 builds the real extractor and generators from the spike's
  design: roles inferred from types and names, metadata from `api:` doc
  lines, the blob schema from constants in the boundary crate, one
  `common` module of naming and documentation rules shared by every
  emitter. The spike's code is the model, not the code.
- The result blob (`TSPB` in the spike) becomes the designed wire
  encoding of `02-architecture/07-binding-architecture.md`: versioned,
  table of contents first, columns 8-aligned, decoders generated.
- Every binding ships a finaliser-backed handle and an explicit `dispose`
  or `free`, because a result that waits for the collector can exhaust
  memory (finding 4).
- The C header is generated and is the contract for C, C++, Java (FFM),
  Swift and Kotlin; Diplomat is not used. If a future Diplomat gains
  callbacks in its JavaScript and Dart backends and output slices, the
  comparison can be rerun on the same slice.
- Two defects to design around in Dart: a reserved word as an enum
  variant (`true`) must be renamed by the generator, and the blob copy
  from native memory into a `Uint8List` is replaced by a native
  finaliser in Phase 1.

## Alternatives considered

Diplomat (B): rejected on findings 1, 3 and 4. Per-ecosystem tools
(napi-rs, wasm-bindgen, flutter_rust_bridge, PyO3, UniFFI): five
descriptions of one API, rejected in ADR-0004. Diplomat for C, C++ and
Kotlin with A for the rest: two toolchains for one API, the parity gate
would compare two generators' opinions; rejected.

## Evidence

The spike directory: `spikes/02-binding-toolchain/`, with the four result
files under `results/` (medians of best-of-three rounds on an Apple
Silicon laptop, Node 26, Dart 3.13 JIT), the generated outputs of both
options, and the tool's refusal messages recorded in the README.
