# Numerology: Balliett's cycle and Cheiro's table (the `numerology` module)

Status: `built`, 2026-10-07: the kernel (`crates/numerology`), the boundary and every binding.

Numerology reads a name and a civil date. It reads no sky, so it is an
area of its own, like naam milan: `sdk.numerology()`. Two systems are
offered, each from the text that defines it. The baseline engine's
readings that no public-domain text prints ship as rank 2 `BASELINE`
knob values, so a migrating consumer gets the same numbers and sees
that they are not classical (C89's precedent, section 0 of the
Phase 8 survey).

## What the sources decide

**Balliett**, *The Philosophy of Numbers* (1908), IA
`philosophyofnumb00ball`, is rank 1 for the 1–9 letter cycle, read on
the page images.

- **The letters** (p. 17): a–i, j–r, s–z against 1–9. Y is 7 and W
  is 5 as letters (Henry p. 18, Canary p. 30, White p. 31).
- **Each word reduced, then added** (pp. 18–19): Henry 34 → 7,
  Elder 26 → 8, the name 15 → 6.
- **11 and 22 stop a reduction, 33 does not** (pp. 30, 31, 48, 90):
  White 29 = 11, Cream 22, Patterson 38 → 11, Orange 33 = 6,
  Wanamaker 33 → 6 (C320).
- **The birth number** (p. 19): month, day and year each reduced,
  then added: 17 January 1872 is 1 + 8 + 9 = 18 = 9. On p. 90, 11 July
  1838 is printed "9, 11": the day's 11 stands apart from 7 + 2
  (C323).
- **Vowels** (pp. 90, 97–98): "the hidden strength". Twenty-two
  "contains the vowels e, o", so Y and W are not vowels (C325).
  She names no vowel or consonant number.

**Cheiro**, *Cheiro's Book of Numbers* (Herbert Jenkins), IA
`in.ernet.dli.2015.70770`, is rank 1 for the system called Chaldean,
which is his.

- **The letters** (p. 70): A 1 B 2 C 3 D 4 E 5 F 8 G 3 H 5 I/J 1 K 2
  L 3 M 4 N 5 O 7 P 8 Q 1 R 2 S 3 T 4 U 6 V 6 W 6 X 5 Y 1 Z 7; no
  letter is 9.
- **Single and compound** (pp. 71–73, 87): each word's letters total
  its compound and reduce to its single; a name's compound is the
  sum of its words' singles (Lloyd 9 + George 7 = 16), and its single
  number that reduced (C321, C322).
- **No masters** (pp. 35, 72, 84): "an 11 is 1 plus 1, a 2";
  Baldwin 22 → 4.
- **Compound numbers 10–52** (pp. 78–85). From 33 each means what
  the number nine below means, except 37, 43 and 51.
- **The birth number is the day** (pp. 49–64, 73), reduced. Month
  and year are "separate and distinct and not added together"
  (p. 93) (C324).
- **Planets** (pp. 22, 37–64): 1 Sun, 2 Moon, 3 Jupiter, 4 Uranus,
  5 Mercury, 6 Venus, 7 Neptune, 8 Saturn, 9 Mars; the Sun is 1-4
  and the Moon 2-7 (C327).

**Sepharial**, *The Kabala of Numbers* (1911), is rank 2 and a
witness. He reduces every number to a unit (p. 9), and his Hebraic
key (p. 30) differs from Cheiro's in C, H and X. Nothing of his ships
by default.

## What is decided

- **Facts, not a reading.** The answer is numbers and the arithmetic
  that made them. Meanings are interpretation records, keyed by
  number and system.
- **A civil date, never an instant.** `BirthDate`, a date in the
  Gregorian calendar (`crates/numerology/src/lib.rs`). The baseline's host-zone `getDate()` cannot occur.
- **Latin letters only, refused otherwise** (C326). Case is folded; a
  character outside A–Z, a diacritic included (é), is refused with its
  place, because neither table says what é is worth and folding it to e
  would be a guess. Word separators are a space, a hyphen and an
  apostrophe, and are never refused.
- **Every word reported.** Each word's letters, total and single are
  in the answer, so either reduction order can be read back from it.
- **Each variant is a knob, with the source's reading as the default,
  and the baseline's as `BASELINE`.**

| knob | default (source) | other values |
|---|---|---|
| `masters` (Pythagorean) | `ELEVEN_TWENTY_TWO` (Balliett) | `NONE`; `ELEVEN_TWENTY_TWO_THIRTY_THREE` (`BASELINE`) |
| `name_reduction` | `BY_WORD` (Balliett, Cheiro) | `WHOLE` (`BASELINE`) |
| `chaldean_compound` | `SUM_OF_SINGLES` (Cheiro p. 72) | `LETTER_TOTAL` (`BASELINE`) |
| `birth_reduction` (Pythagorean date) | `BY_PART` (Balliett p. 19) | `DIGIT_SUM` (`BASELINE`) |
| `non_latin` | `REFUSE` | `SKIP` (`BASELINE`) |

## The surface

`crates/numerology` reads no sky and needs nothing but `Graha` and the
Gregorian month lengths:

- `System::{Pythagorean, Chaldean}`, each with its `table()`:
  `BALLIETT` and `CHEIRO`, `[u8; 26]`.
- `WordNumber.letters` is a list of `Letter { letter, value }`, so a
  binding reads each letter by name rather than as a pair.
- `NumerologyRules { masters, name_reduction, chaldean_compound,
  birth_reduction, non_latin }`, `Default` the sources' and
  `NumerologyRules::baseline()` every `BASELINE` value; read from JSON
  strictly (`camelCase` fields, `SCREAMING_SNAKE_CASE` values).
- `reduce(n, Masters) -> Reduction { steps, number }`: every sum, so
  "33, so 6" and "38, so 11" read back.
- `name_number(name, System, &rules) -> NameNumber { system, words,
  total, compound, reduction }`, each `WordNumber { text, letters,
  total, reduction }`.
- `BirthDate::new(year, month, day)`, validated against the Gregorian
  calendar and refusing a year before 1.
- `pythagorean_birth(date, &rules) -> BirthNumber { month, day, year,
  sum, apart }`: `apart` holds the parts that are masters, in the order
  month, day, year, and `sum` the rest added, `None` when every part is
  a master.
- `chaldean_birth(date) -> ChaldeanDate { birth, year }`. The month's
  period number waits on reading Cheiro's periods for 1 to 4.
- `compound_class(n)`, `planets_of(n)` (Cheiro's, with Uranus and
  Neptune), `baseline_anka(graha)` (Rahu 4, Ketu 7, named for what it
  is) and `days_of(n)`.
- `profile(name, date, &rules) -> Profile { pythagorean_name,
  pythagorean_birth, chaldean_name, chaldean_birth, baseline }`, with
  `baseline: Some(BaselineProfile { soul, personality,
  chaldean_destiny })` only under `NumerologyRules::baseline()`, so the
  baseline's numbers are never mistaken for the texts'.

The façade re-exports the crate as `teistro::numerology` and asks it as
`sdk.numerology().profile(name, date, rules)`, an area of its own in
every binding (`surface-areas.md`).

**The boundary** is one entry point, `ts_numerology_profile`, taking
`{"name", "date", "rules"}` (`teistro::NumerologyRequest`) and answering
the profile as canonical JSON, one name at a time. The answer is one
nested document per call, so it crosses as the façade's own
serialisation rather than as a blob, and each binding parses it into its
own types. This is the pattern the other Phase 8 modules reuse. A
refusal is named under `numerology`: `numerology.name` with the
character and its place, `numerology.date` for a date the calendar does
not have (`BirthDate` is read through `BirthDate::new`, so JSON cannot
carry 29 February of a common year past it), and
`numerology.rules.<key>` for a reading misspelt.

| binding | the date | the rules | the answer |
|---|---|---|---|
| Rust | `BirthDate::new(y, m, d)?` | `NumerologyRules` | `Profile` |
| Node | `{ year, month, day }` | `{ masters, ... }`, each optional | a frozen `NumerologyProfile` |
| Python | `datetime.date` | a `NumerologyRules` `TypedDict` | a frozen `NumerologyProfile` dataclass |
| Dart | `BirthDate(y, m, d)` | `NumerologyRules(...)`, `.baseline()` | `NumerologyProfile` |

A civil date never crosses as an instant, and no binding takes one:
JavaScript's `Date` and Dart's `DateTime` carry a zone, and moving a
birth by a day is the baseline's defect.

## The acceptance tests

Held now, in `crates/numerology/src/tests.rs`:

- **Each letter table, all 26 cells, against its page:** Balliett p.
  17 and Cheiro p. 70. A test also names the three cells where
  Sepharial's Hebraic differs (C 2, H 8, X 6), so a future
  "correction" towards it is caught.
- **The printed examples, each to the number on the page:** Henry
  Elder (34/7, 26/8, 15/6; birth 18/9), John and Sarah both 2, White
  29/11, Cream 22, Orange 33/6, Mary 21/3 and Patterson 38/11, John
  Wanamaker 2 and 6, so 8, born "9, 11"; Lloyd 18/9, George 25/7,
  Lloyd George 16, David Lloyd George 23, Baldwin 22/4, John Smith
  17/8; born 6 June 1866, 6 and 1866/21/3 apart.
- **Each rival reading pinned:** Orange 6 against 33; Peter Stone 2 by
  word against 11 whole; Lloyd George's compound 16 against 43; 11 July
  1838 "9, 11" against `DIGIT_SUM`'s 29/11; Henry Elder's vowels 15/6,
  which with Y would be 22.
- **Cheiro's 33-and-up rule** at its edges and the three numbers with
  their own meanings.
- **Reduction over every n to 10⁶ under every stopping set:** the
  number keeps n's residue mod 9 and stops only at one digit or a
  master.
- **Refusals:** "Kṛṣṇa" refused at its second character, and read as K
  and A under `SKIP`; "", spaces and separators alone refused; 29
  February of a common year refused.
- **At the boundary** (`crates/ffi/tests/abi.rs`): the crossed JSON is
  the façade's to the byte, and each refusal is named down to its field.
- **In every binding:** Henry Elder and John Wanamaker read in Node,
  Python and Dart to the printed numbers, and the five parity runners
  print both requests (the second under every baseline reading, with
  the baseline's numbers) value for value alike.

Still to hold: the remaining printed names (Canary, Pink, Green, Blue),
Cheiro's seventeen printed aliases one by one, a measured page over
every date 1800 to 2399 (how often a master appears and how often
`BY_PART` and `DIGIT_SUM` part), and the baseline engine's golden
vectors under `NumerologyRules::baseline()`, which an exporter on its
side produces.
