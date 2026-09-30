# Festival rules: deciding the day of an observance

Status: `building`, 2026-09-30: §6 step 1 built. Written from the source
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
- **Vijaya Dashami** (p. 72): aparahna. Later only: later. Both: earlier,
  whether or not Shravana joins. Where one day has Shravana at aparahna,
  that day. And the earlier day with aparahna only yields to a later day
  on which the 10th lasts three muhurtas and has Shravana.
- **Lakshmi puja** (p. 77): the new moon at pradosha. Later only or both:
  later. Earlier only: earlier. Neither: the text records two opinions.

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
    Lasts { day: Which, muhurtas: u8 },                        // after that day's sunrise
}
```

**No predicate is free text and none is open-ended.** The four rules
shipped are each a table of the six cases, plus a nakshatra joined and
a minimum length. `FestivalRule::check` refuses what could not be
judged, naming the field: the yugma verse asked of a tithi it pairs
with nothing (C174), a muhurta count outside a day's fifteen, or an
empty key.

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
   shipped pack at Kathmandu and Delhi for a span of years. It counts
   each case that occurred and each guard that decided. A guard no year
   reaches is listed with its reason, as the muhurta ABI test does for
   clause kinds.
3. **Against the published calendars.** The observed dates the
   governments of India and Nepal publish as holidays are facts. The
   pass compares each shipped rule's day with them. Every disagreement
   is named with its cause: another school, another window, a published
   day decided by administration. It is never absorbed as a tolerance.

## 6. Order of work

1. **Built**: the types, the evaluator (`teistro_panchanga::festival`)
   and the synthetic-day tests (§5.1), proved red by breaking a rule
   and the verse.
2. The façade over founded days, and the shipped rules through it.
3. The measured pass (§5.2, §5.3) and its page.
4. The boundary and the bindings, as muhurta step 7 did.
5. Ekadashi, both schools, held to pp. 11–12's examples.
