# The binding surface, measured

Status: `generated` by `cargo xtask surface` over the API description
extracted from the boundary crates, 2026-09-08. Do not edit:
`check-surface` regenerates this page and fails on any difference. The
design written from it is [`python-binding.md`](python-binding.md).

## 1. What a binding must marshal

The description carries 2 exported constants, 87 enums of 937 members in
all, 1 opaque handle type, 8 callback types, 25 structs, 39 entry points
and 3 result-blob schemas, extracted from 16 source files. A binding's
mechanical layer is a rule per **role**, not a rule per entry point,
which is why a third binding costs what it costs.

| parameter role | how often |
|---|---|
| `value` | 19 |
| `handle` | 25 |
| `handle_out` | 1 |
| `struct_in` | 9 |
| `struct_out` | 12 |
| `vtable_in` | 1 |
| `user_data` | 1 |
| `blob_out` | 2 |
| `blob_free` | 1 |
| `string_in` | 9 |
| `string_out` | 2 |
| `string_free` | 1 |
| `str_out` | 5 |
| `bytes_in` | 1 |
| `array_in` | **none** |
| `length` | 1 |
| `scalar_out` | 8 |

| struct role | how many |
|---|---|
| `object` | 20 |
| `owned_string` | 1 |
| `borrowed_string` | 1 |
| `blob` | 1 |
| `vtable` | 1 |
| `columns` | 1 |

| proposed rule | verdict | measured |
|---|---|---|
| every parameter role the description defines has an instance | falsified | 1 of 17 disagree; unused: array_in |
| every struct role has an instance | **holds** | 0 of 6 disagree |
| a struct a caller fills carries the `struct_size` handshake | **holds** | 17 of 25 structs carry it |

Some roles have no instance at all — 1 of 17, namely `array_in` —
and an emitter that wrote a rule for one of them would be shipping a
rule nothing exercises. The Python emitter **refuses** such a parameter
by name at generation time instead, which turns a silent wrong
marshalling into a build that stops and says which parameter it could
not place.

## 2. What a binding may not spell

ADR-0007 recorded a defect found by hand: Diplomat emitted a Dart enum
member called `true`, which is not a Dart identifier. The Dart emitter
has renamed reserved words ever since, and nothing has ever counted what
that rule catches or asked the same question of another language. This
section asks it of all three, over every identifier each must spell —
a member of an enum, a field of a struct, a parameter, and the name a
call gets in the binding.

| target | identifiers | members | fields | parameters | calls |
|---|---|---|---|---|---|
| Dart | 1234 | 1 | 0 | 0 | 0 |
| TypeScript | 297 | 0 | 0 | 0 | 0 |
| Python | 1234 | 0 | 1 | 2 | 0 |

| proposed rule | verdict | measured |
|---|---|---|
| no identifier the Dart emitter writes is a reserved word there | **holds** | 1 caught and renamed, 0 left; 1234 looked at |
| no identifier the TypeScript emitter writes is a reserved word there | **holds** | 0 caught and renamed, 0 left; 297 looked at |
| no identifier the Python emitter writes is a reserved word there | **holds** | 3 caught and renamed, 0 left; 1234 looked at |
| renaming leaves no two names alike in one scope in Dart | **holds** | 0 of 1234 disagree |
| renaming leaves no two names alike in one scope in TypeScript | **holds** | 0 of 297 disagree |
| renaming leaves no two names alike in one scope in Python | **holds** | 0 of 1234 disagree |

What Dart renames:

- ChartKind: `return` → `returnValue`

What Python renames:

- TsTimeConversion: `from` → `from_`
- ts_time_convert: `from` → `from_`
- ts_intl_transliterate: `from` → `from_`

The two languages are caught in **different places**, which is the
result worth having. Dart spells a member as a camel-case identifier, so
a variant called `Return` collides and the rule fires on the member;
Python spells a member as its catalogue key, upper-cased, and Python's
keywords are all lower-case, so the collision the Dart emitter has to
repair cannot arise there at all. Python is caught instead where Dart is
not — on `from`, a hard keyword there and a contextual word in both
the others — and it is caught on a **parameter** and a **field**,
which the Dart rule never touches. A single list of reserved words
shared by the two emitters would have found neither.

The Dart rule fires on 1 of the names it spells and the Python rule on
3, and no name of either is left reserved once the rule has run.

## 3. What layout a binding must agree with

The generated C header asserts every struct's size at compile time, so a
C consumer that disagrees does not build. A `ctypes` binding has no such
moment: it declares the fields and trusts the interpreter to lay them
out the way the compiler did. The numbers are therefore measured here
and carried into the generated Python, where a test asserts `sizeof`
against them on the machine the library was actually built for.

| struct | 64-bit | align | 32-bit | pointers |
|---|---|---|---|---|
| `ts_observer` | 24 | 8 | same | no |
| `ts_position_request` | 72 | 8 | 56 | yes |
| `ts_position_columns` | 80 | 8 | 44 | yes |
| `ts_obliquity` | 32 | 8 | same | no |
| `ts_horizon_request` | 64 | 8 | same | no |
| `ts_crossing_request` | 96 | 8 | same | no |
| `ts_crossing_event` | 24 | 8 | same | no |
| `ts_data_hash` | 24 | 8 | 16 | yes |
| `ts_capabilities` | 112 | 8 | 72 | yes |
| `ts_provider_vtable` | 72 | 8 | 40 | yes |
| `ts_string` | 24 | 8 | 12 | yes |
| `ts_str` | 16 | 8 | 8 | yes |
| `ts_hash` | 32 | 1 | same | no |
| `ts_blob` | 24 | 8 | 12 | yes |
| `ts_context_options` | 32 | 8 | 20 | yes |
| `ts_error` | 56 | 8 | 36 | yes |
| `ts_frame` | 16 | 4 | same | no |
| `ts_calendar_date` | 24 | 4 | same | no |
| `ts_civil_time` | 12 | 4 | same | no |
| `ts_civil_date_time` | 44 | 4 | same | no |
| `ts_zone_spec` | 32 | 8 | same | yes |
| `ts_zone_resolution` | 48 | 8 | 40 | yes |
| `ts_time_conversion` | 56 | 8 | same | yes |
| `ts_delta_t` | 32 | 8 | same | yes |
| `ts_intl_loaded` | 32 | 8 | 24 | yes |

| proposed rule | verdict | measured |
|---|---|---|
| a struct with no pointer is the same size on every target | **holds** | 0 of 10 disagree |
| every struct is the same size on every target | falsified | 12 of 25 disagree |
| every struct's layout is computable from the description alone | **holds** | 25 of 25 computed |

Of the 25 structs, 12 change size between a 64-bit and a 32-bit target,
and every one of those holds a pointer, a callback or a `size_t`. So a
binding may assert a **fixed** size for the 10 that hold none, and must
ask the interpreter for the rest — which is what a table of two
columns says and a single hard-coded number could not.

## 4. The scalars, and the two spellings Python needs

A `ctypes` layer needs a **type** for every scalar that crosses in a
struct or a signature, and a decoder needs a `struct`/`memoryview`
**format code** for every scalar a blob column holds. Neither may be
guessed: `c_long` is 8 bytes on Linux and 4 on Windows, which is exactly
the class of mistake a generated binding exists to make impossible.

| scalar | `ctypes` | format | at the boundary | in a column |
|---|---|---|---|---|
| `u8` | `c_uint8` | `B` | 61 | 23 |
| `u16` | `c_uint16` | `H` | 14 | 14 |
| `u32` | `c_uint32` | `I` | 44 | 8 |
| `u64` | `c_uint64` | `Q` | 1 | 0 |
| `i8` | `c_int8` | `b` | 0 | 0 |
| `i16` | `c_int16` | `h` | 0 | 0 |
| `i32` | `c_int32` | `i` | 11 | 3 |
| `i64` | `c_int64` | `q` | 4 | 0 |
| `f32` | `c_float` | `f` | 0 | 0 |
| `f64` | `c_double` | `d` | 42 | 34 |
| `usize` | `c_size_t` | `n` | 12 | 0 |
| `isize` | `c_ssize_t` | `N` | 0 | 0 |
| `bool` | `c_bool` | `?` | 0 | 0 |

| proposed rule | verdict | measured |
|---|---|---|
| every scalar has a fixed-width `ctypes` type and a format code | **holds** | 0 of 13 disagree |
| every scalar the boundary uses is one of the thirteen | **holds** | 8 of 13 appear |
| every blob column's scalar has a format code | **holds** | 0 of 82 disagree |

## 5. What a binding can say about a value

ADR-0023 puts the units, ranges, examples and enum links on the `api:`
line of the Rust field, so that one sentence written once reaches every
binding's documentation and every binding's type. What follows is how
much of that there is to reach for: 160 of 160 visible struct fields
carry a doc comment.

| `api:` tag | fields |
|---|---|
| `bitset` | 1 |
| `brand` | 4 |
| `enum` | 21 |
| `example` | 70 |
| `flag` | 14 |
| `len` | 13 |
| `nullable` | 11 |
| `present_if` | 1 |
| `range` | 21 |
| `unit` | 37 |

| proposed rule | verdict | measured |
|---|---|---|
| every visible field carries a doc comment | **holds** | 0 of 160 disagree |
| every floating-point field carries a unit | **holds** | 0 of 160 disagree |

Every number that crosses the boundary says what it is measured in, so
no binding has to document one as a bare `float`.

