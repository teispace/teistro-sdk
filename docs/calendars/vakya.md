# The vakya Moon, tithi and nakshatra

Status: `built`, 2026-11-09. `teistro_siddhanta::vakya` computes the
*Vakyakarana*'s true Sun and true Moon in whole arithmetic, the
almanacs' correction to the Moon (the manyadi), and the tithi and
nakshatra ends of a Kali day as nadis after sunrise. The `tamil-vakya`
profile (`Siddhanta::Vakya`) puts that sky under the panchanga and the
festival rules. The vakya month (`calendar.TAMIL_VAKYA`) is described in
[`regional-solar.md`](regional-solar.md#the-two-tamil-schools); this page
is its Moon.

## Sources

- *Vākyakaraṇa with the commentary Laghuprakāśikā by Sundararāja*, ed.
  T. S. Kuppanna Sastri and K. V. Sarma (Kuppuswami Sastri Research
  Institute, Madras, 1962), Internet Archive item
  `vakyakaranawithlaghuparkasikasudararajaed.kuppannasastrit.s.sarmak.v._865_t`.
  Read: I.4–5 (the Sun), I.6–8 (interpolation), I.9–14 (the Moon) in
  the translation, pp. 251–256, with Examples 2 and 3; Appendix II, the
  248 candravakyas, pp. 125–134, with the editors' check rule on p. 134;
  the introduction on the almanacs' manyadi, p. xvi, note 1; the errata,
  pp. 301–302.
- The *Manonmani Vilasam* suddha vakya panchangam (Manonmani Vilasam
  Press, Chennai), Internet Archive items
  `calw-manonmani-vilasa-suddha-vakkiya-panchangam-isvara-` (leaf 0024,
  Bahudhanya 1998-99, Ani 1–32; leaf 0300, Manmatha 2015-16, Aippasi
  1–15) and `bqfm-manonmani-vilasa-suddha-vakya-panchangam-sarvajitt`
  (leaf 0150, Durmukhi 1956-57, Chittirai 1–14). Its rows were read by
  eye from cropped page images into
  `crates/siddhanta/tests/data/manonmani-vakya.txt`.

The method below is the text's, re-expressed; the vakyas are numbers
read off the tables, not the editors' prose.

## The method

**Units.** An arc is held in quarter-seconds, 14,400 to the degree and
18,662,400 to the circle. A Kali day is held in quarter-vinadis (14,400
to the day), so the Sun's rule, a day for a degree, is the identity on
the two counts (a const assertion holds `ARC_PER_DEGREE ==
PARTS_PER_DAY`).

**The Sun** (I.4b–5a). The days since the vakya Mesha sankranti (the
year's machinery of `regional-solar.md`, reused) are read as degrees,
and a deduction in minutes is taken from them, interpolated by I.6–8's
rule *a* + (*b* − *a*)*x*/*c* between values given for every tenth day:
0, 14, 32, 54, 78, 105, 133, 163, 194, 224, 254, 284, 311, 335, 358, 376,
391, 403, 411, 415, 416, 412, 406, 398, 386, 374, 361, 347, 334, 322, 311,
303, 297, 295, 296, 301, 309, 322 (`SUN_DEDUCTIONS`). Example 2 (5s
23°28′33″) is a test. The Sun so reckoned reaches each sign's first
point within 1′ of the sankranti vakyas of I.4.

**The Moon** (I.9–14a). The Kali days are reduced by 16,00,984 (the
epoch `MOON_EPOCH_DAYS`). The remainder is divided by 12,372, the rest
by 3,031 and the rest by 248, the khandas; the last remainder *r*
indexes the candravakyas. The dhruva is 7s 2°0′7″ plus the quotients
times 9s 27°48′10″, 11s 7°31′1″ and 0s 27°44′6″, and the Moon is the
dhruva plus the *r*-th vakya. The text's correction adds (the day's
motion − 13°11′) in degrees times 32*q*₃ − 8*q*₂ seconds, the day's
motion being the vakya less the one before it (at *r* = 0 the 248th
less the 247th). Example 3 (Kali day 18,44,004: 4s 11°17′32″) is a
doctest of `true_moon`.

**The vakyas** (Appendix II). The 248 vakyas ship as printed in signs,
degrees and minutes (`CANDRA_VAKYAS`), each block cited by page. The
transcription is checked by the editors' own rule (p. 134): the vakyas
are symmetric about the cycle, *V*ₙ + *V*₂₄₈₋ₙ = *V*₂₄₈, and *V*₁₂₄ = 6s +
*V*₂₄₈/2; a test holds all 124 pairs, and the day's motion stays
between 11°50′ and 15°20′. No independent published list of the 248
was found to set the table against, so the second sources are the
edition's own: the symmetry, and the *katapayadi* sentence printed
beside each value. The symmetry fails at one pair; there the sentence
of day 148, *samprataṃ raviḥ*, reads 4s 26° 27′ where the figures are
printed 4 26 17, and 27 restores the pair, so it ships as 27 (C1290).

**The manyadi.** The almanacs add to the text's Moon a correction the
text does not give. The edition's introduction (p. xvi, n. 1) prints
its values at the beginnings of Rishabha, Mithuna, … Mesha as 15, 10,
7, 8, 11, 17, 23, 28, 30, 29, 26 and 21 minutes, and reads it as the
bhujantara, the carardha and the desantara of a place in the Chola
country with a bija of 26′, added about 1885. `almanac_moon` adds it,
interpolated by the Sun's place in its sign (`MANYADI_MINUTES`, held
Mesha first). It is what the print needs: without it every single-end
nakshatra of the measured pages misses by 29 to 92 vinadis, with it by
a few (C1293).

**The ends** (`tithi_ends`, `nakshatra_ends`). A Kali day runs from
one sunrise to the next. The Moon (for a tithi, the Moon less the Sun)
is taken at the day's sunrise and the next, and each 12° (13°20′)
boundary passed in between is placed by linear proportion; the fraction
is the end's time, read as nadis after the local sunrise (C1207).

**The sky.** `VakyaSky` is an `EphemerisProvider` whose Sun and Moon
are the vakya ones and whose every other body is the Surya Siddhanta's
over modern sunrise (`ModernParts`). An instant is mapped to its Kali
day by the modern upper-limb sunrise at Chennai (C1294), and the two
bodies are interpolated linearly across the day, as the almanac does.
The Moon's latitude is 0 and the planets (Kujadi) are not built.

**The day count.** Kali day *d* is the civil day whose noon is Julian
day *d* + 588,465 (`KALI_DAY_JD`); 13 April 1947 is Kali day 1,843,824.

## Measured

Each row is a printed end against the model's, in vinadis (a vinadi is
24 seconds).

| almanac year | page | ends | largest | rms | mean |
|---|---|---:|---:|---:|---:|
| Durmukhi 1956-57, Chittirai 1–14 | leaf 0150 | 30 | 5.8 | 2.4 | −1.4 |
| Bahudhanya 1998-99, Ani 1–32 | leaf 0024 | 65 | 9.7 | 2.5 | −0.9 |
| Manmatha 2015-16, Aippasi 1–15 | leaf 0300 | 32 | 8.6 | 3.9 | −1.5 |
| all | | 127 | 9.7 | 2.9 | |

The 2015 figures leave out the one slip (C1291); with the karana's
end in its place it is within 7. The tolerance was set after the
measurement: 12 vinadis, the largest miss with two to spare (C1295),
and every printed day ends as many limbs as the model. The largest miss,
1998-07-02, falls where the 248-day cycle wraps and the text's
correction steps.

**Vinayakar Chaturthi 2023.** Under `tamil-vakya` the
`GANESHA_CHATURTHI` rule at Chennai gives 18 September 2023; the drik
profile gives 19 September. On the 18th the vakya tritiya ends 13 nadis
42 vinadis after sunrise, so the chaturthi holds the madhyahna; the
Tamil Nadu government kept the holiday on the 18th (C1296). Both are
tests (`crates/sdk/tests/vakya.rs`).

## Cruxes

C1290–C1296 in
[`19-verification-cruxes.md`](../01-research/feature-universe/19-verification-cruxes.md).

## Open

- The reckoning place is Chennai, the almanac's press; a reader
  elsewhere counts the same fraction from a sunrise a few minutes away
  (C1294).
- The vakya planets (Kujadi) and the node (Rahu) are not built; the
  profile's other bodies are the Surya Siddhanta's (C1297).
- A Tamil Nadu festival pack is not built; the vakya days reach the
  existing packs' rules.
