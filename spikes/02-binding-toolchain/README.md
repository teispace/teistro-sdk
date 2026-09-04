# Spike 2: the binding toolchain

**Question.** Which toolchain generates the SDK's bindings: A, a designed
C ABI plus an API description plus generators of our own; or B, Diplomat?
Decided with numbers on the same slice in Node and Dart. **Result:
option A** (`docs/08-decisions/adr-0007-binding-toolchain.md`).

## The slice

One context holding settings and an ephemeris port, and one batch call
that asks the port for nine positions and returns a tree-shaped result:
the classified positions (exact nanoarcsecond classification, ADR-0016)
and a Vimshottari tree to a requested depth, `9 + 81 + … + 9^depth` nodes.
The port is the one host callback the SDK needs (ADR-0002). The numbers
are stand-ins; the shapes are the SDK's.

```text
slice/        teistro-spike-slice    the domain slice, no unsafe, shared by both options
a-ffi/        teistro-spike-a-ffi    A: the designed C ABI (struct_size handshake, status codes, vtable
                                     provider, the TSPB result blob with a table of contents and columns)
a-gen/        teistro-spike-a-gen    A: the extractor (syn over a-ffi's source, roles from types and
                                     names, metadata from `api:` doc lines) and the emitters: C header,
                                     napi glue, TypeScript, blob decoder, Dart; one `common` module of rules
a-c/                                 A: generated C header
a-node/                              A: generated napi crate and TypeScript; hand-written loader, ergonomic
                                     layer with branded types, benchmark
a-dart/                              A: generated dart:ffi layer and typed classes; hand-written extension
                                     types and defaults, benchmark
b-bridge/     teistro-spike-b-bridge B: the Diplomat bridge module over the same slice
b-js/                                B: diplomat-tool's JavaScript (wasm) output, loader config, benchmark
b-dart/                              B: diplomat-tool's Dart output, benchmark
harness/                             the one timing harness per language, shared by both options
results/                             the four result files the tables below quote
api.json                             the extracted API description of option A
```

## How to run

```sh
cargo build --release -p teistro-spike-a-ffi -p teistro-spike-a-node -p teistro-spike-b-bridge
cargo build --release -p teistro-spike-b-bridge --target wasm32-unknown-unknown
cargo run -p teistro-spike-a-gen -- extract spikes/02-binding-toolchain/a-ffi/src/lib.rs spikes/02-binding-toolchain/api.json
cargo run -p teistro-spike-a-gen -- gen spikes/02-binding-toolchain/api.json spikes/02-binding-toolchain
(cd spikes/02-binding-toolchain/b-bridge && diplomat-tool js ../b-js/api && diplomat-tool dart ../b-dart/lib/src)
(cd spikes/02-binding-toolchain/a-node && node bench.mjs)
(cd spikes/02-binding-toolchain/b-js && node bench.mjs)
(cd spikes/02-binding-toolchain/a-dart && dart pub get && dart run bench/bench.dart)
(cd spikes/02-binding-toolchain/b-dart && dart pub get && dart run bench/bench.dart)
```

`diplomat-tool` 0.16.1 (`cargo install diplomat-tool --version 0.16.1`),
Node 26, Dart 3.13. The four benchmarks report the median of the best of
three rounds and the 90th percentile, in microseconds, on an Apple
Silicon laptop; Dart runs in JIT mode.

## Measurements

### Node (A: napi addon) against JavaScript (B: Diplomat over wasm)

| measurement | A median µs | B median µs |
|---|---:|---:|
| chart, depth 3 (819 nodes), built-in provider | 11.1 | 36.8 |
| chart, depth 3, provider implemented in JavaScript (9 callbacks) | 15.4 | not expressible |
| one provider callback into JavaScript (derived) | 0.47 | not expressible |
| decode the result: columns as zero-copy views | 0.67 | (opaque handle) |
| positions as 9 objects | 0.17 | 2.79 (9 accessor calls) |
| eager tree, depth 3 (819 objects) | 6.3 | 178.9 (819 accessor calls) |
| one lazy row | 0.04 | 0.25 |
| chart, depth 4 (7 380 nodes) | 105 | 1 886 |
| eager tree, depth 4 | 61 | 1 689 |
| chart, depth 5 (66 429 nodes) | 946 | 29 000 to 52 000 |
| eager tree, depth 5 | 665 | 19 775 |

Blob sizes: 18 576 bytes at depth 3, 162 912 at depth 4, 1 461 992 at
depth 5. B's depth-5 figure varies with memory pressure: each chart is an
opaque handle freed only when the `FinalizationRegistry` fires, and 300
depth-5 charts in a row exhausted the wasm memory (`rust_oom`), so its
rows run fewer iterations.

### Dart (A: dart:ffi over the C ABI) against Dart (B: Diplomat)

| measurement | A median µs | B median µs |
|---|---:|---:|
| chart, depth 3 (819 nodes), built-in provider | 10.8 (includes a blob copy) | 9.6 |
| chart, depth 3, provider implemented in Dart (9 callbacks) | 11.4 | not expressible |
| one provider callback into Dart (derived) | 0.08 | not expressible |
| decode the result: columns as views | 0.30 | (opaque handle) |
| positions as 9 objects | 0.07 | 0.23 (9 accessor calls) |
| eager tree, depth 3 (819 objects) | 14.0 | 22.2 (819 accessor calls) |
| one lazy row | (a view read) | 0.01 |
| chart, depth 4 (7 380 nodes) | 109 | 84 |
| eager tree, depth 4 | 127 | 209 |
| chart, depth 5 (66 429 nodes) | 1 093 | 937 |
| eager tree, depth 5 | 3 567 | 7 076 |

A's Dart chart includes copying the blob out of native memory into a
`Uint8List` (about 1 µs at depth 3, 150 µs at depth 5); the production
path keeps the native buffer under a `NativeFinalizer`, which is what B
does for its opaque handles.

### Code volume (lines)

| | A | B |
|---|---:|---:|
| the slice (shared) | 782 | 782 |
| the boundary crate | 955 (C ABI, blob encoder, tests) | 308 (bridge module) |
| toolchain we own | 2 894 (extractor and five emitters) | 0 (diplomat-tool) |
| hand-written Node or JavaScript layer | 304 (loader, ergonomic layer, branded types) | 11 (loader config) |
| hand-written Dart layer | 161 (extension types, defaults, chart) | 0 |
| generated for Node or JavaScript | 833 (napi glue, `.d.ts`, decoder) | 2 558 in 27 files (wasm loader, one class per type) |
| generated for Dart | 641 | 852 in 12 files |
| generated C header | 310 | not generated for this run |
| binaries | addon 576 KB, C library 406 KB | wasm 44 KB, library 408 KB |

## What the spike found

1. **Diplomat 0.16 cannot pass a host provider from JavaScript or Dart.**
   Its backends declare `callbacks = false` and `traits = false`; with the
   provider method enabled the tool answers `Lowering error in
   Context::create_with_provider: Traits are not supported by this
   backend` for both, and `&mut [f64] not supported in this backend` for
   the columnar fill. Kotlin, C, C++ and nanobind accept callbacks.
2. **Callbacks through the C ABI are cheap.** 0.47 µs per call into
   JavaScript through napi (a `FunctionRef` borrowed back inside the
   batch call), 0.08 µs into Dart through `NativeCallable.isolateLocal`.
3. **A blob beats accessors for trees by an order of magnitude in
   JavaScript** (6 µs against 179 µs for 819 nodes) and by half in Dart,
   and its decode cost is constant in the tree size because the columns
   are views. Diplomat has no recursive structs and, in these backends,
   no output slices, so a tree is one accessor call per node.
4. **Node native is not a Diplomat target**; its JavaScript is wasm, 3.5
   times slower on the same slice, and its memory is reclaimed only by the
   collector.
5. **The typed surface.** From one `api:` line per field the generators
   produced units, ranges and examples in the C header comments, the
   TypeScript declarations and the Dart classes; string enums with typed
   members; validated value classes; the hand layers added branded
   scalars (`JulianDay`, `DashaDepth`) and Dart extension types with
   validating factories. Diplomat reproduces the Rust doc comment and
   nothing structured beyond it.
6. **Diplomat's Dart output did not compile** on this slice: the variant
   `True` became the keyword `true`; a Dart-only rename attribute works
   around it. Its generated file also carries an unused import the
   analyser flags. Its JavaScript and Dart are otherwise idiomatic and
   well organised, and its finaliser pattern is adopted (ADR-0007).
7. **Errors.** A's napi layer maps a status to a JavaScript `Error` whose
   message ends with the stable code and the provider's own code; napi's
   `code` field is taken by napi's own status names, so the ergonomic layer
   owns the typed error in Phase 1. Dart gets a typed `TeistroException`
   with the status enum. Diplomat throws the error enum's name.
8. **The generators are role-driven.** Nothing in them names a function of
   the slice; roles (handle, handle out, struct in or out, vtable, user
   data, blob out) come from types and naming conventions, and the blob
   layout from constants in the boundary crate. A second entry point
   reached both bindings by re-running the generator.

## What is not covered

Python, Java, Swift and Kotlin (A reaches them through the C header;
Diplomat reaches Kotlin and Python directly); wasm through option A
(the same description and a wasm-bindgen or hand C ABI glue, Phase 5);
threading; packaging. The measurements are one machine, JIT Dart, and a
slice with one callback shape; they are enough to decide the toolchain,
not to set the performance budgets.
