# Parana: when a fast may be broken

Status: `draft`, 2026-10-20. Research and design for the parana window,
written from the texts before any code. Nothing here is built. The
defaults marked **maintainer** in §3 are proposals until the maintainer
confirms them.

Derives from `festival-rules.md` §8 (Ekadashi: which day is fasted). That
page answers *which day*; this one answers *when, on the day after, the
fast is broken*, for Ekadashi and for the other vratas whose texts state a
parana. A vrata answer without it is incomplete: the observer's next
question is "when may I eat?".

## 1. What the sources say

Three texts, read off page images or an e-text checked against its
print:

- ***Dharmasindhu*** (rank 1), the 1888 Nirnaya-sagara edition on the
  Internet Archive (`in.ernet.dli.2015.513461`, PDF page = printed page
  + 23), the edition `festival-rules.md` reads. The parana passages were
  read off the page images of pp. 7, 19, 34, 51–52 and 71.
- ***Hari Bhakti Vilasa*** (rank 1 for the Vaishnava schools), vilasa 13
  (*viṣṇu-vratotsava*), with Sanatana Gosvami's *Dig-darśinī*, in the
  Jiva Institute's e-text of Haridas Shastri's edition (the Internet
  Archive's `hbv_vol2` is the print). The e-text numbers two verses 236
  and two 246; the numbers below are the e-text's.
- ***Nirnayasindhu*** (rank 1), the Nirnaya-sagara edition on the
  Internet Archive (`nirnayasindhukamalakarabhattanirnayasagarpress_837_f`),
  read from the archive's OCR text. **The page image was not checked**:
  the OCR's running heads do not match the scan's page order, so no page
  number is cited for it below; the passage is the one that follows the
  eight Mahadvadashis in the Ekadashi chapter.

### 1.1 Ekadashi

**Parana falls inside the 12th.** DS p. 19 (marginal head *pāraṇānirṇayaḥ*):
"पारणं च द्वादश्युल्लङ्घने महादोषाद् द्वादशीमध्य एव कार्यम्" — parana is
done within the 12th, since passing over the 12th is a great fault. HBV
13.236: "कुर्वीत द्वादशीमध्ये तुलसीं प्राश्य पारणम्"; vv. 238–244 quote the
Padma, Skanda, Kurma and Bhavishya Puranas against passing the 12th, v. 241
(Kurma): "त्रयोदश्यां न तत् कुर्याद् द्वादशद्वादशीक्षयात्".

**Hari vasara is the 12th's first quarter, and parana waits for it to
end.** DS p. 19: "द्वादशीभूयस्त्वे द्वादशीप्रथमपादं हरिवासरसंज्ञकमुल्लङ्घ्य पारणं
कार्यम्" — when the 12th is ample, parana is done after passing its first
quarter, called hari vasara. HBV 13.258 quotes the *Viṣṇudharmottara*:
"द्वादश्याः प्रथमः पादो हरिवासरसंज्ञकः । तमतिक्रम्य कुर्वीत पारणं विष्णुतत्परः";
NS quotes the same verse, through the *Nirṇayāmṛta* and the
*Madanaratna*. HBV 13.257 adds when it matters: if, by the 12th's excess,
hari vasara stands on the parana day, parana is not done in it. Sanatana's
gloss reads "excess" as the 12th running more than 45 ghatis past the
sunrise, which is the same as its first quarter outlasting the sunrise.
HBV 13.259 (*kvacit*, "somewhere") names a wider hari vasara, the 11th's
last quarter and the 12th's first together.

**Pratah, not madhyahna; aparahna as the other view.** DS p. 19: "द्वादश्यां
मध्याह्नोर्ध्वसत्त्वे प्रातर्मुहूर्तत्रयमध्य एव पारणम् न मध्याह्नादौ इति बहवः ।
बहूनां कर्मकालानां बाधापत्तेरपराह्ण एवेति केचित्" — when the 12th lasts past
midday, parana is within the first three muhurtas of the morning, not at
midday or the like, say *many*; *some* say in the aparahna only, since the
morning would obstruct the times of many rites. Three muhurtas of fifteen
are the first fifth of the daylight, the *pratah* of DS p. 6 and of
`DayPart::Pratah`. NS gives the same two views with the weight reversed:
"some" hold the morning ("सर्वेषामुपवासानां प्रातरेव हि पारणम्"), and "our
teachers" (*asmad-guravaḥ*) the aparahna, reading *prātaḥ* there as it is
read in "सायं प्रातर्द्विजातीनामशनम्".

**A short 12th.** DS p. 19: "स्वल्पद्वादशीसत्त्वे रात्रिशेषे आमाध्याह्नान्ताः क्रियाः
सर्वा अपकृष्य कार्याः" — with little of the 12th left, every rite up to the
midday's is drawn forward into the end of the night (the fire offering
and a shraddha excepted, since a shraddha is barred at night), so that the
parana can still fall inside the 12th. HBV 13.245–253 says the same with
more detail: the daily rites at arunodaya when the 12th is short; "after
midnight" (*niśīthordhvam*, v. 245; v. 250) when it is very short. In
distress, parana with water (DS p. 19 "तीर्थजलेन"; HBV 13.251, 254–256;
NS quoting Devala), which "is neither eating nor not eating".

**No 12th at all on the parana day.** DS p. 19: "कलामात्राया अपि द्वादश्या
अभावे त्रयोदश्यां पारणम्" — if not even a *kalā* of the 12th is available,
parana on the 13th. HBV 13.241–244 forbid a 13th parana, and its
short-12th verses move the rites into the night instead. That is the
crux the brief names (C652).

**A moved fast.** HBV 12 repeats it for every fast moved onto the 12th
(vv. 269, 277, 279, 287, 293, 319 and others): "उपोष्या द्वादशी शुद्धा
त्रयोदश्यां तु पारणम्" — fast on the 12th, parana on the 13th. And of the
eight Mahadvadashis, HBV 13.290–291 (*pāraṇa-kāla-nirṇaya*) decides the
parana by the joined nakshatra's end as well as the tithi's.

### 1.2 Every other vrata: the general rule

DS p. 7, closing the *ekabhakta* section: "एकभक्तनक्तायाचितोपवासानां पूर्वतिथावनुष्ठितानां
परेद्युस्तिथ्यन्ते पारणम् । यामत्रयोर्ध्वगामिन्यां तिथौ प्रातः पारणमिति सर्वत्र
ज्ञेयमिति माधवः" — for an *ekabhakta*, *nakta*, *ayācita* or fast kept on a
tithi, the parana is on the next day **at the tithi's end**; when the tithi
runs past three *yāmas*, parana **in the morning**: "so it is everywhere",
says Madhava. This is the rule a vrata takes when its own section says
nothing more.

### 1.3 The vratas whose sections say more

- **Janmashtami** (DS pp. 51–52). "On the second day the parana, the meal
  enjoined as a limb of the vrata": a fast on the tithi alone breaks at
  the tithi's end; one on the tithi joined with Rohini at the end of
  *both*. If one ends by day and both only at night, by day at the end of
  either. If neither ends by day, before niśītha at the end of either or
  both; if one or both end just before niśītha, even at niśītha. Some say
  not at niśītha but on the third day by day; "that is wrong". One unable
  may break it in the morning after the festival's close, having done the
  worship and its dismissal.
- **Rama Navami** (DS p. 34): after the night's vigil and the morning's
  worship, "नवम्यन्ते पारणां कुर्यात्" — at the 9th's end. The text also
  notes (p. 33) that when the 10th shrinks so that the parana day is
  lost, the Smartas keep the pierced 9th, "as for Ekadashi".
- **Navaratri** (DS p. 71). Parana follows the dismissal (*visarjana*),
  on the day of the dismissal, which is the 10th joined with Shravana.
  Others hold the parana on the 9th even when the dismissal is on the
  10th. The text then sets the cases out by which day holds a 9th long
  enough for the parana and which a 10th with Shravana, and ends on
  custom: when the 8th, 9th and 10th each fill a day, southerners break
  on the 9th, and those whose custom is the 10th do both on the 10th.
  Chaitra's Navaratri ends on Rama Navami "as Ashvina's" (p. 34).

### 1.4 What a published almanac does

The published almanac `festival-measured.md` §3 already holds the
Ekadashi days to prints a parana window for each fast and city, and a
note naming the moment that set its edge ("dwadashi end moment", "hari
vasara end moment", or "dwadashi would be over before sunrise"). Read for
146 fasts, 2025 to 2027, at New Delhi and at Kathmandu (Appendix A), every
window is reproduced to within two minutes by one rule, with sunrise and
sunset from a plain solar formula:

1. the window is the **pratah** of the parana day (sunrise to a fifth of
   the daylight), cut at its start by the end of hari vasara and at its
   end by the end of the 12th;
2. when hari vasara outlasts the pratah, the window is the **aparahna**,
   from the later of hari vasara's end and the madhyahna's end to the
   aparahna's end (39 of 146);
3. when the 12th is over before sunrise, the window is the pratah of
   that day, which the 13th holds (2 of 146).

So the almanac follows DS's *bahavaḥ* (pratah) first and its *kecit* (and
NS's teachers') aparahna as the fallback, never parana in the madhyahna,
and DS's 13th when no 12th is left. It publishes only the Smarta day's
parana on these pages, plus the Vaishnava day where its page lists one
(the 31 December 2025 fast).

## 2. The rule, clause by clause

Terms: *D* is the fast's day; *P* the parana day, the civil day after *D*.
*T* is the tithi the parana must fall in: the 12th after an Ekadashi
fasted on its own day, the 13th after a fast moved onto the 12th (HBV 12).
The day parts are the five of DS p. 6 over *P*'s daylight; a muhurta is a
fifteenth (C168).

| # | clause | sets | source |
|---|---|---|---|
| 1 | Parana is on *P*, after *P*'s sunrise | `from` ≥ sunrise | DS p. 19 ("prātaḥ"), every Ekadashi row of Appendix A |
| 2 | Not in hari vasara, the first quarter of *T*'s own span | `from` ≥ hari vasara's end | HBV 13.257–258 (*Viṣṇudharmottara*); DS p. 19; NS |
| 3 | Inside *T* | `to` ≤ *T*'s end | DS p. 19; HBV 13.236–244 |
| 4 | In the pratah (the first fifth) | the window, by default | DS p. 19 "*bahavaḥ*"; NS "*kecit*" |
| 5 | Never in the madhyahna | the window skips the third fifth | DS p. 19 "न मध्याह्नादौ" |
| 6 | When clause 2 leaves no pratah: the aparahna | the window | DS p. 19 "*kecit*"; NS's teachers; Appendix A rule 2 |
| 7 | When *T* is over before *P*'s sunrise | C652 | DS p. 19 (the 13th) against HBV 13.241–253 (inside *T*, before dawn) |
| 8 | A moved fast moves its parana | *P* is the day after the day fasted | HBV 12 (vv. 269 ff.); `festival-rules.md` §8.3 |

**Clause 6 under DS's own words** would make the aparahna the *only*
window (its *kecit*), and under NS's teachers too. The almanac's reading,
pratah first and aparahna when hari vasara forbids the pratah, is a
combination neither text states in one sentence; it is the default
proposed (C651) because it is what the published windows do, with both
texts' single readings offered.

**The window's two edges.** Every edge is one of: sunrise, hari vasara's
end, the pratah's end, the madhyahna's end, the aparahna's end, *T*'s
end, or (C652's other reading) a time before sunrise. The answer names
which, so a reader following another authority sees where it parts.

### 2.1 The other vratas

| vrata | `T` and the clause | source |
|---|---|---|
| any vrata kept on a tithi, *ekabhakta*, *nakta*, *ayācita* | the next day, from the tithi's end; if the tithi runs past three yamas of that day, the pratah | DS p. 7 (Madhava) |
| Janmashtami | the next day, from the 8th's end, or from the later of the 8th's and Rohini's ends for a fast joined with Rohini; by day if either ends by day; otherwise before niśītha | DS pp. 51–52 |
| Rama Navami | from the 9th's end | DS p. 34 |
| Navaratri, Ashvina and Chaitra | after the dismissal, on the dismissal's day (the 10th with Shravana), or on the 9th by custom | DS p. 71, p. 34 |
| a Mahadvadashi | by the tithi's and the joined nakshatra's ends | HBV 13.290–291; **not designed here** |

A *yāma* is a quarter of the daylight (a *prahara*), so "past three
yamas" is past the start of the daylight's last quarter (C659).

## 3. Forks

| # | fork | proposed default | the other reading |
|---|---|---|---|
| C650 | what hari vasara is | **the first quarter of the 12th's own span** (HBV 13.258 quoting the *Viṣṇudharmottara*; DS p. 19; NS) | HBV 13.259's *kvacit*: the 11th's last quarter with the 12th's first, which bars parana while an 11th in excess still stands at sunrise |
| C651 | the window when hari vasara allows the pratah, and when it does not | **maintainer**: proposed pratah, else aparahna, never madhyahna, as Appendix A does in all 146 | `PRATAH_ONLY` (DS p. 19's *bahavaḥ*: within the pratah or not at all; the window is then empty and the answer says so); `APARAHNA_ONLY` (DS's *kecit*, NS's teachers) |
| C652 | the 12th over before the parana day's sunrise | **maintainer**: proposed per observer, `EKADASHI_SMARTA` and `_RENUNCIANT` the 13th's pratah (DS p. 19, Appendix A's two cases), `EKADASHI_VAISHNAVA` inside the 12th before sunrise (HBV 13.241–253), the window from the later of the night's middle and the 12th's quarter to the 12th's end | the 13th for every observer; inside the 12th for every observer |
| C653 | a 12th that ends inside the pratah | **the window from the later of sunrise and hari vasara's end to the 12th's end** (Appendix A, Amalaki 2025 at Delhi: the 12th ends at 08:13 and so does the window); a quarter always ends before its tithi, so the window is never empty | — ; stated so that a window is never invented past *T* |
| C654 | water parana | **not a window**: reported as a note on every Ekadashi parana (DS p. 19, HBV 13.251–256), never chosen by the SDK | a separate "in distress" window, which no text times |
| C655 | the parana day of a moved fast | **the civil day after the day fasted**, with *T* the tithi after the one fasted (the 13th after a 12th) | — |
| C656 | which observers get a parana | **every Ekadashi rule**, each its own (`EKADASHI_SMARTA` and `_RENUNCIANT` differ when the fast differs) | the published almanac's pages give the Smarta day's parana first; nothing in the texts limits it |
| C657 | the madhyahna's edge when hari vasara ends inside it | **the aparahna starts at the madhyahna's end**, not at hari vasara's end (Appendix A: Delhi 2 November 2025, hari vasara ends 12:55, window from 13:11) | from hari vasara's end |
| C658 | the general rule's "tithi end" when the tithi ends at night | **the window opens at the tithi's end and runs to the parana day's sunset**; a tithi ending after sunset takes the next morning's pratah | the night itself, which DS p. 7 does not exclude for a *nakta* |
| C659 | a *yāma* | **a quarter of the daylight**, so "past three yamas" is past the start of the daylight's last quarter | an eighth of the whole day, three hours |
| C660 | Janmashtami's parana with Rohini | **DS pp. 51–52 as read**: the both-ends, either-end and niśītha cases; the third-day view refused as the text refuses it | the Gaudas' and Hemadri's readings NS sets out, not read in full |

## 4. The design

### 4.1 The answer

```rust
pub struct Parana {
    pub day: CalendarDate,      // P
    pub tithi: Tithi,           // T
    pub window: Option<Interval>, // None when C651's PRATAH_ONLY leaves nothing
    pub from: ParanaEdge,
    pub to: ParanaEdge,
    pub reading: ParanaReading, // the C651 / C652 choice that applied
    pub source: String,         // the rule's citation
}

pub enum ParanaEdge {
    Sunrise,
    HariVasaraEnds,
    PratahEnds,
    MadhyahnaEnds,
    AparahnaEnds,
    TithiEnds,          // T's end
    NakshatraEnds,      // Janmashtami with Rohini
    Nishitha,
    BeforeSunrise,      // C652's Vaishnava reading
    Sunset,
}
```

Each edge carries its instant (`ParanaEdge` as a tagged record
`{edge, at}` on the wire), so a binding reads "from 06:54 (hari vasara
ends) to 08:31 (pratah ends)" without recomputing either. `EkadashiFast`
gains `parana: Option<Parana>`; `None` is an unjudged day at the range's
end or a polar day, listed in `unjudged` with its reason as the fast
itself is.

### 4.2 The rule

```rust
pub struct ParanaRule {
    pub after: ParanaAfter,       // Ekadashi | TithiEnd | Janmashtami | Dismissal
    pub window: ParanaWindow,     // PratahElseAparahna | PratahOnly | AparahnaOnly
    pub no_tithi: NoTithi,        // NextTithiPratah | BeforeSunrise   (C652)
    pub hari_vasara: HariVasara,  // TwelfthFirstQuarter | WithEleventhLastQuarter (C650)
}
```

`EkadashiRule` gains `parana: ParanaRule` with a serde default, so a rule
written before it reads unchanged and the shipped three take C652's
per-observer default. A `FestivalRule` gains `parana: Option<ParanaRule>`,
`None` by default: an observance that is a feast, not a fast, has none.
The `DHARMASINDHU` pack's Janmashtami and Rama Navami take theirs; a
consumer's vrata takes `TithiEnd` (DS p. 7) by naming it.

The evaluator is pure over the `FestivalDay`s it already reads: *P*'s
sunrise, sunset and next sunrise, and *T*'s whole span from the limbs.
Hari vasara is `T.whole.at_fraction(0.25)`. A parana needs the day after
the fast (two after, for a fast on D2), which the façade's widening of
two days after a range already founds (`festival-rules.md` §4.3).

### 4.3 Navaratri

Navaratri's parana is a day choice before it is a window, and it reads a
dismissal the SDK does not model (the 10th joined with Shravana, and the
custom of the 9th or the 10th). It is designed as a `FollowingRule` with
a window once a Navaratri rule exists in a pack; until then it is listed
here and not built.

## 5. What is measured

1. **The texts' cases** as synthetic days: hari vasara ending before,
   inside and after the pratah; a 12th ending inside the pratah; a 12th
   over before sunrise under both C652 readings; a 12th fast's 13th
   parana; Janmashtami's four cases of DS p. 52.
2. **Appendix A** as a fixture: every row's window to the minute at
   Delhi on India's clock and at Kathmandu on Nepal's, under the default
   (C651, C652 Smarta), with the published edge's kind matched to
   `from`/`to`. The fixture is the brief's acceptance test: every
   Ekadashi of 2025–2027 at both places, or a named crux for each
   difference. The two 2026 adhika-month fasts are missing from the
   pages read (§A.2) and are to be added.
3. **The other readings** are counted over the same fasts, so the page
   says how often C651 and C652 change a window.

## 6. Order of work

1. The types and the evaluator in `teistro_panchanga::festival::parana`,
   the synthetic tests, proved red by breaking clause 2.
2. `EkadashiFast.parana` and `Observance.parana`, through every binding
   and the parity runners.
3. The Appendix A fixture in the festival pass (`cargo xtask festival`),
   gated in fast-check as the fast days are.

## Appendix A. Published parana windows, 2025 to 2027

Source: the published almanac `festival-measured.md` §3 holds the
Ekadashi days to, its per-fast pages, read 2026-10-20 with the city set
to New Delhi (on India's clock, UTC+05:30) and to Kathmandu (Nepal's,
UTC+05:45). Times are as printed, to the minute. "The published edge" is
the page's own note on the parana day. Every row is reproduced within
two minutes by §1.4's rule with a plain solar formula's sunrise and
sunset; the SDK's own sunrise is what the fixture will use.

### A.1 The windows

| fast (civil date) | Ekadashi | place | parana day | from | to | the published edge |
|---|---|---|---|---|---|---|
| 2025-01-10 | Pausha Putrada | Delhi | 2025-01-11 | 07:15 | 08:21 | dwadashi ends 08:21 |
| 2025-01-10 | Pausha Putrada | Kathmandu | 2025-01-11 | 06:56 | 08:36 | dwadashi ends 08:36 |
| 2025-01-25 | Shattila | Delhi | 2025-01-26 | 07:12 | 09:21 | dwadashi ends 20:54 |
| 2025-01-25 | Shattila | Kathmandu | 2025-01-26 | 06:53 | 09:02 | dwadashi ends 21:09 |
| 2025-02-08 | Jaya | Delhi | 2025-02-09 | 07:04 | 09:17 | dwadashi ends 19:25 |
| 2025-02-08 | Jaya | Kathmandu | 2025-02-09 | 06:46 | 08:59 | dwadashi ends 19:40 |
| 2025-02-24 | Vijaya | Delhi | 2025-02-25 | 06:50 | 09:08 | dwadashi ends 12:47 |
| 2025-02-24 | Vijaya | Kathmandu | 2025-02-25 | 06:32 | 08:50 | dwadashi ends 13:02 |
| 2025-03-10 | Amalaki | Delhi | 2025-03-11 | 06:35 | 08:13 | dwadashi ends 08:13 |
| 2025-03-10 | Amalaki | Kathmandu | 2025-03-11 | 06:18 | 08:28 | dwadashi ends 08:28 |
| 2025-03-25 | Papamochani | Delhi | 2025-03-26 | 13:41 | 16:08 | hari vasara ends 09:14 |
| 2025-03-25 | Papamochani | Kathmandu | 2025-03-26 | 13:23 | 15:51 | hari vasara ends 09:29 |
| 2025-04-08 | Kamada | Delhi | 2025-04-09 | 06:02 | 08:34 | dwadashi ends 22:55 |
| 2025-04-08 | Kamada | Kathmandu | 2025-04-09 | 05:45 | 08:17 | dwadashi ends 23:10 |
| 2025-04-24 | Varuthini | Delhi | 2025-04-25 | 05:46 | 08:23 | dwadashi ends 11:44 |
| 2025-04-24 | Varuthini | Kathmandu | 2025-04-25 | 05:29 | 08:06 | dwadashi ends 11:59 |
| 2025-05-08 | Mohini | Delhi | 2025-05-09 | 05:34 | 08:16 | dwadashi ends 14:56 |
| 2025-05-08 | Mohini | Kathmandu | 2025-05-09 | 05:18 | 07:59 | dwadashi ends 15:11 |
| 2025-05-23 | Apara | Delhi | 2025-05-24 | 05:26 | 08:11 | dwadashi ends 19:20 |
| 2025-05-23 | Apara | Kathmandu | 2025-05-24 | 05:11 | 07:55 | dwadashi ends 19:35 |
| 2025-06-06 | Nirjala | Delhi | 2025-06-07 | 13:44 | 16:31 | hari vasara ends 11:25 |
| 2025-06-06 | Nirjala | Kathmandu | 2025-06-07 | 13:26 | 16:12 | hari vasara ends 11:40 |
| 2025-06-21 | Yogini | Delhi | 2025-06-22 | 13:47 | 16:35 | hari vasara ends 09:41 |
| 2025-06-21 | Yogini | Kathmandu | 2025-06-22 | 13:29 | 16:16 | hari vasara ends 09:56 |
| 2025-07-06 | Devshayani | Delhi | 2025-07-07 | 05:29 | 08:16 | dwadashi ends 23:10 |
| 2025-07-06 | Devshayani | Kathmandu | 2025-07-07 | 05:14 | 08:00 | dwadashi ends 23:25 |
| 2025-07-21 | Kamika | Delhi | 2025-07-22 | 05:37 | 07:05 | dwadashi ends 07:05 |
| 2025-07-21 | Kamika | Kathmandu | 2025-07-22 | 05:21 | 07:20 | dwadashi ends 07:20 |
| 2025-08-05 | Shravana Putrada | Delhi | 2025-08-06 | 05:45 | 08:26 | dwadashi ends 14:08 |
| 2025-08-05 | Shravana Putrada | Kathmandu | 2025-08-06 | 05:29 | 08:09 | dwadashi ends 14:23 |
| 2025-08-19 | Aja | Delhi | 2025-08-20 | 05:53 | 08:29 | dwadashi ends 13:58 |
| 2025-08-19 | Aja | Kathmandu | 2025-08-20 | 05:37 | 08:13 | dwadashi ends 14:13 |
| 2025-09-03 | Parsva | Delhi | 2025-09-04 | 13:36 | 16:07 | hari vasara ends 10:18 |
| 2025-09-03 | Parsva | Kathmandu | 2025-09-04 | 13:18 | 15:50 | hari vasara ends 10:33 |
| 2025-09-17 | Indira | Delhi | 2025-09-18 | 06:07 | 08:34 | dwadashi ends 23:24 |
| 2025-09-17 | Indira | Kathmandu | 2025-09-18 | 05:50 | 08:17 | dwadashi ends 23:39 |
| 2025-10-03 | Papankusha | Delhi | 2025-10-04 | 06:16 | 08:37 | dwadashi ends 17:09 |
| 2025-10-03 | Papankusha | Kathmandu | 2025-10-04 | 05:58 | 08:20 | dwadashi ends 17:24 |
| 2025-10-17 | Rama | Delhi | 2025-10-18 | 06:24 | 08:41 | dwadashi ends 12:18 |
| 2025-10-17 | Rama | Kathmandu | 2025-10-18 | 06:05 | 08:23 | dwadashi ends 12:33 |
| 2025-11-01 | Devutthana | Delhi | 2025-11-02 | 13:11 | 15:23 | hari vasara ends 12:55 |
| 2025-11-01 | Devutthana | Kathmandu | 2025-11-02 | 13:10 | 15:06 | hari vasara ends 13:10 |
| 2025-11-15 | Utpanna | Delhi | 2025-11-16 | 13:10 | 15:18 | hari vasara ends 09:09 |
| 2025-11-15 | Utpanna | Kathmandu | 2025-11-16 | 12:53 | 15:02 | hari vasara ends 09:24 |
| 2025-12-01 | Mokshada | Delhi | 2025-12-02 | 06:57 | 09:03 | dwadashi ends 15:57 |
| 2025-12-01 | Mokshada | Kathmandu | 2025-12-02 | 06:38 | 08:44 | dwadashi ends 16:12 |
| 2025-12-15 | Saphala | Delhi | 2025-12-16 | 07:07 | 09:11 | dwadashi ends 23:57 |
| 2025-12-15 | Saphala | Kathmandu | 2025-12-16 | 06:47 | 08:52 | dwadashi ends 00:12 (after midnight) |
| 2025-12-30 | Pausha Putrada | Delhi | 2025-12-31 | 13:26 | 15:31 | hari vasara ends 10:12 |
| 2025-12-30 | Pausha Putrada | Kathmandu | 2025-12-31 | 13:09 | 15:14 | hari vasara ends 10:27 |
| 2025-12-31 | Vaikuntha | Delhi | 2026-01-01 | 07:14 | 09:18 | dwadashi over before sunrise |
| 2025-12-31 | Vaikuntha | Kathmandu | 2026-01-01 | 06:55 | 09:00 | dwadashi over before sunrise |
| 2026-01-14 | Shattila | Delhi | 2026-01-15 | 07:15 | 09:21 | dwadashi ends 20:16 |
| 2026-01-14 | Shattila | Kathmandu | 2026-01-15 | 06:56 | 09:03 | dwadashi ends 20:31 |
| 2026-01-29 | Jaya | Delhi | 2026-01-30 | 07:10 | 09:20 | dwadashi ends 11:09 |
| 2026-01-29 | Jaya | Kathmandu | 2026-01-30 | 06:52 | 09:02 | dwadashi ends 11:24 |
| 2026-02-13 | Vijaya | Delhi | 2026-02-14 | 07:00 | 09:14 | dwadashi ends 16:01 |
| 2026-02-13 | Vijaya | Kathmandu | 2026-02-14 | 06:42 | 08:56 | dwadashi ends 16:16 |
| 2026-02-27 | Amalaki | Delhi | 2026-02-28 | 06:47 | 09:06 | dwadashi ends 20:43 |
| 2026-02-27 | Amalaki | Kathmandu | 2026-02-28 | 06:29 | 08:48 | dwadashi ends 20:58 |
| 2026-03-15 | Papamochani | Delhi | 2026-03-16 | 06:30 | 08:54 | dwadashi ends 09:40 |
| 2026-03-15 | Papamochani | Kathmandu | 2026-03-16 | 06:12 | 08:36 | dwadashi ends 09:55 |
| 2026-03-29 | Kamada | Delhi | 2026-03-30 | 06:14 | 07:09 | dwadashi ends 07:09 |
| 2026-03-29 | Kamada | Kathmandu | 2026-03-30 | 05:57 | 07:24 | dwadashi ends 07:24 |
| 2026-04-13 | Varuthini | Delhi | 2026-04-14 | 06:54 | 08:31 | hari vasara ends 06:54 |
| 2026-04-13 | Varuthini | Kathmandu | 2026-04-14 | 07:09 | 08:14 | hari vasara ends 07:09 |
| 2026-04-27 | Mohini | Delhi | 2026-04-28 | 05:43 | 08:21 | dwadashi ends 18:51 |
| 2026-04-27 | Mohini | Kathmandu | 2026-04-28 | 05:27 | 08:05 | dwadashi ends 19:06 |
| 2026-05-13 | Apara | Delhi | 2026-05-14 | 05:31 | 08:14 | dwadashi ends 11:20 |
| 2026-05-13 | Apara | Kathmandu | 2026-05-14 | 05:15 | 07:57 | dwadashi ends 11:35 |
| 2026-06-25 | Nirjala | Delhi | 2026-06-26 | 05:25 | 08:13 | dwadashi ends 22:22 |
| 2026-06-25 | Nirjala | Kathmandu | 2026-06-26 | 05:10 | 07:57 | dwadashi ends 22:37 |
| 2026-07-10 | Yogini | Delhi | 2026-07-11 | 13:50 | 16:36 | hari vasara ends 10:32 |
| 2026-07-10 | Yogini | Kathmandu | 2026-07-11 | 13:32 | 16:17 | hari vasara ends 10:47 |
| 2026-07-25 | Devshayani | Delhi | 2026-07-26 | 05:39 | 08:22 | dwadashi ends 13:57 |
| 2026-07-25 | Devshayani | Kathmandu | 2026-07-26 | 05:23 | 08:06 | dwadashi ends 14:12 |
| 2026-08-09 | Kamika | Delhi | 2026-08-10 | 05:47 | 08:00 | dwadashi ends 08:00 |
| 2026-08-09 | Kamika | Kathmandu | 2026-08-10 | 05:31 | 08:10 | dwadashi ends 08:15 |
| 2026-08-23 | Shravana Putrada | Delhi | 2026-08-24 | 13:41 | 16:16 | hari vasara ends 10:49 |
| 2026-08-23 | Shravana Putrada | Kathmandu | 2026-08-24 | 13:23 | 15:58 | hari vasara ends 11:04 |
| 2026-09-07 | Aja | Delhi | 2026-09-08 | 06:02 | 08:33 | dwadashi ends 14:42 |
| 2026-09-07 | Aja | Kathmandu | 2026-09-08 | 05:46 | 08:16 | dwadashi ends 14:57 |
| 2026-09-22 | Parsva | Delhi | 2026-09-23 | 06:10 | 08:35 | dwadashi ends 22:50 |
| 2026-09-22 | Parsva | Kathmandu | 2026-09-23 | 05:52 | 08:18 | dwadashi ends 23:05 |
| 2026-10-06 | Indira | Delhi | 2026-10-07 | 06:17 | 08:38 | dwadashi ends 23:16 |
| 2026-10-06 | Indira | Kathmandu | 2026-10-07 | 06:29 | 08:20 | hari vasara ends 06:29 |
| 2026-10-22 | Papankusha | Delhi | 2026-10-23 | 06:27 | 08:42 | dwadashi ends 14:35 |
| 2026-10-22 | Papankusha | Kathmandu | 2026-10-23 | 06:08 | 08:24 | dwadashi ends 14:50 |
| 2026-11-05 | Rama | Delhi | 2026-11-06 | 06:37 | 08:48 | dwadashi ends 10:30 |
| 2026-11-05 | Rama | Kathmandu | 2026-11-06 | 06:18 | 08:30 | dwadashi ends 10:45 |
| 2026-11-20 | Devutthana | Delhi | 2026-11-21 | 13:10 | 15:18 | hari vasara ends 12:07 |
| 2026-11-20 | Devutthana | Kathmandu | 2026-11-21 | 12:53 | 15:02 | hari vasara ends 12:22 |
| 2026-12-04 | Utpanna | Delhi | 2026-12-05 | 06:59 | 09:04 | dwadashi ends 00:51 (after midnight) |
| 2026-12-04 | Utpanna | Kathmandu | 2026-12-05 | 06:40 | 08:46 | dwadashi ends 01:06 (after midnight) |
| 2026-12-20 | Mokshada | Delhi | 2026-12-21 | 07:10 | 09:13 | dwadashi ends 17:36 |
| 2026-12-20 | Mokshada | Kathmandu | 2026-12-21 | 06:50 | 08:55 | dwadashi ends 17:51 |
| 2027-01-03 | Saphala | Delhi | 2027-01-04 | 07:15 | 09:19 | dwadashi ends 18:15 |
| 2027-01-03 | Saphala | Kathmandu | 2027-01-04 | 06:55 | 09:01 | dwadashi ends 18:30 |
| 2027-01-18 | Pausha Putrada | Delhi | 2027-01-19 | 13:35 | 15:42 | hari vasara ends 13:02 |
| 2027-01-18 | Pausha Putrada | Kathmandu | 2027-01-19 | 13:18 | 15:26 | hari vasara ends 13:17 |
| 2027-02-02 | Shattila | Delhi | 2027-02-03 | 07:08 | 09:19 | dwadashi ends 13:50 |
| 2027-02-02 | Shattila | Kathmandu | 2027-02-03 | 06:50 | 09:01 | dwadashi ends 14:05 |
| 2027-02-17 | Jaya | Delhi | 2027-02-18 | 06:57 | 09:12 | dwadashi ends 14:27 |
| 2027-02-17 | Jaya | Kathmandu | 2027-02-18 | 06:39 | 08:54 | dwadashi ends 14:42 |
| 2027-03-04 | Vijaya | Delhi | 2027-03-05 | 06:42 | 09:03 | dwadashi ends 09:53 |
| 2027-03-04 | Vijaya | Kathmandu | 2027-03-05 | 06:25 | 08:45 | dwadashi ends 10:08 |
| 2027-03-18 | Amalaki | Delhi | 2027-03-19 | 07:12 | 08:52 | hari vasara ends 07:12 |
| 2027-03-18 | Amalaki | Kathmandu | 2027-03-19 | 07:27 | 08:34 | hari vasara ends 07:27 |
| 2027-04-02 | Papamochani | Delhi | 2027-04-03 | 13:40 | 16:10 | hari vasara ends 09:15 |
| 2027-04-02 | Papamochani | Kathmandu | 2027-04-03 | 13:22 | 15:52 | hari vasara ends 09:30 |
| 2027-04-17 | Kamada | Delhi | 2027-04-18 | 05:53 | 07:42 | dwadashi ends 07:42 |
| 2027-04-17 | Kamada | Kathmandu | 2027-04-18 | 05:37 | 07:57 | dwadashi ends 07:57 |
| 2027-05-02 | Varuthini | Delhi | 2027-05-03 | 05:39 | 08:19 | dwadashi ends 20:03 |
| 2027-05-02 | Varuthini | Kathmandu | 2027-05-03 | 05:23 | 08:02 | dwadashi ends 20:18 |
| 2027-05-16 | Mohini | Delhi | 2027-05-17 | 05:29 | 08:13 | dwadashi ends 16:28 |
| 2027-05-16 | Mohini | Kathmandu | 2027-05-17 | 05:14 | 07:56 | dwadashi ends 16:43 |
| 2027-06-01 | Apara | Delhi | 2027-06-02 | 05:23 | 08:10 | dwadashi ends 08:28 |
| 2027-06-01 | Apara | Kathmandu | 2027-06-02 | 05:08 | 07:54 | dwadashi ends 08:43 |
| 2027-06-14 | Nirjala | Delhi | 2027-06-15 | 13:45 | 16:33 | hari vasara ends 08:13 |
| 2027-06-14 | Nirjala | Kathmandu | 2027-06-15 | 13:28 | 16:14 | hari vasara ends 08:28 |
| 2027-06-30 | Yogini | Delhi | 2027-07-01 | 05:27 | 08:14 | dwadashi ends 18:14 |
| 2027-06-30 | Yogini | Kathmandu | 2027-07-01 | 05:11 | 07:58 | dwadashi ends 18:29 |
| 2027-07-14 | Devshayani | Delhi | 2027-07-15 | 05:33 | 08:19 | dwadashi ends 14:40 |
| 2027-07-14 | Devshayani | Kathmandu | 2027-07-15 | 05:18 | 08:02 | dwadashi ends 14:55 |
| 2027-07-29 | Kamika | Delhi | 2027-07-30 | 13:49 | 16:31 | hari vasara ends 10:29 |
| 2027-07-29 | Kamika | Kathmandu | 2027-07-30 | 13:31 | 16:13 | hari vasara ends 10:44 |
| 2027-08-12 | Shravana Putrada | Delhi | 2027-08-13 | 13:45 | 16:24 | hari vasara ends 09:34 |
| 2027-08-12 | Shravana Putrada | Kathmandu | 2027-08-13 | 13:28 | 16:06 | hari vasara ends 09:49 |
| 2027-08-28 | Aja | Delhi | 2027-08-29 | 05:57 | 08:31 | dwadashi ends 09:35 |
| 2027-08-28 | Aja | Kathmandu | 2027-08-29 | 05:41 | 08:14 | dwadashi ends 09:50 |
| 2027-09-11 | Parsva | Delhi | 2027-09-12 | 06:04 | 08:33 | dwadashi ends 22:12 |
| 2027-09-11 | Parsva | Kathmandu | 2027-09-12 | 05:47 | 08:16 | dwadashi ends 22:27 |
| 2027-09-26 | Indira | Delhi | 2027-09-27 | 06:12 | 08:36 | dwadashi ends 16:55 |
| 2027-09-26 | Indira | Kathmandu | 2027-09-27 | 05:54 | 08:18 | dwadashi ends 17:10 |
| 2027-10-11 | Papankusha | Delhi | 2027-10-12 | 06:20 | 08:39 | dwadashi ends 16:13 |
| 2027-10-11 | Papankusha | Kathmandu | 2027-10-12 | 06:02 | 08:21 | dwadashi ends 16:28 |
| 2027-10-25 | Rama | Delhi | 2027-10-26 | 13:12 | 15:27 | hari vasara ends 08:47 |
| 2027-10-25 | Rama | Kathmandu | 2027-10-26 | 12:55 | 15:10 | hari vasara ends 09:02 |
| 2027-11-10 | Devutthana | Delhi | 2027-11-11 | 06:40 | 08:50 | dwadashi ends 10:07 |
| 2027-11-10 | Devutthana | Kathmandu | 2027-11-11 | 06:21 | 08:32 | dwadashi ends 10:22 |
| 2027-11-24 | Utpanna | Delhi | 2027-11-25 | 06:51 | 08:58 | dwadashi ends 10:50 |
| 2027-11-24 | Utpanna | Kathmandu | 2027-11-25 | 06:32 | 08:39 | dwadashi ends 11:05 |
| 2027-12-09 | Mokshada | Delhi | 2027-12-10 | 13:16 | 15:20 | hari vasara ends 09:13 |
| 2027-12-09 | Mokshada | Kathmandu | 2027-12-10 | 12:59 | 15:04 | hari vasara ends 09:28 |
| 2027-12-23 | Saphala | Delhi | 2027-12-24 | 07:11 | 09:15 | dwadashi ends 23:08 |
| 2027-12-23 | Saphala | Kathmandu | 2027-12-24 | 06:51 | 08:56 | dwadashi ends 23:23 |

### A.2 Gaps

- The two fasts of 2026's adhika Jyeshtha (Padmini and Parama) have no
  page in the list read; they are to be read and added.
- Where a Vaishnava or Gauna fast falls a day after the Smarta one, only
  the 31 December 2025 Vaishnava parana is in the table; the others are
  to be read from the Vaishnava list `festival-measured.md` §3 holds.
