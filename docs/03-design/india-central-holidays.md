# India's central holidays

Status: `built`, 2026-10-30. SR-03, the central list. State sets and the
regional packs are not here. Cruxes C851 to C856
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §9.

## 1. The source

The Department of Personnel and Training issues an office memorandum each
year, "Holidays to be observed in Central Government Offices". It is rank 1 for
the list. Seven are read here, 2021 to 2027 (the numbers are on the measured
page), each from a copy of the memorandum: a scan through page images, or the
text layer where the copy has one. The Department's own site does not serve the
files to a script, so the copies come from offices and staff sites that
republish them. Annexure I was read from the page for 2021, 2022, 2024, 2025 and
2026 and from the text for 2023 and 2027. The 2024 copy is damaged and renders
blank in some readers, so its page was rendered through PDFKit and read by eye. Secondary web tables were not trusted:
the copy of 2022's list on a staff site prints Maha Shivratri on 29 March, where
the memorandum says 1 March.

What the memorandum says, unchanged across the seven:

- **Para 2:** fourteen holidays are compulsory for every central office: Republic
  Day, Independence Day, Mahatma Gandhi's birthday, Buddha Purnima, Christmas,
  Dussehra, Diwali, Good Friday, Guru Nanak's birthday, Idu'l Fitr, Idu'l Zuha,
  Mahavir Jayanti, Muharram, and the Prophet's birthday.
- **Para 3.1:** three more are chosen from twelve occasions. The Department
  chooses Delhi's; each state capital's committee chooses its own.
- **Para 5.1:** in Delhi, a change of date of Idu'l Fitr, Idu'l Zuha, Muharram or
  Id-e-Milad "depending upon sighting of the Moon" is declared by the Ministry.
- **Para 6:** where a state keeps Diwali on Naraka Chaturdasi alone, central
  offices there may too.

Over the seven years Delhi's three came from four of the twelve: Holi every year,
Janmashtami every year, and Ram Navami or Maha Shivratri.

## 2. The pack

`INDIA_CENTRAL` holds the fourteen and those four, at any place, and is measured
at Delhi on India's clock. It reuses the keys of festivals the SDK already ships.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic, Independence, Gandhi's birthday, Christmas | `INDIA_REPUBLIC_DAY`, `INDIA_INDEPENDENCE_DAY`, `GANDHI_JAYANTI`, `CHRISTMAS` | civil date | the memorandum |
| Good Friday | `GOOD_FRIDAY` | two days before Easter, Gregorian computus | the memorandum |
| Dussehra, Diwali, Ram Navami, Maha Shivratri | `VIJAYA_DASHAMI`, `LAKSHMI_PUJA`, `RAMA_NAVAMI`, `SHIVARATRI` | the lunar rule | *Dharmasindhu*, as `DHARMASINDHU` ships them |
| Buddha Purnima, Mahavir Jayanti, Guru Nanak's birthday | `BUDDHA_JAYANTI`, `MAHAVIR_JAYANTI`, `GURU_NANAK_JAYANTI` | the lunar rule | a printed panchanga, as `NEPAL_PANCHANGA` ships them |
| Holi | `HOLI` | a day after `HOLIKA` | *Dharmasindhu* p. 95 |
| Janmashtami | `JANMASHTAMI` | the sunrise rule | fitted: C851 |

A key of para 3.1 is `optional` in its `meta`, so a reader knows a year's list may
not hold it. Ram Navami falls in the rules every year and in Delhi's list in five of
seven. The pack does not model which three a year chose: the choice is the
Department's, it is made for each year, and §9's table shows it.

`HOLIKA` is in the pack as Holi's anchor. It is on no list, so it is not
`GAZETTED`.

## 3. The days set by the moon

The four are not reckoned. A crescent-visibility estimate is a different feature, and
para 5.1 says the date is the Delhi authorities' to declare. So they ship as what the
memoranda print: `PublishedSet::india_central`, `ADD` entries under `EID_AL_FITR`,
`EID_AL_ADHA`, `MUHARRAM` and `MAWLID`, each with `subjectTo: MOON_SIGHTING`, which an
answer carries on the day's `published`. A year with no memorandum holds none of
them; a pack listing says each key's `lastYear`.

A day the Department later moved stands on the day it declared, unmarked, citing the
order: Muharram 2021 from 19 to 20 August (O.M. of 11 August 2021) and Id-ul-Zuha 2026
from 27 to 28 May (O.M. of 22 May 2026). Those two are the changes this search found,
not a proven full list: a printed day of a past year keeps its mark because no order
was found, which is the honest reading of it (C852).

A consumer lays a later order or a state's date over the pack with its own published
set: an `ADD` of the same key within a week replaces the pack's day, and can drop the
mark.

Five rows of §9 part from the notices, each named with its cause: Holi in 2023, 2026
and 2027 (C854), Dussehra in 2022 (C855) and Diwali in 2024 (C856). No rule was moved
to meet them. C199's clause looked like Holi's answer and measuring it ruled that out.

## 4. What is not here

- **State sets and para 3.1's other eight occasions.** Each is one gated pack per
  state, from that state's own notice (SR-03's next step).
- **Diwali on Naraka Chaturdasi** (para 6): a state's choice, for its pack.
- **Annexure II**, the restricted list: it applies to no one's day off by itself.
- **Which three a year chose** (§2).
