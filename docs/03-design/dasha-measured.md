# The Vimshottari dasha, measured

Status: `generated` by `cargo xtask dashas` over the conformance
corpus's `dashas` section, 2026-09-15. Do not edit: `check-dashas`
regenerates this page and fails on any difference. The design it
measures is [`dasha-kernels.md`](dasha-kernels.md).

The corpus records 93 Vimshottari dashas: 55 charts under the default
profile and the rest under the variant profiles that change the Moon
(other ayanamshas, a geocentric Moon, the Surya Siddhanta, a temporal
balance). 148 method answers in all, each with its inputs, its balance,
its period tree (spatial to depth 2, spatial to depth 3, temporal to
depth 2), the children at sampled deeper paths, and the chain of periods
running at two instants. So every rule below is decided rather than
argued.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| the seed is the nakshatra the Moon's sidereal longitude falls in, counted from Ashwini | **holds** | 0 of 93 disagree |
| the first lord is the sequence's, Ketu 7 · Venus 20 · Sun 6 · Moon 10 · Mars 7 · Rahu 18 · Jupiter 16 · Saturn 19 · Mercury 17, indexed by the seed modulo nine | **holds** | 0 of 93 disagree |
| the cycle is 120 years, the sum of the nine | **holds** | 0 of 93 disagree |
| the elapsed fraction is the Moon's longitude into its nakshatra over the nakshatra's width | **holds** | worst 3.2e-15 |
| spatially, what remains is one less the elapsed fraction | **holds** | worst 0.0e0 |
| temporally, what remains is the time from birth to the Moon leaving its nakshatra over the time the Moon spends in it | **holds** | worst 1.1e-16 over 65 |
| the balance is what remains times the first lord's years times the year length | **holds** | worst 9.1e-13 days over 148 |
| the balance is written as whole years of the year length, whole months of a twelfth of it and whole days, the rest **rounded** to the minute | **holds** | 0 of 148 disagree |
| the same with the rest **floored** to the minute | falsified | 73 of 148 disagree |
| the same with months of thirty days | falsified | 145 of 148 disagree |
| the mahadashas run from birth for the balance, then each lord's whole years in sequence; every period's children run from its own lord, each its share of **the period it is in**, the birth period's too | **holds** | 0 of 53415 disagree; every bound within 2.8e-9 days (0.241 ms) |
| the birth period's children are sized against the **whole** period, which began before birth, the ones already over dropped | falsified | 148 of 148 disagree |
| a boundary reproduces the recorded double to the bit | falsified | 21 587 of 53 415 rows bit-identical adding lengths in turn, 35 442 adding shares so far; worst 2.8e-9 and 9.3e-10 days |
| the children the corpus samples at deeper paths are the same rule's, to the fifth level | **holds** | 0 of 14841 disagree |
| the periods running at an instant are found by descending the same tree, five levels deep | **holds** | 0 of 296 disagree; 18 of the instants fall past the ninth mahadasha, where the corpus records no period at all and the cycle does not begin again |
| a year of 365.25 days | **holds** | every one of the 93 records uses it |
| any other year length (360, sidereal, tropical, lunar, 324) | untested | no record uses one, so crux C6 is not settled here |

## What it means for the module

**The birth period's sub-periods are compressed.** Each child is its
share of the period it is in, and the birth period is only as long as
its balance, so its antardashas are that much shorter. The other
reading, sizing them against the whole period with the elapsed ones
dropped, is refused by every record. Both readings are taught: a
tutorial works the example the compressed way, and another says the
sub-periods are sized against the full period. Neither is a rank-1 text,
so the module ships both as a knob, `dasha.birth_period`, with the
corpus's reading as the default, and the question as crux C48.

**The cycle ends.** Past the ninth mahadasha the recording engine
answers no period rather than beginning the sequence again, so a chart
older than its cycle has no running dasha. That is a choice rather than
a law, and the module makes it a knob, `dasha.after_cycle`, defaulting
to the corpus's.

**Boundaries are within a quarter of a millisecond, and not to the
bit.** No order of float arithmetic reproduces the recorded doubles, and
the corpus's tolerance for a boundary is a thousandth of a day. So the
module computes a boundary as its period's start plus an exact rational
share of its length, which is the design page's `Ratio` arithmetic, and
the golden comparison is to the tolerance.

**The balance is written with its minutes rounded.** Floored minutes
disagree with half the records and thirty-day months with nearly all.

**The year length is not settled.** Every record uses 365.25 days, so
the other lengths the settings offer stay a knob with no measurement
behind any of them, which is crux C6 as it stood.