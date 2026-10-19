# Where the official and the modern reckoning disagree

Status: `draft`, 2026-10-20. Design for the consumer brief's P1-10 (its
addendum 1, item 6), written before the code. The numbers in §4 are
measured once the comparison is built, not before.

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
for day in &differ.value {
    for limb in &day.limbs {
        // limb.limb, limb.official.member, limb.other.member,
        // limb.official.minutes_past, limb.other.minutes_past
    }
}
```

`disagreements(other, from, to, place, offset)` reckons the range under
both contexts, each in one crossing, and answers only the days on which
some limb's member at sunrise differs: a `Vec<DayDisagreement>`, each with
the date and one `LimbDisagreement` per limb that differs. Each side of a
limb disagreement is a `Reckoned { member, ends, boundary, minutes_past }`:
the member that side names at its sunrise, the instant **the official
member** ends under that side's sky, that side's sunrise, and the signed
minutes from the sunrise to that end. The official member ending after the
official sunrise and before the other's is the whole explanation, and the
numbers say it.

Both contexts must be at the same place and clock; the call takes one
place and one offset, so they cannot drift apart. The provenance is the
official context's, with the other context's settings hash recorded
beside it, so a cached comparison is invalidated when either changes.

Neither context needs to be the committee's: any two contexts compare,
which is what a reader comparing Lahiri with Raman, or Teimeris with the
built-in, wants too. The names in the record are `official` and `other`
because the first context is the one whose day the app shows.

## 3. The festival

`festival_disagreements(other, from, to, place, offset, request)` runs one
festival request under both contexts and answers each rule whose day
differs: the rule, the day each reckoning gives with the case and guard
that decided it (`Observance`), and the limb disagreements of the days
between the two (the official day and the other's day, inclusive), which
are where the deciding limb moved. A festival that one reckoning gives and
the other does not (a kshaya tithi in one sky only) is a disagreement with
one side absent, never dropped.

The festival's own boundary is the rule's time (`at`), which may be
aparahna or nishitha rather than sunrise. A first version reports the
sunrise limbs of the days involved and the case each side took; a reason
measured against the rule's own window is §5's second item.

## 4. Measured

Filled in by the comparison itself: the share of days, festivals and
Ekadashis that differ between the official and the modern reckoning over
Kathmandu, 2020 to 2030, and the spread of the minutes past the boundary.

## 5. Open

1. **Bindings.** The record crosses the boundary as JSON first (a
   `disagreements` section beside the almanac's), and as a typed record
   once its shape has held through one consumer release.
2. **The rule's own window as the boundary** for a festival (§3).
3. A reckoning on **a different engine** (the built-in against Teimeris)
   moves the sunrise as well as the limb; the record already carries each
   side's boundary, and the sentence names whichever moved.

[`ContextBuilder::nepal_official`]: ../../crates/sdk/src/context.rs
