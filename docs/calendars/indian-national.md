# The Indian national calendar

Status: `built`, 2026-10-20. `Calendar::IndianNational`
(`calendar.INDIAN_NATIONAL`), `crates/calendar/src/indian_national.rs`.

The civil calendar of the Government of India, counted in the Saka era.
The Calendar Reform Committee proposed it (*Report of the Calendar Reform
Committee*, Council of Scientific and Industrial Research, 1955, part C),
and India adopted it from 1 Chaitra 1879, 22 March 1957. The *Gazette of
India* and All India Radio date by it beside the Gregorian date.

It is arithmetic on the Gregorian calendar. Nothing in it is astronomical,
so it needs no place and no ephemeris.

## The definition

| | |
|---|---|
| year | the Saka year *S* begins in the Gregorian year *S* + 78 |
| leap year | when that Gregorian year is a leap year |
| 1 Chaitra | 22 March, or 21 March in a leap year |
| Chaitra | 30 days, 31 in a leap year |
| Vaishakha to Bhadra (months 2–6) | 31 days each |
| Ashvina to Phalguna (months 7–12) | 30 days each |

A common year has 30 + 5 × 31 + 6 × 30 = 365 days. A leap year adds its
day to Chaitra, so every month after Chaitra begins on the same Gregorian
date every year: Vaishakha on 21 April, Ashvina on 23 September and
Phalguna on 20 February.

Years are astronomical, as the Gregorian years they follow are. The
calendar converts every Saka year that begins and ends within the
Gregorian calendar's own range, −10077 to 9920. A date carries the
`SHAKA` era.

## The months

The months are named in the order the committee gives. en-Latn spells
them as the Government of India's English publications do, without
diacritics: Chaitra, Vaisakha, Jyaistha, Asadha, Sravana, Bhadra, Asvina,
Kartika, Agrahayana, Pausa, Magha and Phalguna. ne-Deva-NP writes them in
Devanagari: चैत्र, वैशाख, ज्येष्ठ, आषाढ, श्रावण, भाद्र, आश्विन, कार्तिक,
अग्रहायण, पौष, माघ and फाल्गुन. These are calendar months and not lunar
months, so they are not the `masa` catalogue's members, though they share
the names.

## How it is checked

- **The committee's definition.** 1 Chaitra 1879 is 22 March 1957, and
  1 Chaitra 1946 is 21 March 2024, a leap year. Vaishakha, Ashvina and
  Phalguna begin on their fixed Gregorian dates. A year ends on the day
  before the next 1 Chaitra.
- **A published date.** India's independence, 15 August 1947, is
  Shravana 24, 1869.
- **Every day.** Every day of the range converts and converts back. The
  dates rise strictly, and no day falls outside its month.

## What it is not

- **Not the Shalivahana Shaka lunisolar reckoning.** The Saka *era* also
  numbers the years of the lunisolar calendar of South and West India, as
  the `SHAKA` era of `Calendar::IndianLunisolar`. That calendar's months
  are lunar, and they begin at new or full moons, not on fixed dates.
- **Not a regional solar calendar.** The Tamil, Bengali, Odia and
  Malayalam calendars begin each month at a sankranti, placed on a civil
  day by the tradition's rule. Those rules are built as
  `solar::MonthStartRule` (`SankrantiDay`, `FollowingDay`, `BeforeSunset`,
  `BeforeAparahna`). The calendars that use them are not yet catalogue
  members.
