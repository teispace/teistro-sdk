# Saait, measured

Status: `generated` by `cargo xtask saait`. Do not edit:
`check-saait` regenerates this page and fails on any difference.

It measures `muhurta.md` §4.6: the rites Raman's *Muhurtha* gives
beyond marriage, and his marriage, against the days the Nepal Panchanga
Nirnayak Samiti printed for them on its VS 2083 muhurta sheet. Each
printed day is founded at Kathmandu (27.7172° N, 85.324° E, 1400 m) on
Nepal's clock, over the committee's own sky (`nepali-committee`, the
Surya Siddhanta) and the modern one (`nepali-default`), and searched
with the rite's shipped rules. A day is **day-clean** when the rite's
grades of the tithi, nakshatra, yoga, karana, vara and month leave some
of its daylight unrejected, and **clean** when some window of that
daylight is also struck by no bar and rejected by no lagna or quarter.
Where a day is not, the clauses named are those rejecting every
window, or, where no one clause does, each window's.

## 1. The sheet

The sheet prints 94 rows (15 bratabandha, 41 vivaha, 36 pasni, 2 griha pravesh), each a Bikram Sambat date and its
weekday, read off the page images. Each date is converted by the shipped
Bikram Sambat calendar and its weekday compared with the one printed;
a row whose two disagree contradicts itself and is read neither way.

| proposed rule | verdict | measured |
|---|---|---|
| every printed weekday is its date's | falsified | 6 of 94 disagree; the pass refuses a change to these: bratabandha 10-21 (printed SOMAVARA, the date's GURUVARA); bratabandha 10-23 (printed BUDHAVARA, the date's SHANIVARA); bratabandha 12-4 (printed SOMAVARA, the date's GURUVARA); vivaha 3-14 (printed SOMAVARA, the date's RAVIVARA); vivaha 3-15 (printed MANGALAVARA, the date's SOMAVARA); pasni 10-25 (printed BUDHAVARA, the date's SOMAVARA) |

## 2. Each rite against its printed days

The year is VS 2083's days at Kathmandu; a day the rite's season closes
is not open. A printed day the day rules leave open is necessarily one
of the year's open days, so the year's count says how much the rules
narrow the year before the committee chose.

| rite | rules | sky | printed and read | day-clean | clean | open days of the year |
|---|---|---|---:|---:|---:|---:|
| bratabandha | `RAMAN_UPANAYANA` | the committee's | 12 | 12 | 12 | 131 of 365 |
| bratabandha | `RAMAN_UPANAYANA` | modern | 12 | 12 | 12 | 129 of 365 |
| vivaha | `RAMAN_MARRIAGE` | the committee's | 39 | 10 | 5 | 32 of 365 |
| vivaha | `RAMAN_MARRIAGE` | modern | 39 | 10 | 4 | 33 of 365 |
| pasni | `RAMAN_ANNAPRASANA` | the committee's | 35 | 32 | 24 | 140 of 365 |
| pasni | `RAMAN_ANNAPRASANA` | modern | 35 | 32 | 25 | 138 of 365 |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | the committee's | 2 | 2 | 2 | 37 of 365 |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | modern | 2 | 2 | 2 | 38 of 365 |

| proposed rule | verdict | measured |
|---|---|---|
| over the committee's sky, every printed bratabandha day has a clean window | **holds** | 0 of 12 disagree |
| over the committee's sky, every printed vivaha day has a clean window | falsified | 34 of 39 disagree |
| over the committee's sky, every printed pasni day has a clean window | falsified | 11 of 35 disagree |
| over the committee's sky, every printed griha pravesh day has a clean window | **holds** | 0 of 2 disagree |

## 3. What parts them

Over the committee's sky, each clause that rejects every daylight
window of a printed day, counted over the days it does: a day-level
clause on a day not day-clean, an instant one on a day-clean day with
no clean window. A day whose windows are each rejected by a different
clause is counted under "no one clause".

| rite | clause | printed days |
|---|---|---:|
| vivaha | VARA MANGALAVARA | 10 |
| vivaha | no one clause | 9 |
| vivaha | MONTH PAUSHA | 5 |
| vivaha | MOON_JOINED | 2 |
| vivaha | TITHI KRISHNA_CHATURTHI | 2 |
| vivaha | TITHI KRISHNA_NAVAMI | 2 |
| vivaha | TITHI SHUKLA_NAVAMI | 2 |
| vivaha | YOGA VYAGHATA | 2 |
| vivaha | NAKSHATRA JYESHTHA | 1 |
| vivaha | NAKSHATRA PURVA_ASHADHA | 1 |
| vivaha | SEVENTH_OCCUPIED | 1 |
| vivaha | TITHI KRISHNA_ASHTAMI | 1 |
| vivaha | TITHI KRISHNA_DWADASHI | 1 |
| vivaha | TITHI SHUKLA_CHATURTHI | 1 |
| vivaha | YOGA ATIGANDA | 1 |
| vivaha | YOGA GANDA | 1 |
| vivaha | YOGA SHOOLA | 1 |
| vivaha | YOGA VAJRA | 1 |
| pasni | no one clause | 6 |
| pasni | UNWANTED_PLACEMENT 10 | 4 |
| pasni | YOGA SHOOLA | 1 |

Raman reports that "some sages" allow a marriage in Pausha while the
Sun is in Capricorn, and does not take it (`ActivityRules::raman_marriage`).
Read with it, over the committee's sky, 10 of the printed marriages'
days are day-clean, against 10 under his rules as shipped.

## 4. Every row

| rite | BS 2083 | day | printed | committee's sky | modern sky |
|---|---|---|---|---|---|
| bratabandha | 1-20 | 2026-05-03 | RAVIVARA | clean | clean |
| bratabandha | 1-21 | 2026-05-04 | SOMAVARA | clean | clean |
| bratabandha | 1-23 | 2026-05-06 | BUDHAVARA | clean | clean |
| bratabandha | 3-10 | 2026-06-24 | BUDHAVARA | clean | clean |
| bratabandha | 10-25 | 2027-02-08 | SOMAVARA | clean | clean |
| bratabandha | 10-21 | 2027-02-04 | SOMAVARA (the date's GURUVARA) | — | — |
| bratabandha | 10-23 | 2027-02-06 | BUDHAVARA (the date's SHANIVARA) | — | — |
| bratabandha | 11-10 | 2027-02-22 | SOMAVARA | clean | clean |
| bratabandha | 11-13 | 2027-02-25 | GURUVARA | clean | clean |
| bratabandha | 11-26 | 2027-03-10 | BUDHAVARA | clean | clean |
| bratabandha | 12-03 | 2027-03-17 | BUDHAVARA | clean | clean |
| bratabandha | 12-04 | 2027-03-18 | SOMAVARA (the date's GURUVARA) | — | — |
| bratabandha | 12-10 | 2027-03-24 | BUDHAVARA | clean | clean |
| bratabandha | 12-25 | 2027-04-08 | GURUVARA | clean | clean |
| bratabandha | 12-28 | 2027-04-11 | RAVIVARA | clean | clean |
| vivaha | 1-07 | 2026-04-20 | SOMAVARA | day clean; MOON_JOINED | day clean; MOON_JOINED |
| vivaha | 1-08 | 2026-04-21 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | 1-22 | 2026-05-05 | MANGALAVARA | TITHI KRISHNA_CHATURTHI, VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | 1-23 | 2026-05-06 | BUDHAVARA | clean | clean |
| vivaha | 1-24 | 2026-05-07 | GURUVARA | each window by one of: NAKSHATRA PURVA_ASHADHA, TITHI KRISHNA_SHASHTHI | NAKSHATRA PURVA_ASHADHA |
| vivaha | 1-25 | 2026-05-08 | SHUKRAVARA | each window by one of: KARANA VISHTI, NAKSHATRA SHRAVANA, TITHI KRISHNA_SHASHTHI | each window by one of: KARANA VISHTI, TITHI KRISHNA_SHASHTHI |
| vivaha | 1-30 | 2026-05-13 | BUDHAVARA | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_EKADASHI, YOGA VISHKAMBHA | YOGA VISHKAMBHA |
| vivaha | 1-31 | 2026-05-14 | GURUVARA | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_TRAYODASHI | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_TRAYODASHI |
| vivaha | 3-09 | 2026-06-23 | MANGALAVARA | TITHI SHUKLA_NAVAMI, VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | 3-13 | 2026-06-27 | SHANIVARA | day clean; each window by one of: KARTARI, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, SEVENTH_OCCUPIED | day clean; each window by one of: KARTARI, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, SEVENTH_OCCUPIED |
| vivaha | 3-14 | 2026-06-28 | SOMAVARA (the date's RAVIVARA) | — | — |
| vivaha | 3-15 | 2026-06-29 | MANGALAVARA (the date's SOMAVARA) | — | — |
| vivaha | 3-17 | 2026-07-01 | BUDHAVARA | day clean; each window by one of: KARTARI, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, SEVENTH_OCCUPIED | day clean; each window by one of: KARTARI, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, SEVENTH_OCCUPIED |
| vivaha | 3-18 | 2026-07-02 | GURUVARA | each window by one of: NAKSHATRA SHRAVANA, YOGA VAIDHRITI, YOGA VISHKAMBHA | each window by one of: NAKSHATRA SHRAVANA, YOGA VAIDHRITI, YOGA VISHKAMBHA |
| vivaha | 3-23 | 2026-07-07 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | 8-09 | 2026-11-25 | BUDHAVARA | clean | clean |
| vivaha | 8-10 | 2026-11-26 | GURUVARA | clean | clean |
| vivaha | 8-16 | 2026-12-02 | BUDHAVARA | TITHI KRISHNA_NAVAMI | TITHI KRISHNA_NAVAMI |
| vivaha | 8-17 | 2026-12-03 | GURUVARA | clean | clean |
| vivaha | 8-19 | 2026-12-05 | SHANIVARA | TITHI KRISHNA_DWADASHI | TITHI KRISHNA_DWADASHI |
| vivaha | 8-24 | 2026-12-10 | GURUVARA | each window by one of: YOGA GANDA, YOGA SHOOLA | each window by one of: YOGA GANDA, YOGA SHOOLA |
| vivaha | 8-25 | 2026-12-11 | SHUKRAVARA | NAKSHATRA PURVA_ASHADHA | NAKSHATRA PURVA_ASHADHA |
| vivaha | 8-26 | 2026-12-12 | SHANIVARA | clean | day clean; each window by one of: KARTARI, LAGNA CAPRICORN, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, MOON_JOINED, SEVENTH_OCCUPIED |
| vivaha | 10-04 | 2027-01-18 | SOMAVARA | MONTH PAUSHA | MONTH PAUSHA, NAKSHATRA KRITTIKA |
| vivaha | 10-12 | 2027-01-26 | MANGALAVARA | MONTH PAUSHA, VARA MANGALAVARA, YOGA ATIGANDA | MONTH PAUSHA, VARA MANGALAVARA, YOGA ATIGANDA |
| vivaha | 10-15 | 2027-01-29 | SHUKRAVARA | MONTH PAUSHA, YOGA SHOOLA | MONTH PAUSHA, TITHI KRISHNA_ASHTAMI |
| vivaha | 10-19 | 2027-02-02 | MANGALAVARA | MONTH PAUSHA, VARA MANGALAVARA, YOGA VYAGHATA | MONTH PAUSHA, VARA MANGALAVARA, YOGA VYAGHATA |
| vivaha | 10-20 | 2027-02-03 | BUDHAVARA | MONTH PAUSHA | MONTH PAUSHA |
| vivaha | 10-27 | 2027-02-10 | BUDHAVARA | TITHI SHUKLA_CHATURTHI | TITHI SHUKLA_CHATURTHI |
| vivaha | 11-03 | 2027-02-15 | SOMAVARA | TITHI SHUKLA_NAVAMI | TITHI SHUKLA_NAVAMI |
| vivaha | 11-04 | 2027-02-16 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | 11-11 | 2027-02-23 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA, YOGA SHOOLA |
| vivaha | 11-12 | 2027-02-24 | BUDHAVARA | TITHI KRISHNA_CHATURTHI, YOGA GANDA | TITHI KRISHNA_CHATURTHI, YOGA GANDA |
| vivaha | 11-15 | 2027-02-27 | SHANIVARA | YOGA VYAGHATA | YOGA VYAGHATA |
| vivaha | 11-16 | 2027-02-28 | RAVIVARA | TITHI KRISHNA_ASHTAMI | TITHI KRISHNA_ASHTAMI |
| vivaha | 11-17 | 2027-03-01 | SOMAVARA | NAKSHATRA JYESHTHA, TITHI KRISHNA_NAVAMI, YOGA VAJRA | TITHI KRISHNA_NAVAMI, YOGA VAJRA |
| vivaha | 11-18 | 2027-03-02 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | 11-20 | 2027-03-04 | GURUVARA | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_EKADASHI | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_EKADASHI |
| vivaha | 11-25 | 2027-03-09 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | 11-26 | 2027-03-10 | BUDHAVARA | day clean; MOON_JOINED | day clean; MOON_JOINED |
| vivaha | 11-30 | 2027-03-14 | RAVIVARA | day clean; SEVENTH_OCCUPIED | day clean; SEVENTH_OCCUPIED, VENUS_IN_SIXTH |
| pasni | 1-07 | 2026-04-20 | SOMAVARA | day clean; UNWANTED_PLACEMENT 10 | day clean; each window by one of: LAGNA ARIES, UNWANTED_PLACEMENT 10 |
| pasni | 1-10 | 2026-04-23 | GURUVARA | clean | day clean; each window by one of: LAGNA ARIES, UNWANTED_PLACEMENT 10 |
| pasni | 1-21 | 2026-05-04 | SOMAVARA | day clean; each window by one of: LAGNA ARIES, UNWANTED_PLACEMENT 10 | day clean; each window by one of: LAGNA ARIES, UNWANTED_PLACEMENT 10 |
| pasni | 2-04 | 2026-05-18 | SOMAVARA | day clean; UNWANTED_PLACEMENT 10 | clean |
| pasni | 2-07 | 2026-05-21 | GURUVARA | each window by one of: TITHI SHUKLA_SHASHTHI, YOGA GANDA | each window by one of: TITHI SHUKLA_SHASHTHI, YOGA GANDA |
| pasni | 2-22 | 2026-06-05 | SHUKRAVARA | day clean; UNWANTED_PLACEMENT 10 | day clean; UNWANTED_PLACEMENT 10 |
| pasni | 3-03 | 2026-06-17 | BUDHAVARA | clean | clean |
| pasni | 3-10 | 2026-06-24 | BUDHAVARA | clean | clean |
| pasni | 3-17 | 2026-07-01 | BUDHAVARA | clean | clean |
| pasni | 3-18 | 2026-07-02 | GURUVARA | clean | clean |
| pasni | 4-04 | 2026-07-20 | SOMAVARA | clean | clean |
| pasni | 4-08 | 2026-07-24 | SHUKRAVARA | clean | clean |
| pasni | 4-15 | 2026-07-31 | SHUKRAVARA | clean | clean |
| pasni | 4-18 | 2026-08-03 | SOMAVARA | clean | clean |
| pasni | 4-20 | 2026-08-05 | BUDHAVARA | each window by one of: KARANA VISHTI, TITHI KRISHNA_ASHTAMI, YOGA GANDA, YOGA SHOOLA | each window by one of: KARANA VISHTI, YOGA GANDA, YOGA SHOOLA |
| pasni | 5-01 | 2026-08-17 | SOMAVARA | clean | clean |
| pasni | 5-03 | 2026-08-19 | BUDHAVARA | clean | clean |
| pasni | 5-10 | 2026-08-26 | BUDHAVARA | clean | clean |
| pasni | 5-29 | 2026-09-14 | SOMAVARA | clean | clean |
| pasni | 6-05 | 2026-09-21 | SOMAVARA | clean | clean |
| pasni | 6-26 | 2026-10-12 | SOMAVARA | clean | clean |
| pasni | 7-13 | 2026-10-30 | SHUKRAVARA | day clean; UNWANTED_PLACEMENT 10 | day clean; UNWANTED_PLACEMENT 10 |
| pasni | 7-30 | 2026-11-16 | SOMAVARA | clean | clean |
| pasni | 8-10 | 2026-11-26 | GURUVARA | day clean; each window by one of: LAGNA ARIES, LAGNA PISCES, LAGNA SCORPIO, UNWANTED_PLACEMENT 10 | clean |
| pasni | 8-17 | 2026-12-03 | GURUVARA | day clean; each window by one of: LAGNA PISCES, LAGNA SCORPIO, UNWANTED_PLACEMENT 10 | day clean; UNWANTED_PLACEMENT 10 |
| pasni | 9-01 | 2026-12-16 | BUDHAVARA | clean | clean |
| pasni | 9-15 | 2026-12-30 | BUDHAVARA | clean | clean |
| pasni | 10-15 | 2027-01-29 | SHUKRAVARA | YOGA SHOOLA | TITHI KRISHNA_ASHTAMI |
| pasni | 10-25 | 2027-02-08 | BUDHAVARA (the date's SOMAVARA) | — | — |
| pasni | 11-07 | 2027-02-19 | SHUKRAVARA | clean | clean |
| pasni | 11-13 | 2027-02-25 | GURUVARA | clean | clean |
| pasni | 11-26 | 2027-03-10 | BUDHAVARA | clean | clean |
| pasni | 12-03 | 2027-03-17 | BUDHAVARA | clean | clean |
| pasni | 12-10 | 2027-03-24 | BUDHAVARA | clean | clean |
| pasni | 12-18 | 2027-04-01 | GURUVARA | day clean; each window by one of: LAGNA ARIES, LAGNA PISCES, UNWANTED_PLACEMENT 10 | day clean; each window by one of: LAGNA ARIES, LAGNA PISCES, UNWANTED_PLACEMENT 10 |
| pasni | 12-25 | 2027-04-08 | GURUVARA | clean | clean |
| griha pravesh | 1-31 | 2026-05-14 | GURUVARA | clean | clean |
| griha pravesh | 11-20 | 2027-03-04 | GURUVARA | clean | clean |
