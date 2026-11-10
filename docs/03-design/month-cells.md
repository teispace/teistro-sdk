# A month of calendar cells

Status: `built`, 2026-11-10. Design for a consumer's SR-84
(`consumers/addendum-3.md`), written before the code and corrected by it. The
cost in §5 was measured through the built call.

A month grid prints, for each date, the tithi at its sunrise and its paksha,
whether a tithi is skipped or repeated, and, on request, the festivals. A
consumer drawing one asked for a full day per date: 30 full days took 17.6 ms
native, far more in wasm on an older phone, and almost all of it was the Moon's
rises and sets, the periods and the muhurtas, none of which a cell prints.

## 1. What is asked and answered

`almanac.cells` (`ts_almanac_cells`, `AlmanacArea::cells`, every binding's
`almanac.cells` and an agent server tool) takes what a range of days takes:
`first`, optional `last`, `calendar` (any shipped calendar: a Bikram Sambat
month as well as a Gregorian one), `latitudeDeg`, `longitudeDeg`, `altitudeM`,
and `utcOffsetSeconds` or `zone`; and an optional `festivals`, a festival
request. The range holds at most 366 dates (`MOST_DAYS`), a year view's worth;
more is refused `OUT_OF_RANGE` on `to`, before anything is computed.

The answer is `{cells, observances}`. `observances` is there only when
festivals were asked, and is what `almanac.festivals` answers over the range.
Each cell is:

| field | what it is |
|---|---|
| `date` | the civil date, in the calendar asked |
| `vara` | its weekday, sunrise to sunrise |
| `sunrise`, `nextSunrise` | UTC Julian days; `sunrise` is the full day's `day.sunrise`, to the bit |
| `normal` | whether the Sun rose and set; a polar day is read under the profile's `day.polarDayPolicy` |
| `tithi`, `paksha` | the tithi at the sunrise, the date's name, and its paksha |
| `month` | the lunar month at the sunrise under both conventions, with its `kind` (`NIJA`, `ADHIKA`, `KSHAYA`) |
| `ends` | every tithi ending after the sunrise and at or before the next, in order, each `{tithi, at, kshaya}` |
| `kind` | `PLAIN`, `KSHAYA`, `VRIDDHI` or `VRIDDHI_AND_KSHAYA` |
| `festivals` | the keys kept on the date, festivals, fasts, dated days, anniversaries and published days in that order; left out when none or not asked |

Every member is a catalogue key or a closed enum (`CellKind`), never free text,
so the words come from `sdk.entity` in any locale, as a day's limbs do. The cell
adds no message of its own.

## 2. Kshaya and vriddhi

A date is named by the tithi at its sunrise (`festival-rules.md`'s sunrise
rule), so:

- **Vriddhi.** The tithi at this sunrise held the sunrise before as well: it
  names two dates running, and this cell is the second. The first date of the
  range is judged against the day before it, which the range founds for that
  (§4).
- **Kshaya.** A tithi began after this sunrise and ended before the next: it
  names no date. It is listed in `ends` with `kshaya: true`, so the grid can
  print it beside the date it falls in.
- **Both.** Tithis next to one another differ in length by less than an hour,
  so a cell is both only when a tithi a little over a day long stands beside one
  a little under. It is possible, so it has a kind of its own rather than being
  folded into either; a kind that dropped it would say `PLAIN` or hide one of
  the two facts.
- **A polar day.** Under a nearest-event policy two civil dates can open on the
  same sunrise; that repeats nothing, so a vriddhi needs the sunrise before to
  be earlier. A polar day an undefined policy refuses is refused here as the
  full day refuses it.

"Ends at or before the next sunrise" is read on the spans' own half-open
bounds (`span.rs`): a tithi ending exactly at the next sunrise ends in this
cell, and the next cell's tithi is the one beginning there.

## 3. Under a zone

Each date is read on the zone's offset then, as a range on a zone is
(`time-and-timezone.md` §3.4). The festivals reckon on the first date's
offset, as `festivals` on a zone does. A request by zone and one by the offset
the zone kept answer the same cells and seal apart (C1252,
`crates/sdk/tests/zone_requests.rs`).

## 4. One code path

A cell is read off cell days (`Almanac::cell_days`, `cell::CellDay`): each
day's sunrise and next sunrise, vara, the lunar month and the tithi spans. A
cell day is founded by the steps a festival day is (`Almanac::festival_value`),
over the same memo and in the same order: the local day, the window, the frame,
the range's crossings, the tithi spans and the month, each the value
`Almanac::between` gives that day, to the bit. It leaves out what a cell does
not print. The range's crossings are found for the elongation alone
(`RangeCrossings` at `Reach::Elongation`), not for the nakshatras, the yogas
and the two bodies' signs too. The tithi spans are read by `limb::tithis_with`,
which is `limbs_with`'s tithi step and shares its edges (`tithi_edges`).

`Almanac::cell_days` founds the range and one day either side as one run
(`Almanac::run_over`) and stamps it as the range: the day before lends its
sunrise to the first cell's vriddhi, and the day after its tithis, so a tithi
ending before the last date's next sunrise is found whatever the profile's day
window. `cell::cells` then reads each cell from three days. When festivals are
asked, the range's festival days are founded as `almanac.festivals` founds
them and reckoned by the same `reckoning`, and each cell takes its date's keys
(`Observances::keys_on`).

A first version read the cells off the festival days themselves. It was one
step shorter and took twice as long (§5): most of a festival day's time went
to the crossings a cell never reads.

## 5. The cost

November 2026 at Kathmandu, 30 cells, median of 21 calls through
`AlmanacArea::cells` on one context, built-in ephemeris, release build on the
maintainer's Apple-silicon Mac:

| | per month |
|---|---|
| cells read off the festival days (first version) | 10.7 ms |
| cells read off cell days | **5.3 ms** (5.2 on a zone; 5.6 on a fresh context) |
| the same range's festival days with no rule (`festivals`) | 10.2 ms |
| the same range's full days (`almanac.of`) | 19.0 ms |

A sampled profile of the 5.3 ms: 47% finding the elongation's crossings, 40%
the sunrises and sunsets, 11% the lunar month. Under all three, 40% of the time
is the IAU 2000B nutation series, summed once at each instant the provider is
asked for. The SDK summed it twice: the obliquity step and the corrections step
each summed it at the same instant. The corrections step now takes the
nutation in longitude from the SDK's obliquity, the same bits, so every
request that asks for apparent positions is cheaper. The same month's full
days went from 24.8 ms to 19.0 ms.

The request asked for well under 2 ms a month. That is not met. What remains
is one sunrise, one sunset and the elongation's crossings for each day, and
each is a search the full day makes the same way, so meeting it would change
the engine's evaluation, not the cell. The cell's share is already gone. The
gate's instruction counts for `almanac.cells` come from `costs.json`'s next
pricing run (`requests.json` carries the example). No wasm timing is recorded:
the repository has no wasm speed harness.

## 6. Tested

`crates/sdk/tests/cells.rs`: every date of 2026 at Kathmandu on its zone, and
Baishakh 2083 in Bikram Sambat, against the full days of `in_zone`: the date,
sunrise, next sunrise, vara, month and sunrise tithi to the bit, the tithis
ending in the day, and the kind the full days give; Tromsø's polar winter under
the nearest-event policy the same way, and under an undefined policy refused
with the full day's status; a month's festivals equal to `almanac.festivals`',
keyed onto their dates; the request read strictly and held to a year. Each
binding's test and parity line (`almanac-cells`) reads November 2026.
