# Strengths and measures

Status: `research`, 2026-09-04. Checked against the baseline engine's Shadbala,
Ashtakavarga and Bhava Bala services, JHora's features page and PyJHora's
bala list. Formulas are from BPHS and standard commentaries; items marked
**verify** need confirmation against a reference implementation.

## Shadbala (six-fold strength)

| component | sub-components | inputs | variants | baseline | tier |
|---|---|---|---|---|---|
| Sthana Bala (positional) | Uchcha (exaltation), Saptavargaja (dignity across seven vargas), Ojayugma (odd-even sign and navamsa by gender), Kendradi (kendra 60, panapara 30, apoklima 15), Drekkana (by gender and drekkana third) | D1 and six vargas, dignities | Saptavargaja dignity weights (30, 22.5, 15, 7.5, 3.75, 1.875 by relationship) **verify** | yes | P0 |
| Dig Bala (directional) | strength by distance from the house of full strength (Sun and Mars 10th, Moon and Venus 4th, Mercury and Jupiter 1st, Saturn 7th) | cusps (Bhava-Chalit) | measured from cusp midpoints versus cusp starts | yes | P0 |
| Kala Bala (temporal) | Nathonnatha (day/night), Paksha (waxing/waning with benefic/malefic), Tribhaga (thirds of day and night), Abda (year lord), Masa (month lord), Vara (weekday lord), Hora (hour lord), Ayana (declination based), Yuddha (planetary war) | birth time, sunrise, weekday, year and month lords from Kali ahargana | Abda and Masa lord derivation (Kali ahargana formulas) **verify**; Ayana bala formula with the 24° declination normalisation | yes | P0 |
| Cheshta Bala (motional) | from the mean and true longitudes and speed (Sun and Moon use Ayana and Paksha respectively) | mean positions: needs a mean-motion model | Cheshta from anomaly (BPHS) versus from speed fraction (simplified) | yes | P0 |
| Naisargika Bala (natural) | fixed: Sun 60, Moon 51.43, Venus 42.86, Jupiter 34.29, Mercury 25.71, Mars 17.14, Saturn 8.57 | | | yes | P0 |
| Drik Bala (aspectual) | sphuta drishti from benefics minus malefics, quartered | positions, aspect values | | yes | P0 |
| totals | in shashtiamsas and rupas; minimum required per planet (Sun 390, Moon 360, Mars 300, Mercury 420, Jupiter 390, Venus 330, Saturn 300) and ratio tiers | | | yes | P0 |
| Ishta and Kashta phala | from Uchcha and Cheshta balas: ishta = sqrt(uchcha × cheshta) | | | yes | P0 |
| Bhava Bala | Bhavadhipati (lord's Shadbala), Bhava Dig (sign type by house), Bhava Drishti (aspects on the cusp), plus optional Bhava Kala (day/night and the 1/7/10/4 conventions) **verify** | Bhava-Chalit cusps | | yes | P0 |

Rahu and Ketu have no Shadbala in the classical scheme; some software gives a
proxy. The SDK reports "not applicable" rather than a proxy unless a profile
selects one.

## Ashtakavarga

| feature | inputs | variants | baseline | tier |
|---|---|---|---|---|
| Bhinna Ashtakavarga (BAV) for seven planets and the lagna, 8 contributors each | positions | the lagna's own BAV included or not; the benefic-point tables per BPHS | yes | P0 |
| Sarvashtakavarga (SAV) | BAVs | with or without the lagna BAV | yes | P0 |
| Trikona Shodhana and Ekadhipatya Shodhana | BAV | Ekadhipatya: classical, zero, transfer (the baseline engine offers three) | yes | P0 |
| Shodhya Pindas: Rashi Pinda, Graha Pinda, Yoga Pinda | shodhita BAV | multipliers per sign and planet | yes | P0 |
| Kakshya (eight sub-divisions of a sign with kakshya lords) for transit scoring | BAV | | yes (transit strength) | P0 |
| Prastara Ashtakavarga (contributor by contributor) | | | partial | P0 |
| longevity and timing applications (Ashtakavarga dasha, experimental in PyJHora) | | | no | P2 |

## Vimshopaka and varga-based measures

| feature | inputs | variants | baseline | tier |
|---|---|---|---|---|
| Vimshopaka Bala over Shadvarga, Saptavarga, Dashavarga and Shodashavarga weight sets | varga dignities | weight tables per set; dignity multipliers (own 20, great friend 18, friend 15, neutral 10, enemy 7, great enemy 5, and exaltation and mooltrikona handling) **verify** | verify | P0 |
| Vaiseshikamsa names (Parijatamsa, Uttamamsa, Gopuramsa, Simhasanamsa, Paravatamsa, Devalokamsa, Brahmalokamsa, Airavatamsa, Sridhamamsa and so on) for counts of own or exalted placements per varga set | varga placements | thresholds differ per set | no | P1 |

## Tajika balas

| feature | inputs | variants | baseline | tier |
|---|---|---|---|---|
| Pancha Vargeeya Bala: Kshetra, Uchcha, Hadda (terms), Drekkana, Navamsa | annual chart | Hadda table (Egyptian terms as used in Tajika) | partial | P0 |
| Dwadasha Vargeeya Bala | twelve vargas | | no | P1 |
| Harsha Bala | house, sex of sign, day/night | | partial | P0 |
| Lord of the year, month and 60-hour chart from the five candidates and their balas | | | partial | P0 |

## Western measures (cross-reference)

Essential dignity scores (Ptolemaic, Egyptian, Dorothean tables), almutens
(Ibn Ezra, chart almuten), Astrodynes, temperament weightings: see
`16-hellenistic-medieval.md`. These reuse the dignity engine with different
tables, which is why dignity tables are data in the SDK.

## Closing checklist

- Reproduce the baseline engine's Shadbala on ten charts as golden vectors and reconcile
  every sub-bala against JHora's printout to find deliberate differences.
- Decide the mean-longitude model for Cheshta Bala (needs a mean-motion
  provider or a built-in table; note this is an SDK-side computation that
  the ephemeris port does not supply).
- Record the Vimshopaka weight tables with citations.
- Ekadhipatya default per profile: the baseline engine uses classical.
