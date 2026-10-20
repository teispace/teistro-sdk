# The Hijri calendar

Status: `built` for the tabular calendar, 2026-10-20.
`Calendar::HijriTabular` (`calendar.HIJRI_TABULAR`), era `era.HIJRI`,
`crates/calendar/src/hijri.rs`. A sighted month waits on published
overrides (P2-4).

## Two Hijri calendars

The Islamic calendar is lunar. A month begins when the new crescent is
**seen**, so where it begins depends on the sky, the weather and the
authority that declares it. Nepal and India date Eid and Muharram by a
sighting. That answer is published each year rather than computed, and
it can part from any rule by a day or two.

The **tabular** calendar replaces the sighting with a rule. Historians,
astronomers and software use it to convert dates, and it is what the SDK
computes:

| | |
|---|---|
| epoch | 1 Muharram 1 AH is Friday 16 July 622, Julian (19 July 622, proleptic Gregorian): the *civil* epoch |
| months | twelve, alternately 30 and 29 days, Muharram first; Dhu al-Hijja has 30 days in a leap year |
| leap years | eleven in every thirty: the 2nd, 5th, 7th, 10th, 13th, 16th, 18th, 21st, 24th, 26th and 29th |
| a cycle | 30 × 354 + 11 = 10,631 days |

The rule is Reingold and Dershowitz's (*Calendrical Calculations*,
ch. 7). It is the same as ICU's `islamic-civil`, which every major
browser and operating system ships. Years are astronomical: the year
before 1 AH is year 0. The calendar converts every Hijri year inside the
Gregorian calendar's range, −10946 to 9665.

Two variants are not shipped. The *astronomical* epoch (Thursday, 15 July
622) moves every date a day earlier. Saudi Arabia's Umm al-Qura calendar
is computed from the conjunction and moonset at Mecca, so it is not
tabular at all.

## How it is checked

- **The epoch.** 1 Muharram 1 AH is 19 July 622 (Gregorian), a Friday.
  The leap years of the first cycle are the list above, and a cycle has
  10,631 days.
- **Against ICU.** Every day from 1900 to 2100, and every tenth day from
  622 to 3000, gives the same date as ICU's `islamic-civil` read through
  Node's `Intl.DateTimeFormat`: 160,249 days, none different. Seven of
  them are pinned in the crate's tests, among them 1 Ramadan 1446
  (1 March 2025) and the leap day 30 Dhu al-Hijja 1445 (7 July 2024).
- **Every day.** Every day of the range converts and converts back. The
  dates rise strictly, and no day falls outside its month.

## Names

en-Latn names the months and date forms as CLDR's English `islamic`
calendar does (Muharram, Safar, Rabiʻ I … Dhuʻl-Hijjah), and writes "AH"
in the date. CLDR gives Nepali no Hijri month names of its own: its
`ne` data keeps the English ones, and orders the date "AH 1446 Ramadan 1".
ne-Deva-NP, which must carry every key en-Latn names, says exactly that,
with its own digits and weekday. No vetted Nepali or Hindi name for the
era or the calendar is in hand, so `era.HIJRI` and the calendar are
listed as unnamed in `xtask/src/intl.rs`, and Hindi falls back to English.

## What waits

**A sighted calendar.** A consumer that needs the declared Eid wants the
published month beginnings, carried as data and laid over the tabular
calendar: P2-4's published overrides. The tabular date is then the
answer only where no declaration covers the month.
