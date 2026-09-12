# The Rust binding

Status: `built`, 2026-09-12 — the first increment of
[`03-design/rust-consumer-surface.md`](../../docs/03-design/rust-consumer-surface.md)'s
order of work.

Node, Dart and Python read `sdk.<area>.<operation>` off a context. This
crate is that, in Rust. It is **designed** rather than improvised: the
design page above decides its shape, from the measurement in
[`rust-consumer-surface-measured.md`](../../docs/03-design/rust-consumer-surface-measured.md),
which is generated and gated.

```rust
use teistro::catalogue::Calendar;
use teistro::{CalendarDate, Context, Ephemeris};

let sdk = Context::builder()
    .profile("nepali-default")
    .locale("ne-Deva-NP")
    .ephemeris([Ephemeris::Builtin])
    .build()?;

let day = CalendarDate::defined(Calendar::Gregorian, 2015, 4, 14);
let bs = sdk.calendar().convert(&day, Calendar::BikramSambat)?;
```

## What it is, and what it is not

**It composes the crates; it does not call the SDK's own C ABI.** The
measurement found that eight of the boundary's forty-six entry points are
marshalling with no computation in them, so a Rust consumer reaching
itself through C would write a request struct for the boundary to decode
and decode a result blob to read numbers the crates had already returned
as `JulianDay` and `Longitude`. Rust would be the only binding whose
implementation language is its consumption language and which still
crossed a C ABI to reach itself.

**An area is a borrowing view**, which is what it is in every other
binding — a frozen instance in Node, a `late final` field in Dart, a
`cached_property` in Python. `sdk.calendar()` costs a pointer copy,
allocates nothing, and cannot outlive its context, so a consumer who
wants to keep one writes `let cal = sdk.calendar();` exactly as a Node
consumer writes `const cal = ctx.calendar`.

**No `dispose`.** `Drop` is the whole of it, and that is the first place
this surface is smaller than the C one rather than the same size.

**A context is neither `Send` nor `Sync`**, so a worker builds its own —
the rule every binding states, here enforced by the compiler. The part
that decides it is not the one you would guess: the port requires
`Send + Sync` of an ephemeris, and it is the locale engine whose plural
rules hold an `Rc`-backed icu4x payload. `tests/surface.rs` builds one
per worker, which is the pattern this leaves; the design page's §9
records what lifting it would take and why that wants a measurement
first.

## What is here, and what is next

| area | operations | state |
|---|---|---|
| `calendar` | `date_of`, `fixed_of`, `convert`, `weekday_of`, `month_length`, `is_leap` | built |
| `time` | `resolve`, `civil_of`, `convert`, `delta_t` | built |
| `intl` | `locale`, `set_locale`, `render`, `render_typed`, `has`, `entity`, `transliterate`, `load_pack` | built |
| `keys` | `id`, `name` | built |
| `frame` | `canonical`, `pack`, `unpack` | built |
| the root | `positions`, `profile`, `settings`, `settings_hash`, `ephemeris`, `locale_engine` | built |
| `engine` | `manifest`, `manifest_json`, `names`, `signature`, `call`, `call_json` | built |
| `chart` | `found`, `found_many` | built |
| `almanac` | `of`, `day` | built |

**All eight areas and the root**, and the parity runner is what says so
rather than the compiler: 125 keys, every one of them a key the Node
runner prints, and every value identical.

`intl`'s **`messages`** is not a method here. The typed accessor tree
every other binding spells
`sdk.intl.messages.sdk.reason.grahaInBhava({ … })` is a *module* tree in
Rust — `teistro::messages::sdk::reason::GrahaInBhava { graha, bhava }`,
handed to `render_typed` — because a module tree is what a namespace is
in this language. `teistro-intl` already generates it.

`locale_engine` at the root is the engine itself, for what the area does
not wrap; it is deliberately not called `intl`, which is the area's name
in all four bindings.

**`positions` is at the root** because an operation whose name is its own
area's name is a root operation — nobody writes
`sdk.positions.positions`. It answers with the astronomy crate's own
`Completed`: a Rust consumer reads a longitude off `sky.columns.at(0, 0)`
where every other binding decodes a result blob to get the same number
back as a double. The types it takes and answers with are re-exported
here, so one dependency is enough; the test for it reached past this
crate before they were, which is how the gap was noticed.

`time.convert` is where this surface owns a type rather than
re-exporting one. The crates keep a scale in the *type system* —
`JulianDay<Ut1>`, `<Tt>`, `<Utc>` — which gives a caller who names a
scale at run time nothing to name it with, so `Scale` and `Conversion`
are the surface's. That is the rule and not an exception: the surface
owns a type exactly where an operation is dynamic and the crates are
static. It answers with **what was applied** as well as the number,
because a ΔT of 63.8 seconds from one model is not the same answer as
63.8 from another.

The operations in an area are the ones
[`surface-areas.md`](../../docs/03-design/surface-areas.md) puts there and
no others, and that is not tidiness: `check-areas` holds every binding's
list to the same canonical paths, so an operation invented here would be
one the other three lack.

**And the boundary depends on this crate**, which was the last step of
the design's order of work. `TsContext` wraps a `teistro::Context`, and
`TsContext::build` — which used to resolve the profile, parse the patch,
load the embedded bundles, start the locale engine, read the ΔT knob and
wrap the provider in its cache — is a call into the builder above. So
the composition has one home, and a C caller and a Rust consumer get the
same context built the same way rather than two compositions kept equal
by hand.

The boundary keeps what only it needs: the handles, the `struct_size`
handshake, the blob writer, the panic guard, the `last_error` slot. It
keeps `teistro-intl` too, because it still *marshals* the engine's types
— the composition moved, the types are shared. Its build script is only
`build_info` now; the locale bundles are built here.

## The examples

Eight programs in [`examples/`](examples/), one per scenario, each
runnable and each run by `cargo xtask check-rust` so none of them can
describe a surface nobody has executed:

```sh
cargo run --release -p teistro --example birth_chart
```

Release, because the built-in ephemeris is a truncated VSOP87 and
ELP2000 and a debug build of it computes a year of the sky slowly enough
to notice. [`examples/README.md`](examples/README.md) has the table of
what each one is really teaching, and each file repeats its own small
helpers on purpose: an example is a program a reader is invited to
*copy*.

## The parity runner

`examples/parity.rs` prints this binding's half of the parity report:
`key<TAB>value` lines, sorted, the same scenario the other three runners
walk. Run it against one of theirs:

```sh
cargo run -p teistro --example parity > /tmp/rust.tsv
(cd bindings/node && node parity.mjs) > /tmp/node.tsv
join -t $'\t' /tmp/rust.tsv /tmp/node.tsv | awk -F'\t' '$2 != $3'
```

**519 keys, every one of them a key Node prints, and every value
identical** — which is what proves this composition equal to the one at
the C boundary rather than merely compiling. It was also the oracle the
last two areas needed: a chart's lagna, day lagna and ayanamsha offset,
and an almanac day's sunrise and window, are asserted by no smoke test,
and this report is where the bindings agree on them. It began at 125 and
every key added since has agreed on its first run.

It does **not** print every key the other three do, and that is §6 of the
design page arriving in a gate rather than a gap. Among theirs are `abi`,
`build-commit`, `build-target` and the result blobs' sections: a Rust
consumer has none of them, because Cargo resolved the versions, `Drop`
freed the memory, and the crates handed back their own types. So
`check-parity` holds this report to every key it *prints* and **counts**
the 155 it does not on every run, which is what makes a subset that
stops shrinking visible.

## Testing it

```sh
cargo test -p teistro
```

`tests/surface.rs` asserts the facts `bindings/c/tests/smoke.c` already
asserts, in the same words, so a difference between the two compositions
is a failing test rather than something a reader has to notice.

`cargo xtask check-rust` runs those, the doctests and all eight
examples, and is the gate CI runs on five platforms.

Every target that names `Ephemeris::Builtin` declares
`required-features = ["builtin-ephemeris"]`, so
`cargo build -p teistro --no-default-features` skips them rather than
failing on them; `check-lints`'
`target-declares-the-feature-it-needs` holds that by reading the
sources.
