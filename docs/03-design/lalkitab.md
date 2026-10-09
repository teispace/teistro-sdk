# `lalkitab`: the computable parts of Lal Kitab

Status: `draft`, 2026-10-09. Track B of the completion plan
(`07-roadmap/00-roadmap.md`), the module catalogue's `lalkitab` row ("facts
only, cited by edition: the books are likely in copyright to 2042"). The
kernel, `crates/lalkitab`, is built; the façade, the boundary and the
bindings are the next steps (§7).

## 1. The source and what may ship

Lal Kitab is five books in Urdu script by Pt. Roop Chand Joshi (1898–1982):
*Lal Kitab ke Farman* (1939), *Lal Kitab ke Arman* (1940), the *Gutka*
(1941), the amended *Farman* (1942) and *Ilm-e-Samudrik ki Buniyad par ki
Lal Kitab* (1952). The 1952 book is the complete and final statement; the
earlier four are drafts it supersedes, and every modern derivative names
it as its base. **The module reads the 1952 edition** (crux LK-C10), and
every page number here is that edition's own.

The pages were read through a page-wise Devanagari transliteration of the
1952 grammar portion (pp. 1–234, the Internet Archive item
`lal-kitab-1952-grammer-portion-page-1-to-234`), whose footers carry the
1952 pagination, checked against the Urdu scan
(`LalKitabEdition1952Part1Of3`) where the transliteration was unclear. Each
fact carries a rank: **R1** read on the page, **R2** read through OCR only
(meaning clear, wording garbled), **R3** derived from a rule the book
states.

**Copyright.** The author died in 1982. In India the books are protected
to the end of 2042 (life plus sixty years); the 1952 edition's restored US
term runs to the end of 2047. So the SDK ships method — house numbers,
planet lists and conditions, in its own words, each cited to its page —
and no sentence, verse or remedy wording of the books. A table of numbers
can still be argued to be the author's protectable selection, which is why
the 120-year annual list is not shipped (LK-C9, §3.6). This is not legal
advice; the maintainer signs off on LK-C9.

## 2. The chart (*teva*)

1952 pp. ~6–7 (R2): draw the ordinary sidereal birth chart, renumber its
boxes so the lagna's is house 1 and the rest follow in order, and drop the
signs. House `n` is then treated as rashi `n` (Aries is 1) for every rule
that names a sign. So a planet's house is its **whole-sign house from the
lagna**:

```
house = ((sign − lagna_sign) mod 12) + 1
```

`Teva::from_signs(lagna, signs)` does exactly this, and `Teva::from_houses`
takes a teva a reader already has. One open-source implementation numbers
houses by the planet's sign with Aries always 1; that is an Aries-lagna
chart, which pp. 6–7 contradict (LK-C1). Bhava chalit houses are not
mentioned in the pages read.

## 3. What is computed

### 3.1 Dignity: pakka ghar, own, exalted, debilitated

The **pakka ghar** (permanent houses) are the p. 29 diagram (R1): Sun 1,
Moon 4, Mars 3 and 8, Mercury 7, Jupiter 2, 5, 9 and 11, Venus 7, Saturn 10
and 8, Rahu 12, Ketu 6.

**Own houses** are the classical rulership read by number, because house
`n` is rashi `n` (R2), so `tables::own` reads them off the catalogue
(`Graha::attributes().own`) rather than keeping a copy. The nodes own
none: houses 6 and 12 are held jointly, Mercury with Ketu and Jupiter with
Rahu (p. ~46, R2).

**Exaltation is a rule, not a list** (pp. ~44–46, R2): the book defines it,
says a planet is debilitated in the seventh house from its exaltation, and
its examples are exactly the classical exaltation signs read as houses.
So the seven come from the catalogue's exaltation sign and debilitation is
`seventh_from` it; the nodes take the book's own rule, Rahu exalted in
Mercury's houses 3 and 6 and Ketu in Jupiter's 9 and 12, each debilitated
in the other's. Modern books' multi-house lists and the baseline engine's
(§5) have no page (LK-C6). The text's remarks that houses 2, 5 and 11 hold
no division and that nothing but the Moon is exalted in 8 sit oddly with
the rule and are left to a second reading.

**Regard** (p. 31, R1) is directed — the page says so, and the table is not
symmetric (the Moon counts Venus equal; Venus counts the Moon an enemy) —
so `regard(of, to)` is stored that way.

### 3.2 Conditions of the teva

| condition | rule (paraphrase) | page | rank |
|---|---|---|---|
| **ratandha** (night-blind) | the Sun in 4 and Saturn in 7 | ~43 | R2 |
| **nabalig** (a minor's chart) | houses 1, 4, 7 and 10 empty, or holding only one papi (Saturn, Rahu, Ketu) or Mercury alone | ~48 | R2 |
| **dharmi** papis | Rahu or Ketu in 4 or with the Moon; Saturn in 11 or with Jupiter | ~43 | R2 |
| **sathi** (companions) | two planets each in a house the other claims (own, exalted or pakka) | ~43 | R2 |
| **kayam** (established) | a planet in a dignity, alone in its house, and looked at from no occupied house | 34 | R1 |
| **masnui** (artificial planets) | a listed pair in one house counts as a third planet | 27 | R1 |

The *andha* (blind) teva — house 10 spoiled by mutually hostile or
"worthless" planets — is not computed: "worthless" is not defined in the
pages read.

### 3.3 Aspects

Aspects run **forward only** (pp. 102–104, R1): the earlier house looks at
the later and the later does not look back. The printed arrow table:
1 → 7 and 4 → 10 in full, 3 → 9 and 11 and 5 → 9 at a half, 2 → 6, 6 → 12
and 8 → 2 at a quarter. The verse pairs the quarter aspects more loosely
(2/6 and 8/12); the arrow table is explicit and wins (LK-C7).

The **yog drishti** relations of p. 112 (R1) are a rule over every house
`h` — help sees `h+4` and is seen from `h+8`, general `h+6`/`h+6`,
collision `h+7`/`h+5`, foundation `h+8`/`h+4`, deceit `h+9`/`h+3`, the
joint wall `h+1`/`h−1` — checked against all twelve printed rows, so
`yog_drishti` computes it. The "sudden strike" column is irregular and the
p. 113 combined-strength table carries fractions and "eclipse" cells that
need a second reading against the Urdu; both are deferred (LK-C8).

### 3.4 Sleep and waking

pp. 96–99 (R1): an occupied house is awake, and so is an empty one an
occupied house looks at; **a house sleeps when it is empty and
unaspected** (LK-C11: the page's "or" between the two clauses reads as
both, because an occupied house is "always awake" on the same page). A
planet in its pakka ghar is always awake (the book's examples: Venus in 7,
Mars in 3). Otherwise a planet of the first side (1–6) sleeps when every
house it looks at is empty; one of the later side (7–12) sleeps when the
first side is empty, and from 8 also when 2 is. The specific pairs of
p. 98 put planets in 2 to sleep when 10 is empty, and in 9 and 10 when 2
is. Each house has its waker (p. 98) and each planet its waking age
(p. 99), which is also where its second round of the 35-year cycle starts
(§3.5) — a table that checks itself, held by a test.

### 3.5 Periods

The **35-year cycle** (p. 33, R1; p. ~219, R2) runs the planets in Lal
Kitab's order for their cycle years — Jupiter 6, Sun 2, Moon 1, Venus 3,
Mars 6, Mercury 2, Saturn 6, Rahu 6, Ketu 3 — and repeats. The general
table starts it with Saturn in the first year of life. The book also
starts a life elsewhere (its worked example has Venus from the 17th year)
but ties the choice to a chart-checking passage that is not determinate,
so the start is an explicit input with the general table as its default
(LK-C4). A start runs the cycle backwards as well as on.

A **year of life** is the running year: year 1 runs from birth to the first
birthday (LK-C2). The year changes on the civil birthday (LK-C3); the pages
read give no astronomical moment, so a solar-return year is a reader's
choice the façade can offer, not the default.

The **mahadasha** years (p. 33, R1) are Vimshottari's values in Lal
Kitab's order, 120 in all, and each year divides into thirds ruled by the
planets of p. 34 (R1). The page keys the thirds to "the year's planet";
the module reads that as the planet whose period of the 35-year cycle the
year falls in (LK-C12), the one planet a year has in the pages read.

### 3.6 The annual chart (*varshphal*)

The 1952 list (pp. ~222–225) gives, for every year of life from 1 to 120,
the house each natal house's planets reach; planets that share a natal
house move together, so Rahu and Ketu need not stay opposite. The list is
**data, not a rule**: its rows are not the powers of one permutation and
only its first column repeats with period 12. What the list does satisfy
is checkable: every row is a permutation of the twelve houses, and every
block of twelve years is a Latin square, each house reached once in each
column.

LK-C9 decides that the SDK ships the reader and those checks, not the
list. `VarshphalTable::from_rows` takes the rows a reader supplies and
refuses the first cell that breaks either check by its index, which is
what settles a misread cell. The baseline engine's "annual chart" is a
twelve-year rotation; the list is not a rotation (§5).

### 3.7 Debts (*rin*)

p. 125 (R1): each of nine debts arises when one of a planet's listed
enemies sits in one of that planet's houses — Jupiter's 2, 5, 9 and 12 for
the ancestors' debt, the Sun's 5, the Moon's 4, Venus's 2 and 7, Mars's 1
and 8, Mercury's 3 and 6, Saturn's 10 and 11, Rahu's 12, Ketu's 6. The
ancestors' debt has a first state (p. 128, R1): a planet in 9 while Mercury
sits in that planet's root house. Its second state's rows were not read
in full, and the modern three-debt schemes have no page (LK-C5). The debt
is read from the birth chart only, never from an annual one (p. ~128, R2).

### 3.8 Not computed

The remedies (*upay*) are the books' wording and do not ship; a remedy is
a key (planet, house, context, category) whose text comes from a vetted
pack, as every other interpretation does. Chandra kundali and the teva
read from the palm (Farman 12, p. ~200) are named but not specified in the
pages read.

## 4. The model

`crates/lalkitab` depends on `teistro-core` alone and holds no sky: a
`Teva` is nine houses, so every rule is a function of a value a reader can
write by hand, and the façade only builds the teva from a chart.

| item | what it is |
|---|---|
| `Teva` | each planet's house, built from signs and a lagna or from houses; refuses a graha outside the nine, a planet given twice or missing, a house outside 1–12, each by its field |
| `life(&Teva, &LifeRules) -> Life` | the reading, the cycle's periods over years 1 to 120, and a year asked for: its ruler, its thirds and, with a list, the annual teva's reading |
| `read(&Teva) -> Reading` | per planet: dignities, owners and its regard for them, awake, kayam, what it casts; per house: occupants, who looks at it, awake, waker; masnui formed; debts with the enemies seated; the ancestors' debt's first state; the teva's flags |
| `cycle::{CycleStart, ruler, periods}` | the 35-year cycle from the general start or a reader's |
| `aspects::{looks_at, looked_at_by, yog_drishti}` | the arrow table and the p. 112 rule |
| `tables` | every table, cited to its page, written as its rule where it has one |
| `VarshphalTable` | a reader's annual list, checked, and the annual teva it gives |

## 5. The baseline engine

Its Lal Kitab part takes a ready planet → house map and never builds the
chart. It is a feature list, not an oracle: its citations name chapters
that do not exist under those titles and carry no page. Where it departs:

| feature | the baseline engine | the 1952 book |
|---|---|---|
| pakka ghar | one house per planet | Jupiter also 5, 9, 11; Saturn also 10; Mars also 8 |
| exaltation | multi-house lists, most of them wrong | the classical signs as houses, the nodes by their rule |
| sleep | conjunction rules | empty and unaspected houses, the two sides, the pakka exception |
| debts | three, with invented triggers and a severity score | nine by enemy-in-house, and the ancestors' first state |
| annual chart | a twelve-year rotation | the 120-year list, which is not a rotation |
| empty-house lords | its own table | the pakka ghar, which differs at 7, 8 and 11 |
| yogas | generic ones (budh-aditya, kaal sarp and the like) | none of these in the grammar read |
| aspects, masnui, kayam, dharmi, the cycle, thirds | absent | present |

Parity with it is therefore declared as a departure, row by row, not
measured.

## 6. Cruxes

| id | question | decided | why |
|---|---|---|---|
| LK-C1 | how a planet's house is found | whole-sign from the sidereal lagna | pp. 6–7 renumber the drawn chart's boxes; the Aries-is-1 variant contradicts them |
| LK-C2 | which annual row applies at an age | the running year: row `completed years + 1` | the book counts "the Nth year"; row 1 is not the birth chart, so no row 0 exists |
| LK-C3 | when the year changes | the civil birthday; a solar return as an option | no astronomical moment in the pages read |
| LK-C4 | where the 35-year cycle starts | the general table (Saturn from year 1), a reader's start as input | the general table is printed; the individual start's rule is not determinate |
| LK-C5 | which debts | the p. 125 table and the ancestors' first state | the modern three-debt scheme has no page |
| LK-C6 | exaltation and debilitation | the seventh-house rule over the classical signs; the nodes by the book's rule | the book states the rule and its examples match it |
| LK-C7 | the quarter aspects | 2 → 6, 6 → 12, 8 → 2 | the printed arrow table is explicit; the verse's pairing is loose |
| LK-C8 | the p. 113 combined strengths and the sudden-strike column | deferred | fractions and "eclipse" cells need a second reading against the Urdu |
| LK-C9 | shipping the 120-year list | not shipped: the reader supplies it, and the SDK checks it | the selection is the author's and in copyright to 2042 (India) and 2047 (US); maintainer sign-off pending |
| LK-C10 | which edition | 1952 | complete and final; practice follows it |
| LK-C11 | when a house sleeps | empty **and** unaspected | an occupied house is "always awake" on the same page, so the "or" cannot mean either clause alone |
| LK-C12 | whose thirds divide a year | the planet ruling it in the 35-year cycle | p. 34 keys the thirds to the year's planet, and the cycle is the only rule in the pages read that gives a year one |

## 7. Order of work

1. The kernel, `crates/lalkitab`, with the worked examples as acceptance
   tests: the Libra-lagna chart (pp. ~6–7), the companions and the
   night-blind chart (p. ~43), the always-awake pakka planets (p. 97), the
   house-4 yog drishti row (p. 112), the ancestors' first state (p. 128),
   the general cycle and the Venus-from-17 start (p. ~219), the period
   totals (p. 33), and a synthetic annual list with the book's structure
   and none of its numbers. **Built 2026-10-09.**
2. The façade: `sdk.lalkitab()` reading a chart's lagna and grahas into a
   teva, its reading, its cycle and, with a supplied list, its annual
   teva.
3. The boundary (JSON), the bindings and parity, and a guide.
4. A second reading of LK-C6's remarks, LK-C8's tables and the debt's
   second state, against the Urdu.
