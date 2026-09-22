# What the catalogue names and what this build computes

Status: `generated` by `cargo xtask dasha-coverage` over
`DashaSystem::ALL` and `teistro::dasha::systems()`. Do not edit:
`check-dasha-coverage` regenerates this page and fails on any
difference. The design it measures is
[`dasha-kernels.md`](dasha-kernels.md).

The catalogue names 40 dasha systems and this build computes 18. The 22
left are **not** a backlog of unwritten code: every one of them is
blocked on something that is not typing, and this page is the list of
what, grouped by the blocker that would have to go first.

A catalogue is a **key space** and a build is a set of kernels, so the
two were never going to be the same size ([Q38](../QUESTIONS.md)). What
would be a defect is the gap going **undeclared**, because then a
consumer reading the catalogue cannot tell a feature from a name waiting
for one. It is declared twice over: named here, and refused at the call.

## 11 systems: the text is not settled

What closes them is a cited source read by someone who reads it.
Building is not what is missing: every row here is one line of table
data once the verse settles it, which is why none of them is scheduled
and all of them are listed.

| system | why it is not computed |
|---|---|
| `SHODASHOTTARI` | the row is stated — from Pushya, eight lords, 116 years — and its verse numbers in BPHS ch. 46 are to be confirmed before it ships |
| `SHATTRIMSHA_SAMA` | the same verse numbers to confirm. Its table is Yogini's, lord for lord and year for year, differing only in the reference nakshatra and the offset — so shipping it on a guess would ship Yogini twice |
| `SHASHTIHAYANI` | the received text gives Jupiter 13, Sun 13, Mars 13 and then six each, which sums to 69 and not to the 60 the name states. A row cannot be written from a text that disagrees with itself |
| `TITHI_ASHTOTTARI` | Ashtottari's table seeded by the **tithi**. The reference index does not carry over from the nakshatra rows because the cycles differ, 30 against 27, so the seat is unknown |
| `TITHI_YOGINI` | Yogini's table on the same thirty-fold cycle, blocked by the same unknown seat |
| `YOGA_VIMSHOTTARI` | Vimshottari's table seeded by the **yoga**, on a twenty-seven-fold cycle whose reference is not the nakshatra's |
| `KARANA_CHATURASHITI` | Chaturashiti-sama's table seeded by the **karana**, on a sixty-fold cycle |
| `NAISARGIKA` | the natural order and the lifespan periods it divides are both unsettled here, which is the whole of the row |
| `PANCHASWARA` | no attested shape at all. It is catalogued because a system by that name is named, which is what a key space is for |
| `STHIRA` | the row is stated — from the lagna, consecutive, seven, eight or nine years by modality — and nothing here verifies it. It is one boolean from Mandooka's row, which is exactly why a guess would go unnoticed |
| `VARNADA` | the Varnada lagna it starts from **is** built (`teistro_points`), so what is missing is not the point but which of five school variants of it the dasha counts from |

## 5 systems: the kernel cannot express it

What closes them is a change to the kernel, which is the decision
ADR-0017 set a kill criterion for: a third chart-query field serves
three of these at once, or the kernel is redesigned as an interpreter
and serves them all. They are taken together for that reason, never one
at a time.

| system | why it is not computed |
|---|---|
| `TARA` | its periods come from the chart's tara counts, and the row schema states its periods. A row that asks the chart is a field K-udu has not got |
| `KARAKA` | its lord **order** comes from chara-karaka strength, which is a second thing a row would have to ask the chart for |
| `ASHTAKAVARGA` | its periods come from the bindu counts. The three above are one mechanism asked for three ways, and the kernel takes them together or not at all |
| `YOGARDHA` | the mean of two systems, if that is what it is. A composition over kernels is not an algorithm inside one |
| `SUDARSHANA_CHAKRA` | three rashi progressions running at once, from the lagna, the Sun and the Moon. The same composition, and the reason it is a combinator |

## 6 systems: it waits on a module

What closes them is the module itself. Each is the **last** step of one,
so scheduling the dasha separately would schedule the module twice.

| system | why it is not computed |
|---|---|
| `AAYU` | the longevity module decides the span this divides, and that span is three methods with a reconciliation between them (`crates/rules` `longevity`). The dasha is what the module ends with |
| `SUDASA` | it starts from the karakamsha, which is the navamsha of the Atmakaraka and is not built — the Sree lagna beside it in the sources is. Karakamsha belongs to the Jaimini module and this follows it |
| `MUDDA` | Vimshottari scaled to the year, which needs the **annual chart** — the Varsha Pravesha solar return. The scale decorator it would use is built and Tribhagi uses it |
| `VARSHA_NARAYANA` | Narayana over the annual chart, blocked on the same solar return |
| `VARSHA_YOGINI` | Yogini over the annual chart, blocked on the same solar return |
| `PATYAYINI` | periods from the grahas' strengths **in the annual chart**, so it needs a kernel of its own as well. Filed under the module because finishing the kernel would not unblock it |

## Asking for one

Every one of the 22 unbuilt systems was asked of a real founded chart.
None of them answered: each came back refused, naming the system asked
for and hinting with every system this build does compute. That is what
makes the gap a declared one rather than a dead end — a member that
answered an empty reading would be indistinguishable from a bug, and
nothing but a call can tell the two apart.

```text
unsupported: SHODASHOTTARI is a dasha the catalogue names and this build does not compute yet (field `dashas[0]`); the dashas built are VIMSHOTTARI, ASHTOTTARI, DWADASHOTTARI, PANCHOTTARI, SHATABDIKA, CHATURASHITI_SAMA, DWISAPTATI_SAMA, YOGINI, TRIBHAGI, CHARA, NARAYANA, PADANADHAMSA, TRIKONA, DRIG, SHOOLA, NIRYANA_SHOOLA, MANDOOKA, KALACHAKRA
```

## Who can supply one

"Not built" is not "not available". `DashaSystems::register` takes a
`UduDefinition` — lords, years and a **nakshatra** reference, checked
by the same `UduRow::validate` a shipped row passes — so a consumer
holding the text can register the system on their context and ask for it
by key, today, with no change here. That covers 3 of the 22 systems
left: `SHODASHOTTARI`, `SHATTRIMSHA_SAMA` and `SHASHTIHAYANI`, each of
which is a stated row waiting only on its citation.

The path is walked rather than cited: `DEMO_SHODASHOTTARI`, 8 lords and
116 years, registered and accepted, with the total the design page
states for it falling out of the lords rather than copied beside them.

**The other 19 cannot be supplied by anyone, and that is the finding.**
The registry takes nakshatra-seeded rows and nothing else, so a
sign-based system a consumer has the text for — `STHIRA`, `VARNADA`
— has no definition to arrive as, and neither has a tithi, yoga or
karana seed. Under the no-dead-ends mandate that is a gap in the SDK and
not in the sources: the text being unsettled blocks *this* build, while
a missing definition blocks *everyone*. A `RashiDefinition` beside
`UduDefinition` is what would close it.

## What the types decide

| proposed rule | verdict | measured |
|---|---|---|
| every catalogued system this build does not compute is listed here with a reason | **holds** | 0 of 22 disagree |
| no reason here outlives its blocker: nothing listed is already computed | **holds** | 0 of 22 disagree |
| every reason names a system the catalogue names | **holds** | 0 of 22 disagree |
| every system said to be registrable is one this build does not compute | **holds** | 0 of 3 disagree |
| asking for an unbuilt system is refused and never answered | **holds** | 0 of 22 disagree |
| the refusal names the system asked for | **holds** | 0 of 22 disagree |
| the refusal names every system this build does compute | **holds** | 0 of 22 disagree |
| the counts on this page are read from the types and not written down | **holds** | `DashaSystem::ALL` 40, `teistro::dasha::systems()` 18 |

