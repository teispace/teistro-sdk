# Relocation and astrocartography

Status: `built`, 2026-11-01. Written for the application's SR-47
(addendum 3; feature universe ch. 15, "Relocation and mapping"): a
chart recast at other places, and each body's lines on the Earth, as
geometry. The map is the consumer's.

## 1. Relocation

A relocated chart has the same instant at another place. The grahas
stand where they stood, and only the angles and the houses move.
`ChartArea::relocated` (`ts_chart_relocated`, the record
`chart.relocated`) recasts one instant at one to 256 places. For each
place it answers the ascendant, the midheaven and the twelve cusps, in
the context's zodiac. The cusps are of the division asked, or else the
profile's Western override, or else Placidus, Leo's division (C1157).
Each chart is `chart_kind.RELOCATED`.

## 2. The lines

A body is on the **MC** where the local sidereal time equals its right
ascension, so its MC line is a meridian and its IC line the opposite
meridian. It is on the **ASC** or the **DSC** where its hour angle is
minus or plus the semi-diurnal arc `H₀`, with `cos H₀ = −tan φ tan δ`.
Past the latitude `90° − |δ|` there is no such arc: the body never rises
in one hemisphere and never sets in the other. At that latitude its
rising and setting lines meet on the lower or upper meridian. That
latitude is the line's polar limit. The line is closed there and stops,
and `polarLimitDeg` says where (C1156).

Each line is a polyline of `[longitudeDeg, latitudeDeg]` points from
south to north. The points fall at the request's step of latitude
(1° by default, at most 10°), out to `latitudeLimitDeg` (80° by default,
below 90°). Longitude is east, in `[-180, 180)`, so two consecutive
points more than 180° apart lie on either side of the antimeridian.

**Two projections are practised** (C1155):

- **In mundo**, the default, reads the body where it stands, with its
  ecliptic latitude, against the true horizon. The horizon is geometric,
  at altitude zero, with no refraction and the geocentric sky.
- **Zodiacal** reads the body's degree of the ecliptic, with its latitude
  zeroed. A point on its ASC line then has the body's longitude as the
  chart's own ascendant there. Inside the polar circles a chart's
  ascendant need not be the ecliptic degree on the eastern horizon, so a
  zodiacal rising or setting line stops at the polar circle.

The bodies and the sidereal time are read from a chart founded at the
instant at 0° N 0° E. The Greenwich meridian's right ascension is that
chart's midheaven turned to right ascension, so the lines rest on the
same angles the chart code casts.

## 3. Proof by geometry

`crates/sdk/tests/mundane.rs` reads back every point of every line of
the seven, in both projections, at a 10° step. It casts a chart at each
point through `relocated` and checks:

- **In mundo, MC and IC:** the body's right ascension is the meridian's,
  or the meridian's plus 180°.
- **In mundo, ASC and DSC:** the body's altitude, read from the chart's
  own midheaven (`altitude_by_midheaven_deg`), is zero, with the body
  east of the meridian on the ASC line and west of it on the DSC line.
- **Zodiacal:** the chart's midheaven, imum coeli, ascendant or
  descendant is the body's longitude.

Every point is on its angle to `LINE_TOLERANCE_DEG`, 1e-6°. The lines
are closed-form, so that margin is a few rounding steps, held wide.
