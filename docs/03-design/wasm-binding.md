# The wasm binding

Status: steps 1 (**one libm**) and 2 (**the loader compiled out**)
built 2026-09-25, and step 3 answered by step 1; the binding, steps 4
to 7, designed.

The fourth binding the order names (ADR-0004: Node native, wasm, Dart,
Python) and the last deliverable of Phase 5's list
(`07-roadmap/00-roadmap.md`: "wasm binding and the wasm column of the
determinism matrix"). Written 2026-09-25 from measurements, before any
code, as every binding page is.

## 1. What was measured

- **The ABI crate compiles for `wasm32-unknown-unknown` but for one
  thing.** `cargo check -p teistro-ffi --target wasm32-unknown-unknown`
  fails only on `libloading` (`crates/ffi/src/provider.rs`), which loads
  a native engine plugin. Every other crate, including the embedded zone
  database, the intl engine and the built-in ephemeris, compiles.
- **Nothing in the library's code panics on that target.** The target
  compiles `std::thread`, `SystemTime::now` and `Instant::now` and
  panics when they run. No library crate calls any of them. The only
  `std::fs` calls are in tooling paths (the ELP and ingest readers, the
  intl CLI and migration), and on this target they return an error rather
  than panic.
- **The Node JavaScript layer is almost all portable.** `blob.js`,
  `catalogue.js`, `messages.js` and `records.js` (5 473 lines) import no
  Node module and use no `Buffer`. `index.js` (3 520 lines) is Node-only in
  its loader (`node:module`, `node:fs`, `node:path`, `node:url`) and two
  `Buffer` calls: one copies a pack's bytes, one hex-encodes a hash. Both
  have platform spellings (`Uint8Array`, a hex loop).
- **The native surface the layer reads is the generated addon's.** 72
  `#[napi]` items in `native/src/generated.rs`, emitted by
  `crates/idl/src/emit/node.rs` as *Rust* marshalling code over the C ABI;
  the napi-specific parts of that emitter are its attributes, the
  `Buffer` and `Env` types, and how a refusal becomes a thrown error.
- **napi-rs's own wasm build is not usable for a browser SDK.** napi-rs 3
  targets `wasm32-wasip1-threads` through emnapi and
  `@napi-rs/wasm-runtime`, which needs `SharedArrayBuffer` and so a
  cross-origin-isolated page (COOP and COEP headers). A site embedding
  third-party content often cannot set them, and the architecture's rule
  for this package is "no Node built-ins on the browser path"
  (`02-architecture/07-binding-architecture.md`).

- **wasm32 computes the whole scenario, and differs from native only in
  the last bits of two sections.** Run under Node's WASI, all 481 539
  values of the nine sections: seven sections bit-identical to native
  macOS; `astro` 217 values and `houses` 4 976, 4 049 of them by one ulp
  and the worst by 507 ulps, a relative 7.9e-14 (about 6e-12° on a
  cusp). The nightly matrix reports the same two sections for Linux
  aarch64 against macOS aarch64 (139 and 1 876 values), which it prints
  and does not fail. The cause is the platform's maths library:
  ADR-0022 promises byte-identical output on macOS, Windows and wasm32
  as well as Linux, and today only the two Linux architectures, which
  share glibc, are held to it.
- **One libm makes every target agree.** Over two million inputs each,
  the pure-Rust `libm` crate gives identical bits natively and in wasm32
  for all thirteen functions the SDK calls (sin, cos, tan, asin, acos,
  atan, atan2, exp, ln, log10, powf, hypot, powi). Apple's library
  differs from it in all thirteen, and even wasm's own C library differs
  in exp, ln, pow and powi, so "std on wasm" is not enough. `sqrt`,
  `mul_add` and `rem_euclid` are exact by IEEE 754 and need nothing.
- **Its price** is 2.2× on a loop of nothing but those calls (0.33 s
  against 0.15 s for 26 million natively on Apple Silicon). The SDK's
  scenario is mostly not transcendental calls, so the price to report is
  the scenario's, measured by the instruction gate once it is built.

## 2. Determinism first: one libm on every target

Before any binding, the SDK's 175 transcendental call sites (29 files,
16 of them in `astro`) go through one module, `teistro_core::math`,
whose functions are the `libm` crate's. A lint refuses `.sin(`,
`.atan2(` and the rest anywhere else, with an exception list that fails
both ways. The hash matrix then **fails** on macOS as it does on the
two Linux architectures, gains Windows, which ADR-0022 names and the
matrix never ran, and gains the wasm32 column. The instruction gate
will read the change as a regression in every section that calls them,
and it has no way to accept a deliberate one; so the change carries the
measured price, and the gate learns an explicit, reviewed acceptance
(a file naming the section, the percentage and the reason, refused once
stale) rather than being bypassed.

**Built (2026-09-25).** `teistro_core::math` holds the thirteen, and
190 call sites in 28 files go through it; `powi` is square-and-multiply
in Rust rather than the intrinsic. The `uses-one-libm` lint reads every
crate under `crates/` from the tree, not the eight on the computation
list — the geometry, the renderer and the strengths call these too — and
was proved red both ways: a platform `.sin()` in `strength` fails it, and
so does a `lint: uses-one-libm` excuse left on a line that no longer
needs it (a check `scan` now makes for every rule it runs). **Measured
after**: the scenario built natively on macOS and for `wasm32-wasip1`
agrees on **all 481 539 values, bit for bit**, where 5 193 differed
before; every test of the twelve crates the change touches passed
unchanged. The scenario binary's `values` mode writes the file the matrix
compares, through the same `write_values` `cargo xtask hashes` uses, and
`crates/scenario/wasi.mjs` runs it under Node's WASI. The matrix fails on
every platform now and gains Windows and wasm32. `compare-bench` accepts a
cost only as an `Instruction-cost: <section> +<percent>% (<reason>)`
trailer on the pull request's own commits, held both ways.

**What the matrix did not see.** Profiling the fast check's slowest
pages afterwards found one platform cosine left, in the built-in
ephemeris's series term: written `f64::cos(x)`, the path form, which a
lint looking for `.cos(` cannot see, and which the 481 539 values
happened not to separate. The lint now reads both forms of all 27 float
functions that reach a C library, the thirteen in use and the fourteen
not yet (`exp2`, `ln_1p`, the hyperbolics and the rest), and was proved
red on the line it missed. The same profile priced the change on CI: the
Muntha page 205 s → 251 s and the annual chart 40 s → 50 s, `libm`'s
argument reduction being slower than the platform's. Two changes that
move no bit take most of it back: the built-in ephemeris builds
optimised in dev like `teistro-astro`, and a planet's position and rate
come from one `sin_cos` per term rather than a cosine and then a sine
and cosine (`series::sum_and_rate`, tested bit for bit against the two
sums over every table), 31.6 s → 25.4 s of CPU for the annual chart
locally.

## 3. The binding: a second backend of the Node glue emitter

`emit/node.rs` gains a backend: **napi** (today's) or **wasm-bindgen**.
The marshalling bodies — the unsafe calls into the C ABI, the struct
filling, the blob copies, the lent-string reads — are the same Rust for
both and are emitted once. The backend supplies the spellings that
differ:

| | napi | wasm-bindgen |
|---|---|---|
| a function or method | `#[napi]` | `#[wasm_bindgen(js_name = …)]` |
| a class and its constructor | `#[napi]`, `#[napi(constructor)]`, `#[napi(factory)]` | `#[wasm_bindgen]`, `#[wasm_bindgen(constructor)]`, a static |
| a plain object (`#[napi(object)]`) | mapped by napi | `serde` derive with `rename_all = "camelCase"`, crossed by `serde-wasm-bindgen` |
| bytes | `Buffer` | `Vec<u8>` / `Uint8Array` |
| a refusal | `napi::Error` with the status and fields | `JsError` carrying the same fields |

Both backends expose **the same `native` object**, member for member, so
`index.js` and its four pure modules serve both packages unchanged but
for the loader. That is the property worth the design: the ergonomic
layer, its decoders and its typed surface (`index.d.ts`) are written once
for JavaScript.

Rejected:

- **A JavaScript glue over the raw C exports** (the table's other option).
  It is the Python binding's shape (`ctypes` over the same ABI), but it
  is a third implementation of the marshalling, in a third language,
  where the backend above reuses the second. Its one advantage, no
  wasm-bindgen CLI, is a pinned tool in CI (ADR-0014 allows Rust tools).
- **napi-rs's WASI build**, for the cross-origin isolation above.

## 4. What differs on wasm

- **No plugin loader** (ADR-0029: "wasm gets the built-in and a host
  adapter, and not the loader"). `ts_provider_load` is compiled out on
  `target_family = "wasm"`; the description marks it native-only so the
  wasm backend emits nothing for it, and `plugin` in the context's
  options is refused with a field and a hint naming `provider`.
- **The host provider** is a JavaScript object, as in Node. The adapter
  is hand-written like Node's `provider.rs`, over `js_sys::Function`, and
  goes through `Exported` exactly as Node's does.
- **The built-in tier.** A browser consumer runs on `compact` unless it
  brings a provider (ADR-0029). The package ships the profile binaries
  ADR-0005 names, one `.wasm` per profile, and a size gate holds each.
- **Memory.** ADR-0007 finding 4: wasm results freed only by the
  collector exhausted memory after 300 depth-5 charts. Every handle keeps
  the explicit `dispose` the Node layer already has, and the test that
  walks the scenario runs past that count.
- **Single-threaded**, as the architecture says; a worker pool is the
  consumer's, one context per worker.

## 5. Gates

- `check-wasm` (verify): builds the profile binaries, runs the Node test
  suite's portable half against the wasm package in Node, and loads the
  package in a headless browser bundle with no Node built-ins resolved.
- `check-parity` gains a fifth runner, so the wasm surface is held value
  for value against Rust, Node, Python and Dart.
- The hash matrix gains its wasm column (ADR-0022): the scenario's
  digests computed in wasm32 must equal the native ones, section by
  section; a difference is a finding, recorded with its cause (ADR-0016
  names fused multiply-add contraction).
- A size gate per profile binary.

## 6. Order of work

1. One libm (section 2): `teistro_core::math`, the lint, the matrix
   failing on macOS and gaining Windows, the instruction gate's reviewed
   acceptance.
2. `libloading` behind `cfg(not(target_family = "wasm"))`, the loader's
   entry points marked native-only in the description, and a wasm32
   `cargo check` in fast-check so the target cannot rot.
   **Built.** Measured first: of the whole boundary only
   `src/provider.rs` failed to build for `wasm32-unknown-unknown`, and of
   the workspace only `teistro-node`, the napi addon. The file carries
   `#![cfg(not(target_family = "wasm"))]` and the extractor reads that
   condition off it, so `idl/api.json` marks its three functions
   `native_only`, the C header guards them with `#ifndef __wasm__` and
   their reference pages say so; any other `cfg` on an export is refused,
   since no binding could say which builds hold the symbol. The fast
   check runs **clippy** for wasm32 over every library but the addon, not
   `check`: what a `cfg` breaks is code left unused, which is a warning —
   and the first run found one, `read_options`, whose only caller is the
   loader.
3. The hash matrix's wasm column. **Answered by step 1**, whose column
   runs the scenario as `wasm32-wasip1` under Node. The browser target
   is the same instruction set with another system interface, and float
   arithmetic is IEEE in both; the scenario's maths comes from the same
   `libm` compiled for the same architecture. What `wasm32-unknown-unknown`
   adds — the binding's own build — step 7's parity runner exercises.
4. The emitter's backend split, napi output byte-identical before and
   after (the generated file is gated).
5. The wasm crate, the package and its loader; `index.js`'s two `Buffer`
   calls made portable.
6. The host provider adapter.
7. `check-wasm`, the parity runner, the size gate.
