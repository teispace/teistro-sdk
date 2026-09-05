# Vedic Parashari core: positions, frames, points, state, relationships

Status: `research`, 2026-09-04. Checked against the baseline engine (read in
full), the JHora feature list, PyJHora's README and Solar Fire's point
catalogue. Items marked **verify** are from memory of the classical texts and
must be confirmed before they become golden vectors.

## A. Astronomy inputs the chart needs

| feature | inputs | variants | baseline | field | tier | module |
|---|---|---|---|---|---|---|
| nine grahas: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu | ephemeris positions with speed | Rahu and Ketu mean or true; Ketu always opposite Rahu | yes | all | P0 | `chart` |
| outer planets Uranus, Neptune, Pluto as optional bodies | ephemeris | include or exclude in yogas and vargas | partial (positions only) | JHora, PL, Maitreya | P0 | `chart` |
| lunar apogee (Lilith) mean and osculating, interpolated apogee and perigee | ephemeris | mean, osculating, interpolated | no | Solar Fire, Maitreya | P1 | `chart` |
| asteroids Chiron, Ceres, Pallas, Juno, Vesta and numbered minor planets | ephemeris with asteroid files | catalogue-dependent | no | Solar Fire (1,081), Maitreya | P1 | `western` |
| planetary latitude, distance, declination and right ascension | ephemeris equatorial output or SDK transform via obliquity | | partial | JHora, Solar Fire | P0 | `chart` |
| topocentric correction | observer lat, lon, alt; ephemeris flag | on or off, per profile (the baseline engine default on) | yes | JHora | P0 | port |
| true versus apparent positions (light-time, aberration, deflection) | ephemeris flags | apparent (default), true | partial | JHora | P0 | port |
| Surya Siddhanta positions | SDK siddhantic model or provider | drik (default), Surya Siddhanta with bija corrections on or off | yes | JHora (Makaranda variant) | P0 | `siddhanta` (optional module) |
| sunrise and sunset with disc and refraction conventions | rise/set search with horizon altitude | centre without refraction, upper limb with refraction (Hindu apparent), Drik Panchang convention, custom altitude | yes (modes) | all panchanga tools | P0 | port with SDK fallback |
| moonrise and moonset | rise/set search | same conventions | no | JHora, PyJHora | P0 | port |
| solar and lunar eclipses, global and local, with type and magnitude | eclipse search | | no (docs mention eclipse charts) | JHora, Solar Fire, Maitreya | P1 | port |
| fixed stars: positions, magnitude, parans, aspects | star catalogue | catalogue, orb | no | Solar Fire (290 stars), JHora (nakshatra yogatara) | P1 | `western` |
| planetary stations and ingresses | crossing search or SDK sample-and-bisect over positions | | yes (transit service) | JHora, Solar Fire | P0 | `gochar` |
| planetary conjunction and opposition instants | crossing search on a composite angle | | partial | JHora (conjunction charts) | P0 | `gochar` |

## B. Frames and references

| feature | inputs | variants | baseline | field | tier | module |
|---|---|---|---|---|---|---|
| zodiac | ayanamsha id, jd | tropical; sidereal with any of 47 ayanamshas (Swiss catalogue) plus user-defined epoch and rate | yes | all | P0 | `core` |
| ayanamsha value | provider | mean or nutated (Teimeris and Swiss differ in default; see `platform/03`) | yes | all | P0 | port |
| house systems | ARMC, obliquity, latitude from provider or SDK arithmetic | 26 Swiss systems; Vedic policy: whole sign for rules, Bhava-Chalit (Sripati or Vehlow or Porphyry) for bhava bala, Placidus for KP, equal from lagna, equal from MC | yes (22) | JHora 4 to 17, PL, SJS, PyJHora 17, Solar Fire 30 | P0 | `houses` |
| Bhava-Chalit cusp and span model | cusps | Sripati (Porphyry-trisected midpoints), Vehlow (equal from midpoint), KP (Placidus with cusp as start) | yes but mislabelled | all Vedic | P0 | `houses` |
| house placement of a body | cusps and longitude | by span (cusp to cusp), by whole sign, by equal-from-lagna | yes | all | P0 | `houses` |
| coordinate transforms ecliptic, equatorial, horizontal | obliquity, sidereal time, observer | | partial | Solar Fire, JHora | P0 | `core` |
| date range | provider coverage | | 1800–2399 with Moshier fallback | JHora 5400 BCE–5400 CE | P0 | port capability |

## E. Derived points

| feature | inputs | variants | baseline | field | tier | module |
|---|---|---|---|---|---|---|
| lagna (ascendant), MC, descendant, IC | provider cusps | | yes | all | P0 | `houses` |
| special lagnas: Bhava, Hora, Ghati (Ghatika), Vighati, Varnada, Sree, Indu, Pranapada, Nisheka? | lagna, Sun, sunrise, time since sunrise | Varnada has at least three published methods (BPHS, Sanjay Rath, Iranganti Rangacharya) **verify**; Pranapada from Sun or from lagna; Hora lagna rate 1 sign per 2.5 ghati | yes (the baseline engine computes 11 lagnas) | JHora 11+, PyJHora | P0 | `points` |
| upagrahas from the Sun: Dhuma, Vyatipata, Parivesha, Indrachapa, Upaketu | Sun longitude | fixed offsets (Sun+133°20′, 360−Dhuma, Vyatipata+180, 360−Parivesha, Indrachapa+16°40′) **verify** | yes | JHora, PyJHora | P0 | `points` |
| upagrahas from day divisions: Gulika, Mandi, Kala, Mrityu, Ardhaprahara, Yamaghantaka | sunrise, sunset, weekday, birth time | day and night eighths by weekday lord; rising sign at the start (Parashara) or the middle (Kalidasa) of the portion; Gulika versus Mandi distinction (start versus middle) **verify**; Saturn's portion | yes (Gulika, Mandi) | JHora (all 9), PyJHora | P0 | `points` |
| arudha padas A1–A12 (bhava arudhas) | lord positions | exception rule when the pada falls in the 1st or 7th from the house (10th from it, or 4th from it by some) **verify**; Rahu/Ketu co-lordship for Aquarius and Scorpio (stronger lord rules) | yes (11 padas) | JHora, PyJHora | P0 | `jaimini` |
| graha arudhas, Chandra and Surya arudhas | | two configurations | no | JHora | P1 | `jaimini` |
| Bhrigu Bindu | Moon and Rahu midpoint | | yes | JHora, PyJHora | P0 | `points` |
| Yogi, Avayogi, Sahayogi points and lords | Sun+Moon+93°20′ | | partial | JHora, PyJHora | P0 | `points` |
| sphutas: Trisphuta, Chatussphuta, Panchasphuta, Prana, Deha, Mrityu, Beeja, Kshetra, Sookshma Trisphuta, Tithi, Yoga, Rahu Tithi sphutas | lagna, Moon, Gulika, Sun, other lords | formulas differ by text (Prasna Marga versus Parashara) **verify** | partial | JHora, PyJHora (12) | P1 | `points` |
| Kunda (JHora) | | | no | JHora | P2 | `points` |
| sahamas (36 Tajika sahamas) | day/night birth, planets, cusps | day and night formulas; JHora and PyJHora both list 36 | no | JHora, PyJHora | P1 | `tajika` |
| 64th navamsa and 22nd drekkana lords, mrityu bhaga, pushkara navamsa and bhaga | positions | 22nd drekkana has 4 definitions (JHora) | partial (mrityu bhaga, pushkara) | JHora, PyJHora | P0 | `points` |
| Ayudha, Sarpa, Pakshi drekkanas | drekkana tables | | no | JHora | P1 | `points` |
| Marana Karaka Sthana | body and house | | yes | JHora, PyJHora | P0 | `state` |

## Vargas (divisional charts)

| feature | inputs | variants | baseline | field | tier | module |
|---|---|---|---|---|---|---|
| standard vargas D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D16, D20, D24, D27, D30, D40, D45, D60 | longitudes | Parashari mapping; D2 six variants (Parashara, Kashinatha, Jagannatha, odd-even, and others), D3 four (Parashara, Jagannatha, Somnath, Parivritti), D4 two, D5 two, D8 two, D9 three (Parashara, Kalachakra, Parivritti), D11 two, D30 three, D81 two, D108 two (JHora counts) | yes (the baseline engine has the standard set; variant coverage to verify) | JHora 23, PyJHora 22, PL, SJS | P0 for standard, P1 for all variants | `vargas` |
| extended vargas D81, D108, D144 | | | partial | JHora, PyJHora | P1 | `vargas` |
| custom D-N for any N from 1 to 300, and mixed D-m of D-n | | cyclic mapping rules | no | JHora, PyJHora | P1 | `vargas` |
| vargottama, varga counts for vimshopaka (shad-, sapta-, dasha-, shodasha-varga) | vargas | | partial | JHora | P0 | `vargas` |
| varga lagna and varga house placement | varga longitudes | whole sign in vargas (standard) | yes | all | P0 | `vargas` |
| time adjustment: when does the lagna or a body change in a given varga | crossing search on the varga boundary | | no | JHora | P1 | `vargas` |

## F. Qualification and state

| feature | inputs | variants | baseline | field | tier | module |
|---|---|---|---|---|---|---|
| dignities: exaltation and debilitation with degrees, mooltrikona ranges, own sign, friend, neutral, enemy | positions, relationship tables | mooltrikona ranges per text (Moon 4–20 Taurus BPHS 47); relationship: natural, temporary (2,3,4,10,11,12 from), compound (five-fold) | yes | all | P0 | `dignity` |
| exaltation strength as a continuous measure (uchcha bala) | degrees from debilitation | | yes (Shadbala) | all | P0 | `strength` |
| combustion (asta) | distance from Sun | orbs per planet with retrograde variants (Mercury 14/12, Venus 10/8, Mars 17, Jupiter 11, Saturn 15; Moon 12): verified 2026-09-05 as the Surya Siddhanta's own numbers (IX.6 to 8 and X.1, Burgess 1860), which the text defines as degrees of time in oblique ascension and the tradition reads as degrees of longitude (C44); both compute in `astro::visibility` | yes | all | P0 | `state` |
| retrogression and stations | speed | direct, retrograde, stationary threshold | yes | all | P0 | `state` |
| planetary war (graha yuddha) | two bodies within 1° | winner by declination, latitude, or brightness; Rahu, Ketu, Sun and Moon excluded | partial | JHora, PyJHora | P0 | `state` |
| avasthas: Baladi (bala, kumara, yuva, vriddha, mrita by degree bands, reversed in even signs), Jagradadi (jagrat, svapna, sushupti), Deeptadi nine (deepta, svastha, mudita, shanta, deena, dukhita, vikala, khala, kopa), Lajjitadi six (lajjita, garvita, kshudita, trishita, mudita, kshobhita), Sayanadi twelve with sub-states (drishti, cheshta) | positions, dignity, aspects, nakshatra counts | Sayanadi formula variants **verify** | yes | JHora, PyJHora, the baseline engine has all five | P0 | `avastha` |
| gandanta (junctions of water and fire signs), abhukta mula, sandhi | longitudes | orb definitions (last 3°20′ of Revati, Ashlesha, Jyeshtha and first 3°20′ of the next; tithi and lagna gandanta) | partial (dosha rules) | all | P0 | `state` |
| sign, nakshatra, pada, navamsa sign, nakshatra lord, sub lord | longitudes | | yes | all | P0 | `core` |
| tara (navatara) from the natal Moon, special taras (Karma, Samudayika, Sanghatika, Jaati, Desa), latta | nakshatra indices | JHora definitions **verify** | partial (tara chakra) | JHora | P0 for navatara, P1 for special taras | `nakshatra` |
| special tithis (Janma, Dhana, Bhratri, Matri) | Sun and Moon | JHora | no | JHora | P2 | `panchanga` |
| dispositor chains and final dispositors | lordships | | no | Western tools, some Vedic | P1 | `relationship` |

## G. Relationships

| feature | inputs | variants | baseline | field | tier | module |
|---|---|---|---|---|---|---|
| graha drishti (Parashari): full aspect on the 7th, three-quarter on 4th and 8th, half on 5th and 9th, quarter on 3rd and 10th, with the special full aspects of Mars (4, 8), Jupiter (5, 9), Saturn (3, 10) | house counts | Rahu and Ketu aspects: none, or 5/7/9, or 3/7/11 (schools) | yes | all | P0 | `aspect` |
| sphuta drishti (degree-based aspect value used in Drik Bala) | longitudes | BPHS formula with the Mars, Jupiter and Saturn special cases | yes | JHora, PyJHora | P0 | `aspect` |
| rashi drishti (Jaimini): movable to fixed except adjacent, and so on | signs | | yes | JHora, PyJHora, Kala | P0 | `jaimini` |
| argala and virodha argala | positions in 2, 4, 11 (and 5) and their obstructors in 12, 10, 3 (and 9) | Rahu and Ketu reversed; strength comparison rules | partial | JHora, PyJHora, Kala | P1 | `jaimini` |
| conjunction and mutual aspect detection with orb-less sign logic | signs | | yes | all | P0 | `aspect` |
| Parivartana (exchange) classification: Maha, Khala, Dainya | lordships | | yes (yoga rules) | all | P0 | `rules` |
| Tajika aspects: ithasala, ishrafa and the sixteen yogas with orbs (deeptamsha) | annual chart | | yes (partial) | JHora, PyJHora, Kala | P0 | `tajika` |
| Western aspects with orbs | see `15-western-modern.md` | | partial (synastry) | Western tools | P1 | `western` |

## Closing checklist for the design phase

- Confirm upagraha offsets and the Gulika versus Mandi convention against
  BPHS and Jataka Parijata; JHora and the baseline engine may differ.
- Confirm the arudha exception rules and the co-lord strength rules.
- Enumerate every varga variant JHora offers and decide which are P0.
- Combustion orbs confirmed against the Surya Siddhanta (IX.6 to 8,
  X.1; `astro::visibility::Thresholds::SuryaSiddhanta`); the
  settings-selectable table with other texts' values remains for the
  chart layer.
- Confirm the three Varnada methods and the Sayanadi sub-state formulas.
- Decide the Rahu/Ketu aspect default per profile (the baseline engine: none).
