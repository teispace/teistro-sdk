# The adapter gate, and moving the Teimeris adapter out

Status: `planned`, 2026-11-11. Step 1 is built; steps 2 to 4 are the plan.

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
engine-checkout = "../../../../teimeris"   # whose commit a measurement records
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
Without an adapter they check only the SDK. Until step 3, a gate with
nothing named falls back to `adapters/ephemeris-teimeris` when it is in
the tree.

## Each gate that measured over Teimeris

| gate | verdict |
|---|---|
| `check-teimeris-adapter`, `relock-teimeris-adapter` | replaced by `check-adapter` and `relock-adapter` |
| `check-node`, `check-python`, `check-dart`, `check-java`: the adapter's packages | parameterised by the adapter |
| `check-package`: the adapter's npm package installed and run | parameterised: `TEISTRO_ADAPTER_LIBRARY` and the adapter |
| `measure-mobile` (the mobile sizes record) | parameterised: the adapter's crate, stem, embedded build and engine checkout |
| `app-speed-device` | parameterised: the adapter's crate, `<package>-app-speed` and embedded build |
| `measure-ab` and `bench/ab.mjs` | parameterised: `TEISTRO_ADAPTER_LIBRARY` |
| `check-app-speed`, `check-mobile-sizes`, `check-ab`, `check-agreement`, `check-pluto`, `check-accuracy`, `check-batching`, `check-topocentric`, and the moon and VSOP floors | stay public and unchanged: each renders committed records and builds no adapter. A re-record runs the adapter's binary from the private repository |
| `check-engine` (`gen engine`: the IDL page, `dispatch.rs` and the typed façades) | moves: it reads Teimeris's IDL and writes Teimeris's own code (`use teimeris::sys`, `TeimerisEngine`) |
| `check-teimeris-ranges` | moves: the engine's data blocks, read from and written into the adapter |
| `check-wasm` step 8, Teimeris on the web | moves: the engine's own wasm module, its variables and its day hash |
| `check-guides`: the adapter READMEs and consumer files it holds | moves: the private repository holds its own READMEs |
| `check-lints`: the adapter in the `unsafe` inventory | moves with the crate |

## How the private repository runs it

The adapter lives in the Teimeris repository as `teistro/`, beside the
engine, and is Teispace-proprietary. Its crates reach the SDK by path
through one directory, `teistro/sdk`, a checkout of the public SDK at a
pinned revision (`teistro/SDK_REV`). From that checkout:

```sh
cargo xtask check-adapter --adapter ../teistro    # from teistro/sdk
```

`engine-checkout` becomes `../..`. The repository's own CI runs this
locally under `act`, as every private repository does. Before each SDK
batch merges, a queue job points `teistro/sdk` at the batch's worktree
and runs the same command, so an SDK change that breaks the adapter
fails before it reaches `main`; then `SDK_REV` moves to the merged
commit.

## Step 2: what moves

To `teimeris/teistro/`, pinned to an SDK revision:

- the six packages, `adapters/ephemeris-teimeris/{rust,node,python,dart,java,flutter}`;
- `crates/astro/tests/teimeris_*.rs`, which compare the SDK with the engine;
- the generators that write Teimeris's own code and pages: `engine`
  (with `facade`) and `teimeris-ranges`, with
  `docs/03-design/engine-passthrough-measured.md` and
  `docs/05-testing/teimeris-ranges-measured.md`;
- `check-wasm`'s Teimeris-on-the-web step.

The consumer gets a migration note: `adapters/ephemeris-teimeris/node/web.js`
moves to the private repository with the same API (SR-67).

## Step 3: what is deleted from the public tree

- the workspace's `exclude` entry for `adapters/ephemeris-teimeris/rust`
  (`Cargo.toml`), and the in-tree fallback in `xtask/src/adapter.rs`;
- verify's `teimeris` job, and the fast-check steps for
  `check-teimeris-ranges` and `check-engine`;
- the adapter entries in `check-guides` (`READMES`, `EXECUTED`,
  `IMPORTS`) and in `check-lints`' `unsafe` inventory;
- the `engine`, `teimeris-ranges` and `facade` modules and their rows in
  `PASSES`.

## Step 4

`adapters/README.md` gets one licence line per adapter (it said Apache
while the packages said AGPL), the documentation names Teimeris as a
private engine only, and the public tree is audited for anything else of
Teimeris's that belongs private (`fixtures/teimeris/`, the recorded
oracle tables, is the first to judge).
