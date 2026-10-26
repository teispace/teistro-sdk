# A rule in prose, measured

Status: `generated` by `cargo xtask rule-doc` over the shipped rule
packs and the conformance corpus's own rules, 2026-09-20. Do not edit:
`check-rule-doc` regenerates this page and fails on any difference. The
design it measures is [`rule-doc.md`](rule-doc.md).

## What was rendered

5476 rules: 17 in `DOSHAS`, 8 in `YOGAS`, 4 in `GANDANTAS`, 184 in `ARISHTAS`, 1688 in `READINGS`, 2918 in `NABHASAS`, 657 in `the corpus's own`. Their 26 869 conditions are written 9243 ways, which say 9218 things, and the renderer gives those 9238 sentences.

The 4819 shipped rules are what a consumer evaluates; the corpus's own
are rendered beside them because they exercise predicates no shipped
pack uses.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| no two conditions that differ in meaning read alike | falsified | 1 of 9218 disagree |
| every condition of every rule renders | **holds** | 0 of 26869 disagree |

These renderings are shared, each by the conditions beneath it:

- `LAGNA stands between 26.6667° and 30° of its sign`
  - `{"type":"planet-in-degrees","planet":"LAGNA","from":26.666666666666668,"to":30.0}`
  - `{"type":"planet-in-degrees","planet":"LAGNA","from":26.6667,"to":30.0}`

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
| `and` | 2743 | MOON aspects SUN and SUN aspects MOON |
| `or` | 1771 | MARS is combust or MARS is retrograde |
| `not` | 1754 | it is not the case that SATURN is weak |
| `planet-in-house` | 2714 | SUN stands in the 1st house |
| `planet-in-sign` | 2333 | SUN stands in LEO |
| `planet-dignity` | 794 | the dignity of SUN is exalted |
| `planet-in-kendra` | 194 | SUN stands in a kendra |
| `planet-in-trikona` | 15 | SUN stands in a trikona |
| `planet-in-kendra-from` | 63 | SUN stands in a kendra from MOON |
| `lord-of-house-in-kendra` | 78 | the lord of the 1st house stands in a kendra |
| `lord-of-house-in-house` | 667 | the lord of the 1st house stands in the 1st house |
| `planet-conjunct` | 517 | MARS and SUN share a sign |
| `planet-in-house-from` | 757 | MARS stands in the 1st from SUN |
| `no-planet-in-houses-from` | 63 | no graha stands in the 7th from LAGNA |
| `mutual-exchange` | 170 | the lords of the 1st and 2nd houses stand each in the other's sign |
| `lord-conjunct-lord` | 93 | the lord of the 1st house shares a sign with the lord of the 2nd |
| `all-planets-between-nodes` | 14 | the seven classical grahas all stand between the nodes |
| `occupied-sign-count` | 28 | SUN, VENUS and MARS occupy exactly 3 signs |
| `all-classical-grahas-in-houses` | 102 | the seven classical grahas stand only in the 1st or 4th house, filling every one of them |
| `n-grahas-conjunct-with` | 64 | at least 4 classical grahas share the sign of SUN, it among them |
| `chara-karaka-in-house` | 34 | the AK stands in the 1st house |
| `planet-combust` | 145 | SUN is combust |
| `planet-retrograde` | 24 | MARS is retrograde |
| `planet-aspects-planet` | 1747 | KETU aspects SUN |
| `planet-aspects-house` | 224 | SUN aspects the 1st house |
| `planet-at-table-degree` | 12 | SUN stands at the degree MRITYU_BHAGA gives it in its sign |
| `planet-in-table-sign` | 7 | SUN stands in a sign DAGDHA_RASHI gives the birth tithi |
| `lord-of-house-debilitated` | 5 | the lord of the 1st house is debilitated |
| `lord-of-house-combust` | 3 | the lord of the 5th house is combust |
| `lord-of-house-strong` | 8 | the lord of the 4th house is exalted, in its own sign or in its mooltrikona |
| `lord-of-house-is` | 24 | the lord of the 5th house is SUN |
| `lord-of-house-conjunct-planet` | 2 | the lord of the 5th house shares a sign with KETU |
| `lagna-in-sign` | 643 | the lagna rises in LEO |
| `planet-in-house-and-sign` | 26 | SUN stands in the 1st house and in LEO |
| `planet-at-gandanta` | 4 | SUN stands at a gandanta junction, within the usual 3°20′ |
| `planet-in-degrees` | 1036 | SUN stands between 0° and 5° of its sign |
| `panchanga-tithi` | 63 | the birth tithi is AMAVASYA |
| `panchanga-paksha` | 60 | the birth falls in the SHUKLA paksha |
| `panchanga-vara` | 20 | the birth weekday is GURUVARA |
| `panchanga-nakshatra` | 79 | the Moon's birth nakshatra is MULA |
| `panchanga-yoga` | 9 | the panchanga yoga at birth is GANDA |
| `panchanga-karana` | 3 | the karana at birth is VISHTI |
| `birth-during-eclipse` | 6 | the birth falls in an eclipse |
| `birth-on-sankranti` | 8 | the birth falls on a sankranti's day |
| `transit-in-house` | 0 | in no pack |
| `planet-in-nakshatra` | 81 | MOON stands in the nakshatra MULA |
| `same-nakshatra` | 2 | MOON and KETU stand in one nakshatra |
| `planet-strong` | 443 | SUN is strong |
| `planet-weak` | 82 | SUN is weak |
| `planet-stronger-than` | 161 | MOON is stronger than SUN |
| `rashi-aspects` | 50 | MERCURY aspects SUN by rashi drishti |
| `argala` | 16 | an intervention from the 5th, unobstructed from the 9th, stands on the pada of house 1 |
| `vipareeta-argala` | 0 | in no pack |
| `for-any` | 673 | some one of SUN, MARS or SATURN meets: the body found aspects KETU |
| `count-of` | 206 | at least 4 of the nine grahas meet: the body found aspects MOON |
| `n-of` | 14 | exactly 1 of these hold: the rule PHALADEEPIKA_RUCHAKA holds, the rule PHALADEEPIKA_BHADRA holds, the rule PHALADEEPIKA_HAMSA holds, the rule PHALADEEPIKA_MALAVYA holds and the rule PHALADEEPIKA_SASA holds |
| `in-varga` | 344 | in the D9, SUN stands in LEO |
| `count-in-houses` | 827 | at least 1 malefic stands in the 7th from SUN |
| `count-aspecting` | 30 | at least 1 malefic aspects SUN |
| `rule` | 2468 | the rule NABHASA_GADA holds |
| `at-limb-edge` | 26 | the birth falls within the last 2 ghatikas of the tithi |
| `limb-part` | 64 | the birth falls in the 1st of 6 equal parts of the tithi |
| `birth-by-day` | 30 | the birth fell by day |
| `birth-in-kshaya-tithi` | 1 | the birth's tithi touched no sunrise, a kshaya tithi |
| `same-sign` | 958 | MOON and SUN stand in one sign |
| `same-body` | 809 | RAHU and SUN are one body |
| `planet-is` | 373 | SUN is a malefic |
| `natural-relation` | 85 | SUN counts the body found a friend |

2 kinds of the language occur in no pack at all: `transit-in-house`, `vipareeta-argala`. The corpus cannot falsify their prose, so the golden test in `crates/rules/src/prose.rs` is their only reader.

## Where the prose is at its weakest

The longest passage is `NABHASA_GOLA` at 32 lines; the shortest is
`NEECHA_BHANGA_RAJA` at 2 lines; the deepest nest is in
`PARIJATA_LOKA_AFTER_DEATH_TO_THE_GODS`, 10 conditions deep. A reader
arguing about the wording starts at the first and the last of those.

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
