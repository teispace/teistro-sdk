# The remaining spans of life (*Jataka Parijata* ch. 5)

Status: `research`, 2026-10-07. Nothing here is built yet. The three
spans BPHS and *Jataka Parijata* share (Pindayu, Nisargayu, Amsayu) are
built (`rules-engine.md`), and are read as the book reads them under
`AyurdayaRules::PARIJATA` (C302 to C304). This page records what the
chapter gives beyond them, and the worked figures each must be held to
before code.

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

## The Rasmi crux, to decide before code

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
sum like Pindayu. The proposal to decide on this page is that both
ship, each held to its own figure: the rays with their dwadasamsa
modifications and the bands as clauses, and Rasmijayus with v. 24's
doublings. Neither is to be read as the other.

## Order of work

1. Rasmi: the rays with the note's figure, and Rasmijayus with v. 23's.
2. v. 33's choice, reported as which body is strongest and the span it
   names, never a single verdict, as v. 28 and BPHS's choice are.
3. Dasayus over the dashas the SDK already computes.
4. Chakrayus, once ch. 17's Kalachakra years are read.
