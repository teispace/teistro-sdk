# Tajika and Varshaphala (annual charts) and other pravesha charts

Status: `research`, 2026-09-04. Checked against the baseline engine's Tajika (whole-sign
solar return), JHora (annual, monthly, 2.5-day, 5-hour, 25-minute, 2-minute
charts, true or mean solar motion, sunrise charts, Tithi/Yoga/Nakshatra
Pravesha) and PyJHora (Muntha, balas, Ithasala family, Mudda, Patyayini).

| feature | inputs | variants | baseline | field | tier |
|---|---|---|---|---|---|
| Varsha Pravesh: solar return to the natal sidereal Sun longitude | crossing search | true solar motion (default) or mean (365.2564 days × n); precession-corrected in Western usage | yes | all | P0 |
| Maasa Pravesh (monthly, every 30° of Sun) and finer charts (60-hour, 2.5-day, 5-hour ...) | crossing search | | no | JHora, PyJHora | P1 |
| Muntha (progressed lagna one sign per year) and its lord | natal lagna, age | | partial | all | P0 |
| lord of the year (Varshesha) from the five candidates (Muntha lord, lagna lord of annual chart, natal lagna lord, Trirashi lord, Dinaratri lord) by Pancha Vargeeya Bala | | | partial | all | P0 |
| Pancha Vargeeya, Dwadasha Vargeeya and Harsha balas | see strengths | | partial | JHora, PyJHora | P0 |
| Tajika aspects: conjunction, sextile, square, trine, opposition with deeptamsha orbs per planet; applying versus separating by speed | annual chart | orb table (Sun 15, Moon 12, Mars 8, Mercury 7, Jupiter 9, Venus 7, Saturn 9) **verify** | partial | all | P0 |
| the sixteen Tajika yogas (Ikkavala, Induvara, Ithasala kinds, Ishrafa, Nakta, Yamaya, Manau, Kamboola, Gairi Kamboola, Khallasara, Radda, Duphali Kutta, Dutthotha Davira, Tambira, Kuttha, Durapha) | aspects, balas | | partial | JHora, PyJHora | P0 |
| sahamas (36) with day and night formulas | annual chart | | no | JHora, PyJHora | P1 |
| Mudda (Varsha Vimshottari) and Patyayini dashas | annual chart | | partial | all | P0 |
| Tithi Pravesha, Yoga Pravesha, Nakshatra Pravesha annual and monthly charts | Sun–Moon composite crossing | | no | JHora | P1 |
| Tajika sunrise charts | | | no | JHora | P2 |
| Western solar and lunar returns, planet returns, precession-corrected, demi and quarti returns | crossing search | tropical | no | Solar Fire | P1 (`western`) |

## Closing checklist

- Confirm deeptamsha orbs and the applying rule for retrograde bodies.
- The Varsha Pravesh instant depends on the ayanamsha and the natal Sun's
  exact longitude, so it is a good cross-provider conformance case.
