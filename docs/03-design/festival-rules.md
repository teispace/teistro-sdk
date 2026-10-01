# Festival rules: deciding the day of an observance

Status: `building`, 2026-10-01: §6 steps 1 to 5 built, Ekadashi's (§8) held to the text's twelve examples and to four years of a published almanac, and Nepal's first four rules (§9) held to the committee's printed days. Written from the source
before any code; §4 is as built, and says where the building corrected
it.

Derives from `01-research/feature-universe/08-panchanga-calendar-muhurta.md`
(festivals and vratas "as calendar rules", P1 as a rule pack; its closing
checklist asks for "the festival rule-pack format" so a published
calendar can be both data and computed rules, with a diff), from
`02-architecture/04-calendar-time-architecture.md` ("festivals are
region-scoped rule packs … each rule cited; the engine never privileges
one region's determination"), and from the roadmap, which puts
"festival rule-pack **hooks**" in Phase 7 and the festival rule
**packs** in v1.x. This page is the hook: the format, the evaluator and
a first set of rules small enough to hold each one to its source.

## 1. What the source says

*Dharmasindhu* (Kashinatha Upadhyaya, 1790; rank 1), in the 1888
Nirnaya-sagara edition on the Internet Archive
(`in.ernet.dli.2015.513461`). The edition is out of copyright, and its OCR can be
searched though it garbles digits. Every number below was read off the
page image (the PDF's page is the printed page plus 23). The
condensed English on the Kanchi Kamakoti site is too thin to settle a
rule and is not cited.

**The governing principle** (pp. 5–6). An observance takes the tithi that
*pervades the time of its rite*, its **karmakala**: "कर्मणो यस्य यः
कालस्तत्कालव्यापिनी तिथिर्ग्राह्या". Vinayaka's vow is worshipped at
midday, so its tithi is the one present at midday. When the tithi
pervades that time on both days or on neither, or on part of it, the
day is settled by the **yugma** verse and the like.

**The yugma verse** (p. 6) pairs the tithis that are "of great fruit"
joined: 2 with 3, 4 with 5, 6 with 7, 8 with 9, 11 with 12, 14 with the
full moon, and the new moon with the 1st. The text spells the reading
out: the 2nd is taken joined to the 3rd, and the 3rd joined to the 2nd.
So the first of a pair is taken on the day it runs into its partner (the
**later** day), and the second on the day its partner runs into it (the
**earlier** day). The 10th and the 13th are in no pair. A festival's
own verse can overrule it: Ganesha's 4th is praised joined to the 3rd,
the pair's reverse.

**The parts of the day** (p. 6). The day is divided into five: *pratah*,
*sangava*, *madhyahna*, *aparahna* and *sayahna*. *Pradosha* is three
muhurtas after sunset. Janmashtami's section (p. 49) takes *nishitha*
as the middle of the night, and adds that "coarsely" it is the eighth
muhurta of the night. Ekadashi's (p. 11) puts *arunodaya* at four
ghatis before sunrise.

**Six cases, and two tables** (pp. 6–7). For the *ekabhakta*, the meal
taken once at midday, the text lists the cases between an earlier and a
later day:

| case | the tithi at the rite's time | *ekabhakta* (madhyahna) | *nakta* (pradosha) |
|---|---|---|---|
| 1 | on the earlier day only | earlier | earlier |
| 2 | on the later day only | later | later |
| 3 | wholly, on both | by yugma | later |
| 4 | on neither | earlier ("the secondary time is pervaded") | later |
| 5 | in part on both, equally | earlier | later |
| 6 | in part on both, unequally | by yugma if the tithi suffices for the rite on both, else earlier | the one with more, if the excess suffices for the rite, else later |

The two columns differ on four of the six cases. **What a rite decides is
a table, not a principle**, and the table is the rule's to carry.

**Worked examples, Janmashtami** (p. 49). The niśītha of the eighth
tithi, with its examples in ghatis after sunrise at which the 7th ends
and then the 8th ends on the following day:

| 7th ends | 8th ends | case | the text takes |
|---:|---:|---|---|
| 40 | 42 | earlier only | earlier |
| 47 | 46 | later only | later |
| 42 | 46 | both | later |
| 47 | 42 | neither | later |

On an equinoctial day of 30 ghatis, the night's middle is ghati 45 and
its eighth muhurta ghatis 44 to 46, and every row agrees with both. The
text also says the 8th counts as present at niśītha if "even a kala" of it stands at the end of
the night's first half. So niśītha is an **instant** there, not an interval.
A second table (p. 49–50) adds the Rohini nakshatra joined at
niśītha, which outranks the tithi alone.

**Worked examples, Ekadashi** (pp. 11–12). The 11th is *viddha*, pierced,
if the 10th touches its sunrise. It is touched at **arunodaya** for
Vaishnavas, the 10th lasting past 56 ghatis, and at **sunrise** for
Smartas. A pierced 11th is left and the 12th fasted. An unpierced one is
of four kinds, by which of the 11th and the 12th stand past the next
sunrise (*adhikya*). Each kind has an example and the schools
differ: with the 11th alone in excess Vaishnavas fast the later day and
Smarta householders the earlier. With both in excess, all fast the
later. With neither, all fast the earlier.

**The other festivals the first set carries**:

- **Rama Navami** (p. 33): madhyahna. Earlier only: earlier; on both or
  neither: later, because the 9th pierced by the 8th is forbidden, "even
  one pervading only part of madhyahna". Some would take the pierced one
  when the later day's 9th lasts less than three muhurtas.
- **Vijaya Dashami** (p. 71): aparahna. Later only: later. Both, or
  neither: earlier, when Shravana joins the 10th on both days or on
  neither. When it joins on one day only, that day. The earlier day with
  aparahna alone yields to a later day on which the 10th lasts three
  muhurtas and Shravana joins it *there only*. The Nirnaya-sindhu adds
  that Shravana must stand in that day's aparahna, and the text endorses
  it. In the author's own view, the later day with aparahna alone yields
  to the earlier when Shravana joins the 10th only on the earlier
  evening.
- **Lakshmi puja** (p. 77): the new moon at pradosha. Later only or both:
  later. Earlier only: earlier. Neither: the text records two opinions.
  And when the new moon holds the later day's sunrise and lasts more than
  a ghati past its sunset, the text says there is no doubt: the later
  day.

## 2. What the baseline engine does

It computes no festival. Its backend scrapes two published Nepali
calendars and reconciles each scraped event name with a curated
catalogue by exact alias, sending the rest to a triage queue rather
than guessing. The vratas, Ekadashis and sankrantis are deliberately
not catalogued. There is therefore nothing to migrate and nothing to
hold the SDK to. The closing checklist's "diff" between the published
calendar and the computed rules is what §5 measures.

## 3. Forks

| # | fork | default | the other reading |
|---|---|---|---|
| C168 | what a muhurta is inside a window | **a fifteenth** of the day or the night, the arc the almanac already cuts | two ghatis, 48 minutes, whatever the season |
| C169 | niśītha | **the night's middle**, an instant (p. 49's "even a kala") | the eighth muhurta of the night, an interval |
| C170 | "equally", in case 5 | **the extents equal to the second**, the comparison stated, never a tolerance invented | a tolerance the rule names |
| C171 | the month of an observance | **the nija month**: an adhika month holds no festival unless the rule says so | the rule names an adhika month as well |
| C172 | the two days | **the sunrise day the tithi begins in and the next**; when the tithi holds two sunrises, the two days it holds them on | — ; stated so that case 4, where neither window meets the tithi, still has an earlier and a later day, and checked against p. 49, whose "later only" example begins after the earlier day's niśītha |
| C173 | Lakshmi puja when neither day holds pradosha | **the earlier**, as when the earlier alone holds it (p. 77's "evam … abhave 'pi") | the *Purusharthachintamani*'s reading the same page records, which moves it to the later day when the new moon there lasts past three yamas |
| C175 | Janmashtami for the Vaishnava schools | **not shipped**: *Dharmasindhu* gives the Smarta rule, and the government's compulsory holiday is the Vaishnava day, the later one in each year the two part (`festival-measured.md` §1) | a Vaishnava rule taken from a Vaishnava text, which has not been read |
| C176 | a ghati inside a window | **a thirtieth** of the daylight after sunrise or of the night after sunset, as C168 takes a muhurta | 24 minutes whatever the season; the difference moves Lakshmi puja only when the new moon ends within minutes of a ghati past sunset |
| C174 | the yugma verse and the dark fortnight's 14th and 1st | **unpaired**: the verse joins the 14th to the full moon and the new moon to the 1st, which are the bright 14th and the bright 1st | pairing the dark 14th with the new moon and the dark 1st with the full moon, which the verse does not say |

The five-part day (p. 6) is the default of every day window. Aparahna in a
three-part day, the other common reckoning, is a window the rule may
name instead. It is not a fork of the evaluator.

## 4. The design

### 4.1 A rule

```rust
pub struct FestivalRule {
    pub key: String,          // "JANMASHTAMI", the pack's own key
    pub source: String,       // "Dharmasindhu pp. 49-50: …"
    pub month: Masa,          // under `convention`, amanta by default
    pub convention: Convention,
    pub tithi: Tithi,         // the paksha is the tithi's
    pub in_adhika: bool,      // C171
    pub at: Window,           // Sunrise | Part { part } | Pradosha | Nishitha
    pub decide: Vec<Guard>,   // the first whose predicates all hold decides
    pub otherwise: Choice,    // Earlier | Later | ByYugma
}

pub enum Predicate {
    Case { case: Case },                                       // the six of §1
    Joined { day: Which, nakshatra: Nakshatra, at: Option<Window> },
    Stands { day: Which, nakshatra: Nakshatra, at: Window },   // whatever the tithi
    Lasts { day: Which, from: Edge, ghatis: u8 },              // past its sunrise or sunset
}
```

**No predicate is free text and none is open-ended.** The four rules
shipped are each a table of the six cases, plus a nakshatra joined and
a minimum length. `FestivalRule::check` refuses what could not be
judged, naming the field: the yugma verse asked of a tithi it pairs
with nothing (C174), a ghati count outside a daylight's or a night's
thirty, or an empty key.

**What the building corrected.** The draft carried two more
predicates, *pierced* and *outlasts*, so that Ekadashi could be a guard
list too. Worked through its examples, it cannot be one. Its two days
are the eleventh's own sunrise day and the twelfth's, not the two days
the eleventh touches, and in two of its eight kinds the fast moves to a
day that pair does not contain. Its decisions also differ by school
*and* by stage of life. Ekadashi is §6 step 5, built and tested against
its own twelve examples. The draft's *greater extent* choice asked
whether the tithi is "enough for the rite", and that asks for a rite's
duration, which the text does not give. No shipped rule needs it, so it
is not offered.

The measured pass corrected two rules and one predicate. Lakshmi puja
shipped as "later unless the earlier alone, or neither, holds
pradosha", so a new moon that touched the later day's pradosha for
minutes took the later day. p. 77's "a ghati past sunset" is the
condition that decides it. `Lasts`, which counted muhurtas from sunrise,
now counts ghatis from sunrise or sunset (C176). Vijaya Dashami's
exception read Shravana joined on the later day where the text says
joined *there only*, and it lacked the Nirnaya-sindhu's condition,
which needs the nakshatra in a window after the tithi has ended. That
condition is `Stands`. Both were found against the published days, and
both are held by synthetic-day tests.

### 4.2 What is answered

```rust
pub struct Observance {
    pub rule: String,
    pub day: CalendarDate,
    pub tithi: Interval,        // the occurrence judged
    pub case: Case,
    pub extents: [Extent; 2],   // each day's window, and the fraction of it the tithi held
    pub decided_by: Decided,    // Guard { index } or Otherwise
    pub choice: Choice,
}
```

As in the muhurta search, the answer reports the clauses and names the
guard that decided, so a reader who follows another authority sees
exactly where it would part. An occurrence no day can be given to, at
the edge of the days or on a polar day, is listed in `unjudged` with
its reason, beside the observances.

### 4.3 The pack and the reach

A pack is a list of keyed rules. The SDK ships
`FestivalRule::dharmasindhu()`, the rules of §1, each citing its
page. A consumer spells out another as a JSON record the façade reads, the
way `MuhurtaRequest` reads activity rules, and may take the shipped one
and replace a rule by key. The façade answers
`sdk.almanac().festivals(from, to, place, clock, &pack)` over days it
founds once, beside the almanac (`festivals_with_days`), as the muhurta
search does.

The evaluator is pure over `FestivalDay` values, which a `Panchanga`
gives and a test writes by hand. The windows divide the day's own
daylight and night, the tithis and nakshatras are the `Limbs` spans
with their whole bounds, and the month is `LunarMonth`. An occurrence
beginning just before the range can fall inside it, and one at the end
needs the day after (two, when the tithi holds two sunrises). So the
façade widens the days it founds by one before and two after, and
reports that it did.

## 5. What is measured

1. **The page's own examples.** Janmashtami's four rows (and, with
   step 5, Ekadashi's twelve) are unit tests on a synthetic day of
   sixty ghatis (`crates/panchanga/tests/festival.rs`). The expected
   answer is the book's, not a reading of its prose.
2. **Every rule, every year, over a real sky.** A pass evaluates the
   shipped pack at Delhi, the place of the list it is held to, for a
   decade. It counts
   each case that occurred and each guard that decided. A guard no year
   reaches is listed with its reason, as the muhurta ABI test does for
   clause kinds.
3. **Against the published calendars.** The observed dates the
   Government of India publishes as holidays are facts. Nepal's list is
   not yet read. The
   pass compares each shipped rule's day with them. Every disagreement
   is named with its cause: another school, another window, a published
   day decided by administration. It is never absorbed as a tolerance.

## 6. Order of work

1. **Built**: the types, the evaluator (`teistro_panchanga::festival`)
   and the synthetic-day tests (§5.1), proved red by breaking a rule
   and the verse.
2. **Built**: `sdk.almanac().festivals(from, to, place, clock, &request)`
   over days founded once, widened by two days before and two after, so
   that a whole year can be asked for. A rule is refused by its place in
   the request (`rules[2].decide.when.ghatis`). Over 2026 at Delhi each
   shipped rule falls once, in its season, with nothing unjudged. The
   widening is founded as runs of its own (§7.1, step 4).
3. **Built**: the measured pass (`cargo xtask festival`, gated in
   fast-check) and `festival-measured.md`. It corrected two rules
   (§4.1). What remains apart from the published days is the Vaishnava
   Janmashtami (C175) and the 2024 Lakshmi puja. For that one the text
   gives the later day beyond doubt, and the list takes the earlier.
   The two guards no year of the decade reached each cite the crate test
   that reaches them, and the pass checks that the test exists.
4. **Built**: the boundary and the bindings (§7). `FestivalRequest`
   amends a pack rule by rule, `festivals_json` on the panchanga request
   carries it, and the `festivals` section answers it in every binding.
   `sdk.almanac().asked` founds the days once when muhurta and festivals
   are asked together. The parity runners ask the shipped pack and the
   pack amended by a rule of the runner's own, and all five agree.
5. **Built**: Ekadashi (§8), both schools, held to pp. 11–12's twelve
   examples and to a published almanac's Smarta, Gauna and Vaishnava
   dates (`festival-measured.md` §3). The `DHARMASINDHU` pack carries
   the three Ekadashi rules beside the karmakala ones, an item with a
   `vedha` is read as one, and every binding reads `ekadashis` back; the
   parity runners print each fast, and all five agree.

## 7. At the boundary

Derived from how the muhurta search crossed
(`muhurta-at-the-boundary.md`) and from reading the façade as built.

### 7.1 What reading the façade found

- **The days would be sealed twice over, differently.** A C request's
  `days` section is the caller's range, sealed as `of_each` seals it.
  The rules need a day before it and two after (§4.3). Slicing the
  widened run would carry its provenance, whose input hash names the
  wider range, so a binding would get days whose hash differs from
  `of_each`'s for the same range. **The widening is founded as three
  runs instead**: the day before, the caller's range, and the two days
  after. The middle run is exactly `of_each`'s, and nothing is founded
  twice. This also removes the chunking loop, because the caller's range
  is already held to the almanac's limit.
- **A pack cannot yet be amended, though §4.3 promises it.** In Rust a
  consumer edits the `Vec`. A binding sends a record, and spelling out
  four rules to change one is a dead end. The record takes a list whose
  items are either a pack's name or a rule. A later rule replaces an
  earlier one with the same key, in its place.
- **Two rules with one key are accepted today.** The answer names an
  observance by its rule's key, so two such rules would give two
  answers a reader cannot tell apart. The request refuses the second,
  naming `rules[i].key`.

### 7.2 The request

```rust
let asked = FestivalRequest::from(FestivalPack::Dharmasindhu).with_rule(my_lakshmi_puja);
let found = sdk.almanac().festivals(&from, &to, &place, clock, &asked)?;
let FestivalDays { days, day_hashes, answer } =
    sdk.almanac().festivals_with_days(&from, &to, &place, clock, &asked)?;
```

The record a binding writes as `festivals` (`festivals_json` in C) is
read by `FestivalRequest::from_json`:

| Key | Type | Default |
| --- | --- | --- |
| `rules` | `"DHARMASINDHU"`, or a list whose items are `"DHARMASINDHU"` or a `FestivalRule` in its serde spelling; a later rule replaces an earlier one with its key | required |

A catalogue member may be written bare or in full, as for every
request. A refusal names its path: `festivals.rules[2].decide`.

### 7.3 The answer

The `festivals` section holds canonical JSON, the envelope
`{value, provenance}` of `Observances`. Its catalogue members are
written in full: the day's calendar and era, the only ones the answer
names. The envelope is sealed over exactly those bytes, as muhurta's
is. The provenance carries `festival.days`, the widened range.

- **Node and wasm** hand it through deep-frozen as `almanac.festivals`,
  typed in `index.d.ts`. A typecheck fixture reads an observance's rule
  key and `decidedBy` back.
- **Python** builds frozen dataclasses, and **Dart** sealed final
  classes, as for muhurta.
- **The parity runners** print every observance (rule, day, case,
  guard, choice, the tithi's start and both extents) and the content
  hash, for the shipped pack and for the pack with one rule replaced,
  and all five must agree.

### 7.4 Errors

| Case | Error |
| --- | --- |
| Malformed `festivals_json`, an unknown pack or field | `INVALID_ARG` naming the key under `festivals` |
| A rule `check` refuses, or a key given twice | `INVALID_ARG` on `festivals.rules[i].<field>` |
| The range | the almanac's own refusal, which comes first |

## 8. Ekadashi

Derived from pp. 11–12, read off the page images (PDF pp. 34–35), before
any code; §6 step 5.

### 8.1 What the text says

**Two vedhas** (p. 11). *Arunodaya* is four ghatis before sunrise. The
10th entering "even a pala" past 56 ghatis pierces at arunodaya, which is
the Vaishnavas' concern. The 10th standing a pala past "the sixty-ghati
sunrise" pierces at sunrise, which is the Smartas'. Both counts run
from the 10th's own sunrise. A pierced (*viddha*) 11th is left, and for
Vaishnavas the 12th is fasted.

**Excess** (*adhikya*) is "standing past sunrise". The 11th is in excess
when it holds the next day's sunrise, and the 12th when it holds the
sunrise after that. That makes four kinds: the 11th alone, the 12th
alone, both, or neither. With the two vedhas there are eight.

**The notation.** Each example lists almanac lines, the ghati at which
a tithi ends past a sunrise. `60 । 1` is two lines: a tithi that fills
a day and ends a ghati past the next sunrise. *kshaya* marks one that
holds no sunrise at all.

| | 10th | 11th | 12th | vedha | excess | text |
|---|---:|---:|---:|---|---|---|
| V1 | 55 | 60 । 1 | kshaya 58 | none | 11th | Vaishnava later; Smarta householder earlier |
| V2 | 55 | 58 | 60 । 1 | none | 12th | Vaishnava the 12th's day; Smarta earlier |
| V3 | 55 | 60 । 1 | 5 | none | both | all later |
| V4 | 55 | 57 | 58 | none | neither | all earlier |
| S1 | 58 | 60 । 1 | kshaya 58 | arunodaya | 11th | householder earlier; renunciant, Vaishnava later |
| S2 | 4 | 2 | kshaya 58 | sunrise | 11th | as S1 |
| S3 | 58 | 60 । 1 | 4 | arunodaya | both | all later |
| S4 | 2 | 3 | 4 | sunrise | both | all later |
| S5 | 58 | 59 | 60 । 1 | arunodaya | 12th | Madhava: Smarta earlier; Hemadri: all later |
| S6 | 1 | kshaya 58 | 60 । 1 | sunrise | 12th | Smarta later "because pierced"; Vaishnava later |
| S7 | 57 | 58 | 59 | arunodaya | neither | Smarta earlier; Vaishnava later (57 > 56) |
| S8 | 2 | kshaya 56 | 59 | sunrise | neither | Smarta earlier; renunciant, Vaishnava later |

The text sums it up in its own words. With both in excess, or the 12th
alone, a Smarta leaves a pierced 11th, "and not otherwise". A Vaishnava
leaves every pierced kind that has an excess, and S7 and S8 show a
pierced one with neither is left too. The renunciant list covers yatis,
the desireless householder, the forest-dweller, widows and those
seeking release. They take the later day in S1, S2 and S8. Those
"desiring Vishnu's favour" fast both days, "some say".

**The closing** (p. 12). "Today" the learned set aside Hemadri's view
and the distinction of desire, and give every Smarta decision by
Madhava. "न तु क्वचिदुपवासद्वयं ... एकं परोपवासं वा": they give
*neither* two fasts *nor* one later fast for all in S5. The first
draft of this page dropped the "वा" and read it the other way. That
contradicted V2 and was refused by the published almanac (§8.4), which
is what sent the clause back to the page image (C182).

### 8.2 Which day is the 11th's (C181)

**D1 is the day in whose daylight the 11th begins; one beginning at
night belongs to the next day.** Equivalently, it is the first day
whose sunset follows the 11th's start. D2 is the day after. A kshaya
11th is at least 47 ghatis long and ends before a sunrise, so it
always begins in daylight. That means it needs no case of its own:
D1 is the day it runs in (S6, S8).

The first draft chose another rule that fits all twelve rows: "the day
after the 10th's own sunrise day", which makes S2's and S4's pierced D1
a vriddhi 10th's second sunrise. It rejected the rule above as moving
the fast off the sunrise-tithi day, "which no almanac does". Measured,
the almanac does. In 2024 (Papankusha) and 2025 (Devutthana) the 10th
ended about three hours after sunrise and the 11th held the next
sunrise for minutes. The almanac gives the Smarta the day the 11th
began in, and the renunciant and the Vaishnava the next, S2's pattern,
with no vriddhi 10th. The draft's rule parted on both.

### 8.3 The design

An Ekadashi rule is not a `FestivalRule`. Its two days are D1 and D2,
not the pair a karmakala rule uses. Its facts are a vedha and a kind,
not one of the six cases. What decides is a table:

```rust
pub struct EkadashiRule {
    pub key: String,          // "EKADASHI_SMARTA"
    pub source: String,
    pub vedha: Vedha,         // Arunodaya | Sunrise
    pub table: EkadashiTable, // pure: [Fast; 4], pierced: [Fast; 4]
}
pub enum Fast { Ekadashi, Dvadashi } // D1, D2
```

The kinds are ordered 11th alone, 12th alone, both, neither. The pack
ships the text's columns, each citing its rows:

| key | vedha | pure | pierced |
|---|---|---|---|
| `EKADASHI_VAISHNAVA` | arunodaya | D2 D2 D2 D1 | D2 D2 D2 D2 |
| `EKADASHI_SMARTA` | sunrise | D1 D1 D2 D1 | D1 D2 D2 D1 |
| `EKADASHI_SMARTA_RENUNCIANT` | sunrise | D2 D1 D2 D1 | D2 D2 D2 D2 |

In S5, `EKADASHI_SMARTA` takes Madhava's D1, which the closing says
present practice follows (C182). Hemadri's D2 for all is one cell a
consumer changes. The renunciant's D2 in S5 is only what "some say"
(केचित्), so the shipped column takes the householder's day. The
renunciants are not named in S7 either, so they keep the Smarta D1
there. The two-fast view is not shipped: the closing refuses it, and
a rule answers one day.

The answer is an `Ekadashi` observance beside the others: the rule, the
tithi (bright or dark 11th) and the lunar month, D1 and D2, the vedha
found (the 10th's end against D1's arunodaya and sunrise), the kind, the
cell that decided, and the day.

**As built** (`teistro_panchanga::festival::ekadashi`), three things moved:

- A cell is the shared `Which`, `EARLIER` for D1 and `LATER` for D2, not
  a `Fast` of its own, because a consumer who writes a karmakala rule
  already spells a day that way. The four kinds are named fields of an
  `EkadashiKinds` (`eleventh`, `twelfth`, `both`, `neither`), not an
  array, so a rule in JSON names each cell it changes.
- The answer is an `EkadashiFast` in its own list, `Observances.ekadashis`,
  beside the karmakala observances: its facts are a vedha and an excess,
  not a case and two extents. `piercedAt` is where the 10th reached,
  whatever the rule's vedha, and `pierced` whether that counts under it,
  so a fast read under one school says what another would see.
- The façade founds two days before a range, not one, since a fast whose
  day is the range's first may have its 10th two sunrises back (§8.4).

### 8.4 What is measured

- **The twelve rows** as synthetic sixty-ghati days, each under each
  shipped rule, asserting the text's day.
- **Drik Panchang's New Delhi lists** for 2023 to 2026: its Ekadashi
  list (the Smarta day, and a Gauna day where the renunciant's
  differs), and its ISKCON list for the Vaishnava. These are held
  against `EKADASHI_SMARTA`, `_RENUNCIANT` and `_VAISHNAVA` in
  `festival-measured.md` §3. The Smarta and renunciant rules fall on
  every published day. The Vaishnava rule parts on three, each a
  fortnight whose new moon holds two sunrises (ISKCON's *Paksha
  Vardhini Mahadvadashi*, a rule *Dharmasindhu* does not state), and
  the pass founds each fortnight to check that it is so.

**What the measuring corrected.** The first draft parted from the
record on 13 days and was wrong twice. It had read the closing as
"one later fast for all", against its "वा" (C182). And it had chosen
the D1 rule that the almanac refutes (C181). Both were corrected
from the page image and the record together; neither was absorbed.
The façade now founds two days before a range, not one: an 11th
beginning the day before is judged at an arunodaya in the night
before that.

## 9. Nepal

Roadmap Phase 7, step 4: the observances Nepal's national panchanga
prints, measured against the days the Nepal Panchanga Decision
Committee printed. Derived from the page images before any code; §9.2
is as built.

### 9.1 What the sources say

*Dharmasindhu* gives the rule for each observance below, and the
committee's national panchanga for VS 2082 and 2083 (npns.gov.np, the
only two years it hosts, with no text layer) prints the day each fell
on. Every clause was read off the page image (the PDF's page is the
printed page plus 23).

| rule | tithi | the text | window |
|---|---|---|---|
| `HARITALIKA` | Bhadrapada bright 3rd | p. 55: the later day whenever its sunrise holds the 3rd, "even less" than a muhurta of it, and even when the earlier day held all sixty ghatis, for the 4th joined to it (Ganesha's yoga); the earlier, joined to the 2nd, only when kshaya leaves the later day without it | sunrise |
| `NAVARATRA_ARAMBHA` | Ashwina bright 1st | p. 65: the 1st at sunrise and after it, three muhurtas ideally, two failing that, one "in some" texts; never on the day the new moon joins it, unless the later day holds it less than a muhurta or not at sunrise; the earlier when the 1st holds all sixty ghatis of it and grows two muhurtas into the next | sunrise |
| `YAMA_DWITIYA` | Kartika bright 2nd | p. 79: aparahna; the earlier day when it alone holds it, "in every other case" the later | aparahna |
| `SHIVARATRI` | Magha dark 14th (amanta) | p. 90: niśītha, which the page defines as "the night's eighth muhurta"; the later day when it alone holds it, the earlier when it alone does; neither, the later; both, wholly or in part, the earlier by the *Kaustubha* and the later by Madhava, the *Nirnayasindhu* and the *Purusharthachintamani*, "many"; the earlier holding it whole and the later in part, the earlier; the reverse, the later | the night's 8th muhurta |

Shivaratri is the first rule whose window is a stretch of the night
rather than an instant, and the first to weigh a whole holding against
a part. Janmashtami's niśītha stays the night's middle (C169), because
p. 49 counts "even a kala" there; p. 90 reasons in wholes and parts,
which an instant cannot.

### 9.2 The design

Two members, each closed:

- `Window::NightMuhurta { muhurta }`: the muhurta-th fifteenth of the
  night counted from sunset, 1 to 15, as C168 takes a muhurta.
  `FestivalRule::check` refuses one outside the fifteen wherever it
  stands (`at.muhurta`, `decide.when.at.muhurta`).
- `Predicate::Wholly { day }`: the tithi holds that day's window whole,
  or its instant. It is read off the extents the answer already
  reports, so nothing is computed twice.

The four rules join `FestivalRule::dharmasindhu()`, so the
`DHARMASINDHU` pack carries eight karmakala rules. Each is held to its
page's clauses on synthetic sixty-ghati days
(`crates/panchanga/tests/festival.rs`, proved red by breaking the
predicate and the window). The forks:

| # | fork | default | the other reading |
|---|---|---|---|
| C195 | Shivaratri when both nights hold niśītha | **the later**, with Madhava, the *Nirnayasindhu* and the *Purusharthachintamani*, whom p. 90 calls "many" | the *Kaustubha*'s earlier: one guard a consumer adds |
| C196 | how long the later day's 1st must last for Navaratra | **a muhurta** (two ghatis of the daylight), the least p. 65 allows before it gives the new moon's day | three muhurtas, the text's ideal: a ghati count a consumer changes |
| C197 | Nepal's Vijaya Dashami | **the aparahna rule of p. 71**, shipped since §6 step 3 | the day whose sunrise holds the 10th, which the committee printed in VS 2083 (§9.3); one year is not a rule |

### 9.3 What is measured

`festival-measured.md` §4 holds every shipped rule at Kathmandu, on
Nepal's clock, against the fifteen days the committee printed for VS
2082 and 2083. Each is found over the committee's own sky
(`nepali-committee`, the Surya Siddhanta with its bija, C187), which
the pass holds to the print, and over the modern sky
(`nepali-default`), which it counts beside it. A parting under the
committee's sky must carry its cause, and the cause is checked against
the case and guard found.

**What the measuring found.** The sky decides the day: the modern sky
parts from the print on four rows, and the committee's on one. Lakshmi
puja in VS 2082 is printed on the day of the 14th, 2025-10-20, because
the text's new moon ends before the next sunset. The modern one lasts
past it by more than a ghati, which p. 77 makes the later day.
Janmashtami (2025) and Rama Navami (2026) part the same way. The one
parting under the committee's sky is Vijaya Dashami in VS 2083. The
committee keeps the day whose sunrise the 10th holds (until 10:51 by its
print), where p. 71
gives the earlier day, which alone holds aparahna, with Shravana
joining the 10th that day only. It is named with its cause (C197) and
not absorbed: one year in two is not a rule, and a consumer who
follows the committee replaces the rule by its key.

The Delhi pass founds February too now, so that the decade's reach
counts Shivaratri.
