# Remedies: functional nature, graha-śānti and the subjects of a remedy (the `remedies` module)

Status: `built` for step 1 (functional nature, `crates/remedies`) and
step 2's per-graha table (graha-śānti), 2026-10-07; the antardaśā
śāntis and steps 3 and 4 are designed below and not built.

Phase 8 lists remedies after matching. The baseline engine reaches them
from a controller, and 39 interpretation readings wait on them to name
their subject (`state-readings-measured.md`). The work is in the
doctrine. The baseline mixes three layers that the texts keep apart:

1. which graha is good or bad for a lagna (rank 1, computed);
2. what is done for a graha: its japa, its dakṣiṇā, its ṛk (rank 1
   facts);
3. which stone to wear, for which no rank 1 text was found.

This page builds the first layer and fixes the order of the rest.

## What the sources decide

***Laghu Parashari*** (*Uḍudāya-pradīpa*), the 1894 Khemraj print with
Ram Svarupa Sharma's *Pradīpodyota*, read in its OCR text for vv. 6, 9,
10 and 11, is rank 1 for functional nature.

- **v. 6:** every trikona lord gives good, and the lords of 3, 6 and 11
  give evil, benefic or malefic alike (C329).
- **v. 7:** a natural benefic that rules a kendra does not give good,
  and a natural malefic that rules one does not give evil (C331).
- **v. 8:** the lords of the 2nd and 12th give the results of what they
  join.
- **v. 9:** the 8th lord is not good, being the 12th from the 9th,
  unless it is also the lagna lord.
- **vv. 10–11:** the kendra blemish is strongest for Jupiter and Venus,
  then Mercury, then the Moon; the Sun and the Moon carry no 8th-lord
  blemish.
- **v. 20:** one graha owning a kendra and a trikona is a yogakaraka
  (C332).
- **vv. 23–25:** the 2nd and 7th are the maraka houses, the 2nd the
  stronger.

**BPHS** ch. 13 (the 1923 Venkateśvara print, the English ch. 34)
restates vv. 6 to 11. Its v. 12 voids a lagna lord's own 6th or 8th
lordship. Its vv. 14 to 38 print each lagna's pāpa, śubha, yogakaraka
and killer. That table names the lagnesha pāpa for Taurus and Scorpio,
against its own v. 12, so it is not shipped until two page images are
read (C333).

## What is decided

- **Clauses, not a verdict.** A graha's functional nature is every
  clause that holds for it, each with the house that makes it hold and
  the verse it reads (`ClauseKind::source`). The summary `nature` is
  derived from the clauses by the text's precedence (C330):
  1. a yogakaraka;
  2. a trikona lord or the lagnesha, whatever else it owns;
  3. a lord of 3, 6 or 11;
  4. an 8th lord nothing voids;
  5. anything else, which is neutral.
- **The 12th lord and a kendra-only lord are neutral** (C331), each
  with its clause.
- **The baseline engine's reading is a scheme, not the default.**
  `Scheme::Baseline` makes a lord of 1, 5 or 9 good and a lord of 6, 8
  or 12 bad unless it also owns a trikona. It reproduces the baseline's
  sets, and its own spec's yogakarakas, badhaka houses and badhakeshas.
  The clauses are the same under both schemes; only the summary
  differs.
- **A lagna, not a chart.** Functional nature reads only the lagna's
  sign. The relations LP vv. 14 to 17 make a yoga of, and the 2nd and
  12th lords' association, need a chart, and wait for step 3.

| lagna and graha | *Laghu Parashari* | baseline | why |
|---|---|---|---|
| Aries, Saturn (10, 11) | malefic | neutral | v. 6: the 11th lord |
| Libra, the Sun (11) | malefic | neutral | v. 6 |
| Gemini, the Sun (3) | malefic | neutral | v. 6 |
| Leo, the Moon (12) | neutral | malefic | v. 8: by association |
| Virgo, the Sun (12) | neutral | malefic | v. 8 |
| Capricorn, the Sun (8) | neutral | malefic | v. 11: a luminary |
| Sagittarius, the Moon (8) | neutral | malefic | v. 11 |

## The surface

`crates/remedies`, re-exported as `teistro::remedies`:

- `functional(lagna: Rashi, FunctionalRules) -> Functional { lagna,
  scheme, rows, yogakarakas, marakas, badhaka }`. `rows` holds the seven
  grahas that own signs, Sun to Saturn. Each row is
  `FunctionalRow { graha, houses, clauses, nature }` and each clause
  `Clause { kind, house }`.
- `FunctionalRules { scheme }`, where `scheme` is `LAGHU_PARASHARI` (the
  default) or `BASELINE`. It is read from JSON strictly.
- `Functional::row(graha)` answers `None` for the nodes and the outer
  planets, which own no sign.
- `Badhaka { house, lord }`: the 11th from a movable lagna, the 9th from
  a fixed one, the 7th from a dual one, as the rules engine's
  `SignRef::badhaka` reads it.

## The steps that remain

2. **What is done for a graha** (built, C343 to C347).
   `shanti(graha, rules)` answers one graha's graha-śānti, each value
   the verse word read on the page:
   - **From BPHS (the 1952 print) ch. 84**, read beside *Yājñavalkya*
     I.295–307:
     - the image's material (v. 4);
     - the ṛk (vv. 17–18);
     - the japa in thousands (vv. 19–20: 7, 11, 10, 9, 19, 16, 23, 18,
       17);
     - the samidh (v. 21), the food (v. 23) and the dakṣiṇā (v. 25);
     - the 108 or 28 offerings (v. 22) as `OFFERINGS`.
   - **From the other texts:**
     - the gem a graha owns (*Jataka Parijata* II.21);
     - the substance it rules (*Brihat Jataka* II.12 = *Jataka
       Parijata* II.20);
     - its direction (*Brihat Jataka* II.5);
     - its place in the Matsya maṇḍala the Mitākṣarā quotes.
   - **The two texts agree verse by verse except for Rahu's ṛk.**
     `RikSource` chooses between them (C343). Yājñavalkya gives no japa
     counts.
   - **The translations differ from the verse in three places**, and
     the verse word wins (C344):
     - Mars's *diśaḥ* is 10, not the English print's 11 000;
     - Saturn's *tri-pakṣāḥ* is 23, its digits read right to left;
     - Ketu's *chāga* is a goat, not the 1918 English "sheep".
   - **The nodes get nothing the verses do not give them.** Ketu has no
     direction (C345) and the nodes rule no substance (C346). Ketu's gem
     is *Jataka Parijata*'s *vaidūrya*; BPHS 2.32's *nīlamaṇi* waits for
     its page image (C347).
   - **Commentary stays out of the values.** The Mitākṣarā's yellow
     cloth and grey horse, and the 1952 Hindi's cow with her calf, are
     notes on this page, not values.
   - **Still to come:**
     - the antardaśā śāntis of BPHS (1923) chs. 37–45, 79 rows, each
       conditioned on a lordship or a placement, become rule records
       once their conditions are transcribed;
     - the baseline engine's bījas, grains, fingers, carats and ring
       metals become a `BASELINE` pack, marked unsourced.
3. **Whom a remedy is for.** BPHS 84.26 and *Yājñavalkya* I.307 name the
   graha ill-placed (*duḥstha*) for the person at the time. The answer
   is a list of subjects, each with its reasons:
   - the running daśā lords;
   - debility, combustion, or a place in 6, 8 or 12;
   - a functional malefic, a maraka or the badhakesha;
   - an active dosha.

   The answer carries no priority by default, and the instant is always
   given; nothing reads the clock.
4. **The ishṭa-devatā.** BPHS (1923) ch. 9 vv. 70–75 reads it from the
   grahas in the 12th from the karakamsha. The baseline instead looks up
   its 12 sign-keyed records by the Moon's sign.

The 39 waiting readings become sayable at step 3, when each has a
subject graha.

## The acceptance tests

Held now, in `crates/remedies/src/tests.rs`:

- **Every lagna, every graha:** the houses it owns and its nature under
  *Laghu Parashari*, typed lagna by lagna from the verses rather than
  computed.
- **The six yogakarakas and no other**, under both schemes.
- **The lagnesha** is always good, and its own 8th is voided for Aries
  and Libra.
- **The luminaries' 8th** is neutral for Capricorn and Sagittarius,
  against the baseline's malefic.
- **The lords of 3 and 11 against the 12th:** each rival reading is
  pinned both ways.
- **The kendra rule:** the baseline's four kendradhipati cases, and the
  Sun's 4th for Taurus.
- **The badhaka and the marakas**, lagna by lagna, against the
  baseline's own table.
- **The baseline scheme's malefic sets**, lagna by lagna.
- **Keys:** each key is what serde writes, and the rules are read
  strictly.
