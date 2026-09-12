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
| the root | `profile`, `settings`, `settings_hash`, `ephemeris`, `intl` | built |
| `time`, `intl`, `keys`, `frame`, `chart`, `almanac`, `engine`, and the root's `positions` | — | next |

The operations in an area are the ones
[`surface-areas.md`](../../docs/03-design/surface-areas.md) puts there and
no others, and that is not tidiness: `check-areas` holds every binding's
list to the same canonical paths, so an operation invented here would be
one the other three lack.

**Still owed**, and both are in the design page's order of work: the
fourth parity runner, which is what proves this equals the other three
rather than merely compiling, and the dependency inversion that moves the
composition out of `crates/ffi` and has the boundary call this crate. The
composition is written twice until then, knowingly — and so is the
thirty-line build script that compiles the locale bundles, because a
build script cannot use the crate it builds.

## Testing it

```sh
cargo test -p teistro
```

`tests/surface.rs` asserts the facts `bindings/c/tests/smoke.c` already
asserts, in the same words, so a difference between the two compositions
is a failing test rather than something a reader has to notice.
