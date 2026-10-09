# Rectification: the birth time narrowed by the verses that test it (the `rectification` module)

Status: steps 3 to 6 `built` (`crates/rectification`: the purifier and the
façade's `ChartArea::rectify`, 2026-10-08; the conception reports and
`ChartArea::conception`, the circumstances and `ChartArea::circumstance`, the
baseline cascade and `ChartArea::rectify_baseline`, 2026-10-09); steps 7 and
8 open. The sources are read on
their pages ([`rectification-sources.md`](rectification-sources.md)); the cruxes
are numbered X1 onwards and take C-numbers when they enter the register.
Each was decided on 2026-10-08 as its recommendation reads, on the
evidence beside it; X2 and X8 are measured again when the first stage is
built, and a measurement that contradicts its premise reopens it.

A birth record gives a window, not an instant: "before dawn", "about four",
a clock that may have been read late. Rectification answers **which parts of
that window survive** the tests the texts put to a birth time, and why each
part survived. It never answers a single minute. A minute is the middle of an
interval, and the interval is the answer.

The texts test a birth time in two ways, and the module keeps them apart:

- **a purification** (*śodhana*): a relation the lagna must stand in, which
  removes the times where it fails (BPHS ch. 2 vv. 71–78);
- **an indication** (*lakṣaṇa*): what a sky at birth shows about the
  circumstances of the birth, to be compared with what the family remembers
  (*Brihat Jataka* ch. V).

The baseline engine's cascade is neither. It scores candidates against dated
life events by dasha boundaries under a tattva prior. No text read gives
either stage, so they ship at rank 2 as a call of their own
(`ChartArea::rectify_baseline`), reached only when asked, as C89's
precedent requires (X20).

## What the sources decide

The texts read, each on its page images unless marked:

- **BPHS, Subodhini prints**: Gyan, Mumbai 1899 and Khemaraj, Mumbai 1923,
  Purvakhanda ch. 2 vv. 67–78, pp. 11–13 (1923 leaves n40–n42). This is the
  roadmap's numbering. Sitaram Jha's print carries a different passage
  (its ch. 3 vv. 66–74) and is cited by verse number only.
- **BPHS, the nisheka lagna**: the same prints, ch. 3 vv. 25–30 (1923 p. 15;
  Jha's ch. 4 vv. 25–30, pp. 35–36, with the only worked example).
- **Brihat Jataka**: Vijnanananda 1912 for the Sanskrit (IV.21 on p. 89),
  Chidambaram Iyer 1885 for Bhattotpala's reading (pp. 37–50).
- **Shiva Svarodaya**: Rama Prasad 1894 (v. 193, p. 217) and the 1899
  Khemraj Sanskrit print (the same verse as v. 197, p. 48).

### The purifiers (BPHS ch. 2 vv. 67–78)

- **Gulika** (vv. 67–70): the day, or the night, cut into eighths with lords
  in weekday order from the day's lord, the night from the fifth lord on;
  Saturn's eighth is Gulika. The gloss's multiplier table and its worked
  case (a Wednesday, day length 33;14 ghatis, Gulika at 16;37) place the
  Gulika time at the **end** of Saturn's eighth; the SDK's `Gulika` is its
  start (X5).
- **The pranapada** (vv. 71–74):
  - one sign for every fifteen palas since sunrise, the left-over palas
    doubled for degrees, the whole taken mod 12 (v. 71–72);
  - joined to the Sun's sign, counted from the movable sign of its
    triplicity (v. 72), which v. 74 states as the Sun itself, its 9th or its
    5th for a Sun in a movable, fixed or dual sign;
  - "from it correct the lagna as before" (v. 74).
- **The test** (v. 75): a lagna not made pure by the pranapada, or by Gulika,
  or by the Moon (*vā*: any one) is impure, "the birth of a plant".
- **The relation** (v. 77): the lagna stands in the purifier's sign or its
  trine for a human birth. vv. 77–78 give the other houses to beasts, birds
  and creeping or water creatures, so the twelve are all assigned and only
  the trine is human.
- **Gulika's extension** (v. 76): "when the two are weak", judge from Gulika,
  from its 7th, from its navamsha and from the 7th of that, each with its
  trine.
- **The degree agreement** (v. 72): *lagnāṃśa-prāṇāṃśa-padaikyatā*, the
  lagna's degree and the pranapada's should agree. The gloss's example shows
  it as equal degrees and adds "but not everywhere" (X4).
- **The worked example** (p. 13): a birth 6;17 ghatis after sunrise with the
  Sun at 2s 4°28′1″ gives a pranapada of 3s 4°. The 1899 print's working is
  the verse's arithmetic and the 1923 print's is not; the printed answer is
  reached by readings the verse does not state (X3).

### The conception (BPHS ch. 3 vv. 25–30; BJ IV.21)

- **BPHS**: having purified the birth lagna, purify the conception (v. 25).
  The distance from Saturn to Mandi plus the distance from the lagna to the
  9th bhava, read as months and days, is how long before birth the conception
  was; with the lagna lord in the invisible half the Moon's elapsed degrees in
  her sign are added (vv. 27–28). Cast that moment's lagna and purify it "as
  before" (v. 29).
- **BJ IV.21**, reported as others' teaching (*pravadanti*): the number of the
  Moon's dvadashamsha at conception, counted on, gives the sign the Moon holds
  at birth; the rising sign's class gives a day or a night birth; the risen
  fraction of the rising sign is the fraction of the day or night elapsed at
  birth. Commentators count from different signs and some read the navamsha
  for the sign (X11).
- Read together, the two make a closed check on a candidate: derive the
  conception from the candidate by BPHS, then ask whether the candidate's own
  Moon and day-or-night agree with BJ IV.21 read at that conception. No text
  states that composition, so it is reported, never barred (X10).

### The circumstances (BJ ch. V)

The chapter says what the sky shows about the birth. Each verse is a clause
over a fact the caller may know:

- the father present or absent (V.1–2);
- the child's presentation, head or feet first, by how the rising sign rises
  (V.17);
- the lamp's oil by the Moon's place in her sign and its wick by the rising
  degree (V.18);
- the number of women attending, by the grahas between the lagna and the
  Moon, those above the horizon outside the room (V.22, with the reversed
  opinion noted).

V.18's wick and oil are continuous in the degree, so they are the only
indications that see inside a sign. The rest change with the lagna's sign or
navamsha.

### The tattvas (Shiva Svarodaya)

- **v. 193** (1899 v. 197) gives the durations: earth 50 palas, water 40,
  fire 30, air 20, ether 10, two and a half ghatis together, which is one
  nadi's turn (1899 v. 63).
- Which nadi rises at sunrise goes by the tithi in runs of three days from
  pratipada (1899 v. 62), the tattvas rise in each turn in the order air,
  fire, earth, water with ether at the junction (vv. 71, 72, 154), and the
  Moon's nadi is female, the Sun's male (v. 60).
- **No verse applies a nadi or a tattva to the birth moment.** The sex
  rules read a tattva at a question about a pregnancy (1899 v. 294) or at
  conception (v. 299), and the two disagree about water.
- Rama Prasad's "a ghari each" and "five gharis each" mistranslate 1899
  vv. 64 and 72, so the text is one rule (X12).

## The surface

`sdk.rectification()` is an area of its own. It takes the record, not a
chart: the window, the place and what the family knows.

```rust,ignore
let found = sdk.rectification().narrow(&Record {
    window: Window::between(from, to),        // civil or UT instants
    place,
    native: Native::Human,                     // v. 77's species; the default
    facts: Facts {
        father_present: Some(false),           // BJ V.1–2
        presentation: Some(Presentation::Head),// BJ V.17
        attendants: None,                      // BJ V.22
        sex: None,                             // read only by a tattva stage
        events: vec![],                        // read only by BASELINE stages
    },
    stages: Stages::default(),
})?;
```

| answer | contents |
|---|---|
| `intervals` | the maximal runs of the window no bar removed, each with `from`, `to`, and per stage the clauses that held, with their graha, sign or degree |
| `removed` | the runs a bar removed, each naming the clause that failed and its verse |
| `edges` | every instant a clause changes, found exactly, so an interval's ends are clause edges and not grid cells |
| `weights` | per interval and stage, the clauses that weigh for or against it, never summed across stages unless the caller asks |
| `stages` | per stage: applied, skipped for want of a fact, or flat (ran and distinguished nothing) |
| `grid` | the seed grid's step and the count of cells, so a run reproduces |

**A stage** is one component, keyed and tied to its verse:

```rust,ignore
pub trait Stage {
    fn key(&self) -> StageKey;               // a catalogue member
    fn cites(&self) -> &'static [Citation];  // ADR-0018
    fn needs(&self) -> Sections;             // what each candidate chart must carry
    fn edges(&self, sky: &WindowSky) -> Vec<Edge>;          // where a clause can change
    fn judge(&self, at: &CandidateSky) -> Verdict;          // clauses, a bar, weights
}
```

- `needs` names every section the stage reads, so a candidate batch asks
  the chart crossing for nothing more, and refuses without it (the lesson
  of `phala`'s composer).
- `edges` lets the module cut the window where clauses change: the lagna
  crossing a sign or navamsha, the pranapada crossing a sign, Gulika's
  portion boundary, sunrise and sunset. Between two edges every clause is
  constant, so judging one instant judges the run, and the answer is exact
  without a fine grid (the saait precedent).
- `Verdict` carries the clauses that held and failed, an optional bar (a
  clause the verse states as a must) and weights (clauses the verse states
  as indications).

**The stages:**

| stage | verse | kind | needs |
|---|---|---|---|
| `PURIFIER` | BPHS ch. 2 vv. 71–78 | **bar** on v. 75 (no purifier holds), clauses for each purifier, v. 72's degree agreement as a weight | lagna, Moon, Sun, ishtakaal, the day's arcs, Gulika's instant |
| `PRANAPADA_HOUSE` | Jha's print ch. 3 vv. 72–74 | report only: the pranapada's house from the lagna as Jha's print judges a birth | as above |
| `NISHEKA` | BPHS ch. 3 vv. 25–29 | report: the conception instant and whether its lagna passes `PURIFIER` | Saturn, Mandi, 9th bhava, lagna lord, Moon |
| `CONCEPTION_MOON` | BJ IV.21 on `NISHEKA`'s instant | weight | the conception Moon's dvadashamsha, the candidate Moon, day or night |
| `CIRCUMSTANCE` | BJ V.1–2, 17, 18, 22 | weights over the facts given; skipped clause by clause where a fact is absent | aspects, house occupancy, the Moon's and the lagna's degree |
| `TATTVA` | Shiva Svarodaya vv. 59, 61–64, 193 | weight, **off by default** | sunrise, tithi, sex |
| `PRIOR`, `DASHA_BOUNDARY` (`BaselineStage`) | the baseline engine | rank 2, marked unsourced, a call of its own (`rectify_baseline`, X20) | sex, dated events, reported time |

**The knobs** (`Stages`, one per crux):

- `purifiers`: which of pranapada, Gulika and the Moon may purify (all, by
  default) and whether Gulika's v. 76 extension applies (yes);
- `purify_as`: `BAR` (default) or `WEIGHT`;
- `pranapada`: `VERSE` (default), `PRINTED_EXAMPLE`, `SDK_POINT` (X2, X3);
- `gulika_at`: `END` (default, the gloss) or `START` (the SDK's point) (X5);
- `degree_agreement`: `OFF`, `SAME_DEGREE`, `WITHIN(orb)` (X4);
- `conception_count`: `NEXT_AFTER_DVADASHAMSHA` (default, Bhattotpala),
  `FROM_MOON_SIGN`, `FROM_ARIES`; `conception_rising`: `SIGN` or `NAVAMSHA`
  (X11);
- `circumstance` (`CircumstanceRules`): `moonSees` `ANY_ASPECT` (default)
  or `FULL` (X16); `sunFallen` `NINTH_OR_EIGHTH` (default) or
  `EITHER_SIDE` (X17); `presentationBy` `RISING_SIGN` (default) or
  `LAGNA_LORD_MOTION` (X18); `betweenBy` `DEGREE` (default) or `SIGN`, and
  `outside` `VISIBLE` (default, V.22) or `INVISIBLE` (X19);
- `tattva`: `OFF` (default), `SVARODAYA` (X12; the baseline's cycle is
  read inside `rectify_baseline`'s prior);
- `seed_step`: the grid step that seeds edge finding and the baseline
  comparison (X15).

A migrating consumer calls `rectify_baseline` with a `BaselineRequest`
(the reported time, its uncertainty and accuracy, the sex, the dated
events, the coverage and the dasha rules) under a context whose frame is the
baseline's; nothing in `Stages` reaches it.

**Reuse:**

- the candidate charts come from one `found_many` crossing over the seed
  grid and the edges (`area/chart.rs`; `chart-at-the-boundary.md` §3a);
- Gulika and Mandi from `crates/points/src/eighth.rs`, which already takes an
  `Ascendant` at instants that are not the birth;
- the ishtakaal, sunrise, sunset and tithi from the chart's day;
- the navamsha and dvadashamsha from `crates/vargas`;
- the 9th bhava from `angles_of` under Sripati, never from `chalit` (the
  lesson of the named division);
- the edges by the same bracketing root finder the muhurta search uses.

**Refusals:**

- a window that is empty, reversed or longer than a day and a half (the
  bound `points::lagna::LONGEST_HOURS` already sets);
- a fact of the wrong kind for its clause;
- `TATTVA` without a sex, a stage whose sections the chart did not carry;
- for `rectify_baseline`, an uncertainty outside 1 to 720 minutes, a
  coverage outside 0.5 to 0.99, and an event ending before it begins.

Each names its field under `rectification`.

## Cruxes

**X1. Whose BPHS.** The Subodhini prints carry vv. 71–78 as a correction of
the birth time; Jha's print carries a pranapada that judges the birth
auspicious by its house from the lagna and calls the Subodhini verses
interpolated.
- Options: the Subodhini text as the default; Jha's as the default; both,
  each as its own stage.
- Evidence: the roadmap and the baseline's own research cite the Subodhini
  numbering; the 1899 and 1923 prints agree with each other verse for verse;
  Jha's objection is an editor's judgement, not a manuscript variant shown on
  the page.
- **Decided:** `PURIFIER` from the Subodhini text, and `PRANAPADA_HOUSE`
  from Jha's verses as a reported fact that never bars. Check Santhanam's
  numbering in the maintainer's print and record it beside both.

**X2. The pranapada's rate.** The verse moves the pranapada a sign every
fifteen palas, four signs a ghati, 300° an hour. The SDK's
`points::lagna::pranapada` moves 60° an hour, a rate derived from the
conformance corpus (`points-measured.md` §5).
- Options: the rectification stage computes its own pranapada from the
  verse; it reuses the SDK point; the SDK point is changed.
- Evidence: every print read (both Subodhini prints, Jha's v. 71 with its
  gloss of fifteen palas a sign) gives a sign per fifteen palas; the
  printed example multiplies the ghatis by four for signs.
- **Decided:** the stage computes the verse's pranapada (`VERSE`); keep
  `SDK_POINT` as a knob value; raise the gap with the corpus point as a
  finding of its own and do not change `points` inside this module.
- **Measured** (`rectification-measured.md`): at the recorded instants the
  verse's pranapada and the SDK's point seldom share a sign, as two rates a
  factor of five apart must; the gap stands as the finding of its own.

**X3. The pranapada's start and the printed example.** v. 72 joins the
count to the Sun's sign counting from its movable trine; v. 74 adds it to
the Sun, its 9th or its 5th. The example prints 3s 4°, which the verse's
sequence does not give (it gives Scorpio 4°). The answer is reached either
by adding to the Sun's sign with no shift, or by the gloss's mod-15 count
from Libra; the 1923 print's intermediate figures do not even match its
answer.
- **Decided:** `VERSE` as the default, the Sun's longitude as the start
  with v. 74's shift; a test that reproduces the 1899 working figure for
  figure up to the count, names the differing cell, and shows which
  `PRINTED_EXAMPLE` reading reproduces 3s 4° (the precedent of the Yogardha
  table's named cells).

**X4. What "agreement" of degrees means** (v. 72). Equal degree, equal
navamsha, or within an orb. The example's 4° against the Sun's 4°28′ and its
"but not everywhere" suggest a tolerance, not equality.
- **Decided:** report the separation of the lagna's and the pranapada's
  degrees in the sign as a fact; `degree_agreement: OFF` by default, so no
  invented orb shapes an interval. Revisit with a second worked example.

**X5. Gulika's instant.** The gloss's multipliers put it at the end of
Saturn's eighth; the SDK's `Gulika` is the start, settled against the
conformance corpus, and `Mandi` the end.
- **Decided:** `END` for this stage, since the gloss is the only operative
  instruction attached to these verses; `START` as a knob. The example's
  16;37 ghatis is a test (the lagna it gives needs a date the page does not
  print, so only the time is asserted).

**X6. The relation.** v. 77 gives the trine; does it hold for all three
purifiers, and does v. 76's extension (the 7th, the navamsha, its 7th) hold
for Gulika alone?
- **Read** (2026-10-08) on the page images of the 1899 Gyan print (twice,
  two scans), the 1923 Khemaraj print and Jha's 1952 note reproducing the
  Bombay verses. The prints agree on vv. 75–77 but for a particle in v. 75
  (*tadaiva* 1899, *tadeva* 1923). **No public-domain print glosses vv.
  75–77**: the Subodhini passes from v. 78 to the pranapada example, and
  Jha prints the verses only to reject them. In v. 76 every pronoun after
  *gulikāt* returns to Gulika, so the extension is Gulika's; *kalatrataḥ*
  reads best as the navamsha's 7th, four references in all. v. 77's
  *tatra* points at Gulika's references, but v. 75's single notion of
  purity, vv. 77–78's division of all twelve houses as a general relation,
  and the Moon's having no other relation in the passage favour the trine
  for all three; practice agrees (a modern Hindi edition's worked example
  applies it to the pranapada, and Phaladeepika III.16 to Gulika and the
  Moon).
- **Decided:** the trine for every purifier, the extension for Gulika only;
  each extension reported as its own clause. The basis is the text's
  structure and received practice, with no gloss behind it, and the page
  says so.

**X7. "When the two are weak"** (v. 76: *dvayor hīnabale 'py evaṃ gulikāt
paricintayet*). The condition governs the whole sentence, extension
included, so the extension is conditional, and v. 76 settles v. 75's *vā*
as "any one purifier suffices". "The two" are most plainly the pranapada
and the Moon, v. 75's purifiers beside Gulika; the Subodhini's own example
uses *dvayor aikyam* for the lagna's and the pranapada's degrees, a second
candidate. Weakness is not defined; Phaladeepika III.16's parallel
(*tadvad vidhau balayute*, from the Moon "when the Moon is strong") leaves
strength undefined too.
- **Decided:** `gulika_extension: WHEN_TWO_FAIL` by default: the extension
  counts when neither the pranapada nor the Moon purifies. It follows the
  verse's grammar without inventing a strength, and it keeps exactly the
  instants an extension always counted keeps (an instant either of the two
  holds is pure already), so X8's measurement stands; what changes is the
  report, where each clause says whether it counted (`counted`) and
  `purified_by` names what made the instant pure. The reading is the SDK's
  and the page says so. `ALWAYS` counts it without precedence; `NEVER`
  drops it. A strength-based reading (a Moon strength from paksha bala or
  shadbala, after Phaladeepika) is not the default: it would be an
  invented threshold shaping the interval, which X4 refused. Add it as a
  knob only with a source that defines the strength; the degree-pair
  reading of "the two" only with a second source.

**X8. A bar or a weight.** v. 75's verb is *vijānīyāt*, "one should know it
impure": for a human native, a lagna no purifier holds is a contradiction.
- **Decided:** a bar by default (`purify_as: BAR`), a weight on request.
  Measure, on a corpus of births with trusted times, how much of each day the
  bar removes and whether trusted times survive it, before the default ships.
- **Measured** (`rectification-measured.md`, 2026-10-08): how much of each
  recorded day every reading keeps. The bar with v. 76 keeps most of a day
  and on some days all of it, because Gulika's four references with their
  trines can cover the zodiac; without v. 76 it removes far more. So v. 76
  decides how much the bar can say, which makes X6 the question to settle
  before the default ships. The corpus's instants are synthetic, so whether
  trusted times survive still waits for a corpus of them.

**X9. Day and night.** The pranapada counts palas from sunrise through the
night; Gulika's eighths switch to the night arc after sunset; a birth before
sunrise belongs to the previous day.
- **Decided:** the chart's own day (its sunrise, its ishtakaal, its
  `chart_day` rule) for every stage, so a stage never reckons a second day.
- **Found building it** (2026-10-08): the chart's ishtakaal is counted
  under its ghati reckoning, proportional by default, so the verse's
  "fifteen palas a sign" runs faster or slower than six clock minutes as
  the daylight is shorter or longer than twelve hours. `Sky::ishtakaal_hours`
  carries the chart's count, and the façade's read-back test holds every
  clause to the chart founded at its instant.

**X10. The conception check.** BPHS gives the conception instant, BJ IV.21
a test at conception; neither says to compose them. Its units (a sign a
month, a degree a day: solar, lunar or civil) and the 9th bhava's division
are unstated.
- **Decided:** `NISHEKA` reports the instant and its own purification;
  `CONCEPTION_MOON` is a weight; months of 30 days and Sripati's 9th bhava,
  each a knob; find a public-domain worked example before either weighs.
- **Found** (2026-10-08): Jha's print numbers the passage **ch. 4 vv.
  25–30** and works the only example found (p. 36): Mandi 9s 29°36′53″ less
  Saturn 7s 13°24′27″ is 2s 16°12′26″, the 9th bhava (Sripati, his bhava
  table) 6s 29°0′36″ less the lagna 10s 26°28′5″ is 8s 2°32′31″, and the
  sum 10s 18°44′57″ is read as 10 months 18 days 44 ghatis 57 palas, the
  lagna's lord visible so no Moon added. His gloss fixes the units (a sign
  a month, a degree a day, a minute a ghati, a second a pala), the
  invisible half (the six signs ahead of the lagna) and **Mandi at the start
  of Saturn's eighth**, which he defends against the end. So `NISHEKA` has
  its own `mandi_at`, `START` by default, apart from the purifier's `END`
  (X5), and the example is its acceptance test to the second.
- **Found while building:** the conception moves by a day for every degree
  its arcs move, so its lagna turns while the birth moves by minutes. The
  reports are therefore read at an instant (`ChartArea::conception`), not
  over a run, and the count carries `daysPerBirthMinute`. And the
  conception is a chart of its own: read in the birth's zodiac its lagna
  was 0.0226° off its own chart's, the ayanamsha's drift over the 591 days
  between them, so its sky is founded at its own instant.

**X11. BJ IV.21's count.** From the sign after the dvadashamsha's sign
(Bhattotpala, per Iyer), from the Moon's sign (the 1912 main text, Gargi), or
from Aries; and the rising sign or navamsha.
- **Decided:** Bhattotpala's as the default, the rest as knob values; read
  the 1864 *Vivṛti* page before shipping, since both translators summarise it.
- **Read** (2026-10-08) on the lithograph's page (the "1864" and "1852" IA
  items are two scans of one lithograph, one witness): Bhattotpala gives the
  count from Aries to "some" and the count from the sign after the
  dvadashamsha's to "others", and calls the second "the right explanation"
  after Garga. With the Moon in sign `S` (Aries 0) and the dvadashamsha she
  occupies `k` (1 to 12), the birth Moon's sign is `S + 2k − 1`; the 1912
  main text's is `S + k − 1`, the Aries count's `k − 1`. Iyer's and the 1912
  print's example (the middle of Aquarius's 8th dvadashamsha) gives Taurus,
  Rohini by Bhattotpala's proportion, and Virgo by the 1912 main text; all
  are tests, and the rival ordinal (the completed count) must not give
  Aries. Pisces is a day sign to Iyer and either to the 1912 print
  (`pisces`, `EITHER` by default); the rising fraction is measured in rising
  time, as both translators do, and reported, never weighed.

**X12. The tattvas.** Read in the 1899 Sanskrit and checked against the
1919 and 1931 prints (`rectification-sources.md` §3), the Shiva Svarodaya
is one rule: a nadi's turn is two and a half ghatis, twenty-four turns
through day and night from the sunrise (1899 vv. 62–63); inside each turn
the five tattvas rise afresh in the order air, fire, earth, water (vv. 71,
72) at v. 197's palas (air 20, fire 30, earth 50, water 40), and ether's
10 flow at the junction (v. 154). Rama Prasad's "a ghari each" and "five
gharis each" (his vv. 63, 70) mistranslate vv. 64 and 72, which put all
five within the turn. The sex rules belong to questions and conception and
disagree on water; no verse reads a nadi or a tattva at a birth. The
baseline's cycle (unequal minutes on 90, a weekday-lord start,
alternating direction, its own sex map) is a modern exposition.
- **Decided:** "Svarodaya v. 193" is the Shiva Svarodaya in Rama Prasad's
  numbering, the durations verse; no other Svarodaya has one. `TATTVA` is
  off by default; `SVARODAYA` reads the nadi at sunrise from the tithi
  then (v. 62), runs the turns on from it through the sixty ghatis of the
  sunrise-to-sunrise day (v. 63), places the tattvas in each turn in
  v. 71's order with ether last, at the junction, and reports the nadi,
  the tattva and the sex v. 60 gives the nadi, **labelled an application
  the text does not make**, a report and never a bar. The sushumna is the
  junction itself and is reported as an instant, not a span. The
  baseline's cycle stays rank 2 inside `rectify_baseline`'s prior (X20,
  X24).

**X13. Kunda.** The ×81 check is widely taught and attributed to Prasna
Marga; it was not found on a public-domain page.
- **Decided:** not shipped; listed as unread, so the gap is visible.

**X14. Life events.** Fitting dasha boundaries to dated events is the
baseline's main discriminator and has no classical text.
- **Decided:** the baseline's `DASHA_BOUNDARY` and `PRIOR` at rank 2, with
  its interval assembly, reproduced only by `rectify_baseline` (X20); the
  classical `narrow` never sums likelihoods across stages.

**X15. Grid against edges.** The classical stages are piecewise constant, so
exact clause edges give exact intervals; the baseline answers on a refined
grid.
- **Decided:** exact edges for the answer; the seed grid only to find
  edges and to compare with the baseline, and the comparison allows one cell
  of the baseline's final step.

**X16. "The Moon not seeing the lagna" (V.1).** The verse says only
*na paśyati*; BJ II.13 gives every graha its quarter, half, three-quarter
and full aspects, so "not seeing" may mean no aspect at all or no full one.
- **Decided:** any aspect sees (`ANY_ASPECT`), since II.13 is
  Varahamihira's own measure of seeing; `FULL` is the knob value. The aspect
  is counted from the Moon's sign to the lagna's, so the Moon in the lagna's
  own sign does not see it, and the strength is reported either way.

**X17. "The Sun fallen from the 10th" (V.1).** Iyer, following Bhattotpala,
gives the 9th or 8th, the houses the Sun falls through after culminating;
the 1912 gloss of *madhyād bhraṣṭe* gives the 11th, 12th, 9th or 8th,
fallen either way.
- **Decided:** `NINTH_OR_EIGHTH` by default, the commentator's; the
  whereabouts are reported only where V.1's first half holds, since the
  verse makes the Sun a further condition, not a clause of its own.

**X18. The presentation (V.17).** As the rising sign rises (BJ I.10), head
first, back first, and Pisces both ways, which the commentator reads as the
hands; or by the lagna lord, direct a natural birth and retrograde an
irregular one, the reading Manittha supports.
- **Decided:** the rising sign by default, the core's own `Rising` table,
  which matches I.10 sign for sign; the lord's motion as the knob value,
  where a natural birth is the head and every other presentation agrees
  with an irregular one. The lord's motion is differenced over an hour
  either side (`ConceptionSky::graha_speed_deg`), and the façade's test
  holds it to the chart's own speed. V.17's malefics in the 4th or 7th (a
  hard labour) have no fact to weigh and are not read.

**X19. The attendants (V.22).** "Between the lagna and the Moon" may count
by degree or by sign; "some" put the visible half inside, and the
commentator says Varahamihira, in his *Swalpa Jataka*, does not. The
commentator also trebles a graha exalted or retrograde and doubles one in
its own sign, which the verse does not say.
- **Decided:** by degree in the zodiac's order from the rising degree by
  default, the seven only (the nodes are not grahas V.22 counts); by sign
  the lagna's own sign and the Moon's are not between. The visible half is
  outside by default. The commentator's multiplying is not shipped: it is
  his, not the verse's, and a family's count of women is weighed exactly,
  so a reading that multiplies would need its own knob and a source that
  states it.
- **Found while building:** the oil and the wick are the only clauses that
  see inside a sign, so they are the ones that can tell neighbouring
  minutes apart; each is read as the nearest of the gloss's three points
  (a sign's start, middle and end), so the levels change at 7.5° and 22.5°.

**X20. One call, not stages.** The design first put the baseline's stages
under `narrow` behind `Stages::baseline()`. Built, they share nothing with
it: the baseline sums log-likelihoods on a grid it refines three times by
posterior mass and answers the cells that carry a coverage of it, where
`narrow` cuts the window at clause edges, never sums, and answers what no
bar removed. One request type serving both would carry fields each side
ignores.
- **Decided:** `teistro_rectification::baseline::rectify` and
  `ChartArea::rectify_baseline`, with their own request and answer and no
  `Stages` member; the two answers are read side by side, never combined.

**X21. The first mahadasha's sub-periods.** The baseline squeezes the
balance of the birth mahadasha's antardashas in proportion, so all nine
fit inside the remainder, where the classical count runs them from the
start and cuts the elapsed part.
- **Decided:** reproduced through the dasha crate, not copied:
  `baseline_dasha_rules()` is Vimshottari with `BirthPeriod::Compressed`,
  spatial balance, Julian years and the cycle ending at its end, and is the
  request's default; any other `teistro_dasha::Rules` may be passed, so the
  classical cut is one field away.

**X22. Which periods an event is fitted to.** The baseline descends from
the mahadashas into the one period that best signifies the event at each
level, never the others overlapping it, and reads three levels only once
the grid step is two minutes or less.
- **Decided:** reproduced as is, since both choices move every score; the
  answer reports each pass's resolution, so the depth is visible.

**X23. Intervals from a grid.** Each pass's grid is built segment by
segment, half-open, with points two segments share kept twice; the
answer's intervals are runs of consecutive grid *index*, so two cells
either side of a gap between segments make one interval, and its end is
the last cell plus one step. With no prior and no event the posterior is
flat and the mode is the first cell, the window's start.
- **Decided:** reproduced as is, under the black-box run (X26); the answer
  carries the resolution and the concentration (one less the posterior's
  effective share of the candidates), so a flat answer reads as one.

**X24. The sex and the tattva cycle.** The baseline does not veto a
candidate whose tattva gives the other sex: it lowers it by 2.5 nepers. Its
cycle counts from the sunrise of the reported time's civil date and is not
re-anchored, so a birth before that sunrise reads negative minutes and the
reversed turns.
- **Decided:** reproduced as is (`Note::TattvaSex` reports when it moved
  anything); the sunrise is the SDK's own, which the black-box run holds to
  three seconds of the baseline's.

**X25. The frame.** The baseline's candidate charts are geocentric, with the
true node and Lahiri; the SDK's chart crossing is the context's.
- **Decided:** `rectify_baseline` reads the context's frame, so a
  consumer's own settings apply; the black-box run pins the baseline's
  (`conformance-baseline` with a geocentric centre and the true node).

**X26. The black-box run.** The acceptance below asked for each interval
within one cell of the baseline's final step.
- **Measured** (2026-10-09): the baseline engine run over eight cases
  (Pokhara with no events, every accuracy, both sexes, an uncertainty from
  an hour to twenty hours, five dated events; Kathmandu at dawn and New York
  with loose, ranged and held-out events). The SDK answers the same
  intervals, mode and top candidates to a second, the same posterior
  probabilities and concentration to a millionth and the same hold-out
  verdicts; its sunrise is within two seconds. The cases and answers are
  `crates/sdk/tests/rectify.rs`'s `CASES`.

## Acceptance

- Every clause in an answer names its verse and print.
- `Stages::default()` reaches no `BASELINE` value, a test asserts so.
- **The printed examples pass as printed:**
  - BPHS ch. 2 gloss, p. 12: a day of 33;14 ghatis, a Wednesday, gives a
    Gulika time of 16;37 under `END`, and the multiplier table reproduces cell
    for cell;
  - BPHS ch. 2 gloss, p. 13: 6;17 ghatis and the Sun at 2s 4°28′1″ give the
    1899 working figures, and 3s 4° under the named reading, with the
    differing cell asserted both ways (X3);
  - BJ IV.21 in Iyer's note: the Moon in Aquarius's 8th dvadashamsha gives
    Taurus under `NEXT_AFTER_DVADASHAMSHA`, and the half-way refinement
    gives Rohini.
- An interval's ends are clause edges, each re-judged a second either side
  and found to change.
- A window where no purifier holds anywhere answers no interval and says so,
  never the window.
- Every stage that ran and distinguished nothing is reported flat.
- **The black-box baseline run:** the baseline engine's rectification
  exported once over its regression cases (window, place, sex, events, and
  its intervals); `rectify_baseline` reproduces each interval's ends
  within one cell of the baseline's final grid step. **Held** to a second
  over every case, with no difference to count (X26).
- The measured page reports, over the corpus, the share of each day each
  bar removes and how many trusted times survive it (X8).

## Order of work

1. **Read what is still open on the page.** The 1899 page image of vv. 71–78
   word by word against 1923; Bhattotpala's IV.21 in the 1864 lithograph;
   Santhanam's numbering in the maintainer's print; a public-domain Kunda
   verse and a worked nisheka example if they exist.
2. **Measure the premises.** The verse's pranapada against the corpus point
   (X2), Gulika's start against its end on the corpus, and the share of a day
   the purifier bar removes over charts with trusted times (X8).
3. **The kernel** (`crates/rectification`): the `Stage` trait, verdicts,
   edges, interval assembly, and `PURIFIER` with the BPHS examples passing.
   It takes a sky per candidate and reads no ephemeris. **Built with
   `PURIFIER`** (2026-10-08): `narrow`, `Sky`, `Verdict` and the edge
   finder; the `Stage` trait waits for the second stage, so its shape is
   fixed by two implementations, not one.
4. **`PRANAPADA_HOUSE`, `NISHEKA` and `CONCEPTION_MOON`**, report and weight
   only, with BJ IV.21's example. **Built** (2026-10-09):
   `crates/rectification/src/conception.rs` and `ChartArea::conception`,
   with Jha's two worked examples (the pranapada in the 2nd by sign, p. 31;
   the nisheka to the second, p. 36) and BJ IV.21's printed counts as
   tests, and the façade's read-back against the charts of both instants.
   The pranapada's house counts by sign, which the example forces (its
   pranapada is 23°09′ past the lagna's degree and printed in the 2nd), and
   leaves the 1st inauspicious, as the gloss's list does.
5. **`CIRCUMSTANCE`** over BJ V.1–2, 17, 18 and 22, clause by clause.
   **Built** (2026-10-09): `crates/rectification/src/circumstance.rs` and
   `ChartArea::circumstance`, read at an instant as step 4's reports are.
   Every clause is reported whether or not a fact is given, and each fact
   given is one weight; none bars. The tests write skies down placement by
   placement (each of V.1's houses under both readings of X17, the hemmed
   Moon across 0°, every rising class, the lamp at its thresholds, the
   Moon on the lagna with no one between), and the façade's read-back holds
   every placement and the lord's motion to the candidate's chart.
6. **The `BASELINE` stages and the black-box export**, then the parity page.
   **Built** (2026-10-09): `crates/rectification/src/baseline.rs` and
   `ChartArea::rectify_baseline`, a call of its own (X20). The kernel tests
   hold the tattva cycle (its weekday starts, the reversed turns, the
   female share of an hour and a half), the grid's half-open segments, the
   runs by index, the likelihood's floor and the refusals; the façade's
   tests hold the black-box run (X26), a reported time that is the true one
   staying inside the answer, and events that narrow it. With nothing to
   differ, the parity page is the test's table.
7. **`TATTVA` under `SVARODAYA`**, opt-in. X12 is decided (2026-10-09).
8. **The façade and every binding**, on prashna's pattern: a request member,
   one section, parity across the runners.
