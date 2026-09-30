# The season, measured

Status: `generated` by `cargo xtask ritu`. Do not edit: `check-ritu`
regenerates this page and fails on any difference.

It measures the almanac's `sun.ritu` and `sun.ayana` at Kathmandu
against Nepal's daily panchanga. The Surya Siddhanta (XIV.10) counts
six seasons from the Sun's entry into Capricorn, two signs each
(C178); which *day* a season begins on is the day its solar month
begins on, and a sankranti falls at an instant, so the month's first
day is placed by a rule (C186). `panchanga.ritu` chooses the months
the season is read from and `panchanga.solar_month_start` the rule.

## 1. The claims

| proposed rule | verdict | measured |
|---|---|---|
| the Bikram Sambat calendar's months (`BIKRAM_SAMBAT`, `nepali-default`'s): the seasons Nepal printed | **holds** | 0 of 341 disagree |
| the punya-kala over the modern Sun (`PUNYAKALA`, the root's): the seasons Nepal printed | falsified | 2 of 341 disagree; a rival |
| the civil day of the modern sankranti (`SANKRANTI_DAY`): the seasons Nepal printed | falsified | 2 of 341 disagree; a rival |
| the civil day after it (`FOLLOWING_DAY`): the seasons Nepal printed | falsified | 4 of 341 disagree; a rival |
| the almanac day holding it (`SUNRISE_TO_SUNRISE`): the seasons Nepal printed | falsified | 3 of 341 disagree; a rival |
| before sunset, else the next day (`BEFORE_SUNSET`): the seasons Nepal printed | falsified | 1 of 341 disagree; a rival |
| before aparahna, else the next day (`BEFORE_APARAHNA`): the seasons Nepal printed | **holds** | 0 of 341 disagree; a rival the records cannot separate |
| the modern Sun's sign as the day opens (no setting): the seasons Nepal printed | falsified | 2 of 341 disagree; a rival |
| the lunar month's season (`panchanga.ritu` `LUNAR`): the seasons Nepal printed | falsified | 62 of 341 disagree; a rival |
| the tropical Sun's (`panchanga.ritu` `TROPICAL`): the seasons Nepal printed | falsified | 139 of 341 disagree; a rival |
| the ayana, the sidereal Sun's sign as the day opens: the ayana Nepal printed | **holds** | 0 of 341 disagree |
| before aparahna, else the next day (`BEFORE_APARAHNA`): the season changes of Bikram Sambat 2070 to 2095 | falsified | 25 of 156 disagree; a rival the records cannot separate |

## 2. The records

Makalukhabar's daily "आजको पञ्चाङ्ग" posts, 341 days from 2025-04-09 to 2026-09-30,
each read for its Gregorian date, its season and its ayana; the page
holds them in `xtask/src/ritu.rs`, a month a row. 73 VASANTA, 97 GRISHMA, 87 VARSHA, 35 SHARAD, 22 HEMANTA, 27 SHISHIRA.

## 3. Each reading over the records

| reading | days it misses |
|---|---|
| the Bikram Sambat calendar's months (`BIKRAM_SAMBAT`, `nepali-default`'s) | none |
| the punya-kala over the modern Sun (`PUNYAKALA`, the root's) | 2025-11-16 (HEMANTA), 2026-07-16 (VARSHA) |
| the civil day of the modern sankranti (`SANKRANTI_DAY`) | 2025-11-16 (HEMANTA), 2026-07-16 (VARSHA) |
| the civil day after it (`FOLLOWING_DAY`) | 2025-05-15 (VASANTA), 2026-03-15 (SHISHIRA), 2026-05-15 (VASANTA), 2026-09-17 (VARSHA) |
| the almanac day holding it (`SUNRISE_TO_SUNRISE`) | 2025-11-16 (HEMANTA), 2026-03-14 (VASANTA), 2026-07-16 (VARSHA) |
| before sunset, else the next day (`BEFORE_SUNSET`) | 2025-11-16 (HEMANTA) |
| before aparahna, else the next day (`BEFORE_APARAHNA`) | none |
| the modern Sun's sign as the day opens (no setting) | 2026-05-15 (VASANTA), 2026-09-17 (VARSHA) |
| the lunar month's season (`panchanga.ritu` `LUNAR`) | 62 of 341 |
| the tropical Sun's (`panchanga.ritu` `TROPICAL`) | 139 of 341 |

## 4. The days a season changed

Each recorded day whose season differs from the recorded day before
it, with its Bikram Sambat date. A cell is what the reading names;
a miss is marked.

| day | Bikram Sambat | printed | BIKRAM_SAMBAT | PUNYAKALA | SANKRANTI_DAY | FOLLOWING_DAY | SUNRISE_TO_SUNRISE | BEFORE_SUNSET | BEFORE_APARAHNA | sign at opening |
|---|---|---|---|---|---|---|---|---|---|---|
| 2025-05-15 | 2082 02 01 | GRISHMA | GRISHMA | GRISHMA | GRISHMA | **VASANTA** | GRISHMA | GRISHMA | GRISHMA | GRISHMA |
| 2025-07-17 | 2082 04 01 | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA |
| 2025-09-20 | 2082 06 04 | SHARAD | SHARAD | SHARAD | SHARAD | SHARAD | SHARAD | SHARAD | SHARAD | SHARAD |
| 2025-11-17 | 2082 08 01 | HEMANTA | HEMANTA | HEMANTA | HEMANTA | HEMANTA | HEMANTA | HEMANTA | HEMANTA | HEMANTA |
| 2026-01-16 | 2082 10 02 | SHISHIRA | SHISHIRA | SHISHIRA | SHISHIRA | SHISHIRA | SHISHIRA | SHISHIRA | SHISHIRA | SHISHIRA |
| 2026-03-15 | 2082 12 01 | VASANTA | VASANTA | VASANTA | VASANTA | **SHISHIRA** | VASANTA | VASANTA | VASANTA | VASANTA |
| 2026-05-15 | 2083 02 01 | GRISHMA | GRISHMA | GRISHMA | GRISHMA | **VASANTA** | GRISHMA | GRISHMA | GRISHMA | **VASANTA** |
| 2026-07-17 | 2083 04 01 | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA | VARSHA |
| 2026-09-17 | 2083 06 01 | SHARAD | SHARAD | SHARAD | SHARAD | **VARSHA** | SHARAD | SHARAD | SHARAD | **VARSHA** |

## 5. Where the records cannot separate a rival

A rival that names every recorded season is founded around each
month the Bikram Sambat calendar begins a season with, over the
years below: the day before the month and its first day.

| reading | Bikram Sambat years | season changes it misses |
|---|---|---|
| before aparahna, else the next day (`BEFORE_APARAHNA`) | 2070 to 2095 | 25 of 156 |

## 6. What the records decide

The season follows the civil calendar's month. Of the 6 rules
over the modern Sun, 5 miss a recorded day, and the Sun's
sign as the day opens misses the months whose sankranti fell after
dawn; a rule that names every recorded season still misses the
calendar's season changes in §5, so it agrees with these months by
coincidence and not by rule. So `nepali-default` reads the shipped
Bikram Sambat calendar, and the root, which has no civil calendar
of Nepal's, places the modern sankranti by the punya-kala, the
Dharmasindhu's rule the calendar itself follows
(`calendar-bikram-sambat.md` §4, where the modern Sun's months are
measured against the official table).
