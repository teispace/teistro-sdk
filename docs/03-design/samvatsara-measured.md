# The sixty-year cycle, measured

Status: `generated` by `cargo xtask samvatsara`. Do not edit:
`check-samvatsara` regenerates this page and fails on any difference.

It measures `teistro_calendar::samvatsara` and `Almanac::years`: the
Surya Siddhanta's Jovian count (I.55), and the name Nepal's panchanga
committee gives a lunar year, the Jovian year that meets Chaitra
Shukla Pratipada (C180), with a name never naming two years (C184).

## 1. The claims

| proposed rule | verdict | measured |
|---|---|---|
| `BARHASPATYA`, shipped: the years Nepal named | **holds** | 0 of 7 disagree |
| `BARHASPATYA_RUNNING`: the years Nepal named | falsified | 1 of 7 disagree; a rival |
| `CHANDRAMANA`: the years Nepal named | falsified | 7 of 7 disagree; a rival |
| the text with Burgess's bija: the years Nepal named | falsified | 5 of 7 disagree; a rival |
| running at Baishakh 1: the years Nepal named | falsified | 2 of 7 disagree; a rival |
| the baseline engine's fixed phase: the years Nepal named | falsified | 5 of 7 disagree; a rival |
| the text's count names Burgess's worked example: the 5019th year, Prajapati, begun 23 February 1859 or 3 April with the bija | **holds** | 0 of 1 disagree |
| the press's dated Jovian years begin within 1.5 days of the text's | **holds** | 0 of 2 disagree |
| the shipped count's walk of at most 4 years back is the full recursion | **holds** | 0 of 587 disagree |

## 2. The years Nepal named

Each year is the one holding 1 August of its Vikrama year, founded at
Kathmandu under `nepali-default`; a mismatch is marked.

| Vikrama | named | source | `BARHASPATYA`, shipped | `BARHASPATYA_RUNNING` | `CHANDRAMANA` | the text with Burgess's bija | running at Baishakh 1 | the baseline engine's fixed phase |
|---|---|---|---|---|---|---|---|---|
| 2076 | PARIDHAAVI | Onlinekhabar, 25 March 2020: परिधावी ran until the new year | PARIDHAAVI | PARIDHAAVI | **VIKARI** | PARIDHAAVI | **PRAMADICHA** | PARIDHAAVI |
| 2077 | PRAMADICHA | Onlinekhabar, 25 March 2020: प्रमादी from that day | PRAMADICHA | PRAMADICHA | **SHARVARI** | PRAMADICHA | **ANANDA** | PRAMADICHA |
| 2078 | RAKSHASA | Lokpath, 13 April 2021, the committee's chair: आनन्द did not meet the pratipada | RAKSHASA | RAKSHASA | **PLAVA** | **ANANDA** | RAKSHASA | **ANANDA** |
| 2079 | NALA | Ratopati, 14 April 2022, the committee's chair: नल from Chaitra 19 | NALA | NALA | **SHUBHAKRIT** | **RAKSHASA** | NALA | **RAKSHASA** |
| 2080 | PINGALA | Makalukhabar, January 2023, the committee on the year's patros | PINGALA | **NALA** | **SHOBHAKRIT** | **NALA** | PINGALA | **NALA** |
| 2081 | KALAYUKTA | Nagarik, April 2024 | KALAYUKTA | KALAYUKTA | **KRODHI** | **PINGALA** | KALAYUKTA | **PINGALA** |
| 2082 | SIDDHARTHI | Nepal Khabar, 14 April 2025: सिद्धार्थी from Chaitra 17 | SIDDHARTHI | SIDDHARTHI | **VISHVAVASU** | **KALAYUKTA** | SIDDHARTHI | **KALAYUKTA** |

## 3. Jovian years dated

Days from the recorded date's midnight to the computed first instant:
Burgess's on Greenwich's clock, the press's on India's.

| Jovian year | recorded | source | computed first instant (UTC JD) | days |
|---|---|---|---|---|
| PRAJOTPATTI (the text) | 1859-02-23 | Burgess, note to I.55 | 2400097.3756 | -1.12 |
| PRAJOTPATTI (with the bija) | 1859-04-03 | Burgess, note to I.55 | 2400137.1685 | -0.33 |
| ANANDA | 2020-04-04 | Patrika (Jaipur), April 2021 | 2458944.7311 | +1.46 |
| RAKSHASA | 2021-04-01 | Patrika (Jaipur), April 2021 | 2459305.7579 | +0.49 |

## 4. 1865 to 2455, at Kathmandu

Every year founded under `nepali-default`. A repeat is a year whose
running Jovian year already ran at the year before's pratipada; the
shipped count names it one ahead, and the lead lasts until a year
whose count rises by two, where the expunged name falls. The longest
run of advanced years is 2, inside the walk's bound of
4.

| what | count | years |
|---|---|---|
| years founded | 591 | |
| repeats under `BARHASPATYA_RUNNING` | 8 | 1909, 1993, 2080, 2164, 2251, 2334, 2418, 2422 |
| named one ahead under `BARHASPATYA` | 10 | 1909, 1993, 2080, 2164, 2251, 2334, 2335, 2418, 2419, 2422 |
| the Jovian year changed between the opening new moon and the first sunrise | 2 | 1992, 2163 |
| a Jovian year expunged under `BARHASPATYA` | 7 | 1907 (DUNDUBHI), 1991 (SARVAJIT), 2077 (ANANDA), 2162 (VIKRAMA), 2248 (PLAVANGA), 2332 (ANGIRAS), 2416 (HEVILAMBI) |
