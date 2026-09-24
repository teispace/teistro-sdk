# The Ashtakavarga, measured

Status: `generated` by `cargo xtask ashtakavarga` over the conformance
corpus's `baseline/ashtakavarga`, 2026-09-15. Do not edit:
`check-ashtakavarga` regenerates this page and fails on any difference.
The design it measures is [`strength-schemes.md`](strength-schemes.md).

The corpus records the recording engine's Ashtakavarga for 77 charts,
computed from each chart's recorded lagna and seven grahas' signs.
Beside it stands a rank-1 text, BPHS chs. 66 to 69 in English
translation, so every rule the engine takes after the bindu tables is
measured beside the text's.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| each graha's bindus are the table's places counted from the seven grahas and the lagna | **holds** | 0 of 77 disagree |
| whatever the chart, the grahas hold 48, 49, 39, 54, 56, 52 and 39 bindus, 337 in all | **holds** | 0 of 77 disagree |
| the sum is the seven grahas' bindus by sign, and the trine reduction takes the least of each trine from the **sum** | **holds** | 0 of 77 disagree |
| the same reduction made in each graha's Ashtakavarga and then summed, as the text makes it | falsified | 77 of 77 disagree |
| the engine's classical Ekadhipatya on the reduced sum: both signs occupied, nothing; both empty, both the smaller number or both zero when equal; one occupied, the empty sign **zero** | **holds** | 0 of 77 disagree |
| the text's: the same, except that an empty sign beside an occupied sign with the **smaller** number keeps the difference | falsified | 26 of 77 disagree |
| the rashi pinda is the reduced sum times each sign's measure, Virgo's **8** | **holds** | 0 of 77 disagree |
| the same with Virgo's measure **6**, as the text gives it | falsified | 42 of 77 disagree |
| a graha's graha pinda is all its raw bindus times its own measure, and its yoga pinda that plus the rashi pinda of the sign it stands in | **holds** | 0 of 77 disagree |
| the text's yoga pinda: each graha's own reduced Ashtakavarga times the signs' measures, plus each sign's reduced bindus times the measure of every graha standing in it | falsified | 77 of 77 disagree |

## What it means for the module

**The bindu tables are settled**: the engine's are the text's, every
chart holds the classical 337, and nothing about them is a choice.

**Everything after them is the engine's reading, and the text reads it
otherwise.** The engine reduces the sum where the text reduces each
graha's own Ashtakavarga; zeroes an empty co-ruled sign where the text
keeps a difference; and builds its pindas from the reduced sum and raw
bindus, with Virgo's measure 8, where the text builds each graha's from
its own reduced Ashtakavarga and the grahas standing in each sign, with
Virgo's 6. The corpus confirms the engine and refuses the text on every
chart where they part. A rank-1 text corrects a rank-2 value, so the
module's default is the text's and the engine's reading is two settings,
which the conformance profile takes (cruxes C59–C62).
