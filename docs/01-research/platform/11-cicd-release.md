# CI/CD and release engineering

Status: `research`, 2026-09-04. Feeds `06-cicd/`. The model is Teimeris's
`verify.sh` (40 checks), its release workflow (addon, wasm, packages,
publish jobs) and its rule that CI is a release gate because a full matrix
costs money.

## Pipelines

| pipeline | trigger | what it runs |
|---|---|---|
| fast check | every push and pull request | format, lint (`clippy` deny warnings), unit tests on Linux, generated-artefact diff, dependency graph gate, docs link check |
| full verify | nightly and on tag | the whole matrix: Linux (glibc and musl), macOS (arm64 and x64), Windows (if in scope), wasm; sanitizers and Miri; conformance and parity; fuzz smoke; benchmarks against baselines; size gates; install checks per binding; docs examples |
| release | tag `vX.Y.Z` | build every binding's artefacts per platform, run install checks from the built artefacts in throwaway projects, generate accuracy, conformance and size reports, sign, attach provenance and SBOM, publish (registries or hand-over per the licence decision) |
| docs | on tag and on docs changes | build the site from `docs/` and the generated reference; deploy |
| scheduled | weekly | dependency audit, fuzz corpus run, upstream drift check for Teimeris and tzdb versions |

## Build matrix per binding

| binding | artefacts | tooling |
|---|---|---|
| C ABI | static and shared libraries per target, header, IDL | cargo with `cbindgen`, cross via `cross` or zig-based linkers |
| Node native | prebuilt addons for darwin-arm64, darwin-x64, linux-x64 (glibc and musl), linux-arm64, win-x64; N-API so one addon serves Node 18 to 24 and Electron; source fallback | `napi-rs` or the IDL-generated N-API layer with `node-gyp`-free CMake or cargo build |
| wasm | per-profile wasm plus JS glue and types; browser bundle check | `wasm-pack` or wasm-bindgen with `wasm-opt`; a gate that bundles for a browser (Teimeris's lesson) |
| Python | wheels per platform (manylinux, musllinux, macOS, Windows) with the shared library; sdist | `maturin` (PyO3) or `cibuildwheel` with ctypes bindings |
| Dart and Flutter | a Flutter plugin building the native library for Android (NDK, arm64 and x64), iOS (xcframework), macOS, Linux, Windows, web (wasm) | `cargo-ndk`, `cargo-xcode` or CMake integration via `flutter_rust_bridge`'s build tooling or a hand `build.rs` |
| Rust | crates | `cargo publish` (or vendored) |
| Java | JAR with native libraries per platform, FFM bindings | Gradle, `jextract`-free generated bindings |

## Versioning and compatibility

- Semantic versioning with Teimeris's contract: patch never changes an
  answer or a shape; minor may append fields and add entry points and enum
  members; major may remove, reorder or change a default.
- Every artefact carries `buildinfo.json` (version, commit, dirty flag,
  profile, modules, packs, compiler, flags, hashes).
- The changelog leads with **Numbers**: which outputs moved and by how much,
  measured by the conformance run against the previous release.
- One version across all packages; a gate checks every version site.

## Cost control

A full macOS matrix costs real money (Teimeris measured about four dollars
per run, mostly macOS). The fast check runs everywhere; the full verify runs
nightly and on tags; a Linux container script reproduces the Linux matrix
locally.
