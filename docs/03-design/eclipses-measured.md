# Eclipses, measured

Status: `generated` by `cargo xtask eclipses`. Do not edit:
`check-eclipses` regenerates this page and fails on any difference.

It holds `astro::eclipse` (`eclipses.md`) over the built-in ephemeris
against every eclipse of NASA's Five Millennium Canon between the years
it reads, both ways, and measures the placings and the shadow rule the
design chose between.

## 1. The claims

| proposed rule | verdict | measured |
|---|---|---|
| every catalogued lunar eclipse of 1900 to 2100 is found, and no other | **holds** | 0 of 459 disagree; 459 found, 0 missed, 0 not catalogued |
| every catalogued solar eclipse of 1900 to 2100 is found, and no other | **holds** | 0 of 454 disagree; 454 found, 0 missed, 0 not catalogued |
| a lunar eclipse's kind is the catalogue's | **holds** | 0 of 459 disagree |
| a lunar eclipse has the phases the catalogue times, and no others | **holds** | 0 of 459 disagree |
| a solar eclipse's kind is the catalogue's, hybrids read at the greatest eclipse | **holds** | 0 of 454 disagree |
| the greatest moment is the catalogue's to the seconds it prints | **holds** | worst 2.8826 s, bound 5 s |
| gamma is the catalogue's | **holds** | worst 0.0001 Earth radii, bound 0.0005 Earth radii |
| every magnitude is the catalogue's, the umbra's under Danjon's rule | **holds** | worst 0.0003, bound 0.001 |
| a lunar eclipse's phases last as long as the catalogue says | **holds** | worst 0.6096 min, bound 1 min |
| a solar eclipse is greatest where the catalogue says | **holds** | worst 0.3130 °, bound 0.5 ° |
| a lunar eclipse reads both bodies apparent, nearer the catalogue than either other placing | **holds** | median +0.16 s |
| the catalogue enlarges the shadow by Danjon's rule and not Chauvenet's | **holds** | Chauvenet's median +0.0083 umbral, +0.0285 penumbral |

## 2. The records

Fred Espenak and Jean Meeus, *Five Millennium Canon of Solar
Eclipses* (NASA TP-2006-214141) and *of Lunar Eclipses* (NASA
TP-2009-214172), NASA Goddard Space Flight Center, in the public
domain: the catalogues `5MCSEcatalog.txt` and `5MKLEcatalog.txt`,
459 lunar and 454 solar eclipses from 1900 to
2100, held in `xtask/src/eclipses/` a line an eclipse. The
catalogue's instants are TT; ours are UT1 and taken back to TT
under the Delta T model the search ran with, so the model cancels.
An answer and a record are one eclipse when their greatest moments
are within 0.1 days.

Missed lunar: none. Not catalogued lunar: 0. Missed solar: none. Not
catalogued solar: 0.

## 3. Lunar eclipses

| quantity, ours less the catalogue's | eclipses | least | median | greatest | bound |
|---|---|---|---|---|---|
| greatest moment | 459 | -2.13 | +0.16 | +2.88 | 5 s |
| gamma | 459 | -0.00014 | +0.00000 | +0.00013 | 0.0005 |
| umbral magnitude | 459 | -0.00027 | -0.00006 | +0.00017 | 0.001 |
| penumbral magnitude | 459 | -0.00032 | -0.00011 | +0.00005 | 0.001 |
| total phase, minutes | 166 | -0.35 | -0.01 | +0.05 | 1 min |
| partial phase, minutes | 288 | -0.14 | +0.01 | +0.07 | 1 min |
| penumbral phase, minutes | 459 | -0.61 | +0.05 | +0.13 | 1 min |

The catalogue prints a phase to a tenth of a minute, and a phase grazing
its shadow's edge is long for a small error in gamma: the greatest
differences are the shallowest eclipses.

## 4. Solar eclipses

| quantity, ours less the catalogue's | eclipses | least | median | greatest | bound |
|---|---|---|---|---|---|
| greatest moment | 454 | -0.94 | +0.12 | +1.00 | 5 s |
| gamma | 454 | -0.00011 | -0.00000 | +0.00012 | 0.0005 |
| magnitude | 454 | -0.00018 | -0.00001 | +0.00016 | 0.001 |
| latitude of greatest eclipse | 454 | -0.21 | -0.00 | +0.21 | 0.5 ° |
| longitude of greatest eclipse | 454 | -0.31 | -0.02 | +0.27 | 0.5 ° |

The catalogue prints the point to a tenth of a degree. Its gamma is the
sphere's and is read so here; the kind and the magnitude are the
flattened Earth's (`eclipses.md` §4.3).

## 5. The placing and the shadow

Which place of each body the shadow is read from decides the greatest
moment; the shipped placing is the light's path (`eclipses.md` §4.1),
and the other two are measured here over the same records.

| the lunar eclipse's placing | greatest moment, median offset |
|---|---|
| both apparent (shipped) | +0.2 s |
| the Moon apparent, the Sun astrometric | +38.5 s |
| both astrometric | +76.8 s |

The shadow enlarged by Chauvenet's rule rather than Danjon's reads the
umbral magnitudes +0.0083 and the penumbral +0.0285 from the catalogue
at the median, where Danjon's reads -0.00006 and -0.00011: the catalogue
is Danjon's, which is the default, and Chauvenet's stays a knob for the
almanacs that used it.

## 6. What the records decide

The search finds the canon's eclipses and no others over two centuries,
of every kind, and agrees with each to the precision the canon prints.
So `astro::eclipse` is the SDK's eclipse, ready for the almanac's day
and the muhurta's blackouts: the eclipse's nakshatra (grahanotpatha) and
its sutak.
