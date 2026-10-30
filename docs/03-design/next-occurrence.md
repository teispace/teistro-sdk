# The next occurrence

Status: `built`, 2026-10-30. Design for the almanac app's SR-06
(`consumers/addendum-3.md`), written before the code and corrected by
it. The cost in §5 was measured through the built call
(`crates/sdk/tests/next.rs`).

An app's home screen shows "the next Ekadashi", "the next Purnima" or
"Dashain in 47 days" without asking a year. The festival reckoning
(`festival-rules.md` §4) already answers a range. This call finds the
first day in the ranges after an instant, so the app does not have to
choose a range or search one.

## 1. What is asked and answered

| part | what it is |
|---|---|
| `after` | an instant, as a UTC Julian day. The search starts on the civil day that holds it, on the request's clock, and **that day counts**. An app that wants the day after today passes the next midnight. |
| place and clock | `latitudeDeg`, `longitudeDeg`, `altitudeM` and `utcOffsetSeconds`, as every almanac request takes them |
| `calendar` | the calendar the answer's days are written in, Gregorian by default |
| `kind` or `rule` | one of the two. A **kind** is `EKADASHI`, `PURNIMA`, `AMAVASYA` or `SANKRANTI`, read by the rules in §2. A **rule** is a key of `festivals`, a festival request: a shipped pack's festival (a catalogue festival), a rule the caller writes, an anniversary, or a day a published set adds. |
| `limitDays` | how far ahead to look, the first day included: 400 by default, 1 to 3,660 (§3) |

The answer is `{day, keys, festivals, from, limitDays, searchedDays}`.
It gives the day and the keys of the rules that fall on it. For a kind,
these are each of its rules that does, for example the Smarta and the
Vaishnava Ekadashi when they agree. It also gives the festival
reckoning's entries for those keys on that day, as the festival answer
holds them, published days included. Last come the first day searched,
the limit and the days reckoned. The answer is sealed with the festival
reckoning's provenance. Its input hash is taken over the request, not
over the window that held the day. A convention `festival.next` says how
far the search went.

## 2. The kinds

Each kind is read by rules the SDK states, so no pack has to be named:

| kind | rules | source |
|---|---|---|
| `EKADASHI` | *Dharmasindhu*'s Smarta and Vaishnava fasts (`EkadashiRule::dharmasindhu`). The earlier of the two when they part. | *Dharmasindhu* pp. 11–12 (`festival-rules.md` §8) |
| `PURNIMA` | the day whose sunrise holds the full moon, or the day an unrisen full moon falls in, every month, the adhika month too | the day a patro printed in Nepal names by its sunrise tithi (C197), C804 |
| `AMAVASYA` | the same, for the new moon | as `PURNIMA` |
| `SANKRANTI` | the first day of each Bikram Sambat month (`DatedRule::sankrantis`, the twelve that `NEPAL_PANCHANGA` prints) | Nepal's national panchanga, VS 2082, the punya convention (C619) |

A reading other than these is asked as a rule. The full-moon fast that
Nepal's committee prints on the day whose sunset holds the full moon is
`PURNIMA_VRATA` in `NEPAL_PANCHANGA`. India's regional sankranti days
are dated rules on a regional solar calendar's first day.

## 3. The look-ahead limit

A rule kept in one lunar month recurs within thirteen lunations (383.9
days, 13 × 29.5306), and its day may move by a day either side. A solar
date recurs within 366 days. So **400 days** finds every yearly
festival from any instant. A rule kept only in an adhika month waits for
the next one: adhika months fall 28 to 35 lunations apart, at most about
1,034 days. **3,660 days**, ten years, is the most a request may ask,
past that gap. Nothing within the limit is `LIMIT`, naming `limitDays`,
with the most as its hint. A limit of 0 or past the most is
`OUT_OF_RANGE`. The limit is the request's to change, so a rarer rule is
never a dead end.

## 4. The search

The reckoning runs over **windows**: 32 days, then each twice the last,
none longer than a range's 366 days (`MOST_DAYS`), until the limit. For
400 days the windows are 32, 64, 128 and 176 days. The first window
holding the target answers, and its earliest day is the one returned.

- **The windows reuse the reckoning.** Each one is
  `AlmanacArea::festivals` over that range: its edges are widened by two
  days, by what a rule counts, and by a lunation for a rule kept only in
  the adhika month (`festival-rules.md` §4, §7.1). So a day found in a window
  is the day that range's reckoning would give. A test reads each kind,
  and a festival 293 days away, back against one reckoning over the
  whole range (`next.rs`).
- **A rule is searched alone.** A rule target cuts the request to what
  its days need (`FestivalRequest::narrowed_to`): the rule itself, the
  karmakala rule it counts from, its saits, and each published set's
  entries for them. A pack's other rules cost nothing. Moonrise, for
  example, is found only when the rule reads one.
- **Doubling bounds the waste.** A day found in the first window costs
  32 days of reckoning. A day 293 days away costs four windows and 400
  days, about one year's reckoning of one rule.

## 5. Cost

Measured on this Mac (Apple M-series, release build) through the
façade on the committee's setup, the Surya Siddhanta at Kathmandu:

| request | median |
|---|---:|
| the next Ekadashi (one window, 32 days) | see the report |
| Vijaya Dashami from 1 January (four windows, 400 days) | see the report |

`app-speed-measured.md` gates a row only when every device record
carries it: Android, the iOS simulator and the desktop. A budget row
added here would fail the gate until the two devices are recorded again,
which this change cannot do. So the row waits for the next device
recording. The bound it would hold is the `festivals` row's: a year of
the whole Nepal pack, 160 ms. A next occurrence reckons at most 400
days of one rule, which is less work.

## 6. At the boundary and in the bindings

`AlmanacArea::next` and `next_in_full` in the façade; `ts_almanac_next`
at the boundary, a JSON request and a canonical `{value, provenance}`
answer; `almanac.next` in every binding; and the MCP tool
`almanac.next`, whose input schema is the request's.
