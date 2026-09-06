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
