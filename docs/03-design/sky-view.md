# The sky view

Status: `built`, 2026-11-02.

SR-58's sky view (`consumers/addendum-3.md`, P2 by addendum 5). What a
"look up" screen draws: at a place and an instant, where every graha and
every nakshatra's junction star (yogatara) stands in the local sky, and
the ecliptic and the horizon to lay them against. The device pose (P3) is
`almanac.skyPose` (`sky-pose.md`), laid on this record without changing
it (§7).

Built in `crates/astro/src/sky_view.rs`, reached as
`AlmanacArea::sky_view`, the record `almanac.skyView`, `ts_almanac_sky_view`
at the C boundary and `almanac.skyView` (`sky_view`) in every binding.

## 1. The answer

One `SkyView`, sealed in an envelope:

| Field | Meaning |
|---|---|
| `at` | the instant, UT1 read as UTC, as every other instant of a day is |
| `siderealTimeDeg` | the local apparent sidereal time |
| `obliquityDeg` | the true obliquity of the ecliptic |
| `atmosphere` | `{pressureHpa, temperatureC}` the refraction was reckoned under |
| `eclipticStepDeg`, `horizonStepDeg` | the steps asked |
| `grahas` | each graha asked, in the order asked |
| `stars` | each star asked, in the order asked |
| `ecliptic` | the ecliptic as a point list |
| `horizon` | the horizon as a point list |

A graha or a star is `raDeg`, `decDeg`, `altitudeDeg`,
`refractedAltitudeDeg` and `azimuthDeg`, with `graha`, or with `star`,
`nakshatra` (the nakshatra it is the junction star of, or null) and
`magnitude`. An ecliptic point is `kind` (`SAMPLE`, `ASCENDANT`,
`DESCENDANT`), `longitudeDeg`, `altitudeDeg`, `refractedAltitudeDeg` and
`azimuthDeg`. A horizon point is `azimuthDeg`, `raDeg`, `decDeg`,
`longitudeDeg` and `latitudeDeg`, so a chart drawn on the equator or the
ecliptic can draw the horizon too. At the boundary every catalogue
member is written as its full key (`graha.MOON`, `star.SPICA`,
`nakshatra.CHITRA`; `SkyView::MEMBERS`), as every binding reads members
back; the Rust value keeps the enums.

## 2. Frames

Every direction is **topocentric, apparent and of date**: the ephemeris's
apparent geocentric place (light time, aberration and deflection applied,
on the true equator and equinox of date) less the station's position on
the WGS 84 ellipsoid (`sky::observer`, SOFA `pvtob`, no polar motion),
turned by the local apparent sidereal time (IAU 2006 with the IAU 2000B
nutation, `gst06b`) into the hour angle, and then into altitude and
azimuth with the geodetic latitude (Meeus, *Astronomical Algorithms*, 2nd
ed., eq. 13.5 and 13.6, the azimuth turned to run from north through
east). It is the day's sky's frame (`day-sky.md` §2), and the same code.

| Part | Frame |
|---|---|
| Sun to Saturn, and the outer planets when asked | the ephemeris through the frame completion, less the station: the diurnal parallax is in, the Moon's up to a degree |
| Rahu and Ketu | the node the settings name (`frame.node`, mean or true), a direction on the ecliptic of date with no distance: nothing to take the station from, so no parallax; Ketu the opposite direction |
| The junction stars | the star table's apparent place (`astro-star-table.md`: proper motion, parallax, the Sun's deflection, the annual aberration, frame bias, IAU 2006 precession, IAU 2000B nutation); a star's diurnal parallax is under a microarcsecond and is left out, and the diurnal aberration (under 0.32″) is left out for stars and planets alike |
| The ecliptic | the true ecliptic of date, at latitude nought, by tropical longitude |
| The horizon | the airless geometric horizon, altitude nought |

**Refraction.** Each altitude is given twice: `altitudeDeg`, airless, and
`refractedAltitudeDeg`, lifted by Sæmundsson's refraction (1986, Meeus eq.
16.4) scaled to the air by `(P / 1010) · (283 / (273 + T))` (Meeus, ch.
16). The air is 1010 hPa and 10 °C, Sæmundsson's own, unless the request
names `pressureHpa` (0 to 1100) and `temperatureC` (−90 to 60); a pressure
of nought turns refraction off. Below the horizon nothing is refracted
(C973), as on the day's sky. The provenance names the air under the
convention `sky.refraction`.

**Sidereal.** Longitudes are tropical. A sidereal reading subtracts the
context's ayanamsha, which a chart answers; the sky view does not repeat it.

## 3. The junction stars

The default stars are the twenty-seven nakshatras' yogataras in the
nakshatras' order, as the star catalogue identifies them
(`stars::yogatara`, `astro-star-table.md` §3). The rank 1 text is the
*Surya Siddhanta*, ch. VIII: verses 1–9 give each asterism's junction
star by its polar longitude and latitude, and verses 16–19 say which
member of its group it is; the identifications with modern stars are Burgess's notes to the
chapter (Burgess 1860, reprinted Calcutta 1935, pp. 211–230, read in the
1935 reprint's pages). Each row below is the catalogue's member and the
page where Burgess argues it.

| Nakshatra | Star | Burgess |
|---|---|---|
| Ashwini | β Arietis | p. 212, the northern of the two (v. 16) |
| Bharani | 41 Arietis | pp. 212–213, the southern (v. 18) (C1180) |
| Krittika | η Tauri (Alcyone) | p. 213, by the position (C1186) |
| Rohini | α Tauri (Aldebaran) | p. 214, the most easterly (v. 19) |
| Mrigashira | λ Orionis | p. 214, the northern (v. 16) |
| Ardra | α Orionis (Betelgeuse) | p. 215 (C1186) |
| Punarvasu | β Geminorum (Pollux) | p. 216, the latter of the pair (v. 19) |
| Pushya | δ Cancri | p. 216, the middle one |
| Ashlesha | ε Hydrae | p. 217, the brightest of Hydra's head |
| Magha | α Leonis (Regulus) | pp. 217–218, the southern (v. 18) |
| Purva Phalguni | δ Leonis | p. 219, the northern (v. 18) |
| Uttara Phalguni | β Leonis (Denebola) | p. 219 (C1186) |
| Hasta | δ Corvi | pp. 219–220 (C1181) |
| Chitra | α Virginis (Spica) | p. 220 |
| Swati | α Boötis (Arcturus) | p. 220 |
| Vishakha | α² Librae | p. 221 (C1182) |
| Anuradha | δ Scorpii | p. 222, the middle one (v. 18) |
| Jyeshtha | α Scorpii (Antares) | pp. 222–223, the central one (v. 18) |
| Mula | λ Scorpii (Shaula) | p. 223, the eastern (v. 19) (C1186) |
| Purva Ashadha | δ Sagittarii | p. 224 |
| Uttara Ashadha | σ Sagittarii | p. 225, the northern (C1186) |
| Shravana | α Aquilae (Altair) | p. 226, the middle one (v. 18) |
| Dhanishtha | β Delphini | p. 227, the western (v. 17) |
| Shatabhisha | λ Aquarii | p. 227, the brightest (v. 19) |
| Purva Bhadrapada | α Pegasi (Markab) | p. 228, by the position (C1183) |
| Uttara Bhadrapada | γ Pegasi (Algenib) | p. 229 (C1183) |
| Revati | ζ Piscium | p. 230, the southernmost (v. 18) |

**Cruxes.** Burgess is unsure in five places, the default leaves one asterism out, and the catalogue decides
each; every one is a crux in `19-verification-cruxes.md`:

- **C1180, Bharani.** The text's southern star is 35 or 41 Arietis, which
  were nearly equally far from the equator in AD 560; Burgess takes 41 as
  the brighter and nearer the ecliptic, while the text's defined position
  "agrees better with 35". The catalogue holds 41.
- **C1181, Hasta.** Burgess reads v. 17's description as γ Corvi; the
  position "in which all the authorities agree" points to δ Corvi. The
  catalogue holds δ, the text's number.
- **C1182, Vishakha.** The text's position is the faint ι Librae
  (magnitude 4.8 in his catalogue), and Burgess believes the group was first α and β
  Librae. The catalogue holds α² Librae, and has no ι Librae.
- **C1183, the Bhadrapadas.** Purva Bhadrapada's position is α Pegasi
  while v. 16 calls its star the northern, which would exclude it;
  Uttara Bhadrapada's longitude is γ Pegasi's and its latitude α
  Andromedae's. The catalogue holds α and γ Pegasi.
- **C1184, Abhijit.** The 28-nakshatra scheme's Abhijit has its junction
  star (Vega, p. 225), and the nakshatra kind has twenty-seven members.
  The default leaves it out; `stars: ["VEGA"]` adds it.
- **C1186, Burgess over the text's letter.** For Krittika the text's
  southern member would be Atlas or Merope, and its position is Alcyone's;
  for Ardra the position is 135 Tauri's, a sixth-magnitude star, and
  Burgess keeps α Orionis; Uttara Phalguni's β Leonis is the southern of
  its pair, not the northern v. 18 names; Uttara Ashadha's latitude is in
  error, which led Colebrooke to τ Sagittarii; Mula's λ Scorpii is
  "probable". The catalogue follows Burgess in each.

A request's `stars` names any catalogue star instead (the 128 members),
so a reader who takes Alpheratz for Uttara Bhadrapada asks for it. A star
the catalogue does not carry, ι Librae or 35 Arietis, waits for the full
star catalogue (v1.x, `07-roadmap/00-roadmap.md`).

## 4. The ecliptic and the horizon

**The ecliptic** is a point every `eclipticStepDeg` of tropical longitude
from 0°, 5° by default, with the two points where it meets the horizon
added in their places: the `ASCENDANT`, the eastern crossing, and the
`DESCENDANT` opposite it. They are found as the directions perpendicular
to both the zenith and the ecliptic's pole, the ascendant the one on the
east's side, so they are exact and need no search; a test holds the
ascendant to the house module's to 1e-9°. At the one instant a latitude of
90° less the obliquity has the ecliptic lie in the horizon there is no
crossing, and the list has only its samples.

**The horizon** is a point every `horizonStepDeg` of azimuth from north,
5° by default, each with its right ascension, declination, ecliptic
longitude and latitude.

Both steps are 0.1° to 90°, a refusal naming `eclipticStepDeg` or
`horizonStepDeg` outside it. A drawing joins the points; it closes the
circle itself.

## 5. Grahas

The nine, Sun to Ketu, unless the request's `grahas` names others or
another order. Uranus, Neptune and Pluto are asked for by name.

**A classical astronomy is refused (C1185).** Under a profile whose
ephemeris is the *Surya Siddhanta* (`nepali-committee`, `nepal_official`'s
first member) the grahas' places are the text's reckoning, degrees from
where the planet is seen. The sky view answers `UNSUPPORTED` there,
saying so, rather than point a camera at a text's Mars. Such an app asks
it of a second context over a modern ephemeris.

## 6. Measured against JPL Horizons

`crates/sdk/tests/sky_view.rs` holds the record against JPL Horizons'
observer tables (DE441), recorded on 2026-11-02 into
`crates/sdk/tests/horizons-grahas.json`: the Sun, the Moon, Mercury, Venus,
Mars, Jupiter and Saturn at Kathmandu, Sydney, Tromsø and Quito, on
2026-10-31 and 2025-01-14, at four instants each, 224 rows, with
`QUANTITIES='2,4'` airless and `'4'` with `APPARENT=REFRACTED`. Over the
built-in ephemeris, worst of the rows:

| Quantity | Worst difference | Held under |
|---|---|---|
| Altitude, airless | 0.00031° (1.1″) | 0.001° |
| Azimuth | 0.00049° (1.8″) | 0.001° |
| Refracted altitude, above 2° | 0.00035° (1.2″) | 0.001° |
| Right ascension, topocentric | 0.00016° (0.6″) | 0.001° |
| Declination, topocentric | 0.00010° (0.4″) | 0.001° |

The test prints the worst under `--no-capture`. What is left is the
built-in theories' own error against DE441 and the UT1-for-UTC reading,
which |UT1 − UTC| < 0.9 s bounds at 0.004° of hour angle.

The stars have no Horizons table; their places are the star table's,
measured on its own page, and the sky view adds the hour angle and the
horizontal turn the grahas' rows hold.

## 7. The pose (P3)

The device pose is a second record on this one, not a change to it, and
is built (`sky-pose.md`): `almanac.skyPose` reads this record's request
with a `pose` beside it (the camera's line of sight, roll and field of
view), answers this record's `value` unchanged as its `sky`, and adds
where each of its points falls in the camera's picture. Every field here
keeps its name and meaning, so a reader of this record reads that one's
`sky` as is. The rising lagna point the pose asks for is this record's
`ASCENDANT`. The request is strict, so a `pose` sent to this record is
refused by name (`skyView.pose`) rather than ignored.

## 8. Cost

The ephemeris is read once per graha asked; a star is the star table's
own arithmetic; the lists are arithmetic on one frame. No fit, no search.
