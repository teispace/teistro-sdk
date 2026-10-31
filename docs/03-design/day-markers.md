# Day markers and Muhurta Chintamani's yoga tables

Status: `built`, 2026-10-31. Written before the code, from the texts read
for the application's SR-21 (addendum 3): ganda mula, vinchudo, Ravi and
Guru Pushya, and the special yogas (Amrit Siddhi, Sarvartha Siddhi,
Dwipushkar, Tripushkar) as day facts, each from a cited table.

## 1. What was asked, and what a day already had

A day already carried the muhurta yogas (`panchanga-day.md` §12) under
`panchanga.muhurta_tables = CLASSICAL`: a vara-and-star yoga read at the
star of sunrise, which the conformance corpus measured, and a pushkara
yoga at the tithi and star of sunrise. Two things were missing:

1. the yogas the full tables give over the whole day, with Guru and Ravi
   Pushya, which an almanac prints by the hour and the classical reading
   cannot, since it stops at sunrise;
2. the markers an almanac prints over a stretch from the Moon alone:
   ganda mula, gandanta and vinchudo.

Gandanta existed only as a chart rule at an instant
(`classical-gandanta.json`, BPHS ch. 92), not as a day span.

## 2. Muhurta Chintamani's tables (`MUHURTA_CHINTAMANI`)

A second value of the existing string knob `panchanga.muhurta_tables`,
so no knob, profile or hash changes; `CLASSICAL` stays the default and
the corpus's measurement stands. Under `MUHURTA_CHINTAMANI` every yoga
holds while its limbs do inside the day:

| Yoga | Source | Rule |
|---|---|---|
| Amrit Siddhi | Shubhashubha-prakarana vv. 20–21 (1954, p. 20), Narada's pairs as the Piyushadhara quotes them | Sun Hasta, Mon Mrigashira, Tue Ashwini, Wed Anuradha, Thu Pushya, Fri Revati, Sat Rohini; **lost while** the tithi 5, 6, 7, 8, 9, 10, 11 (in either paksha) of the same vara runs, *varjayet sarvakāryeṣu hastārkaṁ pañcamītithau …* |
| Sarvartha Siddhi | vv. 28–29 (1954, p. 22) | the table as printed (§2.1) |
| Guru Pushya | vv. 21–22 (p. 20), *gurupuṣya* | Pushya on a Thursday; v. 22 bars it for marriage, which the reader judges |
| Ravi Pushya | v. 28's Sunday stars; the name is the almanacs' | Pushya on a Sunday (crux C1050) |
| Dwipushkar, Tripushkar | Nakshatra-prakarana v. 49 (p. 86) | a Bhadra tithi (2, 7, 12) on a Sunday, Tuesday or Saturday in a star a sign boundary cuts: in two padas a side Dwipushkar, three and one Tripushkar |

Each yoga is one span per star, clipped to the day, and the list is in
the order the spans begin. An Amrit Siddhi a losing tithi cuts is two
spans either side of it.

### 2.1 Sarvartha Siddhi

The Piyushadhara says no rule of the Anandadi yogas generates this
table, so it is transcribed: Sunday Hasta, Mula, the three Uttaras,
Pushya, Ashwini; Monday Shravana, Rohini, Mrigashira, Pushya, Anuradha;
Tuesday Ashwini, Uttara Bhadrapada, Krittika, Ashlesha; Wednesday
Rohini, Anuradha, Hasta, Krittika, Mrigashira; Thursday Revati,
Anuradha, Ashwini, Punarvasu, Pushya; Friday Revati, Purva Phalguni,
Ashwini, Punarvasu, Shravana; Saturday Shravana, Rohini, Swati.
Friday's *bhāgya* (Purva Phalguni) is the printed text's; the 1954
edition's footnote reads *maitra* (Anuradha), and the 1906 print's gloss
keeps Purva Phalguni. The text ships; the variant is in crux C1050.

### 2.2 The pushkara rule is a generator

The verse names the stars by their feet: *dvipāda* and *tripāda*, stars
a sign boundary leaves two padas or three on one side. Each star is four
padas and each sign nine, so star *n* (from Ashwini as 0) is cut where
4*n* mod 9 is 6, 7 or 8: 7 leaves two padas each side (Mrigashira,
Chitra, Dhanishtha: Dwipushkar), 6 and 8 three and one (Krittika,
Punarvasu, Uttara Phalguni, Vishakha, Uttarashadha, Purva Bhadrapada:
Tripushkar). `omen::pushkara_of` is that one line, and a test counts the
padas star by star.

The `CLASSICAL` table carried Anuradha among Dwipushkar's stars.
Anuradha lies whole in Scorpio, so no sign boundary cuts it and the
verse cannot name it; the generator drops it under both tables. This
moves a number: a Bhadra tithi on a Sunday, Tuesday or Saturday at
sunrise in Anuradha no longer reads Dwipushkar (C1050, CHANGELOG).

## 3. The day's markers (`markers`)

A new catalogue kind `day_marker` (79), each member a span over the day
in the order they begin, read only from spans the day already holds: its
stars and the Moon's signs.

| Marker | Source | Span |
|---|---|---|
| `GANDA_MULA` | the junctions: Muhurta Chintamani, Vivaha-prakarana v. 41, BPHS ch. 92; the whole star is the almanacs' | each of Ashwini, Ashlesha, Magha, Jyeshtha, Mula and Revati the day held, the whole star |
| `GANDANTA` | Vivaha-prakarana v. 41, *ghaṭikāyugmaṁ … ghaṭikādvayam* | two ghatis (a ghati a sixtieth of a day, 48 minutes for the two) each side of each Revati–Ashwini, Ashlesha–Magha and Jyeshtha–Mula junction, one span a junction, so the same span on each day it reaches |
| `VINCHUDO` | Gujarati almanacs (વીંછુડો); no text read gives it | while the Moon is in Scorpio, from Vishakha's last pada to Jyeshtha's end |

### 3.1 Which readings are printed

Ganda mula's extent differs by source: v. 41 marks only two ghatis at
the junction (gandanta), Nakshatra-prakarana vv. 52–53 an *abhukta
mūla* of 4 + 4 ghatis by Narada, 1 + 2 by Vasishtha, 1 + 1 by
Brihaspati and 5 + 8 by others, and BPHS 92.5 six and eight. The
almanacs mark the whole of each of the six stars, and that is what an
application prints, so `GANDA_MULA` is the whole star and `GANDANTA` the
text's two ghatis; the abhukta mula variants are not shipped and wait
in crux C1051. Vinchudo is printed with no text behind it. Both members
carry the catalogue attribute `printed: true` and the mark `T`.

They are not knob-chosen conventions (`printed-conventions.md`): each
has one printed form, nothing chooses between forms, and a day carries
them only when asked, so the asking is the choice. A second form found
in print becomes a knob then.

### 3.2 Asked, not computed by default

The markers cost a pass over the day's stars and the Moon's signs, but
they add a section to every day's answer and change its document and
hash. A caller who does not ask pays nothing:
`AlmanacRequest::with_markers`, `"markers": true` in the JSON days
request (bindings and the agent server's `almanac.days`), and
`TS_PANCHANGA_MARKERS` (128) at the C boundary, which writes the span
section `markers` (41). `Panchanga::markers` is empty and left out of
the day's document unless asked; `Panchanga::mark` fills it.

## 4. Tests

- `omen` unit tests: the pushkara generator against the padas of each
  of the 27 stars; `without` over cuts inside, across, over and outside
  a span; a Thursday in Pushya under `MUHURTA_CHINTAMANI` (Amrit Siddhi
  lost in the Navami, Guru Pushya, Punarvasu's Sarvartha before it) and
  the same sky on a Sunday (Ravi Pushya); a pushkara after sunrise the
  classical reading misses; gandanta the same span from either side of
  a junction; vinchudo exactly the Moon's Scorpio.
- `crates/sdk/tests/day_markers.rs`: a quarter at Kathmandu, each
  gandanta read back from a chart at its middle (the Moon at 0°, 120° or
  240° sidereal, within a degree for parallax), ganda mula and vinchudo
  against the day's stars and signs; a day not asked is unchanged and
  hashes as before; half a year under `MUHURTA_CHINTAMANI` gives Guru
  Pushya on every Thursday in Pushya and the classical tables none.
- The C boundary's bit, and every binding's option and getter through
  each parity runner's `markers` line (11 June 2025, members only).
