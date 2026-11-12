# The adapter gate, and moving the Teimeris adapter out

Status: `built`. Step 1 on 2026-11-11, steps 2 to 4 on 2026-11-12.

Teimeris may be used only in Teispace's own products, so its adapters
become private and the public SDK stays engine-agnostic: the ephemeris
port, the conformance kit, the built-in ephemeris, the Swiss Ephemeris
adapter (`adapters/ephemeris-sweph`) and gates that name no engine.
Teimeris stays first-class through the generic gate, run against the
private adapter on the maintainer's machine.

## The gate

```sh
cargo xtask check-adapter --adapter DIR     # or TEISTRO_ADAPTER_DIR=DIR
cargo xtask relock-adapter --adapter DIR    # the adapter's Cargo.lock again
```

`DIR` is laid out by language: `rust/` (a standalone crate with a
`cdylib`, a `<package>-kit` binary and a committed lock), and the
packages `node/`, `python/`, `dart/`, `flutter/` and `java/`. What the
layout cannot say is listed in `rust/Cargo.toml`, never spelled in the
SDK:

```toml
[package.metadata.teistro-adapter]
engine = "teimeris"                        # what the engine's manifest reports
library-variable = "TEISTRO_TEIMERIS_ADAPTER"  # what its own packages read
java-module = "com.teispace.teistro.teimeris"
engine-checkout = "../.."                  # whose commit a measurement records
embedded-env = { TM_EMBED_TIER = "MODERN" }  # a build with its data inside
embedded-unset = ["TEIMERIS_LIB_DIR"]
```

It runs the lock check, the adapter's tests, its kit, and then
`check-node`, `check-python`, `check-dart` and `check-java` with
`TEISTRO_ADAPTER_DIR`, `TEISTRO_ADAPTER_LIBRARY` (the built library),
`TEISTRO_ADAPTER_ENGINE` and the adapter's own `library-variable` set.
Those gates then type-check or analyse the adapter's packages against
the SDK, and each binding's plugin test loads the library, checks the
engine it reports, that it brought its own operations, each with a
signature, and that Nepal's official preset takes its sunrise from it.
Without an adapter they check only the SDK; nothing in this tree is
taken as one.

## Each gate that measured over Teimeris

| gate | verdict |
|---|---|
| `check-teimeris-adapter`, `relock-teimeris-adapter` | replaced by `check-adapter` and `relock-adapter` |
| `check-node`, `check-python`, `check-dart`, `check-java`: the adapter's packages | parameterised by the adapter |
| `check-package`: the adapter's npm package installed and run | parameterised: `TEISTRO_ADAPTER_LIBRARY` and the adapter |
| `measure-mobile` (the mobile sizes record) | parameterised: the adapter's crate, stem, embedded build and engine checkout |
| `app-speed-device` | parameterised: the adapter's crate, `<package>-app-speed` and embedded build |
| `measure-ab` and `bench/ab.mjs` | parameterised: `TEISTRO_ADAPTER_LIBRARY` |
| `check-ab`, `check-agreement`, `check-pluto`, `check-accuracy`, `check-batching`, `check-topocentric`, and the moon and VSOP floors | stay public: each renders committed records and builds no adapter. A re-record runs the adapter's binary in the private repository, which writes into the SDK checkout it links |
| `check-engine` (`engine`: the passthrough page, `dispatch.rs` and the typed façades) | moved: it reads Teimeris's IDL and writes Teimeris's own code (`use teimeris::sys`) |
| `check-wasm` step 8, Teimeris on the web | moved: the engine's own wasm module, its variables and its day hash |
| `check-guides`: the adapter READMEs and consumer files it holds | moved: the adapter's READMEs are in its repository |
| `check-lints`: the adapter in the `unsafe` inventory | moved with the crate, which keeps `unsafe_code = "deny"` itself |

### The three fast-check steps that measured over Teimeris

Each was judged by what it measures: the SDK, which a public gate should
hold over an engine anyone can build, or Teimeris, which belongs to the
private gate.

- **`check-teimeris-ranges` measures Teimeris: moved.** It renders the
  engine's own data blocks, their files, sizes and digests, from the
  manifest the adapter compiles in, and writes the web provider's
  `ranges.js`. Nothing of the SDK is in it. It is `teistro-gen ranges`
  and `teistro-gen check` in the private repository now, with its page.
- **`check-mobile-sizes` measures the SDK: stays, already generic.** Its
  rows are the SDK's own slices, with the built-in and without it, which
  is the measurement the step gates; the adapter column is whatever adapter
  `measure-mobile` was given (`--adapter`, through the generic gate), and
  the page now names it from the record's prose rather than from a path in
  this tree. The step renders a committed record and builds nothing, so it
  needs no engine on CI. Its rows were measured over Teimeris's adapter,
  and a re-record runs `measure-mobile --adapter <the private adapter>`.
- **`check-app-speed` measures the SDK over a native engine: stays, as a
  rendered record; re-pointing it was tried and refused.** Its budgets are
  the performance page's, written for a native provider, and it gates the
  two setups over Teimeris. Re-pointing the gate at the built-in fails on
  the measurement already in the records: the Android emulator's
  `muhurta` is 474 ms over the built-in against a ceiling of twice 200 ms,
  so the budget and the built-in disagree by design (the page says the
  built-in "is held to accuracy, not to the native budget"). The Swiss
  Ephemeris adapter cannot stand in either: it is built from sources a
  user brings, which CI does not have, and it has no `app-speed` binary.
  So the step keeps rendering the committed records, which name no code;
  a re-record runs the private adapter's `<package>-app-speed` on a
  desktop, or `cargo xtask app-speed-device ROW` with `TEISTRO_ADAPTER_DIR`
  naming it on a device. Whether a public budget over a native engine
  should exist at all is the open question this leaves.

### What stayed public on purpose: `crates/astro/tests/teimeris_*.rs`

The five tests read recorded oracle tables from the conformance corpus
(`fixtures/teimeris/`, CC0) and link no engine. They are the SDK's
measurement of its own astronomy, and they write the measured column of
`05-testing/ACCURACY.md` for sidereal time, precession, nutation, the
ayanamsha catalogue, the star table, the house systems, the phenomena and
the equation of time. Moving them would have emptied those rows, so they
stay; the recorders that wrote the tables are the adapter's, and moved.

## How the private repository runs it

The adapters live in the Teimeris repository as `teistro/`, beside the
engine, and are Teispace-proprietary: every package is marked never
publishable. They reach the SDK through one link, `teistro/sdk`, made by
`teistro/sdk.sh`:

- **the pin** is the commit in `teistro/SDK_REV`, the one place it is
  recorded; without an override `sdk.sh` clones
  `https://github.com/teispace/teistro-sdk` at that commit, with its
  corpus, and links it;
- **the override** is `TEISTRO_SDK_DIR=<an SDK checkout>`, which links that
  checkout instead.

The Rust crates depend on the SDK by path through the link
(`../sdk/crates/...`), as the Dart and Flutter packages, the Node package's
types and the tests' fixtures do, so one variable moves all of them.
`teistro/gate.sh` runs the generators' check, then this repository's
`cargo xtask check-adapter --adapter <teimeris>/teistro` from the linked
SDK, then Teimeris on the web. Before each SDK batch merges, a queue job
runs it with `TEISTRO_SDK_DIR` at the batch's worktree, so an SDK change
that breaks the adapter fails before it reaches `main`; then `SDK_REV`
moves to the merged commit (`teistro/README.md` there says how).

## For a consumer

An app that took the Teimeris adapter from this repository takes it from
the Teimeris repository, which Teispace's products can read. The web
provider (SR-67) moved from `adapters/ephemeris-teimeris/node/web.js` here
to `teistro/node/web.js` there; its API is unchanged:
`teimerisWeb(engine, options)` over the engine's own `bindings/wasm`
module, the same `files` for years outside 1800 to 2399, each checked
against the same published size and digest. To move:

1. add the Teimeris repository as a submodule, or a git dependency, at a
   commit, the same commit for the engine's wasm module and the adapter;
2. point the import at `teistro/node/web.js` (or keep importing
   `@teistro/ephemeris-teimeris/web` and resolve that name to
   `teistro/node`);
3. move the pin with the SDK's: that commit's `teistro/SDK_REV` names the
   SDK commit it was gated against.

Nothing is published to a registry, as before.

## Step 4: what was audited

`adapters/README.md` gives one licence line per adapter (it said Apache
while the Teimeris packages said AGPL), the README and `NOTICE` name
Teimeris as private, and `check-lints`' `no-private-engine` fails on a
Teimeris path or API anywhere in the tracked tree but its history files.
`fixtures/teimeris/`, the recorded oracle tables, is the conformance
corpus's (CC0, its own repository): reference values, not the engine, so
it stays.
