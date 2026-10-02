# Essential dignities (the `hellenistic` module, step 1)

Status: `draft`, 2026-10-01 — written **after** the falsification pass
([`terms-measured.md`](terms-measured.md), `check-terms`) and before any
crate. Phase 7's `hellenistic` module begins here (crux C46, step 1).

## What it is

A planet's **essential dignity** at a degree is what the zodiac itself
grants it there, before anything about the chart's houses or aspects: the
sign may be its house, its exaltation, its triplicity; the degree may lie
in its term or its face; or the sign may be its detriment or fall. A
planet with none of the five dignities is **peregrine**. Lilly scores
them (p. 115, read off the page image): house 5, exaltation 4,
triplicity 3, term 2, face 1; detriment −5, fall −4, peregrine −5.

The Vedic ladder the catalogue's `Dignity` kind names (deep exalted to
deep debilitated) is a different structure. It gives one verdict per
graha. A Western planet can hold several dignities at once: Mars at 5°
Aries is in its house and its face, and Jupiter at 3° Aries by night is
in its triplicity and its term. So this is a new type, and it does not
reuse that kind.

## What the sources decide

Measured in `terms-measured.md`; the page is the authority and this
section only names what each finding decides.

- **The Egyptian terms ship as printed**: the table is well-formed and
  gives the totals I.XXIII states.
- **The Chaldean terms ship as a rule**: one rule reproduces both orders
  the chapter spells out and all seven stated totals. The sect chooses
  whether Saturn or Mercury leads the air triplicity's pair.
- **Ptolemy's own terms ship as two cited tables, neither derived**
  (C208): Lilly's p. 104, and Ashmand's alternate lords on Ashmand's
  first-line ends. The rule fixes the first and last terms and the
  double right's extra degree, and leaves the rest open.
- **Houses and exaltation signs are the catalogue's** (`Rashi::lord`,
  `Graha::exaltation.sign`), held by the pass. A Western dignity reads
  them from there and keeps no copy.
- **Exaltation degrees are not the catalogue's.** The catalogue's are
  the Vedic deep exaltations. Lilly's table differs for the Sun,
  Jupiter and Saturn, so the Western degrees are their own table.
  Lilly counts exaltation through the whole sign (p. 102), so no score
  reads the degree. It is reported for display only.
- **Triplicity lords are a named scheme.** Ptolemy (I.XXI) gives water
  to Venus by day and the Moon by night, together with Mars. Lilly gives
  water to Mars by day and by night, and says so in his prose as well as
  in his table (p. 102). Both are rows. The Dorothean scheme, with its
  participating lord, is not read here and so is not shipped
  (ADR-0018).
- **Faces, detriments and falls are rules**: Lilly's 36 decans in the
  Chaldean order from Mars in Aries, the lord of the opposite sign, and
  the planet exalted opposite.

## The types

`crates/hellenistic` is pure arithmetic over tables. It has no
ephemeris, no chart and no instant: every input is a planet, a
longitude in the chart's own zodiac, and the chart's sect. The module
does not care whether that zodiac is tropical or sidereal. Western
practice is tropical and a sidereal Hellenistic reading is not, and the
chart's frame already says which.

```rust
pub enum Sect { Day, Night }

/// Five terms a sign, each a lord and the degree it ends at.
/// `TermsTable::new` refuses a table whose sign does not hold each of the
/// five planets once and end at 30°, naming the sign.
pub struct TermsTable([[(Graha, u8); 5]; 12]);

impl TermsTable {
    pub const EGYPTIAN: TermsTable;           // Tetrabiblos I.XXIII
    pub const PTOLEMAIC_LILLY: TermsTable;    // Lilly 1647, p. 104
    pub const PTOLEMAIC_ASHMAND: TermsTable;  // I.XXIV, the alternates' lords
    pub fn chaldean(sect: Sect) -> TermsTable; // I.XXIII, the rule
    pub fn new(cells: [[(Graha, u8); 5]; 12]) -> Result<TermsTable, Refusal>;
    pub fn lord_at(&self, longitude: f64) -> Graha;
}

pub enum Terms { Table(TermsTable), Chaldean }
pub enum Triplicities { Ptolemy, Lilly }

pub struct Scores { house, exaltation, triplicity, term, face,
                    detriment, fall, peregrine: i8 }   // Scores::LILLY

pub struct DignityRules { terms: Terms, triplicities: Triplicities }
// DignityRules::LILLY: Lilly's terms, Lilly's triplicities.

pub struct EssentialDignity {
    house: bool, exaltation: bool, triplicity: bool, term: bool,
    face: bool, detriment: bool, fall: bool,
}
impl EssentialDignity {
    pub fn peregrine(&self) -> bool;   // none of the five dignities
    pub fn score(&self, scores: &Scores) -> i16;
}

pub fn essential_dignity(planet: Graha, longitude: f64, sect: Sect,
                         rules: &DignityRules) -> EssentialDignity;
```

The reasons for this shape:

- **A dignity is a set of flags, not a score.** The score is one reading
  of the flags, and Lilly's is one school's. A consumer with another
  weighting passes its own `Scores` and does not re-derive the flags.
- **A consumer's own terms table is a value, refused if malformed.**
  Under the no-dead-ends mandate, `TermsTable::new` checks what the pass
  checks: each planet once a sign, ending at 30°. So a school the SDK
  does not ship is one constructor away, and a typo is refused by sign.
- **The Chaldean terms are a rule, not three tables.** They differ by
  sect, so they are a variant that takes the sect, not a table frozen
  at one sect.
- **Only the five planets take terms.** `lord_at` answers one of the
  five. `essential_dignity` gives the Sun and the Moon no term ever, and
  says so in the doc, rather than looking them up and finding nothing.

Mutual reception (Lilly scores it as house or exaltation) needs the
other planet's place, so it is a chart-level question and not this
function's (§Reception).

## Sect

`essential_dignity` takes the sect; it does not decide it. Ptolemy
(I.VII) says which planets are of the day (the Sun, Jupiter, Saturn) and
which of the night (the Moon, Venus, Mars), and that Mercury is of the
day when matutine and of the night when vespertine. **He does not say
when a chart is a day chart.** The above-the-horizon rule in Ashmand's
edition is Whalley's note, not Ptolemy's text.

Valens decides it (C209, rank 1, Riley's translation). He reckons "the
hemisphere above the earth" in degrees from the Descendant and the
Ascendant (Book I, 51K–52K, worked on a Moon at Libra 26° under a
Capricorn 24° Ascendant). In Book IX (362K–363K) a birth is "during the
day" or "after sunset", by a day hemisphere and a night hemisphere. So
the horizon decides, and whole-sign houses do not. The Sun stands on the
ecliptic, which meets the horizon at the Ascendant and the Descendant.
Wherever the zodiac rises in order, three readings are one rule to
within the Sun's latitude (under 1″):

- the Sun above the ecliptic horizon, Valens's degrees from the
  Ascendant;
- the Sun's geometric altitude above zero;
- the Sun in houses 7 to 12 of a quadrant division.

Inside the polar circles part of the zodiac rises backwards. There the
half of the ecliptic below the Ascendant in longitude is no longer the
half that has risen, so only the altitude still says whether the Sun is
up. [`sect-measured.md`](sect-measured.md) finds the degrees and the
altitude agree at all 53 recorded births outside the circles, and part
at the polar-night birth inside them.

The text cannot part the geometric horizon from the apparent one, where
refraction and the Sun's limb move sunrise by minutes; the same page
measures 3 to 25 minutes at each end of the day. So the façade reads a
chart's sect by a named rule:

```rust
pub enum SectRule {
    Horizon,   // the default: the Sun's centre above the true horizon
    Daylight,  // the chart's own sunrise to sunset, under its sunrise convention
    Day,       // the caller's: every chart a day chart
    Night,     // the caller's: every chart a night chart
}
impl Sect {
    pub fn from_altitude(sun_altitude_deg: f64) -> Sect;
}
```

The altitude is read from the chart's own Midheaven, its obliquity and
the place's latitude (`teistro_astro::sky::altitude_by_midheaven_deg`).
The Midheaven's right ascension is the sidereal time the chart was cast
for, so no clock is read again. A sidereal chart is shifted back to the
equinox by the Sun's own two longitudes, so the answer does not depend
on the zodiac. A caller's sect is two unit members rather than a member
carrying a value, so every binding spells the rule as one string. The
answer reports the sect it used and the rule that chose it.

## The façade (step 3)

```rust
pub struct DignityRequest { sect: SectRule, rules: DignityRules, scores: Scores }
// DignityRequest::default(): Horizon, DignityRules::LILLY, Scores::LILLY

pub struct Dignities {
    sect: Sect, sect_rule: SectRule, rules: DignityRules, scores: Scores,
    planets: [PlanetDignity; 7],   // in the Chaldean order
}
pub struct PlanetDignity { planet: Graha, longitude_deg: f64,
                           dignity: EssentialDignity, score: i16 }

impl ChartArea<'_> {
    pub fn dignities(self, chart: &Document, request: &DignityRequest)
        -> Result<Dignities, Error>;
}
```

It needs **no ephemeris** unless the chart's angles were its provider's.
The Sun, the planets, the angles and the day part are all on the
founded chart. The request is a value with a
default and builders, and every knob in it is reported back in the
answer.

## The boundary (step 4)

A chart request's `dignities_json` is the request record as serde writes
it, with every member optional: `{"sectRule": "DAYLIGHT", "rules":
{"terms": "EGYPTIAN", "triplicities": "PTOLEMY"}, "scores":
{"peregrine": 0}}`.
`DignityRequest::from_json` reads it strictly, so a misspelt key is
refused at its path (`dignities.scores.peregrin`). A member left out takes
the default's value; that holds inside `rules` and `scores` too, so
`{"scores": {"peregrine": 0}}` changes one score and keeps Lilly's other
seven.

A table of the caller's own crosses as `{"terms": {"TABLE": [[{"lord",
"end"}, …], …]}}`: twelve signs of five terms, a lord in either spelling
(`MARS` or `graha.MARS`). `TermsTable::new` checks it, and a malformed
table is refused at `dignities.rules.terms.TABLE`. So no system of terms
is out of a binding's reach.

Two sections answer it:

- **`dignities`**, a row a chart: the sect, the rule that read it, the
  terms and the triplicities, as `TsSect`, `TsSectRule`, `TsTerms` and
  `TsTriplicities`. The eight scores follow, as `i8`. The answer reports
  what was applied, defaults included, so a binding never infers a rule
  it did not see.
- **`dignity_planets`**, seven rows a chart in the Chaldean order: the
  planet, its longitude and the seven flags as 0 or 1, then the score as
  `i16`. Peregrine is left out because it is derived (none of the first
  five flags is set). Each binding derives it once, as Rust's
  `EssentialDignity::peregrine` does.

**`TsTerms::TABLE` reports a table without repeating it.** The table is
the caller's own, and sixty cells a chart would cost bytes to say what
the request already holds. It is the one place an answer cannot be fed
back as a request unchanged. Node's types make that a compile error
(`terms: 'TABLE'` is not a `DignityRequest`), and the boundary refuses it
by `dignities.rules.terms.TABLE`.

Node, Python and Dart each rebuild Rust's `Dignities`, with the planet
as a catalogue member and `peregrine` beside the flags. Parity asks every
runner for a request with each knob turned from its default (`DAYLIGHT`,
`EGYPTIAN`, `PTOLEMY`, a peregrine score of 0), and the five runners agree
on every value. The ABI test holds every cell to the façade's own answer
to the bit, and the keys test holds the four enums to serde's spelling.

## Reception (step 5)

Lilly (p. 112, read off the page image) calls it reception when two
planets "are in each others dignity": by house, the strongest, or "by
triplicity terme or face, or any essentiall dignity". He works three
examples, and each holds on the shipped tables:

- the Sun in Aries and Mars in Leo, by house;
- Venus in Aries and the Sun in Taurus, by triplicity, "if the Question
  or Nativity be by day";
- Venus at 24° Aries and Mars at 16° Gemini, by term.

Whose dignity a planet stands in is the other planet's
`essential_dignity` at its place, so reception needs no table of its own.
`Dignities.receptions` lists every pair in which each planet stands in
at least one of the other's five dignities. Each `Reception` carries both
sides whole (`first_in`, `second_in`), so one record answers both
questions:

- **mutual**: the same kind both ways (`Reception::mutual`, Lilly's three
  examples);
- **mixed**: different kinds each way, which Lilly does not name.

**Scoring (C210).** Lilly's table (p. 115) scores a planet "in mutual
reception … by house" as its own house, 5, and "reception by exaltation"
as its exaltation, 4, and names no other kind. So
`PlanetDignity::reception` adds the request's house score for a mutual
reception by house and its exaltation score for one by exaltation, and
nothing else. It is kept apart from `score`, never folded into it, so a
reader who scores mixed reception, or reception by the lesser
dignities, adds what the pair carries. A received planet stays peregrine,
because Lilly defines peregrine by the planet's own dignities (p. 112).

**One partner a kind, but for Ptolemy's water.** A place has one lord,
one exalted planet, one term lord and one face lord. So a planet is
received by any of those kinds by one partner at most, and a kind's
score is earned once. Ptolemy's water triplicity is ruled jointly
(Venus or the Moon, with Mars), so there a planet can be received by
triplicity by two partners, and both pairs are listed.

At the boundary, `dignities` gains `reception_count`. `dignity_planets`
gains `reception`. A third section, `dignity_receptions`, holds each
chart's pairs, ragged by that count. Each pair is a row: `first` and
`second`, then the seven flags of each side. [`reception-measured.md`](reception-measured.md)
holds Lilly's examples and counts each kind over the corpus.

## Accidental fortitudes (step 5)

Lilly's "ready Table" (p. 115) gives, beside the essential dignities, a
planet's **accidental** fortitudes and debilities: its house, its
motion, its place about the Sun, its partile aspects, a siege and three
fixed stars. They are read in `accidental_dignities`, from the seven's
longitudes (the same ones the essential dignities read, held once in
`ChartSky`) and an `AccidentalSky`:

- the seven's daily motions;
- the twelve cusps and the division they are of;
- the North Node;
- Regulus, Spica and Algol at their places of date.

As with the essential dignities there is no ephemeris here. Each planet
gets a `PlanetAccidents`: the house it is counted in, every other line
it meets (`Accident`), and its fortitudes and debilities summed apart,
the way Lilly prints them. `AccidentalScores::LILLY` is the table's
worth for each line, and every orb Lilly states is a field of
`AccidentalRules::LILLY`.

What the text decides (pp. 33, 106, 113–115, read off the page images):

- **House.** The Midheaven or Ascendant 5; the 7th, 4th and 11th 4; the
  2nd and 5th 3; the 9th 2; the 3rd 1; the 12th −5; the 8th and 6th −2.
  A planet within five degrees of a cusp counts in the house "to whose
  Cusp he is neerest" (p. 33), before or after it (C214).
- **Motion.** Direct 4, retrograde −5, both void for the Sun and the
  Moon. Swift 2 and slow −2, against Lilly's mean motions (Saturn 2′01″,
  Jupiter 4′59″, Mars 31′27″, the Sun, Venus and Mercury 59′08″, the
  Moon 13°10′36″).
- **Orientality.** Oriental means behind the Sun in longitude, rising
  before him. It is worth 2 to Saturn, Jupiter and Mars and −2 to Venus
  and Mercury, and occidental the reverse. The Moon is increasing (2)
  from conjunction to opposition and decreasing (−2) after. The Sun has
  none.
- **The Sun.** Each planet but the Sun holds exactly one of four:
  - cazimi, within 17′ (5);
  - combust, within 8°30′ and in the Sun's own sign (−5, C211);
  - under the beams, within 17° (−4, C212);
  - free from combustion (5).

  The merchant's figure scores Venus in cazimi as cazimi alone, not
  "free" as well.
- **Partile aspects.** Partile means the same degree of signs the
  aspect apart (C216). The lines are:
  - conjunction with Jupiter or Venus 5, with the North Node 4;
  - trine to Jupiter or Venus 4, sextile 3;
  - conjunction with Saturn or Mars −5, with the South Node −4;
  - opposition to Saturn or Mars −4, square −3.

  Each line counts once however many bodies hold it, and a planet is
  never in aspect with itself.
- **Besieged** between the bodies of Saturn and Mars, −5 (C215).
- **Fixed stars**, each within five degrees (C213): Regulus 6, Spica 5,
  Algol −5.

**The acceptance test** is two figures whose tallies Lilly prints line
by line:

- Chapter XXVIII's "Rich or Poore" (pp. 177–180);
- Book III's English merchant (pp. 742–745).

Both are fed as printed: positions, cusps, motions, and the stars at
their places of date. Each planet's computed lines and net are then held
to the tally cell by cell. Under the stated rules every printed line is
reproduced except these, which the tests list:

- The beams. In the first figure Jupiter (15°39′ from the Sun), Mercury
  (14°35′) and the Moon (15°57′) are within 17°, yet the tally prints
  each as "free from combustion" (C212).
- Omissions. The first figure's Jupiter is oriental, unprinted. The
  merchant's Mercury, combust, is oriental and peregrine, neither
  printed. The first figure's Sun holds its fire triplicity by day,
  unprinted (C217).
- The eighth house. The merchant's tally charges it 4 where the table
  charges 2, for the Sun, Venus and Mercury (C217).

The stars of date matter. Lilly prints Regulus at "24 Leo", which would
put the first figure's Moon (19°07′ Leo) within five degrees. Of date,
Regulus stood near 24°43′, 5°36′ from her, and the tally gives her no
Regulus.

**At the façade**, `ChartArea::fortitudes(chart, &FortitudeRequest)`
answers both halves of the table: `Fortitudes` holds the chart's
`Dignities`, the accidental lines and `net(planet)`, Lilly's sum of
the two. The request is the essential `DignityRequest` with the
accidental rules and scores beside it, read strictly from
`{"dignities": …, "rules": …, "scores": …}`. The chart fills the
`AccidentalSky`, which the answer returns as read:

- each planet's daily motion, from the chart;
- the cusps of Lilly's Regiomontanus, or of the division a profile names
  under `houses.module_overrides.hellenistic`, the KP mechanism;
- the chart's own North Node;
- Regulus, Spica and Algol at their apparent places of date, from the
  SDK's star catalogue.

Everything is in the chart's zodiac. Like the dignities it needs no
ephemeris. A chart whose angles were its provider's own is refused,
because the sphere cannot rebuild its cusps. `dignities` stays as it was
for such charts. [`fortitudes-measured.md`](fortitudes-measured.md)
reads the corpus in Lilly's tropical zodiac and counts what each crux's
rival would move.

**At the boundary**, a chart request's `fortitudes_json` is the
`FortitudeRequest` record, read strictly by `FortitudeRequest::from_json`,
so a misspelt key is refused at its path (`fortitudes.rules.beamDeg`).
Its essential half answers in the sections `dignities_json` fills
(`dignities`, `dignity_planets`, `dignity_receptions`), from the
fortitudes' own `Dignities`. Asking for both is refused by name, because
two requests for one table could disagree about it. Four sections carry
the accidental half:

- **`fortitudes`**, a row a chart: the house division (`HouseSystem`),
  the North Node and the three stars as read, then every rule as applied.
  These are the three solar orbs, the sign clause, the cusp and star orbs,
  and the partile and siege readings as `TsPartile` and `TsSiege`, each
  with its orb or span. Then the 26 line scores, as `i8`. A binding
  never infers a rule it did not see.
- **`fortitude_houses`**, twelve rows a chart: each cusp and the house's
  score.
- **`fortitude_planets`**, seven rows a chart in the Chaldean order: the
  planet, its daily motion and the mean motion it was judged against,
  its house, its fortitude and debility, and how many accidents it holds.
  The mean motion sits here rather than on the chart row because it is
  the planet's own.
- **`fortitude_accidents`**, ragged by those counts: each accident as
  `TsAccident` and the points it scored. The points are reported because
  orientality scores by the planet (Saturn's +2 is Venus's −2), and a
  binding should not need that rule to total a line.

Lilly's net is not a column: each binding sums it as `Fortitudes::net`
does, from the four values the sections already carry.

## The almuten (step 5)

Lilly defines two almutens on one page (p. 49, Chapter VI's list of
terms, read off the page image). That **of a house** is the planet "who
hath most dignities in the Signe ascending or descending upon the
Cusp". That **of a figure** is the planet "most powerfull in the whole
Scheame" in essential and accidental dignities together. His merchant
(Book III, p. 742, the second fortitudes figure) is called "Almuten of
the Geniture" in Venus's name twice, and Venus has that figure's
greatest net: 16 in his tally and 18 by his table (C217). That is the
acceptance value.

Chapter CV (pp. 530–532, read off the page images) reports a rival for
the lord of the geniture. It sums the *essential* dignities over five
places, the ascendant, the mid-heaven, the Sun, the Moon and the Part of
Fortune, and calls the runner-up "partaker" in the judgment. Lilly
calls it "rationall" and keeps his own. Both ship, because a consumer
reading Ibn Ezra's school needs the second.

```rust
/// The seven's dignities at one place: what each would score there
/// by its five dignities, debilities left out (a planet is not in a
/// place it rules).
pub struct Almuten { pub totals: [i16; 7] }   // Chaldean order
impl Almuten {
    pub fn total(&self, planet: Graha) -> Option<i16>;
    pub fn almutens(&self) -> Vec<Graha>;      // every planet tied at the top
}
pub enum PlaceReading { Degree, Sign }        // C218
pub fn almuten_of(longitude_deg, sect, &DignityRules, &Scores, PlaceReading) -> Result<Almuten>;
pub fn almuten_of_places(&[f64], ...) -> Result<Almuten>;  // summed
pub fn part_of_fortune(asc, sun, moon, FortuneRule) -> f64; // C220
impl Fortitudes { pub fn almuten(&self) -> Almuten }       // the nets
```

- **A house's almuten** is the almuten of its cusp. By the degree
  (`PlaceReading::Degree`, the default) it counts all five of Lilly's
  dignities, two of which (term and face) only a degree has. By the
  sign it counts the house, exaltation and triplicity alone (C218).
- **The figure's almuten** is the greatest `net`, essential and
  accidental, so it needs nothing the fortitudes do not already hold.
- **The places' almuten** sums the essential dignities over a list of
  places. Lilly's list is the five above, and the list is the caller's,
  so a reader who adds the prenatal syzygy passes six. The ascendant and
  mid-heaven are read from the angles, never from cusps, which are not
  the angles under whole-sign or equal houses.
- **The Part of Fortune** is Lilly's: ascendant + Moon − Sun, "by day
  or night" (pp. 143–144). His worked example is the acceptance value:
  the Moon at 21°18′ Virgo less the Sun at 4°18′ Aries is 5 signs 17°,
  which added to 23°27′ Leo rising puts Fortune at 10°27′ Aquarius. He
  reports the night reversal ("Some have used to take ⊗ in the night
  from the ☽ to the ☉") and sets it aside for Ptolemy's rule, which
  every practitioner of his day followed. So C220 is decided to the
  text, and the reversal, which the Tajika Punya saham also uses,
  ships as a knob.
- **Ties** are reported, never broken (C219). Lilly gives no rule for
  them; "posited best, and elevated most" (p. 532) is a judgment in
  words, so it is not encoded as a score.

The acceptance tests are the merchant's figure almuten (Venus) and the
Part of Fortune's worked example (10°27′ Aquarius). A measured page counts how
often the three almutens agree over the corpus, how often each is tied,
and what each crux's rival moves, before anything reaches the façade.

## The order of work

1. `crates/hellenistic` with the types above. Unit tests:
   - Lilly's worked cases (p. 102): the Sun in Aries by night holds
     exaltation and not triplicity; Jupiter in the first six degrees of
     Aries holds its term; Mars in the first ten holds its face.
   - Every shipped table passes its own constructor.
   - The Chaldean rule reproduces I.XXIII's stated totals.
2. `terms-measured.md` moves onto the shipped tables, so the pass keeps
   no copy of what it measures. Its page must not move by a byte.
   **Done**: every earlier row is unchanged. Four rows were added, holding
   the shipped exaltation degrees, triplicity schemes and Ashmand table
   to their transcriptions. One of them finds the Tajika decanate lord
   (`drekkana_lord`) to be Lilly's face on all 36 decans. So the two
   traditions share one rule, which still lives in two crates. It belongs
   in the catalogue once a third reader needs it, and the gated row is
   what will hold that move.
3. The façade and the chart document: dignities per planet, with the
   sect and the rules that made them (§The façade).
4. The boundary and the four bindings, under the parity gate.
   **Done** (§The boundary).
5. Mutual reception, the almuten (Lilly's own definition, and Ibn
   Ezra's weights once they are read), and Lilly's accidental
   fortitudes (p. 115's second half), each a falsification pass first.
   **Reception done** (§Reception, C210). **The accidental fortitudes'
   doctrine is done** (§Accidental fortitudes, C211–C217), held to two
   printed figures, and so is its façade (`ChartArea::fortitudes`,
   `fortitudes-measured.md`), and so is its boundary (§At the boundary:
   sections 63–66 in every binding, under the parity gate). Next is the
   almuten, which needs both halves.

## What is not decided

- **C208**: which printing of Ptolemy's own terms is the default. Both
  ship, and nothing is a default until a third printing (the Greek)
  is read.
- **C209** is decided to the horizon (§Sect), measured in
  [`sect-measured.md`](sect-measured.md).
- **The default `DignityRules`**: `LILLY` is the only complete, scored,
  cited row, because Ptolemy scores nothing. It is named rather than
  implied, and a request names its rules.
