# The Bangladeshi national calendar

Status: `built`, 2026-10-23. `Calendar::Bangladeshi`
(`calendar.BANGLADESHI`), `crates/calendar/src/bangladeshi.rs`. P2-1.

This is the revised Bengali calendar of Bangladesh, counted in the
Bengali era (Bangabda). Bangla Academy keeps it, and Bangladesh's
national days are dated by it: Pohela Boishakh, Ekushey on 8 Falgun,
Victory day on 1 Poush. Its current rules took effect on 1 Kartik 1426,
17 October 2019.

The calendar is arithmetic on the Gregorian calendar. Nothing in it is
astronomical, so it needs no place and no ephemeris. The Bengali
calendar of West Bengal, Tripura and Assam is a different calendar: it
is a solar calendar whose months begin at the sankrantis
(`calendars/regional-solar.md`), and it waits on a published list.

## The definition

| | |
|---|---|
| year | the Bengali year *B* begins in the Gregorian year *B* + 593, on 14 April |
| Boishakh to Ashwin (months 1–6) | 31 days each |
| Kartik to Magh (months 7–10) | 30 days each |
| Falgun (month 11) | 29 days, or 30 when the Gregorian year it falls in is a leap year |
| Choitro (month 12) | 30 days |
| leap year | *B* is a leap year when the Gregorian year *B* + 594 is |

A common year has 6 × 31 + 4 × 30 + 29 + 30 = 365 days. Falgun runs
from 14 February to 14 March, so it takes the Gregorian leap day, and
every month begins on the same Gregorian date every year:

| month | first day |
|---|---|
| Boishakh | 14 April |
| Joishtho | 15 May |
| Asharh | 15 June |
| Srabon | 16 July |
| Bhadro | 16 August |
| Ashwin | 16 September |
| Kartik | 17 October |
| Ogrohayon | 16 November |
| Poush | 16 December |
| Magh | 15 January |
| Falgun | 14 February |
| Choitro | 15 March |

The national days hold their Bengali dates every year. 21 February
is 8 Falgun, 26 March is 12 Choitro and 16 December is 1 Poush. Ending
the drift of Ekushey between 8 and 9 Falgun was the revision's stated
purpose.

Years are astronomical, as the Gregorian years they follow. The calendar
converts every Bengali year that begins and ends within the Gregorian
calendar's own range, −10592 to 9405. A date carries the `BENGALI` era.

## Before the revision

From 1987 Bangladesh kept the 1966 committee's scheme (Muhammad
Shahidullah's): Boishakh to Bhadro had 31 days and the rest 30, with
Falgun lengthened to 31 in a leap year. Under that scheme Ashwin had 30
days, so Kartik to Magh began a day earlier and Pohela Falgun fell on
13 February. None of the sources read here states that scheme's leap
year, whether Gregorian or counted in Bengali years. So the SDK reckons
every date by the 2019 rules, proleptically, as its Gregorian calendar is
proleptic, and does not claim the dates printed between 1987 and 2019.
A consumer that needs those dates can ask for the 1987 scheme as a
second calendar, once a source settles its leap year.

## Sources

- Wikipedia, "Bangladeshi national calendar" and "Bengali calendars"
  (read 2026-10-23). Both give the 2019 month lengths and month starts
  and the 1966 scheme. The first notes the revision in force from 16 and
  17 October 2019, citing *Prothom Alo*, 17 October 2019.
- *Dhaka Tribune*, "Ashshin in 31 days in revised Bangla calendar"
  (2019), quoting Bangla Academy's secretary, and its 2026 report. Both
  say Falgun has 29 days, and 30 in a Gregorian leap year.
- *The Daily Star*, "Pahela Falgun, Valentine's Day: '2 days' in 24
  hours" (2020). It reports Pohela Falgun moving from 13 to 14 February.

## How it is checked

- **The table.** Every month of 101 years (1400 to 1500) begins on its
  Gregorian date.
- **Dated facts.** These dates are checked:
  - 1 Kartik 1426 is 17 October 2019, the revision's first day.
  - 1 Boishakh 1433 is 14 April 2026.
  - Ekushey (21 February 2025) is 8 Falgun 1431.
  - Victory day (16 December 2025) is 1 Poush 1432.
  - Independence day (26 March 2026) is 12 Choitro 1432.
  - Falgun 1430 has a 30th day, 14 March 2024, because 2024 is a leap
    year. 30 Falgun 1431 is refused.
- **Every day.** Every day of the range converts and converts back. The
  dates rise strictly, and no day falls outside its month.
