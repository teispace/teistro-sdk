# Changelog

Every release answers one question first, because it is the only one an
astrology engine's consumer actually needs:

> **Does this move any number, and by how much?**

A chart computed with the previous version and stored somewhere is a fact
someone may still be looking at. So each entry begins with **Numbers**, and
"none" is an answer that has to be earned by the conformance run against
the previous release, not by nobody having looked.

## Unreleased

**Numbers:** the Bikram Sambat table's computed rows moved. Every year
outside the official span (BS 1970 to 2095) is now computed by the SDK's
own engine (the Surya Siddhanta as the text prints it, Nepal's clock,
Kathmandu, the punya-kala rule) and the table runs from 1700 to 2500 BS;
the earlier rows for 1856 to 1969 and 2096 to 2457 were the baseline
engine's projections and differ from these by a day at some month
boundaries, and 2096 to 2100 are computed, no longer marked official.
Inside the official span no date moved; eleven boundaries there now
report `Divergent`. Sunrise and sunset from modern positions compute for
the first time (the rise and set solver); Delta T's values are
unchanged by their move from `time` to `astro`. Nothing else computes
yet.

- Project founded: research, architecture, decisions, roadmap and the
  open-source scaffolding. See `docs/STATUS.md`.
- `crates/core`, `crates/calendar` (the arithmetic calendars and Bikram
  Sambat), `crates/siddhanta` (the Surya Siddhanta model), the seed of
  `crates/astro` (the boundary solver), and the Bikram Sambat engine with
  its measurement (`docs/calendars/bikram-sambat.md`).
- `crates/time` and `crates/port-timezone`: time scales with Delta T as
  the IERS table (1956 to the present) then Espenak and Meeus (2006)
  with Morrison and Stephenson's uncertainties, the IANA leap-second
  table, civil time, zone resolution over the embedded tzdb with the
  metadata a stored chart replays, local mean time, the sunrise-anchored
  day with the polar policies, ghati-pala. Every zone resolution of the
  55 fixture charts reproduces the baseline's instant and metadata.
- `crates/port-ephemeris` (spike 3's port promoted, with the rise and
  set override), `crates/astro` (Delta T moved here from `time`; the
  IAU routines ported from ERFA with a provenance table; sidereal time
  and the obliquity; frame completion over the port; the rise and set
  solver under the sunrise conventions, with polar days reported),
  `crates/ephemeris-kit` (the conformance kit: fifteen checks, both
  engines passing), the drik solar model for the calendars, the local
  day's convention and the `SUNRISE` unknown-time fallback in `time`,
  and the adapters under `adapters/`. Measured: the geometric sunrise
  agrees with Teimeris's own search within 0.13 s; the refracted one
  within 2.5 s of the baseline's fixtures below 60° of latitude (the
  refraction convention, cruxes C34). The committee's stated method for
  Bikram Sambat (the Surya Siddhanta) recorded in the memo, and modern
  positions measured at 65 % of the official months against the text's
  98.5 %.
