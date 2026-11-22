# Nepal's festival names and days

Status: `built` 2026-11-17, phase 1 of SR-145. It extends
[`festival-metadata.md`](festival-metadata.md) (what a festival is) and
[`festival-groups.md`](festival-groups.md) (a festival over several days),
and reads Nepal's gazetted holidays as
[`nepal-gazette-holidays.md`](nepal-gazette-holidays.md) gives them.
Cruxes C2300 to C2311.

## 1. What is asked

The consumer, an app for Nepal, asks three things of `NEPAL_PANCHANGA`:

1. **Every day of a multi-day festival named** as Nepal names it, each day
   placed by a rule and a member of its festival's group: Tihar's five
   days, Dashain's fifteen, Chhath's four, the sixteen shraddhas with the
   Magha shraddha, Teej's three, and three days its table missed
   (आकाशदीपदान आरम्भ, यमदीपदान on the right day, छठ समापन).
2. **Nepal's primary name** for each festival, its other names, and a
   display rank, as data on the rule, not as code per festival:
   भाइटीका leads in Nepal and यमद्वितीया is its other name; लक्ष्मी पूजा
   leads Kartik 22 over कुकुर तिहार.
3. **Day marks**: a holiday for some only (women employees, a community,
   the valley), and the `*` of a tithi that changes during the day.

Accuracy comes before coverage. A day with no sure rule is left to a
published set, never guessed. Each rule is read from a text or from the
committee's book, and measured against the book's VS 2082 and 2083
editions (`festival-measured.md` §6). The book's own print is cited by
page and year, and not copied.

## 2. Every day

The committee's book prints the days below. Each is now a rule of
`NEPAL_PANCHANGA`, read three ways:

- **By a text.** `GAI_PUJA`, the cows' worship: *Dharmasindhu* p. 78
  keeps the Bali puja and the cows' play on the later day only when
  Kartika's bright 1st lasts nine muhurtas (18 ghatis) past its sunrise,
  and on the earlier otherwise (`Fit::LaterPastSunrise(18)`). The print
  meets it in VS 2082 (the later day), VS 2083 (the new moon's day the
  1st begins on, Kartik 23) and VS 2076 (the earlier day) (C2301). The
  `NEPAL` pack reads its `BALI_PRATIPADA` at sunrise (C197), which is
  Govardhan puja's day and not the cows'.
- **At a window, as its kin are.** `PRAUSHTHAPADI_SHRADDHA`, the full
  moon's shraddha, at aparahna as the fortnight's shraddhas are (C2302).
- **On another rule's day**, as the print sets it in that rule's row
  (C2304): `KALARATRI` (the Mahashtami), `AKASHDEEPADAN_ARAMBHA` (Kartik
  snan's start), `YAMADEEPADAN` (Dhanteras), `SUKHRATRI` (Lakshmi puja),
  `GORU_PUJA` (Mha puja, C2301), `KIJA_PUJA` (Bhai Tika) and
  `SORHA_SHRADDHA_SAMAPTI` (the new moon's shraddha).
- **Counted from Chhath's evening arghya** (C2306): `CHHATH_NAHAY_KHAY`
  two days before, `CHHATH_KHARNA` the day before and
  `CHHATH_SECOND_DAY`, the morning arghya, the day after. The committee
  prints only the 6th. Bihar's notification lists Kharna and the morning
  arghya (C2106). Nahay-Khay, the 4th, is kept the day before Kharna in
  every almanac read, at rank 3. The `COUNTED` table replaces `DAY_AFTER`
  and carries the count, negative before.

The groups, in the year's order (`FestivalGroup::nepal_panchanga`):

| group | main | members |
|---|---|---|
| `SORHA_SHRADDHA` | `AMAVASYA_SHRADDHA` | the full moon's, the start, the fourteen tithis', Magha's, the new moon's, the close |
| `DASHAIN` | `VIJAYA_DASHAMI` | Ghatasthapana, Bilva nimantran, Phulpati, the Mahashtami, Kalaratri, the Mahanavami, Vijaya Dashami, Kojagrat |
| `TIHAR` | `LAKSHMI_PUJA` | Kag Tihar, Lakshmi puja, Kukur Tihar, Narak chaturdashi, Sukhratri, Gai puja, Goru puja, Govardhan puja, Mha puja, Hali Tihar, Bhai Tika, Kija puja |
| `CHHATH` | `CHHATH` | Nahay-Khay, Kharna, the evening arghya, the morning arghya |
| `TEEJ` | `HARITALIKA` | Dar khane din, the Haritalika fast, Rishi panchami |

BS 2083's Tihar has no gap any more:

```
7:KAG_TIHAR 8:LAKSHMI_PUJA+KUKUR_TIHAR+NARAK_CHATURDASHI+SUKHRATRI 9:GAI_PUJA
10:GORU_PUJA+GOVARDHAN_PUJA+MHA_PUJA+HALI_TIHAR 11:YAMA_DWITIYA+KIJA_PUJA
```

**Dashain's fifteen days.** The book names the 1st, the 6th to the 10th
and the full moon. It prints no Dashain name for the 2nd to the 4th or the
11th to the 14th in either edition, so no rule names them, and the group
keeps them as gap days. A consumer that names them publishes them
(C2305).

The new keys take their values from their group's (`NEPAL_KEYS`): Gai and
Goru puja are gazetted with Tihar's holiday days (`days_of`, Kartik 23 and
24, so `NATIONAL`); Kija puja is the Newars'; the three Chhath days are the
Maithil and Bhojpuri communities' (a new group, kept by the communities
Chhath's gazetted day is granted to); the rest are Nepal's, unclassified
like their kin. Akashdeepadan's start and the shraddhas' close are
`PERIOD`, as every stretch's ends are.

## 3. Names as data

A festival's names belong to a region, not to a language. Nepal leads
with भाइटीका in Nepali and in English alike, and India with Bhai Dooj or
Yama dwitiya. So the names are a field of the rule's metadata, one entry
a region:

```rust
pub struct FestivalNames {
    pub region: String,          // "NP"
    pub lead: Option<String>,    // a name key; None: the rule's own key
    pub aliases: Vec<String>,    // name keys, the rule's own among them when another leads
    pub rank: Option<u16>,       // its place among a day's observances of its tier
    pub source: String,
}
// FestivalMeta.names: Vec<FestivalNames>
```

Each name is a **key of the locales' festival names** (`BHAI_TIKA`,
`GHATASTHAPANA`, `DEEPAMALIKA`), named in all five locales as a rule's
key is. A consumer shows `lead` in the reader's language, falls back to the
rule's key, and lists the aliases beside it. Twenty-one keys are new:
ten rules' and eleven names'. The Nepali names are the book's spelling;
the Hindi and Sanskrit forms follow the locales' practice for their kin.

Nepal's entries (`NAMES` in `meta.rs`):

| rule | lead | other names |
|---|---|---|
| `YAMA_DWITIYA` | `BHAI_TIKA` | `YAMA_DWITIYA` |
| `NAVARATRA_ARAMBHA` | `GHATASTHAPANA` | `NAVARATRA_ARAMBHA` |
| `LAKSHMI_PUJA` | | `DEEPAMALIKA` |
| `KAG_TIHAR` | | `KAKABALI` |
| `VIJAYA_DASHAMI` | | `DASHAIN_TIKA` |
| `AMAVASYA_SHRADDHA` | | `PITRI_VISARJAN` |
| `HARITALIKA` | | `TEEJ` |
| `CHHATH` | | `SANDHYA_ARGHYA`, `DALA_PUJA` |
| `CHHATH_SECOND_DAY` | `USHA_ARGHYA` | `CHHATH_SAMAPAN`, `CHHATH_SECOND_DAY` |
| `MAKARA_SANKRANTI` | `MAGHE_SANKRANTI` | `MAKARA_SANKRANTI`, `GHYU_CHAKU_KHANE_DIN` |
| `KARKATA_SANKRANTI` | `SAUNE_SANKRANTI` | `KARKATA_SANKRANTI`, `SHRAWAN_SANKRANTI` |
| the other ten sankrantis | | the month each opens (`BAISAKH_SANKRANTI`, `JESTHA_SANKRANTI`, …, `CHAITRA_SANKRANTI`) |
| `UPAKARMA_MADHYANDINA` | `JANAI_PURNIMA` | `UPAKARMA_MADHYANDINA`, `RISHI_TARPANI` |
| `BASANTA_PANCHAMI` | | `SARASWATI_PUJA`, `SHREE_PANCHAMI` |
| `NATIONAL_PADDY_DAY` | | `DAHI_CHIURA_KHANE_DIN` |

The sankrantis' names are measured over Nepal's patros (C2312): the locales
once named the Makara and Karkata sankrantis माघे and साउने in Nepali and
English, a region's name held in a language's file, which §3 rejects; they
name all twelve by the sign again, and Nepal's entries lead with the month.
The upakarma the committee prints as जनैपूर्णिमा leads with that name, and
Basanta Panchami keeps the committee's वसन्तपञ्चमी and lists the patros'
सरस्वती पूजा and श्रीपञ्चमी (C2314). Raksha Bandhan, printed on the same row,
keeps its own name, so no two rules of one day lead with one name. The
two eating days are other names of a day already kept: Magh 1's घ्यू-चाकु
खाने दिन is Maghe Sankranti, and Asar 15's दही-चिउरा खाने दिन is Paddy Day
(C2315).
A test holds every listed name to a festival name in every locale.

Not shipped (C2311): गाई तिहार, which the book does not print (it prints
गाईपूजा), and धनत्रयोदशी as Dhanteras's other name, which is its Sanskrit
name already.

A request's `meta` patch replaces the names like any field, and its
check refuses an entry with no region or source, a name that is no key,
or a name given twice, by its field (`…meta.names[0]`).

**Alternatives weighed.**

- *A rule per name* (`BHAI_TIKA` beside `YAMA_DWITIYA`): two observances
  of one day, which the day's list, the groups and the counts would each
  have to fold back into one. Rejected.
- *Renaming the locale's name*: a locale is a language. Nepal's English
  reader wants Bhai Tika and India's Hindi reader Bhai Dooj; the name a
  region leads with cannot live in a language's file. Rejected.
- *Region forms in the locale files* (`name@NP`): the fact would sit in
  five files with no check that they agree, and no rule would say it.
  Rejected.
- *Names on each observance of the answer*: every answer type in the
  hand-written Java and Dart bindings would change, for a value the rule
  already holds. The meta is where every "what a festival is" fact lives,
  and the pack listing and `FestivalRequest::meta_of` reach it. Chosen:
  the meta.

## 4. A day's order

`printed_order` folds into the names: an entry's `rank` is the place it
held, and `FestivalMeta::rank` reads the entry of the rule's first region.
A day's observances sort by tier, then rank, then the rule's place. The
ranks carry the print's order for the 24 keys it held, with three changes:

- **Kartik 22**: Lakshmi puja 1, Kukur Tihar 2, the maintainer's order
  over the book's, which sets Kukur Tihar first in every edition read
  (C2303).
- **Kartik 24**: Goru puja 1, Govardhan puja 2, as the book prints them.
- Mha puja and Nepal Sambat's new year move to 3 and 4, keeping their
  order after the moonrise.

Gai puja is `NATIONAL` (a day of the notice's Tihar holiday), so it now
leads Kartik 23, before Somabare aunsi.

## 5. The `*`: a tithi that changes during the day

`Observance.beginsAfterSunrise` is true when the occurrence begins after
the day's sunrise and before the next: the sunrise holds the tithi
before it, and the day's tithi changes during it. It is set where the
rule decides the day (`judge`), so it costs a comparison an observance.
A day counted to one of its own (`days` not 0) does not carry it, and a
day a published entry moves clears it, since neither is the tithi's day.
It is left out of the wire when false. The bindings carry it
(`beginsAfterSunrise`, `begins_after_sunrise`).

No page of the book says what the consumer's `*` means. This reading is
the plain one, and C2308 stays open until the consumer confirms it.

## 6. A holiday for some

The notice's scope (`HolidayScope`: `WOMEN`, `COMMUNITY`, `REGION`, …)
already reaches a consumer: the pack listing's `holidays` give each clause
with its scope, festivals, communities and regions, and a rule's tier
reads it (`nepal-gazette-holidays.md` §4). A second mark on the
observance would say the same thing in two places, so there is none
(C2310). A consumer marks a day by looking its key up in the listing's
holidays.

## 7. Measured

Every new rule's day was read off the book for VS 2082 and 2083 and is
held to it by `cargo xtask festival` (`festival-measured.md` §6), except
the three Chhath days the book does not print. VS 2076, outside the
measured pass, was read by hand too: it prints Yamadeepadan, Sukhratri,
Kija puja and the oxen's worship in the same rows as VS 2082 and 2083 do,
and the cows' worship on the day the text's rule gives from its printed
tithi ends. Hali Tihar's udaya rule misses VS 2076 by a
day while meeting both measured years (C2309).

## 8. Left to phases 2 and 3

- Dashain's 2nd to 4th and 11th to 14th, once a Nepal source names them
  (C2305).
- The `*`'s meaning, from the consumer (C2308).
- Names for regions other than Nepal, and for the India packs' rules.
- Hali Tihar's VS 2076 day (C2309).
- The parity runners print the new mark when a binding's parity case
  reads a day that carries it.
