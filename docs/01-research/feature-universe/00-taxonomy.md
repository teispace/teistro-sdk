# The feature universe: a taxonomy

Status: `research`, 2026-09-04.

Astrology software across traditions computes a surprisingly small number of
kinds of thing. Every product feature in the competitive matrix falls into one
of the fifteen capability areas below, and the areas, not the traditions, are
what the SDK's modules follow. A tradition is then a profile: a selection of
variants and rule packs over shared machinery. This is the single most
important structural observation of the research, because it is what lets a
Vedic SDK become a general one without a rewrite.

## The fifteen capability areas

| # | area | what it is | examples across traditions |
|---|---|---|---|
| A | astronomy inputs | what the ephemeris port supplies | positions and speeds, nodes, apogees, cusps, sidereal time, obliquity, rise and set, eclipses, crossings, fixed stars, asteroids |
| B | frames and references | how positions are expressed | tropical or sidereal with an ayanamsha, geocentric or topocentric, ecliptic, equatorial and horizontal coordinates, house systems |
| C | time and calendar | when | Julian day and timescales, timezones and local mean time, civil calendars, eras, lunar months, the sunrise-anchored day, ghati-pala, planetary hours |
| D | chart construction | which chart | natal, transit, prashna or horary, event, annual and monthly returns (Tajika, Tithi Pravesha, solar and lunar returns), composite and Davison, relocated, harmonic, divisional |
| E | derived points | computed points that behave like bodies | special lagnas, upagrahas, arudhas, sahamas and lots, sphutas, midpoints, Arabic parts, Uranian points, fixed stars projected to the ecliptic |
| F | qualification and state | what condition a body is in | dignity, avastha, combustion, retrogression, planetary war, sect, term and face rulers, vargottama, gandanta |
| G | relationships | how bodies relate | graha drishti, rashi drishti, argala, Ptolemaic aspects with orbs, parallels, Tajika aspects, receptions, dispositors |
| H | strength and measure | how much | Shadbala, Ashtakavarga, Bhava Bala, Vimshopaka, Tajika balas, essential dignity scores, almutens, Astrodynes |
| I | time-lord systems | which period | nakshatra and rashi dashas, conditional dashas, Tajika annual dashas, profections, zodiacal releasing, firdaria, decennials, progressions and directions as time keys |
| J | pattern rules | which combinations hold | yogas, doshas, Western configurations, horary considerations, matching kootas, muhurta event rules, remedy triggers |
| K | transits and timing | what is happening now | gochar with vedha, Sade Sati, transit calendars and hit lists, ingresses, stations, eclipses, void-of-course Moon |
| L | electional | when to act | panchanga shuddhi, tara and chandra bala, blackouts, lagna and karaka evaluation, published auspicious days |
| M | compatibility | who with whom | Ashta Koota, Dasha Koota, Mangal matching, synastry, composite |
| N | diagnostics and applications | specialised procedures | rectification, longevity, remedies, numerology, Lal Kitab, Pancha Pakshi, namakarana, rashifal, statistics and research |
| O | interpretation and presentation | how it is said and shown | state text, rule text, composers, dossier serialisation, chart geometry for drawing, reports |

## Traditions as profiles over the areas

| tradition | distinctive areas | shares with others |
|---|---|---|
| Vedic Parashari | E (upagrahas, special lagnas), F (avasthas), G (graha drishti), H (Shadbala, Ashtakavarga), I (nakshatra dashas), J (yogas, doshas) | A, B, C, D, K, L, M, N, O |
| Jaimini | E (arudhas, karakas), G (rashi drishti, argala), I (rashi dashas) | everything else Parashari |
| KP | B (Placidus cusps), E (sub-lords), N (ruling planets, horary numbers) | Vimshottari from I |
| Tajika | D (annual chart, muntha), G (Tajika aspects and yogas), H (pancha vargeeya bala), I (Mudda, Patyayini) | Parashari chart machinery |
| Nadi and Bhrigu | J (rule packs keyed on nakshatra and degree), O (large corpora) | Parashari positions |
| Western modern | B (tropical), G (aspects and orbs), E (midpoints, Arabic parts, fixed stars), I (progressions, directions, returns), K (hit lists), M (synastry) | A, C, D, O |
| Hellenistic and medieval | F (sect, terms), H (dignity scores, almutens), E (lots), I (time lords), J (horary considerations) | Western machinery |
| Uranian and cosmobiology | E (transneptunian points, midpoints), G (dials, harmonics) | Western machinery |
| Chinese, Tibetan, Burmese | C (their calendars), D (pillar charts rather than ecliptic charts), I (luck pillars) | little; they are a v2 study |

The consequence for architecture: the chart data model must carry the
tropical and sidereal frames together, the aspect model must be pluggable
(orb-based and sign-based), the time-lord registry must accept both
nakshatra-seeded and rashi-seeded and event-seeded systems, and the rule
engine must be tradition-neutral. Each of those is specified in
`02-architecture/`.

## Feature pages

| page | areas | tier of most content |
|---|---|---|
| [`01-vedic-parashari-core.md`](01-vedic-parashari-core.md) | A, B, E, F, G | P0 |
| [`02-strengths.md`](02-strengths.md) | H | P0 |
| [`03-dashas.md`](03-dashas.md) | I | P0 for the baseline engine's eighteen, P1 for the rest |
| [`04-yogas-doshas.md`](04-yogas-doshas.md) | J | P0 |
| [`05-jaimini.md`](05-jaimini.md) | E, G, I | P0 partial |
| [`06-kp.md`](06-kp.md) | B, E, N | P0 |
| [`07-tajika-varshaphala.md`](07-tajika-varshaphala.md) | D, G, H, I | P0 partial |
| [`08-panchanga-calendar-muhurta.md`](08-panchanga-calendar-muhurta.md) | C, L | P0 |
| [`09-prashna-horary.md`](09-prashna-horary.md) | D, J, N | P0 partial |
| [`10-matching.md`](10-matching.md) | M | P0 |
| [`11-transits-gochar.md`](11-transits-gochar.md) | K | P0 |
| [`12-chakras-special-charts.md`](12-chakras-special-charts.md) | D, O | P1 |
| [`13-rectification-longevity.md`](13-rectification-longevity.md) | N | P0 |
| [`14-remedies-numerology-misc.md`](14-remedies-numerology-misc.md) | N | P0 |
| [`15-western-modern.md`](15-western-modern.md) | B, E, G, I, K, M | P1 |
| [`16-hellenistic-medieval.md`](16-hellenistic-medieval.md) | E, F, H, I, J | P1 |
| [`17-other-traditions.md`](17-other-traditions.md) | C, D, I | P2 |
| [`18-interpretation-rendering.md`](18-interpretation-rendering.md) | O | P0 |

## Counting the universe

To size the work, the features were counted per area after de-duplicating
across products (the same dasha under two names is one feature; a variant is
counted under its feature). The count is a research figure, not a gate, and it
will be superseded by the module catalogue.

| area | distinct features | with named variants |
|---|---:|---:|
| A astronomy inputs | 14 | 6 |
| B frames | 6 | 4 |
| C time and calendar | 18 | 9 |
| D chart construction | 16 | 5 |
| E derived points | 22 | 8 |
| F state | 12 | 6 |
| G relationships | 9 | 5 |
| H strength | 11 | 4 |
| I time lords | 68 | 20 |
| J pattern rules | 8 engines over ~900 rules | 8 |
| K transits | 12 | 3 |
| L electional | 10 | 4 |
| M compatibility | 6 | 3 |
| N diagnostics | 14 | 6 |
| O presentation | 9 | 2 |

The baseline engine covers roughly the P0 column of each page; the largest gaps against
the field are in I (dasha systems beyond eighteen), D (returns and pravesha
charts beyond Tajika), E (arudha variants, sahamas, lots), K (transit hit
lists as a first-class search) and everything Western.
