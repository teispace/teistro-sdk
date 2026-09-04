# Panchanga, calendar and muhurta

Status: `research`, 2026-09-04. Checked against the baseline engine's panchanga package
(daily panchanga, timing root-finder, hora, choghadiya, muhurta search,
blackout calendar, event rules, saait), PyJHora's panchanga list, JHora's
panchanga features and Drik Panchang's conventions.

## The sunrise-anchored day

| feature | inputs | variants | baseline | field | tier |
|---|---|---|---|---|---|
| sunrise, sunset, next sunrise with the disc and refraction convention | rise/set | centre-no-refraction (traditional Hindu), upper-limb-with-refraction (civil), Drik Panchang convention, custom altitude | yes | all | P0 |
| day and night durations, ghati-pala from sunrise (civil 24-minute ghati versus proportional) | | | yes | JHora, PyJHora | P0 |
| moonrise and moonset | | | no | JHora, PyJHora | P0 |
| midday, midnight (local apparent) | equation of time | | no | PyJHora | P0 |
| Brahma muhurta, Godhuli, Sandhya, Vijaya, Nishita, Abhijit, Udaya lagna muhurta | sunrise, sunset | Abhijit as the 8th muhurta (24 minutes each side of local noon); not Wednesday rule | Abhijit yes | PyJHora (all) | P0 |

## The five limbs and derived limbs

| feature | inputs | variants | baseline | field | tier |
|---|---|---|---|---|---|
| tithi (30) with exact begin and end instants; paksha | Moon − Sun in 12° steps, root-found | | yes | all | P0 |
| vara (weekday) with lord | sunrise-based day | | yes | all | P0 |
| nakshatra (27) with padas and end instants; 28-nakshatra variant with Abhijit (276°40′–280°53′20″) | sidereal Moon | | yes (27; Abhijit noted) | all | P0 |
| yoga (27) with end instants; nature classification | Sun + Moon | | yes | all | P0 |
| karana (11 kinds over 60 halves) with end instants; Vishti flag | half tithi | | yes | all | P0 |
| lunar month (Amanta, Purnimanta), adhika and kshaya masa, solar month (sankranti), ritu, ayana | Sun sign, new moons | | yes (month systems) | all | P0 |
| samvatsara (60-year Jovian cycle), eras (Vikram, Shaka, Kali, Nepal Sambat, Buddha) | calendar | Jovian cycle by mean Jupiter or by year count (north versus south schemes) **verify** | partial | all | P0 |
| Rahu kaal, Yamaganda, Gulika kaal | day eighths by weekday | | yes | all | P0 |
| Durmuhurta, Varjyam, Amrita kalam, Chandrashtama, Panchaka (with the five types), Bhadra | nakshatra and tithi tables | | Panchaka yes | PyJHora, Drik Panchang | P0 |
| choghadiya day and night; Gauri choghadiya; hora (24 planetary hours from sunrise) | | | yes | all | P0 |
| Disha shool, Chandra niwas (Moon's direction), Nakshatra shool | vara and nakshatra | | yes | all | P0 |
| muhurta yogas: Amrit Siddhi, Sarvartha Siddhi, Siddha, Dwipushkar, Tripushkar; and Tamil yogas (Siddha, Amrita, Marana, Prabalarishta); Ravi yoga, Guru Pushya, Ravi Pushya | vara + nakshatra + tithi tables | | five yes | all | P0 |
| Anandadi yogas (28), Shiva vaasa, Agni vaasa, Yogini vaasa | | | no | PyJHora | P1 |
| Tarabalam and Chandrabalam (day-level and natal) | nakshatra and sign counts | | yes (day-level uses a heuristic; see baseline) | all | P0 |
| special tithis and karaka tithis (JHora) | | | no | JHora, PyJHora | P2 |
| Sahasra Chandrodayam, Triguna, Vivaha Chakra Palan, Kaala Chakra of the day | | | no | PyJHora | P2 |
| eclipses of the day | | | no | JHora | P1 |
| festivals and vratas (Ekadashi, Pradosha, Sankranti, Amavasya, Purnima, Shivaratri, Chaturthi variants, Navaratri, Dashain, Tihar and the Nepali cycle) as calendar rules | lunar calendar | rule packs per region; Nepal official calendar as data | The baseline engine scrapes the official calendar and reconciles festivals; rule-based computation absent | PyJHora (vrata finder), Drik Panchang | P1 (rule pack) |
| published auspicious days (saait: marriage, bratabandha, ...) as data | | | yes (import and query) | | P0 as a data module |

## Calendars

See `platform/05-calendars-timezones.md` for the inventory and algorithms.
The panchanga needs Gregorian, Julian and the Indian lunisolar calendar in
v1; BS is table-driven and needs the official table plus a computed extension
outside the published span.

## Muhurta (electional) computation

| feature | inputs | variants | baseline | field | tier |
|---|---|---|---|---|---|
| activity catalogue with rules per activity (marriage, griha pravesh, upanayana, naming, annaprashana, travel, business, education, vehicle, property, medical ...) | rule pack | Muhurta Chintamani and regional practice | yes (MUHURTA_EVENT_RULES) | all | P0 |
| permitted nakshatras, tithis, weekdays per activity | tables | | yes | all | P0 |
| blackout periods: Chaturmas, Adhika masa, Kharmas (Dhanu and Meena sankranti), Pitru paksha, Guru and Shukra asta (combustion of Jupiter and Venus), Holashtaka, eclipses | Sun sign, lunar month, combustion | | yes (six kinds) | all | P0 |
| Panchanga shuddhi scoring (yoga purity, Vishti, Panchaka, muhurta yogas, paksha bala) | daily panchanga | | yes | all | P0 |
| Tara bala and Chandra bala against the native | natal Moon | | yes | all | P0 |
| lagna evaluation at the elected instant (benefics in kendra, 7th empty for marriage, lagna lord strength, Dosha-Bhanga) | chart at instant | | yes | all | P0 |
| event karaka evaluation (Venus for marriage, Jupiter for griha pravesh ...) | | | yes | | P0 |
| Mahadosha veto (Vyatipata, Vaidhriti, Vishti, combust karaka) | | | yes | | P0 |
| intra-day windows from choghadiya, hora, Abhijit, minus avoid windows | | | yes | all | P0 |
| ranked search over a date range with reasons and warnings | | | yes (90-day cap) | all | P0 |
| Muhurta helper for a fixed time (evaluate a given instant) | | | partial | Kala | P0 |
| Kala's "Muhurta Module", SJS "Election tools", PL Muhurta: same shape | | | | | |
| Western electional: void-of-course Moon, planetary hours, Moon phase, dignity searches (Solar Fire's electional searches) | | | no | Solar Fire | P1 |

## Closing checklist

- Replace the baseline engine's day-level Chandrabalam heuristic (Sun sign as the
  weekday-lord sign) with the classical definition, and drop the invented
  bhagyat/srudhakat/bhumika markers or cite them.
- Confirm the Panchaka types and the Bhadra (Vishti) residence rules.
- Define the festival rule-pack format so the Nepali official calendar can
  be both scraped data (as the baseline engine does) and computed rules with a diff.
