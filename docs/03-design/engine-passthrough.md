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

Five of them, and the rule was read off the first before the others
existed:

| what | role or field | who supplies it |
|---|---|---|
| the context | `handle` | the adapter's own, held for the call |
| a buffer's capacity | `string_cap` | the fill protocol, which allocated it |
| a struct's extent | `out_struct_size`, and the `struct_size` **field** | `size_of` of the struct the arm declared |
| an array's length | `array_len` | the length of the array the caller passed |
| an output's capacity and its count | `array_cap`, and the `scalar_out` an extent names | the room the arm made, and the length of the array the answer holds |

One of these changes sides with the output beside it, which is why the
rule is per function and not per role: the capacity of a **search** is
not bookkeeping. "The next `out_capacity` eclipses" is the caller's
question, so for an output whose extent is `asked` the capacity is an
argument (§4).

The struct's extent is the strongest case of them. Teimeris's public structs each carry a `struct_size` first
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

## 4. How an array crosses

> **An array crosses as a JSON array of whatever its element crosses
> as, and an output array is sized by what the engine says its length
> is — never by a capacity the caller passes for an answer whose length
> is already decided.**

The element rules are the ones already written: a number narrows and is
refused rather than truncated, a struct is an object with every field
required. A bad element is refused by its index and its path —
`dts[1].month is required` — through the same helpers the scalar roles
use, because the path is what makes a thousand-element batch debuggable.

### What the engine did not say, and now does

An output array is the one shape the engine's description could not
size. `double *out, size_t out_capacity` is one C declaration for four
contracts, and the extractor recorded none of them — so the engine's own
Node generator makes the caller pass `capacity` and hands back that many
elements whether or not the engine wrote them. That is a dead end twice
over: the caller has to know a length the engine already knows, and an
over-large guess returns zeroed elements that look like answers.

It is fixed where it belongs, in the engine (`df3945e`,
`05-testing/02-engine-findings.md` D2): every one of the forty output
arrays now carries an `extent`, listed in `tools/idl/extract.py` rather
than inferred, because "one length in, so the output is that long"
holds for every `_many` function and is wrong for
`tm_houses_calc_many`. The extractor refuses an output with no entry and
an entry with no output.

| extent | of the forty | the room | the answer |
|---|---:|---|---|
| `length` | 16 | the named input's length | all of it |
| `product` | 2 | the named inputs' lengths, multiplied in layout order | all of it |
| `asked` | 11 | the caller's `out_capacity`, an argument | cut to the count the engine gives |
| `total` | 4 | 64, then exactly what the engine reported | all of it |
| `unstated` | 7 | — | — |

A `length` or `product` that names a struct input's field
(`req.day_count`) is sized by nothing this marshaller reads, and is
queued with the `unstated` ones; the measured page lists all nine with
the engine's own reason.

### Three rules the room follows

1. **Every element of a struct output is `default`**, not zeroed. The
   engine reads each element's `struct_size` to learn the stride it
   writes at (`core/src/tm_out.h`, "every output struct must arrive with
   `struct_size` set"), and `default` is the binding's way of setting it.
2. **The room is bounded at 256 MiB, and the allocation is fallible.** An
   `asked` capacity comes from the caller, and a caller who asks for ten
   billion eclipses is told so by name rather than having the process
   aborted. A product that overflows `usize` is refused the same way.
3. **`total` asks at most twice**, the fill protocol's shape for an
   array: into room for 64, and only if the engine reported more, again
   into exactly that. A second report larger than the first is refused
   as an answer that changed underneath the call, never truncated.

The count an output is cut to is not reported beside it: it is the
array's length. A count that is not an output's — `tm_scan_grid`'s
`out_samples` — is still an answer.

### What it does not do

A batch whose engine status is not `OK` is refused whole, as a scalar
call is. The engine's batches keep computing after an element fails and
record each element's own status, so this discards the elements that
succeeded; answering them with the call's status beside them is the
right shape, and it is not built because a status that means "the
capacity was too small" arrives the same way and fills nothing. Telling
the two apart needs the engine to say which statuses are per element,
which it does not yet (§7).

## 5. What it costs a consumer

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

A list is `readonly T[]` in TypeScript, `List<T>` in Dart, and in
Python `list[T]` coming back and any `Sequence[T]` going in — because
Python's `list` is invariant, and `list[int]` would be refused where
`list[float]` is taken.

Only the structs a callable function reaches get a type: a generator
that emitted all fifty-seven would put names in four languages for
structs no method mentions.

## 6. What each step releases

Measured, not estimated — each figure is the measured page's, computed
from the same reading that writes the code, and each later step's is
exact because the queue groups a function by its hardest blocker.

| step | callable | what it adds |
|---|---:|---|
| before | 62 | |
| plain structs | 95 | §3 |
| **arrays** | **112** | §4 |
| a struct carrying a string | 118 | a `CString` that outlives the call |
| a struct pointing at another | 135 | a nullable nested object |
| an output sized by another call, and a parallel output | 139 | `tm_house_cusp_count()`, `req.day_count`; an optional twin |

The last ten are genuinely different: three carry opaque bytes, four a
function-pointer vtable or a `char**`, and three return a pointer into
the engine's own memory whose lifetime the JSON boundary has no way to
state.

The plan this table replaced said an array of numbers would reach 99 and
an array of structs 114, as two steps. They are one step, because the
shape is the array and not its element, and it reached 112: the two
house functions whose cusps are sized by the house system were counted
as arrays and are a different job, which reading their extents showed.

This replaces the sentence the measured page used to carry — *81 of the
87 are behind structs* — which was true and was not actionable. Eighty-one
was one row because the classifier knew a struct only by the word
`struct`. Reading the field lists splits it into four shapes with four
different jobs behind them, and the largest of the four was also the
easiest.

## 7. How it is held

- `check-engine` regenerates the page, the dispatch and the four façades
  and fails on any byte of difference.
- The adapter's own `tests/passthrough.rs`, against the real engine: a
  struct crosses both ways and its extent in neither, a field is refused
  by its whole path, a struct left out is null and the engine's refusal
  is the engine's, an extent a caller names is not read, and a nested
  struct comes back nested; an array answers one value per input and the
  same values its scalar twin does, an array of structs round-trips and
  a bad element is refused by index, a grid is bodies-by-epochs long and
  body-major, an output the engine counts is gathered, and a search
  answers exactly as many as asked.
- The helpers' unit tests, for what the real engine never reaches: a
  gather that must ask twice and one whose answer grows, room past the
  bound, and a product that overflows.
- The façades' consumer files (`typecheck/consumer.ts`,
  `typecheck/consumer.py`, `example/consumer.dart`) call a struct both
  ways and leave an optional one out, under `tsc`, `mypy --strict` and
  `dart analyze`.
- Two assertions in the generator, where an assumption would otherwise
  be believed: nesting follows declaration order, and no Python record
  shares a struct's name.

## 8. Open questions

- **A batch's per-element statuses** (§4). Answering the elements that
  succeeded beside the call's status needs the engine to say which of its
  statuses are per element and which mean nothing was filled.
- **A struct pointing at another** (§6) has to say what a `null` *field*
  means, where rule 5 settled a null *argument*.

ADR-0030's rule that what proves universal is promoted into the port
applies to all of it.
