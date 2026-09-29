# What a batch asks the ephemeris for, measured

Status: `generated` by `cargo xtask batching` over the analytic test
provider. Do not edit: `check-batching` regenerates this page and fails
on any difference. The plan written from it is
[`../07-roadmap/02-plan-performance-and-passthrough.md`](../07-roadmap/02-plan-performance-and-passthrough.md).

## 1. Why calls and not seconds

An ephemeris call is the expensive thing in this library: a file read, a
series evaluation, a lock. How long one takes belongs to the engine and
the machine — Teimeris answers in microseconds and a JPL kernel over a
network file system in milliseconds — but **how many the SDK makes**
belongs to the SDK, is the same everywhere, and is therefore the only
part of the performance a gate can hold. Every number below is a count.
Nothing here is a timing and nothing here would change on another
machine.

The provider underneath is the analytic test one, which answers
instantly. That is deliberate: it makes the counts visible without
making them slow, and the counts are what a real engine multiplies by
its own cost.

## 2. What each operation asks for

| operation | size | calls | cells | distinct cells | widest call | mean width | calls per item |
|---|---:|---:|---:|---:|---:|---:|---:|
| positions | 1 | 1 | 3 | 3 | 3 | 3.00 | 1.0 |
| positions | 2 | 1 | 6 | 6 | 6 | 6.00 | 0.5 |
| positions | 10 | 1 | 30 | 30 | 30 | 30.00 | 0.1 |
| positions | 50 | 1 | 150 | 150 | 150 | 150.00 | 0.0 |
| charts | 1 | 73 | 80 | 62 | 8 | 1.10 | 73.0 |
| charts | 2 | 146 | 160 | 88 | 8 | 1.10 | 73.0 |
| charts | 10 | 730 | 800 | 296 | 8 | 1.10 | 73.0 |
| charts | 50 | 3434 | 3784 | 1336 | 8 | 1.10 | 68.7 |
| almanac | 1 | 251 | 789 | 704 | 66 | 3.14 | 251.0 |
| almanac | 2 | 505 | 1560 | 1078 | 66 | 3.09 | 252.5 |
| almanac | 10 | 2529 | 7819 | 4256 | 66 | 3.09 | 252.9 |
| almanac | 50 | 12 522 | 38 816 | 19 648 | 66 | 3.10 | 250.4 |
| sunrise | 1 | 18 | 18 | 18 | 1 | 1.00 | 18.0 |

Read the first row and the last of each operation together. A
grid of positions is **one call** whether it holds one instant or
50: that is what the port was shaped for, and it is the control
everything else is compared with. A batch of 50 charts is 3434 calls
— 69 per chart, the same as one chart costs on its own — and a
range of 50 almanac days is 12 522, or 250 a day. Neither is a batch
in any sense the ephemeris can see. They are loops that share a
provenance stamp.

## 3. The calls are cells, not grids

The widest call a batch of charts makes is 8 cells — the grahas of one
chart, asked for once — and the widest an almanac makes is 66.
Everything else is one instant and one or two bodies. The mean width is
1.10 for charts and 3.10 for an almanac: the SDK asks its ephemeris for
**one cell at a time**, tens of thousands of times, through a port whose
one required operation takes a grid.

One sunrise is worth naming on its own: 18 calls, every one of them a
single cell, none of them a repeat. Meeus's iteration answers it, and it
is serial by construction — each instant is computed from the sample
before it, so there is no grid to ask for. A day's 250 calls are **not**
made of searches like it: an attribution of every one of them to its
caller (recorded in
[`../07-roadmap/02-plan-performance-and-passthrough.md`](../07-roadmap/02-plan-performance-and-passthrough.md),
A1) found 44% of them in the Moon's sign search, which reaches forty
days either side of the day because the constant that sizes it was
chosen for the Sun. The dominant cost is a window, not a round trip.

## 4. How much of the work is asked for twice

The distinct column counts cells rather than calls: one instant, one
body, one frame, however many times it was asked for. The gap between it
and the cells column is what a memo would have answered without touching
the engine.

| operation | 1 item | largest batch |
|---|---:|---:|
| charts | 22.5% | 64.7% of 50 items |
| almanac | 10.8% | 49.4% of 50 items |

The share **rises with the batch**, which is the finding. Within one
chart 22.5% of the cells are asked for more than once; across 50 charts
it is 64.7%. That growth can only come from sharing *between* the items
— consecutive instants at one place scan overlapping windows for the
same sunrise, and read the same grahas at the same instants — and it
is exactly the sharing a batch exists to exploit and this one does not.

A memo is only sound over a provider that answers identical requests
with identical bits, and the port already asks every provider to say
whether it does: `Capabilities::deterministic`, declared by both shipped
adapters and read by nothing. It has a reader now, and it is the right
one — the cache is correct exactly when that flag is true, and refuses
to exist when it is not.

## 5. What the memo actually saves

[`CachingProvider`](../../crates/port-ephemeris/src/caching.rs) over the
same batches, the cache off and on. Both arms run the same code through
the same types — a cache of nothing is a cache that does nothing —
so what separates the numbers is the memo and nothing else, and the
answers are identical cell for cell.

| days | calls | cells | calls, cached | cells, cached | answered from memory |
|---:|---:|---:|---:|---:|---:|
| 1 | 251 | 789 | 155 | 496 | 12.4% |
| 2 | 505 | 1560 | 211 | 705 | 37.2% |
| 10 | 2529 | 7819 | 694 | 2519 | 55.4% |
| 50 | 12 522 | 38 816 | 2974 | 11 127 | 60.1% |

The share answered from memory **rises with the batch** — 12.4% for a
single day, 60.1% across 50 — which is the same finding as §4 read
from the other side, and the reason the memo is worth more than a cache
of one call's own repeats. A range of 50 days asks the ephemeris for 11
127 cells instead of 38 816, in 2974 calls instead of 12 522.

Of the 19 168 cells the 50-day range asks for again, the memo answers 16
721 with a cell. The other 2447 are **refusals it remembers**: the
completion asks a provider for the frame it wants before completing that
frame from the native one, the provider measured here produces its
canonical frame alone, and a refusal as unsupported is a property of the
request's shape, so the memo keeps it by everything but the instants and
refuses again without asking (10 978 cells refused from memory). This
pass refuses a page on which the remembered refusals cannot account for
every repeat the memo left.

## 6. What is reachable at all

The other half of the same question. A batch that asks well is
still limited to what it may ask for, and the port names 8 operations: `positions` and seven declared
overrides. Teimeris's public header names
177. The difference — its eclipses, its stars
and orbits, its own calendar grids and chart blobs, its scans —
was unreachable through this SDK, and a consumer who wanted any
of it had to open a second handle to the same engine: the largest
dead end in the project.

**The route is built** — the port's `native_manifest` and
`native_call`, and `port_ephemeris::Native` over them. The SDK holds no
list of an engine's operations. It reads the manifest the engine ships
and relays a call by the name that manifest gives, so an operation the
engine gains *after* the SDK ships is callable the day it appears. The
count below is what the measured provider declares, which is why it
moves when an engine does and not when the SDK does; the number for
Teimeris arrives with its adapter's own manifest.

The count of Teimeris's operations is recorded rather than computed: the
engine is not in this workspace, so this pass cannot count it on every
run. It was counted from `core/include/teimeris/*.h` on 2026-09-09 and
is a floor, since the same is true of every other engine an adapter
might wrap.

## 7. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| a batch of positions is one call whatever its size | **holds** | 1 call for 50 instants, 150 cells wide |
| a batch of charts is one call whatever its size | falsified | 3434 calls for 50, 69 per item |
| a batch of almanac is one call whatever its size | falsified | 12 522 calls for 50, 250 per item |
| the calls a batch makes are grids rather than cells | falsified | a chart's calls are 1.10 cells wide on average and an almanac day's 3.10; the widest either makes is 8 and 66 |
| a batch asks for each cell once | falsified | 64.7% of a batch of 50 charts and 49.4% of 50 almanac days are cells already fetched |
| a memo answers a repeated cell without touching the engine | **holds** | 16 721 of the 19 168 cells a range of 50 days asks for again are answered from memory, and the other 2447 are refusals it remembers; 60.1% of all its cells: 11 127 asked of the engine instead of 38 816 |
| a consumer can reach what their engine offers beyond the port | **holds** | the port names 8 and an engine's own manifest is read through it; the provider measured here declares 2 beyond them |

four of the seven claims are falsified, and they are falsified in an
order that matters. Threads would multiply the work rather than reduce
it while more than half of a range's cells are asked for twice; the
design fixes the arithmetic first and spends hardware last, and the
numbers on this page move as each step of it lands
([`../07-roadmap/02-plan-performance-and-passthrough.md`](../07-roadmap/02-plan-performance-and-passthrough.md)).

