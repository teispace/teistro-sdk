# The C binding

Status: `generated`, 2026-09-06.

`include/teistro.h` is the C header of the SDK's boundary, rendered from
`idl/api.json` by `cargo xtask gen ffi` and held by `cargo xtask check-ffi`.
It is the contract C, C++, Java (FFM), Swift and Kotlin consume directly
and the reference every generated binding is checked against.

The library behind it is `crates/ffi` (`teistro-ffi`), built as a shared
and a static library by Cargo. Every struct the header declares begins
with `struct_size`, which a caller sets to `sizeof` before the call; the
header asserts the sizes the library was built with on 64-bit targets.

Never edit the header: change the Rust source and regenerate.

## Installing it

A release attaches one bundle per platform,
`teistro-c-<version>-<platform>.tar.gz`:

```
teistro-c-0.1.0-linux-x64/
  include/teistro.h
  lib/libteistro_ffi.so        # and libteistro_ffi.a
                               # on Windows: teistro_ffi.dll,
                               # teistro_ffi.dll.lib, teistro_ffi.lib
  LICENSE  NOTICE  README.md
```

```sh
cc -std=c11 -Iteistro-c-0.1.0-linux-x64/include app.c \
   -Lteistro-c-0.1.0-linux-x64/lib -lteistro_ffi -o app
```

That is the whole line against the **shared** library, on Linux and
macOS. A shared library resolves its own imports, so nothing else goes
beside it — no `-lm`, no Windows import libraries, whatever the Rust
toolchain reports for the static one. At run time the loader has to find
it: `LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH` on macOS, or an
rpath you set yourself.

**On Windows, link the import library by path** —
`teistro-c-0.1.0-win32-x64/lib/teistro_ffi.dll.lib` — and put
`teistro_ffi.dll` beside the executable or on `PATH`. Not
`-lteistro_ffi`: the static library, `teistro_ffi.lib`, sits in the same
directory, and that is what a searching linker finds first — which
turns a shared link into a static one, and the section below says why
that cannot work here.

### The static library

```sh
cc -std=c11 -Iteistro-c-0.1.0-linux-x64/include app.c \
   teistro-c-0.1.0-linux-x64/lib/libteistro_ffi.a -lm -o app
```

**`-lm` is not optional on Linux.** The astronomy calls `sin`, `atan2`
and the rest; `libteistro_ffi.a` carries Rust's standard library as
object code and an archive records nothing about what it needs, so the
link line has to say. On macOS the maths functions are in libSystem and
the flag does nothing. Omitting it is a page of `undefined reference` at
link time and nothing at all at compile time.

Where an archive needs more than `-lm` — and on some targets Rust's
standard library needs a dozen more — the authoritative set for a build
is what the compiler reports:

```sh
rustc --print native-static-libs --crate-type staticlib …
```

It is not written out here because a list copied from one toolchain
version is a list that will be wrong for another. **Read it, do not paste
it:** its dialect is the Rust target's linker's, not your compiler's, and
the two are not always the same — which is the next paragraph.

**On Windows the static library needs MSVC.** The release builds
`x86_64-pc-windows-msvc`, so `teistro_ffi.lib` is MSVC-ABI object code
that wants the MSVC C++ runtime. `cl` links it. MinGW's `gcc` cannot, and
no set of `-l` flags closes the gap: it fails on `__chkstk`, on
`__imp_NtReadFile` and on `??_7type_info@@6B@`, the last of which lives
in a runtime MinGW does not have. With MinGW, link
`teistro_ffi.dll.lib` against the DLL instead.

Both lines are compiled and run by `cargo xtask check-package` on every
platform before a release is published, out of the unpacked bundle and
against this same smoke test, with the lines this page gives. On Windows
the gate prints why it is skipping the static one rather than pretending
it passed.

Every archive's SHA-256 is in `checksums.txt` on the release, in the
format `sha256sum -c` reads, and `manifest.json` carries the digest of
each library uncompressed.

## The test

`tests/smoke.c` is a consumer that uses nothing but this header and the
built library. `cargo xtask check-c` builds the library, compiles the
test with warnings as errors and runs it:

```sh
cargo xtask check-c
```

It needs a C compiler (`cc`, or whatever `CC` names), so it runs by hand
and in the nightly matrix rather than in the fast check, which needs the
Rust toolchain and nothing else (ADR-0014).
