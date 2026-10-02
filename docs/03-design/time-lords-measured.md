# The Hellenistic time lords, measured

Status: `generated` by `cargo xtask time-lords` from the corpus's
recorded births, 2026-10-02. Do not edit: `check-time-lords` regenerates
this page and fails on any difference.

Releasing and the profected year (`hellenistic-time-lords.md`) are held
to Valens's worked nativities by the unit tests of
`crates/dasha/src/releasing.rs`, and the firdaria
(`hellenistic-firdaria.md`) to al-Biruni's table by those of
`crates/dasha/src/firdaria.rs`. Those give signs, lords and years from a
stated start, so they cannot say how often a real sky puts Daimon in
Fortune's sign, whether a life reaches the loosing of the bond, or how
many births the sect rule decides; this page answers all three. It
founds each of the corpus's 55 births in the tropical zodiac with
`RELEASING_FORTUNE`, `RELEASING_DAIMON`, `PROFECTION` and `FIRDARIA`
asked for, and reads their lots and sect through `ChartArea::lots` under
Valens's rules.

| proposed rule | verdict | measured |
|---|---|---|
| releasing from Fortune starts at the sign of the Lot of Fortune (IV.4) | **holds** | 0 of 55 disagree |
| releasing from Daimon starts at its sign, or at the next when it shares Fortune's (IV.4, C223) | **holds** | 0 of 55 disagree |
| the profected year starts at the Ascendant's sign (IV.11) | **holds** | 0 of 55 disagree |
| the firdaria begin from the Sun by day and the Moon by night, the sect by Valens's horizon (al-Biruni §395) | **holds** | 0 of 55 disagree |
| a stored reading rebuilt from its start sign or first lord gives back every row it carries | **holds** | 0 of 220 disagree |
| C223: births whose Daimon falls in Fortune's sign, so that activity is released from the next | **holds** | 3 of 55 |
| births whose sect by the horizon and by daylight differ, so that the request's sect rule moves their whole firdaria | **holds** | 1 of 55 |
| C224: releasing from any of the twelve signs reaches a second-level loosing before the age of 80, so its reading decides a period in every life | **holds** | 0 of 12 disagree; the latest is released from Libra, at 51.8 calendar years |

## What a document carries

| system | depth | rows, median | rows, most |
|---|---|---|---|
| `RELEASING_FORTUNE` | 3 | 1966 | 1966 |
| `RELEASING_DAIMON` | 3 | 1966 | 1966 |
| `PROFECTION` | 1 | 120 | 120 |
| `FIRDARIA` | 2 | 116 | 116 |

## What it means

The first five rows hold what the façade must do on every birth: start
each time lord where its source counts it, and record enough that a
stored chart rebuilds the same periods. The next counts the births C223
decides, where the activity count moves to the sign after Fortune's, and
the one after it the births whose Sun stands near enough the horizon
that Valens's rule and the daylight one name different sects, and so
different firdaria from birth. The last asks the twelve start signs
rather than the births, because whether a life meets the loosing at the
second level is arithmetic: it falls only in a sign whose years outlast
17 years 7 months (Gemini, Cancer, Leo, Virgo, Capricorn and Aquarius),
one of them begins within any 80 years, and C224's reading of the
opposite sign then decides every period after it. A document carries
releasing's whole 211-year cycle to its depth, and the firdaria's two
75-year rounds to their sevenths, which the row counts above price.
