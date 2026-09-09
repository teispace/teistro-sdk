# Implementation plan: the batch, the threads and the engine passthrough

Status: `plan`, written 2026-09-09. Two bodies of work the maintainer
asked for in one sitting, kept in one document because they answer the
same brief — **an SDK a consumer never has to work around**. The first is
measured (`03-design/batch-and-parallelism-measured.md`); the second is
designed here and measured in the same pass. Read
[`../STATUS.md`](../STATUS.md) first: it says which step of this is done.

## Why these two, together

The maintainer's standing brief has two halves that turn out to be the
same requirement seen from either end.

**No dead ends.** A consumer must be able to change anything and reach
anything. Defaults yes; a wall, no. The largest wall in the project is
that the ephemeris port names **eight** operations and Teimeris's public
header names **161 functions, 57 structs and 40 enums** — so a consumer
who wants an eclipse, a star, an orbit or the engine's own calendar grid
must open a second handle to the same engine and manage it themselves.

**Efficiency designed in, not bolted on.** Batching, parallelism and
threads from the start rather than as a later optimisation. The
measurement says the SDK is at the opposite end today: `positions` is a
true batch (one call whatever its size) and **everything built on it is a
loop** — 69 ephemeris calls per chart, 1 213 per almanac day, at a mean
width of 1.1 cells through a port designed for grids.

Both are the same defect in different clothes: the SDK decides for the
consumer how the engine is used, and decides it narrowly.

## Part A — the batch, the cache and the threads

Designed from `03-design/batch-and-parallelism-measured.md`, whose
numbers are counts and therefore gated (`check-batching`). **The order is
the point**: threads would multiply redundant work rather than reduce it,
so the arithmetic is fixed first and the hardware spent last.

### A1. Size a search by what it is searching for, then grid what is left

*Corrected 2026-09-09, before a line of it was written.* The step as
first planned read: "one sunrise costs 18 provider calls … seventeen
round trips in eighteen become one". Attributing an almanac day's calls
to the code that makes them falsified all three of its premises, and the
correction is worth more than the original.

*What the attribution found.* One almanac day (Kathmandu, 2024-04-01,
the analytic provider) makes 1228 provider calls. Recorded by a probe
that captured the caller of every one of them:

| calls | share | what asks |
|---:|---:|---|
| 543 | 44% | `limb::signs` for the **Moon**, over a window of ±40 days |
| 344 | 28% | `Almanac::moon`: the Moon's rise and set |
| 101 | 8% | `limb::signs` for the **Sun**, over the same ±40 days |
| 72 | 6% | the Sun's rise and set, twice (the day, and the night before it) |
| 32 | 3% | `find_sankranti` |
| 136 | 11% | the lunar month's span, the masa, and the four limbs |

The premises that fell:

1. **A sunrise does not cost 18 calls.** It costs **4**, and all four
   are Meeus's iteration, which is serial by construction: each instant
   is computed from the sample before it, so there is no grid to ask
   for. A whole `day()` — a rise, a set and a midday reading — costs 9.
   The 18 on the measured page is `SolarModel::day_light`, which is two
   solvers and a second day.
2. **The scan the plan meant to grid does not run** at a temperate
   latitude. It is the fallback for when the iteration will not settle.
   At 69.65°N it does run, and there it costs 146 calls an event — so
   gridding it is worth doing, for the places where it is reached.
3. **The dominant cost is not a round trip at all. It is a window.**

*The finding, stated plainly.* To report which signs the Moon stood in
during **one day**, the SDK searches **eighty-one days** of sign
crossings and finds about thirty-five of them, when the day can touch at
most two. The constant that sizes it says why:

> `const SIGN_SEARCH_DAYS: f64 = 40.0;`
> "The Sun takes a month to cross a sign and the Moon two and a half
> days … Forty days covers the Sun."

Forty days *does* cover the Sun, and the same helper serves both bodies,
so the Moon is searched at the Sun's reach — a body that crosses a sign
every 2.3 days, looked for over ±40. Nothing is wrong with the answer;
the search is simply sized for the slowest thing that uses it.

*The change, in three parts, in this order:*

**A1a. The reach follows the body — built.** `events::greatest_rate`
already tables how fast a body can move, because the search's *step* is
sized from it. Its companion is `events::least_rate`, how slowly a body
can move, which is what sizes the search's *reach*:
`events::longest_dwell_days(quantity, lattice)` turns the two into the
longest a value can stand between two lattice lines. Thirty degrees at
the Sun's slowest is **31.579 days**, at the Moon's **2.564**, and
`None` for anything that can retrograde — a body that turns can cross a
line and come back, so nothing bounds its dwell, and there today's
constant stays as a *cap* (`SIGN_SEARCH_CAP_DAYS`) with its documented
truncation. A caller who wants another reach asks for one:
`limb::signs_within(…, reach_days)` beside `limb::signs`, `None`
resolving to the body's own, and a reach that is not a positive number
of days refused by name.

*Measured, on the gated page:* an almanac day fell from **1228 calls to
841** and fifty days from **60 631 to 41 492** — a third of the work,
gone, for a rule that was already implied by a table the search half
used. Charts are untouched, because a chart does not read a sign span.

The elongation's least rate falls out of the same table (the Moon's
least less the Sun's greatest, 10.75°/day), which bounds the slowest
tithi at 1.12 days and so *derives* `limb::LONGEST_SPAN_DAYS`, until now
a hand-chosen 1.5. Left as it is, and now covered by a test that says
why it is enough.

**A1b. Grid the uniform scan — built.** `events::Search::between` walked
its window in fixed steps, one round trip a step, when every instant it
would ask for was known before it asked for the first. The scan now
produces its instants by the same recurrence and asks for them a **grid**
at a time; only the ITP refinement stays serial, because each of its
steps is chosen from the answer to the last and so cannot be asked for
in advance.

`Longitudes` grows the grid beside the pair method it already had —
`longitudes_and_speeds` and `longitudes_and_speeds_pair`, defaulting to
the walk, overridden by `FrameLongitudes` with one `positions` request
and forwarded by the panchanga's `Sidereal` shift. The chunk is
`Search::with_chunk`, defaulting to `SCAN_CHUNK` = 512 instants, which
is a **memory** bound and not a waste bound: the scan visits every
instant between its ends, so every sample in a chunk is one it needs.

*Measured:* an almanac day fell from **841 calls to 665**, and fifty days
from 41 492 to 32 692. The widest call rose from 2 cells to 66, which is
the grid becoming visible; the **cells are unchanged**, because the same
work is being asked for in fewer requests. Over a four-hundred-day
ingress search the scan's 401 round trips became **1**.

*And this one is bit-identical*, not merely inside a tolerance: the
instants are the same instants and the values the same values, so the
brackets handed to the refinement are the same brackets.
`tests/events.rs` holds the grid against the walk over a retrograde
planet — three quantities, two lattices, five chunk sizes — comparing
`to_bits()` and the evaluation counts, and every generated page
regenerates unchanged.

**A1b′. A crossing is a property of the crossing, not of the question —
built, and it was a defect.** Building A1c asked a question first: is a
crossing found in a range-wide search the crossing a per-day search
would have found? Measured, no. The scan stepped from the caller's own
`from`, so where its samples fell — and so which bracket the refinement
was handed — depended on where the window started. The same sign
ingress came back **up to 2.2 milliseconds apart** from windows offset
by a fraction of a day, and only four of fifteen comparisons agreed to
the bit.

Samples are now aligned to `SCAN_ANCHOR_JD` (J2000.0) and computed as
`anchor + k × step` by multiplication rather than by accumulating steps,
so a narrower window's samples are a **subset** of a wider one's and any
two windows that both contain a crossing bracket it identically. The
ends of the scan are lattice points rather than the caller's instants,
so a bracket may reach outside the window; a crossing found out there is
real but not this window's, and is dropped rather than reported.

*What it cost, and what it bought.* An almanac day rose from 665 calls
to **681**, 2.4%, which is the extra sample at each end. In exchange the
distinct cells of a fifty-day range fell from 39 675 to **21 446** and
the repeat share rose from 24.7% to **60.2%** — because consecutive days
now ask for *the same instants* rather than nearby different ones. The
overlap between neighbouring days was always there; before this it was
invisible to anything that could exploit it. That is what makes A1c and
A2 possible, and it is worth having for its own sake besides.

**A1c. Hoist what a range shares.** Consecutive days in a range search
almost the same window, which is why 50 almanac days cost 50 × a day and
why the repeat share rises with the batch. A range computes its shared
crossings once and slices them per day.

*Reordered by the measurement above.* With the grid anchored, **A2's
memo captures 60.2% of a fifty-day range without restructuring
anything**, where A1c needs `Almanac::between` rebuilt around a shared
search. The memo is the cheaper and safer of the two and now reaches
most of the same work, so **A2 comes first** and A1c follows for what a
memo cannot reach: the arithmetic above the ephemeris, which a cache
does not save.

*What it costs, measured rather than feared.* A1a and A1b both move
where a scan's samples fall, so the crossing instants they refine to are
handed a different bracket and land elsewhere inside the search's
tolerance. Asked how far, for A1a: the Sun's span bounds moved at most
**2.8 milliseconds** and the Moon's **0.04**, against a tolerance of
8.6. Nothing that is published moved at all — every generated page
regenerates identically, and the determinism digest over the fixed
scenario is the same to the bit before and after. So no golden vector
and no hash needed regenerating, and the test that watches this prints
both numbers rather than asserting that nothing happened
(`crates/panchanga/tests/kernel.rs`,
`the_signs_a_day_touches_are_the_same_however_far_the_search_reached`).

One thing that measurement also says: the determinism digest has
sections for the calendar, the astronomy, the houses and the classical
model, and **none for the panchanga**, so it could not have caught a
change here. That is a gap in the matrix rather than a licence, and it
is worth a section of its own before A1c moves the same instants
further.

*Gate:* the almanac and chart rows of
`03-design/batch-and-parallelism-measured.md`, whose counts fall when
each part lands; and a test that the sign spans a day reports are the
same spans, to the tolerance, at either reach.

### A2. A memo across a batch, gated on the provider's own declaration

*The finding:* the repeat share **rises with the batch** — 22.5% within
one chart, 64.7% across fifty. That growth can only come from sharing
between the items, which is what a batch exists to exploit.

*The change:* one memo, scoped to a single call, keyed by (instant, body,
frame). Not a global cache: a batch's memo dies with the batch, so
nothing outlives a request and no consumer is surprised by a stale sky.

*The correctness argument, which is already in the port:*
`Capabilities::deterministic` — "whether identical requests give
identical bits" — is declared by both shipped adapters and **read by
nothing**. The memo is sound exactly when that flag is true, and must
refuse to exist when it is false. The flag finally has its reader, and it
is the right one.

*The knob:* `provider.cache` — `Auto` (memo when the provider declares
determinism), `Off`, `On` (refused with the reason when the provider does
not declare it). Reported in provenance, as everything applied is.

### A3. Parallelism, as a knob, without a pool

*The finding:* the items of a batch are independent — `Founder::value`
takes `&self` and every field of `&self` is `Sync`, because the port
already requires `EphemerisProvider: Send + Sync`.

*The design, and why it is not rayon:* a library must not install a
global thread pool in its consumer's process. `std::thread::scope` costs
no dependency, starts and joins inside the call, and leaves nothing
behind. Chunk boundaries are fixed by index rather than by scheduling, so
**the answers are bit-identical whatever the thread count** — there is no
reduction to reorder, only independent writes into pre-sized columns.
That is what keeps the determinism matrix and the cross-architecture
hashes valid.

*The knob:* `compute.parallelism` — `Serial` (the default), `Auto`
(threads above a measured threshold, serial below it, because a batch of
two loses to the spawn), `Threads(n)`. Reported in provenance. The C ABI
carries it; the three bindings expose it.

*The threshold is measured, not guessed*: the pass gains a row for the
batch size at which threading begins to pay on this machine, and the
default is stated with that number beside it.

### A4. The order, and the number to beat

A chart batch of fifty costs 3 434 calls today. A2 alone removes about
two thirds of them; A1 removes most of what remains; A3 divides what is
left by the cores. The page is regenerated at each step and the number
moves in public.

## Part B — the engine passthrough

**The requirement, in the maintainer's words:** if the ephemeris is
updated later and gains new functions, they must be callable from the SDK
**without the SDK being updated**. Not a port method per operation — a
direct link.

### B1. Why this is achievable rather than aspirational

Teimeris already describes itself. `tools/idl/teimeris.idl` is 13 472
lines of machine-readable JSON, extracted from its own headers by its own
tooling, and it is what every Teimeris binding is generated from. It
carries every function's signature **and a role for every parameter** —
`handle`, `value`, `struct_in`, `scalar_out`, `struct_out`, `array_in`
with its `array_len`, `array_out` with its `array_cap`, `string_in` —
which is the same role vocabulary this SDK's own IDL uses, arrived at
independently.

A role vocabulary is exactly what a generic dispatcher needs. The engine
is not opaque; it is documented in a form a machine can read.

### B2. The shape

Three entry points, and no per-function code anywhere in the SDK.

| entry point | answers |
|---|---|
| `ts_ephemeris_describe` | the engine's manifest, as JSON: every operation, its parameters and their roles |
| `ts_ephemeris_call` | one operation by **name**, arguments as JSON, result as JSON |
| `ts_ephemeris_manifest_version` | what the manifest was extracted from, for the stamp |

The SDK holds no list of engine operations. It holds a *dispatcher* that
reads the manifest the engine ships and marshals by role. A function
added to the engine appears in the engine's manifest and is callable the
same day, with no SDK release — which is the requirement, stated as a
mechanism.

### B3. The one hard part, stated plainly

Calling a C function chosen at runtime needs its ABI, and **doubles do
not travel in the same registers as integers** on any platform the SDK
ships. 82 of Teimeris's functions take doubles by value, so a uniform
word-sized dispatcher is wrong and would be wrong silently.

Two ways, and the plan takes both in order:

1. **Generated dispatch (first).** The adapter's build reads the
   manifest and emits a correctly typed call per function. No runtime
   dependency, full type safety, and it proves the marshalling by role is
   right. New engine functions need an adapter rebuild — no SDK source
   change, but a rebuild.
2. **`libffi` dispatch (then).** A call frame built from the manifest at
   runtime. A consumer drops in a new engine and its manifest and the new
   functions work **with no rebuild at all**, which is the maintainer's
   requirement in full. The dependency belongs to the adapter, never to
   the SDK core, and is behind a cargo feature so a consumer who does not
   want it does not link it.

Shipping (1) first is not a detour: it is how (2) is tested, because both
must produce the same JSON for the same call.

### B4. What a consumer sees

- **Rust:** `context.ephemeris().describe()` and
  `context.ephemeris().call(name, args)`.
- **Node / Python / Dart:** `ctx.ephemeris.native.<name>(…)` through a
  dynamic proxy — a JS `Proxy`, Python `__getattr__`, Dart
  `noSuchMethod` — so a function added to the engine after the binding
  shipped is reachable **without regenerating the binding**. Optional
  typed wrappers can be generated from the manifest for consumers who
  want completions, and are a convenience over the dynamic path rather
  than the only way in.

### B5. What it must not do

- A native result **never enters the completion**. It is the engine's
  answer in the engine's own frame, returned as such. The SDK does not
  pretend it is a port result, does not complete it, and does not stamp
  it as one.
- A native call **is** recorded in provenance, because a result a
  consumer got with one is not reproducible from the settings alone.
- The manifest is the contract: a name not in it is refused, with the
  near matches named, as every other refusal in this SDK is.

## Part C — what the same brief also asks for

### C1. One spelling per name

A name should be spelled in one place. The known live instance:
`Body::key()` hand-writes `"SUN"` while the generated catalogue's
`Graha::Sun` also spells it, and nothing checks that they agree. They are
genuinely different vocabularies — the port's `MEAN_NODE` and
`TRUE_NODE` are both the catalogue's `RAHU` — so the fix is not to merge
them but to **gate the overlap**: every `Body` that maps to a `Graha`
must spell the same key. A `check-names` rule, in the same family as
`knob-has-a-reader`.

### C2. Generated code in other languages

Asked and answered, recorded here so it is not asked twice.
`bindings/python/teistro/__init__.py` is hand-written and is a real
`.py` file. What is generated from Rust is `_ffi.py`, `catalogue.py` and
`_blob.py` — ctypes declarations, 81 enums, blob decoders — and those
must stay generated: hand-writing them guarantees drift, which is what
`check-python` exists to disprove. Moving them to `.py` template files
was considered and rejected on a measurement: the emitters are about
eight lines of logic per line of emitted text, so they are computation
and not templates, and a template engine would cost a dependency and buy
nothing. The generated files are type-checked by mypy in strict mode, so
the emitted code is verified in its own language.

## Sequence

1. **A0** the measurement and this plan — *done*, `check-batching`.
2. **B0** the passthrough measured: what the port reaches of an engine,
   counted rather than asserted — part of the same pass.
3. **A1** size each search by what it searches for (A1a — *done*, 1228
   calls a day to 841), grid the uniform scan (A1b — *done*, 841 to
   665), anchor the grid so a crossing does not depend on the window
   (A1b′ — *done*, 665 to 681, and the repeat share of a fifty-day
   range from 24.7% to 60.2%).
4. **A2** the batch memo and `provider.cache`, which the anchoring moved
   ahead of A1c.
5. **A1c** hoist what a range shares, for the arithmetic a memo cannot
   save.
6. **B1** the manifest, `ts_ephemeris_describe`, generated dispatch,
   Rust surface.
7. **B2** the dynamic proxy in the three bindings.
8. **A3** `compute.parallelism` with the threshold measured.
9. **B3** `libffi` dispatch behind a feature.
10. **C1** the `check-names` rule.

Each step regenerates the measured page, so the numbers move in public
and a regression is a failed gate rather than a memory.
