# A saham's strength

Status: built — 2026-09-24, and across the boundary the same day. `crates/tajika/src/saham_strength.rs`,
reached at `sdk.chart().saham_strength` and `saham_strength_with_rules`.

The source is K. S. Charak, *A Textbook of Varshaphala*, ch. XI, "The
Strength of Sahams", and the worked judgements that follow it (rank 2).

## A report, not a verdict

The source lists what makes a saham strong and what makes it weak, and
then judges its worked sahams in words — "extremely strong", "very
strong", "not as strong as in the earlier case", "not too strong" —
weighing the clauses against each other and against the houses, the
aspects and the year lord. **It never scores them**, and a saham can meet
clauses on both lists at once: the forty-sixth year's Punya sits with its
own lord and Mercury, and also with Mars in the Rahu-Ketu axis.

So the module reports every clause, each named and evaluated, and the
facts it was judged from, and **gives no verdict**. Measured over every
recorded birth's forty years ([`muntha-measured.md`](muntha-measured.md)
§16), **three placements in five meet clauses on both lists** — a
verdict would be deciding most sahams by a weighting the source never
gives — and none meets neither, because the strong list's "its lord
aspects or conjoins it" and the weak list's "neither aspects nor conjoins
it" are each other's negation. A score would be a
rule the source does not state, and a caller who wants one can build it
from the clauses; one who wants the source's own words has them.

## The clauses

**Strong** (1 A): the saham becomes strong when

| clause | read as |
|---|---|
| (a) its lord is exalted, or in its own house | the lord's sign: its exaltation sign, a sign it owns |
| (a) … "as well as in the vargas" | the lord in a Hudda, Drekkana or Navamsha it rules: the Tajika vargas of the Panchavargiya bala, reported one by one |
| (a) … "or in those [houses] belonging to its friends" | the lord in a sign whose lord is its friend, by Tajika's positional friendship — the `Relation` the Panchavargiya bala reads |
| (b) associated with a friend, a natural benefic, or the year lord | a planet in the saham's sign that is the lord's friend, a benefic, or the year lord (none for a birth chart) |
| (c) its lord aspects it or conjoins it, or aspects the lagna | the Tajika sign aspect from the lord's sign to the saham's, the same sign, the aspect to the lagna's sign |

**Weak** (2): the saham becomes weak when

| clause | read as |
|---|---|
| (a) its lord is under five units of Panchavargiya bala | the lord's Vishwa bala below 5, exactly, in sub-sub units |
| (b) its lord lacks Harsha bala | the lord's Harsha bala Nirbala, nothing at all |
| (c) its lord neither aspects nor conjoins it | the negation of (c) above, carried as its own clause so the list reads as the source's |
| (d) associated with inimical planets or natural malefics | a planet in the saham's sign that is the lord's enemy, or a malefic |

And the source's two notes: a saham in the **6th, 8th or 12th** "is
handicapped", and "according to some" the Shatru, Roga, Kali and Mrityu
sahams are best **weak** (`Saham::best_weak`). Its worked forty-sixth
year also counts the **Rahu-Ketu axis** against a saham, which the lists
do not; it is reported as a fact, `in_node_axis`, when the chart places
the nodes.

**"Benefic houses"** — clause (a)'s "or is located in benefic houses" —
is the one clause not read: the source does not say which houses are
benefic, and a guess would be a rule the source does not state. It is
named in the not-built row below rather than filled.

## Benefics and malefics: the chapter's own

The yogas module's malefics are Manau's two, Mars and Saturn (crux C114).
**This chapter counts the Sun a malefic too**: its forty-seventh year
calls the Sun "another malefic", and its birth-chart Punya is strong for
being with "all the natural benefics (viz., the Moon, Mercury, Jupiter and
Venus)". So the default natures are the chapter's — benefics the Moon,
Mercury, Jupiter and Venus; malefics the Sun, Mars and Saturn — and the
catalogue's Parashari natures are a named rival.

| `SahamStrengthRules` | default | rival |
|---|---|---|
| `natures` | `Chapter`: the Moon, Mercury, Jupiter and Venus benefic; the Sun, Mars and Saturn malefic | `Parashari`: the catalogue's natures, Mercury neutral |
| `weak_below` | 5 units of Vishwa bala | any floor |
| `harsha` | the Harsha bala's own rules | Venus's rival place |
| `sahams` | the sahams' own rules | every reading of them |

## The worked judgements

The acceptance tests are the source's own, founded end to end from the
birth and the year it prints at Bombay, or read from its printed
longitudes:

| saham | the source says | clauses it names, each held by a test |
|---|---|---|
| birth Punya, Leo 27°52′ | "extremely strong" | in the lagna, with its own lord the Sun and the Moon, Mercury, Jupiter and Venus — the company exactly those five |
| year 41 Punya, Leo 15°16′ | "very strong" | in the tenth with its lord and Mercury and Venus; the lord in its own sign and the year lord |
| year 41 Raja | the native "attained the most powerful status" | its lord Saturn exalted |
| year 41 Matri | the lord "though exalted … receives no friendly aspect" | its lord the Moon exalted |

The forty-sixth year's Punya — "not as strong", with its lord and Mercury
but also the malefic Mars, in the Rahu-Ketu axis — is the source's
clearest case of a saham on both lists. Its year was not cast at Bombay,
and the source does not say where, so it is not founded here; the facts it
names are the ones the report carries, `with_malefic` and
`in_node_axis`.

## Crossing the boundary

The clauses, the Harsha bala and the birth chart's sahams cross together,
on the chart request's `varsha_json`:

| `varsha_json` | what it adds |
|---|---|
| `sahams` | as before; each saham now answers with its strength, under its year's own lord. **Without a place** it answers the birth charts' sahams alone, where it was refused: a birth chart needs no annual chart to hold sahams. |
| `sahamStrength` | `{natures, friendship, weakBelow}`, the readings above |
| `harshaRules` | `{venus}`, the Harsha bala's reading |

| section | what it carries |
|---|---|
| `year_sahams`, `natal_sahams` | each saham's place as before, and its strength: the strong and weak clauses as bit sets over `TsSahamStrong` and `TsSahamWeak`, the lord's Vishwa bala exact and its Harsha grade, and the Rahu-Ketu axis. The birth's are ragged by `cast.natal_saham_count` |
| `year_saham_seven`, `natal_saham_seven` | seven rows under each saham, one per planet: its aspect on the saham's sign, its relation to the lord, and whether it keeps the saham company |
| `year_harsha` | seven rows under each founded year: the four parts, the total and the grade |

**The clause bits are enums the generator reads**, `StrongClause` and
`WeakClause` in Rust mirrored as `TsSahamStrong` and `TsSahamWeak`, so
every binding names a clause from its own generated catalogue and none
keeps a copy of the source's wording. A seven-row section is fixed and
not ragged: the seven are always seven, so a reader indexes it by the
saham's row times seven.

## What is decided and what is not

| | |
|---|---|
| **decided by the source** | the clauses and their lists; the chapter's natures; the 6-8-12 handicap; the four sahams best weak |
| **a reading, not a decision** | the vargas as the Tajika three; friendship as Tajika's positional one; "lacks Harsha bala" as none at all |
| **not built** | a verdict, which the source never gives; "benefic houses", which it does not define |
