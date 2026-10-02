# The firdaria (the `hellenistic` module, step 8)

Status: `built`, 2026-10-02 — written from al-Biruni's text and Abu
Ma'shar's as quoted, before the kernel, and corrected by building it;
measured in `time-lords-measured.md`.

The firdaria are the Persian time lords: life is divided into periods,
each ruled by a planet or a node for a fixed number of years, and each
planetary period is shared out in sevenths among the seven. They are the
third Hellenistic-tradition kernel behind the dasha crate's `Timeline`,
beside releasing and the profected year (`hellenistic-time-lords.md`),
so a chart request asks for them as it asks for Vimshottari and they
cross the boundary in the dasha sections every binding already reads.

## What the sources decide

**al-Biruni**, *The Book of Instruction in the Elements of the Art of
Astrology* (1029), tr. R. Ramsay Wright (1934), §395 and §§438–439, is
the rank 1 text read here. The table is on the printed page 48; its OCR
is lost, and it was read from the page image.

- **The order.** "The first period always begins with the sun in a
  diurnal nativity and with the moon in a nocturnal one"; the second is
  Venus's or Saturn's, "the remaining periods with the other planets in
  descending order" (§395). Day: Sun, Venus, Mercury, Moon, Saturn,
  Jupiter, Mars. Night: Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury.
- **The years.** Sun 10, Venus 8, Mercury 13, Moon 9, Saturn 11,
  Jupiter 12, Mars 7, then the Dragon's Head 3 and the Tail 2, "whether
  day or night": "a span of 75 years" (§438 and its note).
- **The sevenths.** A period's years are "distributed equally between
  the seven planets, the first seventh belonging exclusively to the
  chronocrator", the second shared with "the planet next below it", and
  so on (§395). The table prints each seventh: the Sun's 1 year 5 months
  4 days 7 hours, which is 10/7 years in months of 30 days.
- **The nodes.** They "have no association times with the planets"
  (§439): a node's period is not divided, and a node takes no seventh.

**Abu Ma'shar**, *On the Revolutions of the Years of Nativities* IV.1,
in Dykes's translation (2019), is read only as quoted at length by
Steven Birchfield, "The Fardārāt in Nativities" (2020). It agrees with
al-Biruni on the order, the years and the sevenths, and adds two things:

- **After 75 years** "it returns to the Sun": the cycle begins again.
- **The nodes by night** come "after the years of Mercury", when the
  native "enters year 71", which is al-Biruni's table read across.

## The acceptance tests

1. **§438, the day table.** A day birth's periods run Sun 10, Venus 8,
   Mercury 13, Moon 9, Saturn 11, Jupiter 12, Mars 7, Head 3, Tail 2,
   75 years in all.
2. **§438, the night table.** Moon 9, Saturn 11, Jupiter 12, Mars 7,
   Sun 10, Venus 8, Mercury 13, Head 3, Tail 2; the Head begins at year
   71 (Abu Ma'shar).
3. **§439, the sevenths.** The Sun's period is seven parts of 10/7 years
   each, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars; the Moon's
   sevenths are 9/7 years, Moon, Saturn, Jupiter, Mars, Sun, Venus,
   Mercury. The printed times (Sun 1y 5m 4d 7h, Moon 1y 3m 12d 21h,
   Venus 1y 1m 21d 5h, Saturn 1y 6m 25d 17h, Mercury 1y 10m 8d 7h,
   Jupiter 1y 8m 17d 7h, Mars 1y) agree with years/7 in 30-day months in
   every year, month and day. Their hours agree for the Sun (6.9), the
   Moon (20.6), Saturn (17.1) and Mars (0), and differ in three cells,
   named rather than fitted: Venus prints 5 for 10.3, Mercury 7 for 13.7
   and Jupiter 7 for 3.4. Venus's and Mercury's are the hours halved, as
   if counted in twelfths of a day, and Jupiter's repeats the Sun's; the
   test holds the years, months and days, and the hours of the four that
   agree.
4. **§439, the nodes.** The Head's and the Tail's periods have no
   sevenths.
5. **IV.1, the return.** Year 76 is the Sun's again by day.

## The design

- **`FirdariaDasha`** holds the round's nine lords (from the luminary
  that begins them, the nodes where the knob puts them), the birth and a
  year length. A mahadasha is one of nine periods; a planetary period
  has seven children, each a seventh, from its lord in descending order
  (Saturn after the Moon); a node's has none. A cycle is two rounds,
  150 years (`FIRDARIA_ROUNDS`), and cycles repeat after it: a document
  stores only cycle 0, and every binding answers `at` from the stored
  rows, so the stored cycle must outlast a life, as the profected
  year's 120 years do. A document stores two levels, the firdars and
  their sevenths, whatever the dasha group's depth asks beyond that.
- A new `DashaSystem` member **`FIRDARIA`** (43) in a new family
  `FIRDARIA` (8). Its year is the dasha group's `year_length`, defaulting
  to `JULIAN_365_25` as the profected year's does: neither source counts
  the firdaria in 360-day years, and al-Biruni's 30-day months are how
  he writes a seventh, not a year.
- **The sect** is read under the request's lot rules (`with_lot_rules`'s
  sect rule, Valens's horizon unless said), the same as the lots, so one
  chart is one sect for every Hellenistic time lord. A stored reading
  rebuilds from its first lord, the Sun by day and the Moon by night,
  and from where its stored round puts the Head
  (`DashaReading::firdaria_nodes`), so a document read under another
  context's knob still rebuilds its own periods.
- **The nodes' place by night** is a knob, `firdaria_nodes`: `END`
  (al-Biruni, Abu Ma'shar; the default) or `AFTER_MARS` (Bonatti, read
  through al-Qabisi, in both sects).

## What is not decided

- **C225, the nodes by night.** al-Biruni's table and Abu Ma'shar put
  them after Mercury; Bonatti after Mars. `END` ships, `AFTER_MARS` is
  the knob.
- **C226, the year.** Neither source says 360 days for the firdaria.
  The calendar year ships; a consumer may set `SAVANA_360`.

## The order of work

Both steps are done.

1. The kernel in `crates/dasha`, held to the five acceptance tests; the
   catalogue member and family; the knob.
2. The SDK's dispatch (the sect under the request's lot rules), the
   rebuild from the first lord, an integration test, every parity
   runner, a measured page (how many births change sect between the
   horizon and the daylight rules, which moves their whole firdaria:
   1 of the corpus's 55).
