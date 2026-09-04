# Core language

Status: `research`, 2026-09-04. Feeds Q1. The candidates are the ones the
maintainer named (Rust, C, C++) plus Zig for completeness.

## What the core has to be good at

The SDK is not an ephemeris. Its code is mostly: data tables (entities,
rules, interpretation keys, matrices), rule evaluation, tree-shaped results
(dasha trees, chart slices), searches over the ephemeris port (root finding,
grids), string-free computation with keys, and a large API surface that must
cross an FFI boundary safely in both directions (the consumer implements the
ephemeris). The language question is therefore about type-safe data
modelling, safe FFI, tree-shaking granularity, WebAssembly, and the binding
tools available, more than about arithmetic speed, where all four candidates
are equal.

## Comparison

| criterion | Rust | C | C++ | Zig |
|---|---|---|---|---|
| memory safety at the FFI boundary and in rule evaluation over untrusted data packs | guaranteed outside `unsafe`; panics can be caught at the boundary | manual; every array with a capacity, every struct with a size (Teimeris discipline) | RAII helps; UB still reachable | safety by convention and runtime checks in debug |
| modelling large closed vocabularies (entities, states, keys) and tree results | enums with data, exhaustive matching, `const` tables, serde | enums as ints, tables as arrays, no exhaustiveness | enums and templates, verbose | tagged unions, comptime tables |
| ephemeris port as an interface the consumer implements | traits with object safety; C vtable at the ABI | function-pointer vtable | abstract classes or vtable | vtable structs |
| tree-shaking unit | cargo features and crates; dead code removed by the linker; ICU4X data slicing model exists | compile-time defines and separate objects | same as C plus templates bloat | comptime and lazy analysis (only referenced code compiles) |
| WebAssembly | first class (wasm32 targets, wasm-bindgen, wasi) | via Emscripten or wasi-sdk (Teimeris does this) | same as C | first class |
| binding generators | Diplomat (C, C++, JS/TS, Dart, Kotlin; Python in progress), UniFFI (Kotlin, Swift, Python, Ruby), napi-rs, wasm-bindgen, flutter_rust_bridge, PyO3, cbindgen; Java FFM over the C ABI | hand IDL and generators (Teimeris built its own) | SWIG, hand | hand |
| calendars, locale data, time zones in the ecosystem | ICU4X (`icu_calendar`, `icu_locale`, plural rules, decimal formatting), `calendrical_calculations`, `jiff` (bundled tzdb) | none comparable; would be written | ICU4C (heavy) | none |
| testing and quality tooling | proptest, insta snapshots, criterion, cargo-fuzz, Miri, sanitizers, cargo-deny, cargo-audit, cargo-vet | sanitizers, libFuzzer, hand harnesses (Teimeris) | same as C | built-in test runner, sanitizers |
| performance | equal to C with LTO; no runtime, no GC | reference | equal | equal |
| binary size | good with `panic=abort`, LTO, `opt-level=z`; std pulls in some size; `no_std` possible for the core | smallest | larger with templates | small |
| compile times | slow on large crates; mitigated by crate splitting (which the module design wants anyway) | fast | slow | fast |
| team familiarity | to confirm (Q14); Teimeris's Rust binding exists, so some | Teimeris is C, so yes | ? | unlikely |
| reuse of Teimeris tooling | C ABI via cbindgen means the IDL extractor and generators can be reused | direct | direct | via C header |
| interop with Teimeris (a C library) | trivial (bindgen or hand `extern "C"`) | direct | direct | direct |
| risk | learning curve, `unsafe` at the boundary must be audited, Diplomat's Swift gap | every safety property is manual and has to be gated; data modelling is painful at this scale (900 rules, 38 tables) | complexity, UB, two ways to do everything | small ecosystem, pre-1.0 |

## Recommendation

**Rust for the core, exposing a stable C ABI** with Teimeris's ABI
conventions (`struct_size` first in every boundary struct, explicit
capacities, structured errors, batch shapes). Reasons, in order:

1. The SDK is data and rules. Rust's enums, exhaustive matching, `const`
   tables and serde make a 900-rule, 38-table, 60-dasha codebase checkable
   by the compiler; in C the same is arrays of ints and discipline.
2. The consumer implements the ephemeris. Callbacks from the core into
   untrusted foreign code are exactly where memory errors happen; Rust
   contains them to an audited boundary.
3. ICU4X is the architectural model for localisation and calendars and it is
   Rust; `icu_calendar` and `calendrical_calculations` give Hebrew, Hijri,
   Chinese, Persian, Ethiopian and Coptic for free later.
4. Tree-shaking maps onto cargo features and crates; the ICU4X data-slicing
   model (datagen, baked or blob providers) is directly reusable for locale
   packs.
5. WebAssembly and Node are first class; Dart, Python, Kotlin, Swift and
   Java all have mature routes.

C remains the right choice for Teimeris, because it is a port of a C library
whose oracle is C. Nothing here argues for changing that.

## Mitigations for the risks

- Compile times: one crate per module from the start (the module DAG is a
  Cargo workspace).
- `unsafe`: confined to the `ffi` crate and audited; Miri and sanitizers in
  CI; `#![forbid(unsafe_code)]` in every domain crate.
- Diplomat's Swift gap: UniFFI or a hand-written Swift layer over the C
  ABI if a native iOS consumer appears; Flutter covers mobile first.
- Team familiarity: the Phase 0 spikes double as onboarding; the Teimeris
  Rust binding is a worked example in the same house.
