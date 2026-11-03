# Election search: the times a rule set leaves, ranked with reasons

Status: `built`, 2026-11-01. Design for the consumer addendum's A2-1 (its
first finder), written before the code. The record is `almanac.elect`
(`ts_almanac_elect`, every binding's `almanac.elect`, the MCP tool
`almanac.elect`).

An app planning an event asks one question: over these weeks, at this
place, which times does this rule set leave, best first, and why each one.
The muhurta engine (`03-design/muhurta.md`) already answers it inside
`almanac.days`, with the windows spread over each day's record. The
election search is the same search asked on its own: one request, one
ranked list, and each window's clauses split into the ones that count for
it and the ones that count against it. It computes nothing the muhurta
engine does not; it is `AlmanacArea::muhurta` and a reshaping of its answer.

## 1. The request

```json
{"first": {"year": 2026, "month": 11, "day": 20},
 "last": {"year": 2026, "month": 12, "day": 15},
 "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
 "utcOffsetSeconds": 20700,
 "muhurta": {"rules": "RAMAN_MARRIAGE", "most": 3}}
```

- `calendar`, `first`, `last`: the days, as `almanac.days` reads them
  (Gregorian when `calendar` is left out, `last` the first when left out).
- the place and the clock, as every almanac record reads them.
- `muhurta`: the rule set, exactly the `muhurta` record `almanac.days`
  reads (`rules` an activity or rules spelt out, `native`, `ranking`,
  `daysWithWindows`, `most`, `asta`, `direction`). Its reader is the
  muhurta crate's own, so a field added there is read here unchanged.

## 2. The answer

`{windows, closed, daysJudged, daysCut, windowsBlackedOut, ranking,
unjudged}`. Every field but `windows` is the muhurta answer's own.
Each window is

| field | what it holds |
|---|---|
| `rank` | 1 for the best, in the ranking the request chose |
| `at` | the window, UTC Julian days |
| `open` | whether no bar closes it |
| `holds` | the clauses that count **for** the time over it: a favourable yoga, a good tara, a strong lagna |
| `fails` | the clauses that count **against** it: an unfavourable limb, a dosha, a weak lagna |
| `barredBy` | the bars that close it, each with its rule |
| `score` | the ranking's score, where the ranking scores |

The split is the clause's own verdict (`Clause::favourable`), not a second
judgement: a clause the engine reports is a reason for or against, and the
answer only files it (C1160). A window closed by a bar is listed when the
request's `most` leaves room, so an app can say why a tempting time is not
offered.

## 3. Refusals

Named under `elect`: a key it does not read; a place or offset out of
range (`elect.latitudeDeg`); a day the calendar does not hold
(`elect.first`); a rule set the muhurta reader refuses (`elect.muhurta.*`,
its own message). A range the almanac or the muhurta search refuses is
refused as `almanac.days` refuses it. A context without an ephemeris is
`CAPABILITY`, as is a build without the muhurta family.

## 4. Cost

One muhurta search over the range, as `almanac.days` with only `muhurta`
asked; the reshaping is linear in the windows. Measured by criterion in
`crates/sdk/benches/finders.rs` (`election search, 26 days`).

## 5. Cruxes

C1160 (what a reason for or against is).
