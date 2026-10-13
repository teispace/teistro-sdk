# Prashna: the query chart read as Shatpanchashika and the Tajika print it (the `prashna` module)

Status: steps 1, 3 and 4 `built` (`crates/prashna`, 2026-10-07: the
verdict, change, timing and the unspoken question; the `BASELINE` values;
the façade and every binding), step 2 partly built (the Tajika links,
what they say of the matter with Kamboola's grades and the tri-kāla, and
the Moon, 2026-10-12; the Bhuvanapradīpa's grades built off, C373; the
akṣara-lagna and the six points open, C353). The
`SHADVARGA` rising varga is built (C336, 2026-10-13).

A prashna is a chart cast for the moment a question is asked. Casting it
is already done: the chart kind `PRASHNA` exists, and the chart is an
ordinary chart for that instant and place. What this page designs is
the **reading**: whether the matter succeeds, when, what an unspoken
question is about, and where the querent's own lagna comes from when the
clock is not used.

The default of every reading is a rank-1 text read on its page. The
baseline engine's prashna readings were checked against those texts and
none of its four was found in them (the timing rule, the point score,
the 1–108 number and the twelve-house mook table). They ship as rank 2
`BASELINE` knob values marked unsourced, so a migrating consumer gets
the same answer and can see that it is not classical (C89's precedent).

## What the sources decide

The texts read, each on its page images unless marked:

- **Shatpanchashika** (Prithuyashas), ed. V. Subrahmanya Sastri, 1941,
  IA `dli.ministry.22678`. The printed page *p* is leaf n(*p*+7). The
  Sanskrit is quoted; the English is only paraphrased here, because it
  may still be in copyright.
- **Tajika Nilakanthi** with Vishvanatha's tika, Nirnaya Sagara 1893,
  IA `tajika-nilakanthi-satika-sodaharana-...`: the Prashna Tantra,
  leaves n174–n205, and the Samjna Tantra. The Prashna Tantra is mostly
  a **compilation**. It names its sources inline (Shatpanchashika,
  Utpala, Bhuvanapradipa, Samarasimha, Chintamani, Jnanaprakasha), so
  each rule below is cited both by its leaf and by the source it names.
- **Prasna Marga**, part 1 (chapters 1–16), with Punnasseri Nambi's
  notes, IA `prasnamarga-035823mbp-1`. **Part 2 (chapters 17–32) is in
  no public-domain scan found**, so anything Prasna Marga says there is
  unread (C342).

### Success or failure

- **Shatpanchashika I.4** (p. 6) gives three outcomes:
  - **success** when a benefic is in the lagna, or a benefic's varga
    rises, and the lagna is *śīrṣodaya*;
  - **failure** on the reverse;
  - **success with difficulty** when the conditions are mixed.
- **I.3** (p. 6): a house occupied or aspected by its own lord or by a
  benefic prospers, and one held by malefics declines. The text says
  this holds "for a query or a birth".
- **II.2** (gloss pp. 14–15): a benefic aspect on the lagna or on the
  Moon gives success.
- **The two lists** (the gloss to I.3):
  - benefics: Jupiter, Venus, an unafflicted Mercury and the full Moon;
  - malefics: the Sun, Mars, Saturn, the waning Moon and a Mercury
    joined to a malefic.
- **Aspects:** Varahamihira's graded aspects (a quarter, a half, three
  quarters and full).
- **The rising-sign list** (the 1941 gloss's verse):
  - *pṛṣṭhodaya*: Aries, Taurus, Cancer, Sagittarius and Capricorn;
  - *śīrṣodaya*: the other six apart from Pisces;
  - Pisces: "ubhayato mīnaḥ", rising both ways.
  - Brihat Jataka I.10, quoted in Prasna Marga's notes, leaves Pisces
    among the *śīrṣodaya* signs (C336).
- **No text gives point weights.** A yes or no is three outcomes over
  stated clauses, not a score (C337).

The Tajika reading (Prashna Tantra, leaves n178 and n200–n204) judges
the **lagna lord** and the **lord of the house asked about** (the
kāryesha):

- **ithasala** between them: success;
- **nakta** (a faster third graha carries the light between them) or
  **yamaya** (a slower one does): success through another person;
  vv. 17–20 give a worked example;
- **isharafa** (separation): a failed matter, or one that is past;
- **kamboola**, where the Moon joins the ithasala: grades the result by
  dignity (v. 21);
- **v. 96**: "all yogas are fruitless without the Moon";
- **Samarasimha**: with the lagna not involved, "nothing is to be said";
- **orbs** (v. 13): Sun 15°, Moon 12°, Mars 8°, Mercury 7°, Jupiter 9°,
  Venus 7°, Saturn 9°. Ithasala is judged by orb, even across signs.

### Timing

No text read says what the baseline engine does: movable = days,
fixed = months, dual = years, counted from the lagna lord's house
(C335). What the texts do say:

| rule | source | count | unit |
|---|---|---|---|
| **default** | Shatpanchashika II.14–15 (pp. 21–22); Tajika Nilakanthi repeats it as Chintamani vv. 22–23 (n188) | the strongest graha's house from the lagna | months, ×1, ×2 or ×3 as that graha's **navāṁśa** is movable, fixed or dual |
| `FIRST_OCCUPIED` | Shatpanchashika V.5 (pp. 35–36) | the first occupied sign from the lagna | ×12 days, or the count itself in days if that graha is retrograde; the verse's example: Taurus rising, Virgo first occupied, 60 days |
| `MOON_DAYS` | Shatpanchashika II.17 (p. 23, OCR) | the Moon's sign from the lagna | days, and no time at all when a graha stands between them |

- Modality on the **lagna** is a different thing in the text. Under
  Shatpanchashika II.1–2 a fixed lagna means the matter stays as it is
  and a movable one means it changes; a dual sign is fixed in its first
  half and movable in its second. That is a fact about change, not a
  unit, and the reading reports it as such.
- **"Strongest" is not defined by the verse.** The SDK takes the
  Shadbala total in rupas over the seven grahas. A tie goes to the
  graha earlier in the weekday order, and the answer says that it
  broke a tie (C338).
- Utpala's numeric timing (Tajika Nilakanthi vv. 6–11, n201) is only
  partly legible: multipliers 5, 21, 14, 9, 8, 3, 11, and a unit chosen
  by the graha. It is not shipped until the leaf is transcribed (C342).

### The unspoken question (mook prashna)

- **Shatpanchashika VII.7–8** (p. 57) names the **person** in the
  querent's mind from the house of the strongest graha:
  - 1: the querent, or one like them;
  - 3: a brother;
  - 4: the mother or a sister;
  - 5: a son;
  - 6: an enemy;
  - 7: the wife;
  - 9: a religious person;
  - 10: the guru.
- With the strong graha in the lagna, the lord of the lagna navāṁśa
  refines the answer: that lord itself means the querent, its friend
  a friend, its enemy an enemy.
- Houses 2, 8, 11 and 12 are **not listed** and answer `UNSPECIFIED`
  (C339).
- **Jnanaprakasha**, quoted at n202, gives the topic from the house the
  Moon occupies, or from the lagna lord's house when he is the stronger.
  That covers all twelve houses and ships as `MOON_HOUSE`.
- **Shatpanchashika I.6–7** (pp. 9–12, OCR) sorts the thing thought of
  into mineral, root or living being by navāṁśa: in an odd sign the
  nine run mineral, root, living, three times over, and an even sign
  reverses the order.

### The lagna from the querent

- **Prasna Marga 4.53** (p. 75): the **ārūḍha** is the sign where the
  child places the gold. It is an act, not a computation, so the caller
  supplies it.
- **Prasna Marga 8.1** reads six points together: the ārūḍha, the
  lagna, the lagna navāṁśa, the *chatra*, the sign of the limb touched
  and the Moon's sign.
- **The akṣara-lagna** (Utpala, quoted in Tajika Nilakanthi vv. 12–15
  at n201, and in Prasna Marga 2.110–113):
  - the first syllable the querent speaks selects one of the eight
    letter groups, whose lords run Sun, Mars, Venus, Mercury, Jupiter,
    Saturn and the Moon;
  - the group's 1st, 3rd and 5th letters give its lord's odd sign, and
    the 2nd and 4th its even sign.
  - It is the only "lagna from the querent" the texts give.
- **No 1–108 number appears in any text read.** The baseline's
  querent number ((n − 1) mod 12 as a navāṁśa sign) ships as
  `BASELINE` and is marked unsourced (C340). The KP horary number,
  1–249, is a different rule and is already built (`kp.md`).

### The Moon

- **Lilly's void of course** is already built (`crates/hellenistic`'s
  considerations, C230, C234). It stays labelled "after Lilly": no Indian text read has
  that definition (C341).
- **Tajika's *śūnyamārga*** (Samjna Tantra vv. 55–56, n60) is a
  different fact: a graha with no dignity and **no aspect received**.
- **vv. 73–74** (n66) count the Moon weak in a query as well as at birth
  when she is 12th from the Sun, in Scorpio's first half or Libra's last
  half (the gloss names the halves), unseen by her sign's lord or by every
  graha, *śūnyamārga*, waning (the gloss: from the dark 8th to the bright
  8th; "some" say the dark 11th to the new Moon), at the end of a sign
  ("bhānte"; the gloss reads the last navāṁśa), or under the hungry aspect,
  from the 1st, 4th, 7th or 10th, of Mars in the bright half or of Saturn
  in the dark (C352).
- Each ships as a fact of its own; none is an alias for another.

## The surface

`sdk.prashna()` is an area of its own and takes the chart the caller
cast for the moment of the question. The chart is not re-cast: a
prashna is about the sky that was, at the place it was asked.

```rust,ignore
let reading = sdk.prashna().read(&chart, &PrashnaQuestion {
    house: Some(7),                 // the kārya house, when the question names one
    arudha: None,                   // the sign the gold was placed on
    first_syllable: Some("ka"),     // for the akṣara-lagna
    rules: PrashnaRules::default(),
})?;
```

| answer | contents |
|---|---|
| `verdict` | each clause of SP I.4, I.3 and II.2 with its graha or varga, and `SUCCEEDS`, `WITH_DIFFICULTY` or `FAILS` |
| `change` | the lagna's modality read as II.1–2: `STAYS`, `CHANGES`, or a dual sign's half |
| `links` | the sixteen Tajika yogas between the lagna lord and the kāryesha, as a year's matter; present only when a house is asked |
| `link` | what they say of the matter: `kind` (`DIRECT`, `THROUGH`, `DESTROYED`, `SEPARATED`, `SAME_LORD`, `UNLINKED`), its `meaning` under the question's `tense` (C370), the yoga and third graha, and a Kamboola's grade (C371, C372); present when `links` is |
| `triKala` | the lagnesha's own yogas as past (Ishrafa), present (Vartamana or Poorna Ithasala) and future (Bhavishyat, or applying outside the orb), C375 |
| `weak` | each of the seven's clauses of the Samjna Tantra v. 72: unseen by the lagna, in the Sun's setting part (C377), officeless, in Ishrafa with a malefic |
| `timing` | count, unit, multiplier and the graha it came from, with `tie: bool` |
| `mook` | the person (SP VII.7–8) or the topic (`MOON_HOUSE`), plus the class of the thing thought of |
| `aksharaLagna`, `sixPoints` | the sign from the syllable; PM 8.1's six signs when an ārūḍha is supplied |
| `moon` | `voidAfterLilly`, `shunyaMarga`, `bhante`, `waning` and `hostileAspect`, each its own fact |

`PrashnaRules` holds one knob per crux:

- `pisces`: `BOTH_WAYS` (default) or `SHIRSHODAYA`;
- `rising_varga`: `NAVAMSHA` (default) or `SHADVARGA` (C336): the
  rising navāṁśa's lord, or the lords of the lagna's six vargas (D1, D2,
  D3, D9, D12, D30 under the SDK's classical schemes), a clause for when
  four or more are benefics, against when four or more are malefics, and
  both ways otherwise (`RISING_SHADVARGA`);
- `timing`: `STRONGEST_GRAHA` (default), `FIRST_OCCUPIED`, `MOON_DAYS`
  or `BASELINE`;
- `mook`: `SHATPANCHASHIKA` (default), `MOON_HOUSE` or `BASELINE`;
- `score`: `OFF` by default, `BASELINE` adds the baseline's points
  beside the clauses and never replaces them;
- `moon`: `MoonRules`, Tajika's reading of the Moon's weaknesses
  (Samjna Tantra vv. 73–74), reused;
- `offices`: `OfficeRules`, how Kamboola's offices are read: `order`
  (`GLOSS` by default, `HIGHEST`, `LOWEST`; C371), `drekkana` (`TAJIKA`
  or `PARASHARI`), `enemy` (`TAJIKA` or `IGNORED`) and `hudda`
  (`NILAKANTHI` or `CHARAK`, C109);
- `weakness`: `WeaknessRules`, v. 72's setting part (`NAVAMSHA_SIGN` by
  default or `NAVAMSHA_ARC`, C377);
- `moon_scope`: which questions v. 96's Moon governs (`EVERY` by
  default, or `GAIN`, C376);
- `graded`: the Bhuvanapradīpa's graded yogas, `OFF` by default while
  C373 is open, or read by `PARASHARI` or `TAJIKA` aspect.

`Question::tense` (`UNSPECIFIED` by default, `FUTURE` or `PAST`) says
when the matter falls, which is what a separation's meaning turns on
(C370).

`PrashnaRules::baseline()` sets every `BASELINE` value at once.

**Reuse:**

- the Tajika links come from `crates/tajika`'s ithasala machinery,
  read over the prashna chart's sky rather than a year's;
- the strongest graha comes from `crates/strength`'s Shadbala;
- the syllable's letter group comes from `crates/matching`'s name
  reader. That reader already knows the eight groups (`NameVarga`) and
  gains the letter's place in its group (1–5);
- Lilly's void comes from the hellenistic considerations.

**Refusals:**

- a house outside 1–12;
- a syllable not in Devanagari or IAST;
- `MOON_HOUSE` or a mook reading on a chart without a Moon.

Each names its field under `prashna`.

## Steps

1. **The kernel** (`crates/prashna`), built. It covers:
   - the verdict clauses and their three outcomes;
   - change;
   - timing: the three sourced rules and `BASELINE`;
   - mook: both readings and the class of the thing thought of.

   V.5's example passes as printed: Taurus rising and Virgo give 60
   days, or 5 when retrograde. The kernel takes a `PrashnaSky` of
   longitudes, navāṁśas and Shadbala totals, so it reads no ephemeris.

   **How it reads the verses:**
   - A clause's graha is benefic or malefic by the gloss's list, and the
     nodes, which the list leaves out, make no clause.
   - An aspect is a full aspect under `crates/aspect`'s graded drishti,
     a special aspect included. The test that found Mars aspecting a
     Gemini lagna's 7th from Taurus holds this.
   - The rising varga is the navāṁśa's lord. The baseline's score and
     Tajika's Moon knob came with steps 2 and 3, and the `SHADVARGA`
     knob on 2026-10-13: the gloss names the six vargas and not how many
     must be a benefic's, so a majority decides (four of six), and three
     of each leave the clause both ways, as Pisces rising does. The sky
     carries the lagna's six signs, the navāṁśa fourth, so the clause can
     be checked against them.
2. **The Tajika links**, partly built. When a house is asked,
   `Prashna::links` holds `crates/tajika`'s sixteen yogas judged between
   the lagna lord and the lord of that house, with each graha's
   retrogression and combustion from `Placed`. The nakta example of
   vv. 18–19 passes as printed: Virgo rising, Mercury in Leo, Jupiter in
   Pisces, the Moon carrying the light. `Prashna::moon` reports each
   clause of the Samjna Tantra vv. 73–74 that holds, read by
   `crates/tajika`'s `moon_weakness` with the empty road of vv. 55–56
   (C352). `Prashna::link` reads what the links say, as the Prashna
   Tantra's final section does: an Ithasala promises the matter (v. 14),
   Nakta or Yamaya promise it through the third graha (vv. 17–20), Manau
   destroys it (the Samjna Tantra's example to v. 35 is a query: Libra
   rising, the kingdom "taken away", which passes as printed), and an
   Ishrafa fails a matter to come or finds one past (v. 16, C370). Where
   the Moon joins the Ithasala the link carries **Kamboola's grade**
   (`crates/tajika`'s `kamboola_grade`, Samjna Tantra vv. 36–46): each
   of the Moon, the lagnesha and the karyesha holds an office, *uttama*
   (own sign, exaltation), *madhyama* (own hudda, drekkāṇa, navāṁśa),
   *sama* or *adhama* (debilitation, an enemy's sign), every clause it
   meets reported. The gloss does not say which office counts when a
   graha holds several; its worked examples do, read in the order
   debilitation, division, sign, enemy, which reproduces nine of the
   twelve it grades where "the highest counts" reproduces six (C371),
   and the three it misses each miss on a different question, named by
   the test. The pair's office is named only when the lords agree (C372).
   `Prashna::tri_kala` reads the lagnesha's yogas with every other graha
   as past, present and future (n179, C375). The link also reports
   v. 96's `moonless` (C376) and Samarasiṁha's conditions that hold, his
   "loss" clause left open (C374), and `Prashna::weak` each of the
   seven's weaknesses of v. 72 (C377), the empty road and the Moon's
   own clauses reading the same offices. Under `graded`,
   `Prashna::graded` reports the Bhuvanapradīpa's grades as the
   general section quotes them (n178): every clause they are built
   from (the lords' three configurations, the benefics on the lagna and
   on its lord, the Moon's freedom) and each grade whose clauses hold,
   by name, with the grahas that give ill. It is off by default (C373).
   The akṣara-lagna and the six points are open (C353): two
   prints carry vv. 12–15 without a gloss, and the root leaves four
   readings unsettled.
3. **The `BASELINE` values**, built, transcribed from the baseline
   engine's source as it runs:
   - `ScoreRule::Baseline` puts its points beside the clauses: the lagna
     lord in a kendra or trikona (+2) or a dusthana (−2), the waxing Moon
     (+1), its void Moon over grahas held still (−3), benefics and
     malefics in kendras (±1 each, the Moon always a benefic), and the
     Moon applying by conjunction distance alone (±1); over 1 is yes,
     under −1 no;
   - `MookRule::Baseline` takes its strongest graha, 2 for a kendra and
     1 for a trikona, the first in its order (Sun, Moon, Mercury, Venus,
     Mars, Jupiter, Saturn) taking a tie;
   - `number_sign` and `Question::number` give (n − 1) mod 12 from Aries.

   **Found:** the baseline scores three dignity factors (the lagna lord
   exalted, own sign, debilitated) and its topic two, but the planets it
   scores never carry those flags, so none of the five ever fires. They
   are left out, so a migrating consumer gets the answers the baseline
   actually gave.
4. **The façade and every binding.** The Rust façade is built:
   `sdk.chart().prashna(&document, &PrashnaRequest)` reads the chart the
   caller cast for the moment, its lagna and navāṁśa, the nine grahas'
   motion and combustion under the context's settings, and the seven's
   Shadbala, refusing a document without it (`shadbala`); `prashna_sky`
   answers what it read. `PrashnaRequest::from_json` takes the record a
   binding will send, refusals named under `prashna`. Every binding is
   built on KP's pattern: a `prashna_json` member of the chart request,
   which also asks for the Shadbala, and section 120 `prashna`, canonical
   JSON with one reading a chart. Every binding reads it as
   `chart.prashna`, with the links in a year's matter's shape.

## Acceptance

- Every clause in the verdict names its verse.
- `PrashnaRules::default()` reaches no `BASELINE` value, a test asserts
  so, and no `BASELINE` value is reached unless the caller asked.
- The verses' printed examples pass as written.
- A tie in strength is reported, never broken silently.
- Houses 2, 8, 11 and 12 under Shatpanchashika answer `UNSPECIFIED`,
  never a guess.
