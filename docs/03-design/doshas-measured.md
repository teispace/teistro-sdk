# The doshas, measured

Status: `generated` by `cargo xtask doshas` over the conformance
corpus's `baseline/doshas`, 2026-09-16. Do not edit: `check-doshas`
regenerates this page and fails on any difference. The design it
measures is [`rules-engine.md`](rules-engine.md). The pass evaluates the
engine's own rules over every recorded chart under a reading, and the
SDK's rules for the ones it computes in code against what that code
recorded.

## What the corpus holds

52 rules over 93 charts, 77 of them with a recorded panchanga: 3255 decisions of the 35 rules the engine evaluates from their conditions, 885 of them present. The other 17 it computes in code: `KALSARPA`, `DAGDHA_RASHI_DOSHA`, `MRITYU_BHAGA_DOSHA`, `KALSARPA_ANANT`, `KALSARPA_KULIK`, `KALSARPA_VASUKI`, `KALSARPA_SHANKHPAL`, `KALSARPA_PADMA`, `KALSARPA_MAHAPADMA`, `KALSARPA_TAKSHAK`, `KALSARPA_KARKOTAK`, `KALSARPA_SHANKHACHUD`, `KALSARPA_GHATAK`, `KALSARPA_VISHDHAR`, `KALSARPA_SHESHNAG`, `KALA_AMRITA_YOGA`, `BADHAKA_DOSHA`.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| the engine's dosha reading, every choice below as its dosha evaluator makes it | **holds** | 0 of 3255 disagree; 0 decisions, and of 885 presences 0 found from, 0 planets, 0 houses, 0 severities, 0 cancellations, 0 statuses |
| an aspect condition involving the two bodies, as the engine's yoga evaluator has it | untested | moves none of the 3255 decisions or 885 presences |
| a table's degree the ordinal degree, n − 1° to n°, rather than a degree either side | untested | moves none of the 3255 decisions or 885 presences |
| a rule asking for exaltation or debilitation met by that dignity alone, not its deep form | untested | moves none of the 3255 decisions or 885 presences |
| houses counted whole-sign from the lagna rather than recorded | untested | moves none of the 3255 decisions or 885 presences |
| an involved planet only from the branch that decided | untested | moves none of the 3255 decisions or 885 presences |
| an unqualified conjunction within 10° rather than in one sign | falsified | 131 of 3255 disagree; 40 decisions, and of 854 presences 19 found from, 19 planets, 3 houses, 17 severities, 26 cancellations, 7 statuses; 12 rules move (ANGARAK_DOSHA, CHANDAL_DOSHA, CHANDRA_DOSHA, GRAHAN_DOSHA, …) |

## The seventeen, written as rules

`crates/rules/rules/computed-doshas.json` says in the language what the
engine says in code, each rule carrying the citation and the severity
the engine's own rule declares. Measured against what its code recorded:

| rule | decisions | presences | what parts |
|---|---|---|---|
| `KALSARPA` | 0 of 93 | 2 | the planets on 2 presences, the houses on 2 presences |
| `KALSARPA_ANANT` | 0 of 93 | 1 | the planets on 1 presence, the houses on 1 presence |
| `KALSARPA_KULIK` | 0 of 93 | 0 | nothing |
| `KALSARPA_VASUKI` | 0 of 93 | 0 | nothing |
| `KALSARPA_SHANKHPAL` | 0 of 93 | 0 | nothing |
| `KALSARPA_PADMA` | 0 of 93 | 0 | nothing |
| `KALSARPA_MAHAPADMA` | 0 of 93 | 0 | nothing |
| `KALSARPA_TAKSHAK` | 0 of 93 | 0 | nothing |
| `KALSARPA_KARKOTAK` | 0 of 93 | 0 | nothing |
| `KALSARPA_SHANKHACHUD` | 0 of 93 | 0 | nothing |
| `KALSARPA_GHATAK` | 0 of 93 | 0 | nothing |
| `KALSARPA_VISHDHAR` | 0 of 93 | 0 | nothing |
| `KALSARPA_SHESHNAG` | 0 of 93 | 1 | the planets on 1 presence, the houses on 1 presence |
| `KALA_AMRITA_YOGA` | 0 of 93 | 0 | nothing |
| `MRITYU_BHAGA_DOSHA` | 0 of 93 | 48 | nothing |
| `DAGDHA_RASHI_DOSHA` | 0 of 93 | 62 | nothing |
| `BADHAKA_DOSHA` | 0 of 93 | 25 | nothing |

The Kalsarpa family parts from the engine in one way, deliberately: its
code names the two nodes as the graha involved, and these rules name the
seven grahas the nodes caught, whose houses follow.

### What a reading moves in them

Their presences, severities, cancellations and statuses; the planets the
Kalsarpa family names are left out, as the row above says they part on
purpose.

| proposed rule | verdict | measured |
|---|---|---|
| the engine's dosha reading, its table degree a degree either side | **holds** | 0 of 1581 disagree; 0 decisions, and of 139 presences 0 found from, 0 planets, 0 houses, 0 severities, 0 cancellations, 0 statuses |
| an aspect condition involving the two bodies, as the engine's yoga evaluator has it | untested | moves none of the 1581 decisions or 139 presences |
| a table's degree the ordinal degree, n − 1° to n°, rather than a degree either side | falsified | 18 of 1581 disagree; 6 decisions, and of 133 presences 0 found from, 0 planets, 0 houses, 12 severities, 0 cancellations, 0 statuses; 1 rule move (MRITYU_BHAGA_DOSHA) |
| a rule asking for exaltation or debilitation met by that dignity alone, not its deep form | untested | moves none of the 1581 decisions or 139 presences |
| houses counted whole-sign from the lagna rather than recorded | untested | moves none of the 1581 decisions or 139 presences |
| an involved planet only from the branch that decided | untested | moves none of the 1581 decisions or 139 presences |
| an unqualified conjunction within 10° rather than in one sign | untested | moves none of the 1581 decisions or 139 presences |

## What it means for the kernel

**The language says what the engine's code said.** All 17 rules it
computes in code decide presence exactly as that code did on every
recorded chart, and Mrityu Bhaga, Dagdha Rashi and Badhaka reproduce
every recorded field as well: their planets, houses, severities,
cancellations and statuses. What the language needed for them was the
arc's side named in the rule, a badhaka reference, and a weight on a
group so the luminaries and the lagna count double in a Mrityu Bhaga
severity.

**The corpus decides one reading and cannot see five.** Conjunction in
one sign against a 10° orb moves 131 answers over twelve rules; the
other five move nothing here, as they move nothing in the yogas. The
degree a Mrityu Bhaga table names is the reading this corpus does see
(crux C82), and the rows above say what it moves.