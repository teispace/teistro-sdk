# The decennials (the `hellenistic` module, step 9)

Status: `built`, 2026-10-02 — written from Valens's text before the
kernel and corrected by building it; measured in
`time-lords-measured.md`.

The decennials are Valens's distribution of 10 years and 9 months: each
of the seven takes 129 months in turn, from the luminary of the sect,
and shares them out among the seven in proportion to their minimum years.
They are the fourth Hellenistic-tradition kernel behind the dasha
crate's `Timeline`, beside releasing, the profected year
(`hellenistic-time-lords.md`) and the firdaria
(`hellenistic-firdaria.md`), so a chart request asks for them as it asks
for Vimshottari.

## What the source decides

**Valens**, *Anthologies* VI (Riley's translation), chapters 5–7
(Kroll 252K–257K), is the rank 1 text read here.

- **The apheta.** The Sun for a day birth and the Moon for a night one,
  "if it is well situated"; when both are badly situated, the star
  following the Ascendant (252K). Book IV's marginal note on chapter 28
  gives the other luminary first.
- **The order.** After the apheta, "the next star in the zodiacal circle
  at the nativity": the seven by their places "by sign and by degree",
  not the Chaldean order from Saturn, which Valens rejects because it
  gives most nativities the same lords (257K).
- **The first level.** Each star rules 10 years 9 months, 129 months.
- **The second level.** A star's 129 months are shared from itself, in
  the same order, each star taking its minimum years as months: Saturn 30,
  Jupiter 12, Mars 15, the Sun 19, Venus 8, Mercury 20, the Moon 25,
  which sum to 129 (253K).
- **The year.** "The days and the cycles are calculated using 360-day
  periods, but the years of a nativity are calculated using 365 1/4"
  (253K): the periods are counted in 360-day years of real days, as
  releasing's are.

## The acceptance tests

1. **253K, the first level.** Valens's night nativity (the Moon in Pisces
   18°, Venus in Aries, Jupiter in Libra, Saturn in Sagittarius, Mars
   early in Aquarius, the Sun in Aquarius, Mercury in Pisces): the Moon,
   Venus, Jupiter and Saturn take four cycles, 43 years, and Mars's
   begins. The text names three of the four; the count of four and the
   zodiacal order supply Jupiter.
2. **253K, the second level.** Mars's cycle gives Mars 15 months (44
   years 3 months), then the Sun 19, Mercury 20, the Moon 25, Venus 8 and
   Jupiter 12 (51 years 3 months), and Saturn 30 (53 years 9 months).
3. **254K, the date.** 52 years and 124 days from birth is 667 days into
   Saturn's 30 months, Saturn receiving from Mars.
4. **254K–255K, the third level (6K).** The seven subdivision tables:
   each star's months shared again in proportion, a to b being `ya·yb/129`
   months of 30 days. 44 of the 49 printed cells agree within a quarter
   of an hour. Five differ, and the table's own symmetry, since a to b
   must equal b to a, shows each to be a printing slip rather than another
   rule: Saturn to itself (6¼ hours for 7¼), Jupiter to the Sun and the
   Sun to Jupiter (20½ and 8½ hours for half an hour, and the pair
   disagree), Venus to Mercury (4¼ hours where Mercury to Venus prints
   5) and Mercury to itself (9½ hours for half an hour). The test holds
   the 44 and names the five.
5. **254K, the third level counted in cycles.** Under the `CYCLES`
   division the same day is five 129-day cycles (Saturn, Mars, the Sun,
   Mercury, the Moon) and 22 days into Venus's, which gives Venus 8
   days, Jupiter 12 and Saturn the last 2; the seventh cycle of
   Saturn's 900 days is cut at 126. In proportion the day falls in the
   Moon's share, which is C228.

## The design

- **`DecennialDasha`** holds the seven in their order from the apheta,
  the birth, a year length and the division below the second level. A
  mahadasha is one 129-month period; each level below divides its parent
  in proportion to the minimum years (`minimum_years`, shared with
  releasing), from the parent's own lord in the same order. Under
  `CYCLES` the third level is 129-day cycles and the fourth the minimum
  years as days, each cut at its parent's end. A cycle is two rounds of
  the seven, 150 years 6 months of 360-day years, so a stored document
  outlasts a life, as the firdaria's does.
- **`decennial_order(apheta, places)`** puts the seven in order by
  longitude onwards from the apheta; a star at the apheta's very degree
  follows it.
- A new `DashaSystem` member **`DECENNIALS`** in a new family. Its year
  defaults to `SAVANA_360`, the year Valens states.
- **The apheta** is the sect's luminary, the sect under the request's lot
  rules. Whether a luminary is "well situated" is a judgement Valens
  gives no rule for, so it is not encoded: `ChartArea::decennials_from`
  begins them from any planet the consumer names, as `profection_from`
  begins the year from any point.
- A stored reading rebuilds from its periods: the first-level lords are
  the order itself (`DashaReading::decennial_order`), and a third-level
  period of exactly 129 days shows the `CYCLES` division
  (`DashaReading::decennial_division`), which no proportion of the
  minimum years gives; a reading of two levels rebuilds under the
  settings' division.
- **The division** is a knob, `decennial_division`: `PROPORTIONAL` (the
  default, chapter 6's tables) or `CYCLES` (chapter 5's worked count).

## What is not decided

- **C227, the apheta when the luminary is badly situated.** Valens's
  "well situated" is a judgement; the sect's luminary ships, and
  `decennials_from` takes any other.
- **C228, the third level.** Chapter 6's tables divide in proportion
  again; chapter 5's worked example gives each star an equal 129-day
  cycle inside Saturn's 30 months (seven of them overrun the 900 days
  by three), with the minimum years as days below that. The
  proportional reading ships; `CYCLES` is the knob. Chapter 7 counts
  the 129-day cycles from birth regardless of the years (19,116 days is
  the 149th cycle, 24 days in, Venus's), and offers 49-cycle "weeks" as
  an alternative; that is a count of its own beside the decennials, not
  a level of them, and is named here rather than encoded.

## The order of work

Both steps are done.

1. The kernel in `crates/dasha`, held to the five acceptance tests; the
   catalogue member and family.
2. The SDK's dispatch (the apheta under the request's lot rules),
   `decennials_from`, the rebuild, an integration test, every parity
   runner, a measured page.
