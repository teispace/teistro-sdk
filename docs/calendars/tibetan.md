# The Tibetan calendar, Phugpa

Status: `shipped`, 2026-10-21. P2-1 step 5: `calendar.TIBETAN_PHUGPA`
(`crates/calendar/src/tibetan.rs`) converts every day from the Tibetan
year 1027 to 9998, and gives Losar on every date the source lists. A
festival rule on a Tibetan date is not reckoned yet (The festival day).

The Tibetan new year, Losar, is the first day of the first month of the
Tibetan calendar. This page designs the Phugpa calendar: in Janson's
words the most common version, the one the Dalai Lama follows. The
Tsurphu version, and Bhutan's and Mongolia's, "frequently differ by a day
or a month" (§1, appendix A). Unlike the Indian calendars it needs no
ephemeris, no sunrise and no place. It is arithmetic, with two short
tables standing in for a sine.

Nepal's day is not this calendar's. The committee prints Gyalpo Lhosar on
Phalgun's bright first, and that is what `NEPAL_PANCHANGA` ships (see
Measured). So the calendar is for a consumer that wants the Tibetan date
or the Tibetan Losar itself, not for Nepal's holiday.

## Sources

- Svante Janson, *Tibetan Calendar Mathematics* (Uppsala, 2007, revised
  2014). It states the whole calendar in modern notation, with the
  traditional constants and three equivalent epochs, and Table 1 lists the
  Gregorian date of Losar for 1927 to 2046.
- Edward Henning, *Kālacakra and the Tibetan Calendar* (2007), Janson's
  reference [7]. It gives the traditional reckoning and the Phugpa
  constants that Janson restates.
- Reingold and Dershowitz, *Calendrical Calculations*, ch. 21, the
  Tibetan calendar as Lisp, built on Henning's and Janson's account. It is
  a second statement to check an implementation against.

## The arithmetic

Every constant is an exact fraction, so the calendar is integer
arithmetic, as `hijri.rs` and `indian_national.rs` are. With the epoch
E806 (month 3 of 806, β* = 61):

| step | formula (Janson) |
|---|---|
| true month count *n* of month *M*, year *Y* | ⌊(67(12(*Y* − 806) + *M* − 3) + 61 + 17) / 65⌋, less 1 for a leap month (5.10) |
| a leap month | the intercalation index (2*M** + 61) mod 65 is 48 or 49 (5.8) |
| mean date | *n* · 167025/5656 + *d* · 11135/11312 + 2015501 + 4783/5656 (7.1) |
| mean sun | *n* · 65/804 + *d* · 13/4824 + 743/804 (7.5) |
| moon's anomaly | *n* · 253/3528 + *d*/28 + 475/3528 (7.11) |
| equations | moon table 0 5 10 15 19 22 24 25, sun table 0 6 10 11, symmetric, interpolated (7.18, 7.21) |
| true date | mean date + moon equation/60 − sun equation/60 (7.22) |
| the day | JD = ⌊true date⌋ of the lunar day's end (8.1) |

A calendar day takes the number of the lunar day that ends in it. When
two end in one day, the second's number is skipped; when none ends, the
day repeats the next day's number, and the first of the pair is the leap
day ("Extra" in the almanacs). Two leap months fall in every 65 years.

The moon's anomaly per day is the rounded 1/28 the almanacs use, not
Henning's exact 3781/105840. The two give different dates about one day
in 4,100, the next on 19 November 2025 (Janson, Remark 14). A variant
that names Henning's value can follow if an almanac is found to print it.

## Measured

The calendar is integer arithmetic over one common denominator,
102,317,040: the least common multiple of the mean date's 11,312, the
moon equation's 126 · 60 and the sun equation's 402 · 60. Every end of a
lunar day it computes equals a prototype's in exact fractions, over
32,000 months either side of the epoch. Its tests hold:

- Losar on the day Janson's Table 1 prints, all 120 years from 1927 to
  2046, as the day after the last day of the year before's month 12; and
  2000's beginning with a leap month 1 on Sunday 6 February (§8).
- Every day from 1900 to 2100 converts and converts back, its marks
  included; the dates rise strictly, a leap month before its twin and a
  leap day before its; and every leap month falls where (5.24) puts it.
- Every month holds 29 or 30 days, and a year's months, its leap month
  included, sum to the days from its Losar to the next.
- A skipped day, a leap day that is not repeated, and a leap month the
  year does not have are each refused by name.

**Gyalpo Lhosar in Nepal is a Hindu-month day.** The committee prints it
on 18 February 2026 (VS 2082 p. 23), which is also the Phugpa Losar. But
VS 2083 p. 25 prints "चन्द्रोदय, ग्याल्पो ल्होसार" on 9 March 2027, a
lunation after the Phugpa Losar of 7 February. A consumer's fourteen-year
list agrees with the print: of its 13 Gyalpo Lhosar days (2014 to 2027),
the Phugpa Losar gives 9, and Phalgun's bright first at sunrise gives all
13. The four that part (2019, 2021, 2024 and 2027) are the years Phalgun
falls a lunation after the Tibetan first month. So `NEPAL_PANCHANGA` ships
`GYALPO_LHOSAR` as that lunar rule, beside Sonam Lhosar.

Two more Lhosars the committee prints are not Tibetan dates. Sonam Lhosar
(19 January 2026, 7 February 2027) and Tol Lhosar (20 December 2025,
8 January 2027) are the first of the bright fortnight of the Hindu months
Magha and Paush: VS 2083's adhika Jyeshtha moves them a lunation against
the Tibetan months, so Tol Lhosar is Tibetan 11/1 in one year and 12/1 in
the next. Both ship as lunar rules at sunrise in `NEPAL_PANCHANGA`.
Sonam Lhosar gives all 14 of the consumer's verified days, 2014 to 2027,
and Tol Lhosar both printed days under the committee's sky. Over the
modern sky Tol Lhosar's 2025 day moves to 21 December, because there Paush's
new moon ends after sunrise on the 20th. An earlier note here read that as
the sunrise rule failing. It was the sky.

## What crosses

A Tibetan date is a year, a month, a day, and two marks: whether the
month is the leap month, and whether the day is the leap day. The
calendar systems in `crates/calendar` carry neither mark today, because
no shipped calendar repeats a month or a day.

- **`CalendarDate` gains `leap: Option<Leap>`** (`MONTH`, `DAY`, or
  `MONTH_AND_DAY`), left out of the wire when absent. No date any other
  calendar answers carries it, so no existing answer's hash changes, and
  every other calendar refuses a date that carries one. At the C boundary
  it takes one of `ts_calendar_date`'s three reserved bytes (`ts_leap`,
  `0xFF` for none), so the struct keeps its 24 bytes; `0`, what a caller
  compiled before left there, is read as none too, so every compiled
  caller keeps working. Each binding's date carries it as an optional
  field, and each `date` helper takes it as an optional last argument.
- **`calendar.TIBETAN_PHUGPA`**, a `CalendarSystem` with no place and no
  ephemeris. Its constants are one record (β*, the epoch and the eight
  fractions), so Tsurphu, Bhutan and Mongolia follow as further members
  once appendix A's constants are checked against an almanac of each. `fixed_of` takes the marks: a leap month or leap day that does
  not exist that year is refused by name. A skipped day is refused too,
  and the refusal says which day stands in for it.
- **The year.** Years are counted as Janson's *Y*, the Gregorian year in
  which the Tibetan year starts and most of it falls. A date carries no
  era: the date forms print the year they are given, so a royal-year era
  would print 2153 for 2026. The royal year is *Y* + 127. The rabjung
  cycle and the element and animal name are a view on the year still to
  come, as the samvatsara already is.
- **The festival day.** Losar is the first day of month 1, or of the leap
  month 1 when there is one, and the day after month 12's day 30 when day 1
  is skipped (Janson §8); `tibetan::losar` gives it. A `DatedRule` on a
  Tibetan date is refused for now, by name: its label alone does not say
  which of a repeated month's two months keeps a festival (Janson's
  Remark 17: holidays are usually not kept in a leap month, Losar
  excepted), nor which day stands in for a skipped number. Nepal's
  `GYALPO_LHOSAR` is another day in some years, so it does not wait on
  this.
- **Names.** en-Latn names the months by number ("month 1", "leap month
  1") and a repeated day "leap day 17", as Janson does. ne-Deva-NP says the
  same plainly ("अधिक महिना १", "तिथि १७ (दोहोरिएको)"). The Tibetan month
  names, and Nepali or Hindi ones, wait on a vetted source.

## How it is checked

- The 120 Losar dates of Janson's Table 1, every one (tested).
- Every day from 1900 to 2100 converts and converts back, the marks
  included, and every leap month falls where (5.24) puts it (tested).
- Every binding converts 6 February 2000 to the leap month 1's first day
  of 2000, and back.
- The committee's print, where it agrees: 18 February 2026 is both the
  Phugpa Losar and the printed Gyalpo Lhosar.
- Not yet: the sample dates in *Calendrical Calculations*' appendix C,
  which is not in hand here.
