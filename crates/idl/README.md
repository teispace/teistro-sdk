# teistro-idl

Status: `draft`, 2026-09-06.

The API description of the Teistro SDK's C boundary and the toolchain over
it (ADR-0007). The description is extracted from the Rust source of the
boundary crates and rendered into every binding's mechanical layer; the
checked-in copy is `idl/api.json`, the C header `bindings/c/include/teistro.h`,
both written by `cargo xtask gen ffi` and held by `cargo xtask check-ffi`.

| module | what it holds |
|---|---|
| `model` | the description: enums, structs, callbacks, opaques, functions, constants, blob schemas |
| `names` | the naming rules for Rust, C and the bindings |
| `layout` | C sizes, alignments and offsets on 64- and 32-bit targets |
| `rules` | parameter roles and struct kinds from types and names |
| `blob` | the `TSRB` result blob: a schema-driven encoder and decoder |
| `extract` | the extractor over Rust source (feature `extract`, on by default) |
| `emit::c` | the C header emitter |

The design page is `docs/03-design/ffi-abi-and-api-description.md`.
