# The limb oracle

Status: `active`, 2026-11-14 (SR-105). The gate that holds every panchanga
limb end, sankranti and sunrise the SDK computes to a recorded reference
sky, per limb and per century, and the measurement of the ayanamsha basis
(SR-100) it was built to settle.

## 1. What is gated

`crates/sdk/tests/limb_oracle.rs` founds the almanac at Kathmandu
(27.7172° N, 85.324° E, sea level, +05:45) over the whole of 1900, 1950,
2000, 2050 and 2100 under `nepali-drik`, `parashari-classical`,
`kp-default` and `conformance-baseline` over the built-in ephemeris, and
compares with the reference:

| quantity | compared | bound |
|---|---|---|
| tithi, karana, nakshatra and yoga ends | every end the reference has inside the days' windows, against the nearest SDK end, in TT (each side's own ΔT kept apart) | 2 s |
| sankrantis | every one, the same way | 4 s (SR-105 asks 3 s: §3) |
| sunrises | every local date's, in UT1, under the profile's own convention (`CENTRE_REFRACTION` for `nepali-drik`, `UPPER_LIMB_REFRACTION` for the rest), read as its definition: the geocentric altitude of the centre at −34′ plus the horizontal parallax, less the semidiameter (959.23″ at 1 AU) for the upper limb | 0.1 s |

The bounds are the ones SR-105 asks for, but for the sankranti's (§3). A reference instant with no SDK
change within ten minutes fails the test as a missing or extra change.
Each limb and century's worst under `nepali-drik` is recorded into
`ACCURACY.md` (`cargo xtask accuracy`, held by `check-accuracy`); every
profile is held to the same bounds.

With `TEISTRO_LIMB_PATCH` set to a settings patch, the test prints the
same table under it and asserts nothing: how the "before" column below
was measured (`{"frame":{"ayanamsha_basis":"MEAN"}}`).

## 2. The reference

`crates/sdk/tests/oracle/limbs.json`, written by
`crates/sdk/tests/oracle/record_limbs.py --de440`:

- **The places are JPL's DE440.** The kernel is `de440.bsp`, from
  <https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de440.bsp>: 119,799,808
  bytes, SHA-256
  `a4ce9bf9b3282becc9f4b2ac3cebe03a2ae7599981aabd7265fd8482fff7c4b5`,
  a DAF/SPK file of 14 segments covering JD 2287184.5 to 2688976.5
  (1549-12-31 to 2650-01-25). It is read by jplephem 2.24 and **never
  committed**: to reproduce the fixture, fetch it, check the digest, and
  run the recorder over it (its docstring has the command).
- **The reductions are the Swiss Ephemeris 2.10.03's**: light time,
  deflection, aberration, precession and nutation, and
  the ayanamshas `SIDM_LAHIRI` and `SIDM_KRISHNAMURTI`, which the SDK's
  catalogue reproduces. Each place is the Swiss place moved by the
  difference of the two astrometric directions, DE440's less the `.se1`
  files'. That difference was at most 0.0024″ for the Sun and 0.0086″ for
  the Moon over every place recorded, so the rotation of date it rides on
  changes it by under a micro-arcsecond.
- **Every sidereal longitude is the apparent one less the true
  ayanamsha**, the Indian Astronomical Ephemeris's nirayana longitude, so
  the reference does not depend on the profile under test (C1750).
- **The sidereal time is IAU 2006's**, the Earth rotation angle and
  Capitaine's polynomial plus the equation of the equinoxes, not
  `swe_sidtime`: from 2050 on (JD 2469807.5) Swiss turns to a long-term
  formula 1.8″ to 1.9″ from IAU 2006, 0.12 s of every sunrise, where over
  1850 to 2050 the two agree to 0.003″. Recorded through `swe_sidtime`,
  the reference put every 2050 and 2100 sunrise 0.12 s late.
- **Limbs are geocentric** and in TT; a tithi and a karana are the
  elongation's twelve and six degrees; a yoga is the sum of the two
  sidereal longitudes.

The Swiss Ephemeris alone over its DE431 fit, the recording this replaced
(C1752), differed from it by at most 0.016 s on a Moon limb, 0.056 s on a
sankranti and 0.0002 s on a sunrise.

## 3. Measured

Worst error under `nepali-drik`, in seconds; `parashari-classical`,
`kp-default` and `conformance-baseline` agree with it to a few
hundredths. "Before" is the mean basis every profile but
`conformance-baseline` took until SR-100; "after" is the true basis the
root takes now. The counts are per century column: 1900 and 1950 make
the 1900s, 2000 and 2050 the 2000s.

| limb | before 1900s | before 2000s | before 2100 | after 1900s | after 2000s | after 2100 | bound |
|---|---|---|---|---|---|---|---|
| tithi (742/741/372) | 0.788 | 0.631 | 0.673 | 0.788 | 0.631 | 0.673 | 2 |
| karana (1484/1483/743) | 0.800 | 0.631 | 0.673 | 0.800 | 0.631 | 0.673 | 2 |
| nakshatra (722/722/362) | 36.772 | 36.364 | 15.081 | 0.904 | 0.562 | 0.519 | 2 |
| yoga (775/776/388) | 68.214 | 66.593 | 27.999 | 1.003 | 0.734 | 0.620 | 2 |
| sankranti (24/24/12) | 440.1 | 425.1 | 188.6 | 3.859 | 2.941 | 2.476 | 4 |
| sunrise (730/731/365) | 0.013 | 0.011 | 0.008 | 0.013 | 0.011 | 0.008 | 0.1 |

- **The basis is the whole of the miss.** A tithi and a karana are a
  difference of two longitudes, so the ayanamsha cancels and the basis
  cannot move them; a nakshatra, a yoga and a sankranti read one or two
  sidereal longitudes, so the mean basis left the nutation in longitude
  (up to 17.2″) in each: 36 s of a nakshatra, 68 s of a yoga and 7.3
  minutes of a sankranti. On the true basis all three come down to the
  ephemeris's own error.
- **The sankranti misses SR-105's 3 s in 1900 and 1950.** The built-in
  apparent Sun runs ahead of DE440 by about 0.05″ to 0.1″ (a tithi's
  median error is +0.1 s to +0.2 s, a sankranti's −1.3 s to −2.3 s),
  and an arcsecond of the Sun is 24 seconds of a sankranti. That is the
  built-in's fit, not the basis, and the gate holds 4 s until it is
  refitted; a native engine plugged in over DE440 meets 3 s.
- **A sunrise is compared on one ΔT.** The reference records each
  sunrise with its ΔT and the instant's sensitivity to it (0.002 s to
  0.0033 s a second, by season), and the test moves it onto the SDK's ΔT
  before comparing, so the column measures the sunrise's geometry and not
  the two ΔT models, 97 s apart in 2100 (0.3 s of a sunrise).

What the basis moved, over 2000 to 2100 at Kathmandu under `nepali-drik`
(every day the true basis changes, against the mean):

- **No festival** of `NEPAL_PANCHANGA` moves, and no Bikram Sambat month
  start: the civil months are the official table's from BS 1970 to 2095
  and the Surya Siddhanta's outside it, neither of which reads the
  ayanamsha basis.
- **Six sankranti days**, each a sankranti within minutes of sunrise that
  the 7 to 8 minutes of nutation carried across it (Nepal time, true
  against mean): Kumbha 2003-02-13 06:39:52 against 06:45:27 (sunrise
  06:44:24), Mithuna 2013-06-15 05:10:53 against 05:05:48 (05:08:55),
  Mina 2058-03-15 06:14:00 against 06:20:44 (06:14:31), Simha 2088-08-17
  05:40:52 against 05:36:33 (05:36:25 the day before), Vrishabha
  2096-05-15 05:11:31 against 05:18:05 (05:15:35) and Mina 2097-03-15
  06:10:04 against 06:14:46 (06:13:53).
- **Two punya-kala days**, a night sankranti that crossed the middle of
  the night, which gives its punya-kala to the day that ended or to the
  one beginning: Mesha 2001-04-13 23:56:44 against 2001-04-14 00:04:00,
  and Vrischika 2011-11-16 23:49:43 against 23:43:33.
- **Ten nakshatra and nineteen yoga days**, an end within a minute of
  sunrise that moved across it.
