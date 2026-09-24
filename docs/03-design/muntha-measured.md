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

## 5. The source's worked year, end to end

The source works one birth all the way through — Bombay, 20 August
1944, 07:11 IST — to its forty-first year's chart and that chart's
five office-bearers (Chart III-1). It is the only rank-2 value in reach
that checks the whole pipeline at once: the return, the chart it founds,
and the lords read from both. Its return is the **mean** one — its
Dhruvanka of 1d 6h 6m 29s for forty years is forty mean sidereal years
modulo a week — so that is the reading held here.

| against what the source prints | the default profile (geocentric, mean ayanamsha) | the conformance profile (topocentric, nutated) |
|---|---|---|
| the return, 13:17:29 IST | +1.5 s | +1.5 s |
| the annual lagna, Scorpio 9°26′ | +0.9′ | +1.1′ |
| the Sun, Leo 3°50′ | -0.8′ | -0.6′ |
| the Moon, Taurus 9°40′ | -2.7′ | **-57.6′** |
| the office-bearers, Jupiter, Sun, Mars, Mars, Sun | all five as printed | all five as printed |
| the true return, after the mean one | +0.83 min | +4.84 min |

The source prints whole arcminutes and seconds. On the default profile
the worst of its three positions is 2.7′ out and its return 1.5 s,
which is an ephemeris a generation apart agreeing to arcminutes and not
a rounding. **Its positions are geocentric**: under the topocentric
profile its Moon is almost a degree out, which is the Moon's parallax at
Bombay and not an error. And the "few minutes" it sets aside between the
true return and the mean one are the Sun's own perturbations on a mean
ayanamsha; on a nutated one nutation adds several more, which the source
does not apply. Neither moves an office-bearer.

## 6. The five-fold strength, against the source's own table

The source tabulates the **Panchavargiya bala** of all seven planets of
that chart (Table VI-10) — five parts each, their total, and the
Vishwa bala the year lord is chosen by. Every one of those figures is
reproduced from the chart **the SDK founded**, not from the source's own
longitudes, and the arithmetic is exact: a unit holds 3600 sub-sub units
and nothing rounds. Every figure below equals the one the source prints;
all 49 of its cells — five parts, a total and a Vishwa bala for each
of the seven — are compared one by one in `crates/tajika`'s own tests,
which is where a wrong relation, table cell, truncation or division
would land.

| | Sun | Moon | Mars | Mercury | Jupiter | Venus | Saturn |
|---|---|---|---|---|---|---|---|
| the Vishwa bala | 14:20:15 | 08:52:30 | 14:01:00 | 13:00:30 | 14:46:00 | 05:40:00 | 16:47:45 |

The strongest is **Saturn**, as the source has it. The year lord is not
the strongest of the seven but the strongest of the five office-bearers.

## 7. The lord of that year

The year lord is **not** the strongest planet, nor even the strongest
office-bearer: it is the strongest office-bearer that **aspects the
annual lagna**. The source's own chart is the case that shows why the
rule needs all three parts, and the SDK reproduces its reckoning
claimant by claimant.

| claimant | Vishwa bala | portfolios | aspects the lagna |
|---|---|---|---|
| Jupiter | 14:46:00 | 1 | **no** |
| Sun | 14:20:15 | 2 | yes |
| Mars | 14:01:00 | 2 | yes |

Jupiter leads on strength and stands in the **second** from the lagna, a neutral house that gives no Tajika aspect, so the source disqualifies it in as many words. The lord of the year is **Sun** at 14:20:15, chosen as `Strongest`. Saturn is stronger than any of them at 16:47:45 and holds no portfolio, so it never enters the reckoning at all.
## 8. The aspects, and the source's worked Ithasala

A pair of planets is governed by the **mean** of their two deeptamshas,
and is coming together — **Ithasala** — when the faster of them is
behind the slower. Behind is **degrees within the sign**, the completed
signs deleted, which is the source's own instruction and the opposite of
what a longitude would say.

The source's Table X-3 gives that coming-together **three kinds**, which
§9 counts over the whole corpus; this pair is the **Vartamana**, the
present one, because the Sun is behind Mars by more than the single
degree that would make it already fulfilled.

Its worked pair is the Sun at Leo 3°50′ and Mars at Scorpio 7°42′,
three whole signs further on. Read from the chart the SDK founded:

| | measured | the source |
|---|---|---|
| the faster of the two | Sun | the Sun |
| their orb, the mean of 15° and 8° | 11.50° | 11°30′ |
| apart, within their signs | 3.87° | 3°52′ |
| what they make | IthasalaVartamana | Ithasala |

Of that chart's twenty-one pairs, **9** make a yoga and the rest make
none — most of them because they stand in the neutral houses, where no
closeness is an aspect. Only the ones that make something cross the
boundary.

## 9. The four kinds, over the recorded years

The source's Table X-3 gives the Ithasala **three** kinds and sets
Ishrafa a degree away from them. This sorts every pair of every annual
chart of every recorded birth into them — 2159 charts, 45 339 pairs,
of which 29 166 stand in signs that aspect at all — through
`sdk.chart().drishtis`, so what is counted is what the module answers.

| kind | what puts a pair there | pairs | of those that aspect |
|---|---|---|---|
| **Vartamana** | behind by a degree or more, inside the orb | 6668 | 22.9% |
| **Poorna** | behind by less than a degree | 940 | 3.2% |
| **Bhavishyat** | outside the orb, reaching from a sign's end | 322 | 1.1% |
| **Ishrafa** | past by a degree or more, inside the orb | 6565 | 22.5% |
| *the contested band* | past by less than a degree | **934** | **3.2%** |

### What turns on the last row

The last row is the one thing the source's two accounts do not settle
(crux C112), and the count is why it is carried as a reading rather than
decided quietly. Under the chapter's prose those 934 pairs are
**Ishrafa**, generally unfavourable and drawing apart. Under Table X-3
read so that its rows interlock they are **Poorna**, the most fulfilled
thing a pair can be. Under the table read at its narrowest they are
nothing at all. One band, three answers, and the three are not near each
other.

Two things about the size of it. It is 3.2% of every pair that aspects
— not a rounding margin, and about a twelfth of every Ishrafa. And it
is almost exactly the size of the **Poorna the table states outright**
beside it, 940 against 934: the two sit symmetrically either side of an
exact aspect, which is the argument for reading Poorna as covering both.
A reading on which one side of exactness is immediate fulfilment and the
other side is nothing would have to explain the asymmetry, and the book
does not.

`SubDegree` carries all three and defaults to Poorna, which is the only
reading under which the degree the table prints does any work at all.
`Between::disputed` marks the pairs, so a reader can say which
judgements are contested.

### What the sweep could not reach

Every recorded birth was asked for 40 years, which is **2200** years
over the 55 of them, and 2159 charts were read. The whole of the
difference is accounted for below, because a sweep that reported only
what it managed would get greener as the corpus got harder.

**40 years** never returned from the search at all, belonging to 1
birth: `c048-kathmandu-2399-12-30`. The built-in ephemeris's span runs
out before those births reach their fortieth year, and the SDK answers
the years it covers rather than refusing the whole request. A further
**1 year** returned but could not be **founded**, belonging to 1 birth:
`c028-troms-1988-06-21`. The SDK refuses rather than inventing a day —
*out of range: JD 2448064.4323329213 UTC is not in the local day from JD
2448098.4552060068 UTC to JD 2448099.477145168 UTC (field `instant`)*
— because a birth above the polar circle in its own summer has no
sunrise to divide a day by, and the hora and ghati a chart is built on
are measured from one. That is a documented bound of the corpus
(`05-testing/01-golden-vectors.md`, note 13) and not a fault of the
aspects.

## 10. The sixteen yogas, and the matters they answer

Fourteen of the sixteen are not facts about a chart. They are judgements
about a **pair** — the *lagnesha*, the lord of the annual lagna, and
the *karyesha*, the lord of the house the matter asked about belongs to
— so the same year answers differently for each of the twelve houses.
Every chart above is asked all twelve, which is **25 908** questions.
Ikabala and Induvara are the two exceptions: facts about a chart, so
each holds in all twelve of a chart's matters or in none — **147**
charts and **1**, a divisibility the pass checks.

| yoga | | held | of the matters asked |
|---|---|---:|---|
| **Ikabala** | built | 1764 | 6.8% |
| **Induvara** | built | 12 | 0.0% |
| **Ithasala** | built | 4250 | 16.4% |
| **Ishrafa** | built | 3050 | 11.8% |
| **Nakta** | built | 603 | 2.3% |
| **Yamaya** | built | 395 | 1.5% |
| **Manau** | built | 1756 | 6.8% |
| **Kamboola** | built | 1094 | 4.2% |
| **GairiKamboola** | built | 0 | 0.0% |
| **Khallasara** | built | 2 | 0.0% |
| **Rudda** | built | 4014 | 15.5% |
| **DuhphaliKuttha** | built | 36 | 0.1% |
| **DutthotthaDavira** | built | 4 | 0.0% |
| **Tambira** | built | 20 | 0.1% |
| **Kuttha** | built | 958 | 3.7% |
| **Durapha** | built | 4 | 0.0% |

**16 of the sixteen are built** and the other no are listed at every
call rather than left out of the answer, because *did not hold* and
*cannot be told* are different statements. `YearYogas::holds` answers
`None` for those no, never `false`.

### The first house is never a pair

**3899** of the 25 908 matters — 15.0% — have one planet for both
lords, so there is no pair to judge. That is not an edge case that crept
in: the **first** house is the lagna itself, so its lord is the lagnesha
by definition, and a question about the native's own self can never be
one of these fourteen judgements. One further house is like it under a
lagna ruled by one of the five that rule two signs, and none is under
Cancer or Leo, where the luminaries rule one each. The answer reports it
as `same_lord` rather than returning an empty list that would read as
*nothing holds*.

The count decomposes, and `cargo xtask muntha` **fails** if it ever stops decomposing, because a printed figure nobody can check is the part of a generated page that rots. Every one of the 2159 charts contributes its first house, and every chart but the 419 whose lagna a **luminary** rules contributes one more, since the Sun rules Leo alone and the Moon Cancer alone: 2159 + (2159 − 419) = **3899**.
### Why Khallasara is so rare

Khallasara and Gairi-Kamboola both need an **unqualified** Moon, which
the source defines outright: neither exalted nor debilitated, nor
aspected or associated, nor in its own Hudda, Drekkana or Navamsha.
Every clause must be false at once, and over the 2159 annual charts the
Moon managed it **once**, one chart in 2159. The clause that does the
disqualifying is not the interesting one to guess at, so it is counted:

| clause | charts |
|---|---:|
| aspected or associated by another of the seven | 2155 |
| in a Drekkana it rules | 291 |
| debilitated | 179 |
| exalted | 168 |
| in a Navamsha it rules | 164 |
| in a Hudda it rules | 0 — *and never can be* |

**The last row is a zero that had to be explained rather than printed.**
The Hudda is the Egyptian terms, which divide every sign among Mars,
Mercury, Jupiter, Venus and Saturn and give the luminaries nothing —
so for the one planet this definition is ever applied to, that clause is
**vacuous**. It is kept in the code because the source states it and a
reader comparing the two should find all six.

**The first row is almost the whole of it**, and it is structural rather
than accidental: Tajika counts **eight of the twelve** sign relations as
an aspect — only the 2nd, 6th, 8th and 12th are nothing at all — so
a Moon that nothing aspects needs all six of the others inside those
four houses at once. The source's own worked chart cannot do it at any
degree of the Moon's circle, because three of its planets share Leo and
two more sit in the signs either side, a spacing no single sign is
neutral to. That is a fact about the definition and not about this
corpus, and it is why C115 asks whether *aspected* here is narrower than
*by any of the seven*.

## 11. Strong and weak, and the floors between them

Five of the six strength yogas turn on whether a planet is **strong** or
**weak** — Rudda alone does not — and Charak never says where either
begins. He gives a figure once, for the office-bearers when he chooses
the year lord: below five units of Vishwa bala, the Muntha lord takes
the year instead. A second book grades the whole scale — under five
*Nirbali*, strengthless; five to ten *Madhya*, middling; ten to fifteen
*Poorna*, fully strong; above fifteen *Parakrami* — and the two meet
at five. So the yogas' *weak* is read as *Nirbali* and their *strong* as
*Poorna* or better, which leaves a **middling** band between that is
neither. Both floors are `YogaRules` fields, and this section measures
what each costs (crux C116).

Strength is a disjunction — "exalted, in its own house or otherwise
strong" — so a **dignified** planet is strong under any floor at all.
Of the 15 113 readings, seven to each of the 2159 charts, **3207**
(21.2%) are dignified and beyond either floor's reach.

### Where the seven stand

| Vishwa bala | readings | |
|---|---:|---|
| [0, 2) | 0 | 0.0% |
| [2, 4) | 26 | 0.2% |
| [4, 6) | 519 | 3.4% |
| [6, 8) | 1744 | 11.5% |
| [8, 10) | 3321 | 22.0% |
| [10, 12) | 3929 | 26.0% |
| [12, 14) | 3116 | 20.6% |
| [14, 16) | 1951 | 12.9% |
| [16, 18) | 488 | 3.2% |
| [18, 20] | 19 | 0.1% |

The weakest reading anywhere in the corpus is **03:45:15**, on a scale
of twenty. Under the default floors, **171** readings (1.1%) are weak,
**5280** (34.9%) middling and **9662** (63.9%) strong.

Of the 25 908 matters asked, **22 009** have two distinct lords to
judge. The two floors are **separable** — whether a planet is weak
turns only on the lower and whether it is strong only on the upper, so
long as they do not cross — so each table moves one and holds the
other at its default.

### The lower floor: where weak ends

| below | readings weak | both lords weak | of the judged |
|---:|---:|---:|---:|
| 4 | 0.2% | 0 | 0.0% |
| **5** (default) | 1.1% | 6 | 0.0% |
| 6 | 3.6% | 44 | 0.2% |
| 8 | 15.1% | 567 | 2.6% |
| 10 | 36.1% | 2871 | 13.0% |

### The upper floor: where strong begins

| from | readings strong | both lords strong | of the judged |
|---:|---:|---:|---:|
| 5 | 98.9% | 21 499 | 97.7% |
| 8 | 84.9% | 15 824 | 71.9% |
| **10** (default) | 63.9% | 9042 | 41.1% |
| 12 | 41.0% | 3727 | 16.9% |
| 15 | 23.6% | 1294 | 5.9% |
| 20 | 21.2% | 1055 | 4.8% |

**At the default floors, both lords are weak in 6 of the 22 009 judged
matters (0.0%), both strong in 9042 (41.1%), and the other 12 961 are
mixed.** The first is a **ceiling**, not a count of anything that held:
Dutthottha-Davira and Durapha both require a weak pair before asking
anything else, so neither can hold in more matters than it allows.
Dutthottha-Davira held in **4**. Kuttha, which wants both strong, has
the second to work with.

Five things hold of both tables, and `cargo xtask muntha` **fails** if
any stops holding, because each is a fact about `Strength` and not about
this corpus: under the default floors the readings **partition** into
weak, middling and strong, and the judged matters into both weak, both
strong and mixed, each part counted on its own rather than by
difference; raising the lower floor never makes a planet less weak, nor
raising the upper one more strong; no upper floor falls below the 3207
dignified readings; where the two floors are **equal** there is no
middle, so the lower table's row at five and the upper table's meet
exactly — the one check that ties the two tables to each other; and no
yoga that needs a weak pair holds in more matters than had one.

## 12. What spoils an Ithasala

Rudda is the Ithasala spoilt: one where either of the pair is
"retrograde, combust, debilitated, in the 6th, 8th or 12th, or under
malefic influence". Retrograde and combustion are not in a chart's
longitudes, so they are read from its graha states — over the corpus's
15 113 readings, **2209** are retrograde and **2051** combust, under the
`BPHS` combustion table the profile sets.

Rudda held in **4014** of the 4250 matters in which an Ithasala stood
(94.4%). Each clause, counted in the Ruddas where it held of either lord
— so a Rudda with two afflictions is counted twice:

| clause | Ruddas | of them |
|---|---:|---|
| under malefic influence | 3315 | 82.6% |
| in the 6th, 8th or 12th | 1622 | 40.4% |
| retrograde | 1201 | 29.9% |
| combust | 1169 | 29.1% |
| debilitated | 623 | 15.5% |

### The reach of the two readings left open

*Under malefic influence* is read as Manau reads it — joined, or
aspected inimically, by Mars or Saturn — with the pair's own malefic
counting against its partner (crux C118). **873** Ruddas (21.7%) held on
that clause and nothing else. That bounds every **narrower** reading at
once: excluding the partner, or any stricter sense of influence, can
remove those and no other. A **wider** one — any aspect at all —
could only add Ruddas, and at most the **236** Ithasalas not spoilt now.

**Read literally, Rudda spoils almost every Ithasala**, and no reading
of that one clause changes it: the narrowest would still leave **3141**
of the 4250 spoilt (73.9%). The breadth is the list itself — any one
of five common afflictions, on either of two planets — which is the
mirror of *unqualified* (crux C115), read so strictly that Khallasara
almost never holds. It is recorded rather than corrected, because no
text in reach narrows the list; every Rudda carries both lords' clauses,
so a reader who holds a narrower reading can apply it without the yoga
being rewritten.

Duhphali-kuttha held in **36**, and Durapha in **4**. Durapha's list is
read as alternatives each lord must meet one of (crux C119); whatever
the reading, it needs a weak pair first, and §11 puts that ceiling at
**6**. `cargo xtask muntha` fails if either ceiling is ever exceeded:
every judgement upon an Ithasala is held under the Ithasala's own count,
and every yoga needing a weak pair under the weak pairs', each read from
`YearYoga` itself.

## 13. What happens next: Gairi-Kamboola and Tambira

Two of the sixteen ask where a planet at a sign's end will stand **on
entering the next**. The module answers by moving that one planet to the
next sign's first degree, the other six where they are, and asking the
same Ithasala question of the sky that leaves (crux C120). The source's
worked Gairi-Kamboola, Chart X-17, comes out as printed under it. Both
counts sit under chains of ceilings, each step a subset of the one
before, and `cargo xtask muntha` fails if any step exceeds the one above
it.

| step | Gairi-Kamboola | step | Tambira |
|---|---:|---|---:|
| an Ithasala | 4250 | the pair do not aspect | 7938 |
| the Moon, not one of the pair, at a sign's end | 107 | the karyesha at a sign's end | 277 |
| that Moon unqualified | 0 | and not retrograde | 215 |
| **held** | **0** | **held** | **20** |

**The step from the second row to the third is the source's
*unqualified*,** which §10 found the Moon meeting in 1 of 2159 charts.
Whatever Gairi-Kamboola loses there is C115's to move, not this yoga's:
narrowing that one reading is what would widen both it and Khallasara.
Where it holds, Khallasara does not, though every printed clause of
Khallasara may: the source's own comment excludes a Moon at a sign's end
from it, because that Moon completes the Ithasala rather than standing
apart (crux C121).

**Tambira's own readings move it.** Letting either lord be the one at a
sign's end (the source's "some authorities", `TambiraMover::EitherLord`)
holds it in **31** matters to the definition's 20. A retrograde karyesha
is going back, not on, so it enters nothing, and a call without the
chart's states cannot answer for Tambira at all.

## 14. The sahams, and what each reading moves

A saham is **a − b + c**, carried one sign further when c does not
fall between b and a counted from b (`03-design/tajika-sahams.md`). The
source gives forty-one, and each is read here in every chart under its
readings and under each rival, through `sdk.chart().sahams_with_rules`.
1094 of the 2159 charts open by day and read the day formulas.

**Two identities hold, and the pass fails if either does not:** a saham
reading no other moves when no sign is ever added exactly as often as it
took one, and a saham that reads no house never moves under equal houses
— Samarthya and Manmatha among them, which read only the lagna's lord.
The five that read another saham also move with it: with no sign ever
added they move in 6756 placements to the 5322 in which they took a sign
themselves, because Punya, Guru and Vidya lose theirs. Over all
forty-one, 40 628 of 88 519 placements took the sign. The three pairs
the source gives one formula each — Vidya and Guru, Raja and Pitri,
Kshama and Kali — agree in every column.

**Counting "between" in whole signs moves 11 790 placements in all
(13.3%).** That is the reading a widely used program applies; the
source's own birth-chart Punya refutes it, the Sun, lagna and Moon all
in Leo. **Equal houses from the lagna put 23.0%** of the placements of
the five sahams that read a house past the first in another sign than
Sripati's mid-points do, which the source builds from the lagna and the
midheaven and prints. This profile's own chalit is Vehlow's, equal
houses centred on the lagna, which is why the default reads Sripati's
from the angles and not the chart's chalit: read off the chalit, this
column was zero. **Roga's second reading** stands in another sign in
**1957** of 2159 charts.

| saham | took the added sign | another sign, whole signs | another sign, equal houses |
|---|---:|---:|---:|
| 1 Punya | 26.7% | 11.9% | no house past the first |
| 2 Guru | 73.3% | 11.5% | no house past the first |
| 3 Vidya | 73.3% | 11.5% | no house past the first |
| 4 Yasha | 50.7% | 21.2% | no house past the first |
| 5 Mitra | 50.0% | 25.6% | no house past the first |
| 6 Mahatmya | 43.4% | 22.0% | no house past the first |
| 7 Asha | 34.8% | 12.2% | no house past the first |
| 8 Samarthya | 39.1% | 13.0% | no house past the first |
| 9 Bhratri | 50.5% | 10.7% | no house past the first |
| 10 Gaurava | 50.5% | 11.9% | no house past the first |
| 11 Pitri | 27.7% | 11.7% | no house past the first |
| 12 Raja | 27.7% | 11.7% | no house past the first |
| 13 Matri | 34.1% | 12.4% | no house past the first |
| 14 Putra | 50.3% | 12.6% | no house past the first |
| 15 Jeeva | 49.0% | 10.7% | no house past the first |
| 16 Roga | 0.0% | 7.8% | no house past the first |
| 17 Karma | 35.9% | 14.5% | no house past the first |
| 18 Manmatha | 34.8% | 11.3% | no house past the first |
| 19 Kali | 45.3% | 12.4% | no house past the first |
| 20 Kshama | 45.3% | 12.4% | no house past the first |
| 21 Shastra | 50.3% | 12.8% | no house past the first |
| 22 Bandhu | 52.0% | 12.2% | no house past the first |
| 23 Mrityu | 50.6% | 11.8% | 13.7% |
| 24 Deshantara | 69.2% | 8.7% | 33.1% |
| 25 Artha | 8.6% | 6.9% | 19.5% |
| 26 Paradara | 50.4% | 16.4% | no house past the first |
| 27 AnyaKarma | 50.1% | 11.9% | no house past the first |
| 28 Vanika | 48.0% | 11.3% | no house past the first |
| 29 KaryaSiddhi | 37.9% | 21.5% | no house past the first |
| 30 Vivaha | 50.3% | 12.0% | no house past the first |
| 31 Prasava | 32.2% | 13.2% | no house past the first |
| 32 Santaapa | 49.2% | 12.4% | 14.0% |
| 33 Shraddha | 50.2% | 14.4% | no house past the first |
| 34 Preeti | 52.3% | 17.3% | no house past the first |
| 35 Jadya | 51.9% | 13.7% | no house past the first |
| 36 Vyapara | 49.7% | 15.0% | no house past the first |
| 37 PaneeyaPaata | 49.9% | 12.3% | no house past the first |
| 38 Shatru | 56.3% | 11.4% | no house past the first |
| 39 Jalapatha | 45.8% | 12.5% | no house past the first |
| 40 Bandhana | 50.1% | 21.6% | no house past the first |
| 41 Labha | 84.4% | 8.0% | 34.6% |

## 15. The Harsha bala

Four places a planet is "happy" in, five units each: its house of joy,
its exaltation or own sign, a house of its own gender, and the year's
own part of the day (`03-design/tajika-harsha.md`). Read in every chart
through `sdk.chart().harsha`.

**The source says fifteen is generally the most a planet reaches and
twenty rather unusual.** Over 15 113 placements of the seven, 15 051
hold fifteen or less and **62** hold all four (0.4%).

**Venus's rival place moves Venus alone, and the pass fails if it moves
anything else.** Read as a widely used program reads it, in the 12th
rather than the verse's 5th, Venus holds another total in **347** of
2159 charts (16.1%). The day part decomposes too: the Sun holds it in
exactly the 1094 charts read by day, the Moon in exactly the 1065 read
by night.

**Sun, Venus and Saturn never hold all four, and the pass fails if one
does or if any other never does.** Each one's house of joy is of the
other gender — the Sun's 9th is feminine, Venus's 5th and Saturn's
12th masculine — so the place and the gender parts exclude each other,
and their twenties are zeros by the rule rather than by chance.

| planet | Nirbala, 0 | Alpabali, 5 | Madhya Bali, 10 | Poorna Bali, 15 | twenty |
|---|---:|---:|---:|---:|---:|
| Sun | 19.8% | 40.7% | 33.4% | 6.1% | 0.0% |
| Moon | 21.6% | 41.5% | 28.5% | 8.1% | 0.3% |
| Mars | 19.0% | 41.1% | 29.5% | 9.5% | 0.9% |
| Mercury | 21.1% | 44.5% | 26.7% | 7.4% | 0.3% |
| Jupiter | 18.8% | 39.7% | 30.3% | 9.9% | 1.3% |
| Venus | 18.6% | 38.7% | 34.0% | 8.7% | 0.0% |
| Saturn | 15.4% | 42.8% | 34.7% | 7.1% | 0.0% |

## 16. A saham's strength

The source lists what makes a saham strong and what makes it weak, and
judges its worked sahams in words, never by a score
(`03-design/tajika-saham-strength.md`). Every one of the forty-one is
read in every chart through `sdk.chart().saham_strength`, under the
year's own lord.

**The two lists are not two verdicts.** Of 88 519 placements, 32 209
meet a strong clause and no weak one, 1805 a weak clause and no strong
one, and **54 505 meet clauses on both lists** (61.6%). A score would
have to weigh one list against the other, which is a rule the source
does not state; the report gives the clauses. **None meets neither, and
the pass fails if one does**: the strong list's "its lord aspects or
conjoins it" and the weak list's "neither aspects nor conjoins it" are
each other's negation, so every saham meets one of them.

**Under positional friendship a saham whose lord conjoins it never has a
friend for company**, since two planets in one sign are enemies there,
and the pass fails if one does.

| list | clause | holds |
|---|---|---:|
| strong | its lord is exalted | 7.9% |
| strong | its lord is in its own sign | 15.4% |
| strong | its lord is in its own Hudda | 16.8% |
| strong | its lord is in its own Drekkana | 13.9% |
| strong | its lord is in its own Navamsha | 15.4% |
| strong | its lord is in a friend's sign | 28.5% |
| strong | it is with a friend of its lord | 12.7% |
| strong | it is with a natural benefic | 28.9% |
| strong | it is with the year lord | 8.9% |
| strong | its lord conjoins it | 8.6% |
| strong | its lord aspects it | 67.9% |
| strong | its lord aspects the lagna | 66.8% |
| weak | its lord is under the Panchavargiya floor | 1.1% |
| weak | its lord has no Harsha bala | 18.8% |
| weak | its lord neither aspects nor conjoins it | 32.1% |
| weak | it is with an enemy of its lord | 13.0% |
| weak | it is with a natural malefic | 23.9% |

## 17. Kuttha

Kuttha wants both lords powerful, in a kendra or a panaphara, under a
benefic's aspect and no malefic's (`03-design/tajika-yogas.md`, crux
C117). It held in **958** of the 22 009 judged matters (4.4%). It can
hold only where both lords are strong, and the default floors find
**9042** such matters, so 8084 of them were turned away by a clause
other than strength.

**Which clause turns a strong pair away.** A clause fails a matter where
it fails of either lord. The last column counts the matters it turned
away alone, the near misses: one clause read otherwise and the Kuttha
would have held. *Powerful* reads zero by construction, since only
strong pairs are counted, and the pass fails if it does not.

| clause | turned away | alone |
|---|---:|---:|
| powerful | 0 | 0 |
| in a kendra or a panaphara | 5116 | 1397 |
| under a benefic's aspect | 628 | 50 |
| under no malefic's aspect | 6555 | 2708 |

**The commentary's full Moon** (`moon_benefic: waxing`) holds Kuttha in
**897** matters to the default's 958. It can only remove a benefic, so
it can never add a Kuttha. The pass asks it of every strong pair and
fails if it adds one. It also fails if `favour` and the yoga disagree
about a single matter, or if Kuttha's strong pairs differ from the
floors' own count.
