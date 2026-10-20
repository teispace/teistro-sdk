# Where the official and the modern reckoning disagree

Status: `built`, 2026-10-20. Design for the consumer brief's P1-10 (its
addendum 1, item 6), written before the code and corrected by it. §4 is
measured through the built comparison (`crates/sdk/tests/disagreements.rs`).

An app in Nepal shows two reckonings of one day. The **official** one is
the Nepali committee's: `nepali-committee` over the Surya Siddhanta, with
a modern engine giving the sunrise ([`ContextBuilder::nepal_official`],
every binding's `nepalOfficial`). The **modern** one is `nepali-default`,
the drik reading, over the same engine. They name the same day's tithi
differently about one day in ten, and for a festival that decides on
which day it falls. The app shows the official day and explains the
modern one, and needs from the SDK both answers and the reason for each
difference.

## 1. What a reason is

Every difference between the two reckonings of a day starts at one place:
**a limb's end and a boundary of the day fall in the opposite order**. A
tithi that ends at 06:02 by the text and at 05:58 by the modern sky,
against a sunrise at 06:00, is the sunrise tithi by the text and the next
one by the modern sky. The two skies differ by minutes (the Surya
Siddhanta's Moon runs up to a few hours from the modern one at the
extremes, its Sun a fraction of a degree), so the difference is always one
member of one limb, and the reason is three instants and their order:

- the **limb** (`tithi`, `nakshatra`, `yoga`, `karana`) and the member
  each reckoning names at the boundary;
- the **boundary**: the day's sunrise, which is what a day's limb is named
  at (an udaya tithi); a festival's own time (aparahna, pradosha,
  nishitha) for a festival, §3;
- the **end** of the official member under each reckoning, and the
  boundary under each (the sunrise moves too, by seconds, when the two use
  different engines; here they share one).

The sentence the app prints is generated from those fields, never stored:
"by the official reckoning Ekadashi ends at 06:04, four minutes after
sunrise; by the modern one at 05:51, nine minutes before". So the record
carries **minutes signed against the boundary**, under each reckoning,
and nothing that is only words.

## 2. The day

```rust
let official = Context::builder().nepal_official([teimeris()]).build()?;
let modern = Context::builder().profile("nepali-default").ephemeris([teimeris()]).build()?;
let differ = official.almanac().disagreements(&modern, &from, &to, &place, offset)?;
for day in &differ.value.days {
    if let Some(tithi) = &day.tithi {
        // tithi.official.member, tithi.other.member, and the instant each
        // gives way: tithi.official.change?.minutes_from_sunrise
    }
}
```

`disagreements(other, from, to, place, offset)` reckons the range under
both contexts, each in one crossing, and answers a `DayDisagreements`:
the other context's profile and settings hash, how many days were
compared, and only the days on which some limb's member at sunrise
differs. Each `DayDisagreement` has the date and an optional
`LimbDisagreement` per limb (`tithi`, `nakshatra`, `yoga`, `karana`),
present when that limb differs. Each side of a limb disagreement is a
`ReckonedLimb { member, span, sunrise, change }`: the member that side
names at its sunrise, that member's whole span under its sky, its
sunrise, and the `LimbChange { at, minutes_from_sunrise }` where the two
members meet under that sky.

Which instant that is follows from which side runs ahead. When the other
names the member after the official one, the official member ended after
the official sunrise and the other's began before the other's: the
official change is its member's end and the other's its member's start.
Run the other way round, they swap. Either way the two minutes have
opposite signs (the test holds that), and the pair is the whole
explanation. Two members that are not neighbours (a member one sky skips)
name no single instant, and `change` is absent on both sides.

A range holds at most 366 days, as `AlmanacArea::of`'s does; a decade is
read a year at a time. Both contexts must be at the same place and clock; the call takes one
place and one offset, so they cannot drift apart. The provenance is the
official context's, with the other context's settings hash recorded
beside it, so a cached comparison is invalidated when either changes.

Neither context needs to be the committee's: any two contexts compare,
which is what a reader comparing Lahiri with Raman, or Teimeris with the
built-in, wants too. The names in the record are `official` and `other`
because the first context is the one whose day the app shows.

## 3. The festival

`festival_disagreements(other, from, to, place, offset, request)` runs one
festival request under both contexts and answers a `FestivalDisagreements`.
Its `observances` hold each occurrence of a rule (paired by rule, amanta
month, adhika and overlapping tithi) whose day differs, with each side's
whole `Observance`: the day, the case between the tithi's two days and the
extent of the rite's window each held, which is the reason. Its
`ekadashis` do the same for the Ekadashi fasts, each side's `EkadashiFast`
carrying the vedha and excess that decided. An occurrence that one
reckoning gives and the other does not (a kshaya tithi in one sky only) is
a disagreement with one side absent, never dropped.

The festival's own boundary is the rule's time (`at`), which may be
aparahna or nishitha rather than sunrise; the observance's extents already
measure each day against it. Linking a moved festival to the day
disagreements between its two days is §5's second item.

## 4. Measured

Kathmandu, 2020 to 2030, one year per call: the official reckoning
(`nepal_official`, the built-in ephemeris giving the sunrise) against
`nepali-default` over the built-in ephemeris. Reproduced by:

```text
cargo test --release -p teistro --test disagreements -- --ignored --nocapture
```

| over 4,018 days | days | share |
|---|---:|---:|
| some limb's member at sunrise differs | 1,223 | 30.4% |
| the tithi | 379 | 9.4% |
| the nakshatra | 335 | 8.3% |
| the nitya yoga | 356 | 8.9% |
| the karana | 748 | 18.6% |

Every one of the 379 tithi disagreements is two neighbouring members, so
each has its instant on both sides; no member is skipped by one sky
alone. The instant lies a median of 85 minutes from the official sunrise
(90th percentile 208, largest 312) and 84 from the modern one (230, 362).
The two instants are one boundary under two skies, on opposite sides of
nearly one sunrise, so the skies place that boundary the sum of the two
apart: an hour or more on most of these days.

| over the same days | given by the official reckoning | on another day | under another month | on one side only |
|---|---:|---:|---:|---:|
| `NEPAL` observances | 312 | 16 | 3 | 4 |
| `NEPAL_PANCHANGA` observances | 1,091 | 95 | 3 | 28 |
| Ekadashi fasts (either pack) | 816 | 72 | 17 | 0 |

**Under another month** is the year the two skies place an adhika month
apart. In 2028 the text has an adhika Kartika the modern sky does not, so
every month from it to January 2029 is named one apart. The first version
paired occurrences by their month and read each fast there as two
one-sided ones (36 of them). An occurrence is one rule over one tithi, so
pairing is now by the overlapping tithi, and a different month is
reported as a difference in its own right. A test holds the 2028 case.

What the comparison costs is what the two reckonings cost, since it
reckons each once: `05-testing/app-speed-measured.md` times both setups
over Teimeris (a year of days in about 0.3 s for the committee's, 0.56 s
for the modern), and the built-in ephemeris at some 25 times that.

## 5. Open

1. ~~**Bindings.**~~ Built 2026-10-20 (§6).
2. **A moved festival linked to its days' limb disagreements** (§3).
3. A reckoning on **a different engine** (the built-in against Teimeris)
   moves the sunrise as well as the limb; the record already carries each
   side's boundary, and the sentence names whichever moved.

## 6. At the boundary

`ts_almanac_disagreements(context, other, request_json, out_json)` is the
first entry taking two contexts, and every binding's
`almanac.disagreements(other, request)` crosses it. The request is the
record `DisagreementRequest::from_json` reads: the range, the place, the
offset, `days` (true when left out) and `festivals`, a festival request;
one that compares nothing is refused as `disagreements.days`. The answer
is `{days, festivals}`, each null when not asked and otherwise the
record above with every catalogue member written in full, as every
other section writes them, sealed by the first context's provenance
(`AlmanacArea::disagreements_request`).

The answer is JSON rather than a blob because it is sparse: a tenth of
the days, a handful of festivals, nested two sides deep. Each binding
reads it into typed records (Node's frozen objects and declarations,
Python's dataclasses, Dart's values, Java's `Disagreements` records),
each side of a festival through the decoder the festival section uses.

A second handle needed three things the generators had never met. The
receiver is the **first** handle (`rules::methods`), so a later one of
the same class is a parameter of the class that holds it, not `self`
again. Its own provider can be reached by the call, so the Node and
wasm wrappers lend it the environment beside the receiver and take both
back before either's failure is reported, Python and Dart bracket the
call with both contexts' provider guards, and Dart checks it is alive
as it checks the receiver. Java holds both contexts' locks, taken in
one order by a creation ordinal, so two threads comparing a pair either
way round cannot deadlock.

[`ContextBuilder::nepal_official`]: ../../crates/sdk/src/context.rs
