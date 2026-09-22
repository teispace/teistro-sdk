# The Tajika aspects, their orbs, and Ithasala

Status: `built`, 2026-09-23. Measured against the source's own worked pair
in [`muntha-measured.md`](muntha-measured.md) §8 (`check-muntha`), and
built on the annual chart ([`annual-chart.md`](annual-chart.md)).

`crates/tajika`'s `drishti` holds it; `sdk.chart().drishtis(&annual)`
gives every pair of the seven; and the pairs that make a yoga cross to
every binding.

## Two things wear the word "aspect"

Keeping them apart is most of this module.

**The aspect** is a relation between two **signs**: friendly at houses 3,
5, 9 and 11, inimical at the kendras 1, 4, 7 and 10, and **nothing at all**
at 2, 6, 8 and 12. The source halves each aspecting kind — Pratyaksha and
Gupta, the open and the secret — and the lord of the year ignores that
division while the yogas do not, so it is carried rather than collapsed.
This is the aspect [`varshesha.md`](varshesha.md) asks for, and the one
that disqualifies its strongest claimant.

**The orb** is a distance between two **planets**, the *deeptamsha* or "orb
of radiance". Each planet has its own, and the one that governs a pair is
the **mean** of the two:

| | Sun | Moon | Mars | Mercury | Jupiter | Venus | Saturn |
|---|---|---|---|---|---|---|---|
| deeptamsha | 15° | 12° | 8° | 7° | 9° | 7° | 9° |

The research page carried these with a "**verify**" against them since
2026-09-04; they are now read from a text, and so is the rule that a pair
takes their mean. The Sun and Mars are governed by 11°30′.

## Behind is degrees within the sign

The source is explicit, and it is the thing an implementer would most
naturally get wrong: a planet is ahead of or behind another **by its
degrees in a sign, the completed signs deleted** — not by longitude along
the zodiac. So the Sun at Leo 3°50′ is *behind* Mars at Scorpio 7°42′,
three whole signs further on. A test asserts exactly that, because the
natural reading of "behind" gives the opposite answer.

## What a pair inside its orb is doing

| | |
|---|---|
| **Ithasala** | the faster planet is behind the slower and coming to it; generally favourable |
| **Ishrafa** | the faster is already past and drawing away; generally not, though the source says an Ishrafa between two benefics is not bad |
| **Rashyanta Ithasala** | the faster is past *but* stands at 29° or beyond, so it "extends its influence to the next house also" and acts from there, where it is behind again |

Neither means anything without the sign aspect as well: a pair in the
neutral houses makes no yoga however close it stands, which a test asserts
rather than leaving to be discovered.

**Faster** is the tradition's ranking and not a measurement — Moon,
Mercury, Venus, Sun, Mars, Jupiter, Saturn — so a retrograde Mars is still
slower than the Sun here. That is stated in the constant's own
documentation so nobody looks for a speed in it.

## What is decided and what is not

| | |
|---|---|
| **decided** | the four aspecting sets and the neutral houses; the deeptamsha table and the mean of a pair; "behind" as degrees within the sign; the speed ranking; 29° as the sign's end |
| **not decided** | whether a retrograde planet reverses the applying direction, which the research page's closing checklist has asked since 2026-09-04 and this text does not answer; the fourteen other yogas |
| **across the boundary** | the pairs that **make** a yoga ride in the ragged `year_yogas` section, counted by `annual_charts.yoga_count`, with the aspect, the yoga, the orb and the signed separation. Each binding reads them as `pravesha.annual.yogas`; all four parity runners print every one of them and **4 bindings agree on 8 090 values** |
| **built** | `teistro_tajika::between` and `drishtis`; `sdk.chart().drishtis`, which needs **no ephemeris** |
| **not built** | the fourteen yogas above Ithasala and Ishrafa, now designed in [`tajika-yogas.md`](tajika-yogas.md); the 36 sahamas. Two things here are also **to be corrected** against the source's Table X-3, read after this shipped: Ishrafa wants the faster planet a whole degree or more ahead (crux C110), the sign's-end kind is the table's **Bhavishyat** rather than the prose's Rashyanta (crux C111), and a third kind, **Poorna** — an Ithasala within one degree — is missing |

## Why only the yoga-making pairs cross

Twenty-one pairs a year is mostly nothing: a typical chart makes six or
eight yogas. The boundary carries **what happened** and not the whole
matrix it was found in, as it carries the year lord and not the strength
table ([`panchavargiya.md`](panchavargiya.md)). A Rust caller who wants
all twenty-one — to ask why a pair makes nothing — has
`sdk.chart().drishtis`.

## What this unblocks

The fourteen other yogas are compositions over pairs: Kamboola needs the
Moon's Ithasala, Nakta and Yamaya need a third planet between two that do
not aspect, Khallasara needs an Ishrafa with a specific quality. The year
lord's own Moon branch wants it too — the source gives two readings that
turn on whether the Moon is in Ithasala with anything, which
[`varshesha.md`](varshesha.md) records as unbuilt for exactly this reason.
