# The Kalachakra dasha, measured

Status: `generated` by `cargo xtask kalachakra` over the conformance
corpus's `baseline/kalachakra`, 2026-09-15. Do not edit:
`check-kalachakra` regenerates this page and fails on any difference.
The design it measures is [`dasha-kernels.md`](dasha-kernels.md).

The corpus records the recording engine's Kalachakra for 148 answers,
computed from the recorded Moon and nakshatra span of every dasha
fixture, each a tree of 27 mahadashas and their antardashas and the
periods running at two instants. The Moon stands in 47 of the 108 padas,
so a table row the corpus never reaches is untested; the worst balance
is 7.5e-11 days.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| the Moon's nakshatra picks the table the manifest lists it under and its pada the row; the first sign runs the unelapsed part of the pada of its years, every other sign its whole years; after nine the same nine run reversed, three cycles in all; each mahadasha's nine antardashas begin from its own place in its nine and share it by their signs' years | **holds** | 0 of 148 disagree |
| the table is the nakshatra's by its place in its triad, the middle of each triad taking the chakra's second table | falsified | 25 of 148 disagree |
| the balance is the elapsed part of the pada taken of the pada's whole span, the signs it covers skipped | falsified | 148 of 148 disagree |
| after nine the same nine run again in the same order | falsified | 148 of 148 disagree |
| after nine the next pada's nine run | falsified | 148 of 148 disagree |
| every mahadasha's antardashas begin from the first of its nine | falsified | 148 of 148 disagree |
| the manifest's lists are the triad rule | falsified | they differ at Ardra, Uttara Phalguni, Jyeshtha, Shatabhisha, Revati |
| a temporal balance's pada, the Moon's time in its nakshatra taken in quarters, is the pada the table was chosen by | falsified | 12 of 65 disagree |
| the mahadasha running at an instant is the same tree's | **holds** | 0 of 296 disagree; 4 of the instants fall past the third cycle; worst start 4.7e-10 days |

## What it means for the module

**The engine's Kalachakra is one reading at every fork, and the sources
read differently at most of them.** The corpus confirms which reading
the engine takes and can say nothing about which is right, so the kernel
ships the engine's as its defaults and each fork is a crux (C54–C58):
the pada tables' membership, the balance, what follows the ninth
mahadasha, the temporal balance's pada, and anything below the
antardashas.

**The tables are sound and the membership is not settled.** The four
pada tables agree sign for sign with the published chakra tables read;
the nakshatras each table serves do not at the five named above, where
the texts' lists follow the triad rule and the engine's do not.

**The engine builds nothing below the antardashas**, and no source read
defines a third level, so the kernel stops there too.