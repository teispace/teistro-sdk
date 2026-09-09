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
| almanac | 1 | 1228 | 1456 | 1241 | 2 | 1.19 | 1228.0 |
| almanac | 2 | 2427 | 2881 | 2268 | 2 | 1.19 | 1213.5 |
| almanac | 10 | 12 270 | 14 617 | 10 713 | 2 | 1.19 | 1227.0 |
| almanac | 50 | 60 631 | 71 853 | 51 456 | 2 | 1.19 | 1212.6 |
| sunrise | 1 | 18 | 18 | 18 | 1 | 1.00 | 18.0 |

Read the first row and the last of each operation together. A
grid of positions is **one call** whether it holds one instant or
50: that is what the port was shaped for, and it is the control
everything else is compared with. A batch of 50 charts is 3434 calls
— 69 per chart, the same as one chart costs on its own — and a
range of 50 almanac days is 60 631, or 1213 a day. Neither is a batch
in any sense the ephemeris can see. They are loops that share a
provenance stamp.

## 3. The calls are cells, not grids

The widest call a batch of charts makes is 8 cells — the grahas of one
chart, asked for once — and the widest an almanac makes is 2.
Everything else is one instant and one or two bodies. The mean width is
1.10 for charts and 1.19 for an almanac: the SDK asks its ephemeris for
**one cell at a time**, tens of thousands of times, through a port whose
one required operation takes a grid.

One sunrise is where it comes from and is worth naming on its own:
18 calls, every one of them a single cell, none of them a repeat.
The horizon solver is thrifty in **count** — it brackets and
bisects rather than scanning blindly — and pathological in
**width**, because each step of the bracket is a separate round
trip where the whole bracket is known in advance and could be one.
A day's limbs are made of searches like it, which is what turns 18
calls into 1213 a day.

## 4. How much of the work is asked for twice

The distinct column counts cells rather than calls: one instant, one
body, one frame, however many times it was asked for. The gap between it
and the cells column is what a memo would have answered without touching
the engine.

| operation | 1 item | largest batch |
|---|---:|---:|
| charts | 22.5% | 64.7% of 50 items |
| almanac | 14.8% | 28.4% of 50 items |

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

## 5. What is reachable at all

The other half of the same question. A batch that asks well is
still limited to what it may ask for, and the port names 8 operations: `positions` and seven declared
overrides. Teimeris's public header names
177. The difference — its eclipses, its stars
and orbits, its own calendar grids and chart blobs, its scans —
is unreachable through this SDK, so a consumer who wants any of it
must open a second handle to the same engine and manage it
themselves. That is a dead end of the kind
`no-dead-ends` forbids, and it is the largest one in the project.

The count of Teimeris's operations is recorded rather than computed: the
engine is not in this workspace, so this pass cannot count it on every
run. It was counted from `core/include/teimeris/*.h` on 2026-09-09 and
is a floor, since the same is true of every other engine an adapter
might wrap.

## 6. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| a batch of positions is one call whatever its size | **holds** | 1 call for 50 instants, 150 cells wide |
| a batch of charts is one call whatever its size | falsified | 3434 calls for 50, 69 per item |
| a batch of almanac is one call whatever its size | falsified | 60 631 calls for 50, 1213 per item |
| the calls a batch makes are grids rather than cells | falsified | a chart's calls are 1.10 cells wide on average and an almanac day's 1.19; the widest either makes is 8 and 2 |
| a batch asks for each cell once | falsified | 64.7% of a batch of 50 charts and 28.4% of 50 almanac days are cells already fetched |
| a consumer can reach what their engine offers beyond the port | falsified | the port names 8; Teimeris names 177; 0 of the difference is reachable through the SDK |

five of the six claims are falsified, and they are falsified in an order
that matters. Threads would multiply the work rather than reduce it
while two thirds of a chart batch's calls are repeats and seventeen of
every eighteen round trips are avoidable width; the design that follows
fixes the arithmetic first and spends hardware last.

