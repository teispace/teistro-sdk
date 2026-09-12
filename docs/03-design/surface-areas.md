# The surface areas

Status: `designed`, written 2026-09-12 from the falsification pass in
[`surface-areas-measured.md`](surface-areas-measured.md). Decides
ADR-0030's first and fourth points — the surface becomes
`sdk.<area>.<operation>`, and the engine's own functions sit under
`engine` and are not hoisted to the root. Derives from ADR-0002 (an
agnostic port), ADR-0023 (type safety in every binding) and ADR-0030
(the consumption surface). `02-architecture/07-binding-architecture.md`
gives each binding its row.

## 1. Purpose and scope

A context carries **35 members on one object** today, measured, and the
phases that remain add dashas, strengths, rules, interpretation, five
traditions and eleven application modules. This design decides the
grouping those go into, before the four binding layers are restructured
around it, because the change is cheap now and breaking after v1.

In scope: the areas, what is in each, what stays at the root, what
happens to the names that already spell their area, and how each
binding's own idiom expresses an area. Out of scope: renaming operations
for their own sake. **Namespacing moves a name; it does not rename an
operation.** Where a name loses a word it is because the word became the
namespace, and every such case is in the measurement.

## 2. What the measurement decided, and what it left

The pass falsified the ADR's derivation rule four ways and left the
grouping standing. What it decided:

- **The areas are not the boundary modules.** Five of the fourteen are
  never reached by anything a consumer calls — `strings` and `blob` are
  the C caller's memory, `context` and `provider` are the context's own
  life, `lib` is the library — so they are not areas and nothing is
  found under them.
- **Size is not a guide.** `calendar` has eight entry points and
  `chart`, `positions` and `panchanga` have one each, and those three
  are what the SDK exists for. An area sized by its entry points would
  rank the surface almost backwards.
- **Six members already spell their area inside their own name.** Those
  six lose the half that becomes the namespace and none of them needs a
  new word invented.

What it left to this page: **the names**. No count decides whether the
engine's namespace is called `ephemeris` or `engine`, or whether an area
holding one operation is an area at all.

## 3. The areas

Seven, and a root.

| area | operations | from |
|---|---|---|
| `calendar` | `dateOf`, `fixedOf`, `convert`, `weekdayOf`, `monthLength`, `isLeap` | `calendar` |
| `time` | `resolve`, `civilOf`, `convert`, `deltaT` | `time` |
| `intl` | `locale`, `render`, `has`, `entity`, `messages`, `transliterate`, `loadPack` | `intl` |
| `keys` | `id`, `name` | `keys` |
| `frame` | `canonical` | `frame` |
| `chart` | `found`, `foundMany` | `chart` |
| `almanac` | `of`, `day` | `panchanga` |
| `engine` | `manifestJson`, `manifest`, `names`, `signature`, `call`, `callJson` | `ephemeris` |

Four names change, and each is a word the namespace now carries:
`convertTime` → `time.convert`, `canonicalFrame` → `frame.canonical`,
`keyId` → `keys.id`, `keyName` → `keys.name`. Two more lose a prefix:
`ephemerisManifestJson` → `engine.manifestJson` and
`ephemerisCallJson` → `engine.callJson`.

### The rule for an area of one

**An operation whose name is its own area's name is a root operation and
not an area of one.** `positions` is the case: `sdk.positions.positions`
is absurd and `sdk.positions.of` invents a word to fill a slot that only
exists because a rule demanded one. It stays `sdk.positions(request)`,
which is also honest about what it is — the SDK's one primitive over the
port, the call every area above is built on.

`frame` keeps its area on the same rule read the other way: its single
operation is `canonical`, which is not the area's name, and a frame is a
thing a consumer holds rather than an operation they perform — every
request carries one. `sdk.frame.canonical()` reads as what it is.

### `almanac`, not `panchanga`

The boundary module is `panchanga` and the binding member is `almanac`,
and they have disagreed since the module was written. The area takes the
**consumer's** word: an almanac is what the operation answers — a day, or
a run of days, with its limbs — and `panchanga` is one tradition's name
for five of those limbs. The boundary keeps `ts_panchanga_days`, because
renaming an entry point is an ABI change for a word, and this page's own
rule is that namespacing does not rename operations.

This is the one place where an area's name is not its module's, and it is
recorded here so the next reader does not "fix" it.

### `engine`, not `ephemeris`

ADR-0030 settles this and the measurement cannot: `engine` says *this
particular engine, not the portable contract*, and a consumer reading
their own code should see the difference between a call that survives
changing provider and one that does not. `sdk.ephemeris.*` says the
opposite — that it is the ephemeris contract — which is precisely the
thing it is not.

Node ships `context.ephemeris` today. It becomes `sdk.engine`, with no
alias: two spellings for one call is the defect this project keeps
finding under other names.

### The dynamic index signature comes out

Node's `Engine` declares `[operation: string]: unknown`, which
ADR-0030 considered and **rejected** under ADR-0023: it type-checks
everything, including the misspelling, and Dart and Rust cannot express
it, so it would be a surface that exists in two of five targets. It goes.
What replaces it is the typed façade the adapter generates — and until
an adapter ships one, `sdk.engine.call(name, args)` is the route, typed
in its arguments and not in its name.

## 4. What stays at the root

The context is not an area and the things that are about the context
itself stay on it:

- `positions(request)` — the primitive, by the rule above.
- `profile`, `settings`, `settingsJson`, `settingsHash` — what this
  context resolved to. These are the provenance a stored chart keeps.
- `dispose()` and the constructor — its life.

Everything else moves into an area. The measurement's ten members that
reach no entry point are cached values, decoded results and delegates;
each goes where the member it delegates to goes.

## 5. Namespaces are values

ADR-0030: "Each is built once when the context is, holds the context, and
is frozen. Nothing allocates per call, and a consumer may destructure one
and keep it."

That is a requirement on all four bindings and it is what makes the areas
worth having rather than merely tidy:

```js
const { calendar, almanac } = sdk;          // Node, and it keeps working
```

```dart
final calendar = sdk.calendar;              // Dart: a field, not a getter
```

```python
calendar = sdk.calendar                     # Python: a cached property
```

```rust
let calendar = sdk.calendar();              // Rust: a borrow of the context
```

Per binding:

- **Node** — a small frozen class instance per area, built in the
  constructor. **Corrected in the building**: this page first said a
  frozen object literal, "because there is nothing to inherit and a
  frozen literal is one allocation". It is not one allocation — a literal
  of six arrow functions is seven, per context — and a class puts the
  methods on a prototype shared by every context. Each area holds one
  closure, the way in, and nothing else. Not a getter, because a getter
  that builds on every read breaks destructuring's promise.

  Destructuring an *area* works, which is the requirement.
  Destructuring an *operation* out of one does not, because a prototype
  method needs its receiver — and that was already true of every method
  on the flat surface, so nothing regresses.
- **Dart** — a `final` field holding a small class that keeps the
  context. Dart has extension methods, and they were rejected: an
  extension is resolved statically and cannot be destructured or passed,
  and the requirement is that an area is a **value**.
- **Python** — `functools.cached_property`, so the first read builds it
  and every later one is the same object.
- **Rust** — a method returning a borrowing view struct. A field would
  make the context self-referential; a view is one word and cannot
  outlive what it borrows.

## 6. The boundary's own names

The pass found four entry points whose names do not carry their module.
Six others are `lib`'s, which the rule does not cover. Of the four:

- `ts_key_parse` and `ts_key_name` live in `keys.rs`, and
  `ts_string_free` in `strings.rs`. **The files are renamed, not the
  functions** — `key.rs` and `string.rs` — because renaming an entry
  point is an ABI change and renaming a file is not, and because the
  function names are the ones a consumer sees and they are already
  consistent with each other.
- `ts_context_new_with_provider` lives in `provider.rs` and is named for
  the context it builds. It stays, as a **declared exception**: it
  belongs to `context`'s family by name and to `provider`'s by what it
  depends on, and the name a consumer calls is the one that should read
  well. The pass keeps counting it, because a rule with its exceptions
  written into it cannot be falsified.

The reference site's groups follow the files, so `keys` becomes `key` and
`strings` becomes `string` there. The *areas* keep `keys` plural, because
`sdk.keys.id` reads as a collection being indexed and `sdk.key.id` reads
as a single key that is not there.

## 7. What the gates gain

- **`check-areas`** holds the measurement, so a member added to the Node
  layer appears on the page and a module that stops being reached is
  reported.
- **`check-parity`** compared values and not shapes; it does both now.
  Each runner prints a `surface.<area>.<operation>` line per operation,
  keyed by a **canonical** path and referencing its own binding's
  spelling of the member, so a binding that moved an operation or renamed
  one prints a key the others do not and the gate reports it on both
  sides. Proven red by moving `calendar.convert` to `time.convert_date`
  in one runner: four disagreements, naming the extra key and the missing
  one in each comparison.

  Referenced rather than called, because a call needs arguments and some
  of them need an ephemeris, and what is being compared is the shape. In
  Dart the reference is a tear-off resolved at compile time, so a member
  that moved does not print `missing` there — the runner stops
  building.
- **`check-surface`** keeps measuring the mechanical layer and is
  unaffected: areas are the ergonomic layer's shape, and nothing
  generated moves.

## 8. Order of work

1. ~~The boundary's two file renames~~, which regenerated the description
   and the site's groups and moved no ABI symbol. **Done.**
2. ~~Node~~, with its tests, its eight examples, its typecheck and its
   parity runner. **Done**, and `check-parity` proved it
   behaviour-preserving: all 635 values still agree with Dart and Python,
   which had not changed.
3. ~~Dart~~ and ~~Python~~, each in its own idiom — a `late final` field
   and a `cached_property` — and each green on its own gate. **Done.**
4. ~~`check-parity` gains the grouping~~, which is what holds all three to
   one shape. **Done**, and proven red.
5. Rust's own consumer surface, the READMEs and the site's prose.

The measured page turned over as step 2 landed, which was expected: it
measured a flat surface and the surface is not flat any more. It now
holds six properties of the **built** grouping — a module reached from
one area, no empty area, no operation spelling its own area, every module
reached or accounted for as plumbing, the boundary's own naming, and
every operation listed by every parity runner — and the argument it used
to make is kept here.

The sixth was added after the other five and was **born red**, which is
why it is there. Step 4 gave `check-parity` the grouping, and that gate
compares what the three runners *print*; the list of canonical paths is
written out once per runner, in three languages, so three runners that
all miss the same new operation agree perfectly and the gate is silent.
The measured page reads all three lists against the layer's own
declarations instead — and found `(root).engine`, the accessor a
consumer reads to reach the engine at all, listed by none of them.

The engine's typed façade is **after** all of it, because it attaches to
`sdk.engine`, which step 2 creates.

## 9. What this design does not settle

- **The areas the remaining phases add.** Dashas, strengths, rules and
  the application modules each want an area, and this page deliberately
  does not name them: an area invented before its operations exist is a
  slot that shapes the work to fit it.
- **Whether an operation should be promoted out of `engine`.** ADR-0030's
  fifth point says what proves universal becomes an SDK operation with a
  portable contract. Nothing here decides which ones have.
- **The Rust consumer surface's own shape.** Rust reaches the crates
  directly as well as through a context, and whether the areas are the
  only way in is a question for the Rust binding's own page.
