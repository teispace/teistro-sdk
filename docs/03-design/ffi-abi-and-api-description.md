# The C ABI and the API description

Status: `draft`, revised 2026-09-06 when the branded quantities were
added (§5); revised the same day when the build handshake was added
(§5's handshake); revised the same day when the parity gate was added
(§8's tests); revised the same day when an ephemeris written in Dart
answered the SDK (§5's provider, §8's tests); revised the same day when
the Dart binding was added (§5's Dart layers, §8's tests); revised the same day when the Node addon
and its
ergonomic layer were added (§3.5's field and parameter roles, §5's
binding); revised the same day when the Node binding's generated layers
were added (§5, the TypeScript surface and the blob decoders);
written 2026-09-06 when `crates/ffi` (`teistro-ffi`) and
`crates/idl` (`teistro-idl`) were built from spike 2
(`spikes/02-binding-toolchain/README.md`, ADR-0007). Derives from
`02-architecture/07-binding-architecture.md`, `06-api-conventions.md`,
ADR-0001, ADR-0004, ADR-0020, ADR-0022 and ADR-0023.

## 1. Purpose and scope

One audited boundary every binding is generated against, and one
description of it every binding is generated from. The boundary is the
`ffi` crate: a C ABI over the SDK's Rust crates, built as a shared and a
static library, holding the workspace's only `unsafe` code. The
description is `idl/api.json`, extracted from the Rust source of the
boundary crates by the `idl` crate and rendered into the C header, the
Node binding's TypeScript surface and napi addon, and the Dart binding's
`dart:ffi` declarations, value classes and decoders. This page settles the ABI
conventions as built, the result blob's wire layout, what the description
carries and how it is inferred, the layout rules, and the gates that keep
the header, the description and the library equal.

What crosses today: the context and its options, the last error, keys and
ids, dates in every shipped calendar, civil times to instants with the
zone metadata and the scale conversions, the locale engine, and positions
over the ephemeris port completed into the requested frame. The chart,
dasha, panchanga and rule modules add their entry points to the same crate
as they are built (`09-guidelines/03-adding-a-module.md`, step 5).

## 2. Inputs, settings and ports

A context is built from `ts_context_options` (a shipped profile id, a JSON
settings patch, a locale, flags), an optional ephemeris provider as the
port's own `ts_provider_vtable` with its `user_data`, and nothing else; it
reads no environment. The settings knobs the boundary consumes directly:
`provider.overrides` (the completion's policy), `time.delta_t` (the Delta
T model), `time.dst_gap`, `time.dst_overlap` and `time.unknown_time` (the
zone resolution's policy). The SDK's four locales are embedded as
bundles built from `i18n/` when the crate compiles; a consumer's packs
load through `ts_intl_load_pack`.

## 3. The data model

### 3.1 The conventions, as built

| rule | how the boundary keeps it |
|---|---|
| no global state | every call but the handful of static answers (`ts_abi_version`, `ts_sdk_version`, `ts_status_message`, the fixed-day arithmetic) takes a `ts_context` first |
| the size handshake | every boundary struct begins with `uint32_t struct_size`; a caller sets it to `sizeof` on inputs and outputs; a size this build does not know is `SCHEMA_VERSION`, before anything is read or written |
| a status on every call | the core's `Status` (`ts_status`) with its stable codes; the message, detail, field, hint and message key of a failure are kept on the context and read by `ts_context_last_error`; a successful call leaves an `OK` record with a null message |
| construction failures | `ts_context_new` has no context to record on, so it writes the error's sentence into an optional owned string |
| no panic escapes | every entry point's body runs under `catch_unwind`; a caught panic is `INTERNAL` with the panic's message; an `extern "C"` function can therefore never unwind into C |
| ownership of memory | what the library allocates it frees: `ts_string` by `ts_string_free`, `ts_blob` by `ts_blob_free`; both descriptors are zeroed on free, so a second free is a no-op |
| lent strings | `ts_str` and the `const char *` fields of result structs point into the context and stay valid until the next call on that context; a caller copies what it keeps |
| one thread at a time | a context is used by one thread at a time and may be moved between threads; the bindings' pools give every worker its own |
| enums never change | every enum is `#[repr(int)]` with explicit discriminants; members are appended; a catalogue enum in the header carries `_UNKNOWN = -1` for a member from a newer library |
| a wrong shape fails the build | the extractor refuses a type it cannot carry, a name the sources do not define, a blob-returning function without a schema, and an `api:` link to nothing (`06-api-conventions.md`, rule 9) |

### 3.2 The settings patch

The settings design named a `ts_settings_patch` struct with a presence
bitmask. The boundary takes the patch as JSON instead: `settings_json` in
the options is a document whose groups and knobs are the settings
document's, every one optional, parsed into `SettingsPatch` with unknown
knobs and out-of-type values refused by name (`INVALID_ARG`, field
`settings_json`, serde's message naming the knob). The reason is rule 8
of the conventions: a `#[repr(C)]` patch struct would change layout with
every knob appended, while the JSON form is stable for the life of the
ABI and every binding's typed patch builder serialises to it. The resolved
settings come back as their canonical document (`ts_context_settings_json`)
and its hash (`ts_context_settings_hash`).

### 3.3 The boundary types

Twenty-four `#[repr(C)]` structs cross today: the port's ten (the
observer, the position request and columns, the obliquity, the horizon
and crossing requests, the crossing event, the data hash, the capabilities
and the vtable, all from `crates/port-ephemeris/src/vtable.rs`) and the
boundary's fourteen: `ts_string`, `ts_str`, `ts_hash`, `ts_blob`,
`ts_context_options`, `ts_error`, `ts_calendar_date`, `ts_civil_time`,
`ts_civil_date_time`, `ts_zone_spec`, `ts_zone_resolution`,
`ts_time_conversion`, `ts_delta_t` and `ts_intl_loaded`. Every field is
an integer, a float, a pointer or an array of bytes, so any bytes a
caller leaves in an output struct are a valid value to read the size
from; padding is explicit (`reserved` fields), so no struct has implicit
padding a caller could leave uninitialised. Units and scales are in the
names (`offset_seconds`, `instant_jd_utc`, `longitude_deg`).

The enums: the core's `Status`, the port's `Body` and `TimeScale` (both
now `#[repr]` with explicit discriminants equal to their ids), the
boundary's `TsScale` (UT1, TT and UTC), `TsResolution`, `TsZoneKind`,
`TsZoneSource`, `TsZoneEra`, `TsDst`, `TsChosen`, `TsZoneWarning` (a bit
set in `ts_zone_resolution.warnings`) and `TsDeltaTSource`, and the
catalogue's kinds: `Kind` over the kind numbers and one `uint16_t` enum
per kind whose members are the catalogue's keys and ids, so
`TS_GRAHA_SUN` is the member and `(TS_KIND_GRAHA << 16) | TS_GRAHA_SUN`
the packed key id every struct and column carries. The boundary's own
enums mirror the time crate's through exhaustive `From` conversions, so a
variant added there is a compile error here.

### 3.4 The result blob

Every tree- or grid-shaped result crosses as one allocation in the `TSRB`
layout, little-endian throughout:

```text
header   32 bytes   magic "TSRB", layout version (1), section count,
                    total length, schema id, three reserved words
table    16 bytes   per section: id, offset, length, count
sections 8-aligned  fixed:   one 8-byte slot per field, in field order
                    columns: a directory of u32 offsets from the section
                             start, one per column, then the columns, each
                             8-aligned, `count` elements each
                    bytes:   raw bytes (UTF-8 where the schema says text),
                             `count` of them
```

A decoder reads the version first and refuses one it was not generated
for with a typed error; the table lets it find a section by id without
knowing the others, so a section appended in a later minor version is
skipped by an older decoder and a section removed is a major version. The
schemas are declared once, in `crates/ffi/src/schemas.rs`, and appear in
the description; the encoder (`teistro_idl::blob::Writer`) refuses a
column of the wrong scalar or length and a blob with a section missing,
and the reference decoder (`Reader`) checks every offset before it reads.
Two schemas ship: `positions` (id 1: a fixed `summary`, the `instants` and
`bodies` columns, the `cells` columns instants outermost with a status
and a source per cell, the completion `steps` and the `provenance` as
canonical JSON) and `intl_render` (id 2: `flags`, `text`,
`resolved_from`, `warnings`).

### 3.5 The description

`idl/api.json` (`teistro-api/1`) carries the ABI and SDK versions, the
sources it was read from, and every constant, enum, opaque, callback,
struct, function and blob schema, each with its documentation and its
source file. A field carries its type, its `api:` metadata (unit, range,
example, the enum an integer stands for, nullability) and its
documentation; a function carries its parameters with their roles, its
return type, its Safety contract and the blob schema it fills; an enum
member of the catalogue carries its key. The roles, inferred from types
and names and never from a function's name:

| role | the shape |
|---|---|
| `handle`, `handle_out` | a pointer to an opaque type, or a pointer to such a pointer |
| `struct_in`, `struct_out` | a pointer to a `#[repr(C)]` struct; `out` when the parameter is named `out` or `out_*` |
| `vtable_in`, `user_data` | a pointer to a struct whose role is a vtable (`*Vtable` or `api: role=vtable`), and a `void *` |
| `blob_out`, `blob_free` | a pointer to the blob descriptor, written or consumed |
| `string_in`, `string_out`, `string_free`, `str_out` | a `const char *`, and pointers to the owned and the lent string descriptors |
| `bytes_in`, `array_in`, `length` | a `const uint8_t *` or a `const` scalar pointer, and the `len`/`*_count` that follows it |
| `scalar_out`, `value` | a `mut` scalar pointer named `out_*`, and everything passed by value |

The `api:` line is the one place a unit, a range, an example or an enum
link is written; it lives in the Rust doc comment, inside backticks, so
rustdoc shows it as code and the extractor reads it once for every
binding (ADR-0023). A line that opens with a parameter's name and a colon
describes that parameter (`` `api: calendar: enum=Calendar` ``), so an
integer parameter standing for an enum is named in the header and crosses
as a member in every binding. Rustdoc's link syntax is flattened to plain
text in the description so no binding shows a Rust path.

**A struct's fields carry roles too**, read once and used by every
emitter, so a C struct's bookkeeping never reaches a binding's surface:

| field role | how it is marked | what a binding sees |
|---|---|---|
| handshake, padding | `struct_size`, `reserved*` | nothing |
| flag | `api: flag` | a boolean |
| bit set | `api: bitset=<Enum>` | the members it holds |
| array | `api: len=<field>` | the elements, as members when the field names an enum, else numbers; the count is gone |
| count, presence | named by another field's `len=` or `present_if=` | nothing |
| optional | `api: present_if=<flag>` | the value or its absence |
| text | a `const char *` | a string |
| fixed bytes | a `[u8; N]` | a buffer |
| columns | the struct's `api: role=columns` | typed arrays the caller allocates |

So `ts_position_request` reaches JavaScript as `{ scale, frameBits,
speeds: boolean, observer?, jds: number[], bodies: Body[] }`: no counts,
no presence flag, and the bodies by their catalogue keys.

## 4. Algorithms

- **Extraction.** Every source file is parsed with `syn`; public
  `#[repr(int)]` enums, `#[repr(C)]` structs, function-pointer type
  aliases, `pub const` integers marked `api: constant` and
  `#[unsafe(no_mangle)] extern "C"` functions are collected; a public
  struct with private fields is an opaque candidate and is kept only when
  some pointer names it. Names are resolved in a second pass (a bare path
  is an enum, an opaque, a callback or a struct, else an error), roles in
  a third, links in a fourth. The catalogue's kinds come from
  `catalogue/catalogue.json`, the blob schemas from the boundary crate,
  the version from its manifest; `teistro_idl::sdk::describe` assembles
  them so `cargo xtask gen ffi` and the boundary crate's own tests
  describe the same API.
- **Names.** One module (`teistro_idl::names`): `TsPositionRequest` or
  `PositionRequestC` is `PositionRequest` to a binding and
  `ts_position_request` in C; a member is `TS_STATUS_INVALID_ARG`, a
  catalogue member keeps its key; a handle's method drops the handle's
  prefix (`ts_context_settings_json` is `settings_json`).
- **Layout.** The C rules over the description alone: each field at the
  next offset aligned to its alignment, the struct padded to the largest,
  pointers and `size_t` at the target's width, 64-bit scalars 8-aligned
  on every target the SDK ships to. The header asserts every struct's
  size on 64-bit targets with `_Static_assert`, so a compiler with other
  rules refuses the header; the boundary crate's tests hold the computed
  layout to Rust's `size_of` and `align_of` for every struct.
- **The guard.** One function runs every body: it releases the strings
  the previous call lent, catches a panic into `INTERNAL`, and records
  the outcome (status, provider code, message, detail, field, hint, key)
  for `ts_context_last_error`.
- **Provenance.** `ts_positions` stamps the SDK version, the calculation
  version (`teistro_core::envelope::CALCULATION_VERSION`), the catalogue
  schema version, the profile, the settings hash, an input hash over the
  request (scale, frame, bodies, instants, observer, speeds), the
  provider's identity and frame with the completion steps as its flags,
  the Delta T model, the leap-second and zone-database versions, and the
  content hash of the columns, all through `canonical_json`.
- **Canonical JSON.** Building the boundary surfaced a determinism defect:
  the settings hash relied on the JSON layer's map ordering, which the
  `preserve_order` feature (enabled by any crate in a build) changes, so
  a crate compiled alone and the same crate compiled in the workspace
  hashed the same settings differently. `teistro_core::envelope::canonical_json`
  now sorts every object's keys itself, and every hash the SDK reports
  goes through it (ADR-0022).

## 5. The API

Thirty-nine entry points, all in the header with their documentation:

| group | entry points |
|---|---|
| static | `ts_abi_version`, `ts_sdk_version`, `ts_catalogue_version`, `ts_default_profile`, `ts_build_info`, `ts_status_message` |
| context | `ts_context_new`, `ts_context_free`, `ts_context_last_error`, `ts_context_profile`, `ts_context_settings_json`, `ts_context_settings_hash` |
| memory | `ts_string_free`, `ts_blob_free` |
| keys | `ts_key_parse`, `ts_key_name` |
| frame | `ts_frame_canonical`, `ts_frame_pack`, `ts_frame_unpack` |
| calendar | `ts_calendar_from_fixed`, `ts_calendar_to_fixed`, `ts_calendar_convert`, `ts_calendar_month_length`, `ts_calendar_is_leap`, `ts_calendar_weekday`, `ts_calendar_jd_of_fixed`, `ts_calendar_fixed_of_jd` |
| time | `ts_time_resolve`, `ts_time_civil`, `ts_time_convert`, `ts_time_delta_t` |
| intl | `ts_intl_load_pack`, `ts_intl_set_locale`, `ts_intl_locale`, `ts_intl_has`, `ts_intl_entity`, `ts_intl_transliterate`, `ts_intl_render` |
| positions | `ts_positions` |

### The Node binding's generated layers

Five files under `bindings/node/lib/`, laid out as a package is so that
every runtime file has its own declarations and a bundler drops what an
application does not use:

| file | what it holds |
|---|---|
| `catalogue.d.ts` | every enum as a string union with the `const` table beside it: a catalogued member is its full key (`'graha.SUN'`), which is what packs, fixtures and serialised results carry, plus the `'unknown'` arm the design's §3.6 requires; any other member is its name in kebab case (`'invalid-arg'`); the `Kind` enum's members are the kind names themselves (`'avastha_baladi'`) |
| `catalogue.js` | those tables, one frozen constant per enum |
| `types.d.ts` | every boundary struct as a readonly interface, importing exactly the enums it names, with each member's documentation, unit, range and example as JSDoc; the `TeistroError` class |
| `blob.d.ts` | each result blob's decoded shape: a column section is one typed array per column plus a `length` |
| `blob.js` | one decoder per schema over the `TSRB` layout, reading columns as views over the blob's own bytes; a buffer that does not start on an eight-byte boundary is copied once rather than misread, and a wrong magic, version, length or schema id is a `TypeError` |

The addon (`bindings/node/native`, `teistro-node`) is the sixth file the
generator writes: `src/generated.rs`, a napi module with a class over the
context handle, an object per boundary struct with the `Held*` value that
owns whatever the C struct points at, the enums as the strings the tables
name, and the calls with their `unsafe` blocks. Above it,
`bindings/node/lib/index.js` is hand-written and thin: it finds the addon,
checks its ABI against the one the types were generated for, validates at
the door, fills in defaults, decodes a result on first use rather than
eagerly, and rethrows a failure as a `TeistroError` carrying the status,
the code, the detail, the field, the hint and the message key the library
gave.

### The Dart binding's generated layers

Three files under `bindings/dart/lib/src/`, rendered from the same
description as the C header, so the two agree by construction:

| file | what it holds |
|---|---|
| `catalogue.dart` | every enum as a Dart enum carrying the id the boundary uses and the key the packs and fixtures spell, the same string the TypeScript surface uses; a catalogued member gains `fullKey` (`graha.SUN`) and an `unknown` member, so a `switch` over a value from a newer library stays exhaustive (§3.6); the boundary's constants come with them, so a flag's value is never a literal in a binding |
| `ffi.dart` | the `dart:ffi` declarations that match the header name for name, the library class that looks every symbol up once, a value class per boundary struct that marshals itself into an arena, a bitset field as a `Set`, the context class with a `NativeFinalizer` over `ts_context_free`, and the exception carrying the status, detail, field, hint and message key |
| `blob.dart` | one decoder per result blob over the `TSRB` layout, each column a typed-data view over the blob's own bytes; a blob at an offset a typed list cannot start on is copied once, and a wrong magic, version, length or schema id is a `FormatException` |

Above them `bindings/dart/lib/teistro.dart` is hand-written and thin, as
the Node layer is: it finds the shared library and checks its ABI against
the one the declarations were generated for, opens contexts with the
defaults, encodes and decodes the JSON the boundary takes as text, and
adds the conveniences a generator cannot know are wanted
(`Calendar.gregorian.date(2015, 4, 14).at(hour: 0, minute: 20)`, a cell of
a position grid by its two indices, a rendered message's warnings).

Two decisions are Dart's own. The generated files carry `// dart format
off`, so the generator's layout is what ships: `cargo xtask check-ffi`
regenerates them byte for byte on a machine with no Dart toolchain, and
`dart format .` still passes over the package. And a context frees itself
when it is collected, through the finaliser, so `dispose` is the explicit
form rather than the only one.

### Quantities that cannot be swapped

A latitude and a longitude are both a `f64` at the boundary, and nothing
in C stops one being passed for the other. The description says which
quantity a number carries (`api: brand=latitude`), and each binding gives
it a type of its own:

| binding | how |
|---|---|
| TypeScript | `type Latitude = number & { readonly __brand: 'latitude' }`, with `latitude(27.7)` the only way to make one |
| Dart | `extension type const Latitude._(double value) implements double`, zero cost at run time |
| C | the header's documentation and its range; C has no way to say more |

The constructor checks the range the description states, so a latitude
beyond ±90 is refused where it is written rather than at the boundary.
The gates prove the point rather than assert it: the TypeScript consumer
marks a swapped pair `@ts-expect-error`, so the check fails if the
surface ever stops refusing it, and `bindings/dart/typecheck/wrong.dart`
is analysed on its own and must report every error it expects. That is
Phase 1's exit criterion, "a swapped latitude and longitude does not
compile in Rust or type-check in TypeScript", in the two bindings.

### The build handshake

The two halves of a binding must be one build: the library carries the
SDK, and the generated files were rendered from a description of it.
`ts_build_info` returns a static JSON object written when the library is
compiled, so asking costs nothing and the answer cannot disagree with the
library it describes: the SDK version, the ABI and catalogue versions,
the commit and whether its tree was clean, the Cargo profile, the target
triple, whether debug assertions and optimisation are on, the sanitizer
if any, and the compiler.

Each loader reads it and applies the same three rules, which each binding
states in its own language and tests with builds of its own making:

| what | outcome |
|---|---|
| another ABI, or another SDK version than the generated files carry | refused |
| a sanitizer build | refused however it was found: it answers differently and slowly and is never chosen by accident |
| an unoptimised build | refused only when the loader searched it out, because naming a path is a deliberate act and a development build is what a developer means by it |

The report is on the surface too (`buildInfo` in Node, `Teistro.build` in
Dart), because an application that stores a chart should be able to store
what computed it. The parity gate carries the SDK version, the ABI, the
catalogue version, the commit, the dirty flag and the target, so a run
where the addon and the shared library came from different trees fails
rather than passing quietly.

### A host-implemented provider

An ephemeris written in the binding's own language reaches the SDK
through the port's vtable, which is what the port exists for (ADR-0002)
and what decided the toolchain (ADR-0007, finding 1: Diplomat cannot
express it at all). The adapter is hand-written per binding, as
`02-architecture/07-binding-architecture.md` says, because each wraps its
own callback mechanism; what it does is small, because the port carries
the machinery: a Rust `EphemerisProvider` becomes a vtable through
`Exported`, whose round trip is tested bit for bit, so the adapter only
has to be that provider.

In Node (`bindings/node/native/src/provider.rs`) the provider is an
object with a `name`, the `bodies` it answers and one `positions`
callback; everything else has a default. The generated class holds the
bound provider for as long as the handle lives, because the vtable points
into it, and lends the environment for the length of each call so a
callback that escaped finds nothing to call into. Four things the
adapter settles:

| what | how |
|---|---|
| the batch shape | one call per grid, never a loop: the request carries the instants, the bodies, the scale, the frame and the observer, and the answer one value per cell |
| refusing a frame | answering with nothing is `UNSUPPORTED`, so the SDK asks again in the provider's native frame and completes the rest, every step stamped |
| what a provider may be asked | the port's own `validate` runs before the callback, so a body, an instant or a frame it did not declare is refused by name rather than left for the callback to discover |
| a failure's words | only a code crosses the C boundary, so the adapter keeps the sentence and the layer above reports it: `the ephemeris provider threw: no data for that instant` |

In Dart (`bindings/dart/lib/src/host.dart`) the provider is a class
extending `EphemerisProvider`, and the adapter binds it through
`NativeCallable.isolateLocal`, whose function pointer is callable only
from the isolate that made it. That is the boundary's contract exactly
(one context, one thread at a time), so the SDK reaching back into Dart
inside a call this isolate made is the only way it is ever reached. The
same four things are settled the same way, and two more are Dart's own:
the vtable, the capability strings and the callbacks are allocated for
the life of the binding rather than in an arena, and the context's
finaliser closes them if nobody disposes it. The codes a vtable function
returns are a described enum (`ProviderCode`), so a binding that
implements a provider never writes a number; the one exception is
`NativeCallable`'s `exceptionalReturn`, which the language requires to be
a compile-time constant and the adapter checks against the catalogue when
it binds. Python's adapter will be over `Py<PyAny>` with the GIL.

The toolchain: `cargo xtask gen ffi` writes `idl/api.json`,
`bindings/c/include/teistro.h`, the six Node files and the three Dart
files; `cargo xtask check-ffi` regenerates them all in memory and fails on
any difference, in the fast check. The crates:
`teistro-idl` (`model`, `names`, `layout`, `rules`, `blob`, `extract`
behind the `extract` feature, `sdk`, `emit::{c, ts, node, dart}`) and
`teistro-ffi`
(`context`, `keys`, `frame`, `calendar`, `time`, `intl`, `positions`,
`strings`, `blob`, `schemas`, and the private `support`).

**The frame is named, not packed by the caller.** A position request
carries the frame as 32 bits, and the packing is the port's business, so
`ts_frame` gives a C caller the fields (centre, equinox, coordinates, the
sidereal flag with an ayanamsha id, the four correction flags) and
`ts_frame_canonical`, `ts_frame_pack` and `ts_frame_unpack` move between
them and the bits. The gap was found by writing the C test below: without
them a consumer had to read the Rust source to build a request.

The profile a context uses when its options name none is
`teistro_core::settings::DEFAULT_PROFILE` (`parashari-classical`, the
texts as read), which `ts_default_profile` returns so every binding's
constructor agrees (Q34).

## 6. Errors and degenerate states

| situation | status | where the detail is |
|---|---|---|
| a null handle or a null required pointer | `INVALID_ARG` | the field names the pointer; a null handle has no record |
| a `struct_size` this build does not know | `SCHEMA_VERSION` | the message names the struct and both sizes |
| a string that is not UTF-8, a scale or kind id outside its enum | `INVALID_ARG` | the field |
| an unknown profile, key, calendar or locale | `UNSUPPORTED` | the hint lists the shipped ids, the nearest key, the calendar ids or the loaded locales |
| the Indian lunisolar calendar | `UNSUPPORTED` | it needs a context's ephemeris and is not built |
| a settings patch that does not parse or contradicts the profile | `INVALID_ARG` | serde's message naming the knob, or the coherence rule and its fields |
| positions without a provider | `CAPABILITY` | the message names the flag and the vtable |
| a provider's own failure | `PROVIDER` | `provider_code` in the last error |
| a pack that does not verify | `PACK` | the message |
| a caught panic | `INTERNAL` | the panic's message |

A missing message renders as its key with a warning, never an error; a
degenerate astronomical outcome is a cell status, never an error.

## 7. Performance budget and benchmark

The boundary's own cost is bounded by the spike's measurements
(`spikes/02-binding-toolchain/README.md`): a host callback 0.47 µs into
JavaScript and 0.08 µs into Dart, a blob decoded as views in constant
time, a depth-3 tree built eagerly in 6 µs. Budget for the entry points
above: under 1 µs over the Rust call they wrap, allocation only for the
result they hand out; `ts_positions` encodes in time linear in the cells.
The bindings' benchmarks measure and publish these when they exist
(`05-testing/01-quality-bar.md`, "FFI cost"); nothing here is measured
yet.

## 8. Tests

- The description: the extractor's sample with every role, its refusals
  (a missing ABI constant, an unknown type, a blob without a schema), the
  catalogue kinds as enums, the naming rules, the layout on both pointer
  widths, the blob round trip with every section aligned and every
  refusal, the C emitter's output for every kind of item (`teistro-idl`,
  17 tests and a doctest).
- The boundary: the static answers; the handshake and the pointer checks;
  the guard turning a panic into `INTERNAL`; the owned string and the
  blob freeing once; the parameter JSON with every tagged type; the
  schema ids unique. The ABI as a C caller: a context with every default
  and its settings, the refusals with their sentences, positions through
  the test provider decoded from the blob with the completion's steps and
  the provenance and byte-identical on a second call, the calendars with
  eras and resolutions, a Nepali birth time resolved with its metadata
  and converted between scales, the locale engine rendering typed
  parameters in Nepali, keys parsed and named with suggestions
  (`teistro-ffi`, 7 unit tests, 9 integration tests, a doctest).
- The description against the library: every struct's computed layout
  equals Rust's, the ABI version and the SDK version are the constants,
  the status enum is the core's, every entry point is described.
- The C binding's own test (`bindings/c/tests/smoke.c`, `cargo xtask
  check-c`): a consumer that uses nothing but the generated header and
  the built library, compiled under `cc -std=c11 -Wall -Wextra -Wpedantic
  -Werror` and run. It proves what no Rust test can, that a C compiler
  agrees with the header about struct layouts, enum values and the
  calling convention, and that the library links. It converts 14 April
  2015 into 1 Baisakh 2072 with its era and resolution, resolves 00:20 on
  1 January 1986 in Kathmandu to JD 2446431.274306 UTC at +05:45 under
  tzdb 2026c, renders `sdk.reason.grahaInBhava` in Nepali out of the
  render blob, reads the Sun's longitude at J2000 out of the positions
  blob's columns in a frame it built with `ts_frame_canonical`, and reads
  a refusal's detail and hint. It needs a C compiler, so it runs by hand
  and in the nightly matrix; the fast check stays Rust-only (ADR-0014).
  The header also compiles as C++17.
- The Node binding's own tests (`cargo xtask check-node`): the generated
  decoders against blobs the library really produced (a positions blob
  over two instants and three bodies, a render blob in Nepali), written
  by `cargo run -p teistro-ffi --example blob_fixtures` through the C
  ABI, with Node's own test runner and no install. They check the grid,
  the cells' statuses and longitudes, that a column is a view and not a
  copy, the steps and the provenance as the JSON the library wrote, the
  refusals (a wrong magic, a wrong version, the other schema, a
  truncation, a value that is not a `Uint8Array`), and a blob at an odd
  byte offset. Then `bindings/node/typecheck/consumer.ts` type-checks at
  maximum strictness (`strict`, `noUncheckedIndexedAccess`,
  `exactOptionalPropertyTypes`, `verbatimModuleSyntax`,
  `noPropertyAccessFromIndexSignature`, `skipLibCheck: false`): every
  wrong usage is marked `@ts-expect-error`, so TypeScript fails the check
  when a surface stops refusing one. A key of another kind, a bare member
  name, a misspelt key (TypeScript answers "Did you mean
  '\"graha.PLUTO\"'?"), a write to a result's field, an unchecked
  optional, a body given by its id, a count the boundary keeps to itself,
  a flag as an integer and a partial observer are all compile errors the
  files assert.
- The Dart binding's provider tests: an ephemeris written in Dart asked
  once for a whole grid, a provider that refuses the frame and is
  completed by the SDK (`positions:NATIVE, delta-t:SDK, obliquity:SDK,
  rotate-equatorial-to-ecliptic:SDK`), one that throws and is reported in
  its own words, a body and an instant it never declared refused by name,
  a nameless or bodiless provider refused at the door, and a short column
  refused rather than read past its end.
- The Dart binding's own tests (`cargo xtask check-dart`): the same
  scenario the other two walk, through the generated declarations and the
  ergonomic layer against the real shared library, and the generated
  decoders against blobs the library produced. Fifteen tests: the ABI and
  the versions, a context's settings and their hash and a patch that
  changes both, a refusal's status, detail, field and hint, 14 April 2015
  in Bikram Sambat with its era and resolution, a Nepali birth time
  resolved and converted back with its metadata and the same instant from
  a named offset and from local mean time, the scales with what they
  applied, positions through the test provider with the frame round trip
  and the provenance, the locale engine in Nepali and English with a
  missing message's warning, keys parsed and named. The package is also
  analysed with `--fatal-infos` and format-checked. It needs the Dart
  SDK, so it runs by hand and in the nightly matrix.
- The parity gate (`cargo xtask check-parity`): one scenario walked
  through both bindings' own ergonomic layers, each printing a report of
  `key<TAB>value` lines, and the two compared value by value. Ninety
  values: the versions, a context's profile, locale, settings hash and
  the hash of the settings document as the library wrote it, the calendar
  conversion with its era and resolution, the fixed day and the weekday,
  the zone resolution with its offset, era, source and tzdb version and
  the civil time back, the scales and Delta T, a key packed and named and
  a refusal's status, detail and hint, a rendered message hashed, the
  frame and its round trip, every cell of a two-by-three grid, the
  completion's steps, and the provenance hashed. Nothing in the gate says
  what a value should be: the point is that the two bindings agree with
  each other, so a fact written into it could only weaken it. Two numbers
  are compared as numbers within 2e-9, which is one in the last place
  both reports print and no more, so a tenth of a second in a Julian day
  is a failure. The gate is checked against itself: a value moved by
  1e-6, an extra key and a wrong integer are each reported.
- Planned: a `cargo-fuzz` target over the blob reader and every entry
  point (the quality bar's fuzzing row), and the sanitizer builds.

## 9. Localisation

The boundary introduces no message keys. `ts_intl_render` renders any key
of the embedded bundles or the loaded packs; the warnings it reports are
the engine's English sentences, as the engine defines them.

## 10. Open questions

- Q34: the default profile. `parashari-classical` is proposed as the
  texts as read; the maintainer may prefer `nepali-default` for the
  product's charts.
- Rich renderers: `ts_intl_render` hands back the plain text; the parts
  (text and markup) wait for a serialisation the bindings agree on.
- The two bindings' packaging: the prebuilt libraries per platform, the
  manifests and the publishing, which wait on the release matrix. The
  loader's half of it, the build handshake, is built (§5).
- The wasm and Python bindings, from the same description.
- A host-language provider is bound through the port's vtable with the
  same contract as a native one (callable from any thread); the bindings
  that register isolate-local callbacks keep one context per isolate.
- 32-bit targets: the layout computes for them and the tests hold the
  host's width, but the header asserts sizes for 64-bit targets only;
  the wasm binding adds the ILP32 assertions when it lands.
- The provenance's `provider.flags_used` carries the completion steps
  today; when the chart layer exists, the envelope grows the calendar and
  time stamps the design names.
