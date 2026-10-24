# Property coverage: each invariant held, by name, at its boundaries

Status: `implemented` in part, 2026-10-25: steps 1, 2 and 4 of §4; step 3, the fixed points, is open. The quality bar's "property tests" row
([`05-testing/01-quality-bar.md`](../05-testing/01-quality-bar.md)) lists
seven invariants and asks that generators "deliberately produce values at
and one microarcsecond either side of every classification boundary". Its
status says the list "is not covered crate by crate, and no gate requires
a boundary generator". This page decides what holds each invariant, and
how a gate keeps it held.

## 1. What holds each invariant now

An invariant can be held two ways. A test can **sweep** its whole space,
which beats sampling wherever the space is enumerable (a varga's 12 signs
by N parts, a calendar's every day). Or it can **sample** with `proptest`
where the space is not enumerable (a longitude, an instant). Either counts.
What does not count is an invariant held at a handful of hand-picked
values.

| invariant | held by | how | at the boundaries |
|---|---|---|---|
| exact partition of the circle, every divisor | `core/src/angle.rs`, `every_divisor_partitions_the_circle` | sampled, 1 to 360 parts | `a_value_just_below_a_boundary_stays_below` |
| exact partition, every varga | `vargas/tests/exhaustive.rs`, invariants 1 and 8 | swept, every cell of every chart and every D-N | invariant 8: a boundary is decided |
| sums | `strength`: the Ashtakavarga's classical totals, the Shadbala's six, the Bhava bala's components | swept over the corpus | not a boundary property |
| tree consistency, and `dasha_at` finds every instant | `dasha/tests/timeline.rs`, one contract for every kernel | swept, every period's children over three cycles, and the last bit before every start | yes, to the bit |
| monotonic transitions | `astro/src/events.rs`, `calendar/src/mixed.rs`, `muhurta/tests/season.rs` | swept over their ranges | events at their boundaries |
| serialise-deserialise fixed points | `rules`: a rule and a condition; `sdk/src/varsha.rs`; the envelope | a few types | — |
| calendar round trips | `calendar/tests/range.rs` and every calendar's own `every_day_round_trips` | swept, every day of its range | its first and last day |

Two gaps are real:

- **Fixed points.** The SDK has 76 request and record types that read
  JSON (`derive(Deserialize)` in `crates/sdk/src`), and three of them are
  held to reading back what they write. A request a consumer stores and
  sends again (a saved chart request, an MCP tool's arguments) must come
  back as itself.
- **The boundary generator.** Every classification the SDK makes from a
  longitude (sign, nakshatra, pada, a varga's part, the tithi's and the
  yoga's twelfths and thirteen-twentieths) has a boundary, and only the
  core angle and the vargas test theirs. No test reaches the others one
  microarcsecond either side, so a `<` that should be `<=` there would
  pass.

## 2. The boundary generator

`teistro_core::angle::edges(divisions)` answers every boundary of a
circle cut into `divisions` equal parts, each at the boundary and one
microarcsecond (1,000 of the crate's nanoarcsecond units) either side:
`3 × divisions` values, in order. It is a plain iterator over `Nas`, with
no `proptest` in core's dependencies, so a sweep walks it whole and a
property test draws from it with `prop::sample::select`. The
classifications it serves, each with its part count: sign 12, nakshatra
27, pada 108, tithi 30 (over the elongation), yoga 27 (over the sum), and
each varga's own part count.

The property each classifier is then held to is the one the core test
already states for a single cut: a value just below a boundary is in the
part below it, the boundary and just above are in the part above, and the
part's index is the integer arithmetic of `Nas`, never a float division.

## 3. The register and the gate

`xtask/src/properties.rs` lists each invariant once, with what holds it:
`(invariant, file, test fn, Sweep | Sample | Example, the boundary
generator it calls)`. A generator is the function the test reaches the
boundaries with: `edges`, or the vargas' `first_nas_of` for unequal
spans, or the dashas' `below`, the last bit before a period's start. An
invariant held only by `Example` tests is listed as open, with why.
`check-properties` (fast check) fails:

- an invariant of the quality bar's row with no entry;
- an entry whose file has no `fn <name>(`, so a renamed or deleted test
  is not silently a hole;
- an entry naming a boundary generator its file does not call;
- an invariant held only at examples and not listed as open, and one
  listed as open that a sweep or a sample now holds.

The row's list and the register are the same list. The quality bar's
row is the prose of the register, and the register is held to the row's
seven names, as `check-micro` holds its budgets to the performance page.

## 4. Order of work

1. `edges`, with its own test: three values a boundary, in order,
   a microarcsecond apart.
2. The classifiers held at their edges: sign, nakshatra and pada in
   `core`, tithi and yoga in `panchanga`, each varga's part in `vargas`.
3. Fixed points: one test in `crates/sdk/tests` reading every request
   type from the JSON its `Default`, or its documented example where it
   has no default, writes, and writing it back byte for byte. The type
   list comes from the request types the MCP server takes and the
   document schema names, so a type added to either joins the test.
4. The register, `check-properties`, and the quality bar's row.
