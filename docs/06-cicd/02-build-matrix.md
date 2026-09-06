# The build matrix

Status: `built`, 2026-09-06.

The platforms the SDK ships a build for, what each one is called, and what
it produces. The table is `PLATFORMS` in
[`xtask/src/platform.rs`](../../xtask/src/platform.rs): the packager, the
loaders, the version gate and both workflows read it, so adding a platform
is adding a row there and repeating it in the two workflow matrices.

## The platforms

| platform | Rust target | runner | npm `os`/`cpu`/`libc` |
|---|---|---|---|
| `linux-x64` | `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `linux` / `x64` / `glibc` |
| `linux-arm64` | `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | `linux` / `arm64` / `glibc` |
| `darwin-arm64` | `aarch64-apple-darwin` | `macos-latest` | `darwin` / `arm64` |
| `darwin-x64` | `x86_64-apple-darwin` | `macos-13` | `darwin` / `x64` |
| `win32-x64` | `x86_64-pc-windows-msvc` | `windows-latest` | `win32` / `x64` |

The short name is Node's `process.platform` and `process.arch`, and Dart's
installer builds the same string from `Abi.current()`. One name means one
artefact per platform on a release page, whichever binding is asking for
it.

Everything is built natively on its own runner. Nothing is
cross-compiled, because a cross-built library is a library nobody ran the
test suite on, and the runners are free.

**Next row: musl.** `Platform` already carries `libc`, the platform
packages already declare it, and npm already refuses a glibc package on a
musl host with it; what is missing is the two rows and the `musl-tools`
step. Until then an Alpine host installs no platform package and the
loader says which one it wanted.

## What each platform produces

`cargo xtask package <platform>` builds `teistro-ffi` and `teistro-node`
in release for that target and writes:

| artefact | what it is | who wants it |
|---|---|---|
| `libteistro_ffi-<version>-<platform>.<ext>.gz` | the shared library, gzipped and nothing else | the Dart installer, and anyone who `dlopen`s it |
| `teistro-c-<version>-<platform>.tar.gz` | `include/teistro.h`, the shared and the static library, the terms | a C, C++, Swift, Kotlin or Java consumer |
| `npm/@teistro/sdk-<platform>/` | the prebuilt addon, with `os`, `cpu` and `libc` for npm to match | npm, which installs exactly one of them |
| `teistro-<version>-<platform>.json` | every file above with its size and its SHA-256, and the library's digest uncompressed | the merge, and anyone checking a download |

Gzip alone, rather than an archive, for the library a binding fetches:
unpacking it needs the decompressor every language already has, and no tar
reader in the installer. A `.tar.gz` for the C bundle on every platform,
Windows included, because Windows has shipped `tar` since 2018 and one
archive format is one code path and one line of documentation.

The archives are written with no owner, no modification time and one file
mode, so two runs of the same source produce the same bytes. The digests
that matter are recorded for the *uncompressed* library as well, so a
consumer verifies the bits it will load rather than the framing they
arrived in.

## What is staged once

`cargo xtask package stage` runs after every platform's manifest is in
`target/dist`. It merges them, and writes:

- `manifest.json`, every platform's artefacts and digests;
- `checksums.txt`, in the format `sha256sum -c` reads;
- `npm/@teistro/sdk/`, the package a consumer installs, which depends on
  all five platform packages as `optionalDependencies` and carries no
  addon of its own;
- `pub/teistro/`, the Dart package, with `lib/src/prebuilt.dart` rewritten
  from the merged manifest so that its installer checks a download against
  a digest taken from the build.

A release stages every platform. `--partial` stages what one machine
built, for trying the packaging out; it says so in what it prints, and the
release workflow never passes it.

## Proving it works

`cargo xtask check-package` packages this host, stages, and then installs
what it built into three throwaway projects under `target/dist/check`:

- the C bundle unpacked, `bindings/c/tests/smoke.c` compiled against its
  header and linked both statically and dynamically, and run;
- both npm packages packed with `npm pack` exactly as `npm publish` would,
  installed into an empty project, and
  [`consumer.mjs`](../../bindings/node/packaging/consumer.mjs) run there —
  copied into the project, because a script inside this repository would
  resolve `@teistro/sdk` to the repository itself;
- a Dart project depending on the staged package, `dart run
  teistro:install --from` the archive the release would publish, and
  [`consumer.dart`](../../bindings/dart/packaging/consumer.dart) run with
  nothing in the environment to help it find the library.

Each consumer asserts the four facts the C smoke test prints — the Bikram
Sambat date, the resolved instant and zone, the rendered Nepali message,
and the Sun's longitude at J2000 — so a package that loads but answers
differently fails here rather than in the field.

On Windows the C step skips: the bundle is built with MSVC and the gate
drives a `cc`-style compiler, which the runner does not have in a form
that links an MSVC `.lib`. The Windows bundle is therefore built and
recorded but not compiled against, which is the one gap in this gate;
the Node and Dart packages are proved there like everywhere else.

This is the only thing that tests the packages rather than the code in
them. A test that imports `../lib/index.js` cannot catch an export left
out of `files`, a subpath that resolves in a checkout and not in an
install, an addon nobody depends on, or a header the bundle forgot.
