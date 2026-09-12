# The Rust consumer surface

Status: `designed`, written 2026-09-12 from the falsification pass in
[`rust-consumer-surface-measured.md`](rust-consumer-surface-measured.md).
Settles ADR-0030 §9, which left Rust's own consumer surface to this page.
Derives from ADR-0002 (an agnostic port), ADR-0023 (type safety in every
binding), ADR-0029 (an ephemeris is plugged in) and ADR-0030 (the
consumption surface). `02-architecture/07-binding-architecture.md` gives
each binding its row.

## 1. Purpose and scope

Node, Dart and Python read `sdk.<area>.<operation>` off a context. Rust —
the language the whole SDK is written in — has no such thing: a consumer
picks among twenty-two crates and assembles what the other three are
handed. This page decides what Rust gets instead.

In scope: whether Rust has a context at all, what it is made of, how the
areas are expressed, how an ephemeris is chosen, what the façade does
**not** carry, and the gates that hold it to the other three. Out of
scope: the crates' own public APIs, which stay what they are — this is a
surface over them, not a rewrite of them.

## 2. What the measurement decided, and what it left

The pass composed three readings — the boundary's description, what every
function of `crates/ffi` names and calls, and which crates depend on
which — and closed the calls over. Three results carry this design:

1. **An area's operations come from one SDK crate 1 time in 8.** `chart`
   needs six crates, `almanac` seven, the root's `positions` four. The
   areas are not a renaming of the crates, so a façade is not a
   re-export list.
2. **Two of the nine crates a context needs are held by `crates/ffi` and
   by nothing else** — `teistro-intl` and `teistro-ephemeris-builtin`.
   The composition that makes a context exists in Rust *nowhere but the
   C boundary*.
3. **Eight of the forty-six entry points reach no SDK crate at all.**
   They are the C caller's memory and its handshake: `ts_string_free`,
   `ts_blob_free`, `ts_context_free`, `ts_abi_version`, `ts_build_info`
   and their like. A Rust façade is **smaller** than the boundary, not
   larger.

What the measurement left, and what this page answers from the other
bindings and from the ADRs rather than from a count: the *shape* — a
context object, a builder, or free functions over a settings value.

## 3. Rust composes the crates; it does not call its own C ABI

**Decided: the façade depends on the crates, not on `teistro-ffi`.**

The alternative is real and worth naming, because it is the cheap one: a
Rust façade could wrap `teistro-ffi` the way the other three bindings do,
and inherit the composition for free. It is refused on the measurement's
third result. Eight of the forty-six entry points are marshalling with no
computation in them, and the blob is the rest of it: a Rust consumer
would write a request struct so the boundary could decode it, and decode
a result blob so they could read numbers the crates had already returned
as `JulianDay` and `Longitude`. Rust would be the only binding whose
implementation language *is* its consumption language and which still
crossed a C ABI to reach itself.

So the façade calls the crates. That leaves the composition written
twice, which this design does not accept either:

**Decided: the composition moves into the façade, and `teistro-ffi`
depends on it.** The boundary keeps what only it needs — the handles, the
`struct_size` handshake, the blob writer, the panic guard, the
`last_error` slot — and calls the façade for the work. That is what the
measurement's second result asks for: the composition gets one home, and
the home is the one both a Rust consumer and the C boundary can reach.

It is also **checkable**, which is why it is written this way round. The
measured page's property *no crate a context needs is brought in by the
boundary alone* is falsified today, 2 of 9. When the façade holds the
composition, it holds those two crates, and the property **holds**. The
pass that measured the gap becomes the gate on the result — the project's
own rule that a pass whose subject you are changing turns over when the
change lands.

## 4. A context, and areas as borrowed views

**Decided: one `Context`, and an area is a borrowing view of it.**

```rust
let sdk = teistro::Context::builder()
    .profile("nepali-default")
    .locale("ne-Deva-NP")
    .ephemeris([Ephemeris::Builtin])
    .build()?;

let bs = sdk.calendar().convert(&date, Calendar::BikramSambat)?;
let noon = sdk.time().resolve(&at, &zone)?;
let sky = sdk.positions(&[2_451_545.0], &[Body::Sun], &frame)?;
```

Why a view rather than inherent methods: the same reason ADR-0030 gave
for the other three. `calendar().convert` and `time().convert` are two
operations with one good name each; flat, one of them is `convert_time`,
and the measurement behind ADR-0030 found six members already spelling
their own area inside their own name. Rust has no reason to be the
binding that keeps them.

Why borrowed rather than owned: an area is a *value* in every other
binding — a frozen instance, a `late final` field, a `cached_property` —
and the Rust equivalent of a value you can hold and pass on is
`Calendar<'a>(&'a Context)`. It allocates nothing, it cannot outlive its
context, and `sdk.calendar()` costs a pointer copy. A consumer who wants
to keep one writes `let cal = sdk.calendar();` exactly as a Node consumer
writes `const cal = ctx.calendar`.

Why a builder rather than an options struct: ADR-0023 asks for
constructors that cannot be half-built, and the "no dead ends" brief asks
that every choice be a knob. A builder with typed setters gives both, and
gives the one thing an options struct cannot — a `build()` that returns
the settings it resolved, so *what was applied is reported*.

**The root keeps what the other three keep**: `positions`, `profile`,
`settings`, `settings_hash`. `positions` is a root operation because an
operation whose name is its own area's name is one — the rule
`surface-areas.md` states and the reason nobody writes
`sdk.positions.positions`.

**No `dispose`.** `Drop` is the whole of it, which is the first place the
Rust surface is smaller than the C one rather than the same size.

## 5. The ephemeris is linked, not loaded

ADR-0029 gives a Rust consumer the adapters as `rlib`s and says they
link directly rather than paying for a loader. So the chain's entries in
Rust are values, not paths:

```rust
.ephemeris([Ephemeris::Provider(Box::new(Teimeris::open(&dir)?)), Ephemeris::Builtin])
```

`Ephemeris::Builtin`, `Ephemeris::Test` and `Ephemeris::Provider(_)` are
the three kinds, and an ordered chain is tried in order and **never
silent** — the same rule the other three keep, and the same refusal
naming every entry that failed. A chain of one is not a chain, so its
refusal keeps its own status, field and hint.

`ts_provider_load` stays a boundary concern: loading a shared library is
what a consumer who is *not* in Rust needs, and a Rust consumer who wants
it can still call it.

## 6. What the façade does not carry

From the measurement's third result, and it is a list rather than a
principle:

- **Freeing.** `ts_string_free`, `ts_blob_free`, `ts_context_free`,
  `ts_provider_free` — `Drop`.
- **The handshake.** `ts_abi_version`, `ts_sdk_version`,
  `ts_catalogue_version`, `ts_build_info` — a Rust consumer's Cargo
  resolved the versions, and `env!("CARGO_PKG_VERSION")` is the rest.
- **The blob.** Every binding decodes one; the crates return their own
  types, so there is nothing to decode.
- **`ts_status_message` and the `last_error` slot.** A refusal is a
  `Result<_, teistro_core::Error>` and carries its own field and hint.

## 7. What the gates gain

- **A fourth parity runner.** The three runners print one line per
  operation keyed by a canonical `surface.<area>.<operation>` path, and
  `check-parity` compares every report against the first. A Rust runner
  joins them, and `check-areas`'s property *every operation the layer
  declares is listed by every parity runner* extends to it — so the
  façade cannot grow an operation the other three lack, or lack one they
  have, without a gate saying so.
- **The measured page turns over**, as §3 says: its second property
  flips from falsified to holding, and that is the acceptance test for
  the refactor rather than a note in a commit message.
- **`check-rust-surface` keeps measuring after the change**, which is the
  point of gating it: the areas' crate counts are what they are, and a
  future operation that pulls a tenth crate into a context shows up as a
  changed page.

## 8. Order of work

1. **The façade crate, beside the boundary.** `Context`, the builder, the
   eight area views, the ephemeris chain — composing the crates, with
   `teistro-ffi` untouched. Composition written twice, briefly and
   knowingly.
2. **The parity runner**, which is what proves step 1 equals the other
   three rather than merely compiling. Red until it does.
3. **Invert the dependency.** `teistro-ffi` calls the façade and keeps
   only its marshalling; the measured page's second property flips. This
   is the step that pays the duplication back, and it is third because
   the two steps before it are what make it safe.
4. **Examples**, the same eight scenarios the other three bindings run,
   held by a gate the same way.
5. **The site**, whose surface page gains a Rust column, and whose
   install page gains a Rust section — after the crate exists, not
   before. The install page already carries a lesson about documenting
   what nobody has run.

## 9. What this design does not settle

- **The crate's name, and whether it is held.** `teistro` is the obvious
  one and every other crate in this workspace holds its name on crates.io
  as a `0.0.0` placeholder. Whether that one is held is a fact this page
  does not have and a decision that is the maintainer's.
- **How much of the façade is re-export.** The measurement counts crates,
  not public items: an area's crate may already expose exactly the
  operation or only the pieces it is built from. Step 1 answers it by
  construction, and if the answer turns out to be "mostly re-export" for
  an area, that area's view is thin and should stay thin.
- **Whether `serial`'s chart document belongs here.** It is a crate a
  context does not need, so the measurement says nothing about it.
- **`no_std`.** Nothing in this design needs an allocator that the crates
  do not already need, and nothing in it has been measured without one.
