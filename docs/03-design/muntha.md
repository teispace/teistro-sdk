# The Muntha, the office-bearers, and what the lord of the year waits on

Status: `built`, 2026-09-22, for the Muntha and **all five
office-bearers**; **designed, not built**, for the year lord chosen among
them. Measured in
[`muntha-measured.md`](muntha-measured.md) (`check-muntha`), and built on
the annual chart's instant ([`annual-chart.md`](annual-chart.md)).

`crates/tajika` holds it; `sdk.chart().muntha(&document, completed_years,
degree)` reaches it; and it crosses to C, Node, Dart and Python in the
charts blob's `praveshas` section, beside the return it stands at.

## What it is

The **Muntha** is the birth lagna's sign advanced one sign for each
completed year of life — "the progressed ascendant". It is the first of
the annual chart's five **office-bearers**, one of whom becomes the lord
of the year, and it is the one that takes the lordship when none of the
others qualifies. So an error in the Muntha is an error in the year lord
as well.

This is the first piece of the SDK built from a **Tajika text** rather
than from the references' feature lists: K.S. Charak, *A Textbook of
Varshaphala*, rank 2 — a modern textbook, not the Tajika Neelakanthi. What
it says and what was verified out of it is
[`01-research/feature-universe/07-tajika-varshaphala.md`](../01-research/feature-universe/07-tajika-varshaphala.md),
"A Tajika source, read".

## The rule, and the one thing about it that is hard

"Add to the lagna sign in the birth chart the number of completed years of
life, divide by twelve, and the remainder is the sign." That is the whole
rule, and the source's worked chart checks it end to end: Leo rising,
forty years complete, the Muntha in Sagittarius.

The hard part is not the rule but **which number it is given**. The
source states it in years *completed* and numbers its own worked chart by
the year of life it *opens* — the chart with forty years complete is the
one it calls "the forty-first year". Measured over every recorded birth
and 120 years, the two readings never once agree on the Muntha's sign,
and they agree on its lord in exactly one case in twelve, only where
Capricorn meets Aquarius, because Saturn rules both. So the wrong reading
is wrong everywhere and **invisible often enough to pass a spot check**.

The SDK closes that in the types rather than in a comment:

- the argument is named **`completed_years`**, never `year`;
- `Pravesha::year` counts **returns**, so a caller holding one passes its
  `year` straight in — a correction made to that field's documentation for
  this reason, before anything read it;
- and a test pins the rule against the rival, not only against itself.

## What the corpus can and cannot settle

The corpus records no Muntha. It records the Muntha's **only input**: the
recording engine's lagna, on every chart. Every Muntha a birth will ever
have is that sign rotated, so the pass holds the façade's answer for every
year 0 to 120 of every recorded birth against the **recording's** lagna,
not the SDK's — 6655 values, none wrong.

A lagna fails a Muntha in one way only: by falling on the other side of a
sign boundary, which moves it for the whole of a life. So the pass
measures each birth's founding error against **that birth's own** distance
to a boundary. The worst spends 0.03% of it. Its first draft compared the
worst error anywhere with the tightest margin anywhere and called that
74% — two different births, and about 2 500 times too alarming.

What no corpus can settle is where the Muntha stands **inside** its sign
(crux C107). The source progresses it 2°30′ a month and 5′ a day, which
fills a sign in exactly a year only if the year begins at the sign's first
degree; the rival carries the natal lagna's degree across. Both give the
same sign at the return, so the house-by-house readings do not care; they
part inside the year, so a Tajika aspect taken to the Muntha does.

## What is decided and what is not

| | |
|---|---|
| **decided** | the natal lagna's sign advanced by the years **completed**; the sign's lord as the Munthesha; the argument named `completed_years`; zero as the birth itself, where the Muntha sits on the lagna |
| **a setting** | `MunthaDegree` — `SignStart`, the source's own reading and the default, or `NatalDegree`; each answer carries the reading that made it |
| **not decided** | the Muntha's longitude inside its sign between the two readings (crux C107), which waits on a text that gives a longitude rather than a sign |
| **across the boundary** | `varsha_json.muntha` names the reading; `muntha_sign`, `muntha_lord` and `muntha_deg` ride in the `praveshas` section beside the return they stand at, so no second call can disagree about which year it is. Node, Dart and Python each read `pravesha.muntha`, each binding's test asserts the rule, and all four parity runners print it under all three return readings — which also holds the Muntha's independence of the reading |
| **built** | `teistro_tajika::muntha` and `Muntha::during`, the progression a fraction through the year; `sdk.chart().muntha`, which needs **no ephemeris**, being the founded chart's own lagna and a count |
| **not built** | the Panchavargiya bala, the Tajika aspects and the year lord — below; and the office-bearers' crossing |

## The office-bearers

`teistro_tajika::office_bearers` reads the five from a `YearCharts` — the
six numbers the rules touch, a struct so that no two longitudes can be
swapped by position — and `sdk.chart().office_bearers(&natal, &annual,
completed_years)` fills it from the birth chart and an annual chart the
caller founded. "By day" is the founded chart's own `day.part`, sunrise to
sunset at the annual chart's place, which is the source's definition and
one the corpus already held the foundation to. It needs no ephemeris.

The **Tri-Rashi** lord ships as the rule and not the table (crux C108),
with the 24 printed cells as its test; a second test holds that every sign
has an element with a triplicity, three signs to each, which is what keeps
the rule's one fallback dead. The answer carries each planet's
**portfolios** and the distinct **claimants**, one to five of them, because
the year lord's own tie-break is the number of portfolios held.

**The source's worked year reproduces end to end** through the façade —
birth, the return to the second, the annual chart to the arcminute, and
all five office-bearers — and the page publishes it
([`muntha-measured.md`](muntha-measured.md) §5). Two things came out of
reading it back. The source's return is the **mean** one, its Dhruvanka
being forty mean sidereal years modulo a week, so the SDK's
`Reading::Mean` is what lands on its 13:17:29 IST; the true return is the
"few minutes" it sets aside. And its positions are **geocentric**: under
the topocentric profile its Moon is 58′ out, the Moon's parallax, and under
the default it is 3′. Neither moves an office-bearer here, but a Moon near
a sign boundary by night would move the Dina-Ratri lord, which is why the
profile's centre is reported and not assumed.

**They do not cross the boundary yet.** Three of the five need an annual
chart founded at a place, and the boundary answers the return's instant
precisely so that it does not choose that place. Crossing them means a
function over two founded charts, or a `varsha_json` place that is the
residence knob the annual chart's page names — a decision of its own, and
the next one to make.

## Why only the Muntha crosses in the `praveshas` section

Of the five office-bearers, **only the Muntha needs nothing but the
birth**. The other four need the annual chart itself:

| office-bearer | what it needs |
|---|---|
| the Muntha's lord | the birth's lagna and the years — **built** |
| the natal lagna's lord | the birth's lagna |
| the annual lagna's lord | the annual chart's lagna, which depends on **where** it is cast |
| the Tri-Rashi lord | the annual lagna, and whether the return falls by day or by night **at that place** |
| the Dina-Ratri lord | the Sun's or the Moon's sign, chosen by day or night at that place |

The boundary answers the return's **instant** and not its chart, because
whether an annual chart is cast for the birthplace or the residence is a
question the schools answer differently and the SDK does not decide it
([`annual-chart.md`](annual-chart.md)). So three of the five cannot be
computed where the instant is: they belong to a function over an annual
chart **the caller has founded**, with the place they mean. The Muntha is
the one that can travel with the instant, and it does.

## The order of work for the lord of the year

The source gives the selection completely (crux C106): the strongest of
the five by Panchavargiya bala **that also aspects the annual lagna**,
with named fallbacks down to the Muntha's lord. That needs three things
this build does not have, and the reading found that two of the three
tables they need are rules rather than tables.

1. ~~**The office-bearers**, over a founded annual chart~~ — **built**
   (above); their crossing is the decision still open.
2. **The Panchavargiya bala** — Griha 30, Uchcha 20, Hudda 15, Drekkana 10,
   Navamsha 5, divided by four — with Tajika's own **positional**
   friendship. Its **Drekkana** lords are one expression,
   `(sign + 5 × decanate) mod 7` over Mars, Mercury, Jupiter, Venus,
   Saturn, Sun and Moon, reproducing all 36 printed cells. Its **Hudda**
   table has no rule: it is the Egyptian terms with three signs
   transposing two lords (crux C109), shipped as printed and held by two
   invariants a test asserts over all 60 cells. The Uchcha and Navamsha
   parts already exist in `strength` and `vargas`.
3. **The Tajika aspects**, with their orbs, for "aspects the annual
   lagna" — which the research page's own closing checklist still asks to
   be confirmed.
4. **The year lord** itself, each fallback a named step so a caller sees
   which one decided it.

The four dashas the annual chart unblocks — Mudda, Varsha Narayana, Varsha
Yogini and Patyayini — come after, and are counted in
[`dasha-coverage-measured.md`](dasha-coverage-measured.md).
