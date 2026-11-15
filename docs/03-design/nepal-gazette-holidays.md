# Nepal's gazetted holidays

Status: `built`, 2026-11-15. SR-123. The set is `HolidaySet::NepalGazette`
(`crates/panchanga/src/festival/holiday.rs`), read with `NEPAL_PANCHANGA`'s
pack listing (`almanac.festivalPack`, `holidays`). The scope words are the
`holiday_scope` catalogue kind (167).

## 1. The source

The Ministry of Home Affairs' notice of VS 2083's public, festival and
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
| `notice` | `{ne, en}`: the clause's words as printed, and in English |
| `festivals` | the festival keys the clause names, in its order |
| `scope` | `holiday_scope`: `NATIONAL`, `CONCERNED`, `COMMUNITY`, `REGION`, `RELIGION`, `WOMEN`, `SCHOOLS`, `DISABILITY`, `OFFICES_OPEN` |
| `regions` | the part of the country a `REGION` clause names, as `festival-metadata.md` §2a keys it (`KATHMANDU_VALLEY`, `HILLS`, `TERAI`) |
| `communities` | the community a `COMMUNITY` or `RELIGION` clause names (`NEWAR`, `DURA`, `KIRAT`, `MUSLIM`, `SIKH`) |
| `first`, `last` | the days the clause prints, Bikram Sambat; absent where it gives the day as the festival's own |
| `source` | the notice and the clause |

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

Six keys the notice names have no rule in Nepal's packs: `EID_AL_FITR`,
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
- **Days.** Over the committee's sky at Kathmandu, every printed day holds
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
the span: Mahashtami, Mahanavami, Kukur Tihar and Govardhan puja do; Kag
Tihar, the day before Tihar's span, does not; Mha puja is a Newar festival
in the pack's grouping and is not counted (`HolidaySet::festival_days`,
each day held to the pack's reckoning by the days test). A key the notice
gives the whole country no holiday is never `NATIONAL`: Kag Tihar and Rama
Navami, gazetted in their group, fall below it. The tiers are VS 2083's notice's; a later notice, when added, is
read the same way and replaces them.

## 5. What moved

Nothing reckoned. `NEPAL_PANCHANGA`'s and `NEPAL`'s pack listings gain
`holidays`, and their rules the notice's tiers; the festival names of
`BALA_CHATURDASHI` and `LOKTANTRA_DIWAS` change in the locales named above.
Clause 1's Saturdays are not carried: they name no festival, and a
weekday is the calendar's.
