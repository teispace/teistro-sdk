# `teistro-rules`

Status: `building`, 2026-09-15: the kernel's first slice, the condition
language over grahas and the lagna, built and held to every recorded yoga;
then references, the trace, cited tables, and the doshas' groups, severities
and cancellations, the last seventeen of them written as rules. The
design is [`docs/03-design/rules-engine.md`](../../docs/03-design/rules-engine.md),
measured in [`docs/03-design/yogas-measured.md`](../../docs/03-design/yogas-measured.md).

The rules kernel: a rule's conditions, read strictly, evaluated over a chart
to whether it is present, which bodies took part, in which houses, and which
cancellations held.

| module | what it settles |
|---|---|
| [`reference`](src/reference.rs) | what a condition is about, as two types: a `BodyRef` (a graha or the lagna, the lord of a sign, the holder of a karaka) and a `SignRef` (any body's sign, a house, an arudha pada, the upapada, a navamsha, a sign counted from another), so a rule asking for the dignity of a pada is refused when read |
| [`rule`](src/rule.rs) | a rule: its conditions, the reference groups at least one of which must hold, its cancellations with their labels, its severity and its cancellation threshold, one shape for yogas and doshas, in the engine's own format |
| [`language`](src/language.rs) | the condition language, typed: three combinators and 22 predicates in the recording engine's own shape, so its rules read unchanged, and anything else (an unknown field or predicate, a thirteenth house, an outer planet) refused with its path |
| [`chart`](src/chart.rs) | what a rule reads of a chart (each body's longitude, sign, house, dignity, motion, combustion, chara karakas and navamsha), and every place the language leaves a meaning open, each a `Readings` field with the recording engine's choice the default and the text's where the engine has none |
| [`eval`](src/eval.rs) | the `Evaluator`: benefics and malefics settled once a chart, then each rule to a `RuleResult` without allocating for its participants, or to an `Explanation` with its trace |
| [`table`](src/table.rs) | tables a rule looks up, each cited: a degree of each sign for a body, or signs for each tithi; `Tables::check` refuses a rule naming a missing table or the wrong kind; `Tables::classical` ships Jataka Parijata's Mrityu Bhagas and Pushkara bhagas, Brihat Prajapatya's Moon, and the Dagdha rashis ([`tables/classical.json`](tables/classical.json)) |
| [`shipped`](src/shipped.rs) | the rules the SDK ships as data: the seventeen doshas and the eight Neecha Bhanga yogas the recording engine computes in code, and, read from the texts themselves, BPHS ch. 92's four gandantas and fifteen of chs. 9 and 10's evils and antidotes ([`rules/`](rules)) |
| [`trace`](src/trace.rs) | how an answer was reached: each condition checked, whether it held, the bodies it added and each reference resolved, as a tree that serialises and reads as prose; one evaluator generic over a recorder, so the untraced answer costs nothing more and cannot differ |

## What the corpus settled

- **The engine's semantics reproduce everything.** Conjunction is one sign
  unless a rule gives an orb; the Moon is malefic when waning and Mercury when
  only malefics share its sign; a deep dignity meets its plain form; all seven
  between the nodes counts either side; the nodes are never retrograde; houses
  are as recorded; and a rule's participants are gathered from every condition
  that held, a failed branch's included.
- **The corpus decides two of those** (the benefics and the orb) and cannot see
  the other five, so each is a `Readings` field rather than a hidden choice.
- **References are two kinds, and the texts name both.** BPHS's rules ask
  about the lord of a house, a karaka's dispositor, the arudha lagna, the
  upapada and the Karakamsha; the lords and karakas are bodies and the padas
  and navamshas are only signs. The upapada's house is a `Readings` field
  (crux C80) and a co-ruled sign's lord the catalogue's (crux C81).
- **Tables are data, and the texts disagree in them.** The Moon has two
  Mrityu Bhaga rows (crux C83), so both ship and a rule names one; how a
  table's degree counts is `Readings::bhaga`, the texts' ordinal degree by
  default and the engine's ±1° in its reading (crux C82); the Dagdha rashi
  table's rank is stated and whom it burns is the rule's to say (crux C84).
- **A dosha is a rule with groups, a severity and a threshold.** The engine's
  natal doshas read unchanged; 35 of its 52 are in the language, and its
  dosha evaluator adds no participant for an aspect where its yoga evaluator
  adds both (`Readings::aspect_gathering`, crux C85).
- **The rules the engine computes in code are sayable.** All seventeen are
  shipped as rules and decide presence exactly as its code does; Mrityu Bhaga,
  Dagdha Rashi and Badhaka reproduce every recorded field. It took a `side` on
  the nodal-arc predicate, a badhaka reference (crux C86) and a weight on a
  group, not a classifying outcome.
- **A rule can carry what its verse says happens.** The texts grade an
  affliction only by the span of life it leaves, so an `Outcome` is a
  `life-span` in the unit its verse uses; Saravali ch. 10's ten evils ship with
  theirs and ch. 12's hundred-year antidote with its own, and nothing else
  claims one.
- **The yogas BPHS gives**: ch. 35's thirty-two Nabhasa yogas, chs. 37 and 38's
  lunar and solar ones, the five Pancha Mahapurusha yogas and ch. 36's named
  yogas ship as `shipped::nabhasas()` with what each verse says, and 41 of the
  49 figures the recording engine also carries answer exactly as it recorded
  over the corpus — every Pancha Mahapurusha yoga among them. The eight that
  differ are readings, pinned and explained.
- **Strength, compared and not computed**: a chart may carry `Strengths` — a
  measure, each body's number and what it must reach — and `planet-strong`,
  `planet-weak` and `planet-stronger-than` read them. Strong and weak are two
  questions: a chart that carries no number answers false to both.
  `Rule::reads_strength` says which rules want such a chart.
- **A sign's own aspect**, `rashi-aspects`: BPHS ch. 26's rashi drishti, held
  to the table the chapter prints over all 144 pairs, with a body lending the
  aspect of the sign it stands in.
- **An intervention**, `argala` and `vipareeta-argala`: BPHS ch. 31's, with the
  obstructing house paired to each intervening one by the `ArgalaPlace` type so
  a rule cannot pair them wrongly, and counted backwards from a node.
- **A verse may say more than one thing**, so `Rule::outcomes` is a list:
  a Pancha Mahapurusha verse describes the native and counts his years, and
  `life_span()` and `effect()` reach either without matching.
- **Where a verse says what follows in words**, the rule carries those words:
  an outcome is a life span or a statement, and `days()` answers only for a
  span. Brihat Jataka ch. 14's twenty-one pairs and Phaladeepika ch. 18's
  seventy-two readings of the Moon ship that way, and so do Jataka Parijata's
  hundred and nineteen lists of two to six grahas sharing a sign, whose grahas
  are generated and whose readings alone are data, and Saravali chs. 22 to 29's
  eighty-four readings of a graha in a sign, ch. 30's eighty-four of a graha in a
  bhava and chs. 49 to 51's hundred and sixty-eight of the part of a sign that
  rises: 548 rules from a table, `shipped::readings()`.
- **A body's degrees within its sign** are readable (`planet-in-degrees`),
  which Brihat Jataka ch. 6 v. 8's last navamsa needs.
- **A rule can name another.** `{"type": "rule", "key": …}` holds when that
  rule holds, so BPHS ch. 9's evils carry ch. 10's antidotes as cancellations
  while the antidotes stay rules of their own; `check_references` refuses a
  dangling key or a circle.
- **A limb's edge is measured in ghatikas.** BPHS ch. 92 puts every gandanta
  in time, not in degrees, so a chart carries how far the birth stood into the
  tithi, the nakshatra and the rising sign, and `at-limb-edge` reads it. The
  engine measures gandanta in space instead; both ship, and crux C92 says they
  are different quantities.
- **A point is a place a rule can name.** `{"point": "GULIKA"}` reads any
  point the catalogue names, given to the evaluator as tables and divisions
  are; BPHS ch. 83's curses read Gulika this way.
- **A class can aspect, and can be counted.** "In aspect to a malefic" (BPHS
  ch. 9) is an aspect predicate over `any-malefic`, and "so many malefics in
  these houses from the Moon" is `count-in-houses`. Papa and shubha kartari
  (Phaladeepika ch. 6 sl. 8) needed nothing new: they are two
  `planet-in-house-from` conditions over a class.
- **The Neecha Bhanga family is sayable too.** What the eight needed was a
  `for-any` quantifier binding `SELF`, a body's exaltation and debilitation
  signs as references, and `same-sign` and `same-body`. All eight say present
  where the engine's code did; their citation is unsettled (crux C87).

## What proves it

- Every one of the engine's 605 rules reads strictly and round-trips
  (`tests/baseline.rs`).
- Under the engine's reading the kernel reproduces all 55 521 recorded
  decisions over 93 charts, and for each of the 5350 presences its
  participants in order, its houses and its cancellations
  (`tests/baseline.rs`); `cargo xtask check-yogas` measures each other reading
  on the built kernel.
- BPHS's own rules as tests, each with a chart where it holds and one where it
  does not: Budh in the second from the arudha lagna (ch. 29 v. 30), the
  amatyakaraka with the atmakaraka's dispositor (ch. 40 v. 3), the second from
  the lord of the seventh from the upapada (ch. 30 v. 42), the fourth from the
  Karakamsha (ch. 40 v. 14), and the upapada under both readings; every
  reference form read and written back, and each wrong kind or shape refused
  with its reason (unit tests).
- Under the engine's dosha reading the kernel reproduces all 3255 recorded
  decisions of the 35 dosha rules it can evaluate over 93 charts and 77
  panchangas, and for each of 885 presences where it was found from, its
  participants, houses, severity, cancellations and net status
  (`tests/doshas.rs`); all 52 dosha rules read strictly and round-trip.
- Every rule the SDK writes from a text is evaluable, cites a verse, and
  answers a pinned number of the corpus's 93 charts, with the ten that answer
  none listed and explained (`tests/classical.rs`).
- BPHS ch. 92's four gandantas hold inside their ghatikas and not outside them,
  at every boundary the verses give, and answer nothing when the chart measures
  no ghatikas (unit tests).
- The eight rules the SDK writes for the Neecha Bhanga family say present
  exactly where the engine's code did on all 744 decisions, with the same
  grahas and houses on all 202 presences (`tests/baseline.rs`).
- The seventeen rules the SDK writes for the engine's code say present exactly
  where it did on all 1581 decisions, with every severity, cancellation and
  status, and the Kalsarpa family's deliberate difference pinned
  (`tests/doshas.rs`, measured in `03-design/doshas-measured.md`).
- Every explanation over the corpus answers what its evaluation answers,
  stops at the first condition that failed, and gathers the participants from
  its steps: 55 521 of 55 521 (`tests/baseline.rs`); a unit test pins what an
  explanation holds, what it leaves out, its JSON and its prose.
- The shipped tables read, hold the page's values at their corners and both
  of the Moon's rows, and read a tithi by its number in either paksha; six
  malformed tables and a set with a key twice are refused; a rule naming a
  missing table or the wrong kind is refused before evaluating; the Moon's
  degree held under each of the four stretches at six places, and the burnt
  signs with and without a tithi, each shown in the trace (unit tests).
- Each body's navamsha is the corpus's recorded D9 on all 930 bodies
  (`tests/baseline.rs`).
- Strict refusals, the benefics by company, each reading's flip, the node
  sides, failed-branch gathering, lords, karakas and aspects (unit tests).
- The recording engine's 597 written rules over one chart take about 11
  microseconds, against a budget of 900 rules in 2 milliseconds, and about 120
  explained (`benches/rules.rs`).
