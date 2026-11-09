# Maharashtra's public holidays

Status: `built`, 2026-11-09. SR-03, the first state set, after the central list
(`india-central-holidays.md`), from the state-holidays research of 2026-11-09 (its
§3). Cruxes C854 to C856 and C1240 to C1244
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §10.

## 1. The source

The General Administration Department (Desk-29) issues a notification each year,
"PUBLIC HOLIDAYS", under s. 25 of the Negotiable Instruments Act, 1881, with powers
the Ministry of Home Affairs delegated on 8 May 1968. It is published in the
*Maharashtra Government Gazette*, Extraordinary Part I, in English and in Marathi,
and each row prints its weekday and its Indian National (Saka) date. It is rank 1 for
the list. It has (A) the public holidays, (B) a day for banks alone, the yearly
closing of accounts on 1 April, and in some years (C) the holidays that fall on a
Saturday or a Sunday. Bhaubeej is added by a second notification, for the State
Government's offices, undertakings and local bodies; it is not a s. 25 day.

The state's servers refused scripted fetches, so the copies are ones other offices
and sites republished:

| year | notification | copy read | rank |
|---|---|---|---|
| 2020 | PHD. 1119/C.R.69/Desk-29, 7 December 2019 | a staff site's copy giving its number and table | 1 |
| 2021 | PHD. 1120/C.R.82/Desk-29, 1 December 2020 | a press list; the number from a staff site whose table is garbled | 3 |
| 2022 | PHD. 1121/C.R.97/Desk-29, 2 December 2021; Bhaubeej by PHD. 1120/C.R.97/Desk-29 of the same day | a copy giving its number and table | 1 |
| 2023 | PHD-1122/C.R.117/Desk-29, 2 December 2022 | the gazette page's text layer | 1 |
| 2024 | सार्वसु-1123/प्र.क्र.148/जपुक (कार्या-29), 9 November 2023, Gazette Extraordinary No. 118 | the Marathi gazette, enclosed in a circular of the Municipal Corporation of Greater Mumbai, read off the page | 1 |
| 2025 | of 4 December 2024, its number not seen | a press list | 3 |
| 2026 | PHD-1125/C.R.199/Japuk (29), December 2025 | a press list | 3 |

The only copy of 2019's list found is a bank-style list whose Diwali rows disagree
with the National Stock Exchange's trading holidays, which follow the state's list,
so 2019 is not used. Each pack entry for a printed day cites its year's notification
with its rank, and the three rank-3 years wait on their gazette (C1244).

## 2. The pack

`INDIA_MAHARASHTRA` holds the list at any place, and is measured at Mumbai on India's
clock. No rule is new. A day the central list also holds is `INDIA_CENTRAL`'s rule,
taken from that pack, not copied, so the two cannot drift apart; in the state's pack
it is on every year's list, and kept in `["IN", "MAHARASHTRA"]`.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanti, Christmas, Good Friday | as `INDIA_CENTRAL` | civil date, or Easter less two days | the notification |
| Chhatrapati Shivaji Maharaj Jayanti, Dr Babasaheb Ambedkar Jayanti, Maharashtra Day | `CHHATRAPATI_SHIVAJI_JAYANTI`, `AMBEDKAR_JAYANTI`, `MAHARASHTRA_DAY` | 19 February, 14 April, 1 May | the notification |
| the yearly closing of bank accounts | `BANK_ACCOUNTS_CLOSING` | 1 April; `communities: ["BANKS"]` | the notification, part (B) |
| Mahashivratri, Ram Navami, Dasara, Diwali Amavasya (Laxmi Pujan) | `SHIVARATRI`, `RAMA_NAVAMI`, `VIJAYA_DASHAMI`, `LAKSHMI_PUJA` | `INDIA_CENTRAL`'s lunar rule | *Dharmasindhu* |
| Buddha Pournima, Mahavir Jayanti, Guru Nanak Jayanti | `BUDDHA_JAYANTI`, `MAHAVIR_JAYANTI`, `GURU_NANAK_JAYANTI` | `INDIA_CENTRAL`'s lunar rule | a printed panchanga, as `NEPAL_PANCHANGA` ships them |
| Holi (second day) | `HOLI` | `INDIA_CENTRAL`'s day after `HOLIKA` | *Dharmasindhu* p. 95 |
| Diwali (Bali Pratipada), Bhaubeej | `BALI_PRATIPADA`, `YAMA_DWITIYA` | `DHARMASINDHU`'s rule | *Dharmasindhu* |
| Gudhi Padwa, Ganesh Chaturthi | `SAMVATSARA_ARAMBHA`, `GANESHA_CHATURTHI` | `NEPAL_PANCHANGA`'s rule | Chaitra shukla 1 at sunrise; Bhadrapada shukla 4 by its share of the madhyahna |

Bhaubeej is `optional`, since its own notification adds it in some years only (2022,
2023, 2025 and 2026 in hand). Janmashtami is on the central list and not the state's,
so the state's pack does not hold it. `HOLIKA` is in the pack as Holi's anchor and on
no list, so it is not `GAZETTED`.

Gudhi Padwa is the amanta Shaka year's first day, the day the Telugu New Year keeps.
In 2022 Bali Pratipada and Bhaubeej share 26 October, the 1st holding no sunrise of
its own.

## 3. The days printed, not reckoned

`PublishedSet::india_maharashtra` carries, for each year a notification is in hand:

- **Ramzan-Id, Bakri-Id, Moharum and Id-e-Milad**, as `EID_AL_FITR`, `EID_AL_ADHA`,
  `MUHARRAM` and `MAWLID`, named as the central list names them and each marked
  `subjectTo: MOON_SIGHTING`. The state's days are its own: in 2023 it printed Bakri-Id
  on 28 June, where the Department printed 29 June. A later notification of 2025 moved
  Id-e-Milad from 5 to 8 September for the Mumbai Suburban District alone; the state's
  day stands, and a consumer for that district lays the order over the pack with its
  own set (C1244).
- **The Parsi New Year (Shahenshahi)**, `PARSI_NEW_YEAR`, tradition `ZOROASTRIAN` (a
  catalogue member added for it) and community `PARSI`: 16 August in 2020 to 2023, 15
  August in 2024 to 2026. It is a 365-day year with no intercalation, so it is
  reckonable, but the catalogue has no Shahenshahi calendar (C1243). It is printed,
  not moved by the moon, so it is not marked.

A year with no notification holds none of them.

## 4. Against the print

At Mumbai over the built-in ephemeris, 75 of the 80 reckoned rows meet the print. Of
the rank-1 years' 46 rows (2020 and 2022 to 2024), 43 do. The five that part are named
with their causes in `festival-measured.md` §10, and no rule was moved to meet them:

- **Diwali Amavasya, 2024 and 2025** (C1240): the state takes the later day both times,
  where *Dharmasindhu* p. 77, as shipped, takes it only when the new moon lasts a ghati
  past the later sunset.
- **Bali Pratipada, 2020** (C1241): the state keeps the day whose sunrise holds the 1st.
- **Dasara, 2022** (C855): the state prints 5 October, as the Department does.
- **Mahavir Jayanti, 2023** (C1242): the 13th begins about six minutes before Mumbai's
  sunrise, and the state and the Department print the next day.

The state's list sides with the shipped rules against the central list on Holi in 2023
and 2026 (C854) and on Diwali in 2024 (C856): on each the state prints the rule's day
and the Department another.

## 5. What is not here

- **The other states.** West Bengal and Tamil Nadu are next (the research's §5); each
  needs vocabulary this pack did not.
- **The Saka dates** the notification prints beside each row. They are Indian National
  calendar dates and could check a transcription against `Calendar::IndianNational`.
- **Part (C)'s weekend days** as a separate class: a day on a Saturday or a Sunday is
  in the pack as any other.
