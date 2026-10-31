# A range's change points

Status: `built`, 2026-10-30. Design for the almanac app's SR-07
(`consumers/addendum-3.md`), written before the code and corrected by it.
The cost in §5 was measured through the built call.

An app that schedules a notification, draws a timeline or wakes a widget
needs one list: every instant in a range at which something on the page
changes, in order, each saying what changed. The days already hold all of it
as spans. Without this call an app walks four lists per day, drops the
repeats where a span crosses midnight of the day list, and merges by hand.

## 1. What is asked and answered

`almanac.changes` takes what a range of days takes (`first`, optional `last`,
`calendar`, `latitudeDeg`, `longitudeDeg`, `altitudeM`, and `utcOffsetSeconds`
or `zone`) and an optional `kinds`, the kinds wanted, all when left out.

The answer is `{points, window}`. `window` is the range as instants, from the
first day's opening to the last day's close. Each point is:

| field | what it is |
|---|---|
| `at` | the instant, a UTC Julian day |
| `kind` | `TITHI`, `NAKSHATRA`, `YOGA`, `KARANA`, `MOON_SIGN`, `SUN_SIGN` or `VARA` |
| `from` | the catalogue key of the member that ended |
| `to` | the catalogue key of the member that began |

`points` are sorted by `at`. Where two fall on one instant, as a tithi's end
and a karana's always do, they stand in the order of the table above, so the
list is one order on every machine.

The kinds and their keys are the catalogue's, so the words come from
`sdk.entity` in any locale, as a day's limbs do. The list is a list of
instants only: an app reads `at` on its own clock, or through
`time.civil`.

## 2. What counts as a change

- **Inside the range, each once.** A point is at or after `window.from` and
  before `window.to`. A span that crosses a day's edge is one span, so its end
  is one point. The first member's start and the last member's end, which lie
  outside the range, are not points.
- **A span's real ends, not the day's.** `at` comes from a span's `whole`, not
  `inside`: a tithi ending at 21:05 changes at 21:05, and the sunrise that
  opens the next day changes nothing (`span.rs`).
- **A skipped member.** A kshaya tithi lies inside a day as a span of its own,
  so it has two points like any other. A member the sky never gave has none.
- **`VARA` changes at sunrise**, since the day's vara holds from sunrise to
  sunrise (`LocalDay::vara`), not at civil midnight. The first day's opening is
  not a point: the range starts there.
- **`SUN_SIGN` is the sankranti.** The instant is the one a day's
  `sun.sankranti` gives, to the bit.
- **Not here.** Periods (Rahu kala, muhurtas, horas), which recur daily and
  which a day's answer lists, and rises and sets. Moonrise costs most of a
  day's requests (§4), and an app asking for "what changes" has not asked for
  it. A later kind extends the enum and the answer stays this shape.

## 3. Under a zone

`at` is an instant, so no point moves with the clock. The clock decides only
which civil days the range names. On a day whose clock changes, the range is
the same arc and the provenance's `time.clock` says which offset and source
applied (`time-and-timezone.md` §3.4).

## 4. Reuse, not search

The range walker (`Almanac::run`) already takes a function from a day to a
value, and `festival_days` uses it to skip what a festival does not read. This
is a third such function. It reads the sunrise day, the limbs, and the Moon's
and Sun's signs, all from the day's shared crossings (`Memo`), and asks for no
rise or set. It searches nothing a range of days does not.

## 5. The cost

Measured by the app-speed harness (`changes`: a year of every kind, cold memo)
on an Apple M2 Pro, median of the runs:

| Setup | `changes` | `year` | Share of `year` |
|---|---|---|---|
| `drik-teimeris` | 51.1 ms | 151.5 ms | 34% |
| `committee-teimeris` | 21.8 ms | 102.2 ms | 21% |
| `drik-builtin` | 111.6 ms | 327.6 ms | 34% |

`drik-teimeris`'s year of festivals, also founded on only what it reads, took
57.8 ms in the same run. A list of change points over a year costs about a third
of the full almanac, so a range of them does not need to be built from the days.

**No budget row yet.** A row on the performance page needs device records, and
this run is one desktop. The harness now times `changes`, so the next device
run records it.
