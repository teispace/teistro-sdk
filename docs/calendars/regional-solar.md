# The regional solar calendars: what is measured, and what waits

Status: `research`, 2026-10-20, the Siddhanta's Sun measured 2026-10-21. P2-1 step 2. Nothing is shipped yet.

The Tamil, Bengali, Odia, Malayalam, Assamese and Punjabi solar calendars
begin each month at a sankranti, the Sun's entry into a sidereal sign.
Bikram Sambat does the same. What differs between them is the civil day
the sankranti is placed on, and the place the almanac reckons for. The
engine that builds Bikram Sambat (`bikram_sambat::Engine`) already takes
the rule as a parameter, and the rules the sources name are built as
`solar::MonthStartRule`:

| rule | tradition (Sewell and Dikshit, *The Indian Calendar*, 1896, §28) |
|---|---|
| `SankrantiDay` | Orissa: the civil day of the sankranti |
| `FollowingDay` | Bengal: the civil day after it |
| `BeforeSunset` | Tamil: that day if before sunset, else the next |
| `BeforeAparahna` | Malabar: that day if before three fifths of the daylight, else the next |

So a regional calendar is a place, a rule and a year-start sign over a
machinery that exists. What it lacks is evidence: a published list of
month beginnings for each region, reckoned with the astronomy the SDK
would ship. That list would be both the acceptance test and the measure
that picks the rule.

## What was measured

**A Tamil list reckoned by another astronomy.** Prokerala's Tamil
calendar for Chennai gave 108 usable month beginnings, Chithirai 2020 to
Panguni 2028. Its pages beyond that window show the current month instead,
and were dropped. Those 108 were set against the Lahiri sankranti
(pyswisseph, Moshier) and Chennai's sunrise and sunset:

| rule | months matched |
|---|---:|
| before sunset | 90 |
| before three fifths of the daylight | 90 |
| the sankranti's civil day | 65 |
| the day after | 43 |
| sunrise to sunrise | 41 |

No shift helps. Moving every sankranti uniformly, in quarter hours across
±12 hours, under a cutoff at sunrise, three fifths of the daylight, sunset
or midnight, never matches more than 92. The months that part do not sit
near any cutoff. Some sankrantis at 07:22 begin the next day, while some at
18:36 begin the same day. So the list's sankranti instants are not the
modern Sun's: it is most likely a vakya reckoning, whose instants differ by
hours. It cannot test a rule for a drik calendar, and it is not a corpus.

**The same list over the Surya Siddhanta's Sun.** A source says the Bengal
calendars in India are reckoned by the Surya Siddhanta, so the 125 month
beginnings of the Chennai list that a sankranti falls near (2020–2030) were
set against the SDK's Siddhanta Sun (`nepali-committee`'s sky) as well as
Lahiri's, each under every rule:

| rule | Siddhanta | Lahiri |
|---|---:|---:|
| the sankranti's civil day | 110 | 66 |
| before sunset | 102 | 107 |
| before three fifths of the daylight | 80 | 107 |
| sunrise to sunrise, then the next day | 33 | 84 |
| the day after | 10 | 59 |

The Siddhanta's civil day comes closest, and its fifteen misses settle it:
sankrantis at 21:08 to 23:39 begin the next day, while ones at 00:13 to 05:52
the morning after are printed on the day before. No cutoff orders those, so
the list's instants are not the Siddhanta's either. It stays what the first
measurement said, a third reckoning (most likely vakya), and still no corpus.

**A drik list, not reachable.** Drik Panchang publishes the Thiru Ganita
(drik) Tamil months, and shows each day's solar date beside its Gregorian
one. Its pages answer automated requests with a reCAPTCHA, so it was not
read further.

## What would unblock it

- A printed or official list per region, reckoned by the modern Sun: the
  *Rashtriya Panchang* (Positional Astronomy Centre, Kolkata), which gives
  the solar months of the regional calendars, or a printed Thirukkanitha
  panchangam.
- With one, the step is a measurement on the existing engine (as
  `calendars bs-fit` does for Bikram Sambat), then a calendar per region
  with its table generated for 1800–2400 and gated against the list.
- Malayalam begins its year at the Simha sankranti, Chingam, not Mesha.
  A second-hand reading of Sewell and Dikshit's "Malabar rule" doubts it,
  noting that Malayalam almanacs began Dhanu on the same day as the Tamil
  Margazhi. So the measurement, not the 1896 rule, should decide Kerala.
