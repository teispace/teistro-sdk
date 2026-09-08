# The Python binding

Status: `designed`, written 2026-09-08 from the falsification pass in
[`binding-surface-measured.md`](binding-surface-measured.md). Derives
from [`ffi-abi-and-api-description.md`](ffi-abi-and-api-description.md)
(the ABI, the description and the result blob), ADR-0004 (one
description, generated bindings), ADR-0007 (a designed C ABI and
generators of our own), ADR-0014 (everything we author is Rust, except a
binding's own layer) and ADR-0023 (type safety in every binding).
`02-architecture/07-binding-architecture.md` gives the binding its row.
Built as `bindings/python`.

## 1. Purpose and scope

The third binding, and the first that is neither compiled nor bundled.
Node loads a napi addon built from Rust; Dart loads the shared library
through `dart:ffi`. Python loads the **same shared library** through
`ctypes`, which is in the standard library, so the package has no
runtime dependency, needs no compiler on the consumer's machine and
works on every CPython the SDK supports without a build per version.

Everything mechanical is rendered from `idl/api.json` by
`cargo xtask gen ffi` and held equal to the boundary crates by
`cargo xtask check-ffi`, exactly as the other two are. What is written
by hand is the thin ergonomic layer, the port adapter, the installer and
the tests, which ADR-0014 allows a binding to have in its own language.

It is not: a second computation of anything (every answer comes from the
library), a `numpy` dependency (a decoded column is a `memoryview`, which
`numpy.asarray` wraps without copying for a caller who wants that), or a
second description of the API (ADR-0004 rejected PyO3 for exactly that
reason).

## 2. Why `ctypes`, and not the alternatives

| option | why not |
|---|---|
| PyO3 | a second description of one API, rejected in ADR-0004; and a compiled extension per CPython version and platform |
| `cffi` | a runtime or build-time dependency, and an API-mode build needs a compiler; ABI mode is `ctypes` with a package in front |
| a hand-written extension | the same as PyO3 with more of it by hand |
| **`ctypes`** | in the standard library; loads the artefact the release already builds; the generated layer is data, not code to compile |

The cost of `ctypes` is that it has **no compiler to check the layout
with**. The C header asserts all 25 struct sizes at compile time
(`_Static_assert`), so a C consumer that disagrees does not build; a
`ctypes` declaration is trusted. §5 is how that is paid for.

## 3. What is generated and what is written by hand

| file | what it is | written by |
|---|---|---|
| `teistro/catalogue.py` | every enum as an `IntEnum` carrying the id the C boundary uses and the key every pack and fixture spells; a catalogue kind gains `UNKNOWN` and a `_missing_` hook, so a member from a newer library is a value and not an exception | the generator |
| `teistro/_ffi.py` | the `ctypes` declarations that match the C header name for name, the branded scalars, a frozen dataclass per boundary struct with its marshalling, the exception, and the context handle with its finaliser | the generator |
| `teistro/_blob.py` | one decoder per result blob, reading the `TSRB` layout into `memoryview` columns over the blob's own bytes | the generator |
| `teistro/messages.py` | the typed accessors: every message of the SDK's locale as a method of its parameters, every catalogued entity as its forms (`cargo xtask gen intl`) | the generator |
| `teistro/__init__.py` | the layer a consumer uses: finding the shared library and checking its build, the defaults, JSON both ways, and the conveniences a generator cannot know are wanted | by hand |
| `teistro/_host.py` | the port adapter: an ephemeris written in Python bound into the vtable through `ctypes.CFUNCTYPE` trampolines | by hand |
| `teistro/_install.py`, the `teistro-install` command | the installer: where a prebuilt library comes from, the digest it must have, and where it is written | by hand |
| `teistro/_prebuilt.py` | the release the installer fetches from and the digest of each platform's library; empty in a checkout, written when a release is staged | the release |
| `tests/` | the surface end to end, the decoders against blobs the library produced, and the sizes against the library that was built | by hand |
| `example/teistro_example.py` | the code the README shows, run by the gate so the two cannot drift | by hand |
| `parity.py` | this binding's half of the parity report, which `cargo xtask check-parity` compares with the other two | by hand |
| `typecheck/wrong.py` | the usages that must not type-check, each with the error it must raise | by hand |

The generator lays the generated files out, so `check-ffi` can
regenerate them on a machine with no Python and still compare them byte
for byte.

## 4. The names, and the one rule the pass made necessary

The pass counted every identifier each binding must spell. Python is
caught in a **different place** from Dart, and that is the whole reason
the measurement was worth taking:

- Dart spells an enum member as a camel-case identifier, so
  `ChartKind::Return` becomes `return`, which is reserved; the Dart
  emitter renames it `returnValue`.
- Python spells an enum member as its **catalogue key, upper-cased**.
  Every Python keyword is lower-case, so no member of any of the 79
  enums can collide, and the rule never fires there.
- Python is caught instead on `from`, which is a hard keyword there and
  a contextual word in Dart and TypeScript. It appears as a **struct
  field** (`ts_time_conversion.from`) and as a **parameter** of two
  entry points.

So the rule is PEP 8's: a name that is a Python keyword gets a trailing
underscore, and `from` becomes `from_`. The word lists of all three
targets live in one module, `teistro_idl::emit::reserved`, because a
list held privately inside one emitter cannot be measured, and the whole
point of the pass is that each emitter's rule is counted rather than
assumed.

An enum member is also checked against the three names `enum.Enum`
keeps for itself (`name`, `value`, `mro`); none of the 919 members
takes one.

## 5. The layout, and how a binding with no compiler is held to it

`ctypes` computes a struct's layout from the fields it is given, by the
platform's own C rules. That is the same rule the compiler used, so the
two agree — but nothing *checks* that they agree, and a wrong layout is
not an error, it is a wrong number.

The pass measured every struct's size on both targets:

- 10 of the 25 hold no pointer, no callback and no `size_t`, and are
  therefore the **same size everywhere**;
- 12 change size between a 64-bit and a 32-bit target, and every one of
  them holds one of those three;
- every layout is computable from the description alone.

So the generated module carries a table of the sizes for the target it
was generated for, and a generated test asserts `ctypes.sizeof` against
it — on the machine the library was actually built on, which is the only
place the question can be settled. A binding that shipped one hard-coded
number would be right on one target and silently wrong on the other; a
table with a column per target and an assertion at test time is right on
both and says so.

The `struct_size` handshake is the second half: 17 of the 25 structs
carry one, and the generated marshalling fills it from
`ctypes.sizeof(...)` rather than from a literal, so a struct that gains
a field keeps working against an older library.

## 6. The types

**Scalars.** Each of the 13 has one `ctypes` spelling and one `struct`
format code, both fixed-width (`c_uint32`, not `c_uint`), because
`c_long` is 8 bytes on Linux and 4 on Windows and that is the class of
mistake a generated binding exists to make impossible. Eight of the
thirteen appear at the boundary; the other five are declared anyway, so
that a field added later needs no change here.

**Brands.** A quantity that would otherwise be swappable is a `float`
subclass with a validating constructor:

```python
class Latitude(float):
    """Degrees, north positive. Unit: deg. Range: [-90,90]."""
    def __new__(cls, value: float) -> "Latitude": ...
```

This is a **deliberate difference** from
`02-architecture/07-binding-architecture.md`, which says `NewType` in a
`.pyi` stub. A `float` subclass gives a type checker everything `NewType`
gives it — `Observer(latitude_deg=Longitude(85.3))` and
`Observer(latitude_deg=27.7)` are both errors — and adds the range check
that `NewType` cannot have, in one name rather than a type and a factory
beside it. At run time it *is* a float, so `ctypes` takes it with no
conversion.

For the same reason there is **no `.pyi`**: the generated module carries
inline annotations and the package ships `py.typed`. One file cannot
drift from itself, and a stub beside a generated module is a second copy
of the same facts.

**Enums.** `IntEnum`, so a member is accepted wherever the boundary
wants an id, carrying `id` and `key` as the other bindings do. A
catalogue kind gains `UNKNOWN = -1` and a `_missing_` hook returning it,
so a value from a newer library is a member rather than a `ValueError`;
a closed enum has no such hook, and an id outside it raises, because a
value outside a closed set is a fault and not a state.

Every member also carries `__bool__` returning `True`, which is not
decoration. `IntEnum` inherits `int.__bool__`, so the member with id
**zero** is falsy — and the member with id zero is `Status.OK`,
`Graha.SUN`, `Era.VIKRAMA` and the first member of every one of the 79
enums. `if graha:` has to mean "there is a graha", and `era and era.key`
has to reach the key. This binding is the only one where the question
arises, because it is the only one whose members are numbers.

**Structs.** Two classes per boundary struct: a private
`ctypes.Structure` with the C layout, and a public frozen dataclass with
the fields a binding shows — the `struct_size` handshake, the reserved
padding, an array's count and a presence flag are all absorbed, exactly
as they are in Dart and TypeScript, because the roles they are absorbed
by are read from one place (`teistro_idl::rules`).

## 7. The result blob

`ts_positions` and `ts_intl_render` answer with a `TSRB` blob the library
allocated. The binding copies it once with `ctypes.string_at`, frees the
library's copy immediately, and decodes the copy: a column becomes a
read-only `memoryview` cast to its scalar's format code, which is a view
over those bytes and not a second copy. `numpy.asarray` wraps such a view
without copying, so numpy interop costs nothing and is not a dependency.

One copy rather than none is the deliberate trade. Dart keeps the
library's memory alive under a `NativeFinalizer`; in Python the same
thing would mean a `ctypes` buffer whose lifetime is tied to a
`weakref.finalize`, and a decoded column that outlived it would read
freed memory. A blob is a few kilobytes; the copy is not measurable
beside the computation that produced it, and it makes a decoded result
safe to keep.

The decoder reads the layout version first and refuses another with a
typed error, finds a section by id so a section appended by a newer
library is skipped, and checks every offset against the blob's length
before it reads.

## 8. Handles, memory and the GIL

A context is a handle with an explicit `close()`, a `weakref.finalize`
behind it, and `__enter__`/`__exit__`, so the idiomatic form is a `with`
block and the explicit form still exists (ADR-0007, finding 4: a result
that waits for the collector can exhaust memory).

`ctypes.CDLL` releases the GIL for the duration of every call and a
`CFUNCTYPE` callback re-acquires it, which is exactly what
`02-architecture/07-binding-architecture.md` asks for: the GIL is
released during native computation, and held while a Python provider
answers. Nothing in this binding has to arrange it.

## 9. A provider written in Python

`EphemerisProvider` is an abstract base with the same shape the Dart
binding gives it: `name`, `bodies` and `positions` abstract, everything
else defaulted, and one call per whole grid rather than a loop.
`HostProvider` binds one into the vtable:

- one `CFUNCTYPE` trampoline per vtable slot, **kept alive on the
  object**, because a `ctypes` callback that is collected leaves the
  library holding a dangling function pointer;
- every trampoline wraps its body in `try`/`except` and returns the
  port's refusal code, keeping the exception for the layer above to
  re-raise. This is not optional politeness: a Python exception that
  escapes a `ctypes` callback prints a traceback and returns **zero**,
  which the port reads as success;
- the capabilities are described once, into memory the object owns, and
  freed with it.

The refusal code is checked against the catalogue's own value when the
provider binds, so the constant and the boundary cannot drift.

## 10. Errors

| when | what |
|---|---|
| the library answers a non-`OK` status | `TeistroError`, carrying the `Status` member, the message, and the field, hint and range the boundary's error struct gives |
| a brand is built outside its range | `ValueError` from the constructor, naming the field and the range |
| a blob of another layout version or schema | `TeistroError` with `Status.UNSUPPORTED` |
| the library is not the build this package was generated for | `TeistroError` with `Status.UNSUPPORTED`, saying which of the ABI, the version, the sanitizer or the optimisation is wrong |
| no library to open | `FileNotFoundError` naming every place that was looked in |
| a provider written in Python raises | the exception itself, re-raised on the caller's side of the boundary |

## 11. Loading and identity

`open()` looks at `$TEISTRO_LIBRARY`, then at the package's own `_lib/`
directory, then at what the installer wrote, then in the workspace's
`target/release` and `target/debug`, and finally asks the platform's
loader for the bare name. It reads `ts_build_info` and refuses a library
that is not the build these declarations were generated from — another
ABI, another SDK version, a sanitizer build however it was found, or an
unoptimised build it searched out rather than was given.

The `_lib/` directory is the one difference from the Dart search order,
and it is what makes a per-platform wheel a **packaging** change rather
than a code change later: a wheel that carries the library puts it
there, and the same loader finds it with nothing else altered.

## 12. Packaging

A pure `py3-none-any` wheel and an sdist, and a `teistro-install`
command that fetches the shared library for this machine from the
release the package's version was cut from, checks it against a digest
recorded when it was built, and writes it where the loader looks. This
is the Dart package's model, and for the same reason: a package that
carried a binary for every platform would make every consumer download
all of them.

`hashlib` is in the standard library, so unlike the Dart package this one
needs no digest of its own.

Per-platform wheels are the more idiomatic Python answer and are an open
question (§14), not a different design: they fill the same `_lib/`
directory the loader already searches.

## 13. Tests and the gate

`cargo xtask check-python` builds the library, writes the blob fixtures
and drives the whole binding, skipping with a note when no Python is on
the machine (ADR-0014: the fast check stays Rust-only):

1. `python3 -m unittest discover` over `tests/`, with the library and the
   fixtures named by environment;
2. the README's example, run so the two cannot drift;
3. `typecheck/wrong.py` under a type checker, when one is present: every
   line marked `# expect:` must be reported and no error may go
   unexpected, which is the Python half of Phase 1's "a swapped latitude
   and longitude does not compile";
4. the package type-checked in strict mode, when a checker is present.

The tests themselves cover: every entry point through the ergonomic
layer; the decoders against blobs the library really produced; every
struct's `ctypes.sizeof` against the generated table; the catalogue's
round trips; a provider written in Python answering a real grid, and one
that raises; and the loader's refusals.

`cargo xtask check-parity` gains a third report. The gate compared two
before; it now compares every binding present against the first, so that
a machine with only two toolchains still gates the pair it has, and the
three agree on all 103 values.

The two gates earned their place on the first run, and from opposite
directions. `convert_time` was written taking `TimeScale`, and the
boundary wants `Scale` — a different enum that knows UTC as well as the
two the port carries, and that **agrees with `TimeScale` on the ids they
share**. So the call compiled, ran, and converted from the wrong scale in
silence. The type checker called it an argument of the wrong type; the
parity gate called it three values the other two bindings disagreed with.
Either alone would have found it; that both did is the reason for having
both.

## 14. Open questions

- **Per-platform wheels.** They would make `pip install teistro` enough
  on its own. They need five wheels in the release matrix and a PyPI
  account, both of which belong to the release rather than to the
  binding, and they change nothing here but where `_lib/` is filled from.
- **A `numpy` extra.** A decoded column is already a buffer numpy wraps
  without copying. An optional `teistro[numpy]` that returned arrays
  directly would save the caller one call and add a dependency; worth
  measuring against a real workload before it is decided.
- **Async.** The core is synchronous and single-threaded per context.
  An `asyncio` wrapper that ran a context on a thread would be a
  convenience over `ContextPool`, which the Node binding has and this one
  does not yet.
