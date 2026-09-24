# The Harsha bala

Status: built — 2026-09-24. `crates/tajika/src/harsha.rs`, reached at
`sdk.chart().harsha` and `harsha_with_rules`.

The source is K. S. Charak, *A Textbook of Varshaphala*, ch. VI (rank 2),
checked against the *Tajika Nilakanthi*, Saṃjñā Tantra v. 76, with
Gangadhara Mishra's commentary (Varanasi, 1976; rank 1).

## Four parts of five

"Harsha" is happiness: a planet is comfortable in certain places, and each
of four is worth **five units**, so a planet holds 0 to 20.

| part | the source's name | a planet takes it when |
|---|---|---|
| `sthana` | Prathama, the place | it stands in its own house of joy: the Sun the 9th, the Moon the 3rd, Mars the 6th, Mercury the 1st, Jupiter the 11th, **Venus the 5th**, Saturn the 12th |
| `uchcha_swakshetra` | Dwitiya | it is in its exaltation sign or a sign it owns |
| `stri_purusha` | Tritiya | a female planet stands in a feminine house (1, 2, 3, 7, 8, 9) or a male one in a masculine house (4, 5, 6, 10, 11, 12) |
| `dina_ratri` | Chaturtha | the year opens by day and it is male, or by night and it is female |

**The genders are Tajika's own**: the Sun, Mars and Jupiter are male, and
the Moon, Mercury, Venus and Saturn female. The commentary says outright
that "there is no neuter here", so the catalogue's Parashari genders —
which make Mercury and Saturn neuter — are not read.

**Houses are whole signs from the annual lagna**: the verse counts "three
signs at a time from the lagna's sign" (*tribhaṃ tribhaṃ lagnabhataḥ*), and
the source's Example Chart places each planet by sign.

The grades the source gives a total: **Nirbala** (0), **Alpabali** (5),
**Madhya Bali** (10), **Poorna Bali** (15), and at 20, which it calls
rare, "extraordinarily strong".

## The worked example

Table VI-1 gives all seven for the Example Chart, the forty-first year by
day, lagna Scorpio: Sun 15, Moon 10, Mars 10, Mercury 0, Jupiter 10,
Venus 0, Saturn 10, each part as printed. A test reproduces all 28 cells
from the printed longitudes, and another the seven totals end to end from
a birth and a year the SDK founded itself.

**Three planets can never hold twenty.** The source calls twenty rare;
for the Sun, Venus and Saturn it is impossible, because each one's house
of joy is of the other gender — the Sun's 9th is feminine, Venus's 5th
and Saturn's 12th masculine — so the place and the gender parts exclude
each other. A test walks every sign, lagna and half of the day to show
it, and that the other four can. What each grade costs over every
recorded birth's forty years is measured in
[`muntha-measured.md`](muntha-measured.md) §15, where the pass fails if
one of the three ever reaches twenty or one of the four never does.

## One rival: Venus's house

The verse reads *nanda tri ṣaṭ … bhava putra vyayā ināt* — from the Sun,
9, 3, 6, 1, 11, **5**, 12 — and the 5th is Venus's joy in the older
doctrine the Harsha places descend from. **A widely used program puts
Venus's place in the 12th.** Measured as a black box over 600 random
charts (values only, CLEAN_ROOM rule 3), the source's rule agrees with it
in 495, every disagreement is Venus by five units, and with Venus in the
12th it agrees in **600 of 600**. It ships as `VenusPlace::Twelfth` so a
caller can reproduce that program; the default is the verse's.

| `HarshaRules` | default | rival |
|---|---|---|
| `venus` | `Fifth`, the verse and the source | `Twelfth`, the program |

## What is decided and what is not

| | |
|---|---|
| **decided by the source** | the four parts, their five units, the places of joy, the genders, the feminine houses, whole signs (Table VI-1, all 28 cells) |
| **decided by the rank-1 text** | Venus's place is the 5th (v. 76) |
| **decided by measurement** | the rival program differs in Venus's place and nothing else (600 of 600) |
| **across the boundary** | `year_harsha`, seven rows under each founded year, with `varsha_json.harshaRules` (`tajika-saham-strength.md`, "Crossing the boundary") |
| **not built** | the Dwadashavargiya bala, the source's twelve-fold strength, which it calls elaborate and seldom used |

## The order of work

1. **Done**: the four parts, the grades, the rival, the façade.
2. **Done**: a saham's strength, over the Harsha bala and the
   Panchavargiya bala (`tajika-saham-strength.md`), and the crossing of
   both with the birth chart's sahams.
