# Transits and gochar

Status: `research`, 2026-09-04. Checked against the baseline engine's predictive package
(gochar overlays from Moon and lagna, transit-to-natal aspects, Sade Sati by
ephemeris scan, transit strength via Ashtakavarga, phala with vedha, transit
calendar), JHora's transit features, Kala's "Transits Hit List" and Solar
Fire's dynamic reports and TimeMap.

| feature | inputs | variants | baseline | field | tier |
|---|---|---|---|---|---|
| transit snapshot at an instant with the natal frame (houses from Moon, lagna, navamsa lagna, navamsa Moon) | positions | | yes (Moon, lagna) | JHora (four references) | P0 |
| classical gochar results per planet from the Moon sign with vedha (obstruction) points and exemptions | tables | vedha tables per text; Sun–Saturn and Moon–Mercury exemptions | yes | all | P0 |
| tara-based and murthi-based transit classification | nakshatra of transit relative to natal Moon; Moon's position at ingress | | partial (tara chakra) | JHora | P1 |
| Ashtakavarga transit scoring with kakshya | BAV | | yes | JHora, PL | P0 |
| Sade Sati and Dhaiyya (Saturn from the Moon) with phases and exact dates, plus Kantaka Shani, Ashtama Shani | Saturn ingress search | 12th, 1st, 2nd from Moon; Panoti variants | yes | all | P0 |
| transit-to-natal aspects with orbs (Western style) and Parashari sign aspects | | | yes | all | P0 |
| ingress dates by sign, nakshatra, navamsa and any varga; stations; retrograde periods | crossing search | | yes (sign) | JHora (rasi, nakshatra, navamsa, varga change charts) | P0, varga P1 |
| transit hit list over a date range: every exact aspect, ingress, station, with entering, exact and leaving times, sorted | crossing search over natal points | Kala's Transits Hit List; Solar Fire dynamic reports | partial (transit calendar) | Kala, Solar Fire, JHora transit search | P0 |
| transit calendar (day, week, month views) | | | yes | JHora, Solar Fire | P0 |
| Moon transit daily (chandra gochar), nakshatra transits, tithi-based triggers | | | partial | all | P0 |
| special tara transits, nakshatra aspects in transit, latta in transit | | JHora | no | JHora | P2 |
| eclipse impact on natal points | eclipse search | | no | Solar Fire, JHora | P1 |
| dasha and transit coincidence (double transit of Jupiter and Saturn on a house, dasha lord transiting significators) | dashas + transits | | partial | many | P1 |
| graphic ephemeris and time map visualisation data (longitude tracks, modulus) | positions grid | | no | Solar Fire, Maitreya | P1 (data only) |
| rashifal: sign-based periodic forecasts from transit context and scoring | see the baseline engine rashifal-core | | yes | consumer apps | P0 |

## Closing checklist

- Design the hit-list search as a first-class batch API over the crossing
  search of the ephemeris port (with the SDK sample-and-bisect fallback).
- Vedha table with citations; confirm the exemptions.
- Sade Sati phase boundaries by sign entry (default) versus by degree from
  the Moon (45° windows); offer both.
