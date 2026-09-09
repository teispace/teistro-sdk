# The Indian lunisolar calendar

Status: `design`. Written from
[`calendar-indian-lunisolar-measured.md`](calendar-indian-lunisolar-measured.md),
which measured every rule this page states over 12 368 lunar months of a
millennium and against all fifty-five recorded days. Phase 2 named this
page and nothing wrote it; Phase 4's `panchanga` is what now needs it.

## 1. Purpose and scope

`panchanga` names the lunar month a day belongs to — Shravana, Ashadha —
from the solar sign the Sun stood in at the month's opening new moon.
That is right, and the corpus bears it out on every recorded day. What it
cannot say is whether that month is **adhika**, the intercalary month
inserted to keep the lunar year with the solar one, or **kshaya**, the
month omitted when the solar year outruns the lunar. `panchanga-day.md`
§8 records that as the calendar's to decide, and the roadmap says the
same: *"adhika and kshaya are the calendar's to decide"*.

**This page is about the mark, and not about dates.** The distinction
matters more than it looks and §6 is why: a full Indian lunisolar
calendar — one that converts a date to a fixed day and back — needs a
date shape the SDK does not have, and the measurement says so with a
number. The mark needs none of that. Separating them is what lets
`panchanga` be finished now without a boundary change nobody has argued
for.

So the scope is: **classify the lunar month an instant falls in**, and
nothing else. Out of scope, and §6 says when each returns: converting a
lunisolar date, naming a lunisolar year, the era, and the regional
variants that begin the year at a different month.

## 2. The rule is a count

A lunar month runs from one new moon to the next. A solar month runs from
one sankranti — the Sun's entry into a sign — to the next. A lunar month
is about 29.53 days and a solar month about 30.44, so a lunar month
usually holds exactly one sankranti and takes that solar month's name.

Usually. **The rule is the count**:

| sankrantis in the lunar month | what the month is |
|---|---|
| none | **adhika**: intercalary. The name repeats — the next month takes it too, as *nija*, the true one |
| one | ordinary |
| two | **kshaya**: omitted. The second name is skipped that year |

Measured over 12 368 months of a millennium: 388 adhika, 11 961
ordinary, 19 kshaya, and **none** with three or more. The classification
is total and the three cases are all of them.

It reproduces the corpus on **all fifty-five** recorded days, including
the two marked adhika — Delhi in August 1947 and Fairbanks in June 2015.
That is the whole of the evidence the corpus can give: it records
`is_adhika` and none of the inputs, so it can falsify a wrong rule and
cannot derive the right one.

## 3. The text is enough, and what it is not enough for

The rule needs new moons and sankrantis, and the **Surya Siddhanta's own
Sun and Moon** give both. So this calendar computes from the text, as the
Bikram Sambat engine does, and takes no ephemeris — which is also what
the tradition did.

That is a design decision and not only a convenience. A calendar that
needed a provider could not be asked for a month without one, and
`panchanga` already has a provider; but the *calendar* should not, because
a calendar is a rule about the sky and not a measurement of it.

What the text is **not** enough for, measured rather than supposed: the
classification is robust and *which month an instant belongs to* is not.
Whether a sankranti falls inside a 29.5-day window barely moves when a
boundary shifts by half an hour. Which month a given moment belongs to
does — and the one recorded day where the text and the recording disagree
is **nineteen minutes** from the eclipse new moon of 8 April 2024, with
the text's conjunction and the recording's on either side of it.

So a consumer is told which answer is which. **The mark is the text's to
give.** *Which month a moment within an hour of a new moon falls in* is
not, and a caller who needs that wants drik values — which wants the
conformance harness over an adapter that Phase 1 deferred.

## 4. An adhika month needs no naming rule

This is the page's one genuine surprise, and it is worth stating plainly
because the usual formulation is different.

An adhika month has no sankranti, so it has nothing to be named by, and
the common statement is that it *takes the name of the following nija
month*. That is a rule, and it would have to be written and tested.

It is not needed. The Sun stands in the **same sign** at the adhika
month's opening new moon as at the following nija month's — that is what
having no sankranti between them means — so the existing rule, "the month
is named for the sign the Sun stood in at its new moon", gives both the
same name by itself. August 1947 is Shravana twice over, once adhika and
once not, and `limb::amanta_month` already returns Shravana for both.

**So `panchanga` needs no change to its naming and only gains the mark.**
The measurement checked this rather than assuming it: the naming
reproduces fifty-four of the fifty-five recorded days, and the one
exception is the nineteen-minute boundary of §3, not a naming rule.

## 5. The API

```rust
/// What a lunar month is, beyond its name.
pub enum MonthKind {
    /// One sankranti: the ordinary month.
    Nija,
    /// None: intercalary, and the name repeats.
    Adhika,
    /// Two: the next name is skipped this year.
    Kshaya,
}

/// The Moon a lunisolar calendar needs, beside the Sun a `SolarModel`
/// gives.
///
/// A trait of its own rather than a method on `SolarModel`: a solar
/// calendar needs no Moon, and five implementations of that trait —
/// three of them test doubles — would have to grow one for nothing.
pub trait LunarModel: Send + Sync {
    /// The Moon's elongation from the Sun, degrees in `[0, 360)`, which
    /// is nought at a new moon.
    fn elongation_deg(&self, jd_ut: f64) -> Result<f64, Error>;
}

/// The lunar month an instant falls in: when it opened and closed, what
/// it is, and the sign that names it.
pub struct LunarMonthSpan {
    pub from: JulianDay<Utc>,
    pub to: JulianDay<Utc>,
    /// The sign the Sun stood in at `from`, which names the month.
    pub sign: u8,
    pub kind: MonthKind,
}

/// The lunar month an instant falls in.
pub fn month_at(
    sun: &dyn SolarModel,
    moon: &dyn LunarModel,
    at: JulianDay<Utc>,
) -> Result<LunarMonthSpan, Error>;
```

Two collaborators rather than one, and the same pair everywhere, because
the two must agree: a new moon found with one model and a sankranti with
another is a month bounded by one sky and named by another.
`SuryaSiddhanta` implements both, so the ordinary call passes it twice.

`panchanga`'s `month::of` gains the kind and keeps everything else. Its
`limb::amanta_month` is already right (§4) and stays.

## 6. What a full calendar would need, and why it waits

`Calendar::IndianLunisolar` is in the catalogue and `shipped()` returns
`None` for it. Making it a `CalendarSystem` — `date_of`, `fixed_of`,
`month_length`, round-tripping — is a larger thing than the mark, and the
measurement says exactly how much larger.

**A lunisolar date's day is the tithi running at sunrise**, and a tithi
is not a day long: it runs about twenty-three to twenty-six hours. So it
can catch two sunrises, and the day repeats; or none, and a day is
missing from the month. Over 365 234 sunrises at Ujjain the day
**repeats one day in forty-four** and is **skipped one in twenty-six**.

`(year, month, day)` is therefore **not a key**. A Hindu lunisolar date
needs **two** flags — which of a repeated pair a day is, and whether its
month is the adhika one — and `CalendarDate` carries neither: `month` and
`day` are plain `u8`s.

That leaves two ways, and this page deliberately takes neither yet:

- **Extend `CalendarDate`.** It is the honest shape, and the struct is
  already a union of what calendars need (`era` is meaningless to the
  Gregorian, `computed_month` to everything but a divergent Bikram
  Sambat). But it crosses the boundary as `ts_calendar_date` and so
  reaches the C header, three ergonomic layers, the parity gate and the
  description's struct inventory.
- **Give `day` a different meaning** — the count of days since the month
  began, which has no repeats — and say plainly that the result is not
  the date a panchangam prints. Cheap, and a dead end for the consumer
  who wanted the printed date.

The reason to wait is not the cost. It is that **nothing needs it yet**:
`panchanga` needs the mark, the corpus records no lunisolar dates to
check a conversion against, and a boundary change argued for by nobody is
the kind of thing this project asks to be measured first. When a consumer
wants the date, the measurement is already on the page and the decision
can be made on it.

## 7. Tests

- **The rule against the corpus**, in `check-lunisolar`: all fifty-five
  recorded days, which the pass already holds.
- **The classification is total**, over the millennium: every month is
  one of three, and none holds three sankrantis.
- **The mark round-trips through `panchanga`**: a day in an adhika month
  reports it, and the day after the month ends does not.
- **The two models must be the same sky**: a `month_at` given a Sun and a
  Moon that disagree is a caller's mistake the type system cannot catch,
  so the doc says it and a test shows what goes wrong.
- **Kshaya has no test**, only a measurement (§8).

## 8. Open questions

- **Kshaya is measured and not tested.** The corpus records none, so
  nothing holds the rule to an authority. What can be said is that its
  frequency is what the astronomy predicts — one in some fifty years —
  and that every one of the nineteen in the sample falls between
  Vrishchika and Kumbha, the perihelion window where the Sun moves fast
  enough to cross two sign boundaries inside one lunar month. A rank-1
  panchangam naming a kshaya year would turn the measurement into a test.
- **The regional year.** Chaitra opens the year in the north and Kartika
  in Gujarat; the year *number* also differs (Vikrama, Shaka). None of
  that touches the mark, and all of it touches a date, so it waits with
  §6.
- **Drik against the text.** Every number on this page is the text's. A
  modern almanac reckons *drik*, and the two part at a month boundary by
  minutes (§3). Which of the two a profile should use is a knob the
  settings do not have, and the answer is probably the same one
  `panchanga.moon_events` already takes: the profile's, with the text as
  the default a classical profile keeps.
