# The Panchavargiya bala

Status: `built`, 2026-09-22. Measured against the source's own tabulated
chart in [`muntha-measured.md`](muntha-measured.md) §6 (`check-muntha`),
and built on the office-bearers ([`muntha.md`](muntha.md)) and the annual
chart ([`annual-chart.md`](annual-chart.md)).

`crates/tajika`'s `bala` holds it; `sdk.chart().panchavargiya(&annual)`
reaches it, reading an annual chart the caller founded.

## What it is

The **five-fold strength**, and the one the lord of the year is chosen by.
Five parts, each a maximum reduced by how the planet stands to the lord of
the division it occupies — the whole of it in its own, three quarters in a
friend's, a half in a neutral's, a quarter in an enemy's:

| part | what it reads | at most |
|---|---|---|
| Griha (Kshetra) | the sign | 30 |
| Uchcha | the arc from the debilitation point | 20 |
| Hudda | the Tajika term | 15 |
| Drekkana | the **Tajika** decanate | 10 |
| Navamsha | the navamsha, which is the Parashari one | 5 |

Eighty at most; quartered, a **Vishwa bala** out of twenty.

## Three things that are Tajika's own

**Friendship is positional, and read off the annual chart.** Friends stand
at houses 3, 5, 9 and 11 from each other, enemies at 1, 4, 7 and 10,
neutrals at the rest. It is not the natural friendship the Parashari
strengths use, and it has a consequence worth stating outright: house one
is an *enemy's* house, so **a planet sharing a sign with that sign's lord
is its enemy**. The source's worked chart turns on it — Mercury and Venus
both sit in the Sun's Leo and both take a quarter of the Griha bala — so a
test asserts it rather than leaving it to be rediscovered.

**The decanate lords are not the Parashari ones.** The source prints
thirty-six of them and states the rule beside the table: one cycle from
Mars — Mars, Mercury, Jupiter, Venus, Saturn, Sun, Moon, the weekday order
begun at Mars — indexed by `(sign + 5 × decanate) mod 7`. That expression
reproduces every printed cell and all seven of the worked chart's values,
so the rule ships and the table is its test.

**The Hudda has no rule**, and is not therefore arbitrary. Read off the
rendered page, its degrees are the **Egyptian terms** exactly and 57 of its
60 lords are; Gemini, Sagittarius and Aquarius each transpose two adjacent
lords while leaving the widths alone (crux C109). It ships as printed, with
the three named in its note, and two invariants hold over the whole of it:
every sign's five widths sum to thirty, and every sign carries Mars,
Mercury, Jupiter, Venus and Saturn exactly once each. The source's own
worked chart touches seven of the sixty cells and none of the three, which
is why the invariants are asserted and not the example alone.

## The arithmetic is exact

The source writes a strength as `14:20:15` — units, sub-units, sub-sub
units, sixty of each in the next — and says the Vishwa bala is "preferable
to do so up to sub-sub units". So `Bala` holds **integer sub-sub units**,
3600 to a unit, and:

- a relation multiplies by 4, 3, 2 or 1 and divides by 4, exactly;
- the five parts add exactly and the total quarters exactly;
- only the **Uchcha** bala truncates, to whole sub-units, because the
  source's own arithmetic says to ("ignore the remainder from the second
  division") — it is the rule's, not a rounding of ours, and the method is
  named `to_sub_units` so it cannot be mistaken for one.

Floating point would have lost the last figure of every cell the source
prints. Two strengths compare as integers, which is what the year lord's
selection needs when two office-bearers are close.

## What holds it

The source tabulates all seven planets of one annual chart: five parts, a
total and a Vishwa bala each — **49 figures**. Every one is reproduced,
cell for cell, from the chart the SDK founded rather than from the source's
longitudes, so a wrong relation, a wrong table cell, a wrong truncation or
a wrong division lands in that test. The strongest of the seven is Saturn,
as the source has it, and the strongest of the five office-bearers —
which is the figure the year lord is actually chosen by — is Jupiter at
14:46:00.

## What is decided and what is not

| | |
|---|---|
| **decided** | the five parts and their maxima; positional friendship read off the annual chart; the Tajika decanate as a rule; the Hudda as printed; exact integer arithmetic in sub-sub units; the Uchcha truncation, because the source states it |
| **not decided** | whether the Hudda's three transpositions are the tradition's or a printing fault (crux C109), which wants a second printing |
| **built** | `teistro_tajika::panchavargiya` over an `AnnualSky` of seven named longitudes — named, because every one of them is a longitude and an array would let a caller swap two and still get an answer; `sdk.chart().panchavargiya`, which needs **no ephemeris** |
| **not built** | the **Harsha** bala and the **Dwadasha-vargiya** bala, the source's two other strengths, which nothing yet asks for. The table itself does not cross: the boundary carries the **year lord and its claimants**, which is what the strength was computed for ([`varshesha.md`](varshesha.md)), and a Rust caller who wants the whole table has `sdk.chart().panchavargiya` |

## What it unblocks, and what it still waits on

The year lord (crux C106) is the strongest of the five office-bearers
**that also aspects the annual lagna**. The strength half is now here; the
aspect half is the **Tajika aspects**, with their own orbs, which are the
next thing to build. Until then the selection cannot be made honestly, and
nothing here pretends otherwise.
