# Perfection: whether a matter is brought to pass (the `hellenistic` module, step 11)

Status: `built`, 2026-10-03 — written from Lilly's text before any code
and corrected by building it, and carried across the boundary into every
binding.

A horary question names two significators, the querent's (the lord of
the Ascendant) and the quesited's (the lord of the house of the thing
asked). Lilly lists the ways a matter between them is brought to pass,
and the ways it is stopped short. As with the
considerations ([`hellenistic-considerations.md`](hellenistic-considerations.md)),
the SDK reports each way and each impediment that holds, with the facts
it rests on, and never a verdict.

## What the source decides

**Lilly**, *Christian Astrology* (1647), is the rank 1 text, read off
the Wellcome scan's page images (leaf = printed page + 33):

- Book I ch. XXI, printed pp. 125–127, the ways of perfection;
- pp. 106–113, the relations they are built from: application and
  separation, prohibition, refranation, translation, frustration.

The relations, each a clause between named planets:

1. **Application** (p. 107) "is three severall wayes": a swifter direct
   planet to a slower direct one; both retrograde; and a direct planet
   in fewer degrees meeting a retrograde one in more. He calls the second
   and the third each "an ill Application" (verified on the page image).
2. **Separation** (p. 110): once past exact, the swifter is separating,
   and "totally separated" only when past the two moieties of orb.
3. **Prohibition** (pp. 110–111): before two significators perfect,
   "another Planet interposeth either his body or aspect". Bodily
   prohibition is a third planet conjoining one of them first; by aspect,
   a third perfecting an aspect with one of them first.
4. **Refranation** (p. 111): the applying planet turns retrograde before
   it perfects, and "refraines to come".
5. **Translation of light** (p. 111): a light planet "separates from a
   more weighty one, and presently joynes to another", by body or
   aspect.
6. **Frustration** (pp. 112–113): a swift planet applies to a slower,
   but the slower perfects with a third first.

The ways of perfection (pp. 125–127), between the two significators:

1. **Conjunction**, meeting "no prohibition or refrenation before they
   come to perfect", soonest from angles, slower from succedent houses,
   "with infinite losse of time" from cadent.
2. **Sextile or trine** out of good houses and well dignified, with no
   malevolent aspect intervening, perfected "to the partill".
3. **Square**, "provided each Planet have dignity in the Degrees wherein
   they are, and apply out of proper and good Houses, otherwise not".
4. **Opposition**, which he has "rarely" seen perfect a matter, and then
   with mutual reception by house and the Moon separating from the
   quesited's significator and applying to the querent's.
5. **Translation**: the significators separated, a third planet
   separating from one of them, by which it is received "either by House,
   Triplicity or Terme", and applying to the other "before he meeteth
   with" any other planet.
6. **Collection**: the significators "doe not behold one another, but
   both cast their severall Aspects to a more weighty Planet then
   themselves, and they both receive him in some of their essentiall
   dignities".
7. **Dwelling in houses**: the quesited's significator in the Ascendant,
   which "holds not true" unless the Moon translates the light as well.

## The acceptance tests

Lilly's own examples, each verified on the page image, become the first
tests, each on a hand-made timeline:

1. **p. 107, application.** Mars 10° Aries and Mercury 5°, both direct:
   Mercury applies. Mercury 10° and Mars 9°, both retrograde: an ill
   application. Mars direct at 15° and Mercury retrograde at 17°: the
   third kind, also ill.
2. **p. 111, bodily prohibition.** Mars 7° Aries, Saturn 12°, the Sun
   6°: the Sun conjoins Mars, then Saturn, before Mars reaches Saturn.
3. **p. 111, prohibition by aspect.** Mars 7° Aries, Saturn 15°, the Sun
   5° Gemini: the Sun passes Mars's dexter sextile and reaches Saturn's
   before Mars conjoins Saturn.
4. **p. 111, refranation.** Saturn 12° Aries, Mars 7°: Mars stations
   before the tenth or eleventh degree and never perfects.
5. **p. 111, translation.** Saturn 20° Aries, Mars 15°, Mercury 16°:
   Mercury separates from Mars and conjoins Saturn.
6. **p. 113, frustration.** Mercury 10° Aries, Mars 12°, Jupiter 13°:
   Mars conjoins Jupiter before Mercury reaches Mars.

## The design

### One search gives every relation

Every clause above is a statement about **the order in which aspects
perfect**, and about stations between them. So the module computes one
timeline and reads every clause off it:

- For each pair of the seven, the instants their longitudes stand at a
  Ptolemaic angle, searched forward from the figure. This is the port's
  crossing search over `Quantity::Composite { a: 1, first, b: -1,
  second }` against the five angles on both sides. The port documents
  the composite as "an aspect" for exactly this, so a provider's own
  crossings override serves it too.
- For each of the seven, its next station: `Quantity::Speed(body)`
  crossing zero, which `events::stations` already searches for.
- For each pair, the last perfection before the figure, and whether the
  two are still within their moieties.

That is the timeline. **Linear projection is not enough here**, unlike
`moon_course`: refranation is a station before perfection, which a
straight line never shows, and the slower planets' perfections lie days
or weeks away. The search uses the chart's own ephemeris and frame.

### What a report holds

`perfection(chart, querent: Graha, quesited: Graha, rules)` returns:

- `application`: the next perfection between the two, if any, with the
  aspect, its instant, which applies, and which of Lilly's three kinds it
  is (`BothDirect`, `BothRetrograde`, `AgainstRetrograde`), or none;
- `separation`: the last perfection between them, with whether they are
  still within the moieties;
- `impediments`: each prohibition (bodily or by aspect, naming the
  third planet and its perfection), each refranation (the station's
  instant and planet) and each frustration that comes before the
  application perfects;
- `ways`: each of the seven ways that holds, with the facts it was read
  from (houses angular, succedent or cadent; each significator's
  dignity at its degree; reception and by what; the translator or
  collector and its perfections).

A way's conditions that are judgements rather than facts — "good
houses", "well dignified" — are reported as facts (the house, the
dignity score from `Fortitudes`), as the considerations report theirs.

### The rules

`PerfectionRules`, every member optional:

- `orbsDeg`: the planets' orbs, by default the first of Lilly's two
  columns on p. 107, as `ConsiderationRules` has it. The two share one
  table type, so a consumer sets the orbs once.
- `horizonDays`: how far forward the search runs, by default until the
  swifter significator leaves its sign (C232).
- `withinSign`: whether a contact with a third planet counts only when
  the planet applying perfects it before leaving its sign; true by
  default (C234). It bounds the impediments, a translator's next contact
  and a collector's contacts, not the significators' own application,
  which the horizon bounds.

### Where it lives

The clauses are arithmetic over a timeline, so they live in
`crates/hellenistic` and are tested on hand-made timelines. The search
is the SDK's (`ChartArea::perfection`), because it needs the ephemeris.
`moon_course` stays as it is: the considerations ask a different
question, and need no ephemeris.

## What building it found

- **A separation needs no search behind the figure.** Lilly's
  separation lasts "untill" the moieties are cleared (p. 110), so it is
  read off the figure itself: the aspect the two are leaving, inside
  their moieties and moving apart. Only what lies ahead is searched.
- **Prohibition and frustration differ in who moves.** In Lilly's
  prohibition (p. 111) the Sun, the third, comes to Mars; in his
  frustration (p. 113) Mars, the significator, gets to Jupiter. So a
  significator's contact with a third before the application is a
  prohibition when the third's motion closes it, and a frustration when
  the significator's does; both are reported with the contact.
- **Refranation is the motions' promise broken.** A significator's
  station before the perfection its motion of the moment promises, and
  before any the timeline finds, is reported with or without an
  application, since the station usually removes it.
- **Every promised contact is where the later chart finds it.** Over
  twelve London figures two months apart and four pairs of
  significators, geocentric, the chart cast at each of 12 applications
  and 104 impediments' contacts stands at the aspect within 3.4 × 10⁻⁹°,
  and each refranation's station is where the planet's motion turns.
  The search scans each pair's separation once over a 30° lattice, so
  `aspect_at` keeps the five aspects and drops the semisextiles and
  quincunxes it also finds.
- **A way is held on what Lilly states as fact, and the judgements are
  left as facts.** `Ways` reports each significator's house and its own
  dignities at its degree, the mutual reception by house, the
  infortunes among the thirds that come between, the Moon's relay and
  the quesited's significator in the Ascendant; `held` names each way
  whose stated conditions hold. "Out of good houses" and "well
  dignified" are judgements, so they are not folded into `held`: the
  reader weighs them on the houses and dignities reported. A square
  needs "dignity in the Degrees wherein they are", so it is held only
  when neither significator is peregrine; a collection only when the
  collector stands in some dignity of each (C233); a translation only
  when its translator is received by house, triplicity or term. The
  houses come from the fortitudes (`Standing::of`), as the accidental
  table counts them.

- **Across the boundary** the chart request's nullable
  `perfection_json` is a `PerfectionRequest`: `querent` and `quesited`
  by key, or `house`, and `rules`, refused by `perfection.<member>`. The
  answer is five sections: a row a chart (the significators, the
  application and separation, `application_present` and
  `separation_present` 0 where there is none, each
  significator's house and dignities, the ways' facts and the ways held,
  and three counts); the impediments, translations and collections
  ragged by those counts; and the orbs seven a chart. Dignities cross as
  a bit set in `EssentialDignity`'s order, the infortunes between and the
  ways held as bit sets over the graha id and `TsWay`, and
  `TsApplicationKind`, `TsImpedimentKind` and `TsWay` are boundary enums
  every binding names. The houses and dignities are the fortitudes'
  the request asked for, or Lilly's. The parity runners ask for the
  seventh house over 120 days, so their charts apply and are hindered.
- **Lilly's p. 238 figure reads as he judged it.** "If the Querent
  should ever have Children?" is dated "Die ♃ 11 June 1635" counted from
  noon, so the morning of 12 June (Julian); recast at the printed
  Ascendant (Saturn, Mars and the Sun within 5′ of the figure), Mercury,
  lord of the Ascendant, applies to the opposition of Saturn, the
  fifth's lord, retrograde, before leaving Gemini: p. 107's third kind.
  Nothing comes between, nothing is heavier than Saturn to collect, and
  no way holds, where Lilly found "no one promising testimony". The
  built-in ephemeris starts in 1800, so the figure is read on its
  recast places in `crates/hellenistic`.
- **A void Moon prohibits nothing.** Lilly's p. 385 figure, "A Lady, if
  marry the Gentleman desired?", gives as its "first" reason that the
  Sun and Saturn, the lords of the first and seventh, apply to a sextile.
  Counted as first built, the Moon prohibited that sextile six times.
  She is void, a quarter of a day from leaving Sagittarius, and every one
  of those contacts comes after. Lilly instead reads her opposition to
  the Sun as "another small argument" for the match, and Jupiter as
  "meeting with no manner of prohibition" (p. 387). Lilly's void of
  course bounds a planet's application by "his being in that Signe"
  (p. 112), so a third planet's contact now counts only while the
  applier is in its sign (`withinSign`, C234). The figure reads as he
  judged it: the sextile in 10.6 days, nothing between, and the
  sextile-or-trine way held.
- **Lilly's p. 437 figure perfects nothing.** "If he should obtain the
  Parsonage desired" is dated 6 August 1644, 8h 24m p.m. Recast at the
  printed Ascendant, it lands two minutes from the printed time in local
  apparent time, with every cusp within 8′. Mars and Jupiter have no
  application. The Moon, leaving Mars's trine, meets Mercury's
  opposition before Jupiter's square, so she translates nothing. Saturn
  reaches neither significator. Lilly finds "no weighty Planet that
  translates or collects", and the SDK finds no relation at all.

## What is not decided

- **C232: does an application run past the sign?** Lilly defines void
  of course as no application "during his being in that Signe" (p. 112),
  but nowhere bounds an application between two significators by the
  sign. The default stops at the swifter significator's sign; the
  horizon is a knob.
- **C233: who receives whom in collection.** Lilly's words are that the
  significators "both receive him" — the collector stands in *their*
  dignities. Later accounts often put the significators in the
  collector's dignities instead, and so does Lilly on p. 239, wishing
  for a collector that "had received ♄ or ☿". The report states the
  reception both ways, and the way follows p. 126.
- **C235: a middle planet carrying light.** Lilly's "maine occasion" on
  p. 387 is Jupiter: it applies "to ✶ of ♄", receives Saturn's virtue,
  and "transferred" it to the Sun, whose conjunction it reaches on "the
  29th of June" (the recast gives the same day). Jupiter is lighter than
  Saturn and heavier than the Sun, so it is neither p. 111's translator,
  lighter than both, nor p. 126's collector, heavier than both. On the
  recast it is also 8′ past the sextile, where the printed places leave
  it 13′ short. One figure does not make a rule, so nothing reports a
  relay; each of the two contacts stands in the timeline.

## Order of work

1. ~~Verify the quotations and the worked examples on the page
   images.~~
2. ~~The timeline search in the SDK (`Founder::contact_events`), read
   back in the chart cast at each promised contact.~~
3. ~~The relations in `crates/hellenistic`, on hand-made timelines.~~
4. ~~`ChartArea::perfection` and the seven ways; then the boundary and
   every binding.~~
5. Recast the Book II figures where Lilly names a translation,
   prohibition or collection, and test each against his judgement: p. 238
   done (an application by opposition, no way held), p. 385 (a sextile,
   the void Moon prohibiting nothing, C234) and p. 437 (nothing
   perfects).
