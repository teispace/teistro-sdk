# The annual chart, measured

Status: `generated` by `cargo xtask varshaphala` over the conformance
corpus's recorded births. Do not edit: `check-varshaphala` regenerates
this page and fails on any difference. The design it measures is
[`annual-chart.md`](annual-chart.md).

The corpus records **no annual chart of any kind**, so nothing here is
compared against a recording. What it records is 55 births — their
places, their instants and their settings — and a return is a property
of a birth, so the rule can be measured over them even where its answer
was never written down. 2160 returns in all, 40 a birth.

**The reading is not free.** A chart's frame is tropical and its
ayanamsha is an offset the SDK applies, so "the Sun returns to its natal
longitude" names two different instants, and the ayanamsha's drift is
what parts them. Both are computed here and the second column of the
rivals' table is what the choice costs: not the separation, which is a
number, but where it puts the **lagna**, which is what a reader reads.

## What the births decide

| proposed rule | verdict | measured |
|---|---|---|
| the search loses no return a birth's window holds | **holds** | 0 of 55 disagree; 1 of them stop early because the built-in ephemeris does, not because a return was missed |
| the Sun stands at its natal sidereal longitude at every return | **holds** | worst 0.000 arcseconds over 108 returns, read back through a founded chart |
| a return is one **sidereal** year after the last | **holds** | mean 365.256381 days, worst 0.0139 from the sidereal year and 0.0251 from the tropical |
| the interval is not constant: the Earth's orbit is not a circle | **holds** | 0.33 h between the widest and the mean |

## The two readings this is not

| rival | worst apart | where that puts the lagna | worst at |
|---|---|---|---|
| the **tropical** return: the Sun to its natal tropical longitude | 14.24 hours | 176.90° | `c040-buenos-aires-1986-06-22`, year 40 |
| the **mean** return: birth plus a whole sidereal year each time | 14.5 minutes | 5.60° | `c035-new-york-2021-03-14`, year 40 |

Neither is a rounding error. A lagna 177° from the one a reader would
have read is a different sign, a different lord and a different chart,
and the tables every Tajika judgement is made from are indexed by
exactly that. **Which reading a module takes is therefore a setting and
not a default it may quietly pick**, and the tradition's own is the
sidereal one, which is what this page measures as the rule and the
others as its rivals.

