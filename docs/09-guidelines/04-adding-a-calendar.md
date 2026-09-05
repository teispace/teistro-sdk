# Adding a calendar

Status: `draft`, revised 2026-09-05 (the research standard added on
2026-09-04). Worked example: Bikram Sambat, whose source memo
(`docs/calendars/bikram-sambat.md`) records R1 to R5, whose engine and
measurement (`cargo xtask calendars bs-fit`) chose the month-start rule
against the official table, and whose shipped table is generated and
gated (`cargo xtask gen calendars`, `check-calendars`).

0. **Research first, and write the source memo** (`docs/calendars/<id>.md`,
   reviewed before code is written), meeting all five requirements:
   R1 identify the authority that defines the calendar in practice (a
   government committee, an almanac, a religious body, a mathematical
   definition); R2 obtain the authority's own publications, not a
   third-party website; R3 cross-validate against at least three
   independent real-world sources including one used by the target
   population, with independence verified (several consumer calendars
   share one underlying table and count once); R4 validate every day in
   the supported range, not a sample, because day-boundary defects
   cluster at month and year edges; R5 document the divergence envelope:
   who says what where sources disagree, and which the SDK follows by
   default and why.
1. Decide the kind: arithmetic (table or formula), astronomical (needs the
   ephemeris port), or hybrid (official table plus computed extension, as
   Bikram Sambat). A hybrid calendar reports its `resolution` (tabular,
   computed, divergent) on every date.
2. Implement the `Calendar` trait: id, capabilities (range, whether it
   needs a place, whether it needs the ephemeris, eras), `from_fixed`,
   `to_fixed`, month lengths, leap rules. For a table calendar, supply the
   table in the data format and use the generic table implementation.
3. Declare month and weekday names, era names and date patterns in the
   locale packs (`calendar:<id>` namespace) for every shipped locale.
4. Tests: round trip every day in range (exhaustive, never sampled);
   every month length against the authority's table; known dates from
   published almanacs, treaties and inscriptions; boundary behaviour at
   the ends of the range; every rare event in range as a fixture (each
   kshaya and adhika month, each kshaya and doubled day, leap days, the
   Gregorian transition, a sankranti within a minute of the day boundary,
   both sides of Nepal's 1986 zone change); the divergence set itself as a
   fixture; for lunisolar calendars, agreement with the panchanga's lunar
   month; the day-boundary knob exercised.
5. Register in the calendar registry (SDK-shipped) or at context creation
   (consumer-supplied); document the `source` flag semantics.
6. Add the calendar to the conversion conformance set and to the docs
   calendar table.
