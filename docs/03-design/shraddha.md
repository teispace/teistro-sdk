# Shraddha, barkhi and janma tithi: personal lunar anniversaries

Status: `building`, 2026-10-22. Steps 1–3 of §6 are built: the windows
and the choice, the origin, and anniversaries through every binding
(§3.1). The `PITRI_PAKSHA` pack (§3.2) and the first-year schedule (C673)
are not yet. The **maintainer** forks in §4 ship as parameters with the
researched default, as the maintainer delegated (C662 muhurtas 8–9, C666
the adhika month with `NIJA` for Nepal); C673 waits.

A family keeps a death anniversary (barkhi, *pratyābdika* or
*sāṃvatsarika śrāddha*) and a birthday (janma tithi, *vardhāpana*) by the
lunar date, and keeps the fortnight of the ancestors (Sora Shraddha,
*pitṛ-pakṣa*, *mahālaya*) by the death tithi. All three are a day chosen
by a tithi's coverage of a rite's time, which is what the festival
evaluator does (`festival-rules.md`); what it lacks is the origin (a
tithi fixed at a moment of death or birth), two windows, one choice, and
the adhika and kshaya policies that depend on the year counted from the
origin. This page sets out what the texts say, the forks, and how each
maps onto `FestivalRule` or what it needs added.

## 1. Sources

| tag | work | edition read | where |
|---|---|---|---|
| DS | *Dharmasindhu* (Kashinatha Upadhyaya, 1790), rank 1 | 1888 Nirnaya-sagara edition (`in.ernet.dli.2015.513461`, PDF page = printed page + 23), the edition `festival-rules.md` reads | page images of pp. 265–266 read; pp. 61–64, 329–331 and the malamasa pages read from the scan's OCR, page images not yet checked |
| NS | *Nirnayasindhu* (Kamalakara Bhatta, 1612), rank 1 | with Venimadhav Dhakal's Nepali *Cūḍāmaṇi* commentary (Kathmandu, Ashvin 2071 BS, archive.org `ryae-nirnaya-sindhu-chudamani-veni-madhav-dhakal-kamala`), and with Daulat Ram Gaud's Hindi (Varanasi, 2000) | quoted with page numbers from the Dhakal edition; read by the consumer's research, the quotations below checked against the archive's text where it could be searched, the page images not checked here |
| Kane | P. V. Kane, *History of Dharmaśāstra*, vol. IV (1953) and vol. V pt. 1 (1958), rank 2 | first edition | pages as cited |
| press | the Nepal Panchanga Nirnayak Samiti's announced Sora Shraddha days, as its chair and secretary gave them to the Nepali press: Nepal Live, 1 Sep 2020; Online Khabar, 2 Sep 2020; Setopati, 22 Sep 2021; Setopati, 12 Sep 2022; Prashasan, 11 Sep 2022 | rank 2 (a committee's published decision) | as dated |

The Garuda Purana's preta chapters were **not read**; the first-year
schedule below is DS's, which collects the smriti and purana verses.

## 2. What the sources say

### 2.1 The origin: the tithi at the moment

NS (*kṣayāha-śrāddha*, Dhakal p. 982) quotes Vyasa through Hemadri,
"मासपक्षतिथिस्पृष्टे यो यस्मिन् म्रियतेऽहनि । प्रत्यब्दं तु तथाभूतं क्षयाहं तस्य तं
विदुः", and the Naradiya, "पारणे मरणे नृणां तिथिस्तात्कालिकी स्मृता": for a
parana and for a death the tithi is the one **at that time**, and the month
is lunar ("आब्दिके पितृकार्ये च चान्द्रो मासः प्रशस्यते", Garga). A death before
sunrise takes the tithi running then, not the civil day's sunrise tithi.
NS applies the kshaya-month rule to the birthday "as to the annual
shraddha" (*ābdikavad vardhāpane 'pi*), so a birth is fixed the same way.

When the date is not known (Kane IV p. 530, from the *Bhaviṣya* through
the digests): the tithi known and the month not, Margashirsha or Magha;
the month known and the tithi not, that month's new moon; neither, the
day the person left home or the day the news came. Nepal's committee
secretary (Online Khabar, 2 Sep 2020): an unknown tithi's shraddha on the
new moon.

### 2.2 The annual shraddha is an ekoddishta, at midday

DS p. 265: the day in fifteen muhurtas, "the seventh *gāndharva*, the
eighth *kutapa*, the ninth *rauhiṇa*". DS pp. 265–266, read off the page
image: "अथ प्रतिसांवत्सरिकमासिकादिनिर्णय उच्यते । तत्रैकोद्दिष्टं मध्याह्ने
सप्तमाष्टमनवममुहूर्तरूपे कार्यम् । तत्रापि कुतपरौहिणसंज्ञकावष्टमनवममुहूर्तौ मुख्यः
कालः । तत्र पूर्वत्रैव परत्रैव वा दिने मध्याह्नव्याप्तौ सैव तिथिर्ग्राह्या । दिनद्वये
मध्याह्नव्याप्तौ मध्याह्नास्पर्शे वा पूर्वैव । दिनद्वये साम्येनैकदेशव्याप्तौ पूर्वा ।
खर्वदर्पादिव्यवस्थेत्यन्ये । वैषम्येणैकदेशव्याप्तावाधिक्येन निर्णयः" — the
ekoddishta at midday, the seventh to ninth muhurtas, kutapa and rauhina
its main time. The tithi pervading the midday of one day only: that day.
On both, or on neither: the earlier. Partly on both, equally: the earlier
("others" decide by the *kharva-darpa* scheme). Unequally: by the greater.

NS (*kṣayāha-dvaidha-nirṇaya*, Dhakal p. 987) gives the same choices and
names the test window "कुतपादिरौहिणान्तो मुख्यः कालः", kutapa to rauhina,
muhurtas 8 and 9; it records the Gaudas' "later day when neither" and
refuses it ("परविद्धानिषेधप्राबल्यात्"). Madhava's madhyahna, which NS
quotes, is "the third of the day divided into five", and that is
muhurtas 7 to 9 exactly: DS's window and `DayPart::Madhyahna` are one
interval. Kane IV p. 515: "ekoddista in the noon". The addendum to the
consumer's brief (2026-10-20) asks for kutapa to rauhina as the default.

**The parvana shraddha is at aparahna**, by the same choices. DS p. 266:
"पार्वणे त्वपराह्णव्यापिनी ग्राह्या । पूर्वत्रैव परत्रैव वापराह्णव्याप्तौ सैव ग्राह्या ।
दिनद्वये तद्व्याप्तौ तदस्पर्शे वांशतः समव्याप्तौ वा पूर्वा । विषमव्याप्तौ त्वधिका
ग्राह्या". Madhava's variant (DS p. 266, NS p. 988): with the aparahna
wholly or equally covered on both days, the earlier if the *following*
tithi is kshaya, the later if it is vriddhi. DS p. 266 adds that the rule
holds "for the yearly, the monthly and the once-done mahalaya", and that
a yearly shraddha missed by day may be done in the night's first watch.

**Which is which.** NS (Dhakal p. 983): both forms are allowed, settled by
the custom of the country ("देशाचाराद् व्यवस्थित"). Parvana is required for
a death on the new moon or in the pitri paksha (Shankha, in NS: "अमावास्यां
क्षयो यस्य प्रेतपक्षेऽथवा पुनः । पार्वणं तस्य कर्तव्यं नैकोद्दिष्टं कदाचन"), and
the commentators add a death in battle and an ascetic. Dhakal, for Nepal
(p. 983 n.): "प्रत्येक वर्ष मृततिथिमा गरिने श्राद्ध एकोद्दिष्ट नै गर्नुपर्छ";
parvana only in Ashvin's dark fortnight, on the new moon and the like.

### 2.3 Adhika and kshaya months

**An ordinary origin in a year with an adhika month.** DS (malamasa,
the OCR of p. 266's continuation): the first anniversary falls in the
thirteenth month, the adhika, when the twelfth masika fell in the
ordinary month; when the twelfth masika falls in the adhika, it is done
twice and the first anniversary in the fourteenth, ordinary, month; the
second anniversary and later ones in the ordinary month only. NS
(Dhakal pp. 25–26) says the same, with Vrddha-Vasishtha's and Marichi's
"in both months" as minority views. Kane IV p. 546 and its corrigendum
agree. Dhakal disagrees for Nepal (p. 25 n. 14): Nepal's barkhi is the
*second* abdika, the first being done on the eleventh day, so the barkhi
falls in the ordinary month, with at most a gift without pinda on the
adhika day; some in Nepal do it in the malmas, "not understanding the
shastra".

**An origin inside an adhika month.** DS (malamasa, pp. 4–5 of the
edition the consumer read) and NS (Dhakal p. 26, Paithinasi): it recurs in
the ordinary month of that name, and **in the adhika month when that
month is adhika again**. Which dark fortnight: Dhakal (p. 982 n. 63),
worked in Nepal's purnimanta names, keeps the bright half's tithi in that
month's bright half and moves the dark half's to the dark half of the
month after the adhika, which is the same amanta month's dark half. Kane
IV pp. 546–547 (Bhrgu) reads the same in amanta terms.

**Kshaya month.** NS (malamasa, Hemadri): "तिथ्यर्धे प्रथमे पूर्वो द्वितीयेऽर्धे
तथोत्तरः । मासाविति बुधैश्चिन्त्यौ क्षयमासस्य मध्यगौ", and DS (Parichheda 1,
the kshaya example): every tithi of the joined lunation is of both
months, the first **half of each tithi** the first month's, the second
half the second's; a death or a birth in it takes the month of the half
it fell in. An ordinary-year anniversary of either month is kept in the
joined lunation on its tithi, with no half to choose (DS's *phalita*). An
adhika month just before a kshaya (*saṃsarpa*) also carries that month's
anniversaries, in DS's own words "as it seems" (*iti bhāti*).

### 2.4 Janma tithi

NS (*janmotsava*, Dhakal p. 531), quoting the *Kṛtyatattvārṇava*: "अत्र
जन्मतिथिरौदयिकी ग्राह्या … रवेरुदयमीक्षन्ते न तत्र तिथियुग्मता" — the birthday
takes the **sunrise tithi**, and the yugma pairing does not apply. DS
(*vardhāpana-vidhi*, p. 163 of the consumer's edition): monthly in the first
year, then yearly; when the tithi spans two days, the day joined with
the **birth nakshatra**; otherwise the sunrise tithi if it lasts **more
than two muhurtas** after sunrise, else the previous day; in an adhika
year, the ordinary month ("न त्वधिके"). Kane V.1 p. 89, for a
sunrise-tithi rite: on two sunrises the earlier; on none, the
*pūrvaviddha* day, the day the tithi runs in.

### 2.5 Sora Shraddha and its special days

The fortnight is Bhadrapada's dark half in amanta naming, Ashvina's in
purnimanta, with the full moon's shraddha before it. Each person's
mahalaya falls on their death tithi, as a parvana (aparahna). The
committee's announced days for 2077–2079 BS are the aparahna rule's,
including its two signatures, a day with no shraddha and a day with
two (§5.1). The special days, from DS pp. 61–64 (OCR):

| day | who | rule | DS |
|---|---|---|---|
| Bharani shraddha | all, once | on Bharani in the fortnight, *sāṅkalpa* without pinda | p. 61 |
| Avidhava Navami | women who died before their husbands | on the 9th | p. 62 |
| Sanyasi Dwadashi | ascetics | on the 12th, at aparahna; DS records some doing it on the Ekadashi parana day and prefers the 12th | p. 62 |
| Magha Trayodashi | all | the 13th joined with Magha, or Magha alone | p. 63 |
| Ghata (Shastrahata) Chaturdashi | the killed by weapon or violence | on the 14th, an ekoddishta; not for death by fire or water; the person's own tithi takes the mahalaya | pp. 63–64 |
| Sarvapitri Amavasya | an unknown tithi, and all | the new moon | p. 64; Kane IV p. 530; the committee |

### 2.6 The first year

DS pp. 329–331 (OCR): **sixteen shraddhas**, "द्वादश प्रतिमास्यानि ऊनमासं
त्रिपक्षकम् । ऊनषाण्मासिकं चोनाब्दिकं चापीति षोडश" — twelve monthly
(*māsika*), the *ūna-māsika* (short of the first month), the
*traipakṣika* (at three fortnights), the *ūna-ṣāṇmāsika* and the
*ūnābdika* (short of the sixth month and of the year). Their times: the
first monthly "on the eleventh day", the others at each month's death
tithi; the *ūna* ones one, two or three days short, or by a third, or (by
Madhava) on the day before the death tithi; the traipakshika on the death
tithi after three fortnights. One view puts the first monthly *and* the
first abdika on the eleventh day, another only the first monthly (the
same split Dhakal records for Nepal). An adhika month in the first year
doubles its monthly: seventeen. **Sapindana** (*sapiṇḍīkaraṇa*, p. 331):
at the year's end, or in the eleventh, sixth or third month, at three
fortnights, at the first month's end, on the twelfth or the eleventh day,
"or on the day a *vṛddhi* (a family rite) comes"; drawn forward, the
sixteen are done on that day and again in their own times. The
*udakumbha* and a lamp are given daily for the year (p. 329). The days of
impurity and the tenth day's rites belong to *aśauca* (the consumer's
P2-3) and are not designed here.

## 3. How it maps onto the festival rules

| what | a `FestivalRule` today | what it needs |
|---|---|---|
| barkhi (ekoddishta) | `month`, `occurs: {tithi}`, `decide`, `otherwise: EARLIER` | **`Window::DayMuhurtas { first: 8, last: 9 }`**, a range (the addendum's point: `NightMuhurta` names one); **`Choice::Greater`**, the day whose window holds more of the tithi, the earlier on a tie to the second (C170) |
| barkhi at DS's madhyahna | `at: {window: PART, part: MADHYAHNA}` | nothing: muhurtas 7–9 are the third fifth |
| parvana, Sora Shraddha | `at: PART APARAHNA` | `Choice::Greater` |
| Madhava's parvana variant | — | a predicate on the following tithi's kshaya or vriddhi (C664); not proposed for the default |
| janma tithi | `at: SUNRISE`, `decide: [LATER_ONLY → LATER]`, `otherwise: EARLIER` (the shape `FestivalRule::udaya` writes) | DS's options: `Predicate::Stands { nakshatra, at: Sunrise }` already exists for the nakshatra tie; the two-muhurta floor needs `Lasts` from sunrise in muhurtas (it counts ghatis today: four ghatis are two muhurtas of a thirty-ghati day, so `Lasts { from: Sunrise, ghatis: 4 }` is the text's floor read on C176's ghati) |
| the origin | — | **new**: from an instant and a place, the tithi, paksha, amanta month, its kind, and in a kshaya month the half of the tithi; with only a date, the sunrise tithi and a flag saying so |
| adhika policies | `in_adhika: bool` | **new**: an anniversary needs the year counted from the origin (first or later), so it is not a property of one rule. An `AnniversaryRule` builds the year's `FestivalRule` (month, `in_adhika`, and in a kshaya year the joined lunation answering to either name) and says which clause chose the month |
| kshaya month | the month at sunrise names the lunation by its first name | the joined lunation must answer to **both** names for an ordinary-year anniversary (DS *phalita*) |
| first-year rites | — | **new**: a schedule from the origin, a list of dated events each with its clause, by tradition (C673) |

So the day choice itself needs only two additions to the rule language
(`DayMuhurtas`, `Greater`), both general. The rest is a thin layer that
turns an origin and a year into the year's rule, which a consumer could
write by hand today for an ordinary year and cannot write for an adhika
or kshaya one.

### 3.1 The request and the answer

As built (`teistro_panchanga::festival::anniversary`). A request names an
anniversary as an item of `FestivalRequest.rules` with an `origin`, or by
`FestivalRequest::with_anniversary`; every binding reads the answer.

```rust
pub struct Anniversary {
    pub key: String,                     // the consumer's own, "GRANDFATHER"
    pub origin: Origin,                  // Instant { at } | Date { date } | Lunar { month, tithi, adhika, half, at }
    pub kind: AnniversaryKind,           // Shraddha (by the origin, C665) | Ekoddishta | Parvana | JanmaTithi
    pub window: Option<Window>,          // overrides the kind's (C662)
    pub first_year_adhika: FirstYearAdhika, // Adhika (the texts) | Nija (Dhakal), C666
    pub janma: JanmaOptions,             // C670: nakshatra_tie, two_muhurta_floor
}

pub struct AnniversaryDay {
    pub key: String,
    pub count: Option<u16>,   // 1 for the first recurrence after the origin; a janma tithi's age turned
    pub kind: AnniversaryKind,           // as kept: never Shraddha
    pub origin: OriginFound,  // tithi, month, kind, half, nakshatra, at, assumed_sunrise, candidates
    pub month_by: MonthClause,           // Nija | FirstYearAdhika | AdhikaOrigin | KshayaJoined
    pub observance: Observance,          // the festival evaluator's answer, case and extents
}
```

The origin is found at the place asked (`Almanac::origin`): an instant
gives its tithi, the amanta month and its kind, and in a kshaya lunation
the half of the tithi (its karana) names the month (C668); a date gives
its sunrise tithi, flagged, with every tithi the civil day touched as
`candidates`; a lunar origin is taken as given, with `count` absent unless
it carries `at`. `count` is round((tithi start − origin) / sidereal year):
the ordinal `an-ordinal-must-say-what-it-counts` asks for. The day choice
is `by_share` (the earlier on both, neither or a tie; the greater share
otherwise, C663, C664), which the `NEPAL_PANCHANGA` share fits use too.
The range is widened back a lunation so an anniversary near its start is
judged, and a refusal names `festivals.anniversaries[i].origin`. The
iCalendar feed writes each as a whole-day event.

Held to: the committee's announced Sora Shraddha days 2020–2022 (§5.1
rows 1–12, `crates/sdk/tests/anniversary.rs`), the origin cases of §5.2,
an adhika origin's recurrence, both first-year policies, the two-muhurta
floor, and the kshaya year's joined lunation (C669) on synthetic days,
proved red (`crates/panchanga/tests/anniversary.rs`).

### 3.2 The `PITRI_PAKSHA` pack

Sixteen rules, Bhadrapada (amanta) full moon and dark 1st to new moon,
each at aparahna with `Greater`, keyed `MAHALAYA_PURNIMA`,
`MAHALAYA_PRATIPADA` … `MAHALAYA_AMAVASYA`; plus `AVIDHAVA_NAVAMI` (an alias
of the 9th's day), `SANYASI_DWADASHI` (the 12th at aparahna),
`MAGHA_TRAYODASHI` (the 13th, with a `Joined { nakshatra: Magha }` note),
`GHATA_CHATURDASHI` (the 14th at aparahna) and `BHARANI_SHRADDHA` (a
nakshatra rule, `Occurs::Nakshatra { Bharani, Krishna }`). Each cites
its DS page. A day with no rule and a day with two are reported as such,
which is how the committee prints them.

## 4. Forks

| # | fork | proposed default | the other readings |
|---|---|---|---|
| C661 | the origin's tithi | **the tithi at the instant** of death or birth (NS, Naradiya); with only a date, the civil day's sunrise tithi, flagged `assumed_sunrise`, and both candidates returned when the day holds a tithi change | the sunrise tithi always, which the consumer's measurement puts 32% wrong at a random hour |
| C662 | the ekoddishta's window | **maintainer**: proposed kutapa to rauhina, muhurtas 8–9 (NS; the addendum) | DS's madhyahna, muhurtas 7–9 (= the third fifth); the consumer's measurement: the two differ on 1.7% of occurrences |
| C663 | neither day covered, and a tie | **the earlier** for both (DS p. 266; NS) | the Gaudas' later day when neither (NS records and refuses it); *kharva-darpa* (DS's "others") |
| C664 | parvana when both days are wholly or equally covered | **the earlier** (DS p. 266, NS's main rule) | Madhava's: earlier if the following tithi is kshaya, later if vriddhi (DS p. 266); Karshnajini's: the tithi at sunset (NS) |
| C665 | which anniversaries are parvana | **ekoddishta** for a barkhi, **parvana** for a death on the new moon or in the pitri paksha, an ascetic's, and Sora Shraddha itself (Shankha in NS; Dhakal for Nepal), as the request's `kind`, defaulted from the origin and overridable | parvana for all yearly shraddhas "by custom" (NS p. 983) |
| C666 | an ordinary origin, the first anniversary in a year whose month is adhika | **maintainer**: the texts (DS, NS, Kane) put it in the adhika month; Dhakal for Nepal's barkhi puts it in the ordinary month. Proposed: `ADHIKA` (the texts), with `NIJA` (Dhakal) offered and named, and a Nepal preset choosing `NIJA` | both months (Vrddha-Vasishtha, Marichi in NS) |
| C667 | an origin in an adhika month | **the ordinary month, and the adhika when that month is adhika again**; the dark half as the same amanta lunation's dark half (Dhakal n. 63; Kane–Bhrgu) | Hemadri's *sauram mānam* read literally as a solar month, which can point a month away; recorded, not offered |
| C668 | an origin in a kshaya month | **the half of the tithi at the instant** names the month (Hemadri in NS; DS); with only a date, the lunation's first name and a flag | — |
| C669 | an ordinary-year anniversary of either name, in a kshaya year | **in the joined lunation on its tithi**, no half split (DS's *phalita*); the *saṃsarpa* adhika before it also carries that month's anniversaries as a second day, flagged as DS's own inference | the joined lunation only |
| C670 | janma tithi | **the sunrise tithi** (NS); on two sunrises the earlier, on none the day it runs (Kane V.1 p. 89) | DS's two options, each a switch: the birth-nakshatra tie, and the two-muhurta floor (previous day when the sunrise tithi lasts less); the consumer measured 5.2% of occurrences moved by the floor |
| C671 | janma tithi in an adhika year | **the ordinary month** (DS "न त्वधिके"; NS "as the annual shraddha") | — |
| C672 | the pitri paksha's month | **Bhadrapada's dark half, amanta** (Ashvina's, purnimanta), the full moon before it included | — |
| C673 | the first-year schedule | **maintainer**: proposed DS pp. 329–331 as a `FIRST_YEAR_DS` tradition: the first monthly on the 11th day, the twelve monthly on the death tithi, the four *ūna* rites by Madhava (the day before the death tithi; traipakshika on it), sapindana on the 12th day; with `NEPAL` taking Dhakal's eleventh-day first abdika and the barkhi as the second abdika | sapindana at the year's end or any of DS p. 331's seven other times, as a parameter |
| C674 | an adhika month in the first year | **that month's monthly twice**, seventeen in all (DS p. 331) | — |
| C675 | an unknown date | **Kane IV p. 530's rules**: month unknown, Margashirsha or Magha; tithi unknown, that month's new moon; neither, Sarvapitri Amavasya (the committee) | — |
| C676 | a yearly shraddha missed by day | **reported, not chosen**: the night's first watch is allowed (DS p. 266; NS) | — |

## 5. Hand-checkable cases

### 5.1 Published or printed answers (22)

| # | case | the answer | source |
|---|---|---|---|
| 1 | Sora Shraddha, 17 Bhadau 2077 (2 Sep 2020), Kathmandu | Pratipada; aparahna 13:17–15:49 | committee, Nepal Live, 1 Sep 2020 |
| 2 | Sora Shraddha, 19 Bhadau 2077 (4 Sep 2020) | Tritiya | committee, as 1 |
| 3 | Sora Shraddha, 20 Bhadau 2077 (5 Sep 2020) | **no shraddha** | committee, as 1 |
| 4 | Sora Shraddha, 5 Asoj 2078 (21 Sep 2021) | Pratipada | committee chair (former), Setopati, 22 Sep 2021 |
| 5 | Sora Shraddha, 6 Asoj 2078 | Dwitiya | as 4 |
| 6 | Sora Shraddha, 7 Asoj 2078 (23 Sep 2021) | Tritiya (the 2nd in vriddhi at sunrise) | as 4 |
| 7 | Sora Shraddha, 8 Asoj 2078 | Chaturthi | as 4 |
| 8 | Sora Shraddha, 9 Asoj 2078 | Panchami | as 4 |
| 9 | Sora Shraddha, 26 Bhadau 2079 (11 Sep 2022) | **Pratipada and Dwitiya**; Pratipada's pinda before 14:19 | committee, Prashasan, 11 Sep 2022; Setopati, 12 Sep 2022 |
| 10 | Sora Shraddha, 31 Bhadau 2079 (16 Sep 2022) | Saptami | as 9 |
| 11 | Sora Shraddha, 1 Asoj 2079 (17 Sep 2022) | **no shraddha** | as 9 |
| 12 | Sora Shraddha, 2 Asoj 2079 (18 Sep 2022) | Ashtami ("बढी परेकाले", by the greater share) | as 9 |
| 13 | a death tithi unknown | the new moon | committee secretary, Online Khabar, 2 Sep 2020; Kane IV p. 530 |
| 14 | month known, tithi unknown | that month's new moon | Kane IV p. 530 |
| 15 | tithi known, month unknown | Margashirsha or Magha | Kane IV p. 530 |
| 16 | death in adhika Vaishakha Shukla 5 | recurs Vaishakha Shukla 5 | Dhakal, NS p. 982 n. 63 (worked example) |
| 17 | death in adhika Vaishakha Krishna 5 (purnimanta) | recurs Jyeshtha Krishna 5 (purnimanta), a month later | as 16 |
| 18 | death in the first half of a tithi of a Margashirsha–Pausha kshaya lunation | anniversary in Margashirsha | DS (Parichheda 1, kshaya example); NS (Hemadri) |
| 19 | the same, second half | anniversary in Pausha | as 18 |
| 20 | an ordinary origin whose twelfth monthly falls in an ordinary month, the next month adhika | first anniversary in the adhika (13th) month; second and later in the ordinary month | DS; NS (Dhakal pp. 25–26); Kane IV p. 546 corrigendum |
| 21 | the same in Nepal's barkhi reading | the barkhi in the ordinary month, a gift on the adhika day | Dhakal p. 25 n. 14 |
| 22 | an adhika-month origin, a later year with the same month adhika | the adhika month, not the ordinary | DS malamasa; NS (Paithinasi) |

Rows 1–12 are a fixture for `PITRI_PAKSHA` at Kathmandu under the
`nepali-committee` setup, which the consumer's research reproduced to the
day with the aparahna rule (and to the minute for row 9's 14:19). Rows
13–22 are unit cases of the origin and month policies, each with its
expected month and tithi; their civil days follow from the evaluator.

### 5.2 Computed cases still needing a published or priest's answer

From the consumer's measurement at Kathmandu (Surya Siddhanta, BS
2080–2084), to be confirmed against a printed panchanga's end times and a
priest's answer before they become fixtures:

- an origin at night before sunrise: 25 Sep 2022 02:00 is Krishna 14 (the
  civil day's sunrise tithi is the new moon);
- an origin in the evening: 12 Sep 2022 20:00 is Krishna 3 (sunrise
  tithi Krishna 2);
- an ekoddishta whose midday day is the day before its sunrise day:
  Krishna 14, 5 against 6 Baisakh 2080;
- an adhika dark-fortnight origin: a death on 22 Jestha 2083 (5 Jun
  2026, adhika Jyeshtha Krishna 5) recurs on 9 Asar 2084 (24 Jun 2027,
  purnimanta Ashadha Krishna 5) by C667;
- a janma tithi whose sunrise tithi lasts less than two muhurtas: Krishna
  5 on 8 Jul 2023, ending 19 minutes after sunrise;
- the abroad case: one of rows 1–12's tithis evaluated at New Delhi and
  at London, to show the day moving with the place's aparahna.

No committee statement on a single family's barkhi day was found; the
madhyahna rule rests on the texts and Dhakal's commentary, not on a
dated committee example.

## 6. What is measured, and the order of work

Steps 1–3 are built (2026-10-22).

1. `Window::DayMuhurtas` and `Choice::Greater` in the festival rules,
   with synthetic-day tests for each of DS p. 266's cases, proved red by
   breaking the tie.
2. The origin: tithi, month, kind and half at an instant, with the
   sunrise fallback, tested on §5.2's two night and evening instants.
3. `AnniversaryRule` and `sdk.almanac().anniversaries`, through every
   binding; §5.1 rows 13–22 as unit tests.
4. The `PITRI_PAKSHA` pack, held to §5.1 rows 1–12 under the committee
   setup, and counted over a decade in the festival pass.
5. The first-year schedule (C673), after the maintainer picks its
   default.
