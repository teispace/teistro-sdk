# Nepal's gazetted holidays

Status: `built`, 2026-11-15; every year from VS 2076, 2026-11-23 (rc.9). SR-123.
The set is `HolidaySet::NepalGazette` (`crates/panchanga/src/festival/holiday.rs`),
each year's notice a data file, `crates/panchanga/data/np-holidays/<year>.json`,
read with `NEPAL_PANCHANGA`'s pack listing (`almanac.festivalPack`, `holidays`)
and named in `coverage().holidays` (`NEPAL_GAZETTE`: VS 2076 to 2083). The scope
words are the `holiday_scope` catalogue kind (167).

## 1. The sources

One notice a year, the Ministry of Home Affairs' yearly notice of public,
festival and substitute holidays in the *Nepal Gazette*, part 5, each a scan
read in full (VS 2076 to 2082 transcribed in the project's notes,
`np-holidays/README.md`, with each scan's SHA-256):

| VS | Nepal Gazette | published | holidays |
|---|---|---|---|
| 2076 | vol. 68 no. 58 | Chaitra 1, 2075 | 44 |
| 2077 | vol. 69 no. 51 | Chaitra 17, 2076 | 40 |
| 2078 | vol. 70 no. 47 | Chaitra 9, 2077 | 41 |
| 2079 | vol. 71 no. 48 | Falgun 26, 2078 | 44 |
| 2080 | vol. 72 no. 65 | Chaitra 2, 2079 | 43 |
| 2081 | vol. 73 no. 54 | Magh 29, 2080 | 45 |
| 2082 | vol. 74 no. 59 | Falgun 15, 2081 | 44 |
| 2083 | vol. 75 no. 67 | Falgun 18, 2082 | 45 |

A year's count is its clauses, a span of days (Dashain; Tihar where the
notice prints its span) one holiday, with the later notices applied. The
later notices found, each cited by its own holiday's `source`:

| VS | notice | change |
|---|---|---|
| 2076 | MoHA, the Council of Ministers' decision of Jestha 10, 2076 | adds Eid al-Fitr, Jestha 22, nationwide |
| 2076 | MoHA, the Bhoto Jatra notice | dates clause 5 (ग), Bhoto Jatra: Jestha 26 |
| 2076 | MoHA, the Council of Ministers' decision of Asoj 23, 2076 | adds Asoj 26, the Chinese President's state visit |
| 2076 | MoHA, the Council of Ministers' decision of Kartik 5, 2076 | adds Kija Puja, Kartik 13, nationwide |
| 2078 | MoHA, Falgun 18, 2078 | adds Gyalpo Lhosar, Falgun 19, nationwide (clause 2.2 (झ) gave it the Sherpa community) |
| 2079 | MoHA, Asoj 31, 2079 | adds Kartik 1, mourning Satya Mohan Joshi |
| 2080 | MoHA, Chaitra 28, 2080 | dates clause 2.1 (त), Eid al-Fitr: Chaitra 29 |
| 2080 | Nepal Gazette vol. 73 no. 54, part 5, notice 2 | cancels clause 6.1 (घ), Janayuddha Day (Falgun 1) |
| 2081 | Nepal Gazette vol. 74 no. 62, part 5 (Chaitra 11, 2081) | adds Ram Navami, Chaitra 24, nationwide |

A cancelled clause leaves the year's list and is answered apart, with the
cancelling notice in its source (`HolidaySet::cancelled`). The search for
later notices was not exhaustive: the Eid dates of VS 2077 to 2079, 2081 and
2082 were not found, so those clauses keep "on the day of".

VS 2083's notice, as read for the first build: the Ministry of Home Affairs' notice of VS 2083's public, festival and
substitute holidays, *Nepal Gazette* vol. 75 no. 67, part 5, Falgun 18, 2082
(`२०८३ सालको सार्वजनिक विदा`, SHA-256 `ffbabf83cad254b4…`, twelve pages, a scan
with no text layer, read by eye). It is rank 1 for the list. Its clauses:

| clause | holiday for | rows |
|---|---|---|
| 1 | the year's Saturdays, by month | none: no festival |
| 2.1 | festivals kept across the country | 18, Fagu Purnima's two regions apart |
| 2.2 | the religion, culture, area or place concerned only | 4 |
| 3 | women employees only | 2 |
| 4 | educational institutions only | 1 |
| 5 | the Kathmandu valley's jatras, the valley only | 4 |
| 6.1 | days kept across the country | 6 |
| 6.2 | employees with disabilities only | 1 |
| 7.1 | birth anniversaries kept across the country | 2 |
| 7.2 | the followers of the religion concerned only | 3 |
| 8 | days kept nationally with offices open | 3 |
| 9 | a mission abroad sets its own, up to 18 days | none |
| 10 | the services kept running on some of the days | none |

## 2. The shape

A holiday is the clause's words kept apart from the festivals it names:

| field | meaning |
|---|---|
| `set` | `NEPAL_GAZETTE` |
| `year` | the Bikram Sambat year whose notice lists it |
| `notice` | `{ne, en}`: the clause's words as printed, and in English |
| `festivals` | the festival keys the clause names, in its order |
| `scope` | `holiday_scope`: `NATIONAL`, `CONCERNED`, `COMMUNITY`, `REGION`, `RELIGION`, `WOMEN`, `SCHOOLS`, `DISABILITY`, `OFFICES_OPEN` |
| `regions` | the part of the country a `REGION` clause names, as `festival-metadata.md` §2a keys it (`KATHMANDU_VALLEY`, `HILLS`, `TERAI`) |
| `communities` | the community a `COMMUNITY` or `RELIGION` clause names (`NEWAR`, `DURA`, `KIRAT`, `MUSLIM`, `SIKH`) |
| `first`, `last` | the days the clause prints, Bikram Sambat; absent where it gives the day as the festival's own |
| `source` | the notice and the clause; for a holiday a later notice added or dated, that notice too |

**The data.** Each year is one file: `year`, `notice` (the citation),
`holidays` (each clause: `clause`, `scope`, `festivals`, `regions`,
`communities`, `first`, `last` as `[month, day]`, `daysOf`, `ne`, `en`) and
`amendments`, each one of `ADDS` (a holiday), `DATES` (a clause's day) or
`CANCELS` (a clause), with its notice. One reader makes every year's list,
and a unit test holds every file to its shape: each day a day of its year,
a span running forward, each key written as one, an amendment naming one
clause. Every festival key a holiday names is held to the catalogue by
`check-intl` and `crates/sdk/tests/nepal_gazette.rs`. A clause's `ne` is the
notice's words; where the clause limits a holiday to districts, the
district list follows them after " - "; where it gives a span with its
days named, the span's words then each day's, in its order. The English is
a plain translation.

Each scope is a clause's own words, not a summary of them. 2.2's heading
limits its four to "the religion, culture, geographic area and place
concerned"; Gai Jatra's and Dura Mhaipru Nakuma's clauses then name a
community, Sirua Pavani's five districts, and Gaura Parva's nothing, so
Gaura Parva is `CONCERNED` and the others `COMMUNITY` and `REGION`. Sirua
Pavani's districts have no key and stay in its notice.

**The festivals named.** A clause names a festival by the words a reader
would, and the key is the festival so named: "नव वर्ष" is
`MESHA_SANKRANTI`, the Bikram Sambat year's first day; "घटस्थापना"
`NAVARATRA_ARAMBHA`; "भाइटिका" `YAMA_DWITIYA`; "दशैं" `VIJAYA_DASHAMI`, whose
print reads "दशैंको टीका"; Fagu Purnima's hill and Terai days are
`HOLI_HILLS` and `HOLI_TERAI`, the keys Nepal's packs give the two days the
notice divides. The five holidays naming more than one festival:

| clause | day | festivals |
|---|---|---|
| 2.1 (ख) | Baisakh 18 | `CHANDI_PURNIMA`, `BUDDHA_JAYANTI`, `UBHAULI` |
| 2.1 (च) | Asoj 31 to Kartik 6 | `VIJAYA_DASHAMI`, `PHULPATI` |
| 2.1 (छ) | Kartik 22 to 26 | `LAKSHMI_PUJA`, `YAMA_DWITIYA` |
| 2.1 (झ) | Poush 9 (24 December 2026) | `DHANYA_PURNIMA`, `UDHAULI`, `YOMARI_PUNHI`, `JYAPU_DIWAS` |
| 2.1 (ठ) | Magh 1 | `MAGHI_THARU`, `MAKARA_SANKRANTI` |

The earlier notices name more festivals in a span's days: Mha Puja, Nepal
Sambat's new year, Govardhan Puja, Kija Puja and Gai Tihar among Tihar's
(VS 2076 to 2081), each keyed, with its printed day in `daysOf`. "राष्ट्रिय
एकता दिवस" (VS 2076 to 2079, offices open) is `PRITHVI_JAYANTI`, the day
VS 2080's notice and later print as "पृथ्वी जयन्ती (राष्ट्रिय एकता दिवस)".
Four clauses name no festival the SDK keys and are listed with none, their
words naming them: Dhawathambu Chheuchik (VS 2076, 2077), Janayuddha Day
(VS 2080, cancelled), the Chinese President's state visit (VS 2076) and
the mourning of Satya Mohan Joshi (VS 2079).

Six keys VS 2083's notice names have no rule in Nepal's packs: `EID_AL_FITR`,
`EID_AL_ADHA` and `MAWLID` (India's packs carry them as printed),
`BHOTO_JATRA`, `SIRUWA_PAVANI` and `DURA_MHAIPRU_NAKUMA`. Each is named in
every locale, so a reader names the holiday; its day is the notice's, or
for the first five, which the notice gives as "on the day of", none.

## 3. The gates

- **Names.** Every festival a holiday names is named in every strict locale,
  and no two festival keys share a name in one locale (`check-intl`): a
  name shared once hid two days, Bala Chaturdashi's two keys (now *Bala
  Charhe* for the Newar one, as `festival-classification.md` names it) and
  the two Democracy Days (now *Loktantra Diwas* for Baisakh 11's).
- **Days.** Over the committee's sky at Kathmandu, every printed day of VS 2083 holds
  the day `NEPAL_PANCHANGA` reckons for each festival of its holiday that
  the pack carries, and a holiday of several days holds each festival's
  day between its first and last (`crates/sdk/tests/nepal_gazette.rs`).
- **The consumer's list.** Tithipatra's `np-holidays-2083.json`, the list
  this set retires, was compared row by row and is not a source: the same
  45 rows, the same days and scopes, with this set's festivals keyed where
  its rows carried the notice's text as a name.

## 4. The tier it gives

A festival's significance (`festival-metadata.md` §7) in Nepal's packs is
first the notice's: a key a clause gives the whole country a holiday for is
`NATIONAL`, one a clause gives only some a holiday for `REGIONAL` (or
`MAJOR` with a text's fast), each marked with its clause as the source
(`nepal_gazette_scope`, `FestivalMeta::with_gazette`). Clause 8's days,
kept with offices open, give none. The Dashain and Tihar holidays are
granted as those festivals' days, so a day of the festival the pack keys
apart (in its gazetted grouping) counts when its VS 2083 day lies inside
the span: Mahashtami, Mahanavami, Kukur Tihar, the cows' worship (Kartik
23), and Govardhan and the oxen's worship (Kartik 24) do (C2301); Kag
Tihar, the day before Tihar's span, does not; Mha puja is a Newar festival
in the pack's grouping and is not counted (`HolidaySet::festival_days`,
each day held to the pack's reckoning by the days test). A key the notice
gives the whole country no holiday is never `NATIONAL`: Kag Tihar and Rama
Navami, gazetted in their group, fall below it. The tiers are the latest
notice's, VS 2083's; a later notice, when added, is read the same way and
replaces them, and the earlier years' give none. `nepal_gazette_scope(key,
year)` reads the year asked, or with none the latest: VS 2077's notice gave
Tamu Lhosar only to those it concerns, VS 2078's gave Gyalpo Lhosar to the
whole country by its later notice.

## 5. What moved

Nothing reckoned. VS 2076 to 2082 (2026-11-23): the listing's `holidays`
gains those years' holidays, each holiday gains `year`, and
`coverage().holidays` lists VS 2076 to 2083; no tier moves, VS 2083's being
the latest notice.

The first build: nothing reckoned. `NEPAL_PANCHANGA`'s and `NEPAL`'s pack listings gain
`holidays`, and their rules the notice's tiers; the festival names of
`BALA_CHATURDASHI` and `LOKTANTRA_DIWAS` change in the locales named above.
Clause 1's Saturdays are not carried: they name no festival, and a
weekday is the calendar's.
