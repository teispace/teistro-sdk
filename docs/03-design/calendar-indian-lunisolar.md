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

**The purnimanta reading does need one rule, and lacked it until
2026-09-30.** A purnimanta month runs full moon to full moon, so through
a dark fortnight its name is the next amanta month's, and `month::of`
advanced it for every dark fortnight. An adhika month is the exception:
a purnimanta almanac sets it whole, new moon to new moon, between the
nija month's dark and bright fortnights, so its own dark fortnight keeps
its name. Drik Panchang's purnimanta Ekadashi list for 2023 names Parama
Ekadashi (12 August, adhika Shravana's dark 11th) "Shravana, Krishna"
and Aja Ekadashi a month later, a nija dark 11th, "Bhadrapada, Krishna";
the code had named the first Bhadrapada. Nepal writes the same whole
month: its committee announces malmas from one new moon to the next
(§9). `nepal-month-measured.md` counts every adhika day's purnimanta month
against its amanta one, and was red before the rule.

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

## 9. Nepal's malmas, measured

Nepal calls the adhika month *malmas* (मलमास), where north Indian usage
often gives that word to Kharmas, the Sun in Sagittarius or Pisces
(C177). The Nepal Panchanga Nirnayak Vikas Samiti announces each one's
span, and [`nepal-month-measured.md`](nepal-month-measured.md) holds the almanac
at Kathmandu to five announcements, BS 2072 to 2083, one of them given
as instants.

Both shipped readings mark exactly the announced days: `nepali-default`
over the built-in ephemeris, and `surya-siddhanta` over the text's own
sky and zodiac. So the drik-against-text question of §8 does not arise
for the month's span in these years, and a Nepali consumer gets the
committee's malmas from either. What the measurement does show is that
the zodiac must match the sky: the text's Sun read in Lahiri's zodiac
puts its sankrantis a day or two late and marks the month before in
most of the recorded years, which the page counts. That is a consumer's own mix (an ephemeris and a
profile chosen apart), and the page keeps it as a rival that must stay
falsified.

The one announcement with instants (BS 2072) closes nearer the text's
new moon than the modern one, and opens hours from both, which reads as
a misprint or a different event. So the instants are set beside the new
moons on the page, in minutes, and decide nothing.

The day rule the announcements follow is the almanac's own: an adhika
day is one whose sunrise falls inside the adhika month.

**Nepal names its months purnimanta** (C183). The committee rules that
Mahashivaratri is फाल्गुन कृष्ण चतुर्दशी, where an amanta almanac and
*Dharmasindhu* say Magha; Gai Jatra, the day after श्रावण शुक्ल
पूर्णिमा, is भाद्र कृष्ण प्रतिपदा, so the name turns at the full moon.
The page's §4 holds six dated designations to the month the profile
leads with, and one of them, a worked example written against the solar
month, rules out reading the names as Bikram Sambat months. So
`nepali-default` leads with the purnimanta month from its version 2.
Only the lead moves: every rule that decides a festival or a muhurta
reads the amanta month by name, and both are carried on every day. The month is said
with its kind by one message, `sdk.calendar.lunarMonth`, which matches on
the kind's key the way `lifeClass` matches a class: अधिक ज्येष्ठ in
Nepali, "Adhika Jyeshtha" in English, and the bare month for a nija one.
The kind stays out of the catalogue, since a catalogue kind is open to
additions and the FFI's month kind relies on matching every member.


## 10. The year's name, measured

A lunar year opens at the new moon that begins its first Chaitra, an
adhika Chaitra when there is one, and its first day is the one whose
sunrise follows that new moon. The sixty-year cycle names it, and
`samvatsara-measured.md` measures how.

**The count is the text's.** The Surya Siddhanta counts the signs its
mean Jupiter has crossed since the Kali age and reads the count from
Vijaya (I.55); `teistro_calendar::samvatsara::JovianYear` is that count
in integers, with each Jovian year's bounds. Burgess's own worked example
is a test: the year begun in February 1859 is the 5019th, Prajapati. His
note adds the bija, eight revolutions of Jupiter fewer an age, which puts
the same year's start forty days later. The Nepali and Indian press date
Jovian years by the text without it, and the bija misnames five of the
seven years on the page. The count is independent of the almanac's sky,
because it is a definition the text makes and not a measurement: a
modern mean Jupiter changes sign two months later.

**The name is read at Chaitra Shukla Pratipada.** The committee's chair
put the rule in terms when Ananda was expunged in 2078: the samvatsara
that meets the day of Chaitra Shukla Pratipada is the one said in the
year's rites. A Jovian year is 361 days, so now and then one begins after
a pratipada and ends before the next, and names no year (`lupta`). The
day is read at its sunrise, as a panchanga day is (C185).

**No name names two years.** The same Jovian year can meet two
pratipadas a 354-day year apart. VS 2080 is the case on record: Nala met
its pratipada, having named 2079, and Pingala began two days later. The
rule as written names 2080 Nala, and some north Indian panchangs printed
that; the committee named it Pingala (C184). `BARHASPATYA` takes the
committee's reading, a year named one ahead when its running name has
been used, and the lead lasting until a year whose count rises by two,
which is where the expunged name falls. A short walk back computes it.
The page holds the walk to the full recursion it stands for over six
centuries. `BARHASPATYA_RUNNING` is the literal reading, and
`CHANDRAMANA` is the south's, which reads the Shaka year and skips
nothing.

**What it costs, and where it is.** `sdk.almanac().years` answers the
lunar years a range falls in, each with its name, its Vikrama and Shaka
numbers, its bounds, the Jovian years that ran in it and the one it
expunges; `AlmanacRequest::with_years` asks for them beside the days in
one crossing. The roadmap asked for the samvatsara on every day. A list
of years beside the days carries the same answer, since a day's year is
the one holding its sunrise, and costs nothing when it is not asked. A
field on every day would have cost every day a few new-moon searches,
which the instruction-count gate refuses at three percent.

**At the boundary.** A panchanga request asks for the years with a bit,
`TS_PANCHANGA_YEARS`, in its `sections`, the field that was reserved and
written zero before, so a caller compiled against an older header asks
for none. The years come back in the blob's `years` section as the
canonical envelope, sealed over the value with every member written in
full (`samvatsara.PRAMADICHA`), which `LunarYear::MEMBERS` lists and a
test holds to serde both ways. Every binding names the bit as an option
(`years: true`, `years=True`) and hands back `{value, provenance}`, as
the façade's `Envelope<Vec<LunarYear>>` has it. Every parity runner
prints the years across a pratipada.

Writing the bit found that a chart request's twelve `sections` bits were
never described as constants. The code said they were "named in the
header", but the header did not carry them, and each binding kept its
own copy of the numbers. They are described now, so the header and
every binding generate them, and the copies are gone.
