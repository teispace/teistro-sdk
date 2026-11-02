# Saait, measured

Status: `generated` by `cargo xtask saait`. Do not edit:
`check-saait` regenerates this page and fails on any difference.

It measures `muhurta.md` §4.6: the rites Raman's *Muhurtha* gives,
his marriage, and *Muhurta Chintamani*'s house, shop and marriage
rites, against the days the Nepal Panchanga Nirnayak Samiti printed
for them on its VS 2083 muhurta sheet, and the Surya Panchanga's VS
2082 lists of the house's foundation, the house entry and the shop,
which its preface says the committee decided. Each printed day is
founded at Kathmandu (27.7172° N, 85.324° E, 1400 m) on Nepal's
clock, over the committee's own sky (`nepali-committee`, the Surya
Siddhanta) and the modern one (`nepali-drik`), and searched with each
of the rite's shipped rules. A day is **day-clean** when the rules'
grades of the tithi, nakshatra, yoga, karana, vara and month leave some
of its daylight unrejected, and **clean** when some window of that
daylight is also struck by no bar and rejected by no lagna, quarter or
bull of the house. Where a day is not, the clauses named are those
rejecting every window, or, where no one clause does, each window's.

## 1. The lists

The committee's sheet prints 148 rows (15 bratabandha, 41 vivaha, 36 pasni, 2 griha pravesh, 9 griharambha, 45 dokan), each a Bikram Sambat date
and its weekday; the Surya Panchanga's lists 41 (3 griha pravesh, 6 griharambha, 32 dokan), 13 of them with
a weekday. All are read off the page images. Each date is converted by
the shipped Bikram Sambat calendar and a printed weekday compared with
its date's; a row whose two disagree contradicts itself and is read
neither way.

| proposed rule | verdict | measured |
|---|---|---|
| every printed weekday is its date's | falsified | 6 of 161 disagree; the pass refuses a change to these: bratabandha 2083-10-21 (printed SOMAVARA, the date's GURUVARA); bratabandha 2083-10-23 (printed BUDHAVARA, the date's SHANIVARA); bratabandha 2083-12-4 (printed SOMAVARA, the date's GURUVARA); pasni 2083-10-25 (printed BUDHAVARA, the date's SOMAVARA); vivaha 2083-3-14 (printed SOMAVARA, the date's RAVIVARA); vivaha 2083-3-15 (printed MANGALAVARA, the date's SOMAVARA) |

## 2. Each rite against its printed days

A year is its Bikram Sambat year's days at Kathmandu; a day the rules'
season closes is not open. A printed day the day rules leave open is
necessarily one of the year's open days, so the year's count says how
much the rules narrow the year before the list chose.

| rite | rules | VS | sky | printed and read | day-clean | clean | open days of the year |
|---|---|---:|---|---:|---:|---:|---:|
| bratabandha | `RAMAN_UPANAYANA` | 2083 | the committee's | 12 | 12 | 12 | 131 of 365 |
| bratabandha | `RAMAN_UPANAYANA` | 2083 | modern | 12 | 12 | 12 | 129 of 365 |
| vivaha | `RAMAN_MARRIAGE` | 2083 | the committee's | 39 | 10 | 4 | 32 of 365 |
| vivaha | `RAMAN_MARRIAGE` | 2083 | modern | 39 | 10 | 4 | 33 of 365 |
| vivaha | `CHINTAMANI_VIVAHA` | 2083 | the committee's | 39 | 32 | 32 | 132 of 365 |
| vivaha | `CHINTAMANI_VIVAHA` | 2083 | modern | 39 | 34 | 34 | 132 of 365 |
| pasni | `RAMAN_ANNAPRASANA` | 2083 | the committee's | 35 | 32 | 27 | 140 of 365 |
| pasni | `RAMAN_ANNAPRASANA` | 2083 | modern | 35 | 32 | 25 | 138 of 365 |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | 2082 | the committee's | 3 | 1 | 1 | 19 of 365 |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | 2082 | modern | 3 | 1 | 1 | 21 of 365 |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | 2083 | the committee's | 2 | 2 | 2 | 37 of 365 |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | 2083 | modern | 2 | 2 | 2 | 38 of 365 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | 2082 | the committee's | 6 | 2 | 2 | 47 of 365 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | 2082 | modern | 6 | 2 | 2 | 46 of 365 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | 2083 | the committee's | 9 | 1 | 1 | 49 of 365 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | 2083 | modern | 9 | 1 | 1 | 48 of 365 |
| dokan | `CHINTAMANI_VIPANI` | 2082 | the committee's | 32 | 31 | 31 | 281 of 365 |
| dokan | `CHINTAMANI_VIPANI` | 2082 | modern | 32 | 31 | 31 | 282 of 365 |
| dokan | `CHINTAMANI_VIPANI` | 2083 | the committee's | 45 | 44 | 44 | 281 of 365 |
| dokan | `CHINTAMANI_VIPANI` | 2083 | modern | 45 | 44 | 44 | 277 of 365 |

| proposed rule | verdict | measured |
|---|---|---|
| over the committee's sky, every printed bratabandha day has a clean window under `RAMAN_UPANAYANA` | **holds** | 0 of 12 disagree |
| over the committee's sky, every printed vivaha day has a clean window under `RAMAN_MARRIAGE` | falsified | 35 of 39 disagree |
| over the committee's sky, every printed vivaha day has a clean window under `CHINTAMANI_VIVAHA` | falsified | 7 of 39 disagree |
| over the committee's sky, every printed pasni day has a clean window under `RAMAN_ANNAPRASANA` | falsified | 8 of 35 disagree |
| over the committee's sky, every printed griha pravesh day has a clean window under `RAMAN_GRIHA_PRAVESHA` | falsified | 2 of 5 disagree |
| over the committee's sky, every printed griharambha day has a clean window under `CHINTAMANI_GRIHARAMBHA` | falsified | 12 of 15 disagree |
| over the committee's sky, every printed dokan day has a clean window under `CHINTAMANI_VIPANI` | falsified | 2 of 77 disagree |

## 3. What parts them

Over the committee's sky, each clause that rejects every daylight
window of a printed day, counted over the days it does: a day-level
clause on a day not day-clean, an instant one on a day-clean day with
no clean window. A day whose windows are each rejected by a different
clause is counted under "no one clause".

| rite | rules | clause | printed days |
|---|---|---|---:|
| vivaha | `RAMAN_MARRIAGE` | VARA MANGALAVARA | 10 |
| vivaha | `RAMAN_MARRIAGE` | no one clause | 10 |
| vivaha | `RAMAN_MARRIAGE` | MONTH PAUSHA | 5 |
| vivaha | `RAMAN_MARRIAGE` | MOON_JOINED | 2 |
| vivaha | `RAMAN_MARRIAGE` | TITHI KRISHNA_CHATURTHI | 2 |
| vivaha | `RAMAN_MARRIAGE` | TITHI KRISHNA_NAVAMI | 2 |
| vivaha | `RAMAN_MARRIAGE` | TITHI SHUKLA_NAVAMI | 2 |
| vivaha | `RAMAN_MARRIAGE` | YOGA VYAGHATA | 2 |
| vivaha | `RAMAN_MARRIAGE` | NAKSHATRA JYESHTHA | 1 |
| vivaha | `RAMAN_MARRIAGE` | NAKSHATRA PURVA_ASHADHA | 1 |
| vivaha | `RAMAN_MARRIAGE` | SEVENTH_OCCUPIED | 1 |
| vivaha | `RAMAN_MARRIAGE` | TITHI KRISHNA_ASHTAMI | 1 |
| vivaha | `RAMAN_MARRIAGE` | TITHI KRISHNA_DWADASHI | 1 |
| vivaha | `RAMAN_MARRIAGE` | TITHI SHUKLA_CHATURTHI | 1 |
| vivaha | `RAMAN_MARRIAGE` | YOGA ATIGANDA | 1 |
| vivaha | `RAMAN_MARRIAGE` | YOGA GANDA | 1 |
| vivaha | `RAMAN_MARRIAGE` | YOGA SHOOLA | 1 |
| vivaha | `RAMAN_MARRIAGE` | YOGA VAJRA | 1 |
| vivaha | `CHINTAMANI_VIVAHA` | TITHI KRISHNA_CHATURTHI | 2 |
| vivaha | `CHINTAMANI_VIVAHA` | TITHI KRISHNA_NAVAMI | 2 |
| vivaha | `CHINTAMANI_VIVAHA` | TITHI SHUKLA_NAVAMI | 2 |
| vivaha | `CHINTAMANI_VIVAHA` | TITHI SHUKLA_CHATURTHI | 1 |
| pasni | `RAMAN_ANNAPRASANA` | no one clause | 5 |
| pasni | `RAMAN_ANNAPRASANA` | UNWANTED_PLACEMENT 10 | 2 |
| pasni | `RAMAN_ANNAPRASANA` | YOGA SHOOLA | 1 |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | VARA SHANIVARA | 1 |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | closed by SHUKRA_ASTA | 1 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | MONTH KARTIKA | 4 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | MONTH MAGHA | 2 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | MONTH MARGASHIRSHA | 2 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | closed by SHUKRA_ASTA | 2 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | no one clause | 2 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | NAKSHATRA REVATI | 1 |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | TITHI KRISHNA_PRATIPADA | 1 |
| dokan | `CHINTAMANI_VIPANI` | TITHI KRISHNA_NAVAMI | 1 |
| dokan | `CHINTAMANI_VIPANI` | VARA MANGALAVARA | 1 |

Raman reports that "some sages" allow a marriage in Pausha while the
Sun is in Capricorn, and does not take it (`ActivityRules::raman_marriage`).
Read with it, over the committee's sky, 10 of the printed marriages'
days are day-clean, against 10 under his rules as shipped.

## 4. Every row

| rite | rules | list | BS | day | printed | committee's sky | modern sky |
|---|---|---|---|---|---|---|---|
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-1-20 | 2026-05-03 | RAVIVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-1-21 | 2026-05-04 | SOMAVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-1-23 | 2026-05-06 | BUDHAVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-3-10 | 2026-06-24 | BUDHAVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-10-25 | 2027-02-08 | SOMAVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-10-21 | 2027-02-04 | SOMAVARA (the date's GURUVARA) | — | — |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-10-23 | 2027-02-06 | BUDHAVARA (the date's SHANIVARA) | — | — |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-11-10 | 2027-02-22 | SOMAVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-11-13 | 2027-02-25 | GURUVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-11-26 | 2027-03-10 | BUDHAVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-12-03 | 2027-03-17 | BUDHAVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-12-04 | 2027-03-18 | SOMAVARA (the date's GURUVARA) | — | — |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-12-10 | 2027-03-24 | BUDHAVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-12-25 | 2027-04-08 | GURUVARA | clean | clean |
| bratabandha | `RAMAN_UPANAYANA` | the committee's sheet | 2083-12-28 | 2027-04-11 | RAVIVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-1-07 | 2026-04-20 | SOMAVARA | day clean; MOON_JOINED | day clean; MOON_JOINED |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-1-07 | 2026-04-20 | SOMAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-1-08 | 2026-04-21 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-1-08 | 2026-04-21 | MANGALAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-1-22 | 2026-05-05 | MANGALAVARA | TITHI KRISHNA_CHATURTHI, VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-1-22 | 2026-05-05 | MANGALAVARA | TITHI KRISHNA_CHATURTHI | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-1-23 | 2026-05-06 | BUDHAVARA | clean | clean |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-1-23 | 2026-05-06 | BUDHAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-1-24 | 2026-05-07 | GURUVARA | each window by one of: NAKSHATRA PURVA_ASHADHA, TITHI KRISHNA_SHASHTHI | NAKSHATRA PURVA_ASHADHA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-1-24 | 2026-05-07 | GURUVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-1-25 | 2026-05-08 | SHUKRAVARA | each window by one of: KARANA VISHTI, NAKSHATRA SHRAVANA, TITHI KRISHNA_SHASHTHI | each window by one of: KARANA VISHTI, TITHI KRISHNA_SHASHTHI |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-1-25 | 2026-05-08 | SHUKRAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-1-30 | 2026-05-13 | BUDHAVARA | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_EKADASHI, YOGA VISHKAMBHA | YOGA VISHKAMBHA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-1-30 | 2026-05-13 | BUDHAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-1-31 | 2026-05-14 | GURUVARA | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_TRAYODASHI | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_TRAYODASHI |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-1-31 | 2026-05-14 | GURUVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-3-09 | 2026-06-23 | MANGALAVARA | TITHI SHUKLA_NAVAMI, VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-3-09 | 2026-06-23 | MANGALAVARA | TITHI SHUKLA_NAVAMI | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-3-13 | 2026-06-27 | SHANIVARA | day clean; each window by one of: KARTARI, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, SEVENTH_OCCUPIED | day clean; each window by one of: KARTARI, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, SEVENTH_OCCUPIED |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-3-13 | 2026-06-27 | SHANIVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-3-14 | 2026-06-28 | SOMAVARA (the date's RAVIVARA) | — | — |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-3-14 | 2026-06-28 | SOMAVARA (the date's RAVIVARA) | — | — |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-3-15 | 2026-06-29 | MANGALAVARA (the date's SOMAVARA) | — | — |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-3-15 | 2026-06-29 | MANGALAVARA (the date's SOMAVARA) | — | — |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-3-17 | 2026-07-01 | BUDHAVARA | day clean; each window by one of: KARTARI, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, SEVENTH_OCCUPIED | day clean; each window by one of: KARTARI, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, SEVENTH_OCCUPIED |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-3-17 | 2026-07-01 | BUDHAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-3-18 | 2026-07-02 | GURUVARA | each window by one of: NAKSHATRA SHRAVANA, YOGA VAIDHRITI, YOGA VISHKAMBHA | each window by one of: NAKSHATRA SHRAVANA, YOGA VAIDHRITI, YOGA VISHKAMBHA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-3-18 | 2026-07-02 | GURUVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-3-23 | 2026-07-07 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-3-23 | 2026-07-07 | MANGALAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-8-09 | 2026-11-25 | BUDHAVARA | clean | clean |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-8-09 | 2026-11-25 | BUDHAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-8-10 | 2026-11-26 | GURUVARA | clean | clean |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-8-10 | 2026-11-26 | GURUVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-8-16 | 2026-12-02 | BUDHAVARA | TITHI KRISHNA_NAVAMI | TITHI KRISHNA_NAVAMI |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-8-16 | 2026-12-02 | BUDHAVARA | TITHI KRISHNA_NAVAMI | TITHI KRISHNA_NAVAMI |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-8-17 | 2026-12-03 | GURUVARA | clean | clean |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-8-17 | 2026-12-03 | GURUVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-8-19 | 2026-12-05 | SHANIVARA | TITHI KRISHNA_DWADASHI | TITHI KRISHNA_DWADASHI |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-8-19 | 2026-12-05 | SHANIVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-8-24 | 2026-12-10 | GURUVARA | each window by one of: YOGA GANDA, YOGA SHOOLA | each window by one of: YOGA GANDA, YOGA SHOOLA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-8-24 | 2026-12-10 | GURUVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-8-25 | 2026-12-11 | SHUKRAVARA | NAKSHATRA PURVA_ASHADHA | NAKSHATRA PURVA_ASHADHA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-8-25 | 2026-12-11 | SHUKRAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-8-26 | 2026-12-12 | SHANIVARA | day clean; each window by one of: KARTARI, LAGNA CAPRICORN, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, MOON_JOINED, SEVENTH_OCCUPIED | day clean; each window by one of: KARTARI, LAGNA CAPRICORN, LAGNA SCORPIO, MALEFIC_IN_LAGNA, MARS_IN_EIGHTH, MOON_JOINED, SEVENTH_OCCUPIED |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-8-26 | 2026-12-12 | SHANIVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-10-04 | 2027-01-18 | SOMAVARA | MONTH PAUSHA | MONTH PAUSHA, NAKSHATRA KRITTIKA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-10-04 | 2027-01-18 | SOMAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-10-12 | 2027-01-26 | MANGALAVARA | MONTH PAUSHA, VARA MANGALAVARA, YOGA ATIGANDA | MONTH PAUSHA, VARA MANGALAVARA, YOGA ATIGANDA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-10-12 | 2027-01-26 | MANGALAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-10-15 | 2027-01-29 | SHUKRAVARA | MONTH PAUSHA, YOGA SHOOLA | MONTH PAUSHA, TITHI KRISHNA_ASHTAMI |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-10-15 | 2027-01-29 | SHUKRAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-10-19 | 2027-02-02 | MANGALAVARA | MONTH PAUSHA, VARA MANGALAVARA, YOGA VYAGHATA | MONTH PAUSHA, VARA MANGALAVARA, YOGA VYAGHATA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-10-19 | 2027-02-02 | MANGALAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-10-20 | 2027-02-03 | BUDHAVARA | MONTH PAUSHA | MONTH PAUSHA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-10-20 | 2027-02-03 | BUDHAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-10-27 | 2027-02-10 | BUDHAVARA | TITHI SHUKLA_CHATURTHI | TITHI SHUKLA_CHATURTHI |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-10-27 | 2027-02-10 | BUDHAVARA | TITHI SHUKLA_CHATURTHI | TITHI SHUKLA_CHATURTHI |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-03 | 2027-02-15 | SOMAVARA | TITHI SHUKLA_NAVAMI | TITHI SHUKLA_NAVAMI |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-03 | 2027-02-15 | SOMAVARA | TITHI SHUKLA_NAVAMI | TITHI SHUKLA_NAVAMI |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-04 | 2027-02-16 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-04 | 2027-02-16 | MANGALAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-11 | 2027-02-23 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA, YOGA SHOOLA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-11 | 2027-02-23 | MANGALAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-12 | 2027-02-24 | BUDHAVARA | TITHI KRISHNA_CHATURTHI, YOGA GANDA | TITHI KRISHNA_CHATURTHI, YOGA GANDA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-12 | 2027-02-24 | BUDHAVARA | TITHI KRISHNA_CHATURTHI | TITHI KRISHNA_CHATURTHI |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-15 | 2027-02-27 | SHANIVARA | YOGA VYAGHATA | YOGA VYAGHATA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-15 | 2027-02-27 | SHANIVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-16 | 2027-02-28 | RAVIVARA | TITHI KRISHNA_ASHTAMI | TITHI KRISHNA_ASHTAMI |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-16 | 2027-02-28 | RAVIVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-17 | 2027-03-01 | SOMAVARA | NAKSHATRA JYESHTHA, TITHI KRISHNA_NAVAMI, YOGA VAJRA | TITHI KRISHNA_NAVAMI, YOGA VAJRA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-17 | 2027-03-01 | SOMAVARA | TITHI KRISHNA_NAVAMI | TITHI KRISHNA_NAVAMI |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-18 | 2027-03-02 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-18 | 2027-03-02 | MANGALAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-20 | 2027-03-04 | GURUVARA | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_EKADASHI | each window by one of: TITHI KRISHNA_DWADASHI, TITHI KRISHNA_EKADASHI |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-20 | 2027-03-04 | GURUVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-25 | 2027-03-09 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-25 | 2027-03-09 | MANGALAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-26 | 2027-03-10 | BUDHAVARA | day clean; MOON_JOINED | day clean; MOON_JOINED |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-26 | 2027-03-10 | BUDHAVARA | clean | clean |
| vivaha | `RAMAN_MARRIAGE` | the committee's sheet | 2083-11-30 | 2027-03-14 | RAVIVARA | day clean; SEVENTH_OCCUPIED | day clean; SEVENTH_OCCUPIED |
| vivaha | `CHINTAMANI_VIVAHA` | the committee's sheet | 2083-11-30 | 2027-03-14 | RAVIVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-1-07 | 2026-04-20 | SOMAVARA | day clean; UNWANTED_PLACEMENT 10 | day clean; each window by one of: LAGNA ARIES, UNWANTED_PLACEMENT 10 |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-1-10 | 2026-04-23 | GURUVARA | clean | day clean; each window by one of: LAGNA ARIES, UNWANTED_PLACEMENT 10 |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-1-21 | 2026-05-04 | SOMAVARA | day clean; each window by one of: LAGNA ARIES, UNWANTED_PLACEMENT 10 | day clean; each window by one of: LAGNA ARIES, UNWANTED_PLACEMENT 10 |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-2-04 | 2026-05-18 | SOMAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-2-07 | 2026-05-21 | GURUVARA | each window by one of: TITHI SHUKLA_SHASHTHI, YOGA GANDA | each window by one of: TITHI SHUKLA_SHASHTHI, YOGA GANDA |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-2-22 | 2026-06-05 | SHUKRAVARA | day clean; UNWANTED_PLACEMENT 10 | day clean; UNWANTED_PLACEMENT 10 |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-3-03 | 2026-06-17 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-3-10 | 2026-06-24 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-3-17 | 2026-07-01 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-3-18 | 2026-07-02 | GURUVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-4-04 | 2026-07-20 | SOMAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-4-08 | 2026-07-24 | SHUKRAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-4-15 | 2026-07-31 | SHUKRAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-4-18 | 2026-08-03 | SOMAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-4-20 | 2026-08-05 | BUDHAVARA | each window by one of: KARANA VISHTI, TITHI KRISHNA_ASHTAMI, YOGA GANDA, YOGA SHOOLA | each window by one of: KARANA VISHTI, YOGA GANDA, YOGA SHOOLA |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-5-01 | 2026-08-17 | SOMAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-5-03 | 2026-08-19 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-5-10 | 2026-08-26 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-5-29 | 2026-09-14 | SOMAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-6-05 | 2026-09-21 | SOMAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-6-26 | 2026-10-12 | SOMAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-7-13 | 2026-10-30 | SHUKRAVARA | clean | day clean; UNWANTED_PLACEMENT 10 |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-7-30 | 2026-11-16 | SOMAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-8-10 | 2026-11-26 | GURUVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-8-17 | 2026-12-03 | GURUVARA | day clean; each window by one of: LAGNA PISCES, LAGNA SCORPIO, UNWANTED_PLACEMENT 10 | day clean; UNWANTED_PLACEMENT 10 |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-9-01 | 2026-12-16 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-9-15 | 2026-12-30 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-10-15 | 2027-01-29 | SHUKRAVARA | YOGA SHOOLA | TITHI KRISHNA_ASHTAMI |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-10-25 | 2027-02-08 | BUDHAVARA (the date's SOMAVARA) | — | — |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-11-07 | 2027-02-19 | SHUKRAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-11-13 | 2027-02-25 | GURUVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-11-26 | 2027-03-10 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-12-03 | 2027-03-17 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-12-10 | 2027-03-24 | BUDHAVARA | clean | clean |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-12-18 | 2027-04-01 | GURUVARA | day clean; each window by one of: LAGNA ARIES, LAGNA PISCES, UNWANTED_PLACEMENT 10 | day clean; each window by one of: LAGNA ARIES, LAGNA PISCES, UNWANTED_PLACEMENT 10 |
| pasni | `RAMAN_ANNAPRASANA` | the committee's sheet | 2083-12-25 | 2027-04-08 | GURUVARA | clean | clean |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | the committee's sheet | 2083-1-31 | 2026-05-14 | GURUVARA | clean | clean |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | the committee's sheet | 2083-11-20 | 2027-03-04 | GURUVARA | clean | clean |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the committee's sheet | 2083-1-21 | 2026-05-04 | SOMAVARA | clean | clean |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the committee's sheet | 2083-8-05 | 2026-11-21 | SHANIVARA | MONTH KARTIKA, NAKSHATRA REVATI | MONTH KARTIKA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the committee's sheet | 2083-8-09 | 2026-11-25 | BUDHAVARA | MONTH KARTIKA, TITHI KRISHNA_PRATIPADA | MONTH KARTIKA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the committee's sheet | 2083-8-10 | 2026-11-26 | GURUVARA | MONTH KARTIKA | MONTH KARTIKA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the committee's sheet | 2083-9-01 | 2026-12-16 | BUDHAVARA | each window by one of: NAKSHATRA PURVA_BHADRAPADA, NAKSHATRA SHATABHISHA | each window by one of: MONTH MARGASHIRSHA, NAKSHATRA PURVA_BHADRAPADA, NAKSHATRA SHATABHISHA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the committee's sheet | 2083-9-04 | 2026-12-19 | SHANIVARA | MONTH MARGASHIRSHA | MONTH MARGASHIRSHA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the committee's sheet | 2083-9-11 | 2026-12-26 | SHANIVARA | MONTH MARGASHIRSHA | MONTH MARGASHIRSHA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the committee's sheet | 2083-11-07 | 2027-02-19 | SHUKRAVARA | MONTH MAGHA | MONTH MAGHA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the committee's sheet | 2083-11-13 | 2027-02-25 | GURUVARA | MONTH MAGHA | MONTH MAGHA |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-1-07 | 2026-04-20 | SOMAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-1-21 | 2026-05-04 | SOMAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-1-25 | 2026-05-08 | SHUKRAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-1-31 | 2026-05-14 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-3-10 | 2026-06-24 | BUDHAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-3-13 | 2026-06-27 | SHANIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-3-18 | 2026-07-02 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-5-01 | 2026-08-17 | SOMAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-5-04 | 2026-08-20 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-5-14 | 2026-08-30 | RAVIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-5-16 | 2026-09-01 | MANGALAVARA | VARA MANGALAVARA | VARA MANGALAVARA |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-5-19 | 2026-09-04 | SHUKRAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-5-28 | 2026-09-13 | RAVIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-5-29 | 2026-09-14 | SOMAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-6-01 | 2026-09-17 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-6-05 | 2026-09-21 | SOMAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-7-15 | 2026-11-01 | RAVIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-7-16 | 2026-11-02 | SOMAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-7-19 | 2026-11-05 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-7-29 | 2026-11-15 | RAVIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-8-05 | 2026-11-21 | SHANIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-8-06 | 2026-11-22 | RAVIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-8-09 | 2026-11-25 | BUDHAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-8-10 | 2026-11-26 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-8-13 | 2026-11-29 | RAVIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-8-17 | 2026-12-03 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-8-19 | 2026-12-05 | SHANIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-8-26 | 2026-12-12 | SHANIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-9-04 | 2026-12-19 | SHANIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-9-11 | 2026-12-26 | SHANIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-9-15 | 2026-12-30 | BUDHAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-9-25 | 2027-01-09 | SHANIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-10-09 | 2027-01-23 | SHANIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-10-14 | 2027-01-28 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-10-29 | 2027-02-12 | SHUKRAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-11-07 | 2027-02-19 | SHUKRAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-11-13 | 2027-02-25 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-11-20 | 2027-03-04 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-11-26 | 2027-03-10 | BUDHAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-12-04 | 2027-03-18 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-12-10 | 2027-03-24 | BUDHAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-12-18 | 2027-04-01 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-12-25 | 2027-04-08 | GURUVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-12-28 | 2027-04-11 | RAVIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the committee's sheet | 2083-12-29 | 2027-04-12 | SOMAVARA | clean | clean |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the Surya Panchanga | 2082-4-26 | 2025-08-11 | SOMAVARA | each window by one of: NAKSHATRA PURVA_BHADRAPADA, NAKSHATRA SHATABHISHA | each window by one of: NAKSHATRA PURVA_BHADRAPADA, NAKSHATRA SHATABHISHA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the Surya Panchanga | 2082-7-21 | 2025-11-07 | SHUKRAVARA | MONTH KARTIKA | MONTH KARTIKA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the Surya Panchanga | 2082-8-11 | 2025-11-27 | GURUVARA | clean | clean |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the Surya Panchanga | 2082-8-19 | 2025-12-05 | SHUKRAVARA | closed: SHUKRA_ASTA | closed: SHUKRA_ASTA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the Surya Panchanga | 2082-8-20 | 2025-12-06 | SHANIVARA | closed: SHUKRA_ASTA | closed: SHUKRA_ASTA |
| griharambha | `CHINTAMANI_GRIHARAMBHA` | the Surya Panchanga | 2082-11-14 | 2026-02-26 | GURUVARA | clean | clean |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | the Surya Panchanga | 2082-10-28 | 2026-02-11 | BUDHAVARA | closed: SHUKRA_ASTA | closed: SHUKRA_ASTA |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | the Surya Panchanga | 2082-11-14 | 2026-02-26 | GURUVARA | clean | clean |
| griha pravesh | `RAMAN_GRIHA_PRAVESHA` | the Surya Panchanga | 2082-11-30 | 2026-03-14 | SHANIVARA | VARA SHANIVARA | VARA SHANIVARA |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-1-07 | 2025-04-20 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-1-08 | 2025-04-21 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-1-17 | 2025-04-30 | BUDHAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-2-04 | 2025-05-18 | RAVIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-2-14 | 2025-05-28 | BUDHAVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-2-17 | 2025-05-31 | SHANIVARA | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-3-27 | 2025-07-11 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-3-28 | 2025-07-12 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-4-05 | 2025-07-21 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-4-29 | 2025-08-14 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-5-09 | 2025-08-25 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-5-15 | 2025-08-31 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-6-11 | 2025-09-27 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-6-25 | 2025-10-11 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-6-26 | 2025-10-12 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-7-07 | 2025-10-24 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-7-17 | 2025-11-03 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-7-21 | 2025-11-07 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-7-29 | 2025-11-15 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-7-30 | 2025-11-16 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-8-14 | 2025-11-30 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-8-19 | 2025-12-05 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-8-20 | 2025-12-06 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-10-24 | 2026-02-07 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-10-28 | 2026-02-11 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-11-14 | 2026-02-26 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-11-30 | 2026-03-14 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-12-06 | 2026-03-20 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-12-11 | 2026-03-25 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-12-14 | 2026-03-28 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-12-27 | 2026-04-10 | — | clean | clean |
| dokan | `CHINTAMANI_VIPANI` | the Surya Panchanga | 2082-12-28 | 2026-04-11 | — | TITHI KRISHNA_NAVAMI | TITHI KRISHNA_NAVAMI |
