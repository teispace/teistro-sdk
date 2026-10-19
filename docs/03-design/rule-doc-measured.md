# A rule in prose, measured

Status: `generated` by `cargo xtask rule-doc` over the shipped rule
packs and the conformance corpus's own rules, 2026-09-20. Do not edit:
`check-rule-doc` regenerates this page and fails on any difference. The
design it measures is [`rule-doc.md`](rule-doc.md).

## What was rendered

3045 rules: 17 in `DOSHAS`, 8 in `YOGAS`, 4 in `GANDANTAS`, 184 in `ARISHTAS`, 1339 in `READINGS`, 836 in `NABHASAS`, 657 in `the corpus's own`. Their 12 218 conditions are written 4546 ways, which say 4542 things, and the renderer gives those 4542 sentences.

The 2388 shipped rules are what a consumer evaluates; the corpus's own
are rendered beside them because they exercise predicates no shipped
pack uses.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| no two conditions that differ in meaning read alike | **holds** | 0 of 4542 disagree |
| every condition of every rule renders | **holds** | 0 of 12218 disagree |

No rendering is shared by two conditions that mean different things, so
a change to what a rule asks changes its prose.

## Two spellings, one sentence

4 renderings read alike because they say one thing twice. This is a finding about the language rather than about the prose, and the three the shipped packs held — a combinator with one condition in it — were simplified when this pass first found them; what remains is the corpus's own rules, which are a recording and are not edited.

- `the AK stands in the 1st house`
  - `{"type":"planet-in-house","planet":{"karaka":"AK"},"houses":[1]}`
  - `{"type":"chara-karaka-in-house","karaka":"AK","houses":[1],"karakaScheme":7}`
- `the AmK stands in the 1st, 5th or 9th house`
  - `{"type":"planet-in-house","planet":{"karaka":"AmK"},"houses":[1,5,9]}`
  - `{"type":"chara-karaka-in-house","karaka":"AmK","houses":[1,5,9],"karakaScheme":7}`
- `the lord of the 11th house stands in a kendra`
  - `{"type":"lord-of-house-in-kendra","houseRuled":11}`
  - `{"type":"and","conditions":[{"type":"lord-of-house-in-kendra","houseRuled":11}]}`
- `the lord of the 7th house stands in the 10th house`
  - `{"type":"lord-of-house-in-house","houseRuled":7,"houseOccupied":10}`
  - `{"type":"and","conditions":[{"type":"lord-of-house-in-house","houseRuled":7,"houseOccupied":10}]}`

## The language, kind by kind

| kind | occurrences | as it reads |
|---|---|---|
| `and` | 958 | MOON aspects VENUS and VENUS aspects MOON |
| `or` | 578 | MARS is combust or MARS is retrograde |
| `not` | 483 | it is not the case that MARS is combust |
| `planet-in-house` | 1365 | SUN stands in the 1st house |
| `planet-in-sign` | 1224 | SUN stands in LEO |
| `planet-dignity` | 415 | the dignity of SUN is exalted |
| `planet-in-kendra` | 186 | SUN stands in a kendra |
| `planet-in-trikona` | 15 | SUN stands in a trikona |
| `planet-in-kendra-from` | 39 | MARS stands in a kendra from MOON |
| `lord-of-house-in-kendra` | 70 | the lord of the 1st house stands in a kendra |
| `lord-of-house-in-house` | 487 | the lord of the 1st house stands in the 1st house |
| `planet-conjunct` | 477 | MARS and SUN share a sign |
| `planet-in-house-from` | 487 | MARS stands in the 1st from SUN |
| `no-planet-in-houses-from` | 22 | no graha stands in the 2nd from SUN, excepting MOON |
| `mutual-exchange` | 127 | the lords of the 1st and 2nd houses stand each in the other's sign |
| `lord-conjunct-lord` | 40 | the lord of the 1st house shares a sign with the lord of the 5th |
| `all-planets-between-nodes` | 14 | the seven classical grahas all stand between the nodes |
| `occupied-sign-count` | 16 | SUN, VENUS and MARS occupy exactly 3 signs |
| `all-classical-grahas-in-houses` | 66 | the seven classical grahas stand only in the 1st or 4th house, filling every one of them |
| `n-grahas-conjunct-with` | 15 | at least 4 classical grahas share the sign of SUN, it among them |
| `chara-karaka-in-house` | 34 | the AK stands in the 1st house |
| `planet-combust` | 61 | SUN is combust |
| `planet-retrograde` | 24 | MARS is retrograde |
| `planet-aspects-planet` | 918 | KETU aspects SUN |
| `planet-aspects-house` | 121 | SUN aspects the 1st house |
| `planet-at-table-degree` | 12 | SUN stands at the degree MRITYU_BHAGA gives it in its sign |
| `planet-in-table-sign` | 7 | SUN stands in a sign DAGDHA_RASHI gives the birth tithi |
| `lord-of-house-debilitated` | 5 | the lord of the 1st house is debilitated |
| `lord-of-house-combust` | 3 | the lord of the 5th house is combust |
| `lord-of-house-strong` | 8 | the lord of the 4th house is exalted, in its own sign or in its mooltrikona |
| `lord-of-house-is` | 15 | the lord of the 5th house is SUN |
| `lord-of-house-conjunct-planet` | 2 | the lord of the 5th house shares a sign with KETU |
| `lagna-in-sign` | 77 | the lagna rises in LEO |
| `planet-in-house-and-sign` | 26 | SUN stands in the 1st house and in LEO |
| `planet-at-gandanta` | 4 | SUN stands at a gandanta junction, within the usual 3°20′ |
| `planet-in-degrees` | 441 | MOON stands between 0° and 5° of its sign |
| `panchanga-tithi` | 33 | the birth tithi is AMAVASYA |
| `panchanga-paksha` | 24 | the birth falls in the SHUKLA paksha |
| `panchanga-vara` | 10 | the birth weekday is GURUVARA |
| `panchanga-nakshatra` | 42 | the Moon's birth nakshatra is MULA |
| `panchanga-yoga` | 3 | the panchanga yoga at birth is VAIDHRITI |
| `panchanga-karana` | 2 | the karana at birth is VISHTI |
| `birth-during-eclipse` | 6 | the birth falls in an eclipse |
| `birth-on-sankranti` | 8 | the birth falls on a sankranti's day |
| `transit-in-house` | 0 | in no pack |
| `planet-in-nakshatra` | 30 | MOON stands in the nakshatra PUSHYA |
| `same-nakshatra` | 1 | MOON and KETU stand in one nakshatra |
| `planet-strong` | 101 | SUN is strong |
| `planet-weak` | 35 | SUN is weak |
| `planet-stronger-than` | 23 | MOON is stronger than SUN |
| `rashi-aspects` | 2 | the AK aspects the PK by rashi drishti |
| `argala` | 16 | an intervention from the 5th, unobstructed from the 9th, stands on the pada of house 1 |
| `vipareeta-argala` | 0 | in no pack |
| `for-any` | 209 | some one of the nine grahas meets: the body found stands in the 1st house |
| `count-of` | 36 | at least 4 of the nine grahas meet: the body found aspects MOON |
| `n-of` | 12 | exactly 1 of these hold: the rule PHALADEEPIKA_RUCHAKA holds, the rule PHALADEEPIKA_BHADRA holds, the rule PHALADEEPIKA_HAMSA holds, the rule PHALADEEPIKA_MALAVYA holds and the rule PHALADEEPIKA_SASA holds |
| `in-varga` | 76 | in the D2, the lagna rises in CANCER |
| `count-in-houses` | 484 | at least 1 malefic stands in the 7th from SUN |
| `count-aspecting` | 28 | at least 1 malefic aspects SUN |
| `rule` | 1567 | the rule NABHASA_GADA holds |
| `at-limb-edge` | 22 | the birth falls within the last 2 ghatikas of the tithi |
| `limb-part` | 33 | the birth falls in the 1st of 6 equal parts of the tithi |
| `birth-by-day` | 12 | the birth fell by day |
| `same-sign` | 203 | SUN and MOON stand in one sign |
| `same-body` | 220 | MARS and MOON are one body |
| `planet-is` | 94 | SUN is a malefic |
| `natural-relation` | 44 | SUN counts the body found a friend |

2 kinds of the language occur in no pack at all: `transit-in-house`, `vipareeta-argala`. The corpus cannot falsify their prose, so the golden test in `crates/rules/src/prose.rs` is their only reader.

## Where the prose is at its weakest

The longest passage is `NABHASA_GOLA` at 32 lines; the shortest is
`NEECHA_BHANGA_RAJA` at 2 lines; the deepest nest is in
`NEECHA_BHANGA_RAJA`, 7 conditions deep. A reader arguing about the
wording starts at the first and the last of those.

## Two passages, whole

`cargo xtask rule-doc <pack|category|key>` prints the rest; the full
text of every shipped rule is derived words and is not checked in.

```text
MAHAPURUSHA_RUCHAKA — mahapurusha. Saravali ch. 37 vv. 5-7 (rank 1).
Note: Mars in his own sign or his exaltation, standing in an angle from the ascendant. Read from R. Santhanam's translation of Kalyana Varma's Saravali; the Parashara chapter gives the same figure at ch. 75 vv. 1 to 2.
When:
  MARS stands in a kendra, and
  the dignity of MARS is exalted, own sign or mooltrikona
Then: long of face, of pure splendour, greatly strong, valorous, of attractive brows and very black hair, warlike, knowing mantras, a leader of thieves, blood-red of complexion, a conqueror of his enemies, conch-necked, a chief, cruel, honouring the gods and Brahmins, thin of shank, a hundred inches tall, and a ruler of the Vindhya and the Sahya and a life of 70 years.
```

```text
BADHAKA_DOSHA — house-based. BPHS ch. 50 vv. 20-21 (rank 1).
Note: The badhaka sthana is the eleventh from a movable sign; the ninth from a fixed one and the seventh from a dual one are later tradition, and the verses speak of a rashi's badhaka in the Chara dasha rather than the lagna's (crux C86). A malefic standing there, or the badhakesh in a dusthana, afflicts. BPHS ch. 50 vv. 20 to 21 give the badhaka sthana inside the Chara dasha, so a natal reading of it is an extension (crux C86).
Found from any of:
  LAGNA, "Saturn in the badhaka sthana":
    SATURN stands in the 1st from the badhaka sthana of house 1
  LAGNA, "Mars in the badhaka sthana":
    MARS stands in the 1st from the badhaka sthana of house 1
  LAGNA, "Rahu in the badhaka sthana":
    RAHU stands in the 1st from the badhaka sthana of house 1
  LAGNA, "Ketu in the badhaka sthana":
    KETU stands in the 1st from the badhaka sthana of house 1
  LAGNA, "Sun in the badhaka sthana":
    SUN stands in the 1st from the badhaka sthana of house 1
  LAGNA, "The badhakesh in a dusthana":
    the lord of the badhaka sthana of house 1 stands in the 6th, 8th or 12th house
Cancelled by:
  "Badhakesh in own/exalted/mooltrikona sign": the dignity of the lord of the badhaka sthana of house 1 is own sign, exalted or mooltrikona
  "Badhakesh in upachaya (3/6/11)": the lord of the badhaka sthana of house 1 stands in the 3rd, 6th or 11th house
  1 of them cancels it fully; fewer cancel it in part.
Severity: 40 for each place it is found from, at most 100.
Remedies: BADHAKA_REMEDY_BADHAKESH_PROPITIATION and BADHAKA_REMEDY_GANESHA_PUJA.
```
