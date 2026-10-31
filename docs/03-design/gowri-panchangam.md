# Gowri panchangam

Status: `built`, 2026-10-31. Written before the code, for the
application's SR-22 (addendum 3): the Gowri panchangam (*gowri nalla
neram*) day divisions, from a cited table.

## 1. What an almanac prints

A Tamil almanac divides the daylight into eight equal parts and the
night into eight, and names each part one of eight Gowri: Uthi
(*udyogam*), Amirtham, Rogam, Laabam, Dhanam, Sugam, Soram and Visham.
Five are marked good (*nalla neram*): Uthi, Amirtham, Laabam, Dhanam
and Sugam. The table runs a row a vara, by day and by night.

## 2. The rule that generates the table

No rank-1 or rank-2 text read gives the Gowri (the Kalaprakasika,
Muhurta Chintamani and Muhurta Martanda were searched; crux C1052).
The tables in print, and the web almanacs that reprint them, are one
rule, which the code encodes rather than transcribing 112 cells:

1. Each Gowri is a graha's: the Sun Uthi, the Moon Amirtham, Mars Rogam,
   Mercury Laabam, Jupiter Dhanam, Venus Sugam, Saturn Soram, Rahu
   Visham.
2. By day the eighths walk the weekday lords in weekday order from the
   vara's own lord, and Rahu takes the rahu kaala eighth (Sunday to
   Saturday the 8th, 2nd, 7th, 5th, 6th, 4th and 3rd, the SDK's own
   `RAHU_KAALA` row).
3. By night the walk starts from the lord of the vara four weekdays on,
   as the night choghadiya does, and Rahu again follows the graha it
   followed by day, which puts him in the night's 4th, 5th, 3rd, 8th,
   2nd, 7th and 6th eighths, Sunday to Saturday.

The rule was checked cell by cell against Drik Panchang's Gowri
pages (1, 3 and 5 November 2026: a Sunday, a Tuesday and a
Thursday, day and night) and ePanchang's (Saturday 31 October 2026):
every eighth agrees. Those are rank-3 sources reprinting the almanacs;
a Tamil almanac in print is the citation still wanted (C1052).

The North Indian *Gauri* table some almanacs print (Amritha, Siddha,
Marana, Rogam, Laabha, Dhanam, Sugam, Sokam) names other parts in
another order; it is not shipped and waits in the crux.

## 3. The shape

A catalogue kind `gowri` (80) with `lord` and `auspicious`, and a day's
`gowri`: sixteen parts, eight of the daylight then eight of the night,
each `{gowri, at, is_daytime}`, the shape a choghadiya part has. A half
with no arc contributes none, as the choghadiya's do on a polar day or
night.

Asked, like the markers (`day-markers.md` §3.2), so a day not asked is
unchanged: `AlmanacRequest::with_gowri`, `"gowri": true` in the JSON
days request, `TS_PANCHANGA_GOWRI` (256) at the C boundary, which
writes the section `gowri`.

## 4. Tests

- `period` unit tests: each vara's day and night rows against the
  printed rows the rule was checked on; Visham in the rahu kaala eighth
  every day; a polar half contributes nothing.
- Through the façade: a week at Chennai, each part an eighth of its
  arc, Visham by day exactly rahu kaala; a day not asked unchanged.
- Every binding's option and getter through each parity runner.
