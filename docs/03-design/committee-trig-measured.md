# The committee's trigonometry

Status: `built`, 2026-11-15. Measured on the committee's two printed
years; the setting is `frame.siddhanta.trig` (C1950). There is
no `check-` gate that re-measures this page: its corpus is the
committee's print, which stays private. What of it is public is held elsewhere:
`check-nepal-day` holds the committee's sky against public daily
records, and `crates/siddhanta/tests/official.rs` holds the public
fixture's Sun to 2 arcseconds under the true sine.

The Surya Siddhanta computes every equation through sines: the equation
of centre of each graha (II.45, II.47 to 51), the declination (II.28)
and the arcs it reads back (II.33). The text gives its sines as a table of twenty-four,
one every 225 arcminutes of the quadrant on a radius of 3438 (II.15 to
27), read between rows by proportion (II.31 to 33). Until this page
every Surya profile used that table. `frame.siddhanta.trig` now chooses:

| value | what the equations take |
|---|---|
| `TABLE` | the text's twenty-four sines, read by proportion (the default) |
| `EXACT` | the true sine on the same radius |

Both are the same on every platform to the bit. The table uses no
transcendental function. The true sine is the SDK's own
(`teistro_core::math`, the pure-Rust port of musl's libm, ADR-0022), never
the platform's C library, whose sine differs by an ulp between Apple's
and Android's.

## 1. Measured

The corpus is Nepal's national panchanga committee's *Rashtriya
Panchangam* for VS 2082 and 2083, 730 days at Kathmandu, read off the
editions' PDFs (the files C1812 names). Every printed tithi, nakshatra,
yoga and karana end, Moon sign change, sankranti, sunrise and sunset is
paired with the SDK's end of the same member under `nepali-committee`
with only the trigonometry changed. Each morning's printed Sun and Moon,
given to the second of arc, are compared at the printed sunrise minute.
The digitised corpus and its scorer are kept in a private repository,
since the print is the committee's. This page gives statistics only,
never a printed value.

The build measured is `lane/siddhanta-trig` on `d8a31ddd9`, with the
setting added. The error is printed minus SDK. Ends are in minutes,
places in seconds of arc. The statistics take the ends printed with a
clock that agrees with its own ghati-pala, as `nepal-day-measured.md`
does.

### 1.1 The ends, per year

Mean |error| and the largest |error|, in minutes:

| limb | n (2082, 2083) | `TABLE` 2082 | `EXACT` 2082 | `TABLE` 2083 | `EXACT` 2083 |
|---|--:|--:|--:|--:|--:|
| tithi | 371, 370 | 0.31, 1.0 | 0.26, 0.6 | 0.32, 1.1 | 0.25, 0.6 |
| nakshatra | 359, 361 | 0.32, 0.9 | 0.27, 0.6 | 0.32, 1.0 | 0.26, 0.6 |
| yoga | 385, 384 | 0.29, 0.9 | 0.26, 0.6 | 0.29, 0.8 | 0.25, 0.6 |
| karana | 364, 372 | 0.35, 1.7 | 0.26, 1.5 | 0.32, 1.0 | 0.26, 0.6 |
| Moon sign | 160, 160 | 0.32, 0.8 | 0.26, 0.6 | 0.27, 1.0 | 0.24, 0.6 |
| sankranti | 12, 12 | 0.65, 1.6 | 0.25, 0.5 | 0.52, 1.3 | 0.23, 0.4 |
| sunrise | 365, 365 | 0.77, 1.6 | 0.77, 1.6 | 0.76, 1.6 | 0.76, 1.6 |
| sunset | 365, 365 | 0.50, 1.1 | 0.50, 1.1 | 0.50, 1.1 | 0.50, 1.1 |

The print rounds to the whole minute, and an error spread evenly over
that minute has a mean |error| of 0.25. Under `EXACT` every limb's end
sits on that floor (0.23 to 0.27) and no end is more than 0.6 minutes
off, except the one karana whose printed clock disagrees with its own
ghati-pala by 1.4 minutes (§1.4). Under `TABLE` the means are 0.29 to
0.35, and the sankrantis 0.52 to 0.65. The other limbs' signed means do not
move (+0.03 to +0.09 under both), so the table does not shift the committee's
clock; it adds a scatter. The ends over 1.5 minutes fall from 17 to 15,
and the 15 left are sunrises (§1.4).

### 1.2 The sankrantis, per sign

The minute the Sun enters each sign, printed minus SDK (2082, 2083):

| sign | `TABLE` | `EXACT` |
|---|--:|--:|
| Aries | −1.61, −1.22 | −0.24, +0.15 |
| Taurus | −0.47, −0.08 | −0.22, +0.17 |
| Gemini | −0.32, +0.08 | +0.02, +0.42 |
| Cancer | −0.28, +0.11 | −0.42, −0.03 |
| Leo | +0.88, +1.27 | +0.01, +0.40 |
| Virgo | +0.93, +0.32 | +0.30, −0.31 |
| Libra | +0.49, +0.88 | −0.15, +0.24 |
| Scorpio | +0.68, +1.07 | −0.17, +0.22 |
| Sagittarius | +0.66, +0.05 | +0.28, −0.33 |
| Capricorn | −0.37, +0.02 | −0.35, +0.04 |
| Aquarius | +0.58, −0.03 | +0.51, −0.10 |
| Pisces | −0.51, −1.12 | +0.30, −0.31 |

Under the table each sign keeps its sign of error from one year to the
next where the error is large: Aries early by 1.2 to 1.6 minutes in both
editions, Leo to Scorpio late, Pisces early. That is a bias of the
model's Sun at those longitudes, not the print's. Under the true sine
no sign errs by more than 0.51 minutes, and none keeps a large error in
both years.

### 1.3 The places at sunrise

Printed minus SDK, in seconds of arc, 365 mornings each:

| year | body | `TABLE` mean, mean \|e\|, max \|e\| | `EXACT` mean, mean \|e\|, max \|e\| |
|---|---|--:|--:|
| 2082 | Sun | −0.1, 2.0, 6.2 | −0.1, 0.7, 1.8 |
| 2082 | Moon | −2.5, 10.0, 28.8 | −2.4, 8.8, 19.9 |
| 2083 | Sun | −0.2, 2.0, 6.4 | −0.2, 0.7, 1.9 |
| 2083 | Moon | −3.0, 10.1, 30.6 | −2.8, 8.5, 20.3 |

The Sun's mean error by the sign it stands in, in seconds of arc
(2082; 2083 agrees within 0.4):

| | Pisces | Aries | Taurus | Gemini | Cancer | Leo | Virgo | Libra | Scorpio | Sagittarius | Capricorn | Aquarius |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `TABLE` | +3.1 | +2.2 | +1.7 | 0.0 | −1.7 | −2.4 | −3.6 | −2.2 | −1.9 | −0.2 | +1.3 | +1.8 |
| `EXACT` | −0.2 | −0.1 | −0.1 | 0.0 | −0.1 | −0.3 | −0.1 | 0.0 | −0.2 | −0.2 | −0.2 | −0.3 |

Under the table the Sun carries a one-cycle yearly wave of about ±3.5″.
The Sun is ahead of the print from Leo to Scorpio, so it enters those
signs early and the print's sankranti is later; it lags from Capricorn
to Aries, so the print's sankranti is earlier. That is §1.2's pattern,
from the same cause. Under the true sine the wave is gone. What is left
is the floor two roundings set. The print gives the place to the
second, and it gives the sunrise to the minute, in which the Sun moves
about 2.5″ and the Moon about 32″. Those spread evenly give a mean
|error| near 0.67″ for the Sun and 8″ for the Moon, and the true sine
measures 0.7″ and 8.5 to 8.8″.

### 1.4 What the trigonometry does not move

The sunrise and sunset rows are the same to the hundredth of a minute
under both settings, as they must be: `nepali-committee`'s sunrise is a
modern one (`day.sunrise` `CENTRE_REFRACTION`, `frame.siddhanta.sunrise`
`MODERN`), computed from the built-in ephemeris and not from the text.
The sunrise residual's yearly wave (C39: the print's sunrise later than
the SDK's by a month's mean of +1.0 to +1.2 minutes from March to May)
remains, and the 15 ends over 1.5 minutes under `EXACT` are all
sunrises on that wave. The one karana at 1.5 minutes under `EXACT` is
the end whose printed clock disagrees with its own ghati-pala, which
agrees with the SDK within 0.3 minutes under both settings.

### 1.5 The public records agree

`nepal-day-measured.md` scores the same profile against public daily
records of the committee's print (`check-nepal-day`). With the true
sine its worst tithi, nakshatra and yoga ends there move from −1.2 to
+0.9 minutes to −0.6 to +0.4, over 326 to 333 days.

## 2. Decided

`nepali-committee` (version 7) computes with `EXACT`. Its print is the
true sine's on every limb, in both years. The table it does not use
leaves a yearly wave in the Sun of three times the print's resolution,
and an Aries sankranti early by more than a minute. The other Surya
profiles (`mithila-makaranda`, and `Siddhanta::surya` wherever a
setting names it) keep `TABLE`, the text as written (II.15 to 33). A
consumer reckoning by the text chooses either, per profile, per context
and per call, as with every setting.
