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

**A1d. Grid the horizon scan — built, and the attribution found it.**
With the memo in place I re-attributed a fifty-day range's remaining
21 663 calls before designing A1c, and A1c was not what they were:

| calls | share | what asks |
|---:|---:|---|
| 14 032 | 65% | `rise_set::Solver::scan` — the fallback scan |
| 4 275 | 20% | `Solver::iterate` — Meeus's iteration |
| 1 902 | 9% | the lattice searches' refinements |
| 842 | 4% | `find_sankranti` |

**Eighty-five per cent of everything left was the horizon solver, and
two thirds of it was a scan that A1's first draft had recorded as not
running at a temperate latitude.** It runs, and this is why:
`almanac::events` collects every rise in a window by searching from the
last one it found, and the search that ends the loop has no event to
find. Proving that costs a walk of the whole remaining window at
ten-minute steps — a hundred and forty-four round trips — and it happens
**twice a day, every day**, once for the rises and once for the sets.

*The change.* `solve::first_zero_gridded` beside `first_zero`: the scan
produces its instants by the walk's own recurrence and asks for them a
chunk at a time, while the narrowing stays serial. `ApparentPositions`
grows `apparent_many` beside `apparent`, defaulting to the walk and
overridden by `Completion` with one `positions` request, exactly as
`Longitudes` did for the lattice searches. `Solver::with_chunk` names the
chunk and `solve::SCAN_CHUNK` is 32.

Thirty-two rather than the whole window because a bracket scan **stops
at the first sign change**, unlike a lattice search which visits every
instant between its ends. So a chunk can be asked for and not used, and
the waste is at most one chunk less one. Thirty-two is sized from what
the scan is for: a rise the iteration could not settle is found within a
few steps of where it left off, and an absence — the case that dominates
— walks the whole window, where thirty-two turns a hundred and
forty-four round trips into five.

*Measured.* An almanac day fell from **681 calls to 395** and a fifty-day
range from 33 270 to **19 632**; with the memo, a day is **333** and
fifty days **8 174**. The cells are unchanged, because the same readings
are being asked for in fewer requests.

*Bit-identical*, and the test says so with `to_bits()` over the grazing
star at 69.6°N where the scan is reached, across four event kinds and
five chunk sizes with a chunk of one as the walk it replaced. The one
thing that does differ is the **count of readings taken**, which may
exceed the walk's by up to a chunk less one, because a chunk can carry
instants past the one that brackets the crossing; the test bounds it
rather than pretending otherwise, and no output carries that count.

**A1c. Hoist what a range shares — closed, measured away three times.**
A1c was to have a range compute its shared searches once. Three
measurements have taken its ground in turn.

The anchored grid made neighbouring days ask for the *same* instants and
the memo then answered them without asking twice, so a fifty-day range
fetches 23 888 cells against a union of 21 446 — **within 11% of the
least it could be**. That removed the ephemeris case for it.

Then the arithmetic case was measured directly: fifty days as one range
against fifty days asked one at a time, over the analytic provider so
that time is arithmetic rather than engine. The range shares **51% of
the calls and 8.5% of the time** — which says the memo is doing the
sharing and the per-day arithmetic is not shared. That looked like A1c's
opening.

Attributing the range's remaining 8 174 calls closed it:

| calls | share | what asks |
|---:|---:|---|
| 2 259 | 27.6% | the Moon's rise and set, `almanac::events` |
| 2 016 | 24.7% | the day's own rise and set, `Solver::day` |
| 1 902 | 23.2% | the lattice searches' refinements |
| 842 | 10.3% | `find_sankranti` |
| 543 | 6.6% | the horizon scan — 543 calls carrying **14 032 cells** |
| 150 | 1.8% | the lattice searches' scans |

**Over half of it is the horizon solver's Meeus iteration**, and that is
not shared work: every day genuinely has a different sunrise, and each
iteration is serial because each instant is computed from the sample
before it. The scans a hoist would share are down to 8% of the calls.
There is no longer a case for restructuring `Almanac::between`, so it is
closed rather than deferred.

*What the same attribution does point at* is the last row's shape: 543
calls carrying **14 032 cells**, which is the scan walking a whole window
at ten-minute steps to prove an event is **absent**. `almanac::events`
ends its loop by searching for a rise that is not there, twice a day.
Proving an absence by walking is the expensive way to do it: for a body
whose altitude cannot reach the target anywhere in the window, the
declination and the latitude say so in constant time. That is astronomy
rather than plumbing and wants its own measured page before any of it is
written — the bound has to hold for the Moon, whose declination moves
five degrees in a day.

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

One thing that measurement also said: the determinism digest had
sections for the calendar, the astronomy, the houses and the classical
model, and **none for the panchanga**, so it could not have caught a
change here. Three changes in a row moved where a scan puts its samples
and the digest was identical to the bit each time, which said nothing
about the layer where they moved. **That gap is now closed**:
`teistro-scenario` has a `panchanga` section — ten almanac days over the
analytic provider, hashing every limb boundary, sunrise and moonrise a
search produces, 374 values for ten milliseconds. It sees what the
others cannot: moving `SCAN_ANCHOR_JD` by half a step leaves the `astro`
digest unchanged and moves this one.

*Gate:* the almanac and chart rows of
`03-design/batch-and-parallelism-measured.md`, whose counts fall when
each part lands; and a test that the sign spans a day reports are the
same spans, to the tolerance, at either reach.

### A2. A memo across a batch, gated on the provider's own declaration

*Built as the mechanism; the settings knob follows.*

**What it is.** `port_ephemeris::CachingProvider<P>` wraps any provider,
including a borrowed one, and answers a cell it has already been asked
for from memory. A request is not all-or-nothing: the cells it already
holds are kept, and it asks the provider for **the instants that lack a
cell by the bodies missing at any of them** — one grid however scattered
the gaps are, over-asking only where a body was already known at an
instant another body was not. That is the shape a batch actually makes,
now that the scan's samples are anchored: consecutive days ask for the
same instants and differ at the ends.

**What makes it sound.** A memo answers a repeat with the first answer,
so it is right exactly when the provider would have answered the same
again. The port has always asked every provider to declare that —
`Capabilities::deterministic` — and nothing read it. This reads it: a
provider that does not declare it is wrapped but **not cached**, and
`caching()` says which happened rather than leaving a caller to assume.

**What it must not change.** The capabilities it reports are the inner
provider's, unchanged, name and all: they reach the provenance stamp of
every value the SDK produces, and the point of a cache is that nothing
downstream can tell it is there.

**What it costs.** One `BTreeMap` behind one lock, bounded by a capacity
the caller chooses (65 536 cells by default, about six megabytes). Past
the bound it stops admitting and keeps serving what it holds, so memory
is a number someone chose rather than a function of how long the batch
ran. A `BTreeMap` rather than a `HashMap` because the determinism lints
forbid unordered iteration in a computation crate and a cache that
iterates in a defined order needs no exception.

*Measured*, on the gated page, both arms through the same types because
a cache of nothing is a cache that does nothing:

| days | calls | cached | cells | cached | from memory |
|---:|---:|---:|---:|---:|---:|
| 1 | 681 | 616 | 1092 | 906 | 17.0% |
| 50 | 33 270 | **21 663** | 53 935 | **23 888** | **55.7%** |

The share answered from memory rises with the batch, which is §4's
finding read from the other side.

**The knob — built.** A Rust consumer wraps their own provider. A
binding consumer cannot: they hand the SDK a vtable and the SDK owns the
box, so nothing but the boundary can put a cache under it.
`provider.cache_cells` is read in `TsContext::build`, which is that
place. One knob and not two — nought is off, any other number is the
capacity — so no pair of settings can disagree about whether the memo
exists. `DEFAULT_CACHE_CELLS` is declared in `core::settings`, beside
the knob a consumer sets, and the port's own default *is* that constant:
one number, one place.

Two things had to exist for it. `EphemerisProvider` is now implemented
for `Box<P>` as it already was for `&P`, without which nothing could
wrap what a binding consumer hands the SDK — a cache, a counter or an
adapter of their own would have been reachable from Rust and from
nowhere else. And the memo reports the inner provider's capabilities
unchanged, so a context that wraps and a context that does not stamp
their values identically.

### A3. Parallelism — closed by measurement, and not built

*Two measurements, taken before any of it was written, closed it.*

**First: how much of a batch a thread could overlap.** A
`teimeris::Context` is `Send` and deliberately **not** `Sync`, and the
engine says why: *"N contexts may be used concurrently from N threads
with no locking — which is a statement about N contexts, not about one
shared between them. A context mutates its caches on every call."* So
the adapter holds `Mutex<Context>` and every ephemeris call queues.
Measured over a year of the Moon's sign ingresses through the real
engine, **46–48% of the work is inside that lock** and 52–54% is the
SDK's own arithmetic
(`adapters/ephemeris-teimeris/rust/tests/parallelism_probe.rs`).

Threads over one provider overlap only the second: **1.7× at four
threads, 2.2× at any number of cores.** That is the ceiling of the
design this step assumed.

**Second: what the way out costs.** The engine's own rule points at a
provider per thread, so the question became what a second context costs
(`tests/context_cost.rs`):

| | |
|---|---:|
| opening one engine context | **43.8 ms** |
| one `positions` call, three cells | 2.25 µs |
| **a context is worth** | **19 491 calls** |
| the largest batch this project measures | **8 174 calls** |

A fifty-day almanac with the memo makes 8 174 provider calls in total.
**A thread that opened its own context would pay more for it than the
whole batch costs** — and four threads would pay three of them, 131 ms,
to save at most half of a batch that runs in about 160. Provider per
thread loses for any single batch, and it is not close.

**So the step is closed rather than blocked, and nothing is built.** The
shape that does win is a pool of contexts that outlives many batches,
and that is the **consumer's** to keep rather than the SDK's to make —
and it needs nothing from the SDK, because it already works: build N
providers, build N almanacs or founders on them, and run them on N
threads. The SDK is `Send + Sync` throughout and has never stood in the
way of that.

What the SDK would add by threading a *single* batch internally is at
most 2.2×, at the cost of a knob, a thread pool's worth of lifetime
questions, and a memo that is either per-thread or contended — and the
counts on the measured page would stop being deterministic the moment a
shared memo is read from more than one thread, which would cost the
project a gate to buy a fraction of a factor. That is a bad trade and
the numbers say so.

*If this is ever revisited*, the test asserts the threshold: it fails if
a context ever becomes cheap enough to be worth fewer calls than the
largest measured batch.

## Part B — the engine passthrough

**The requirement, in the maintainer's words:** if the ephemeris is
updated later and gains new functions, they must be callable from the SDK
**without the SDK being updated**. Not a port method per operation — a
direct link.

### B1. The route, built

*The SDK's side is done; an engine's side is its adapter's.*

**What crosses.** Two methods on the port, both optional overrides like
every other: `native_manifest()` answers with the manifest the engine
ships, as the engine wrote it, and `native_call(function,
arguments_json)` relays a call by the name that manifest gives.
`Capabilities::native` declares that they are there, so a caller asks
once rather than finding out by a failing call — carried across the C
boundary in **one of the two bytes `CapabilitiesC` had reserved**, so
the struct's size and every offset in it are unchanged and an adapter
built against the old header still binds.

**What the SDK knows about an engine: nothing.** It holds no list of
operations. `port_ephemeris::Native` reads the manifest into
`NativeManifest`/`NativeFunction`/`NativeParam`/`Role` and relays; a
function that appears in an engine's manifest is callable the day it
appears, with no SDK release. That is the maintainer's requirement
stated exactly, and it is why the manifest is the engine's document
rather than the SDK's.

**`Role::Other` is the requirement at the level of the vocabulary.** An
engine that gains a role this SDK has never heard of must still describe
itself to a consumer who can read it. One unknown word must not take a
whole manifest down with it, so an unrecognised role is kept as the
engine spelled it, and is treated as a caller's to supply — so the
function stays callable rather than becoming unreachable.

**Why JSON.** Not for speed: it is the path for what the port does *not*
cover, so it is never the hot loop, and everything the SDK does in anger
goes through the eight typed operations. The alternative is a binary
encoding the SDK would have to understand, which is exactly the
knowledge that must not live here for the requirement above to hold.

**Where the marshalling lives, and why not here.** In the adapter,
generated from the engine's own manifest at build time. That is the
safety argument, not a convenience: 82 of Teimeris's parameters are
doubles passed by value across 54 of its 161 functions, and eight more
return one. Doubles do not travel in the same registers as integers on
any platform this SDK ships to, so a dispatcher that treated every
argument as a machine word would read the right number of bytes from the
wrong register and be wrong **silently**. A generated dispatcher writes
a real, typed C call per function and cannot make that mistake. The
SDK's side never sees a register.

**Held by tests without an engine present.** The test provider ships a
two-function manifest — one input role, one array-and-out-role — so the
whole path runs with nothing installed: the manifest parses, the
supplied parameters are the in roles and not the out ones, a call
answers, an unknown function is refused by name, a manifest that is not
JSON is a refusal and not a panic, and an unknown role parses and stays
callable. And because the cache, the counter, a borrow and a box all
stand between a consumer and their engine, each forwards the two
methods, with a test that says so: a wrapper that swallowed them would
quietly close the door this opens.

*What remains for the engine:* the Teimeris adapter generates its
dispatcher from `tools/idl/teimeris.idl` and answers these two methods.
The count on the measured page moves when it does.

### B2. The two entry points, and a proxy in each language

**The boundary — built.** `ts_ephemeris_manifest` answers with the
manifest the context's engine ships, and `ts_ephemeris_call(function,
arguments_json)` relays a call by the name that manifest gives. Two
entry points and no more, because the SDK holds no list of an engine's
operations: neither signature mentions any engine, which is what lets an
operation added after this library ships be callable through it.

`CAPABILITY` when the context has no ephemeris, `UNSUPPORTED` when the
engine describes nothing of its own — naming the engine, so the message
says which one — and `PROVIDER` when the engine itself refuses.

*What the architecture paid back.* Adding one module to the boundary put
both functions into the C header, the Dart `dart:ffi` declarations, the
Python `ctypes` declarations, the Node napi glue and the generated
reference **without any of them being edited**. One description,
generated bindings.

*And a hole it found on the way.* The description is extracted from a
hand-written list of sources (`teistro_idl::sdk::SOURCES`). A module of
the `ffi` crate that holds entry points and is not on that list compiles,
exports its symbols, and **no binding has ever heard of it** — the first
version of this change was exactly that, and nothing said so. Nothing
compared the list against the crate, which is `gate-has-a-runner` a
third time, so `boundary-is-described` now does: every `ffi` module with
a `#[unsafe(no_mangle)]` entry point is on the list, or says at the top
of itself why not.

**The proxy — built, and not identically in all three.** Each binding
has a hand-written ergonomic layer over its generated declarations, and
that is where the two entry points become an engine a consumer can use.
`context.ephemeris` answers with one in every language, and each asks
the engine for its manifest at that moment rather than at the first
call, so a context that cannot offer this says so where a caller can act
on it.

| binding | how an operation is reached |
|---|---|
| Python | `engine.tp_echo(value=6.0)` — `__getattr__`, with `__dir__` so a REPL completes the names |
| Node | `engine.tp_echo({ value: 6 })` — a `Proxy`, with `has` and `ownKeys` so `in`, `Object.keys` and a debugger see them |
| Dart | `engine('tp_echo', {'value': 6.0})` — by name |

**Dart is deliberately different.** Dart spells its members in camel case
and an engine spells its functions as C does, so a member proxy would
have to guess the mapping between `tm_eclipse_when` and `tmEclipseWhen`
— and a guess that is wrong for one engine's spelling would make that
operation unreachable, which is the dead end this whole route exists to
close. Python and Node reach them as members because in those languages
the engine's own spelling *is* the idiomatic one. Uniformity across the
three would have cost reach in one of them, and reach is the point.

**None of the three holds a list.** Python's `names` and Node's `names`
read the manifest; Node's TypeScript declaration is an index signature
rather than a set of methods, precisely because writing the names down
would be a promise the next engine release breaks. A binding that
shipped a list would undo the requirement, so none may.

*Held by each binding's own suite*, all three green: the engine names
its own operations, an operation is called by the name the engine gives
it, a name the engine does not have is absent rather than a function
that fails when called, the manifest carries the role of every
parameter, the engine's own refusal comes back with its words, and a
context without an ephemeris says so.

One thing the Node proxy had to get right: `Engine` keeps its context in
a private field, so reading a member with the *proxy* as the receiver
cannot see it. The trap reads with the target as receiver and binds
methods to it. The binding's own tests caught that, which is what they
are for.

### B3. The one hard part, stated plainly

Calling a C function chosen at runtime needs its ABI, and **doubles do
not travel in the same registers as integers** on any platform the SDK
ships. 82 of Teimeris's parameters are doubles passed by value, across
54 of its 161 functions, and eight more return one, so a uniform
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

### C1. One spelling per name — built

*The known instance, gated.* `Body::key` writes `"SUN"` and the
generated catalogue's `Graha::key` writes it too, and nothing checked
that they still matched. They are **not merged**, because they are
genuinely different vocabularies: a `Body` is something an ephemeris
computes and a `Graha` is something a chart reads, and the port has two
nodes where the catalogue has one Rahu. What is gated is the overlap —
ten names — and the pairing is the one `Body::graha` **already owns**
rather than a third list to keep in step. Two tests: every body that is
a graha spells it the catalogue's way, and the exceptions are exactly
the two nodes (both Rahu, so the port must keep its own spelling to say
which) and the two apogees (not grahas), with Ketu the one graha no body
is because it is derived rather than computed. A new body or a new graha
cannot quietly join them.

*And the audit the same brief asked for.* Looking for other names spelled
twice turned up one thing worth having: the port and the catalogue each
hold a **`Direction`** — a crossing's in one, a compass point in the
other. Every binding is generated from the API description and emits one
declaration per item into **one namespace**, so two items of a name
collide there: a compile error in Dart and TypeScript, and in Python the
later one silently wins and the earlier becomes unreachable.

The extractor already refused that for enums, structs, opaques,
callbacks and functions — and nothing said so, so now two tests do. It
did **not** cover blobs, which is one word in the chain that already
existed. Worth recording how that went: the first version of this change
added a *second* duplicate-name checker beside the one already there,
which is the very fault the step is about. Reading the existing check
before writing a new one is the whole lesson.

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
4. **A2** the batch memo and its `provider.cache_cells` knob — *done*, a
   fifty-day range 33 270 calls to 21 663, 55.7% answered from memory;
   the anchoring moved it ahead of A1c.
5. **A1d** grid the horizon scan — *done*, a fifty-day range 21 663
   calls to 8 174, which the attribution found rather than the plan.
6. **A1c** hoist what a range shares — *closed*: over half of what is
   left is the horizon solver's per-day iteration, which is not shared
   work, and the scans a hoist would share are 8% of the calls.
7. **B1** the manifest, the port's two methods and the Rust surface —
   *done*; the adapter's generated dispatch is the engine's side.
8. **B2** the two entry points and the proxy in all three bindings —
   *done*.
9. **A3** parallelism — *closed, not built*: threads over one provider
   cap at 2.2×, and a provider per thread costs 43.8 ms against a batch
   of 8 174 calls, so a pool that outlives many batches is the shape and
   it is the consumer's, which the SDK already allows.
10. **B3** `libffi` dispatch behind a feature.
11. **C1** one spelling per name — *done*.

Each step regenerates the measured page, so the numbers move in public
and a regression is a failed gate rather than a memory.
