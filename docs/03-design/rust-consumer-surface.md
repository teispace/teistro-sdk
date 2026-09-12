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
   The composition that makes a context is written once, inside the crate
   whose whole purpose is the C ABI.

   Precisely, because the claim is load-bearing: a Rust consumer *can*
   reach that composition. `TsContext::build` is `pub`, and so are
   `settings`, `profile`, `provider` and `intl`. What they cannot reach
   is an **operation** — converting a date means `ts_calendar_convert`
   with three raw pointers. So Rust today has a context it can build and
   cannot use, which is a sharper statement of the gap than "cannot
   reach it" and points at the same façade.
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

And it is less invasive than it sounds, because `TsContext::build` is
already a `pub` function whose body *is* the composition: the inversion
moves that body into the façade and leaves `build` calling it. The
boundary's own tests keep asserting what they assert, which is the
property that makes step 3 safe.

### What the measurement could not see, and the source said

The locale bundles are compiled into `crates/ffi` by **its own build
script**, from `i18n/`, so that a consumer needs no files to render the
SDK's own messages (ADR-0010). A crate dependency reading cannot see
that, and neither can a reading of function bodies: `BUNDLES` is a
`pub(crate) static` written into `OUT_DIR`. It is recorded here because
the façade needs those bundles to have a locale at all, and because a
build script is a third place a composition can hide.

It does not want a crate of its own. A build script cannot use the crate
it builds, so `teistro-intl` cannot compile its own bundles; the façade
can, and `teistro-ffi` then reads them from the façade — which is the
same inversion §3 already decided, arriving a second time from a
different direction. Until then the façade carries its own copy of a
thirty-line script, duplicated as knowingly as the composition is.

`build_info` — the commit, the target, the profile — stays with the
boundary. It describes the C library, and nothing else has a reason to
want it.

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

**And a fourth kind, which the building found and this page had
missed.** In Node, Dart and Python an entry is a *description* — a name,
or a descriptor holding a path — so opening it can fail and a later entry
is the fallback. In Rust an entry as decided above is an already-built
`Box<dyn EphemerisProvider>`, which **cannot fail to open**: every chain
of them succeeds on its first entry, and the ordering is decoration.
Clippy said so before a test did, reporting that the function opening an
entry returned a `Result` that could not be an error.

So an entry a chain can fall back *from* is a **recipe**, not a
provider:

```rust
.ephemeris([
    Ephemeris::opening("teimeris", || teimeris(&dir)),
    Ephemeris::Builtin,
])
```

`Provider(p)` stays for the case a consumer already has one, and the two
are not a second spelling of one thing: they answer different questions —
*here is an ephemeris* and *here is how to get one, which may fail*.
That is the same shape as `rules::factories`, the rule the emitters
needed: one constructor, and more than one way in.

Note what this does **not** need: a `#[cfg(not(builtin-ephemeris))]` arm
that refuses at run time, as the C boundary has. The variant itself is
behind the feature, so a consumer without it cannot name `Builtin` — a
compile error rather than a refusal, which is what ADR-0023 asks for
wherever a binding can have it.

`ts_provider_load` stays a boundary concern: loading a shared library is
what a consumer who is *not* in Rust needs, and a Rust consumer who wants
it can still call it.

### Where the surface owns a type, and it is not an exception

The types an operation takes and answers with are the crates' own, and
this page says so — but reading the boundary for the next increment
found two that are **nobody's**, and they are nobody's for a reason.

`time.convert(jd, from, to)` is dynamic: a caller names the scales at run
time. The crates express a scale in the *type system* — `JulianDay<Ut1>`,
`<Tt>`, `<Utc>` — so there is no runtime value to re-export, and the C
boundary invented `TsScale` because C needs one. A Rust consumer
converting a scale chosen by a request parameter needs one too. Likewise
the record of *what was applied* — the ΔT, whether UTC was proleptic, the
DUT1 seconds — which `crates/ffi` declares as a private `Applied` and no
crate has.

**So the rule is: the surface owns a type exactly where an operation is
dynamic and the crates are static.** Two of them, both for `convert`,
and each should be the façade's public type that the boundary converts
*from* once the inversion lands — not a third copy. It is not an
exception to "the crates' own types"; it is what that rule means when the
type system is where a crate keeps the distinction.

### And where an area's member is a module, not a method

`intl` has seven operations in the other three bindings, and one of them
is not an operation: `messages` is the **typed accessor tree** — every
message of the SDK's locale as a callable of its own parameters, which
`cargo xtask gen intl` writes into each binding.

Rust already has it, and not as a method. `crates/intl/src/messages.rs`
is generated by `teistro-intl` itself — a struct per message
implementing `TypedMessage`, so `render_typed` takes
`messages::sdk::reason::GrahaInBhava { graha, bhava }` where Node writes
`ctx.intl.messages.sdk.reason.grahaInBhava({ … })`.

So the façade's `messages` is `pub use teistro_intl::messages`: a module
tree, because a module tree is what a namespace is in Rust. That is a
**shape** difference from the other three rather than a missing
operation, and it is the right one — the parity gate compares a
canonical path per operation, and `intl.messages` resolves to something
a consumer reaches in each language's own way. Worth writing down
because the next increment would otherwise have to ask.

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

- **A fourth parity runner**, and it cannot have the same key set as the
  other three. That is not a gap in it; it is §6 arriving in the gate.

  The three runners print ~674 keys, and among them are `abi`,
  `build-sdk`, `build-commit`, `build-target` and the result blobs'
  sections and hashes. A Rust consumer has **none of those**: Cargo
  resolved the versions, `Drop` freed the memory, and the crates handed
  back their own types instead of a blob. `check-parity` compares every
  report against the first *by key set*, so a Rust report would fail on
  absences that are the design working.

  So the gate holds the Rust report to **every key it prints**, and
  names the ones it does not.

  It began as a **count** rather than a declared list, and that was a
  deliberate weakening: at 549 absences a list would have had to name
  detail the runner simply did not print yet, a graha at a time and a
  muhurta at a time, so the number a reader sees on every run was the
  honest instrument — the `knob-has-a-reader` shape.

  **549 → 9, in five passes, and every one of the 540 keys added agreed
  with Node on its first run.** No pass needed a second attempt, which
  is the strongest thing to be said about the composition underneath
  them. What went in, in order: every graha's longitude, latitude,
  speed, retrograde flag, house and placement, and all twelve bhava
  madhyas and sandhis, of both charts; every tithi, nakshatra, yoga and
  karana span of three days with its member and both bounds; each day's
  lunar month, convention, ayana and disha shool, and the values a day
  may *not* have, `none` in all four rather than nought in one; then
  each day's periods **item by item** — the kaalas, the first
  choghadiya, the horas at both ends, the muhurtas and what the Moon did
  — because a count agreeing is not the same as the items agreeing, and
  two lists of three can hold different threes; then the chart the
  **topocentric** profile founds, all nine grahas of it, which is the
  completion's centre step running per body and per instant inside the
  library and four layers agreeing on its output; then the batch's own
  rows and both envelopes' canonical JSON, hashed; and last the
  operation inventory.

  **So the count became a list, which is the stronger gate this section
  was waiting for.** `RUST_ABSENCES` names all nine, and it is
  exhaustive in both directions: an absence not on it fails, and one on
  it that the runner has started printing fails too, so a declared
  absence that stops being deliberate is a failed gate rather than a
  number nobody watched. Both branches proved red.

  The nine are of three kinds. `abi` and the six `build-*` keys are the
  **boundary's own handshake**, and there is no boundary here and
  nothing to hand-shake — while `sdk`, `catalogue-version` and
  `default-profile`, which the other three *ask* the library for, this
  runner prints from constants, because Cargo resolved the graph and a
  resolved graph is what a `const` looks like. `provenance-fnv` is the
  positions envelope's canonical JSON, whose input hash is of the
  boundary's **decoded request record**; a Rust consumer holds the
  `PositionRequest` itself, and the three fields of that envelope anyone
  reads — the profile, the settings hash and the provider's frame — the
  runner prints from the context and the columns. And
  `surface.(root).dispose` is the one operation this surface cannot
  have.

  Growing it found two real gaps, which is what a gate is for.
  `jd_of_fixed` and `fixed_of_jd` were operations the façade did not
  have, and are free functions now, as in the other three, because a
  fixed day and a Julian day are two spellings of one integer and no
  profile or locale changes the arithmetic. And `canonical_json` and
  `content_hash` were not re-exported, so a consumer could hold an
  `Envelope` and not reproduce its own hash.
- **`check-areas`'s property extends to the Rust runner**, which this
  section said it would "once it exists". `RUNNERS` is four now, and the
  Rust runner lists all 38 operations it has — a **list** and not a
  probe, since existence here is a compile-time fact, and the list is
  what the property reads. 155 pairs, 0 disagreeing, 1 allowed:
  `(root).dispose`, carried as *that runner's* allowance rather than as a
  path nobody is held to, so Node is still held to listing it.
- **The measured page turns over**, as §3 says: its second property
  flips from falsified to holding, and that is the acceptance test for
  the refactor rather than a note in a commit message.
- **`check-rust-surface` keeps measuring after the change**, which is the
  point of gating it: the areas' crate counts are what they are, and a
  future operation that pulls a tenth crate into a context shows up as a
  changed page.

  It also grew a third property while the examples were being written,
  and that property was **born red four times over**: *every type an
  area's signature names is reachable from the crate root.* An operation
  answering a type a consumer cannot name is an operation whose answer
  cannot be matched on, and four were —
  `calendar().convert` answers a `CalendarResolution`, `chart().found`
  an `Envelope<ChartFoundation>`, `almanac().of` an
  `Envelope<Vec<Panchanga>>`, and none of those names was re-exported.
  So a consumer with one dependency could call the operation and not
  read the answer, which is the opposite of what this crate is for.

  It is measured as an **intersection** of two readings, because neither
  alone is the question: scanning a signature for capitalised words
  finds `Result`, `Option` and every generic parameter, and scanning a
  module's imports finds the types it uses only in its *body* — the
  founder, the solar model, the tzdb — which are implementation and not
  surface. A type that is both imported from an SDK crate and named in a
  `pub fn` signature is exactly one a consumer must be able to name. 35
  of them, all reachable; the reader's own first bug was calling `Frame`
  and `PositionRequest` unreachable because it read only the first line
  of a braced re-export.
- **`check-rust`, the fifth binding gate**, which runs the examples. What
  it adds over the fast check is the thing a compiler cannot say: that
  each program *runs*. `cargo clippy --workspace --all-targets` already
  compiles an example, and a compiling example can still print a
  falsehood — which is what the install page taught on 12 September, and
  what this one proved again within the hour: the Nepali-new-year line
  in all three other bindings' `panchanga` examples said the Sun "has
  not quite arrived" at Aries and printed `359.9023°` short of it, when
  the Sun had crossed two and a half hours earlier. `(360 - sun) % 360`
  of a longitude just past zero is just under 360. Three files
  corrected.

  It is the only binding gate that needs no shared library, because
  there is nothing to build and load (§3).

  One example is left to another gate: `parity.rs` is run by
  `check-parity` as one of the four runners, and running it here as well
  would pay twice for one proof.

## 8. Order of work

1. **The façade crate, beside the boundary.** `Context`, the builder, the
   eight area views, the ephemeris chain — composing the crates, with
   `teistro-ffi` untouched. Composition written twice, briefly and
   knowingly, and the locale bundles' build script with it.

   Area by area rather than all eight at once, because each is
   independently provable: the first one that compiles and reproduces a
   fact the C smoke test already asserts has proved the whole shape, and
   the rest are that shape again.

   **Done:** the context, the builder, the ephemeris chain, and five of
   the eight areas — `calendar`, `time`, `intl`, `keys` and `frame`.
   `time` is where the dynamic-type section earned itself; `intl` is
   where the module-tree one did.

   The root's **`positions`** is built too, and it is the first
   operation here that computes rather than composes:
   `Completion::new(provider, overrides, delta_t).positions(&request)`,
   answering with the astronomy crate's own `Completed` and no blob
   anywhere. The types it takes are re-exported, so one dependency is
   enough — its own test reached past this crate before they were, which
   is how that gap was noticed. `engine` is built too: one reading of the
   provider's manifest behind two refusals — no ephemeris at all, and an
   ephemeris that describes no operations of its own, which is what the
   built-in is.

   **And `chart` and `almanac`, so all eight areas and the root are
   built.** The runner came first, as the paragraph below asks, and it
   was the right order: with it in place the two compositions were
   checkable line by line, and both agreed on their first run. 125 keys,
   every one a key Node prints, every value identical. The step this
   page's first property was waiting for has happened — *the façade owns
   the composition* now **holds**, 0 of 9.

   **`chart` and `almanac` came after the parity runner**, which
   reversed steps 1 and 2 for those two, and the reason was an
   oracle. Every area so far could be checked against a fact the C smoke
   test already asserts — a Bikram Sambat date, a Kathmandu offset, the
   Sun near 280°. A chart's lagna, day lagna, ayanamsha offset and day
   part are asserted by no smoke test; what holds them is the parity
   report, where the three bindings agree on them value by value. So
   lifting that composition without the runner would be writing numbers
   with nothing to check them against, which this project does not do.

   The runner cannot join `check-parity` as it stands, though, and for
   **two** reasons rather than one. The gate compares key sets, so a
   report missing `chart.found` fails rather than saying "not yet" — and
   a complete Rust report *still* will not match, because ~674 of those
   keys include `abi`, `build-commit`, `build-target` and the blobs'
   sections, which §6 says this surface does not have and should not.
   §7 says what the gate must do instead: hold the Rust report to every
   key the others have except a declared list, printed on every run.

   So the order is: write the runner, run it by hand against the other
   three's report for the operations that exist, build `chart` and
   `almanac` against it, teach `check-parity` the declared absences, and
   wire it in.
2. **The parity runner**, which is what proves step 1 equals the other
   three rather than merely compiling. Red until it does.
3. ~~**Invert the dependency.**~~ **Done.** `teistro-ffi` depends on
   `teistro`, `TsContext` wraps `teistro::Context`, and
   `TsContext::build` — which used to resolve the profile, parse the
   patch, load the embedded bundles, start the locale engine, read the ΔT
   knob and wrap the provider in its cache — is now a call into the
   builder. Both of the measured page's last two properties **hold**.

   It was third for the reason this page gave, and the reason held: with
   the façade built and the parity runner agreeing, the inversion was
   checkable rather than hopeful. Every one of the boundary's 30 unit
   tests and 9 ABI tests passed unchanged, as did `check-c`,
   `check-node`, `check-dart`, `check-python` and `check-parity`'s 674
   values.

   Four things moved out of the boundary with the composition:
   `own_provider` became `own_ephemeris`, answering the façade's
   `Ephemeris` rather than a boxed provider; `remembering` and its three
   tests went with the knob they read; `builtin()` became the variant
   selector, since the façade gates the *variant* on the feature and a C
   caller who passes a number still needs the refusal; and the locale
   bundles' build script, so the boundary's own build script is only
   `build_info` now and has no build dependency at all.

   What stayed: the handles, the `struct_size` handshake, the blob
   writer, the panic guard, the `last_error` slot — and
   `teistro-intl` as a dependency, because the boundary still *marshals*
   the engine's types even though it no longer composes it. The
   composition moved; the types are shared.
4. ~~**Examples**, the same eight scenarios the other three bindings
   run, held by a gate the same way.~~ **Done.** Eight files in
   `crates/sdk/examples/`, run by `cargo xtask check-rust`, with the
   README's table saying what each one is really teaching — the same
   eight scenarios, and each one written as a Rust consumer would rather
   than transcribed.

   Where the surface differs, the example is what says so: a context is
   a value with no `dispose`; `positions` answers the astronomy crate's
   own `Vec<f64>` columns with no blob in between; `CalendarResolution`
   is an enum a `match` must cover; an absent muhurta is an `Option` the
   compiler will not let a reader ignore; there is no `buildInfo` to ask
   for, because Cargo fixed the versions, so `ephemeris.rs` logs the
   provider's `capabilities` instead; and `your_own_ephemeris.rs`
   implements the port rather than handing over an object literal, which
   makes coverage a **per-cell** outcome (`provider::validate` says why)
   where the other three shims refuse the batch.

   Three things the writing found, none of them in the examples:

   - Four signature types were not re-exported (§7), so §7's third
     property exists because of this step.
   - **An almanac's provenance named no provider.** The chart foundation
     stamps it and the almanac did not, in all four bindings, because
     nothing had ever printed the field. `flags_used` there is empty and
     **not** a guess: the chart passes the completion's steps, and this
     path reaches its positions through `FrameLongitudes`, which keeps
     no step list, so there is nothing to vouch for.
   - **A `--no-default-features` build of the façade failed**, and had
     always failed, on the seven examples and `tests/surface.rs` that
     name `Ephemeris::Builtin` — a variant that exists only under
     `builtin-ephemeris`. Nobody saw it because nothing had ever built
     this crate without its default: the tier matrix builds
     `teistro-ephemeris-builtin` and `teistro-ffi`, not this.
     `required-features` on each target is the fix, the two doctests
     that name it are `#[cfg]`-guarded, and `check-lints`'
     `target-declares-the-feature-it-needs` holds the class by reading
     the sources — so a target that stops naming the built-in stops
     needing the line, and one that starts cannot be added without it.

   And one thing it deliberately did **not** do: the eight files repeat
   small helpers — a clock formatter, a sign lookup, an
   entity-name-or-key fallback. An example is a program a reader is
   invited to *copy*, and a shared `support` module would make every one
   of them un-copyable. The DRY rule applies to what ships.
5. ~~**The site.**~~ **Done.** The surface page has its Rust column —
   the fourth spelling, an area as a borrowing view, and a section on
   what Rust does differently: it composes rather than crosses, so no
   `dispose`, no blob, `positions` answering `Completed`, and a context
   that is neither `Send` nor `Sync`.

   Three things joined that section once the examples had been written,
   because writing them is what found them worth saying. **The types are
   stricter in two places rather than merely different**: a date's
   resolution is an enum a `match` must cover, and a value a day may not
   have is an `Option` the compiler will not let a reader ignore. **There
   is no `buildInfo` and none is wanted** — Cargo resolved the versions
   and no ABI is crossed, so what a service logs at start-up is the
   *provider's* capabilities. And **an ephemeris of your own is a trait
   you implement**, which makes coverage a per-cell outcome where the
   other three shims refuse the batch; that difference is on the page
   rather than left for a reader to discover, because a provider author
   moving between two of these bindings would otherwise meet it as a
   surprise.

   The install page's Rust section is **deliberately still not
   written**. The crate is `0.0.0` and `publish = false`, so a
   `cargo add teistro` would be an instruction nobody can follow —
   which is the lesson that page already carries about documenting what
   nobody has run. It gets its section when there is a release to name.

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
- **Whether a context should be shareable across threads.** It is not,
  today, and the building found out which part decides that. Not the
  ephemeris: the port requires `Send + Sync` of a provider. The **locale
  engine** — `teistro_intl`'s plural rules hold an
  `icu_plurals::PluralRules` whose data payload is reference-counted with
  `Rc`, so a `Context` is neither `Send` nor `Sync`.

  Every binding already says *one context serves one thread, and a worker
  builds its own*, so this is the stated rule enforced rather than a new
  limit. But it lands differently in Rust, where the obvious shape for a
  server is one context in an `axum` app state behind a `&`, and `!Send`
  forbids it — a Rust consumer needs a context per worker or a pool.

  What it would take is known and checked rather than guessed:
  `icu_provider` has a `sync` feature (`sync = []` in its manifest) which
  moves those payloads to `Arc`. Enabling it is cross-cutting — it costs
  atomic reference counts on every render in every binding, and the
  boundary's `TsContext` is `!Send` for the same reason — so it is a
  decision with a measurement behind it, not a line in this page. **A
  falsification pass belongs in front of it**: what a render costs with
  `Rc` and with `Arc`, over the message set `check-intl` already walks.
