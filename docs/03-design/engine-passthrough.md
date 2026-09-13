# The engine passthrough: how a shape crosses

Status: `designed`, written 2026-09-13 from the measurement in
[`engine-passthrough-measured.md`](engine-passthrough-measured.md).
Derives from [`ephemeris-port-and-adapters.md`](ephemeris-port-and-adapters.md)
(the port, and `native_manifest`/`native_call`) and ADR-0030 (the
namespace and where the typed façade lives). Built by
`cargo xtask engine`, gated by `check-engine`.

The measured page says **what** is callable and what is not. This page
says **how** a shape crosses, which is the thing the measured page
cannot say about itself: it is generated, and a generated page is a
count, not a decision.

## 1. Purpose and scope

The engine describes 161 functions. The adapter marshals a call from a
JSON object to C and back. Every parameter role the engine's own
description uses is either a shape the marshaller carries, a shape it
has not been taught, or bookkeeping it does itself.

This page is the standing account of the third of those, because it is
the one nothing else records. A role that is **bookkeeping** never
appears in the manifest, never appears in a façade's signature, and is
computed by the generated arm — and every time the generator has learned
a shape, the interesting decision has been which part of it the consumer
must not be allowed to supply.

It is not: what is callable (measured), nor whether a call answers
correctly (the adapter's own tests against the engine).

## 2. The bookkeeping rule

> **A value whose wrong answer is a memory-safety bug, and whose right
> answer the marshaller already knows, is bookkeeping and never crosses.**

Three of them, and the rule was read off the first before the other two
existed:

| what | role or field | who supplies it |
|---|---|---|
| the context | `handle` | the adapter's own, held for the call |
| a buffer's capacity | `string_cap` | the fill protocol, which allocated it |
| a struct's extent | `out_struct_size`, and the `struct_size` **field** | `size_of` of the struct the arm declared |

The third is the one this tranche adds, and it is the strongest case of
the three. Teimeris's public structs each carry a `struct_size` first
field so the library can tell which version of the struct it was handed;
the engine **reads past that field only as far as it says**. A consumer
who could set it could tell the engine a `tm_datetime` is larger than
the one the arm put on the stack, and the engine would read memory that
is not the struct. There is also nothing to gain: the only correct value
is the size of the struct the generated arm declared, and the arm knows
it.

So `struct_size` is filled by `sys::T::default()` — which exists for
every struct in the binding and does exactly this — and is stripped from
both directions: not read from the object a caller passes, not written
into the object the caller gets back.

## 3. How a plain struct crosses

> **A struct crosses as a JSON object keyed by the engine's own field
> names, one level per level of nesting.**

`sdk.engine.call('tm_local_to_utc', { local: { year: 2026, month: 9,
day: 13, hour: 6, minute: 30, second: 0 }, utc_offset_hours: 5.75, cal:
1 })` answers `{ out_utc: { year: …, month: …, … } }`.

Five rules settle the rest:

1. **The key is the parameter's name**, as every other role's is. A
   `struct_in` reads its object out of the arguments under its own name;
   a `struct_out` writes its object into the answer under its own name.
   Nothing about a struct makes it the *subject* of the call, and a
   marshaller that hoisted a lone struct's fields to the top level would
   have to un-hoist them the moment a function took two.
2. **Every field is required on the way in.** A C struct has no absent
   field — an omitted one is zero, which for a `tm_datetime` is the year
   0 and for an observer is the Atlantic. The alternative is a default
   the engine did not choose; refusing by name is the error the rest of
   the passthrough already gives.
3. **A field crosses by the same rules as a parameter of that type.** A
   number narrows and is refused rather than truncated; an enum is its
   integer; a nested struct is a nested object. This is not a second
   vocabulary: `narrow`, `number` and `Vocabulary::is_float` are the same
   ones the scalar roles use.
4. **`struct_size` is not a field** (§2).
5. **A struct argument may be left out, or passed as `null`, and crosses
   as a null pointer.** The engine's extractor marks every single struct
   input optional, because whether null is allowed lives in the header's
   prose and not in the type — an observer is optional unless the flags
   ask for a topocentric answer, and a datetime never is. The engine
   refuses a null it does not allow with its own `TM_ERR_INVALID_ARG`,
   so the marshaller does not second-guess it: requiring an observer on
   every call that merely *accepts* one would be a dead end, and a
   zeroed observer passed instead of null would be an answer at 0° N
   0° E that nobody asked for. A value that is present and not an object
   is still refused by name: a number where an observer belongs is a
   mistake, not an absence. The manifest says `"optional": true` on
   such a parameter, and every façade makes it omittable.

A struct is **plain** when every field is a number, an enum, an alias
over either, or a nested plain struct. Forty of the engine's fifty-seven
are, which is the tranche; the rest carry a string, a pointer to another
struct, or a function pointer, and each is its own later shape.

### Order of declaration

A nested struct's marshalling is emitted as a function per struct, so a
struct must be declared before the struct that holds it. The engine's
description is in header order and therefore already is — but the
generator **asserts** it rather than believing it, which is the same
habit that caught the drishti section's shared count. One struct in the
tranche nests at all (`tm_nodes_apsides`, four `tm_position`s), so the
assertion is cheap and the day it fires it will be the only warning.

## 4. What it costs a consumer

The typed façade grows a named type per struct in each target, named for
the struct — `tm_datetime` is `TmDatetime` everywhere:

| target | the type | its fields | the conversion |
|---|---|---|---|
| TypeScript | an `interface`, re-exported from the package's index | camelCase | a generated reader and writer per struct in `engine.js` |
| Dart | a `final class` with `fromJson`/`toJson` | camelCase | the class's own |
| Python | a `TypedDict` | the engine's own names | none — a dict is what crosses |

Node and Dart convert **per struct, generated**, rather than through a
generic key-mapper. A generic one would camel-case whatever it was
handed, so a key the engine does not declare would cross silently and the
field the caller meant would never arrive. The generated ones read
exactly the declared fields, and a missing one reaches the dispatch as
missing and is refused by its whole path — `local.month`, not `month`,
because the key a field is looked up under and the name a refusal must
print stop being the same string the moment structs nest.

Twenty-three of the forty plain structs are reachable from a callable
function, and only those get a type: a generator that emitted all
fifty-seven would put names in four languages for structs no method
mentions.

## 5. What this tranche releases, and what it does not

Measured, not estimated — the figures are on the measured page and are
computed from the same reading that writes the code.

| step | callable | what it adds |
|---|---:|---|
| before | 62 | |
| **plain structs** | **95** | this page |
| an array of numbers | 99 | the fill protocol with a width |
| an array of structs | 114 | the two above, together |
| a struct carrying a string | 120 | a `CString` that outlives the call |
| a struct pointing at another | 139 | a nullable nested object |

The last ten are genuinely different: two carry opaque bytes, five carry
a function-pointer vtable or a `char**`, and three return a pointer into
the engine's own memory whose lifetime the JSON boundary has no way to
state.

This replaces the sentence the measured page used to carry — *81 of the
87 are behind structs* — which was true and was not actionable. Eighty-one
was one row because the classifier knew a struct only by the word
`struct`. Reading the field lists splits it into four shapes with four
different jobs behind them, and the largest of the four was also the
easiest.

## 6. How it is held

- `check-engine` regenerates the page, the dispatch and the four façades
  and fails on any byte of difference.
- The adapter's own `tests/passthrough.rs`, against the real engine: a
  struct crosses both ways and its extent in neither, a field is refused
  by its whole path, a struct left out is null and the engine's refusal
  is the engine's, an extent a caller names is not read, and a nested
  struct comes back nested.
- The façades' consumer files (`typecheck/consumer.ts`,
  `typecheck/consumer.py`, `example/consumer.dart`) call a struct both
  ways and leave an optional one out, under `tsc`, `mypy --strict` and
  `dart analyze`.
- Two assertions in the generator, where an assumption would otherwise
  be believed: nesting follows declaration order, and no Python record
  shares a struct's name.

## 7. Open questions

None new. The step after an array of numbers is an array of structs,
which is both learned shapes together; the first step that needs a new
decision is a struct pointing at another (§5), which has to say what a
`null` *field* means where rule 5 settled a null *argument*. ADR-0030's
rule that what proves universal is promoted into the port applies to all
of it.
