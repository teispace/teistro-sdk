# The grahas' arudhas

Status: `draft`, 2026-09-26; §6 steps 1 and 2 **built** the same day.
Written from the text and a measurement before any code; the building is
expected to correct it, and did once (§4).

Derives from the roadmap's Phase 7 list of what is left of `jaimini` (the
graha arudhas), and from `jaimini-significators.md`, whose reading this
extends: the arudhas are one value a graha, so they cross as two more
columns of the `jaimini_houses` section rather than a section of their own.

## 1. What the text says

BPHS ch. 29 vv. 6 and 7, read in the Sanskrit on the printed page (vol. 1,
p. 297); the archive's OCR of this volume transliterates the Devanagari
into Latin noise, so the page image is the only reading.

- v. 6, यस्माद् यावतिथे राशौ खेटात् तद्भवनं द्विज । ततस्तावतिथं राशिं
  खेटारूढं प्रचक्षते: **however many signs from the graha its own sign
  stands, that many signs on from its sign is the graha's arudha**.
- v. 7, द्विनाथद्विभयोरेवं विज्ञेयं सबलावधि: **for a sign with two lords
  and a graha with two signs alike, count up to the stronger**.

The translation agrees and adds two things the verses do not say, in the
translator's note: that there are **nine** arudhas, the nodes' through
their co-lordship of Aquarius and Scorpio (ch. 46 v. 157), and that **the
exceptions stated for the bhava arudhas do not apply**. Its worked example
is the acceptance case: the Sun in Capricorn counts eight signs to Leo and
eight more to **Pisces**.

## 2. What was measured

PyJHora 4.8.7 as a black box (values only) over the 600 charts drawn for
the Brahma measurement, `graha_arudhas_from_planet_positions`:

| reading | charts' grahas it reproduces |
|---|---|
| the count, the nodes to Aquarius and Scorpio, **with** the bhava arudhas' exception from the graha's own sign (a count landing on its sign or the 7th from it moves to the 10th from there) | **2 400 of 2 400** single-signed (the Sun, the Moon, Rahu, Ketu) |
| the same with no exception | 1 650 of 2 400 |
| the two-signed planets under ch. 46's sign strength (occupancy, modality, the greater count) | 2 463 of 3 000 |
| the same with exaltation first, the ladder `stronger_sign` builds | 2 449 of 3 000 |
| the greater count alone | 1 616 of 3 000 |

So PyJHora applies the exception the translator says does not apply, and
chooses between a planet's two signs by a strength of its own that no
reading of ch. 46 reproduces past 82%, as it chose the Brahma graha.

## 3. The forks (cruxes)

| crux | question | readings | default | why |
|---|---|---|---|---|
| C132 | whether a graha's arudha takes the bhava arudhas' exception | none (the verses are silent, the translator says none); the exception from the graha's sign (PyJHora, 2 400 of 2 400) | **none**, a knob for the other | the verses state the count and no move; the move is the bhava arudhas' own verse |
| C133 | whether the nodes have arudhas | through co-lordship, Rahu to Aquarius and Ketu to Scorpio (the translator, PyJHora); none (a node owns no sign) | **the existing `jaimini.node_co_lordship`**: `NONE` gives the nodes no own sign and so no arudha, the others give Rahu Aquarius and Ketu Scorpio | one knob already says whether a node lords a sign in this school; a second would let the two disagree |
| C134 | the stronger of a planet's two signs | ch. 46 vv. 161 to 163 (occupancy, then modality, then the greater count, C51) and v. 164's exaltation; PyJHora's own | **`stronger_sign`, then the greater count**, the ladder C51 settled from the same chapter's verses | v. 7 names no ladder and ch. 46 is the text's only one |
| C135 | a bhava pada of a two-lorded sign | the stronger lord (v. 7, द्विनाथ); the catalogue's lord (the recording engine, 0 of 852 disagree in `arudhas-measured.md`) | **unchanged**, the catalogue's lord, until built as a knob | the padas are shipped and measured; moving their default is its own change, and this page's is the grahas' |

## 4. The design

`teistro_dasha::jaimini::graha_arudhas(chart, co_lordship, exception)`
returns each graha's arudha, `None` for a node that owns no sign under the
co-lordship. A graha's own signs are its sign lordships, with Rahu on
Aquarius and Ketu on Scorpio when the co-lordship makes them lords; of two,
the stronger by `stronger_sign`, and on a tie the one the greater count
reaches, which is ch. 46 v. 163's last step. The count is `points`'
`on(at, steps(at, own))`, so the arithmetic is the padas' and not a copy.

A new knob, `jaimini.graha_arudha_exception`: `NONE` (the verses) or
`AS_BHAVAS` (the bhava arudhas' move, from where the count landed). The
reading gains `graha_arudhas: [Option<Rashi>; 9]`, the Sun to Ketu, and the
boundary's per-graha section gains `arudha` and `arudha_present`.

**Building it renamed the section.** Step 6 of
`jaimini-significators.md` had named it `jaimini_houses`, for the only
per-graha values it then carried; with the arudhas it is `jaimini_grahas`,
renamed before any release carried the old name rather than given a
second per-graha section beside it. And a test's expected value was
wrong, not the code: the move under `AS_BHAVAS` is to the 10th from where
the count **landed** (Libra's 10th is Cancer), which is the reading the
measurement fitted on 2 400 of 2 400. The arithmetic stayed in
`jaimini.rs` over its own `ahead` and `steps`, since `points`' are private
and the dasha crate does not depend on it.

## 5. Tests

- The translator's example, the Sun in Capricorn to Pisces, under both
  exceptions (Pisces is the third from Capricorn, so neither moves it).
- A count landing on the graha's own sign and on the 7th, moved under
  `AS_BHAVAS` and not under `NONE`, both ways.
- The nodes: `None` under `NONE`, Aquarius and Scorpio counted otherwise.
- A planet in one of its two signs: the verse counts to "its own sign",
  and the ladder decides which; held by a named case.
- Parity across the four languages through the existing parity lines.

## 6. Order of work

1. The function and the knob, with the tests above: **done**.
2. The reading and the boundary columns: **done**, with parity across the
   four languages.
3. C135, the bhava padas' two-lorded signs, as its own change: a knob, the
   arudhas page re-aimed to measure both readings, the default argued from
   the text.
