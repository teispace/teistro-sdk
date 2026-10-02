# The lots, measured

Status: `generated` by `cargo xtask lots` from the corpus's recorded
births, 2026-10-02. Do not edit: `check-lots` regenerates this page and
fails on any difference.

Valens's lots (`hellenistic-lots.md`) are held to his two worked
examples of Fortune by night by the unit tests of `crates/hellenistic`.
Those examples give signs and no degrees, and both have the Moon up, so
they cannot decide between his readings; this page counts how often the
choice falls on real skies. It reads each of the corpus's 55 births in
the tropical zodiac through `ChartArea::lots_with_request`, every lot at
once, under each `FortuneRule`; 32 of them are night births by Valens's
horizon.

| proposed rule | verdict | measured |
|---|---|---|
| Daimon is Fortune reflected in the ascendant, under every rule for Fortune by night | **holds** | 0 of 165 disagree |
| Basis stands no more than half the circle past the ascendant, "from the nearest Lot to the other" (II.22) | **holds** | 0 of 165 disagree |
| C221: night births whose Fortune changes sign under Lilly's rule instead of Valens's reversal (II.22) | **holds** | 30 of 32 |
| C221: night births whose Fortune changes sign under III.11 instead of II.22, which can happen only with the Moon set | **holds** | 20 of 32; the Moon set at 22 |
| night births whose Love changes sign under Lilly's Fortune, the two reversals no longer cancelling | **holds** | 30 of 32 |
| C221's premise: the Moon's ecliptic hemisphere from the ascendant says whether it is up, as its altitude does | falsified | 3 of 32 disagree; its latitude carries it across the horizon the ecliptic does not |

## How many signs each lot falls in

| lot | signs it falls in | most births in one sign |
|---|---|---|
| `Fortune` | 12 | 8 |
| `Daimon` | 12 | 8 |
| `Basis` | 12 | 9 |
| `Love` | 12 | 9 |
| `Necessity` | 12 | 8 |
| `Exaltation` | 12 | 6 |
| `Debt` | 12 | 7 |
| `Theft` | 12 | 8 |
| `Deceit` | 12 | 7 |
| `ForeignLands` | 12 | 7 |
| `Father` | 12 | 9 |
| `Marriage` | 12 | 9 |
| `Brothers` | 12 | 9 |
| `Crisis` | 12 | 10 |

## What it means

The first two rows hold the shapes the text gives on every birth and
every reading. The next three count what the readings of Fortune by
night move: Lilly's against Valens's II.22, which every lot built on
Fortune inherits, and III.11's, which parts from II.22 only on a night
whose Moon has set. The last row is the premise of reading III.11's
"above the earth" by the Moon's altitude rather than by the ecliptic
from the ascendant: where it is falsified, the two disagree on a real
sky, and the altitude is the one that says whether the Moon has set.
