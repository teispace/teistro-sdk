# The wasm binding

Status: **built**, 2026-09-25: steps 1 to 7, step 3 answered by step 1.
The package is released with the others. What remains is ADR-0005's
module profiles, which no binding has yet (§7).

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
   **Built.** `emit/node.rs` renders for a `Backend`, napi or
   wasm-bindgen, and every spelling that differs is one method of it, so
   the table in §3 is that `impl`. Three differences the table did not
   foresee, each met there: a **64-bit integer** crosses as a number in
   napi and would cross as a `BigInt` in wasm-bindgen, so the wasm
   signatures carry `f64` and cast (a test reads every exported wasm
   signature and finds none wide, and finds napi's, which proves the
   reading); a **refusal** needs napi's environment to build an error
   object, so the wasm prelude gives the glue napi's own `Result` and
   `Error::from_reason` and a `thrown` that sets `lastError` on a
   JavaScript `Error`, which leaves the enum conversions and every
   `Held*` body the same text for both; and the **native-only**
   functions are left out of what the wasm backend renders, with a handle
   only they take. `check-ffi` held the napi glue byte for byte through
   the change, and gates the new `bindings/wasm/native/src/generated.rs`
   the same way. A test over the real description finds both backends
   exporting the same members but `newWithProvider`; built and loaded,
   the two modules show the same 42 members but the loader's, and
   wasm-bindgen's own `free`.
5. The wasm crate, the package and its loader; `index.js`'s two `Buffer`
   calls made portable.
   **Built.** The napi loader moved out of `index.js` into `lib/addon.js`,
   and `index.js` imports its native object from `#native`, the package's
   own import map (`package.json` `imports`): the Node package maps it to
   `addon.js`; the wasm package maps it by condition, `node` to a loader
   that reads the module and compiles it synchronously, `default` to one
   that instantiates it from `new URL(…, import.meta.url)` behind
   top-level `await` — the form every mainstream bundler ships as an
   asset. So `index.js` names no Node built-in, and the layer is one file
   for both. Bytes cross as `Uint8Array` (napi's glue takes one, and a
   `Buffer` is one), hex is a loop, and `loadPack` takes an
   `ArrayBuffer` too, which is what `fetch` gives. A plugin on wasm is
   refused **per entry**, by what the loaded module can do
   (`native.Provider`) rather than a flag, with the build's target and
   what to give instead; so a chain written for both packages —
   `[{ plugin }, 'BUILTIN']` — falls back in a browser as it does where
   the file is missing.

   The package is **staged, not kept**: `cargo xtask check-wasm` writes
   it to `target/wasm/package` from the Node package's own `lib/` (every
   file but its loader), the two wasm loaders, the module bound with
   `--target web`, and a manifest *derived* from the Node package's with
   only the name, description, directory, import map and files
   overridden, so no export, version or engine can drift.

   **Released like the others** (maintainer, 2026-09-25: "wasm also
   should be published/handled as others"). `cargo xtask package wasm`
   stages it into `target/dist/npm/@teistro/sdk-wasm`, beside the
   platform packages; the release's own `wasm` job builds it once — it is
   one artefact for every host — and runs `check-wasm`, whose last step
   packs it as `npm publish` would, installs it into an empty project and
   runs the Node package's own consumer against it (`TEISTRO_PACKAGE`
   names which package that file imports, and every answer it asserts is
   the same for both). `package stage` refuses a release without it, as
   it refuses one missing a platform, and `publish` sends it with the
   platform packages. Its manifest is derived from the Node one, so
   `cargo xtask version`'s edits reach it without a line of its own.

   Found on the way: `check-node` passed its fixture directory as an
   argument after `node --test`, which reads arguments as test files and
   gives a test file none, so the suite had only ever found its fixtures
   by running from the repository root. It is `TEISTRO_FIXTURES` now,
   which the staged run needed.
6. The host provider adapter.
   **Built**, and shared rather than copied: the policy Node's adapter
   held — the declared defaults, the answer's length checks, the thrown
   sentence kept for the layer above — moved into
   `teistro_port_ephemeris::host` with its wording unchanged, Node's
   adapter was rewritten over it, and the wasm one is the same over a
   `js_sys::Function` (no environment to lend, so no call can find itself
   without one).
7. `check-wasm`, the parity runner, the size gate.
   **`check-wasm` built**, and stronger than planned: rather than a
   portable half of the Node suite, it runs **the whole Node binding
   suite, unchanged**, from inside the staged package, so the suite's
   `../lib/index.js` is the package's and its `#native` resolves through
   the package's own `node` condition — the wasm glue *and* the loader a
   consumer gets. Then **a headless Chrome** loads the staged package
   unbundled, `#native` mapped to the web loader by an import map as a
   bundler maps it, runs a probe and posts the answer back; the same
   probe under Node must give the same answer **to the bit** (longitudes
   as exact text, the settings hash, the plugin refusal). A Node built-in
   put on the browser path makes the page fail to load, which the gate
   reports: proved red. 70 of 70 pass: charts, almanacs,
   positions, the locale engine and packs, rules, drawings, providers
   written in JavaScript, refusals with their records, `dispose`. The
   `wasm-bindgen` CLI is pinned to the lockfile's library version, read
   from `Cargo.lock`, and installed under `target/tools` when the machine
   has no such version. It runs in verify's `wasm` job. The module is 8.6
   MB unoptimised at the default tier, which is what step 7's profile
   binaries and size gate are for.

   **The parity runner and the size gate, built.** `check-parity` runs
   the Node runner a second time from inside the staged package, so the
   wasm module joins the value-for-value comparison with Node, Dart,
   Python and Rust; the one line it may not share, `build-target`, is
   held to naming a wasm32 target instead (verify's Linux row builds it).

   **Measured before a size was chosen**, and the measurement overturned
   two assumptions. First, the tier was not what made the module large:
   with no built-in at all it was 8.65 MB raw, and `compact` added 0.6
   MB. Second, `compact` and `standard` were 3 KB apart because `compact`
   **could not be built** — the façade's `builtin-ephemeris` pulled in
   `standard`, every tier feature turns it on, and the richest wins
   (ADR-0028, amended; `crates/ffi/tests/tier.rs` now asks which tier was
   compiled, and was proved red on the old features). Of what remained,
   47% was the `name` section, which only a debugger reads. The module
   ships without it, from a Cargo profile of its own (`wasm`: fat LTO,
   one codegen unit, `opt-level = "s"`), chosen over three others by size
   and by a timed workload — as fast as the release build and 23%
   smaller. **From 8.6 MB to 4.77 MB, 1.95 MB to 1.33 MB gzipped**, the
   `compact` tier ADR-0029 names for a browser.

   `bindings/wasm/size.json` is the budget, held both ways: over it fails,
   and so does more than 5% under it, with the value to write, because a
   budget that loose would let the saving go unnoticed. One module, not
   one per profile: ADR-0005's profiles (`panchanga`, `kundali`, …) are
   module families no binding has yet, so the gate has one entry and the
   file names the module it measures, ready for the rest.

## 7. What is left

- **ADR-0005's module profiles**, for every binding at once; the wasm
  package then ships one module per profile and the size file gains an
  entry each.
- **`wasm-opt`** (Binaryen) is **measured and declined** (2026-09-25,
  version 133, on the 4,760,590-byte compact module). It takes 6–7% off
  the raw module and makes the **gzipped** one 6–7% larger, which is what
  a browser downloads:

  | Level | Bytes | Gzipped |
  | --- | ---: | ---: |
  | none | 4,760,590 | 1,313,475 |
  | `-Os` | 4,450,152 | 1,396,229 |
  | `-Oz` | 4,420,722 | 1,400,042 |
  | `-O3` | 4,491,544 | 1,404,436 |

  The code section shrinks 9% and compresses 10% worse (830 KB to 910 KB
  gzipped); the data section, the ephemeris tables, barely moves. A timed
  run of 140,000 positions put every level inside the unoptimised
  module's own run-to-run spread (7.6 s to 9.7 s), so no speed was
  shown to pay for a pinned tool and 7–18 s a build. Measure again if
  the module's code comes to outweigh its compressibility, and by the
  gzipped size.
- **Edge runtimes** that forbid `fetch` of the module's own URL
  (Cloudflare Workers import a module instead); a loader for them is a
  third `#native` condition, `workerd`.
