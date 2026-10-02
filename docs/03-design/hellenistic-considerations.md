# The considerations before judgement (the `hellenistic` module, step 10)

Status: `built`, 2026-10-02 — written from Lilly's text before any code
and corrected by building it; the boundary and the bindings are the next
step.

Before a horary figure is judged, Lilly's astrologer asks whether it is
"radicall and capable of judgment", and names the conditions that make a
judgement unsafe. They are not a verdict but a list of clauses, each
true or false of the figure, with the reason it matters. The SDK reports
the clauses and leaves the deciding to the reader.

## What the source decides

**Lilly**, *Christian Astrology* (1647), Book I chapter XIX,
"Considerations before Judgment", printed pp. 121–123, is the rank 1
text, read off the Wellcome scan's page images (leaf = printed page +
33). Void of course is defined on p. 112, and each planet's nature in its
own chapter.

1. **Radical.** The figure is fit to be judged when the lord of the hour
   and the lord of the Ascendant "are of one Triplicity, or be one, or of
   the same nature". His three examples decide each reading:
   - Mars's hour with Scorpio or Pisces rising: Mars is a lord of the
     water triplicity, as his own table gives it.
   - Mars's hour with Aries rising: the two lords are one planet.
   - Mars's hour with Leo rising: the Sun and Mars are "both of one
     nature, viz. Hot and Dry".

   The natures:
   - Saturn and Mercury are cold and dry.
   - Jupiter is hot and moist.
   - Mars and the Sun are hot and dry.
   - Venus and the Moon are cold and moist.
2. **The Ascendant too early.** "00. degrees, or the first or second", the
   more so in the signs of short ascension (Capricorn to Gemini).
3. **The Ascendant too late.** "27, 28, or 29 degrees".
4. **The Moon late in a sign**, especially in Gemini, Scorpio or
   Capricorn. Lilly does not say how late (C229).
5. **The Moon in the via combusta**, "as some say": the last 15° of Libra
   or the first 15° of Scorpio.
6. **The Moon void of course.** She is "separated from a Planet, nor doth
   forthwith, during his being in that Signe, apply to any other". She
   "performes" somewhat when void in Taurus, Cancer, Sagittarius or
   Pisces. What counts as applying is C230.
7. **The seventh afflicted.** Its cusp is "afflicted", or its lord is
   retrograde or impedited, when the matter is not of the seventh house
   (C231 for "afflicted").
8. **Alkindi's rules**, as Lilly gives them:
   - Saturn in the Ascendant, especially retrograde.
   - Saturn in the seventh.
   - The lord of the Ascendant combust.
   - The lord of the seventh unfortunate, in his fall or in the terms of
     the infortunes.

## The acceptance tests

1. **p. 121–122, radical.** Mars's hour:
   - radical with Scorpio, Pisces, Aries and Leo rising, each for the
     reason Lilly gives;
   - not radical with Gemini, whose lord Mercury is cold and dry and not
     a lord of the air triplicity with Mars.
2. **p. 122, the degrees.** 0°, 1° and 2° rising are early, 27° to 29°
   late, and 3° and 26° neither; the short-ascension signs are flagged.
3. **p. 122, the via combusta.** It holds at Libra 15° and Scorpio 14°59′,
   and not at Libra 14°59′ or Scorpio 15°.
4. **p. 112, void of course, on a sky that can happen.**
   - A Moon late in a sign with no aspect left to perfect is void.
   - One with an aspect ahead is not.
   - The four signs that ease it are named.
   - The first perfection the projection finds is the one a step-by-step
     walk of 200 skies finds.
5. **Read back in the SDK.** Over 48 hourly London charts, the chart cast
   at the moment a perfection is promised finds the Moon at that aspect
   within 0.01° (0.007° measured), geocentrically.
6. **Lilly's two figures that name the Moon's course** (read in full
   under C230 below), recast, with both readings held to their answers:
   - the ship at sea (p. 165): void by neither reading;
   - "If Presbytery shall stand" (p. 439): void by the moieties, not by
     the sign's end.

## The design

- `teistro_hellenistic::considerations(&Fortitudes, hour_lord,
  ConsiderationRules)` returns `Considerations`. It has one field per
  clause, each holding the facts the clause rests on, and never a single
  verdict:
  - `Radicality`: the two lords and every `RadicalGround` that holds;
  - `AscendantClause`;
  - `MoonClause`, with its `MoonCourse`;
  - `SeventhClause`: the seventh's lord's state, with its net strength;
  - Saturn's house and motion, and the Ascendant's lord's combustion.
- **The lord of the hour** is the chart's own planetary hour, under the
  settings' `hora_reckoning` knob (unequal hours by default, as Lilly's
  were).
- **The rest** comes from the fortitudes:
  - the triplicities and terms are the dignities' rules (Lilly's by
    default);
  - combustion, retrogradation and the house a planet is counted in are
    the accidental fortitudes'.

  `ChartArea::considerations(chart, &FortitudeRequest,
  ConsiderationRules)` reads them, and needs no ephemeris.
- **Across the boundary** the chart request's nullable
  `considerations_json` is the rules, every member optional and refused
  by `considerations.<member>`. The answer is three sections: the
  clauses a row a chart, the Moon's two perfections two rows a chart
  (`present` 0 where she is void by that reading), and the orbs seven a
  chart. Bit sets carry the radical grounds and the seventh's
  infortunes, bit `n` the member with id `n`, so `RadicalGround` and
  `PtolemaicAspect` are boundary enums every binding names. When the
  request asks for no fortitudes the clauses are read under Lilly's;
  when it does, under the ones it asked for, so the two always agree.
  Each binding reads them into the Rust serde shape, and an answer's
  `rules` is a request as it stands.
- **The natures are `Temperament`.** The name is not `Nature` because the
  catalogue's `Nature` is a different key space.
- **Void of course is the one new computation.** `moon_course` projects
  each of the seven at its present speed. For each Ptolemaic aspect, on
  either side, it finds the first that perfects before the Moon leaves
  her sign. A `Perfection` names the planet and the aspect, the days to
  exact, and `gap_deg`, the arc still to close.

  `within_orb` is the first of those already within the two planets'
  moieties of orb. The moieties are half of each one's orb in
  `ConsiderationRules::orbs_deg`, by default Lilly's table on p. 107:
  Saturn 10°, Jupiter 12°, Mars 7½°, the Sun 17°, Venus 8°, Mercury 7°
  and the Moon 12½°. So `void()` and `void_by_moieties()` report the two
  readings side by side (C230).

## What building it found

- **A topocentric Moon defeats the projection.** Her apparent place
  swings with parallax by up to a degree a day. Over the same 48 London
  hours under the corpus's topocentric profile, the read-back missed by
  up to 2.3°, against 0.007° geocentrically.

  Lilly's ephemerides were geocentric, so the method's documentation
  says to read a horary figure geocentrically. The SDK test holds the
  geocentric chart.
- **Lilly's own use of "void" is not the modern one.** In the ship
  question:
  - he calls the Moon void at the question;
  - the recast figure has her 4.3° from a trine to Saturn that
    perfects in her sign 7 hours later;
  - he himself names that trine as her next application.

  Neither "no perfection before she leaves her sign" nor a moiety orb
  (Moon 12°, Saturn 10°, so 11°) calls her void there.

  His question "If Presbytery shall stand" (11 March 1646/7, 4h 45m PM,
  recast to 10′ of his printed Moon) reads otherwise. There the Moon,
  "after a little being voyd of course", runs to the squares of Mars
  and Jupiter. Mars's square is 11.9° ahead, beyond the Moon's and
  Mars's moieties together (10°). So the moieties call her void, as
  Lilly does, and the sign's end does not.

  One figure of two supports the moieties, and neither supports the
  sign's end. C230 stays open, and both readings are reported. The shipped reading is the modern one, and
  `gap_deg` is reported so that another reading can be applied.

## What is not decided

- **C229: how late "the later degrees" are.** `ConsiderationRules`'
  `moon_late_from_deg` holds it, at 27° by default, the degree Lilly gives
  for a late Ascendant.
- **C230: what void of course means.**
  - One reading is no perfection before the sign ends (`void()`).
  - The other is none yet within Lilly's moieties of orb
    (`void_by_moieties()`).
  - Both are reported. The Presbytery figure fits the moieties, and the
    ship fits neither.
- **C231: what afflicts a cusp.** The shipped reading is an infortune
  counted in the house (`infortunes_in_house`), not one in aspect to it.

## The order of work

1. ~~The clauses in `crates/hellenistic`, held to the tests.~~
2. ~~`ChartArea::considerations`.~~
3. ~~The boundary and every binding, as the fortitudes crossed.~~
4. Read Book II's other questions for the considerations they name, and
   recast each with a date to test C230 further.
