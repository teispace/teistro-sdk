# Derived points: the upagrahas and the special lagnas

Status: `built`, written 2026-09-07 from the falsification pass in
[`points-measured.md`](points-measured.md), which measured every rule
below against the corpus's own recorded answers before this page
existed. Derives from [`chart-foundation.md`](chart-foundation.md) (the
lagna, the Sun and the Moon, the sunrise and the time since it),
[`core-types-and-catalogue.md`](core-types-and-catalogue.md) (the 49
catalogued points and the eighths' table) and
[`astro-house-systems.md`](astro-house-systems.md) (the ascendant at an
instant). `02-architecture/01-module-catalog.md` gives the module its
row. Built as `crates/points`.

## 1. Purpose and scope

Points that are not bodies but behave like them: they have a longitude,
they fall in a sign and a house, and everything above reads them the way
it reads a graha. Three families ship:

- the **upagrahas the Sun casts** — Dhuma, Vyatipata, Parivesha,
  Indrachapa, Upaketu — a chain of five, each defined from the one
  before it;
- the **upagrahas the day divides** — Gulika and Mandi, which are the
  ascendant at the two ends of Saturn's eighth of the arc, and Kala,
  Mrityu, Yamaghantaka and Ardhaprahara, the ascendant where the Sun's,
  Mars's, Jupiter's and Mercury's eighths begin (§13);
- the **special lagnas** — the bhava, hora, ghati and pranapada lagnas, which
  the clock drives; the Sree lagna, which the Moon's nakshatra drives;
  the Indu lagna, which the ninth lords' kalas count from the Moon (§13);
  and the Yogi and Avayogi points.

It is not: the chara karakas, which the corpus records under `houses`
but which belong to `jaimini`; the sphutas, which
have no citation (crux C24); or the sahamas and Arabic lots, which are a
later phase.

**The corpus decides all of it.** Every input is recorded beside every
answer, so this is the `vargas` shape rather than the `aspect` one: a
formula reproduces a recorded value or it does not. Two of them do not,
and §7 and §8 say what the module does about that.

## 2. Inputs, settings and ports

| input | from |
|---|---|
| the luminaries and the lagna | `ChartFoundation` |
| the arc and the time since sunrise | `ChartFoundation`'s day and birth timing |
| the ascendant at another instant | `astro::houses`, which needs no ephemeris |
| the catalogue | the 49 points with their families, and which eighth is Saturn's on each vara |

**No new settings knob.** Everything here is a rule the corpus decides,
and the one thing the tradition genuinely divides over — where inside
Saturn's eighth Gulika falls — the corpus settles too (§5). A knob for
it would be a knob nothing could choose between on evidence.

## 3. The five the Sun casts are a chain, not five offsets

Dhuma is 133°20′ ahead of the Sun. Vyatipata is Dhuma reflected about
the start of the zodiac. Parivesha is Vyatipata opposed. Indrachapa is
Parivesha reflected. Upaketu is 16°40′ past Indrachapa.

Every one is **exact** on every fixture that records them — not near,
exact, to the last bit of a double. The project's own research page
marks the whole family "verify"; this is that verification.

The chain matters as a chain. Two of its five steps are reflections, and
a reflection does not compose into an offset: **Dhuma, Indrachapa and
Upaketu advance with the Sun while Vyatipata and Parivesha retreat from
it**, because an odd number of reflections stands behind those two.
Written as five constants added to the Sun, two of them would carry the
wrong sign and the code would still look right — so the module states
each direction and a test measures it rather than trusting the reading.

## 4. The Sree lagna is a fraction of the circle

The Moon stands some fraction of the way through its nakshatra; the Sree
lagna is the lagna advanced by that fraction **of a circle**. Exact.

The obvious alternative — the fraction of a *sign* — is wrong by up to
176°, and so are the two readings that start from the lagna's sign
rather than the lagna itself. Nothing here is a matter of taste, which
is worth saying because a reader meeting the formula for the first time
would find all four plausible.

## 5. Gulika begins Saturn's eighth and Mandi ends it

The arc the birth falls in — the daylight, or the night that follows it
— is cut into eight. Which eighth is Saturn's the catalogue already
carries, measured against the recorded panchanga on all 55 days; a night
birth walks the table five weekdays on, the same walk the choghadiya
take.

The research page marks the rest of this "verify" twice over: where
inside the portion the point is read (Parashara at the start, Kalidasa
at the middle), and whether Gulika and Mandi differ at all. The pass
proposed nothing and derived both, by trying all twenty-four candidate
instants against each recorded value:

> **They are two readings of one portion, and it is start against end.**
> Gulika is the ascendant where Saturn's eighth begins, Mandi where it
> ends, on all 71 fixtures, to the precision of the ascendant itself.

That precision is 0.0033° — the SDK's ascendant against the engine's at
the same instant — and the pass measures it first so that the other
claims inherit a known bound rather than an assumed one.

Two things follow for the module. It needs the **ascendant at an instant
that is not the birth**, so `points` takes a house computation and not
just a foundation; and because that computation needs no ephemeris —
the ascendant is the sidereal time and the latitude — the whole module
stays testable without a provider.

## 6. Three lagnas the clock drives, and one time under them

The hora, ghati and pranapada lagnas are one rule at three speeds: start
at the Sun **at birth**, advance 30°, 75° or 60° for each hour since
sunrise. The pranapada adds one thing more — nothing if the Sun stands
in a movable sign, 240° in a fixed one, 120° in a dual one.

Two conventions in that sentence are the corpus's, not a text's, and
both are the kind a reader would get wrong:

1. **From the Sun at birth**, not at sunrise. Most statements of the
   rule say sunrise; the engine uses the birth, and using sunrise is out
   by half a degree.
2. **The hora lagna is 30° an hour** — one sign per hour, or per two and
   a half ghatis — and the ghati lagna 75°, which is one sign per ghati.

## 7. The clock they read is not the ishtakaal beside them

Each of the three is exact on 46 of 71 fixtures and out on the other 25
by an amount **proportional to its own rate**: 0.82° at 30° an hour,
1.63° at 60°, 2.04° at 75°.

That proportionality is the whole finding. A wrong rule is wrong by its
own kind of amount; three rules wrong in proportion to their rates are
three *right* rules reading one wrong clock. The pass confirms it
directly: on every fixture the three imply the same elapsed time as each
other to under a hundredth of a minute, and against that time all three
are exact.

So the disagreement is **one quantity**. The elapsed time the engine's
points are built on differs from the ishtakaal the same fixture records
by at most 1.633 minutes. The SDK uses the ishtakaal it computes, which
is the one it can explain, and the difference is a registry entry rather
than a rule. A harness allows for the bracket; a caller sees at most two
degrees on the fastest of the three.

## 8. The Varnada does not ship

Every recorded Varnada is a **sign** rather than a longitude, which is a
fact about the field worth having. The rule is not: the received reading
— counting from Aries or Pisces by each sign's parity, adding or
subtracting the two counts, and counting the total off again — is wrong
on 31 of 71, and so is every variant the pass could build from the same
parts.

That is crux **C22** met in the data: five published schools disagree
over the Varnada and this engine follows one of them. Naming which would
take that school's own text. So the module ships no Varnada, `Point`'s
catalogue row keeps its `VARNADA_LAGNA` key for whoever cites one, and
the crux stays open. A guess here is a wrong *sign* in a chart —
silently, since every value looks equally plausible.

## 9. The API

```rust
/// Where a derived point stands.
pub struct Derived {
    pub point: Point,               // the catalogue's own key
    pub longitude_deg: f64,
    pub sign: Rashi,
    pub boundaries: Boundaries,     // as `state` and `aspect` carry
}

// the families, each computable on its own
pub mod solar {
    pub const CHAIN: [Point; 5];
    pub fn chain(sun_deg: f64) -> Result<[Derived; 5], Error>;
    pub fn advances(point: Point) -> Option<bool>;   // which way it runs
}
pub mod lagna {
    pub fn hora(sun_deg: f64, hours: f64) -> Result<Derived, Error>;
    pub fn ghati(sun_deg: f64, hours: f64) -> Result<Derived, Error>;
    pub fn pranapada(sun_deg: f64, hours: f64) -> Result<Derived, Error>;
    pub fn sree(lagna_deg: f64, moon_deg: f64) -> Result<Derived, Error>;
    pub const INDU_KALAS: [u8; 7];                   // §13
    pub fn indu_count(lagna: Rashi, moon: Rashi) -> u8;
    pub fn indu(lagna_deg: f64, moon_deg: f64) -> Result<Derived, Error>;
}
pub mod yogi {
    pub fn yogi(sun_deg: f64, moon_deg: f64) -> Result<Derived, Error>;
    pub fn avayogi(sun_deg: f64, moon_deg: f64) -> Result<Derived, Error>;
    pub fn nakshatra(point: &Derived) -> Nakshatra;
}
pub mod eighth {
    pub struct Portion { pub eighth: u8, pub from: JulianDay<Utc>, pub to: JulianDay<Utc> }
    pub fn saturns(arc: Interval, vara: Vara, is_day: bool) -> Result<Portion, Error>;
    // §13: the same rule for any weekday lord, and the four it names
    pub const PORTION_LORDS: [(Point, Graha); 5];
    pub enum Moment { Start, Middle, End }            // Start by default (C405)
    pub fn lords_eighth(lord: Graha, vara: Vara, is_day: bool) -> Option<u8>;
    pub fn lords(arc: Interval, lord: Graha, vara: Vara, is_day: bool) -> Result<Portion, Error>;
    pub fn upagraha(point: Point, arc: Interval, vara: Vara, is_day: bool,
                    moment: Moment, ascendant: &dyn Ascendant) -> Result<Derived, Error>;
    pub fn lorded(arc: Interval, vara: Vara, is_day: bool,
                  ascendant: &dyn Ascendant) -> Result<[Derived; 4], Error>;
}

/// Every point of a founded chart.
pub struct Points { /* … */ }
impl Points {
    pub fn of(foundation: &ChartFoundation, ascendant: &dyn Ascendant) -> Result<Points, Error>;
    pub fn at(&self, point: Point) -> Option<&Derived>;
    pub fn family(&self, family: PointFamily) -> impl Iterator<Item = &Derived>;
    pub fn in_sign(&self, sign: Rashi) -> impl Iterator<Item = &Derived>;
    pub fn house_of(&self, point: Point, foundation: &ChartFoundation) -> Option<u8>;
}

/// What `Points::of` needs that a foundation does not carry: the
/// ascendant at an instant other than the birth.
pub trait Ascendant {
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error>;
}
```

Three shapes are deliberate. Every family computes **on its own**, from
plain longitudes, so a caller with a Sun and nothing else can have the
chain; `Points::of` is the convenience over them and not the only door.
The **`Ascendant` trait** is how the day-division points get what they
need without `points` depending on a provider or on `chart`'s founder —
`chart` implements it, a test implements it in four lines, and the
module stays arithmetic. And every point carries its **`Boundaries`**,
as `state` and `aspect` do, because a point one arcsecond from a sign
edge changes house under a different ayanamsha and the caller is the one
who knows its provider's accuracy.

## 10. Errors

| when | what |
|---|---|
| a longitude that is not finite | `INVALID_ARG` naming the field |
| an elapsed time that is not finite, or beyond a day and a half | `OUT_OF_RANGE` with the bound |
| an arc with no length (a polar day or night) | `UNSUPPORTED`, naming the arc: an eighth of nothing is not an instant |
| an ascendant the provider cannot give | whatever the `Ascendant` implementation returns, unchanged |
| a lord with no weekday (a node, an outer planet) | `UNSUPPORTED`, naming `points.lord` |
| a point no lord's eighth carries, asked of `eighth::upagraha` | `UNSUPPORTED`, naming `points.point` and the five that are |
| the Varnada | `UNSUPPORTED (unsourced)`, naming crux C22 |
| the Vighati lagna | not computed; crux C407 |

## 11. Tests

- **Against the corpus**: every recorded upagraha and special lagna,
  through this crate, at the pass's own bounds — exact for the six that
  are exact, and inside the published bracket for the three the clock
  drives.
- **Exhaustive over the chain**: the five solar points at every degree
  of the Sun, checking that the two reflections retreat and the two
  offsets advance, and that the chain is a bijection of the circle.
- **The eighths**: Saturn's portion for all fourteen vara-and-half
  combinations, that the eight portions partition the arc, and that a
  night walks the table five weekdays on.
- **The verse against the measured table** (§13): the lords' rule gives
  Saturn the catalogue's measured eighth on all fourteen halves of the
  week; the four other lords' eighths are counted by hand for all
  fourteen; the seven lords take seven distinct eighths and leave the
  eighth unlorded.
- **The Indu lagna's printed answer** (§13): the one worked chart a
  translation prints, plus hand counts, the twelve-divides case, and
  all 144 lagna-and-Moon sign pairs.
- **Over every shipped profile**, as `aspect` does.

## 12. Open questions

- **The Varnada** (C22), above.
- **The elapsed time the engine's clock reads** (§7). One quantity, at
  most 1.633 minutes, on 25 of 71 fixtures. Worth another look when the
  conformance harness runs over a real adapter, since that is the first
  thing that could tell a sunrise convention from an arithmetic one.
- **The bhava lagna followed (2026-09-16)**, when BPHS ch. 39 v. 24 named
  it: BPHS ch. 4 vv. 2 to 3 give it as the same rule at one sign in five
  ghatis, 15° an hour, and it starts from the Sun at birth as its siblings
  do. The corpus records none, so what holds it is the arithmetic the
  verses imply — on every chart the SDK reads, it stands half as far round
  from the Sun as the hora lagna (`crates/sdk/tests/rules.rs`).
- **The rest of the catalogue's 49 points.** Bhrigu bindu (C769: only
  modern secondary statements, which leave open which half of the circle
  its midpoint takes), Sahayogi and
  the eight sphutas have catalogue rows and no formula here, and so does
  the Vighati lagna, which no text read defines (C407). The corpus records
  none of them, so each waits on a citation — which is exactly the
  position `aspect` left the sphuta drishti in, and the same answer
  applies: a row without a formula is better than a formula without a
  source. Kala, Mrityu, Ardhaprahara, Yamaghantaka and the Indu lagna
  found theirs on 2026-10-17 (§13).
- **Where inside a portion the four are read** (C405) and **what a sum
  twelve divides counts** for the Indu lagna (C406), §13.
- **Two numberings of BPHS's opening chapters.** This page's §12 entry
  above cites the bhava lagna as ch. 4 vv. 2 to 3; the sanskritdocuments
  e-text puts the special lagnas in ch. 5 (*viśeṣalagnādhyāya*), the
  bhava lagna at vv. 2 to 3, the hora lagna at vv. 4 to 5 and the ghati
  lagna at vv. 6 to 8, one chapter on. `rectification-sources.md` meets
  the same offset between the 1923 print and Jha. Which print the
  ch. 4 citation counts by is worth stating where it is made.

## The arudha padas (2026-09-15)

A first reading of this page put the padas in `jaimini` and said the
corpus recorded none. Both were wrong: the corpus records all twelve of 71
charts under `houses.arudha_padas`, with the sign each first counts to and
whether an exception moved it, and a pada is a function of the lagna's
sign and the grahas' signs, which is exactly what a point is. So
`teistro-points` computes them (`arudha`), measured in
`arudhas-measured.md`: the count from a house's sign to the catalogue's
lord and as far again, a pada in the house or the seventh moving to the
tenth from itself, reproduced on all 852.

## 13. The other lords' eighths, and the Indu lagna (2026-10-17)

The audit of what was buildable without new inputs named five rows of
§12's list: Kala, Mrityu, Ardhaprahara and Yamaghantaka, and the Indu
and Vighati lagnas. Four of the five ship; the Vighati lagna does not.
The corpus records none of them (`crates/points/tests/baseline.rs` says
so by name), so what holds each is its verse, a hand count, and the one
worked chart a translation prints.

### 13.1 Kala, Mrityu, Yamaghantaka and Ardhaprahara

**Source.** BPHS ch. 3, read in the sanskritdocuments Devanagari e-text
(`par0110`, *grahaguṇasvarūpādhyāya*), whose verse numbers are given here;
the 1923 Subodhini print numbers the same verses one higher
(`rectification-sources.md` §1 reads them there).

| v. | key terms | paraphrase |
|---|---|---|
| 66 | *divasān aṣṭadhā bhaktvā vāreśād gaṇayet kramāt* | divide the daylight into eight and count the parts' lords in order from the weekday's lord |
| 67 | *aṣṭamo 'ṃśo nirīśaḥ … rātrim apy aṣṭadhā kṛtvā vāreśāt pañcamāditaḥ* | the eighth part has no lord; the night is divided the same way, counting from the fifth lord from the weekday's |
| 68 | *śanyaṃśo gulikaḥ … ravyaṃśaḥ kālasañjñakaḥ* | Saturn's part is Gulika; the Sun's is called Kala |
| 69 | *bhaumāṃśo mṛtyur … gurvaṃśo yamaghaṇṭakaḥ, saumyāṃśo 'rdhapraharakaḥ, svasvadeśodbhavaḥ sphuṭaḥ* | Mars's part is Mrityu, Jupiter's Yamaghantaka, the saumya's (Mercury's) Ardhaprahara, each placed by what rises at its own place |
| 70 | *gulikeṣṭavaśāl lagnaṃ … gulikaṃ procyate* | the lagna at Gulika's time is what is called Gulika |

So the four are **one rule with Gulika**, a different lord's portion
read the same way. The lords walk in weekday order (Sun, Moon, Mars,
Mercury, Jupiter, Venus, Saturn): on a Sunday's daylight the Sun takes
the first eighth, Mars the third, Mercury the fourth, Jupiter the fifth,
Saturn the seventh; on its night the count starts at Jupiter, the fifth
lord, so Jupiter takes the first eighth and the Sun the fourth. The
Moon's and Venus's eighths carry no named point.

**One table, held to the measured one.** The catalogue's Saturn row
(`Kaala::GulikaKaala`) was measured against the recorded panchanga; the
verse's rule was not. `eighth::lords_eighth` computes the rule for any
weekday lord and a test holds it to the catalogue's Saturn on all
fourteen halves of the week, which is what lets the rule stand for the
four lords nothing recorded. The point-to-lord pairs are one table,
`eighth::PORTION_LORDS`, in the verse's order.

**Where inside the portion: crux C405.** v. 70 reads Gulika "at its
time" and does not say which instant of the portion that is. Three
readings exist:

| reading | evidence |
|---|---|
| the start | the SDK's own Gulika, measured on all 71 fixtures (§5) |
| the end | the 1923 Subodhini print's gloss works Gulika as the eighth's length times the part's ordinal, which is the end (`rectification-sources.md`); Sareen's translation of Uttara Kalamrita, Kanda I section 1 v. 8, reads Gulika at the end of Saturn's part, its example a Monday of 32 ghatis giving 4 × 6 = 24 ghatis |
| the middle | the research page's attribution to Kalidasa; Uttara Kalamrita's Mandi multipliers (26, 22, 18, 14, 10, 6, 2 thirtieths of the day from Sunday) fall inside Saturn's eighth, near its end, at neither edge |

**Default: the start**, because the shipped Gulika is the start and the
verse gives the five one rule; reading Kala at one instant of its
portion and Gulika at another would split a single verse in two. The
reading is a caller's to change (`eighth::Moment`, `eighth::lorded_at`,
`eighth::upagraha`); `Points::of` takes the default.

### 13.2 The Indu lagna

**Source.** Not Phaladeepika: neither the 1937 first edition nor the
1950 second edition of Sastri's translation carries it (both
Internet Archive OCR texts searched, the 1937 index read). It is
**Uttara Kalamrita, Kanda I section 4 ("planets in the several bhavas")
v. 27**, read on the page image of Sareen's print (p. 42), whose verse
gives the numbers as numerals in parentheses; P. S. Sastri's translation
carries the same verse in its ch. 4 with a worked chart.

The verse: the kalās (*kalā*, "rays") of the seven grahas from the Sun
are 30, 16, 6, 8, 10, 12 and 1; add those of the ninth lords
(*bhāgyapa*) from the lagna and from the Moon, divide by twelve (*ina*,
the twelve Suns), and the remainder counted from the Moon's sign
(*vidhoḥ gṛham*) is the sign; a benefic there alone makes a man of
crores, a malefic of thousands, an exalted malefic of crores again. (An
older anonymous translation's OCR reads the series "30, 16, 6, 8, 8, 10,
12"; the Devanagari print's own numerals give Saturn 1, and both later
translations agree.)

**The printed answer.** P. S. Sastri's chart 81 (dated 1905.2.4, drawn
in the South Indian square on p. 98 of his translation, read on the page
image): the lagna at Capricorn 25° and the Moon at Capricorn 17.4,
read as 17°04′. The ninth from both is Virgo, Mercury's: 8 + 8 = 16,
twelve expunged leaves 4, and the fourth sign from the Moon, counting
its own, is Aries, which holds Jupiter. `lagna.rs`'s test takes the
chart's lagna and Moon and asserts Aries; the degree it also asserts is
the SDK's convention below, not the print's.

**What the verse leaves open: crux C406.** Two things.

1. **A sum twelve divides.** 6 + 6, 16 + 8, 12 + 12, 30 + 6 and 30 + 30
   all leave nothing. The default counts the twelfth sign from the Moon,
   which is what reducing by twelves leaves when the sum is a multiple of
   twelve; the alternative counts nothing and lands on the Moon's own
   sign.
2. **A degree.** The verse names a sign. The point is given the Moon's
   own degree in it: with the lagna's sign held, the Indu lagna changes
   sign exactly when the Moon does, so the Moon's distance to its sign
   edge is the point's, and `Boundaries` stays honest. The alternatives
   are the sign's first degree, which would put every Indu lagna on an
   edge, and its middle.

### 13.3 The Vighati lagna does not ship

BPHS ch. 5 of the e-text (*viśeṣalagnādhyāya*) announces three lagnas
in v. 1, *bhāva-horā-ghaṭī-sañjña-lagnāni*, and closes the set at v. 24
with *bhāva-horādi-lagnānām*; the word *vighaṭī* occurs nowhere in the
e-text's 97 chapters. The catalogue row came from a software feature
list, and the unit arithmetic does not suggest a rule: a vighati is a
sixtieth of a ghati, and the ghati lagna already moves a sign a ghati.
So the row stays at mark T, nothing computes it, and crux C407 waits on
the text that defines it — the same answer as the Varnada.

