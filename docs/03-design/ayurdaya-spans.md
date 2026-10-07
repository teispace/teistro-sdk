# The remaining spans of life (*Jataka Parijata* ch. 5)

Status: Rasmi, v. 33's choice, Dasayus and Chakrayus `decided` and
built, 2026-10-07; the two ashtakavarga spans `research`. The
three spans BPHS and *Jataka Parijata* share (Pindayu, Nisargayu,
Amsayu) are built (`rules-engine.md`), and are read as the book reads
them under `AyurdayaRules::PARIJATA` (C302 to C304). This page records
what the chapter gives beyond them, and the worked figures each must be
held to before code.

## The source

*Jataka Parijata* vol. I, V. Subrahmanya Sastri's translation (1932),
public domain, on the Internet Archive as
`JatakaParijataVolIOfIIByVSubrahmanyaSastri`. Adhyaya V, "Length of
Life", runs from book p. 233. The `_text.pdf` page is the book page plus
25. Its OCR loses tables and figures, so the pages are read rendered
(`pdftoppm -r 150`). vv. 7, 9, 11, 17 to 21 are taken from *Brihat
Jataka*, as the chapter's first note says.

## What the chapter gives

- **v. 1**: eight ayurdayas: Nisarga, Pinda, Amsa, Rasmi, Chakra,
  Nakshatra, Daya and Ashtakavarga.
- **vv. 2 to 16**: Nisarga's and Pinda's years, the reductions and the
  lagna's years. These are built.
- **vv. 17 to 21**: Amsa. Jeevasarman gives each planet a seventh of
  120 years 5 days at exaltation, and is used when the lagna, the Sun and
  the Moon are all weak. Satya (the majority) gives a year a navamsha
  from Aries less twelves (built), multiplied threefold in exaltation or
  retrograde and twofold in vargottama, own navamsha, own sign or own
  drekkana, only the highest multiplication and only the greatest
  reduction applying, with no krurodaya. The Satya multiplication is
  built as `amsayu_multiplied`. **Open:** Jeevasarman's Amsa, and
  whether a retrograde graha triples (v. 19 says so; the built knob
  does not).
- **vv. 22 to 25**: Rasmija. Rays at deep exaltation are Sun 10, Moon 9,
  Mars 5, Mercury 5, Jupiter 7, Venus 8 and Saturn 5. v. 24 doubles them
  in the planet's own rasi, its exaltation, a very friendly house or
  retrograde. They lose an eighth as retrogression ends and a twelfth in
  an enemy's house. v. 25 halves them when eclipsed, except Venus and
  Saturn.
- **v. 26**: Chakrayus, the untraversed portions of the nakshatra padas
  the planets occupy, reckoned through ch. 17 v. 6's Kalachakra years.
  Its figure (p. 256) is the Sun at 1s 2° 55′ 30″, 2.653 years.
- **v. 27**: Dasayus (Nakshatra dasayus): the unexpired dasa at birth
  and the dasas after it, in ch. 18 v. 3's years. Also the Ashtakavarga
  ayus, which ch. 10 treats.
- **v. 28** (Manittha): the lagna lord strong with a benefic aspect →
  Amsa; the Sun → Pinda; the Moon → Nisarga.
- **vv. 29 to 32** (Parasara): the yogas in which Pinda is to be used.
- **v. 33**: the span is chosen by whichever of the Sun, the Moon,
  Mercury, Mars, Venus, Jupiter, Saturn and the lagna is strongest:
  Pinda, Nisarga, Rasmi, Bhinnashtakavarga, Kalachakra, Nakshatra,
  Samudaya (ashtakavarga) and Amsa respectively.
- **v. 34**: years of 360 days × 360/365 are solar years.

## The Rasmi crux

The chapter scales rays two ways on facing pages.

- **v. 23 and its own figure (p. 254)** count the arc from exaltation,
  subtract it from 12 signs when under 6, and scale the rays by the
  12 signs. A planet keeps half its rays at debilitation, as Pindayu
  keeps half its years. The Sun at 1s 2° 55′ 30″ gives
  11s 7° 4′ 30″ / 12 × 10 = 9.363.
- **The translator's note (pp. 250 to 254)**, after Mahendra, Manittha,
  Maya, Yavana and Badarayana, makes rays **zero at debilitation**, in
  proportion between. It modifies them by the **dwadasamsa** rather than
  the sign: an own or friendly one doubles, an enemy's takes a sixteenth.
  It works a full figure, "a distinguished personage", with every
  graha's Shadbala:

  | graha | longitude | rupas | rays | facing |
  |---|---|---|---|---|
  | Sun | 1s 2° 55′ 30″ | 8.154 | 8.7264 | parangmukha |
  | Moon | 11s 23° 35′ 24″ | 7.289 | 6.5902 | abhimukha, a sixteenth lost |
  | Mars | 3s 24° 1′ 26″ | 7.354 | 0.2207 | parangmukha, doubled |
  | Mercury | 0s 13° 10′ 48″ | 7.550 | 1.5655 | abhimukha, doubled |
  | Jupiter (retrograde) | 6s 25° 13′ 23″ | 5.678 | 5.3882 | parangmukha, doubled |
  | Venus | 2s 18° 15′ 50″ | 7.719 | 8.7765 | parangmukha, doubled |
  | Saturn | 0s 17° 59′ 38″ | 5.053 | 0.0557 | parangmukha |
  | lagna | 7s 15° 47′ 24″ | 7.345 | | |

  The total is 31.3232. The note reads it against Jatakadesa's bands:
  over 25 rays long life, 15 to 25 medium, under 15 short.

The two scales answer different questions. The note counts **rays**,
which grade a life by bands; v. 23 counts **Rasmijayus years**, which
sum like Pindayu. Neither is to be read as the other.

**Decided (C305 to C307) and built.** `Evaluator::rasmi(RasmiRules)`
reports every graha's `rays` (nothing at debilitation) and `years`
(half there), the same doubling and shares applied to both, the rays'
`total` with its `class` by Jatakadesa's bands, and the years' sum. A
rule request carries it in `longevity.rasmi`, and `rasmi.place` picks
the dwadasamsa (default) or the sign (`RasmiRules::VERSE`). Three
readings of the figure decide the rest:

- **Only a great friend doubles.** The note writes "a friendly planet",
  but calls the Sun's Gemini dwadasamsa "a neutral planet's" though
  Mercury, twelfth from him, is a compound friend, and Venus's
  Capricorn dwadasamsa "a very friendly planet's". v. 24 says "a very
  friendly house". Enmity is compound too: the Moon loses a sixteenth
  in Jupiter's Sagittarius, Jupiter being neutral to her by nature and
  eighth from her.
- **The book's own slips.** Its table prints Jupiter at 6s 25° 13′ 23″
  and its working at 6s 25° 43′ 23″, whose rays it gives; and it works
  Mars's .132537 / 6 × 5 as .1103475 for .110448. With the working's
  Jupiter, every ray reproduces to the fourth place the book truncates
  to, Mars's to the product, and the total is 31.3234 for 31.3232.
- **Not read:** the eighth lost "when the retrograde motion of a planet
  is about to cease", which needs a speed a rule chart does not carry.

## Order of work

1. ~~Rasmi: the rays with the note's figure, and Rasmijayus with v. 23's.~~
   Built (C305 to C307).
2. ~~v. 33's choice, reported as which body is strongest and the span it
   names, never a single verdict, as v. 28 and BPHS's choice are.~~
   Built (C308): `longevity.choice` weighs the seven grahas' Shadbala and
   the lagna's Bhava bala, both in rupas, and reports every candidate,
   the strongest, the span it names and its years where computed. Over
   the corpus's 53 charts it names Amsa 12 times, Rasmi 10, Pinda,
   Nisarga and Nakshatra 7 each, Samudaya 6, Kalachakra 3 and
   Bhinnashtakavarga once: 17 charts name a span not yet computed.
   **Found:** a longevity request asked for no Shadbala unless a rule in
   the set read strength, so the visible half's "strongest of several"
   compared nothing and every one of the several lost; it now asks.
3. ~~Dasayus over the dashas the SDK already computes.~~ Built (C309):
   ch. 18 v. 3, read in vol. II, gives Vimshottari's lords and years, so
   `longevity.dasayus` is the chart's own Vimshottari balance and the
   eight dashas after it, 100 to 120 years. v. 33's Nakshatra candidate
   carries it, and 43 of the corpus's 53 charts now name a computed
   span.
4. ~~Chakrayus, once ch. 17's Kalachakra years are read.~~ Built
   (C310): ch. 17 v. 6, read in vol. II, gives the seven's years, which
   are the dasha crate's Kalachakra sign years by each graha's own sign.
   The verse names the untraversed pada, but the note's one figure takes
   the Sun's whole star and his own years (424.5 × 5 / 800 = 2.653), so
   `chakrayus.portion` is the star by default and the pada by choice.
   With it, 46 of the corpus's 53 charts name a computed span under
   v. 33; the 7 left name Samudaya (6) or Bhinnashtakavarga (1).
5. ~~The two ashtakavarga spans (v. 27's last clause, "treated in ch.
   10").~~ Built (C311 to C314): `longevity.ashtakavarga` gives each
   graha's years from its own reduced ashtakavarga, their sum
   (Bhinnashtakavargaja, 24 to 58 years over the corpus) and Samudaya
   (1.4 to 108.7). Every one of the corpus's 53 charts now names a span
   the SDK computes.

## The ashtakavarga spans (ch. 10, vol. II)

v. 33's Mars names Bhinnashtakavargaja and Saturn Samudaya; v. 27 says
the ashtakavarga ayus is "treated in ch. 10". Volume II (public domain,
the Internet Archive's page images, `page/nNN_w1400.jpg`; the book page
is the image number plus 620) gives both.

- **vv. 44 and 45 (p. 689), each graha's years.** In the graha's own
  ashtakavarga after the trine and the single-lord reductions (vv. 39 to
  42), multiply each sign's bindus by its sign's measure, Aries to
  Pisces 7, 10, 8, 4, 10, 5, 7, 8, 9, 5, 11, 12, and the bindus in the
  signs the seven occupy by the occupant's measure, the Sun to Saturn 5,
  5, 8, 5, 10, 7, 5. Divide the sum by 30; a quotient over 12 casts out
  its twelves. The figure (p. 690): the Sun's reduced ashtakavarga gives
  162 by the signs and 90 by the grahas, 252 / 30 = 8.4 years.
  **Virgo's measure is 5** where BPHS's translator, and the SDK's pinda,
  read 6, and the recording engine 8.
- **The note (p. 690):** Balabhadra and Mantreswara multiply the same sum
  by 7 and divide by 27, casting out twenty-sevens, and reduce it: half
  for another graha in the bhava, debilitation or the Sun's rays; a
  third in an enemy's house or the visible half; the greatest only.
- **v. 46 (p. 691):** doubled in exaltation, halved in debilitation or
  eclipsed, in proportion between; Mars retrograde doubled.
- **v. 48:** the seven's years summed are the span; some add the
  lagna's, from its own ashtakavarga.
- **v. 50 (p. 693):** another Bhinnashtakavarga span, the bindus in the
  signs the seven occupy, "subject to the reductions mentioned already".
- **v. 70 (p. 705):** the sarvashtakavarga's trine and single-lord
  reductions, then each sign's figure less its twelves, a figure of
  exactly 12 kept.
- **v. 71 (p. 706), Samudaya:** those figures times the same sign and
  graha measures, the sum times 7 over 27; "if in excess of the standard
  Ayus, i.e. 100 years, should be diminished by 100 years". No worked
  figure.

Decided:

- **Virgo 5 (C311).** The book's own table, which its figure uses; BPHS's
  translator's 6, the SDK's pinda, by choice (`measures: "bphs"`).
- **Over 30 (C312).** The verse's, which the figure works; the note's
  Balabhadra and Mantreswara 7 over 27 by choice
  (`divisor: "seven-over-twenty-seven"`). The note's reductions are not
  read with either: they are Balabhadra's, and the verse's v. 46 is.
- **v. 46 alone (C313).** Twice at exaltation, half at debilitation,
  linear in the arc from debilitation between, as `by_exaltation`
  measures it; an eclipsed (combust) graha halved instead and retrograde
  Mars doubled. The lagna's years (v. 48's "some") and v. 50's other
  span are not read.
- **A hundred taken off once (C314).** v. 71 diminishes a product "in
  excess of" 100 by 100. Over the corpus the products run from 40.7 to
  208.7 years: 19 are under 100, 32 lose the hundred, and 2 exceed 200
  and keep over 100 years. The verse says no more, so the span is
  literal and `samudaya_product` carries the product before it. v. 71
  calls the span Nakshatra Ayus and gives it in solar years by 324 over
  365 (p. 706): `samudaya_solar`.

## The years a span is counted in (ch. 5 v. 34, C315)

"The Ayus in years, months, etc, multiplied by 360 and divided by 365 is
termed Sourayus, i.e. the period of life in solar years" (p. 261); the
book's index calls the span "in years of 360 days". It follows v. 33, so
it is read for each span v. 33 names: every candidate carries
`solar_years` beside `years`. Samudaya is the exception its own verse
makes, counted in nakshatra years of 324 days and converted by 324 over
365. Balabhadra's note converts his 7-over-27 years the same way, after
his reductions; neither is read, so the note's divisor, when chosen, is
converted as v. 34 says.
