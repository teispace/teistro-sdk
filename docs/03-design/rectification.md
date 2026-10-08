# Rectification: the birth time narrowed by the verses that test it (the `rectification` module)

Status: `research`, 2026-10-08. Nothing built. The sources are read on
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
either stage, so they ship as rank 2 `BASELINE` stages, reached only when
asked, as C89's precedent requires.

## What the sources decide

The texts read, each on its page images unless marked:

- **BPHS, Subodhini prints**: Gyan, Mumbai 1899 and Khemaraj, Mumbai 1923,
  Purvakhanda ch. 2 vv. 67–78, pp. 11–13 (1923 leaves n40–n42). This is the
  roadmap's numbering. Sitaram Jha's print carries a different passage
  (its ch. 3 vv. 66–74) and is cited by verse number only.
- **BPHS, the nisheka lagna**: the same prints, ch. 3 vv. 25–30 (1923 p. 15);
  Jha ch. 4 vv. 24–30.
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

- **v. 193** gives the durations: earth 50 palas, water 40, fire 30, air 20,
  ether 10, an hour together, which is v. 62's span for one nadi's turn.
- Which nadi rises at sunrise goes by the tithi in runs of three days
  (vv. 61, 64), and the Moon's nadi is female, the Sun's male (v. 59).
- **No verse applies a tattva to the birth moment.** The sex rules read a
  tattva at a question about a pregnancy (1899 v. 294) or at conception
  (v. 299), and the two disagree about water.
- vv. 63 and 70 give other, equal durations, so the text is not one rule
  (X12).

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
| `BASELINE_TATTVA`, `BASELINE_EVENTS`, `BASELINE_PRIOR` | the baseline engine | rank 2, marked unsourced, reached only when asked | sex, dated events, reported time |

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
- `attendants`: `VISIBLE_OUTSIDE` (default, V.22) or `VISIBLE_INSIDE`;
- `tattva`: `OFF` (default), `SVARODAYA`, `BASELINE` (X12);
- `seed_step`: the grid step that seeds edge finding and the baseline
  comparison (X15).

`Stages::baseline()` sets every `BASELINE` value at once, for a migrating
consumer.

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
- `TATTVA` without a sex, `BASELINE_EVENTS` without events, a stage whose
  sections the chart did not carry.

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
- **Decided:** the trine for every purifier, the extension for Gulika only,
  as the verse attaches it to Gulika; each extension reported as its own
  clause.

**X7. "When the two are weak"** (v. 76). Weakness is not defined.
- **Decided:** do not define it. Judge every purifier, report each, and bar
  only when none holds (v. 75's *vā*). The answer says which purifier
  passed, so a caller who reads v. 76 as a precedence can apply it.

**X8. A bar or a weight.** v. 75's verb is *vijānīyāt*, "one should know it
impure": for a human native, a lagna no purifier holds is a contradiction.
- **Decided:** a bar by default (`purify_as: BAR`), a weight on request.
  Measure, on a corpus of births with trusted times, how much of each day the
  bar removes and whether trusted times survive it, before the default ships.

**X9. Day and night.** The pranapada counts palas from sunrise through the
night; Gulika's eighths switch to the night arc after sunset; a birth before
sunrise belongs to the previous day.
- **Decided:** the chart's own day (its sunrise, its ishtakaal, its
  `chart_day` rule) for every stage, so a stage never reckons a second day.

**X10. The conception check.** BPHS gives the conception instant, BJ IV.21
a test at conception; neither says to compose them. Its units (a sign a
month, a degree a day: solar, lunar or civil) and the 9th bhava's division
are unstated.
- **Decided:** `NISHEKA` reports the instant and its own purification;
  `CONCEPTION_MOON` is a weight; months of 30 days and Sripati's 9th bhava,
  each a knob; find a public-domain worked example before either weighs.

**X11. BJ IV.21's count.** From the sign after the dvadashamsha's sign
(Bhattotpala, per Iyer), from the Moon's sign (the 1912 main text, Gargi), or
from Aries; and the rising sign or navamsha.
- **Decided:** Bhattotpala's as the default, the rest as knob values; read
  the 1864 *Vivṛti* page before shipping, since both translators summarise it.

**X12. The tattvas.** v. 193's durations sum to v. 62's hour; vv. 63 and 70
give other spans; the sex rules belong to questions and conception and
disagree; the baseline's cycle (unequal minutes on 90, a weekday-lord start,
alternating direction, its own sex map) is a modern exposition. Whether the
roadmap's "Svarodaya v. 193" means the Shiva Svarodaya is itself an
inference.
- **Decided:** `TATTVA` off by default; `SVARODAYA` builds v. 193's
  durations inside v. 62's nadi turns from the tithi-run start of vv. 61 and
  64, with v. 59's sex by nadi, labelled as an application the text does not
  make; `BASELINE` reproduces the baseline's cycle as rank 2 with its own
  citation. Confirm with the maintainer which Svarodaya the roadmap meant.

**X13. Kunda.** The ×81 check is widely taught and attributed to Prasna
Marga; it was not found on a public-domain page.
- **Decided:** not shipped; listed as unread, so the gap is visible.

**X14. Life events.** Fitting dasha boundaries to dated events is the
baseline's main discriminator and has no classical text.
- **Decided:** `BASELINE_EVENTS` and `BASELINE_PRIOR` at rank 2, with the
  baseline's interval assembly reproduced only under `Stages::baseline()`;
  the classical default never sums likelihoods across stages.

**X15. Grid against edges.** The classical stages are piecewise constant, so
exact clause edges give exact intervals; the baseline answers on a refined
grid.
- **Decided:** exact edges for the answer; the seed grid only to find
  edges and to compare with the baseline, and the comparison allows one cell
  of the baseline's final step.

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
  its intervals); `Stages::baseline()` reproduces each interval's ends
  within one cell of the baseline's final grid step, every difference
  counted on a generated page.
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
   It takes a sky per candidate and reads no ephemeris.
4. **`PRANAPADA_HOUSE`, `NISHEKA` and `CONCEPTION_MOON`**, report and weight
   only, with BJ IV.21's example.
5. **`CIRCUMSTANCE`** over BJ V.1–2, 17, 18 and 22, clause by clause.
6. **The `BASELINE` stages and the black-box export**, then the parity page.
7. **`TATTVA` under `SVARODAYA`**, opt-in, after X12 is answered.
8. **The façade and every binding**, on prashna's pattern: a request member,
   one section, parity across the runners.
