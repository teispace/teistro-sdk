# The annual dashas (Mudda, Varsha Yogini, Patyayini)

Status: `built`, 2026-09-24. Written after reading both sources and
before writing any of it, and corrected by the building (the clocks'
gap, below). Measured over every recorded year in
[`muntha-measured.md`](muntha-measured.md) §19 (`check-muntha`). Cruxes
C122 (the clock) and C123 (the balance).
Built on the annual chart ([`annual-chart.md`](annual-chart.md)), the
Panchavargiya bala ([`panchavargiya.md`](panchavargiya.md)) and the
nakshatra kernel's rows ([`dasha-kernels.md`](dasha-kernels.md)).

A natal dasha divides a life. An annual dasha divides **one year**, the
year a return opens, among the same kinds of lord. K.S. Charak's
*Textbook of Varshaphala* ch. V (rank 2) gives three, with a worked year
for each. The *Tajika Nilakanthi*'s Samjna-tantra gives the Patyayini,
and its appendix gives the Mudda. Where the two books differ, the SDK
builds a knob.

## The three, as the sources give them

| system | first lord | shares | source |
|---|---|---|---|
| **Mudda** (Vimshottari-Mudda) | `(completed years + birth nakshatra − 2) mod 9`: 1 Sun, 2 Moon, 3 Mars, 4 Rahu, 5 Jupiter, 6 Saturn, 7 Mercury, 8 Ketu, 0 Venus | Vimshottari's years × 3 days: 360 in all | Charak pp. 40–43; the Nilakanthi's appendix v. 1 gives the same rule |
| **Varsha Yogini** | `(birth nakshatra + completed years + 3) mod 8`: 1 Mangala … 0 Sankata | the natal Yogini's years × 10 days: 360 in all | Charak pp. 44–47 |
| **Patyayini** | the smallest *krishamsha* | the gaps between successive krishamshas, over the largest | Charak pp. 47–49; the Nilakanthi, Samjna-tantra, Patyayini vv. 1–5 |

**The two nakshatra systems are one rule.** Charak's formula for each is
the natal system's own seat for the birth nakshatra, advanced by one lord
for each completed year:
- Vimshottari seats Ashwini at Ketu, and `(0 + 1 − 2) mod 9 = 8` is Ketu.
- Yogini seats Ashwini at Bhramari, the fourth, and `(1 + 3) mod 8 = 4`
  is Bhramari.

So the SDK builds neither formula: **the year's first lord is
`(natal seat + completed years) mod lords`** over the natal row the build
already ships. The Mudda is `VIMSHOTTARI` and the Varsha Yogini is
`YOGINI`, each read as shares of one year. A consumer's own nakshatra
system then has an annual reading for free, and the two formulas become
tests (every nakshatra × 0–119 years, against the printed arithmetic).

**The Patyayini** takes the *krishamsha* of each of the seven and of the
lagna in the annual chart: the longitude with its whole signs dropped. It
orders the eight by it, ascending. Each one's *patyamsha* is its
krishamsha less the one before it, and the first one's is its own. The
patyamshas sum to the largest krishamsha, which stands for the whole
year. So a lord's share is its patyamsha over the largest krishamsha. The
lagna is a dasha lord in its own right: the Nilakanthi reads a separate
*lagna-dasha-phala*. A period therefore names it by its **sign**, with
the sign's lord as its lord. That is the field a sign-based period
already carries, so nothing new crosses the boundary.

### The worked years, which are the acceptance values

Charak's native (Chart V, the forty-first year, from 20 August 1984) was
born with the Moon at Leo 17°08′, 3°48′ into Purva Phalguni, so 9°32′ of
it remained.

| system | first | balance | the first lord's tail | source's table |
|---|---|---|---|---|
| Mudda | Rahu (`40 + 11 − 2 = 49`, remainder 4) | 54 × 9°32′ / 13°20′ = **38.61** days | **15.39** days | V-2 |
| Varsha Yogini | Ulka, Saturn (`40 + 11 + 3 = 54`, remainder 6) | 60 × 9°32′ / 13°20′ = **42.9** days | **17.1** days | V-5 |

The prose on p. 42 says Rahu's tail is 13.39 days. The table beside it
says 15.39, and `54 − 38.61` is 15.39. The table is right and the prose
is a slip. Recorded, not fitted, as with the Venus misprint in Chart VII-1
([`varshesha.md`](varshesha.md)).

Charak's Patyayini (Tables V-6 to V-8), with the year taken as 365 days:

| lord | krishamsha | patyamsha | days |
|---|---|---|---|
| Sun | 3°50′ | 3°50′ | 64.33 |
| Mars | 7°42′ | 3°52′ | 64.89 |
| lagna | 9°26′ | 1°44′ | 29.09 |
| Jupiter | 9°38′ | 0°12′ | 3.36 |
| Moon | 9°40′ | 0°02′ | 0.56 |
| Saturn | 17°13′ | 7°33′ | 126.70 |
| Mercury | 18°20′ | 1°07′ | 18.74 |
| Venus | 21°45′ | 3°25′ | 57.34 |

Every figure reproduces to the two decimals printed. So do the Sun
mahadasha's antardashas in Table V-9: the Sun's own is
`64.33 × 64.33 / 365` = 11 d 8.11 h. The Nilakanthi's own example
(Jupiter 2°49′08″ smallest, Venus 28°19′17″ largest, a 360-day year)
gives Jupiter 35 d 49 gh 52 pa, and that is a second acceptance value from
a second book.

## One kernel: a year cut into shares

All three are the same object:
- an ordered ring of lords, each with a **share** of the year;
- a first lord;
- the part of the first lord's share that is **still to run** when the
  year opens.

The year opens with that remainder. The rest of the ring follows in
order. The part of the first lord's share that was *already run* closes
the year, so the first lord appears twice: Charak's tables end on
"10. Rahu 0 15.39". Nothing else in the build has that shape:
- a natal dasha ends its cycle after the last lord and never returns to
  the first;
- a natal dasha's periods are years of fixed length, where these are
  shares of an interval whose clock is itself a choice (C122).

So it is a fourth [`Timeline`] beside the nakshatra, sign and Kalachakra
kernels. It lives in `teistro-dasha` as `YearDasha`, so the tree's
cursor, `at`, `periods` and the reading rows all come with it unchanged.

- **Mahadashas** are the ring from the first lord round to it again
  when the year opens on a balance: 10 for the Mudda and 9 for the
  Yogini, the first lord split. Where there is no balance (the Patyayini,
  and a nakshatra year under the `whole` reading), the ring runs once:
  8 and 9. A remainder of exactly 1 or 0 leaves one of the two pieces
  empty. It keeps its place as an empty interval rather than being
  dropped, so a path means the same thing in every year, and a list of
  periods leaves it out because it runs at no instant.
- **Antardashas** begin with the parent's own lord and follow the ring,
  each taking the parent's length in proportion to its share. Charak says
  this of the Mudda ("the first sub-period … belongs to the same planet")
  and the Nilakanthi's v. 5 says it of the Patyayini
  ("ādāv antardaśā pākapateḥ"). Charak's Table V-9 is the Sun's, which
  is first in both orders, so it cannot tell the two readings apart. The
  verse can.
- **The first lord's two pieces** are divided as the natal birth period
  is. The `birth_period` setting's *compressed* default divides each
  piece among all the sub-lords. *elapsed* sizes the head against the
  whole share ending where it ends, and the tail against the whole share
  starting where it starts. Neither source addresses it. The knob is the
  natal one, so a consumer changes both in one place.

**Every boundary is computed in shares first and turned into an instant
last.** A period's share of the year comes from its path, walking down
from the root: at most six levels of one multiplication each. Only then
does the clock map it to an instant. Children therefore partition their
parent exactly in shares whatever the clock, which is the tree's
invariant ("a boundary is an exact share of its parent") carried over.

## C122: what a day of the year is

The sources define the unit and leave the calendar open:
- Charak, p. 44: 360 days "represent 360 'solar days', each of which
  would indicate the movement of the Sun by one degree". One "could
  accurately spread the 360 days … over the 365 days of the year by
  proportionately increasing the dasha periods", although it is usually
  "more convenient to round".
- His Patyayini counts a 365-day year and says 360 would do "without much
  appreciable error".
- The Nilakanthi's commentary on vv. 1–3 offers the solar year of 360 or
  the civil year of 365 d 15 gh 30 pa. Its worked example uses 360. It
  then dates each period by **adding its days to the Sun's degree** and
  reading the calendar off the Sun.

So three readings, one knob, `clock`:

| clock | a unit of the year is | needs | whose |
|---|---|---|---|
| `sun_degrees` (default) | the Sun's motion through 1/360 of its circle from where it stood at the return | the Sun over the year | Charak's definition; the Nilakanthi's worked dating |
| `even` | an equal 1/360 of the time from this return to the next | the next return | Charak's "spread proportionately" |
| `days(n)` | `n / 360` civil days from the return | nothing | Charak's printed durations (`days(360)` Mudda and Yogini, `days(365)` Patyayini) |

**The default is the Sun's degree** because both books define the day
that way, and it alone ends the year *on* the next return by
construction. `even` ends there too. Between the returns it drifts from the Sun by the
equation of centre at the boundary less its value at the return. That
can reach twice the equation's 1.92°, so about **3.9 days** when a year
opens near one extreme. The thirtieth year of the SDK's own test birth
parts by 3.86 days, and the pass measures it over every recorded year.
`days` ends wherever its count says, which is a statement about the
calendar and not about the sky. It is the one reading that needs no
ephemeris, so a context with none can still ask for it.

**The clock is built once per year, not once per boundary.** Its knots
are the instants the Sun crosses every whole degree from its return
longitude, 361 of them, and a unit between two degrees is placed linearly
between their crossings. Within one degree (about a day) the Sun's speed
changes by less than 0.02%, so that interpolation is out by under a
second. A tree six levels deep then costs no more ephemeris than the
mahadashas do.

**The knots are found in two batched passes over the Sun, not by a search
for each** (`sun_knots`). The first samples the year a day apart and fits
the quintic through the six samples around each crossing. It fits the
longitudes alone, because a provider's speed need not be the derivative
of its longitude: the built-in's differs from it by up to 0.3″ a day. The
second corrects every knot against the Sun by Newton's method, all the
knots still moving read in one request a round, until each correction is
inside the search's tolerance. The fit sets how soon the knots are found
and never where, which matters topocentrically. There the parallax moves
the Sun up to 8.8″ and back each day, and samples a day apart all see it
at one hour, so the fit alone was 205 s out at a year's close. Against a
search for each crossing, in both zodiacs, geocentric and topocentric,
the knots stand within **0.04 ms** (a unit test holds them to a tenth of
the tolerance). The first build searched for each of the 360 crossings;
this reads the Sun about a quarter as often and in four requests rather
than about 4 000. An `even` clock has one knot, so it skips the fit and is
corrected from the mean motion.

**What Charak's printed dates can settle: nothing to the day.** His
Mudda table writes each duration in months and days and dates the ends by
calendar months. So its ends move by one to two days against every clock
above, in both directions, and do not match any clock throughout. The
**durations** are printed exactly (38.61, 42.9, 64.33) and are
clock-free. They are what the tests assert, and the clock is tested
against its own definition.

## C123: where the Mudda's balance comes from

| balance | reads | whose |
|---|---|---|
| `natal_moon` (default) | how much of the birth Moon's nakshatra was still to run, by arc; the same every year | Charak's worked example |
| `entry_moon` | how far the Moon at the **return** is through *its* nakshatra, by time (*bhayāta* over *bhabhoga*) | the Nilakanthi's appendix, citing the *Hayanaratna*: "vārṣakālika nakṣatra" |
| `whole` | nothing: the first lord runs its whole share from the return | the appendix's "old astrologers", whom it corrects |

The appendix names the older practice to reject it: "prācīna jyautiṣī
bhukta bhogya nahīṃ nikālte … yah ṭhīk nahīṃ". It is still a practice,
and a consumer reproducing an older almanac needs it, so it is a reading
and not the default. `natal_moon` measures by arc, as Charak does, unless
the dasha settings' `balance` says temporal. Then it reads the birth
Moon's stay, as the natal Vimshottari already does. `entry_moon` measures
by time because its source counts ghatis. The Varsha Yogini takes the
same knob: Charak's "as in the case of the Mudda dasha".

**Uttara Kalamrita's Mudda is not built.** Charak reports Kalidasa's
view: the balance *and the first lord* come from the Moon at the return,
over a table of nine durations summing to 365 that includes the lagna.
The table is not legible in the OCR (the figures sum to 371), and Charak
calls the method "not in popular use" and leaves Ketu's place in it "to
be tested". Its row stays unbuilt, and the crux names what would build
it: the table read off the page image.

## The Patyayini's ties

The Nilakanthi's v. 4 settles what Charak leaves silent:
- **Two lords at one krishamsha:** the stronger runs first. Where both
  are equally strong, the **slower** runs first. The commentary gives the
  reason: at the instant after they meet, the slower is behind.
- **A graha at the lagna's krishamsha:** weigh the graha against the
  **lagna's lord**, and the stronger runs first. Where they are equal,
  the slower runs first.

Strength is the Panchavargiya's Vishwa bala, the book's strength, which
the build already computes exactly. The second of a tied pair has a
patyamsha of zero and runs for no time. It is kept as an empty period,
because the verse gives it a place in the order.

Krishamshas are compared exactly, in nanoarcseconds (`Nas`), so "equal"
means equal and not "within a tolerance someone chose". A measured count
of how often the tie rule fires is on the pass's page. A rule that never
fires over the corpus is still built, because the verse states it and a
consumer's chart may be the one where it fires.

## What crosses, and how it is asked

```rust
sdk.chart().annual_dasha(&natal, &annual, completed_years, DashaSystem::Mudda, rules)
sdk.chart().annual_dashas(&natal, &annual, completed_years, &ANNUAL_DASHAS, rules)
```

The second is the first for several systems of one year. It reads
the Sun's year **once** for all of them, and it answers each exactly as
the single call would, which a test holds.

- The call takes the birth chart and an annual chart **you founded**, as
  every other Tajika call does, because the annual chart's lagna is a
  place's and the SDK does not choose the place.
- `rules: AnnualDashaRules { clock, balance, measure, birthPeriod, depth }`,
  each defaulted and each named on the answer. `measure` is by arc or by
  time, and when it is left unset each balance takes its source's own.
- `DashaSystem::Mudda`, `VarshaYogini` and `Patyayini` are accepted, and
  anything else is refused by name, with the three in the hint.
- The natal `sdk.chart().dasha(…)`, asked for one of the three, now hints
  at `annual_dasha` rather than listing the natal systems only. That
  refusal was a dead end before this page.

The answer is an `AnnualDasha`:
- the system, the completed years and the rules;
- the seed nakshatra, for the two nakshatra systems;
- the ring: its lords and their weights, the place it opens at, and the
  remainder that opened the year;
- the year as an interval;
- the periods as the natal reading's own `PeriodRow`s, to the depth
  asked for.

### Crossing the boundary

A binding asks for the annual dashas beside everything else a year's
chart answers, in the same `varsha_json`:

```json
{ "through": 40, "place": "birth",
  "dashas": ["dasha_system.MUDDA", "PATYAYINI"],
  "dashaRules": { "clock": { "days": 360 }, "depth": 1 } }
```

- `dashas` is `"all"` (the three in the catalogue's order) or systems in
  the caller's order, through the same `Asked<T>` reader as the matters
  and the sahams. A system is named by its catalogue key, **full or
  bare**. The full key is what every binding reads a system back as
  (`DashaSystem.Mudda` in Node is `'dasha_system.MUDDA'`), so a caller can
  hand back what it was given; the bare one is what Rust and the natal
  settings spell. Another system is refused as not an annual dasha, a
  misspelt one with the catalogue's "did you mean", and one named twice
  as a mistake.
- `dashaRules` is `AnnualDashaRules`, in the boundary's casing. A clock
  of no length is refused at `varsha_json.dashaRules.clock` by the same
  `YearClock::check` the kernel runs, so the request and the call cannot
  disagree about what a year is.
- **It needs `place`**, as the matters do: a year's dasha opens at its
  own chart, and the Patyayini is read from it.
- Each year asks `annual_dashas` once for every system, so the Sun is
  read over the year once however many divide it.

Three ragged sections carry the answer, beside the year's others:
`annual_charts.dasha_count` says how many rows of `year_dashas` are each
year's; each of those carries its system, seed, the place its ring opens
at, what remained of the first lord's share (NaN for none), the year, and
how many rows of `year_dasha_shares` (its ring) and `year_dasha_periods`
are its. The periods share the natal `dasha_periods` layout and one
decoder in every binding, with one column more: the Patyayini runs **one
sign among seven planets**, so a year's period says row by row whether it
is a sign's, where a birth dasha is all signs' or none and says it once.

Node, Python and Dart answer an `AnnualDasha` with the natal `Dasha`'s
periods and `at(jd)`, its ring as shares, and `firstLord`. The four
parity runners print every dasha of 24 years under the sources' readings
and under a rival clock, balance and birth period three levels deep, and
agree value for value.

## Not built, and why

- **Varsha Narayana:** neither book read gives it, so its row stays
  where [`dasha-coverage-measured.md`](dasha-coverage-measured.md) puts
  it, with its reason changed from "blocked on the solar return" (now
  built) to the text.
- **Kalidasa's Mudda:** above.
- **Patyayini on the Harsha or Dwadashavargiya strengths:** v. 4 says
  "bali", and the Panchavargiya is the strength every other rule in the
  book is judged by. A consumer who reads it otherwise supplies a
  different tie order, which is a knob if anyone asks and not a guess
  made now.

## Order of work

Steps 1 to 6 are built (2026-09-24). What the first five changed:
- The `even` clock's gap from the Sun's is up to **3.90 days**, not two:
  the equation of centre is counted twice.
- The batch call exists because three systems of one year would
  otherwise read the same Sun three times.
- `sun_knots` serves both Sun-reading clocks, with 360 divisions and
  with one.
- A search for each of the 360 crossings made the measured pass 33
  minutes on CI. The knots are now fitted and then corrected in batches,
  which the pass counts on every core; the pass takes about a minute.
  `teistro-astro` is also optimised in the dev profile, which every gate
  runs under. Rust does not reorder float arithmetic at any level, and
  every generated page reproduced byte for byte under the change.

1. `teistro-dasha`: `YearDasha`, with its ring, its clock and its
   `Timeline`. Tests cover the split first lord, a remainder of 0 or 1,
   a share of no weight, the elapsed division and a nonlinear clock.
2. `teistro-tajika`: `nakshatra_ring` and `patyayini_ring`, with the tie
   rule. Acceptance: Charak's Tables V-2, V-5 and V-8/V-9, and the
   Nilakanthi's Jupiter. The two printed formulas are checked against
   the seat rule for every nakshatra and 0–199 years.
3. The Sun-degree clock through its knots, fitted and then corrected
   against the Sun, and `even` through the next return.
4. `sdk.chart().annual_dasha` and `annual_dashas`, the natal refusal's
   new hint, and the coverage page's three rows moved from excused to
   computed.
5. The measured pass over the corpus's 2 159 recorded years
   (`muntha-measured.md` §19).
6. The boundary, the three bindings and parity (2026-09-24), above.
   What the building changed: the page had said the answer crosses as
   JSON, and it crosses as columns like the rest of the year; a system
   is accepted by its full key as well as its bare one, because the
   bindings read it back full and a caller would otherwise be refused
   the value it was given; and Python had never exported `Nakshatra`,
   which a `Dasha`'s seed already returned, so a strict caller could not
   name its own answer's type.
