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
| `darwin-x64` | `x86_64-apple-darwin` | `macos-15-intel` | `darwin` / `x64` |
| `win32-x64` | `x86_64-pc-windows-msvc` | `windows-latest` | `win32` / `x64` |
| `win32-arm64` | `aarch64-pc-windows-msvc` | `windows-11-arm` | `win32` / `arm64` |
| `linux-x64-musl` | `x86_64-unknown-linux-musl` | `ubuntu-latest`, in `alpine:3.21` | `linux` / `x64` / `musl` |
| `linux-arm64-musl` | `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm`, in `alpine:3.21` | `linux` / `arm64` / `musl` |

The short name is Node's `process.platform` and `process.arch`, and Dart's
installer builds the same string from `Abi.current()`. One name means one
artefact per platform on a release page, whichever binding is asking for
it.

Everything is built natively on its own runner. Nothing is
cross-compiled, because a cross-built library is a library nobody ran the
test suite on, and the runners are free.

**The glibc floor is 2.28.** A Linux library asks the loader for the
glibc symbol versions of the machine that linked it, so one built on
`ubuntu-latest` (glibc 2.39) refuses to load on Debian 11, RHEL 8 or
Ubuntu 22.04, and nothing in the build says so. The floor is a field of
the row (`glibc_floor`), and `package` reads every file it ships from
the version-needed table inside it (`xtask/src/floor.rs`): a file that
needs a version above the floor is not packaged. 2.28 is RHEL 8's and
Debian 10's, the oldest still supported, and the floor Node 22's own
Linux builds and `manylinux_2_28` wheels ask of a consumer already.
`check-package` packages the host, so verify's Linux rows hold the floor
on every dispatch and not only on a release.

**musl is built in Alpine.** A musl row's runner is the Linux runner of
its architecture, and the row runs inside the container the table names
(`container`), through `docker run` with the checkout mounted
(`xtask/alpine.sh`). The container is the platform, so the build is still
native and the packages are installed and run where they were built. The
runner supports JavaScript actions inside an Alpine job container on x64
only, so both rows enter the container the same way, by hand, rather than
through the job's `container:`. A musl target links its C runtime statically by default, and
a static runtime cannot make a shared library, so `.cargo/config.toml`
turns `crt-static` off for both triples, as Alpine's own Rust does. musl
has no symbol versions and so no floor to read back; the wheel is tagged
`musllinux_1_2`, which every Alpine since 3.13 meets. The loaders name a
musl host `<os>-<cpu>-musl`: Node from its process report, which names a
glibc and none under musl, and Python from `confstr`. Dart ships no musl
SDK, so `check-package` excuses the Dart package there, and verify's musl
rows leave out the parity check, which compares the Dart runner with the
rest.

## What each platform produces

`cargo xtask package <platform>` builds `teistro-ffi` and `teistro-node`
in release for that target and writes:

| artefact | what it is | who wants it |
|---|---|---|
| `libteistro_ffi-<version>-<platform>.<ext>.gz` | the shared library, gzipped and nothing else | the Dart installer, and anyone who `dlopen`s it |
| `teistro-c-<version>-<platform>.tar.gz` | `include/teistro.h`, the shared and the static library, the terms | a C, C++, Swift, Kotlin or Java consumer |
| `npm/@teistro/sdk-<platform>/` | the prebuilt addon, with `os`, `cpu` and `libc` for npm to match | npm, which installs exactly one of them |
| `npm/@teistro/sdk-wasm/` | the same layer over the wasm module, one package for every host; built once by the release's `wasm` job, not per platform (`cargo xtask package wasm`) | a browser, a worker, a bundler, or a host no addon covers |
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
  every platform package as `optionalDependencies` and carries no
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
  header and linked both statically and dynamically, and run — the
  static half only where this platform's `cc` can link an archive built
  for the Rust target's ABI, which is everywhere but Windows;
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

On Windows the C step proves half of that, and says which half. The
bundle is built for `x86_64-pc-windows-msvc` while the `cc` the gate
drives is the runner's MinGW gcc, and the two ABIs meet in the import
library but not in the static one: `teistro_ffi.dll.lib` links and runs
there like anywhere else, and `teistro_ffi.lib` — MSVC object code
carrying Rust's standard library, which wants the MSVC C++ runtime —
does not, on `__chkstk`, `__imp_NtReadFile` and `??_7type_info@@6B@`.
No set of `-l` flags closes that, so the gate prints the reason it is
skipping rather than pretending, and linking the Windows static library
under `cl` is the one gap left in this gate. The Node and Dart packages
are proved there like everywhere else.

The same mismatch is why `check-c` links the shared library by path on
Windows rather than by `-lteistro_ffi`: a searching linker finds
`teistro_ffi.lib` beside `teistro_ffi.dll.lib` and takes the static one,
turning a shared link into the link that cannot work. `binding::shared_link`
is the one place either gate decides that.

This is the only thing that tests the packages rather than the code in
them. A test that imports `../lib/index.js` cannot catch an export left
out of `files`, a subpath that resolves in a checkout and not in an
install, an addon nobody depends on, or a header the bundle forgot.
