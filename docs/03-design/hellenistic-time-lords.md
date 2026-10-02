# Valens's time lords (the `hellenistic` module, step 7)

Status: `building`, 2026-10-02 — written from Valens's text before the
kernels, and corrected by building them: the first level is one cycle, so
the loosing lives below it.

Valens gives two ways of saying which sign rules a stretch of a life.
**Releasing** is the "vital sector" begun from a lot, each sign holding the
years of its lord, divided into months, days and hours. **Profection** is
the year counted on a sign from a point, one sign per year. ADR-0025 put
both in the same time-lord registry as the dashas, and this page does that
literally. Each is a kernel behind the dasha crate's `Timeline`, named by a
`DashaSystem` member, so a chart request asks for them the way it asks for
Vimshottari. They then cross the boundary in the dasha sections every
binding already reads.

## What the sources decide

Riley's translation of the *Anthologies*, Book IV, is the source; Kroll
pages are given as Valens's text marks them.

**Releasing** (IV.4, IV.6–IV.10):

- **The start.** Begin "the vital sector with the Lot of Fortune" for the
  body, and with Daimon for "employment or rank" (IV.4, 160K). Those two
  points are the two systems.
- **The years.** A sign allots the minimum years of its lord: Saturn 30,
  Jupiter 12, Mars 15, Venus 8, Mercury 20, Sun 19, Moon 25 (IV.1). The
  exceptions are Capricorn, which allots 27, and Aquarius, which allots
  30 (IV.6).
- **The levels.** Each level is "one-twelfth" of the one above: years,
  then months, then days, then hours (IV.10). Aries allots 15 years,
  15 months, 37½ days and 3⅓ hours.
- **The order.** A level begins at its parent's own sign and runs forward
  in the order of the signs. The last period is cut at the parent's end.
  Valens's Aries example gives Capricorn "2 years 3 months" and Aquarius
  "the remaining 11 months to fill out the 15 years" (IV.4).
- **The loosing.** When the twelve signs are spent, 211 units or "17 years
  7 months" at the second level, "the remaining time" continues "using the
  signs in opposition": in Gemini's 20 years, after 17 years 7 months,
  the rest goes to Sagittarius and then Capricorn (IV.4). IV.10 states the
  same for days (the cycle of 528) and hours (44). Valens rejects the
  rival transfer to the sign in trine.
- **The year.** A distribution year has 360 days. A life's elapsed days are
  counted in such years, not in calendar years (IV.9).
- **Fortune and Daimon in one sign.** At a new or full moon the two lots
  share a sign. Then the body is read from that sign and activity "from
  the sign immediately following" (IV.4).

**Profection** (IV.11):

- To find what a year transmits, "divide by 12" and count the remainder
  inclusively from a point. The 35th year leaves 11, and the 11th sign from
  Virgo is Cancer. The ordinal year *n* therefore falls *n − 1* signs on.
- Every point transmits. The Ascendant, the Sun and the Moon come first,
  and Fortune, Daimon, Love and Necessity are named too. Valens finds it
  "more scientific to count from the angles". An empty sign transmits to
  its ruler.
- The year here is the native's own, counted from the birthday in the
  Alexandrian calendar of 365¼ days (IV.9).

## The acceptance tests

Valens's worked nativities give signs and years, and that is the level
the tests hold.

1. **IV.8, Fortune in Leo.** At the first level, Leo has 19 years, Virgo
   20, Libra 8 and Scorpio 15, 62 years in all, so the 70th year falls in
   Sagittarius. At the second level, inside Sagittarius, Sagittarius has
   1 year, Capricorn 2 years 3 months, Aquarius 2 years 6 months and
   Pisces 1 year, followed by Aries.
2. **IV.8, Daimon in Scorpio.** At the first level, Scorpio has 15 years,
   Sagittarius 12 (to age 27) and Capricorn 27 (to age 54). At the second
   level, inside Aquarius, the periods run Aquarius 2 years 6 months,
   Pisces 1 year, Aries 1 year 3 months, Taurus 8 months, Gemini 1 year 8
   months, Cancer 2 years 1 month, Leo 1 year 7 months, Virgo 1 year 8
   months, Libra 8 months, Scorpio 1 year 3 months, then Sagittarius 1
   year, ending at "69 years 4 months".
3. **IV.10, Fortune in Pisces.**
   - Second level: Pisces 12 months, Aries 15, Taurus 8 and Gemini 20.
   - Third level, inside Gemini: Gemini 50 days, Cancer 62½, Leo 47½,
     Virgo 50, Libra 20, then Scorpio.
   - Fourth level, inside Scorpio: Scorpio 3 days 3 hours, Sagittarius
     2½ days, Capricorn 5 days 15 hours, Aquarius 6 days 6 hours, Pisces
     2½ days, Aries 3 days 3 hours, and Taurus the rest.
4. **IV.4, the loosing.** In Gemini's 20 years, the second level reaches
   Sagittarius after 17 years 7 months and gives Capricorn the remaining
   5 months.
5. **IV.11, the profected year.** With the Ascendant in Virgo, the 35th
   year is Cancer. The test pins the ordinal against the rival reading,
   which takes 35 as the native's age.

## The design

Two kernels join `Dasha`, `RashiDasha`, `KalachakraDasha` and `YearDasha`
behind `Timeline`. Each is a family of its own in `dasha_family.yaml`,
because the family names the kernel.

- **`ReleasingDasha`** holds a start sign, the birth and a year length.
  - At the first level a period lasts its sign's years.
  - At level *k*, the unit is the year divided by 12^(k−1). A child is
    its sign's years in that unit, starting at the parent's sign, cut at
    the parent's end, and loosed to the parent's opposite sign once twelve
    are spent.
  - `breadth` is the most children a period can have: Aquarius's 360
    months hold the twelve, then nine more after the loosing. The cursor
    stops at the parent's end, so the rows carry no empty periods.
  - The first level is one cycle of 211 years and then nothing. It is
    never loosed and never repeated, since no life reaches its end, and
    `AfterCycle` has nothing there to act on.
- **`ProfectionDasha`** holds a start sign, the birth and a year length:
  twelve one-year periods, the *n*th sign *n − 1* on, under `AfterCycle`'s
  repeat. It has one level, because Valens's months (IV.28, a quoted
  school's) are read from transits, not divided from the year.

New `DashaSystem` members, appended:

| key | id | kernel | start | year |
|---|---|---|---|---|
| `RELEASING_FORTUNE` | 40 | releasing | the sign of the Lot of Fortune | 360 days |
| `RELEASING_DAIMON` | 41 | releasing | the sign of Daimon, or the next sign when it shares Fortune's | 360 days |
| `PROFECTION` | 42 | profection | the Ascendant's sign | 365¼ days |

The lots are read through `ChartArea::lots`, under Valens's sect rule and
his Fortune (`LotRequest::VALENS`). A `lots` settings group that lets a
context change both comes with step 5; until then a consumer who wants
another rule reads the lot with `lots_with_request` and builds the kernel
from its sign. A document records the start sign it read. A
stored chart can then rebuild its periods however the context's settings
change, the way `RashiRules` does for the sign-based dashas.

The year length is the dasha group's existing `YearLength` knob, with a
per-system default: `SAVANA_360` for releasing and `JULIAN_365_25` for
profection. A consumer can change either, and so can read releasing in
calendar years, which some modern practice does.

Profection from points other than the Ascendant (the Sun, the Moon, a lot)
is one `ProfectionDasha` with another start. It ships as a function
taking a `LotPoint`. A catalogue member is added for a point only when a
consumer must name one in a request.

## What is not decided

- **C223, Daimon in Fortune's sign.** Valens moves the activity count to
  the next sign; IV.4 also reports "some astrologers" who do the same for
  a square of the luminaries, and rejects them. The next sign ships; the
  knob that keeps the shared sign comes with step 5.
- **C224, the order of the loosing at every level.** The text states it
  for the second level (IV.4) and for days and hours (IV.10). The first
  level never reaches it within a life. The second, third and fourth
  levels ship it.
- **The peaks.** Valens judges a releasing period by its place from
  Fortune ("MC relative to the Lot of Fortune", IV.7). That is a reading,
  not a period. It is reported as clauses later, not scored here.

## The order of work

1. **Done.** The two kernels in `crates/dasha` (`releasing.rs`), held to
   the five acceptance tests, and the three catalogue members.
2. **Done.** The SDK's dispatch: the lots and the Ascendant read off the
   chart, a document's start sign (`DashaReading::start_sign`), and
   `DashaCursor` rebuilding them. `crates/sdk/tests/time_lords.rs` reads
   them off a real chart, a new moon's included.
3. **Done.** [`time-lords-measured.md`](time-lords-measured.md): Daimon
   shares Fortune's sign on 3 of the corpus's 55 births, and the
   second-level loosing is not a question of the sky at all, since every
   start sign reaches it before 52 calendar years (Libra's, the latest).
4. The boundary comes with the dasha sections. The bindings' catalogues,
   the parity runners and the intl names of the three members follow.
   The names wait on a vetted Nepali and Hindi rendering, so the three
   are on `xtask/src/intl.rs`'s unnamed list.
5. The knobs: a `lots` settings group (sect and Fortune rules) that
   releasing reads, and C223's choice of the shared sign.
