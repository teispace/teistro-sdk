# The Muntha, measured

Status: `generated` by `cargo xtask muntha` over the conformance
corpus's recorded births. Do not edit: `check-muntha` regenerates this
page and fails on any difference. The design it measures is
[`muntha.md`](muntha.md).

The **Muntha** is the birth lagna's sign advanced one sign for each
completed year of life. It is the first of the annual chart's five
office-bearers and the one that takes the year's lordship when none of
the others qualifies.

**The corpus records no Muntha, and no annual chart of any kind.** It
records the Muntha's only input: the recording engine's lagna, on every
chart. Every Muntha a birth will ever have is that sign rotated by a
count of years, so the recorded lagna falsifies all of them at once.

## 1. What holds

| proposed rule | verdict | measured |
|---|---|---|
| the source's worked chart: Leo rising, 40 years complete, the Muntha in Sagittarius | **holds** | 0 of 1 disagree |
| every year 0 to 120 of every recorded birth: the recorded lagna's sign advanced by the years complete, and that sign's lord | **holds** | 0 of 6655 disagree |

The first row is the only value in the source that checks the rule end
to end. The second is 6655 Munthas across 55 recorded births, each asked
of the façade (`sdk.chart().muntha`) and held against the
**recording's** lagna rather than the SDK's, which is what makes it a
check and not the rule agreeing with itself.

## 2. How wrong the lagna may be

A Muntha is only as right as the lagna under it, and the lagna fails it
in one way only: by falling on the other side of a sign boundary, which
moves the Muntha for the **whole of a life** rather than for a year. So
the unit that matters is not the lagna's error but the error against the
distance to the nearest boundary.

The SDK's founded lagna is at most **12.0 arcseconds** from the
recording's over 55 births, and it falls in a different sign from the
recording's on none of them.

| birth | recorded lagna to the nearest boundary | founded minus recorded | of its margin |
|---|---|---|---|
| `c048-kathmandu-2399-12-30` | 11.0356° | -12.0″ | 0.03% |
| `c046-kathmandu-2350-01-01` | 14.6284° | -7.1″ | 0.01% |
| `c047-london-1800-01-02` | 6.6104° | -0.8″ | 0.00% |
| `c054-kathmandu-2020-05-05` | 0.0045° | +0.0″ | 0.00% |
| `c023-london-1830-06-26` | 9.0954° | -0.1″ | 0.00% |

The five that spent most of their own margin, most first — each
birth's founding error against **that birth's** distance to a boundary.
The worst spends **0.03%** of it. The closest any recorded lagna comes
to a boundary is 0.0045° (16 arcseconds), and that birth's founding
error is smaller still.

Comparing the worst error anywhere with the tightest margin anywhere
would read as 74% of the way to moving a Muntha, and it would be two
different births: the error sits on one and the margin on another.

## 3. The rival: the year of life it opens

The source states the rule in years **completed** — "add to the lagna
sign the number of completed years of life" — and numbers its own
worked chart by the year of life it **opens**: the chart with forty
years complete is the one it calls the forty-first year's. A reader who
takes the chart's number for the count progresses the Muntha one sign
too far.

| over every recorded birth, years 1 to 120 | cases |
|---|---|
| the two readings agree on the Muntha's **sign** | 0 of 6600 |
| they agree on the sign's **lord** | 550 of 6600 |

They never agree on the sign, and they agree on the lord only across
Capricorn→Aquarius: the one step in the zodiac where a single planet
rules both sides, Saturn holding Capricorn and Aquarius. So the rival is
wrong everywhere and **invisible once in twelve** — often enough that
a spot check can land on it and pass. `Pravesha::year` counts returns
and the Muntha's argument is named `completed_years` for this reason.

## 4. The two readings of the degree

The source progresses the Muntha 2°30′ a month and 5′ a day, which
fills a sign in exactly a year only if the year begins at the sign's
first degree — `MunthaDegree::SignStart`, the default. The rival
carries the natal lagna's degree into each new sign
(`MunthaDegree::NatalDegree`). Both give the same sign at the return, on
all 55 births.

They part by the natal lagna's degree within its sign, which over the
recorded births runs from 0.00° to 29.84°. Carried forward, a year of
progression takes the Muntha out of its sign **before** the next return
on 55 births — every birth whose lagna is not on a boundary — so the
two readings differ for any Tajika aspect taken to the Muntha late in a
year and for nothing else (crux C107).

The cap is 200 years, shared with the returns.