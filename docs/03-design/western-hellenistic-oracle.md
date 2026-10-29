# The oracle for the Western and Hellenistic fixtures

Status: `decided` and `built`, 2026-10-29; scored, the corpus not yet tagged. It answers crux **C46**, the
roadmap's Phase 7 exit ("the `western` and `hellenistic` modules
reproduce their own recorded fixtures, whose oracle is not Teimeris and
has to be sourced first") and its list's "an oracle for the Western and
Hellenistic fixtures".

The survey of 2026-09-11 ([`20-western-hellenistic-oracle.md`](../01-research/feature-universe/20-western-hellenistic-oracle.md))
found the rank 1 texts and refused every implementation as a
*dependency*. This page decides what the oracle *is*, records it, and
scores the SDK against it.

## The decision

**Two tiers, each doing what the other cannot.**

1. **Rank 1: the texts' own worked examples are the acceptance oracle.**
   They already hold every technique in both modules, each test asserting
   the author's printed answer
   ([take the acceptance value from the worked example](#what-each-technique-is-held-to)).
   They are the only evidence that is not another implementation, and the
   only evidence at all for releasing, the firdaria, the decennials, the
   considerations and perfection. They are few: one to six figures a
   technique.
2. **Rank 3: two independent programs, run over the corpus's 55
   recorded births, are the breadth oracle.** Their outputs are recorded
   in the conformance corpus beside the places they computed from, and
   the SDK's kernels are fed those places, so the comparison measures the
   doctrine and never the ephemeris, which is what C46 asks for. A
   disagreement is a question for a crux, never a correction
   (`CLEAN_ROOM.md`).
   - **flatlib 0.2.3** (MIT) for the traditional layer: the essential
     dignities under three term tables, the faces, the triplicity lords,
     the scores, the sect and 21 Arabic parts.
   - **immanuel 1.6.0** (AGPL-3.0, run only as a black box) for the
     modern layer: the aspects under its own orbs, the composite chart,
     the secondary progression's date and the solar return.

Nothing else reachable is better, and some of what was weighed is
worse. The weighing follows.

## The candidates

| candidate | licence | independence | covers | rank | verdict |
|---|---|---|---|---|---|
| the texts' worked examples: Valens IV.8 and IV.10, Lilly's figures (pp. 177, 385, 437, 439, 742), al-Biruni's §§438–439 table, Morin's Richelieu return, Leo's Appendix V and p. 150, Addey's Churchill harmonic, Ptolemy's term tables | facts; the books are public domain or freely published | complete: no implementation | every technique, one to six cases each | 1 | **the acceptance oracle**, already in the tests |
| flatlib 0.2.3 | MIT | its own code over the Swiss Ephemeris; a different author and school from the SDK | dignities, terms (Egyptian, *Tetrabiblos*, Lilly), faces, Dorothean triplicities, sect, 21 parts, profections, solar returns, primary directions, almutens, accidental dignities | 3 | **recorded**: the traditional breadth oracle |
| immanuel 1.6.0 | AGPL-3.0 | as above, modelled by its author on astro.com and Astro Gold | aspects with orbs, secondary progressions, solar returns, composites, parts, dignity scores, declinations | 3 | **recorded** as a black box: the modern breadth oracle |
| kerykeion 5.12.10 | AGPL-3.0 | as above | aspects, composites, returns, lunar phase | 3 | not recorded: nothing immanuel lacks; a third witness when two disagree |
| Astrolog | GPL | the broadest program (midpoints, harmonics, parts, progressions, directions, composites, Davison) | as listed | 3 | not recorded; a copyleft program, so only ever a black box. See [The composite rules' grounds](#the-composite-rules-grounds) |
| Brennan, *Hellenistic Astrology* (2017), worked releasing examples | in copyright | complete | releasing, profections, lots | 2 (a modern author's reading) | not used: no copy here, and C423 already marks its use as from memory |
| published ephemerides and tables of houses (Raphael, the *American Ephemeris*) | facts | complete | positions and cusps only | 1 | not needed: JPL (`fixtures/jpl`) and Teimeris already hold the astronomy, and these hold no doctrine |
| astro.com's chart service | terms of use bar automated requests | complete | most of the modern layer | 3 | not used |

**Licence.** A recorded output is a fact, the number a program printed
for an input, and carries no copyright of the program that printed it
(`CLEAN_ROOM.md` rule 3: "the fixture records the number, the tool and
its version, never the code"). The corpus is CC0 and stays so. flatlib's
MIT code may be read; immanuel's AGPL code is not, and its interface was
learnt from its README, `inspect.signature`, its configuration's values
and experiment, exactly as PyJHora's was. The exporters are kept outside
both repositories, as PyJHora's is, so no script importing an AGPL
package is distributed with the CC0 corpus.

**Independence.** Both programs read the Swiss Ephemeris, which the SDK
does not, and both are written by people outside this project from other
sources than the SDK's. Their places differ from the SDK's built-in
ephemeris by fractions of an arcsecond, which is why every comparison but
the solar return feeds the SDK the program's own places.

## What was recorded

Both programs were run once over every chart of `baseline/charts/` from
the fixture's UT instant and place, through the queue, on 2026-10-29.

- **`flatlib/traditional/`**: 53 files, the Tromsø pair refused (the
  tool's houses raise a math domain error at 69°N). Each holds the tool's places of
  the seven planets with their speeds, the ascendant, the midheaven and
  its default (Alcabitius) cusps; its sect; for each planet its sign's
  ruler, exaltation, three triplicity lords, face, detriment and fall,
  and under each of its three term tables the term's lord, its score and
  its peregrine flag; and its 21 parts. The manifest repeats each part's
  formula as the tool states it. The true altitudes of the tool's Sun and
  Moon are the exporter's, through the same `swe_azalt`, and are marked
  so.
- **`immanuel/modern/`**: 52 files, Reykjavik in June 1975 and the
  Tromsø pair refused (its Placidus falls back to Porphyry and raises).
  Each holds the tool's places of the
  ten planets with speeds and declinations, the angles, its sect and its
  parts of Fortune, Spirit and Eros; every aspect it lists between the
  ten planets with its orb, distance and motion; its secondary progression
  at 10, 30 and 60 Julian years; its solar returns one and thirty years
  after the year of birth; and its composite with the next chart in name
  order, with that chart's natal places. The manifest records the
  configuration the run used.

`validate.py` checks each file against its schema and hash, its inputs
against its fixture, and every chart answered or refused.

## The scores

Both scorers run with the corpus that carries the sections; on the
pinned corpus (v0.12.0) each prints why it skipped. Every comparison
either agrees or is counted against a crux; none is left over.

**flatlib, `crates/hellenistic/tests/flatlib.rs`: 7,918 compared.**

- 7,262 agree: the 180 term cells of its three tables against the SDK's
  printings, and for every planet of 53 charts its ruler, exaltation,
  detriment, fall, face, day and night triplicity lords, its term's lord
  and score under each table, and its peregrine flag; the tool's sect
  against the Sun's altitude on every chart; and all 21 parts, each
  evaluated by the SDK's `lot_place` from the tool's own formula and
  places, to 1e-9°.
- 80 against **C208**: the tool's *Tetrabiblos* terms part from the
  SDK's in Leo, Libra, Scorpio, Capricorn and Pisces, and its Lilly
  terms in Pisces (Mars to 25° against the SDK's 26°); its Egyptian
  table is the SDK's to the degree.
- 514 against **C790**: the tool's participating triplicity lord, and its
  score's triplicity counted for any of the three lords by day or night.
- 62 against **C791**: Valens's lots by name where the tool's night
  formula is not his (the father's and the brothers').

Its score was read back over 7,560 planet-table cells before it was
compared: Lilly's weights, a triplicity for any of the three Dorothean
lords and no peregrine debility reproduce every one.

**immanuel, `crates/sdk/tests/immanuel.rs`: 4,340 compared.**

- 4,152 agree: every aspect each way under the tool's orbs (presence,
  distance from exact to 1e-9°, applying), each composite planet's place
  and speed and the composite midheaven, the progressed instant at three
  ages for the epochs where the two years agree, and every solar return
  the built-in ephemeris reaches, within 60 s.
- 154 against **C236**: the progressed instant, where the tool's year is
  a tropical year of the birth's epoch (365.242 203 days in 1800 to
  365.242 166 in 2399) and the SDK's the constant 365.242 189, at most a
  third of a second at sixty years.
- 31 against **C792**: an aspect whose faster planet is retrograde, which
  the tool calls applying however the separation moves.
- 3 against **C247**: the composite lagna, which the tool keeps at the
  near midpoint and the SDK turns to stand east of the midheaven.

Two figures are bounds rather than agreements. c048's composite Moon is
2.15e-8° off the midpoint of the two Moons the tool recorded, because the
tool computes the native's chart afresh for the composite, so composites
are compared to 1e-7°. Solar returns agree within 5 s but for twelve,
each thirty years after a birth and ahead of the present (5.4 to 12.5 s
in 2049 to 2054, 33.9 s in 2380), as two ephemerides' extrapolated ΔT
would part; that cause is inferred. Two returns past 2399 are beyond the
built-in ephemeris and not compared.

**One defect found and fixed.** The SDK's orb engine read `applying` from
the separation a day later, so a pair crossing exact within a day read
the wrong way: the Moon 2° short of a square to Venus in c001, closing at
10.8° a day, read as separating. It now reads the separation's rate at the
instant. Over immanuel's 958 aspects the flag moved on 86, 69 with the
Moon and 17 between planets within 0.6° of exact; each the tool does not
call exact now agrees with it or is C792's.

## What each technique is held to

| technique | rank 1 (acceptance) | rank 3 (breadth) |
|---|---|---|
| terms, faces, rulers, exaltations, detriments, falls | Ptolemy's and Lilly's tables (`terms-measured.md`) | flatlib, every planet of 55 charts under three tables |
| triplicities | Ptolemy I.XXI, Lilly p. 104 | flatlib's day and night lords; its participating lord is C790 |
| essential scores | Lilly p. 115 and his two tallies | flatlib's scores under its reading |
| sect | Valens I, 51K (`sect-measured.md`) | flatlib's sect against the Sun's altitude |
| lots | Valens II.22 and III.11, Lilly pp. 143–144 | every flatlib part through the SDK's evaluator; Valens's five by name, C791 |
| aspects and orbs | Leo pp. 43–47, Edward VII; Lilly's moieties | immanuel's aspects under its orbs, both ways |
| composite | Townley's example (C247, C248) | immanuel's composites; its lagna is C247's rival |
| progressions | Leo's Appendix V contact | immanuel's progressed instant |
| returns | Morin's Richelieu figure recast | immanuel's solar returns on the SDK's own ephemeris |
| releasing, profections | Valens IV.8, IV.10, IV.11 | **none exists**: no program reachable computes releasing; flatlib's profection is recorded only as a capability |
| firdaria | al-Biruni §§438–439 | **none exists** |
| decennials | Valens VI.5–7 | **none exists** |
| considerations, perfection, fortitudes, almutens | Lilly's figures recast | **none recorded**: flatlib's accidental dignities and almutens are the next to record; the horary clauses have no program |
| midpoints, harmonics, antiscia, declinations, configurations, synastry, houses | Leo, Addey, Lilly, Edward VII | not recorded: arithmetic each test already holds, or astronomy the JPL and Teimeris tiers hold |

## The composite rules' grounds

`western-composites.md` once rested part of C247 and C248 on reading a
copyleft program's code, which `CLEAN_ROOM.md` forbids ("code, constant
tables, comments, algorithms read from the code"). On 2026-10-29 that
reading was withdrawn, unopened again, and every rule re-grounded:
the near midpoint, the midpoint cusps and the midpoint ascendant on
Townley's own article and interview, the Davison mean on his worked
example and Astrodienst's published documentation, and each convention
no source read settles (the turned lagna and cusps, the exactly
opposite pair, the mean speed, the longitude across the antimeridian)
marked the SDK's own decision with its reasoning. No behaviour changed.
Any later witness from a copyleft program is a black box like immanuel:
its numbers, never its code.

## What waits

- **The maintainer's yes** to push the corpus branch and tag it, then to
  move the SDK's pin; until then both scorers skip, saying which corpus
  they found.
- flatlib's accidental dignities and almutens, against
  `fortitudes-measured.md` and the almuten three ways.
- A second Hellenistic witness for releasing, which no program reachable
  computes: the rank 1 examples remain its only oracle.
