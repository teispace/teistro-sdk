# People digest: one day for many births

Status: `built`, 2026-11-01. Design for the consumer addendum's A2-1 (its
third finder), written before the code. The record is `almanac.people`
(`ts_almanac_people`, every binding's `almanac.people`, the MCP tool
`almanac.people`).

An app with a family list shows, for today, each person's running dasha,
what the sky is doing to their chart, and what today means for them. Each
part is already the SDK's: a natal reading with its dasha
(`chart.reading`), the transits from the Moon (`chart.gochar`), the hit
list (`chart.hits`), the tara and chandrashtama of the muhurta engine.
The digest asks them all once per person and the day's sky once for
everyone.

## 1. The request

```json
{"day": {"year": 2026, "month": 11, "day": 1},
 "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
 "utcOffsetSeconds": 20700,
 "people": [
   {"name": "Asha", "instant": 2447000.25, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700},
   {"name": "Ravi", "instant": 2451545.0, "latitudeDeg": 19.076, "longitudeDeg": 72.8777, "utcOffsetSeconds": 19800}],
 "dasha": "VIMSHOTTARI", "depth": 3}
```

- `day` in `calendar` at the place: where the people are today, not where
  they were born.
- `people`: 1 to 50, each a unique `name` and a birth (`instant` a UTC
  Julian day, the place and clock of birth).
- `dasha`: the system, `VIMSHOTTARI` when left out; `depth` its levels,
  1 to 6, 3 (to the pratyantardasha) when left out.

## 2. The day

The day runs **sunrise to the next sunrise** at the place (C1164): the
almanac's day, so the star and the Moon's sign it names are the ones a
panchang prints for it. Everything "running" is read at sunrise; every
event is searched over the whole day.

## 3. The answer

`{day, sunrise, nextSunrise, nakshatra, moonSign, dasha, people}`: the
day's star and the Moon's sign at sunrise, then per person, in the
request's order:

| field | what it holds | from |
|---|---|---|
| `birthStar`, `moonSign`, `lagna` | the natal Moon's star and sign, the lagna's sign | the natal reading |
| `dasha` | the periods running at sunrise, one per level: `{level, lord, sign, from, to}` | the natal dasha |
| `dashaOpens` | the periods at any level opening before the next sunrise | the same |
| `transits` | all nine grahas' gochar from the natal Moon, with vedha | `chart.gochar` at sunrise |
| `hits` | the day's transit hits on their natal points | `chart.hits` over the day |
| `tara`, `chandraHouse`, `chandrashtama` | the day's star counted from the birth star; the Moon's house from the natal Moon, and whether it is the 8th | the muhurta engine's tara |
| `sadeSati` | whether Saturn is in the 12th, 1st or 2nd from the natal Moon | the gochar reading |

"Notable" is not a filter the SDK chooses (C1163): every graha's gochar is
given, with its verdict, and every hit the day's search finds. An app
showing only the bad ones filters on the verdicts; nothing is hidden by a
threshold it cannot see.

## 4. Refusals

Named under `people`: a key it does not read; a place, offset or instant
out of range (`people.people[i].latitudeDeg`); no person
(`people.people`) or more than 50 (`LIMIT`); a name given twice
(`people.people[i].name`); a depth past 1 to 6 (`people.depth`). A
context without an ephemeris is `CAPABILITY`, as is a build without the
chart or the muhurta family.

## 5. Cost

One almanac day, one natal reading per person, one hit search for all of
them (`hits_many` walks the day's sky once), one gochar per person.
Measured by criterion in `crates/sdk/benches/finders.rs`
(`people digest, 1 and 10 people`).

## 6. Cruxes

C1163 (what a notable transit is), C1164 (the day a digest reads).
