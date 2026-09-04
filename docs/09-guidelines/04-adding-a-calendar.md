# Adding a calendar

Status: `draft`, 2026-09-04; worked example after Phase 1.

1. Decide the kind: arithmetic (table or formula), astronomical (needs the
   ephemeris port), or hybrid (official table plus computed extension, as
   Bikram Sambat).
2. Implement the `Calendar` trait: id, capabilities (range, whether it
   needs a place, whether it needs the ephemeris, eras), `from_fixed`,
   `to_fixed`, month lengths, leap rules. For a table calendar, supply the
   table in the data format and use the generic table implementation.
3. Declare month and weekday names, era names and date patterns in the
   locale packs (`calendar:<id>` namespace) for every shipped locale.
4. Tests: round trip every date in range; known dates from published
   almanacs; boundary behaviour at the ends of the range; for lunisolar
   calendars, agreement with the panchanga's lunar month.
5. Register in the calendar registry (SDK-shipped) or at context creation
   (consumer-supplied); document the `source` flag semantics.
6. Add the calendar to the conversion conformance set and to the docs
   calendar table.
