# API conventions

Status: `draft`, 2026-09-04. Depends on Q2. These are the Teimeris design
rules extended for tree-shaped results and consumer-implemented ports.

## At the C ABI

1. No global state; every function takes a `ts_context*` first.
2. Every boundary struct begins with `size_t struct_size`, set by the
   caller for inputs and outputs; init functions take the size from the
   call site through a macro over a `_sized` symbol.
3. Every array parameter carries its capacity; parallel outputs share the
   capacity of the array before them.
4. Errors are structured (`ts_status` plus `ts_error` on the context);
   a successful call never writes a message.
5. Batch is the primary shape; scalar calls exist only in ergonomic layers.
6. Results are named fields, never positional array offsets; large results
   are returned as columnar arrays or as a length-prefixed result blob
   with a documented layout and a decoder in every binding.
7. One spelling per concept; no `v2` suffixes.
8. Enum values never change; new members are appended.
9. Every entry point has a role-annotated signature the IDL extractor
   accepts; an unrecognised shape fails the build.
10. Callbacks (ports) are vtable structs of function pointers with
    `user_data`, a `struct_size`, and a capability descriptor; the core
    never stores a callback beyond the context's lifetime.

## Naming

- C: `ts_` prefix, `ts_<module>_<verb>` (`ts_chart_foundation`,
  `ts_dasha_tree`, `ts_panchanga_day`).
- Ergonomic layers: `context.chart.foundation(...)`, `context.dasha.tree(...)`,
  `context.panchanga.day(...)`; method names identical across languages
  (camelCase where the language uses it, snake_case in Python and Rust),
  checked by the parity gate.
- Field names follow the C header's names in every binding (Teimeris rule:
  renaming 365,000 fields costs more than the computation).

## Signatures

- Inputs are request structs (or options objects) with every knob explicit;
  defaults come from the context's profile, never from the function.
- Outputs are result structs with provenance.
- Instants are Julian days with an explicit timescale field; civil times
  are typed; places are `{latitude, longitude, altitude}` with longitude
  east-positive.
- Units are stated in the field name where ambiguity exists (`lon_deg`,
  `speed_deg_per_day`, `duration_days`, `offset_min`).
- Nullability is explicit (`Option` in Rust, `null` in JS, `None` in
  Python); optional fields are always present in serialised JSON (as
  `null`), never absent (the baseline engine's lesson about shape stability).

## Versioning contract (from Teimeris `COMPATIBILITY.md`)

- Patch: no answer moves, no shape changes.
- Minor: fields appended, entry points and enum members added, packs
  extended; no default changes.
- Major: removals, reorders, default changes, rule semantics changes that
  move answers.
- Every release's changelog leads with **Numbers**.

## Deprecation

An entry point or key is deprecated for at least one minor version with a
warning in the result provenance before removal in a major version.
