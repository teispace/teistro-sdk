# The API description

Status: `generated`, 2026-09-06.

`api.json` is the machine-readable description of the SDK's C boundary,
extracted from the Rust source of the boundary crates by `cargo xtask gen
ffi` and held equal to them by `cargo xtask check-ffi` in the fast check.
Every binding's mechanical layer, the C header and the API reference are
rendered from it (ADR-0004, ADR-0007), so it ships inside every package.

What it carries per item: the type with its unit, range, example, enum
link and nullability, the documentation, the role of every parameter, the
catalogue's kinds as enums, and the layout of every result blob. The
schema is `teistro-api/1`; the model is `crates/idl/src/model.rs` and the
design page `docs/03-design/ffi-abi-and-api-description.md`.

Never edit the file: change the source it came from and regenerate.
