# Malmas, measured

Status: `generated` by `cargo xtask malmas`. Do not edit:
`check-malmas` regenerates this page and fails on any difference.

It measures `calendar-indian-lunisolar.md` §9: the days the almanac
marks `ADHIKA` at Kathmandu on Nepal's clock, against the adhika
months the Nepal Panchanga Nirnayak Vikas Samiti announced, which
Nepal calls *malmas* (C177).

## 1. The claims

| proposed rule | verdict | measured |
|---|---|---|
| `nepali-default` over the built-in ephemeris: the adhika days are the committee's | **holds** | 0 of 5 disagree |
| `surya-siddhanta` over the text: the adhika days are the committee's | **holds** | 0 of 5 disagree |
| `nepali-default` over the text's sky: the adhika days are the committee's | falsified | 4 of 5 disagree; a rival: the text's Sun read in Lahiri's zodiac |
| an adhika day's purnimanta month is its amanta month | **holds** | 0 of 442 disagree |

## 2. Each announcement

An announcement that gives instants is read by each reading's own
sunrises: its days are those whose sunrise falls between them.

| year | announced | source | `nepali-default` over the built-in ephemeris | `surya-siddhanta` over the text | `nepali-default` over the text's sky |
|---|---|---|---|---|---|
| 2072 | BS 2072/03/01 23:23 to BS 2072/03/31 06:22 | ekantipur, 16 June 2015, quoting the committee's chair | BS 2072/03/02 to BS 2072/03/31, as announced | BS 2072/03/02 to BS 2072/03/31, as announced | BS 2072/02/05 to BS 2072/03/01, not BS 2072/03/02 to BS 2072/03/31 |
| 2075 | BS 2075/02/02 to BS 2075/02/30 | a1samachar, 18 May 2018, and the year's patros | BS 2075/02/02 to BS 2075/02/30, as announced | BS 2075/02/02 to BS 2075/02/30, as announced | BS 2075/01/04 to BS 2075/02/01, not BS 2075/02/02 to BS 2075/02/30 |
| 2077 | BS 2077/06/02 to BS 2077/06/30 | Nagarik and Ratopati, 18 September 2020, quoting the committee | BS 2077/06/02 to BS 2077/06/30, as announced | BS 2077/06/02 to BS 2077/06/30, as announced | BS 2077/05/04 to BS 2077/06/01, not BS 2077/06/02 to BS 2077/06/30 |
| 2080 | BS 2080/04/02 to BS 2080/04/31 | Gorkhapatra and Ratopati, July and August 2023, quoting the committee | BS 2080/04/02 to BS 2080/04/31, as announced | BS 2080/04/02 to BS 2080/04/31, as announced | BS 2080/03/04 to BS 2080/04/01, not BS 2080/04/02 to BS 2080/04/31 |
| 2083 | BS 2083/02/03 to BS 2083/03/01 | Onlinekhabar and Ratopati, May and June 2026, quoting the committee | BS 2083/02/03 to BS 2083/03/01, as announced | BS 2083/02/03 to BS 2083/03/01, as announced | BS 2083/02/03 to BS 2083/03/01, as announced |

## 3. The announced instants against each sky's new moons

Minutes from the announced instant to the new moon each reading
found, positive when the new moon is later; a reading that found
another month has no new moon to set beside it.

| year | boundary | announced (UTC JD) | `nepali-default` over the built-in ephemeris | `surya-siddhanta` over the text | `nepali-default` over the text's sky |
|---|---|---|---|---|---|
| 2072 | opens | 2457190.23472 | -213 | -238 | another month |
| 2072 | closes | 2457219.52569 | +47 | -6 | another month |
