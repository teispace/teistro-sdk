# What the catalogue names and what this build computes

Status: `generated` by `cargo xtask dasha-coverage` over
`DashaSystem::ALL`, `teistro::dasha::systems()` and
`teistro::tajika::ANNUAL_DASHAS`. Do not edit: `check-dasha-coverage`
regenerates this page and fails on any difference. The design it
measures is [`dasha-kernels.md`](dasha-kernels.md).

The catalogue names 40 dasha systems and this build computes 24. The 16
left are **not** a backlog of unwritten code: every one of them is
blocked on something that is not typing, and this page is the list of
what, grouped by the blocker that would have to go first.

A catalogue is a **key space** and a build is a set of kernels, so the
two were never going to be the same size ([Q38](../QUESTIONS.md)). What
would be a defect is the gap going **undeclared**, because then a
consumer reading the catalogue cannot tell a feature from a name waiting
for one. It is declared twice over: named here, and refused at the call.

Of the computed, 3 divide **one year** rather than a life: `PATYAYINI`,
`MUDDA` and `VARSHA_YOGINI`. They are computed from an annual chart by
`sdk.chart().annual_dasha` ([`annual-dashas.md`](annual-dashas.md)), so
a natal chart asked for one is refused, and 3 of those refusals name
that call in the hint rather than leaving the caller at a list of the
natal systems.

## 10 systems: the text is not settled

What closes them is a cited source read by someone who reads it.
Building is not what is missing: every row here is one line of table
data once the verse settles it, which is why none of them is scheduled
and all of them are listed.

| system | why it is not computed |
|---|---|
| `SHODASHOTTARI` | the row is stated — from Pushya, eight lords, 116 years — and its verse numbers in BPHS ch. 46 are to be confirmed before it ships |
| `SHATTRIMSHA_SAMA` | the same verse numbers to confirm. Its table is Yogini's, lord for lord and year for year, differing only in the reference nakshatra and the offset — so shipping it on a guess would ship Yogini twice |
| `TITHI_ASHTOTTARI` | Ashtottari's table seeded by the **tithi**. The reference index does not carry over from the nakshatra rows because the cycles differ, 30 against 27, so the seat is unknown |
| `TITHI_YOGINI` | Yogini's table on the same thirty-fold cycle, blocked by the same unknown seat |
| `YOGA_VIMSHOTTARI` | Vimshottari's table seeded by the **yoga**, on a twenty-seven-fold cycle whose reference is not the nakshatra's |
| `KARANA_CHATURASHITI` | Chaturashiti-sama's table seeded by the **karana**, on a sixty-fold cycle |
| `NAISARGIKA` | the natural order and the lifespan periods it divides are both unsettled here, which is the whole of the row |
| `PANCHASWARA` | no attested shape at all. It is catalogued because a system by that name is named, which is what a key space is for |
| `VARNADA` | the Varnada lagna it starts from **is** built (`teistro_points`), so what is missing is not the point but which of five school variants of it the dasha counts from |
| `VARSHA_NARAYANA` | Narayana read over one year. The solar return it waited on is built and the three annual dashas beside it are computed (`annual-dashas.md`), but neither book read for them gives this one: Charak's chapter V and the *Tajika Nilakanthi* name the Mudda, the Yogini and the Patyayini and stop |

## 4 systems: the kernel cannot express it

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
| `SUDARSHANA_CHAKRA` | three rashi progressions running at once, from the lagna, the Sun and the Moon. A composition over kernels, and the reason it is a combinator |

## 2 systems: it waits on a module

What closes them is the module itself. Each is the **last** step of one,
so scheduling the dasha separately would schedule the module twice.

| system | why it is not computed |
|---|---|
| `AAYU` | the longevity module decides the span this divides, and that span is three methods with a reconciliation between them (`crates/rules` `longevity`). The dasha is what the module ends with |
| `SUDASA` | it starts from the karakamsha, which is the navamsha of the Atmakaraka and is not built — the Sree lagna beside it in the sources is. Karakamsha belongs to the Jaimini module and this follows it |

## Asking for one

Every one of the 16 unbuilt systems was asked of a real founded chart.
None of them answered: each came back refused, naming the system asked
for and hinting with every system this build does compute. That is what
makes the gap a declared one rather than a dead end — a member that
answered an empty reading would be indistinguishable from a bug, and
nothing but a call can tell the two apart.

```text
unsupported: SHODASHOTTARI is a dasha the catalogue names and this build does not compute yet (field `dashas[0]`); the dashas built are VIMSHOTTARI, ASHTOTTARI, DWADASHOTTARI, PANCHOTTARI, SHATABDIKA, CHATURASHITI_SAMA, DWISAPTATI_SAMA, YOGINI, TRIBHAGI, SHASHTIHAYANI, CHARA, NARAYANA, PADANADHAMSA, TRIKONA, DRIG, SHOOLA, NIRYANA_SHOOLA, MANDOOKA, STHIRA, YOGARDHA, KALACHAKRA
```

## Who can supply one

"Not built" is not "not available". `DashaSystems::register` takes a
`DashaDefinition` of either kernel — lords, years and a nakshatra
reference, or where a system starts, the order it visits the signs in
and how long a sign runs — each checked by the same row validation a
shipped system passes. So a consumer holding the text registers the
system on their context and asks for it by key, today, with no change
here. That covers 3 of the 16 systems left.

| system | the kernel it arrives as | what is still missing |
|---|---|---|
| `SHODASHOTTARI` | nakshatra-seeded | the row is stated — from Pushya, eight lords, 116 years — and its verse numbers in BPHS ch. 46 are to be confirmed before it ships |
| `SHATTRIMSHA_SAMA` | nakshatra-seeded | the same verse numbers to confirm. Its table is Yogini's, lord for lord and year for year, differing only in the reference nakshatra and the offset — so shipping it on a guess would ship Yogini twice |
| `VARNADA` | sign-based | the Varnada lagna it starts from **is** built (`teistro_points`), so what is missing is not the point but which of five school variants of it the dasha counts from |

The path is walked once per kernel rather than cited:
`DEMO_SHODASHOTTARI`, 8 lords and 116 years from Pushya, the
nakshatra-seeded kernel and `DEMO_RASHI`, every sign from the lagna for
seven, eight or nine years by modality, the sign-based kernel.

**The other 13 cannot be supplied by anyone**, and each for a reason in
its own row rather than for want of an arm: `SUDASA` starts from the
karakamsha, which is a place `Start` does not name; the tithi, yoga and
karana seeds want a reference that is not a nakshatra; `TARA`, `KARAKA`
and `ASHTAKAVARGA` ask the chart for their periods; `SUDARSHANA_CHAKRA`
is a composition of systems rather than a system. Those are rows the
kernels do not express, which is a different thing from a row nobody has
written down — and the difference is what this section exists to keep
visible.

## What the types decide

| proposed rule | verdict | measured |
|---|---|---|
| every catalogued system this build does not compute is listed here with a reason | **holds** | 0 of 16 disagree |
| no reason here outlives its blocker: nothing listed is already computed | **holds** | 0 of 16 disagree |
| every reason names a system the catalogue names | **holds** | 0 of 16 disagree |
| every system said to be registrable is one this build does not compute | **holds** | 0 of 3 disagree |
| asking for an unbuilt system is refused and never answered | **holds** | 0 of 16 disagree |
| the refusal names the system asked for | **holds** | 0 of 16 disagree |
| the refusal names every system this build does compute | **holds** | 0 of 16 disagree |
| the counts on this page are read from the types and not written down | **holds** | `DashaSystem::ALL` 40, `teistro::dasha::systems()` 24 |

