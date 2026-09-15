# The sign-based dashas, measured

Status: `generated` by `cargo xtask rashi-dashas` over the conformance
corpus's `baseline/rashi-dashas`, 2026-09-15. Do not edit:
`check-rashi-dashas` regenerates this page and fails on any difference.
The design it measures is [`dasha-kernels.md`](dasha-kernels.md)'s
K-rashi schema.

The corpus records 8 sign-based systems for 77 charts, each computed
from the chart's recorded lagna, graha signs and dignities and arudha
and navamsha lagnas: 616 answers, each a tree to antardashas and the
periods running at two instants. None has a balance at birth: the first
mahadasha runs its whole years from the moment of birth.

## Each system

| system | answers | tree rows | wrong | worst boundary, days | cycle, years |
|---|---|---|---|---|---|
| Chara | 77 | 12 012 | 0 | 0.0e0 | 50 to 101 |
| Narayana | 77 | 12 012 | 0 | 0.0e0 | 50 to 102 |
| Padanadhamsa | 77 | 12 012 | 0 | 0.0e0 | 50 to 101 |
| Trikona | 77 | 12 012 | 0 | 0.0e0 | 50 to 101 |
| Drig | 77 | 12 012 | 0 | 0.0e0 | 50 to 101 |
| Shoola | 77 | 12 012 | 0 | 0.0e0 | 108 |
| Niryana Shoola | 77 | 12 012 | 0 | 0.0e0 | 108 |
| Mandooka | 77 | 12 012 | 0 | 0.0e0 | 96 |

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| every system's tree is its stated start, order and years, each mahadasha divided into twelve equal antardashas from its own sign, forward for an odd sign and back for an even one | **holds** | 0 of 616 disagree |
| the mahadashas run forward when the ninth house is odd-footed and back when it is even-footed, rather than by the start sign's parity | falsified | 160 of 385 disagree |
| each mahadasha's antardashas begin from the sign after its own, its own sign last | falsified | 616 of 616 disagree |
| a sign's years count to its lord forward from an **odd** sign and back from an even one, rather than by footedness | falsified | 385 of 385 disagree |
| Scorpio's and Aquarius's periods count to Ketu and Rahu whatever the chart | falsified | 258 of 462 disagree |
| when one of a dual-lorded sign's lords occupies it, the period counts to the other | falsified | 144 of 462 disagree |
| Narayana's years take no adjustment for an exalted or debilitated lord | falsified | 53 of 77 disagree |
| Drig's ninth, tenth and eleventh houses and the signs each aspects reach all twelve signs | falsified | 33 of 77 charts leave signs out, which follow in the zodiac's order |
| the periods running at an instant are the same tree's, and none runs past the twelfth mahadasha | **holds** | 0 of 1232 disagree; 82 of the instants fall past the cycle, where no second cycle begins; worst start 0.0e0 days |

## What it means for the module

**One kernel, eight rows.** Every system is a start sign, an order, a
length rule and the same antardashas, so the K-rashi schema holds as
designed: three direction rules on two kinds of odd, a dual-lord rule,
and a sub-progression.

**The recording engine takes one school's reading where the schools
differ, and the corpus can only confirm which.** Each rival reading
below is taught by a school, none was checked against a rank-1 text, and
each is registered as a crux: the mahadashas' direction by the start
sign's parity or by the ninth house's footedness (C49); antardashas from
the mahadasha's own sign or from the next (C50); a dual-lorded sign's
lord by kendra, or by the ladder whose first step counts to the lord not
in the sign (C51); Drig's order when the ninth, tenth and eleventh
houses' aspects repeat a sign, where the engine appends the signs left
out (C52); and the start sign and second cycle the systems take (C53).
So each is a field of the row with the engine's reading as its value and
its rival expressible, and nothing here says which school is right.

**Footedness is not parity.** Counting a sign's years by plain parity
instead of by threes from Aries is refused by most charts, which is the
design page's direction error measured: the two kinds of odd are
distinct types.