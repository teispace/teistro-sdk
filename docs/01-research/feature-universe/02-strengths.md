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

## Sripati's reading, as B.V. Raman works it (read 2026-09-15)

B.V. Raman's *Graha and Bhava Balas* (thirteenth edition, 1992; rank 2)
follows Sripati's *Paddhati* throughout and works every strength on one
Standard Horoscope — a female birth at 13° N, 77° 35′ E, 2h 6m 16s p.m. LMT
on 16 October 1918, with its nirayana longitudes, bhava madhyas, sunrise and
diurnal length given — so it is the one source whose reading of every
component can be checked number by number. Where it departs from the BPHS
translation read for the Shadbala (`03-design/shadbala-measured.md`):

- **Sphuta drishti** (Art. 114, quoting Sripati, "Parasara also gives the same
  rules"): with the aspect angle *k* = aspected − aspecting, the drishti is
  (*k* − 30)/2 from 30° to 60°, (*k* − 60) + 15 to 90°, (120 − *k*)/2 + 30 to
  120°, 150 − *k* to 150°, (*k* − 150) × 2 to 180°, (300 − *k*)/2 to 300°,
  and nothing otherwise; the special aspects add 15 for Mars (90°–120°,
  210°–240°), 30 for Jupiter (120°–150°, 240°–270°) and 45 for Saturn
  (60°–90°, 270°–300°). The worked Drishti Pindas (Example 54) reproduce it.
  This closes crux C45 at rank 2.
- **Drik bala** is a quarter of the net pinda, benefics less malefics, with no
  full term for Mercury and Jupiter (Art. 120).
- **Benefics** are Jupiter, Venus, the waxing Moon (from the eighth day of the
  bright half to the eighth of the dark) and a well-associated Mercury;
  Mercury conjunct or combust the Sun counts as malefic (Arts. 53, 117–118).
- **Saptavargaja**: 45 only for the moolatrikona *rasi* in the rasi chart; a
  moolatrikona sign in any other varga is the own sign's 30; then 22.5, 15,
  7.5, 3.75 and 1.875 by the compound relationship (Art. 30).
- **Drekkana**: male grahas in the first decanate, the hermaphrodites
  (Mercury, Saturn) in the middle one and the female grahas in the last
  (Art. 36) — the translation read puts female second and neuter third.
- **Dig** from the bhava madhyas of the kendras (Art. 44–45), which in the
  worked example are the angles.
- **Nathonnatha** from apparent midnight, the equation of time applied
  (Art. 48–51).
- **Abda, Masa and Vara lords** from the ahargana counted to and including
  the day of birth, 714,404,130,045 for the Standard Horoscope (Arts. 58–62)
  — one more than the SDK's elapsed count, which is the same weekdays.
- **Kranti** from the Hindu table of 362′, 341′, 299′, 236′, 150′ and 52′ a
  15° step of the sayana bhuja, a maximum of 24°, and the Ayana bala
  60 × (24° ± kranti)/48°, the Sun's doubled (Arts. 73–75).
- **Yuddha**: two of Mars to Saturn within a degree; the one of lesser
  longitude wins; the difference of their Sthana, Dig and Kaala (to the
  Hora) over the difference of their discs' diameters (Mars 9.4″, Mercury
  6.6″, Jupiter 190.4″, Venus 16.6″, Saturn 158.0″) is added to the winner's
  Kaala and taken from the loser's (Arts. 76–77).
- **Cheshta** from Kedarnath Dutt's mean elements at 0h, 1 January 1900,
  76° E (Tables IV–IX): the mean Sun 257.4568° at 0.98560265° a day; Mars
  270.22° at 0.524019°; Jupiter 220.04° at 0.0830967° less 3.33° + 0.0067 t;
  Saturn 236.74° at 0.033439° plus 5° + 0.001 t; Mercury's seeghrochcha 164°
  at 4.09232° plus 6.67° − 0.00133 t; Venus's 328.51° at 1.602147° less
  5° + 0.001 t (t the years since 1900). The mean longitude of Mercury and
  Venus is the mean Sun's, and the mean Sun is the seeghrochcha of Mars,
  Jupiter and Saturn; the kendra is the seeghrochcha less the mean of the
  mean and true longitudes, folded past 180° and divided by 3 (Arts. 87–107).
  The recording engine's inferior planets take their own seeghrochcha as
  their mean as well, which this refuses.
- **Required rupas**: the Sun 5, the Moon 6, Mars 5, Mercury 7, Jupiter 6.5,
  Venus 5.5, Saturn 5 (Art. 122) — the engine's, where BPHS asks the Sun 6.5.
- **Bhava bala** (Arts. 124–133) is three components only: the lord's
  Shadbala; the Bhava Digbala in ten-virupa house steps from the house the
  madhya's sign class makes weakest — Nara (Gemini, Virgo, Libra, Aquarius,
  the first half of Sagittarius) the seventh, Jalachara (Cancer, Pisces, the
  second half of Capricorn) the tenth, Chatushpada (Aries, Taurus, Leo, the
  second half of Sagittarius, the first half of Capricorn) the fourth, and
  Keeta (Scorpio alone) the first; and the Bhava Drishti bala, the sphuta
  drishti on each bhava madhya, Jupiter's and Mercury's in full and a
  quarter of every other graha's, Mercury always a benefic there. The
  translation read puts Cancer with Scorpio, scales the Dig by a quarter
  for a benefic or malefic drishti, and adds the occupants' rupa and the
  rising classes' fifteen virupas (vv. 30–31), none of which Raman does.

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
