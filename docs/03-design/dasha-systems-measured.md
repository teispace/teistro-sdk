# The other nakshatra-seeded dashas, measured

Status: `generated` by `cargo xtask dasha-systems` over the conformance
corpus's `baseline/dasha-systems`, 2026-09-15. Do not edit:
`check-dasha-systems` regenerates this page and fails on any difference.
The design it measures is [`dasha-kernels.md`](dasha-kernels.md);
Vimshottari's own page is [`dasha-measured.md`](dasha-measured.md).

The corpus records 8 systems beside Vimshottari, each for the same
fixtures and balance methods Vimshottari is recorded under: 1184
answers, each with its first lord, its balance, its tree to antardashas
and the chain of four levels at two instants. They were computed from
each fixture's recorded Moon and nakshatra span, so what they test is
the dasha and not the astronomy. The Moon stands in 23 of the 27
nakshatras across them, so the seat rules are untested at Ashwini,
Mrigashira, Uttara Phalguni, Dhanishta.

## Each system

The seat is the rule data the corpus states. **Standing** is how many
seat rules at the same window survive every recorded first lord when
every reference nakshatra, both directions and every offset are tried:
more than one means the corpus cannot tell them apart, and all of them
are the same map on the seeds it records.

| system | lords | seat | standing | first lords | balance, worst days | tree rows | worst boundary, days | chains |
|---|---|---|---|---|---|---|---|---|
| Ashtottari | 8 × 108 years | from Ardra, window 3, offset 0 | 1 | all 148 | 1.1e-11 | all 10 656 | 9.3e-10 | all 296 (18 past the end) |
| Yogini | 8 × 36 years | from Ashwini, window 1, offset 3 | 2 | all 148 | 9.2e-12 | all 10 656 | 9.3e-10 | all 296 (70 past the end) |
| Dwadashottari | 8 × 112 years | to Revati, window 1, offset 0 | 2 | all 148 | 2.0e-11 | all 10 656 | 1.4e-9 | all 296 (18 past the end) |
| Panchottari | 7 × 105 years | from Anuradha, window 1, offset 0 | 1 | all 148 | 2.3e-11 | all 8288 | 1.4e-9 | all 296 (18 past the end) |
| Shatabdika | 7 × 100 years | from Revati, window 1, offset 0 | 1 | all 148 | 2.6e-11 | all 8288 | 9.3e-10 | all 296 (18 past the end) |
| Chaturashiti-Sama | 7 × 84 years | from Swati, window 1, offset 0 | 1 | all 148 | 1.8e-11 | all 8288 | 1.9e-9 | all 296 (24 past the end) |
| Dwisaptati-Sama | 8 × 72 years | from Mula, window 1, offset 0 | 1 | all 148 | 1.4e-11 | all 10 656 | 1.9e-9 | all 296 (26 past the end) |
| Tribhagi | 9 × 120 years, × 2/3 twice round | from Ashwini, window 1, offset 0 | 27 | all 148 | 1.8e-11 | all 26 640 | 1.4e-9 | all 296 (10 past the end) |

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| the first lord is the count from the reference nakshatra to the seed (or from the seed to it), in windows of the row's span, plus its offset, taken round the lords | **holds** | 0 of 1184 disagree |
| a seed past the windows of a row whose lords cover several nakshatras each goes round to the first lord's window, keeping its place in it | **holds** | 0 of 8 disagree; at Krittika and Rohini |
| a temporal balance over a window of nakshatras counts the time elapsed in the Moon's own nakshatra as that nakshatra's part of the window, the whole nakshatras before it in the window already gone | **holds** | worst 1.8e-12 days over 65 Ashtottari answers |
| the same with the window's remainder taken from the nakshatra's time fraction alone | falsified | 65 of 65 disagree |
| the time fraction of the Moon's stay across the whole window | untested | the corpus records the span of one nakshatra, never of a window |
| every system writes its balance as Vimshottari does, the minutes rounded | **holds** | 0 of 1184 disagree |
| every system's tree is Vimshottari's: the birth period compressed into the balance, each period's children from its own lord round, each its lord's share of the row's years | **holds** | 0 of 94128 disagree |
| children start from the sequence's first lord instead | falsified | 1184 of 1184 disagree |
| Tribhagi runs Vimshottari's nine at a **third** of their years, three times round (forty years a round) | falsified | 148 of 148 disagree |
| past the end of its cycle a system begins its sequence again | falsified | 202 of 202 disagree |

## What it means for the module

**Every one is a row of the K-udu kernel.** Eight systems, one tree: the
seat, the balance, the compressed birth period, the children and the end
of the cycle are Vimshottari's in every one, so a system is its lords,
its reference, its direction, its window and its offset, and nothing is
a module of its own. The corpus decides five of the seats outright: one
reference, direction and offset survives. Yogini's and Dwadashottari's
leave a second, which differs only at a seed the corpus does not record,
and Tribhagi's nine lords divide the 27 exactly, so any reference with
the matching offset is the same map. The rule data comes from the
recording engine's cited rows; where the corpus cannot choose, it
confirms it without choosing it.

**A temporal balance reads a window through the Moon's own nakshatra.**
Building Vimshottari refused a temporal balance over a window because no
source defined one. The recording engine defines it: the whole
nakshatras of the window behind the seed are gone, and the time elapsed
in the Moon's own nakshatra is that nakshatra's share of the window.
Every Ashtottari answer agrees, so the kernel takes that reading. The
other reading, the Moon's time across the whole window, is not recorded
and stays unbuilt.

**Tribhagi is Vimshottari scaled, not a sequence of its own.** Each
mahadasha is two thirds of its Vimshottari years and the sequence runs
twice, eighty years a round, with the sub-period shares still over a
hundred and twenty. The reading that runs a third of the years three
times round is refused by every answer. So a row carries a scale and a
number of rounds, which is the design page's scale decorator as two
fields.

**The cycle ends in every system**, Yogini's thirty-six years included,
which is the corpus's reading and the `dasha.after_cycle` default.