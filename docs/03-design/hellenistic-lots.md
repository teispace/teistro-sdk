# The lots (the `hellenistic` module, step 6)

Status: `built`, 2026-10-02 — written from Valens's text before the
crate, and corrected by building it (Love and Necessity under his rule).

The lots are points a chart does not place: each is the distance between
two of its points, counted from a third. The Part of Fortune is the one
every later author keeps, and it already ships as the almuten's fifth
place (`part_of_fortune`, C220); this page generalises it into the lots
Valens gives, under the rules his text raises.

## What the sources decide

Valens (Riley's translation, *Anthologies*) is the source. Each lot is
read off its own chapter, and each is the same shape: **the distance
from one point to another, counted from a third**, with the order often
reversed for a night birth.

| Lot | By day | By night | Counted from | Where |
|-----|--------|----------|--------------|-------|
| Fortune | Sun to Moon | Moon to Sun | Ascendant | II.22, III.11 (see `FortuneRule`) |
| Daimon | Moon to Sun | Sun to Moon | Ascendant | II.22 |
| Basis | the shorter arc between Fortune and Daimon | the same | Ascendant | II.22 |
| Love | Fortune to Daimon | Daimon to Fortune | Ascendant | IV.25, a marginal note |
| Necessity | Daimon to Fortune | Fortune to Daimon | Ascendant | IV.25, a marginal note |
| Exaltation | Sun to Aries | Moon to Taurus | Ascendant | II.18 |
| Debt | Mercury to Saturn | the same | Ascendant | II.23 |
| Theft | Mercury to Mars | Mars to Mercury | **Saturn** | II.24 |
| Deceit | Sun to Mars | Mars to Sun | Ascendant | II.25 |
| Foreign lands | Saturn to Mars | the same | Ascendant | II.29 |
| Father | Sun to Saturn | **Venus to Moon** | Ascendant | II.31, from Timaios |
| Marriage | Jupiter to Venus | Venus to Jupiter | Ascendant | II.37 |
| Brothers | Saturn to Jupiter | Jupiter to Saturn | Ascendant | II.40 |
| Crisis | Saturn to Mars | Mars to Saturn | Ascendant | V.1, the crisis-producing place |

Three things in the table are not the common shape, and each is the text:

- **Basis** has no day and night. Valens says the distance "will not
  exceed" seven signs and is taken "from the nearest Lot to the other":
  the shorter arc. Daimon is Fortune reflected in the Ascendant, so the
  shorter arc is the same by day and by night.
- **Love and Necessity** reverse by night, and so do Fortune and Daimon
  under Valens's rule; the two reversals cancel, so under his rule each
  stands where it would by day. Under Lilly's Fortune they change places.
- **Theft** is counted from Saturn, not the Ascendant.
- **The Father by night** is not the day's reversal. The text gives the
  Sun to Saturn by day, "(some" take the Sun to Jupiter), and Venus to the
  Moon by night.

What the text leaves open, and what ships:

- **Fortune by night in Valens (C221).** Lilly counts
  the same way day and night; Valens reverses it at II.22, and at III.11
  prefers a third reading: by night, from the Moon to the Sun while the
  Moon is above the earth, "until the time it sets", and from the Sun to
  the Moon after it has set. All three are `FortuneRule` members.
  `FortuneRule::ReversedWhileMoonUp` reads the Moon's own altitude, its
  latitude included, the way `SectRule::Horizon` reads the Sun's; it does
  not read the ecliptic hemisphere from the Ascendant, which is not the
  horizon inside the polar circles (C209). The lots default to
  `ReversedByNight`, the reading the lots built on Fortune presume
  (Daimon is its mirror); the almuten keeps Lilly's.
- **Daimon under each rule.** Daimon is Fortune reflected in the
  Ascendant: whenever Fortune is counted from the Sun to the Moon, Daimon
  is counted from the Moon to the Sun. Valens states it only for his
  II.22 reading; the mirror is what makes Basis sect-free, and it is how
  Daimon is read under the other two.
- **Exaltation's end points (C222).** The text says "to Aries, which is
  the sun's exaltation", and "to Taurus". The lot counts to the
  exaltation degrees, Aries 19° and Taurus 3°, the points the dignity table
  holds. The sign's first degree is a formula away
  (`LotPoint::Degrees(0.0)`), shown in its documentation. Valens's worked
  example counts whole signs ("from the moon to Taurus is eleven signs")
  and gives no degrees, so it cannot tell the two apart.
- **Lots that depend on the native, not the sect.** Children (II.39,
  Jupiter to Mercury for a man, to Venus for a woman) and the second
  Marriage lot (II.38) are not in the catalogue. Each is one `LotFormula`
  the caller writes for the native it has.
- **Other lists.** The later authors' lists of the seven Hermetic lots
  differ from Valens's IV.25 notes for Love and Necessity. None is on
  this machine, so none is encoded and none is cited.

Valens's two worked examples of Fortune by night (III.11) are the
acceptance tests, at the level of the sign, which is all they give: Moon
and Ascendant in Pisces with the Sun in Cancer put Fortune in Cancer, and
Moon and Ascendant in Virgo with the Sun in Aquarius put it in Aquarius.
Both are reversals; Lilly's rule puts them in Scorpio and Aries.

## Why not the sahams' evaluator

The Tajika sahams (`teistro-tajika`) are the same arithmetic, a − b + c,
and the same day and night pair. Their evaluator is not shared because
what it adds is Tajika: the added sign, Sripati's house points as
factors, and a memo keyed by the source's forty-one. What the lots share
with it is one line of arithmetic and the shape of a formula, which is
here as `LotFormula` over `LotPoint`, so a caller's own lot is written
the way a caller's own saham is.

## The types

```rust
pub enum Lot { Fortune, Daimon, Basis, Love, Necessity, Exaltation, Debt,
               Theft, Deceit, ForeignLands, Father, Marriage, Brothers, Crisis }

pub enum LotPoint { Ascendant, Midheaven, Planet(Graha), Lot(Lot), Exaltation(Graha), Degrees(f64) }
pub enum Distance { Forward, Shorter }
pub struct LotArc { pub from: LotPoint, pub to: LotPoint, pub counted_from: LotPoint, pub distance: Distance }
pub struct LotFormula { pub day: LotArc, pub night: LotArc }

pub enum FortuneRule { DayAndNight, ReversedByNight, ReversedWhileMoonUp }
pub struct LotRequest { sect_rule: SectRule, fortune: FortuneRule }   // LotRequest::VALENS
pub struct LotSky { pub chart: ChartSky, pub ascendant_deg: f64, pub midheaven_deg: f64 }

pub struct LotPlace { pub longitude_deg: f64, pub sign: Rashi, pub lord: Graha, pub house: House }
pub struct PlacedLot { pub lot: Lot, pub place: LotPlace }
pub struct LotReading { pub sect: Sect, pub request: LotRequest, pub fortune_reversed: bool,
                        pub lots: Vec<PlacedLot> }

impl Lot { pub const fn formula(self) -> LotFormula }
pub fn lots(sky: &LotSky, which: &[Lot], request: LotRequest) -> Result<LotReading>;
pub fn lot_place(sky: &LotSky, formula: &LotFormula, request: LotRequest) -> Result<LotPlace>;
```

`ChartSky` carries the Moon's altitude beside the Sun's, so the almuten's
Fortune and the lots' read the same horizon. The SDK reads both
altitudes off the chart's own angles, latitude included, and offers
`ChartArea::lots`, `lots_with_request` and `lot_place`.

`Lot::formula` is the table above, as data; Fortune's and Daimon's night
arcs are their reversals, and `FortuneRule` decides when those two take
them. A lot built on Fortune (Basis, Love, Necessity) follows the rule
through its `LotPoint::Lot` factors, and each lot is computed once
however many read it.

## The order of work

1. **The lots in Rust and the SDK.** The catalogue, the evaluator, the
   third Fortune rule (the almuten reads it too), and `ChartArea::lots`.
   Done.
2. **Measured over the corpus** ([`lots-measured.md`](lots-measured.md),
   `check-lots`). Done. Lilly's rule moves Fortune's sign on 30 of the 32
   night births, and III.11 parts from II.22 on 20 of the 22 whose Moon
   had set. The premise of reading the Moon by its altitude is counted
   too: its ecliptic hemisphere from the ascendant says otherwise on 3
   of the 32.
3. **The boundary.** Done. A chart request's `lots_json` (`{ sectRule,
   fortune }`, Valens's when empty) answers sections 67 and 68: `lots`, a
   row a chart with the sect, the rules and whether Fortune was
   reversed, and `lot_places`, the fourteen a chart in `TsLot`'s order
   with each one's longitude, sign, lord and house. Every binding reads
   them back as `lots`, its request fed back as it stands, under the
   parity gate. The lot names are a `TsLot` enum and not a catalogue
   kind: the request asks for every lot and names none, so no consumer
   yet names one, which is what earns a kind. A caller's own formula
   stays in Rust (`lot_place`) until a binding asks to write one.
