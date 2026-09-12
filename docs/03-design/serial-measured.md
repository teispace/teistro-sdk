# The canonical form, measured

Status: `generated` by `cargo xtask serial`, 2026-09-07. Do not edit:
`check-serial` regenerates this page and fails on any difference. The
design written from it is
[`serial-and-the-envelope.md`](serial-and-the-envelope.md).

## 1. What can be measured about a serialisation

Nothing is recorded to compare a serialisation against, so this is the
`aspect` kind of pass rather than the `vargas` kind. What it can measure
is the form's **own invariants** over real values, and what the SDK does
against what it says it does — for which it reads the source, as
`check-lints` does.

The sample is the corpus itself: 55 fixture documents and the 193 366
numbers inside them, which are real longitudes, speeds, distances and
instants rather than round figures chosen to be easy.

## 2. What a stamped value actually carries

`Provenance` has every field ADR-0020 asks for: the calculation version,
the input hash, the provider's data hashes, the Delta T model, the leap
table, the zone database, the calendar's resolution, a classical model's
deviation, the conventions applied, the confidence, and **the hash of
the value itself**.

What no test had checked is whether anything fills them.
`Provenance::new` takes the identifying six as arguments and leaves the
rest empty, so a producer that forgets a field ships a value that claims
less than it knows — and one of those fields is the hash the whole
envelope exists to carry.

| proposed rule | verdict | measured |
|---|---|---|
| every producer stamps the hash of the value it produced | **holds** | 0 of 4 disagree |
| every field the envelope documents is filled by someone | falsified | 5 of 10 disagree |

| producer | fills |
|---|---|
| `crates/chart/src/foundation.rs` | `content_hash`, `provider`, `time.delta_t_model`, `time.leap_table` |
| `crates/ffi/src/positions.rs` | `content_hash`, `provider`, `time.delta_t_model`, `time.leap_table`, `time.tzdb_version` |
| `crates/panchanga/src/almanac.rs` | `content_hash`, `provider`, `time.delta_t_model`, `time.leap_table` |
| `crates/serial/src/seal.rs` | `content_hash` |

**`content_hash` is the hash of nothing on none of the 4.**
`Provenance::new` still sets it to `Hash::of(&[])` as a placeholder, and
every producer now replaces it — but not by remembering to. The shape
problem this section found is answered the way `crates/serial` answered
it: a value and its stamp are joined by a constructor that knows both,
so the one field that cannot be filled until the value exists is filled
where it can be. `Envelope::sealing` is that join for `chart` and
`panchanga`, and the four callers that used to mend the stamp afterwards
— the boundary's two entry points and the Rust façade's two areas —
no longer do. Four callers writing the same line is what decided it.

Nothing at all fills 5: `module_versions`, `packs`,
`time.delta_t_seconds`, `calendar`, `applied_conventions`. Each is a
field the envelope documents and nothing sets, which a consumer reading
the schema would expect to find.

## 3. The canonical form is canonical

Two bindings that agree on a value have to agree on the bytes, or the
content hash is not a content hash. `core`'s `canonical_json` sorts
every object's keys explicitly rather than trusting the JSON layer's
map, whose ordering changes with a feature any crate in a build may
enable — and this measures that over the corpus's own documents, which
nest 8 deep.

| proposed rule | verdict | measured |
|---|---|---|
| every object's keys come out in code-point order, at every depth | **holds** | 0 of 55 disagree |
| the same value gives the same bytes twice | **holds** | 0 of 55 disagree |
| and gives them however the value's own maps were ordered | **holds** | 0 of 55 disagree |

The third claim is the one worth having. Rebuilding every object in the
reverse of its original order and canonicalising again gives the same
bytes on all 55 documents, so the form depends on the value and not on
how the value was built.

## 4. How a double is written, which is where two bindings part

A canonical form has to say how a number is written, because the bytes
are what is hashed. Rust's JSON layer writes the shortest decimal that
reads back as the same double, and so does JavaScript's — but the two
**switch to an exponent at different magnitudes**, and a value written
`0.000001` in one and `1e-6` in the other has two hashes.

| proposed rule | verdict | measured |
|---|---|---|
| every number of the corpus round-trips through the form | **holds** | 0 of 193 366 disagree |
| the form never reaches for an exponent | **holds** | 0 of 193 366 do |
| `output.precision` has a reader | **holds** | 0 of 1 disagree |

Measured over the corpus's own numbers: every one round-trips, and
0 of them are written with an exponent already; the widest
form is 38 characters, `-0.00000000000000000037731895548623034`.

The form reaches for an exponent **below 10^?** and at or above
10^?. That first threshold is the one that bites, and here is the
case, written out:

| value | this form writes | a JavaScript binding writes |
|---|---|---|
| 10^-6 | `0.000001` | `0.000001` |
| 10^-7 | `0.0000001` | `1e-7` |

`JSON.stringify` switches at 10^-7 and this switches at 10^-6, so a
single value one millionth of a degree from nought hashes two ways in
two bindings that agree about the number. Nothing in the corpus is quite
that small, but a speed near a station, a residual or the difference of
two longitudes very easily is.

`output.precision` names the decimals of an angle, an instant and a
score, and until `crates/serial` was written **nothing read it** — the
third such knob found in as many modules, after `state.combustion_orbs`
(registry entry 23) and `houses.module_overrides`, which is a pattern
rather than an accident and worth a gate of its own. It now has
`src/canonical.rs`.

It governs the **rendering** and not the hash: a hash that moved with a
display setting would be a worse cache key, and the settings hash
already tells two results computed under different precision apart. The
hash form's own grammar is fixed, which is what the second claim
measures.

## 5. The content hash is a function of the value

The hash is the cache key's third part and the thing a consumer compares
when it wants to know whether two results are the same answer. It has to
depend on the value and on nothing else.

| proposed rule | verdict | measured |
|---|---|---|
| a value's hash does not depend on how the value was built | **holds** | 0 of 55 disagree |
| no two documents hash alike | **holds** | 0 of 55 disagree |
| a changed value changes the hash | **holds** | it does |

All three hold over the 55 documents, which is what makes the missing
`content_hash` of §2 a real loss rather than a cosmetic one: the hash
works, and almost nothing carries it.

## 6. What this pass decides

- **A value and its stamp are sealed together.** 0 of the
4 producers ship a `content_hash` that is the hash of nothing,
because the field cannot be filled until the value exists and a
caller has to remember. The module makes the sealing the only
way to build the pair, so the constructor computes it.
- **The canonical form holds.** Keys in code-point order at every
depth, the same bytes twice, and the same bytes however the
value's own maps were ordered.
- **A number needs a stated format.** The corpus's own numbers
round-trip, but the form reaches for an exponent where a
binding's own would not, and `output.precision` — which says
how many decimals a quantity is written to — has no reader. A
canonical form that writes to a stated precision is one two
bindings can agree on without agreeing about their float
printers.
- **The hash works.** It depends on the value, on nothing else,
and moves when the value does — which is why the field being
empty matters.
