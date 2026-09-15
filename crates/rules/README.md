# `teistro-rules`

Status: `building`, 2026-09-15: the kernel's first slice, the condition
language over grahas and the lagna, built and held to every recorded yoga;
then references to lords, karakas, padas, the upapada and navamshas. The
design is [`docs/03-design/rules-engine.md`](../../docs/03-design/rules-engine.md),
measured in [`docs/03-design/yogas-measured.md`](../../docs/03-design/yogas-measured.md).

The rules kernel: a rule's conditions, read strictly, evaluated over a chart
to whether it is present, which bodies took part, in which houses, and which
cancellations held.

| module | what it settles |
|---|---|
| [`reference`](src/reference.rs) | what a condition is about, as two types: a `BodyRef` (a graha or the lagna, the lord of a sign, the holder of a karaka) and a `SignRef` (any body's sign, a house, an arudha pada, the upapada, a navamsha, a sign counted from another), so a rule asking for the dignity of a pada is refused when read |
| [`language`](src/language.rs) | the condition language, typed: three combinators and 22 predicates in the recording engine's own shape, so its rules read unchanged, and anything else (an unknown field or predicate, a thirteenth house, an outer planet) refused with its path |
| [`chart`](src/chart.rs) | what a rule reads of a chart (each body's longitude, sign, house, dignity, motion, combustion, chara karakas and navamsha), and every place the language leaves a meaning open, each a `Readings` field with the recording engine's choice the default and the text's where the engine has none |
| [`eval`](src/eval.rs) | the `Evaluator`: benefics and malefics settled once a chart, then each rule to a `RuleResult` without allocating for its participants |

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
- **Eight rules are outside the language**, the Neecha Bhanga family, which
  the engine computes in code; `Rule::is_evaluable` says so.

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
- Each body's navamsha is the corpus's recorded D9 on all 930 bodies
  (`tests/baseline.rs`).
- Strict refusals, the benefics by company, each reading's flip, the node
  sides, failed-branch gathering, lords, karakas and aspects (unit tests).
- The recording engine's 597 written rules over one chart take about 12
  microseconds, against a budget of 900 rules in 2 milliseconds
  (`benches/rules.rs`).
