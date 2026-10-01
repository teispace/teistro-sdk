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
function's. It is the next step's, with the almuten.

## Sect

`essential_dignity` takes the sect; it does not decide it. Ptolemy
(I.VII) says which planets are of the day (the Sun, Jupiter, Saturn) and
which of the night (the Moon, Venus, Mars), and that Mercury is of the
day when matutine and of the night when vespertine. **He does not say
when a chart is a day chart.** The above-the-horizon rule in Ashmand's
edition is Whalley's note, not Ptolemy's text. So the façade's reading
of a chart's sect is a fork with its own crux:

- **C209, when a chart is diurnal**: the Sun's geometric altitude above
  zero, the Sun in houses 7 to 12 of the chart's own division, or the
  Sun above the ecliptic horizon. These part near the horizon and at
  high latitude. The pass that decides it must cite a rank-1 statement,
  Valens or Dorotheus, before any reading becomes a default. Until then
  the façade takes the sect as a request field and reports the one it
  used.

## The order of work

1. `crates/hellenistic` with the types above. Unit tests:
   - Lilly's worked cases (p. 102): the Sun in Aries by night holds
     exaltation and not triplicity; Jupiter in the first six degrees of
     Aries holds its term; Mars in the first ten holds its face.
   - Every shipped table passes its own constructor.
   - The Chaldean rule reproduces I.XXIII's stated totals.
2. `terms-measured.md` moves onto the shipped tables, so the pass keeps
   no copy of what it measures. Its page must not move by a byte.
3. The façade and the chart document: dignities per planet, with the
   sect and the rules that made them.
4. The boundary and the four bindings, under the parity gate.
5. Mutual reception, the almuten (Lilly's own definition, and Ibn
   Ezra's weights once they are read), and Lilly's accidental
   fortitudes (p. 115's second half), each a falsification pass first.

## What is not decided

- **C208**: which printing of Ptolemy's own terms is the default. Both
  ship, and nothing is a default until a third printing (the Greek)
  is read.
- **C209**: when a chart is diurnal.
- **The default `DignityRules`**: `LILLY` is the only complete, scored,
  cited row, because Ptolemy scores nothing. It is named rather than
  implied, and a request names its rules.
