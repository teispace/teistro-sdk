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
  LICENSE  NOTICE  README.md
```

```sh
cc -std=c11 -Iteistro-c-0.1.0-linux-x64/include app.c \
   teistro-c-0.1.0-linux-x64/lib/libteistro_ffi.a -o app
```

or against the shared library with `-L .../lib -lteistro_ffi`. Both are
compiled and run by `cargo xtask check-package` on every platform before
a release is published, out of the unpacked bundle and against this same
smoke test.

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
