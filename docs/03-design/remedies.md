# Remedies: functional nature, graha-śānti and the subjects of a remedy (the `remedies` module)

Status: `built` for step 1 (functional nature, `crates/remedies`) and
step 2 (graha-śānti and the antardaśā śāntis), step 3 (the subjects
of a remedy) and step 4 (the ishṭa-devatā), 2026-10-07.

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
   - **The antardaśā śāntis** (built, C348 to C350). `dasha_shanti(md,
     ad)` gives what BPHS (the 1923 print) chs. 37–45 print for each of
     the 81 antardaśās: the verses, the page, the condition and the
     rites. The table holds:
     - 53 conditions on the antardaśā lord ruling the 2nd or 7th, 4 on
       the 7th alone, 10 on a placement in the 2nd or 7th, and 13 of
       their own;
     - 79 rows with a rite. Venus/Moon prints neither a condition nor a
       rite, and Venus/Mars prints a condition and no rite.

     A condition is the predicate the verse states. Whether it holds
     for a chart is step 3's question. Jupiter/Moon's "lord of the 2nd
     and 6th" differs between the prints and stays open (C349).
   - **Still to come:**
     - the baseline engine's bījas, grains, fingers, carats and ring
       metals become a `BASELINE` pack, marked unsourced.
3. **Whom a remedy is for** (built, C351). BPHS 84.26 and
   *Yājñavalkya* I.307 name the graha ill-placed (*duḥstha*) for the
   person at the time, and neither defines the word. `subjects(sky,
   functional)` answers each graha with every reason that makes it a
   subject:
   - the running mahādaśā or antardaśā lord;
   - debilitated, combust, or in the 6th, 8th or 12th;
   - a functional malefic, a maraka or the badhakesha (step 1).

   **How the answer reads:**
   - It ranks nothing, and it reads no clock: the running daśā is part
     of the input.
   - **The running antardaśā's printed śānti** comes with it, and `holds`
     judges each of its conditions where the verse states a predicate.
   - **What the verse leaves open is not decided.** That covers "with
     the 8th or 12th" and the mixed grammar, Jupiter/Moon's
     2nd-and-6th lord, which the Moon, owning one sign, cannot be
     (C349), and any lordship of a node (C351). Each answers `None`.
   - **An active dosha** as a reason waits for the rules engine's
     doshas to be read here.
4. **The ishṭa-devatā** (built in the kernel, C354 to C356). BPHS (1923)
   ch. 9 vv. 70–76, ch. 33 vv. 68–74 in the later recension, reads it
   from the grahas in the 12th sign from the kārakāṁśa.
   `ishta_devata(karakamsha, signs, rules)` answers each graha there
   with the deity its verse names and whether Ketu shares the sign, and
   Saturn or Venus there in a sign the Sun, Mars or Saturn rules as a
   devotee of minor deities.
   - **The prints exchange vv. 70–71's fruits.** The recension pairs the
     Sun with Śiva and the Moon with Gaurī; the prints' *ravi-bhakti* is
     the knob `SunWithKetu::Surya` (C354).
   - **"With Ketu" is reported, not read in.** Rahu never stands with
     Ketu, so the series cannot all be conjunctions (C355).
   - **The chart is the caller's** (C130): `sdk.chart().ishta_devata(&document,
     rules)` answers the rasi chart and the navāṁśa both, from the
     kārakāṁśa the chart's Jaimini reading names.
   - **The baseline's table** keys twelve pairs by the Moon's sign;
     `baseline_ishta_devata` is reached only when asked (C356).
   - **Not built:** vv. 66–69 (a benefic exalted in the 12th; Ketu there
     and liberation), which turn on aspects the chapter does not define,
     and vv. 77–79's same reading from the amātyakāraka, which waits on
     the Jaimini reading exposing the amātya.

The 39 waiting readings become sayable at step 3, when each has a
subject graha.

## Through the façade and the bindings

`sdk.chart().remedies(&document, &RemedyRequest)` reads every step off
one chart: the lagna's functional natures, the subjects with their
reasons, each subject's graha-śānti, and the ishṭa-devatā in both charts.
`RemedyRequest` is `{at, rules}`. `rules` holds `functional`, `shanti`
and `devata`, each the texts' own when left out. `at` is the Julian day
whose running Vimśottarī mahādaśā and antardaśā name subjects and bring
the antardaśā's printed śānti. It is Vimśottarī because chs. 37–45
prescribe for its antardaśās. Without `at` no daśā is read, since the
kernel reads no clock. With it, a document carrying no Vimśottarī daśā
is refused by `dashas`.

The bindings follow prashna's pattern: a `remedies_json` member of the
chart request, which asks for the Vimśottarī daśā itself, and a
`remedies` section of canonical JSON, one answer a chart.

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
