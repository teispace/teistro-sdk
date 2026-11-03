# Mundane charts

Status: `built`, 2026-11-01. Written for the application's SR-45
(addendum 3): mundane charts as named chart kinds, cast for a supplied
place, and event charts cast from a supplied record with its source.

## 1. The kinds

A mundane chart is a chart of a moment the sky names rather than a
birth. The catalogue kind `mundane_kind` (130) names each moment the SDK
finds, and every chart cast for one is `chart_kind.MUNDANE` (an event's
is `chart_kind.EVENT`).

| Kind | The moment | Source |
|---|---|---|
| `MESHA_SANKRANTI` | the Sun's entry into sidereal Mesha, in the context's zodiac | Narada Samhita 2.2; Jyotirnibandha p. 29 vv. 7, 10 |
| `MAKARA_SANKRANTI` | the Sun's entry into sidereal Makara | Narada Samhita 2.3 |
| `ARIES_INGRESS`, `CANCER_INGRESS`, `LIBRA_INGRESS`, `CAPRICORN_INGRESS` | the Sun at tropical 0°, 90°, 180°, 270° | Ptolemy, *Tetrabiblos* II.10 |
| `NEW_MOON_BEFORE_INGRESS` | the last new moon before an ingress | Brihat Samhita ch. 19; *Tetrabiblos* II.10 |
| `FULL_MOON_BEFORE_INGRESS` | the last full moon before an ingress | *Tetrabiblos* II.10 |
| `SYZYGY_BEFORE_INGRESS` | whichever of the two most nearly precedes it | *Tetrabiblos* II.10 |
| `SOLAR_ECLIPSE`, `LUNAR_ECLIPSE` | the next eclipse, at its greatest | *Tetrabiblos* II.4-II.6 |
| `GREAT_CONJUNCTION` | the next conjunction of Jupiter and Saturn in geocentric longitude | Abu Ma'shar, *On the Great Conjunctions* I.1 (mark T) |
| `EVENT` | an event on record, cast as given | structural |

## 2. What the texts say

**The year's beginning.** Ptolemy (*Tetrabiblos* II.10, Robbins, Loeb
1940) asks what the "new moon of the year" is. He grants the solstices
and the equinoxes each a claim, and holds that the most natural start is
the spring equinox. He then reads the new or full moon most nearly before
each of the four points. The Indian year's offices are read from the
sidereal ingresses instead: Mesha's day names the minister (Narada
Samhita 2.2, Jyotirnibandha p. 29), and Makara's the lord of dry goods
(Narada Samhita 2.3; `year-rulers.md`). The SDK ships all six ingresses
and has the caller name one. No kind is chosen for them (C1150).

**The lunation.** Brihat Samhita ch. 19 (Chidambaram Iyer, 1884) makes
the year's regent the lord of the weekday on which Chaitra's new moon
falls. The amanta Chaitra opens with the new moon before Mesha's ingress,
except in a year with an adhika month. A lunation kind therefore reads
`MESHA_SANKRANTI` when its `ingress` is left out. Ptolemy's own reading,
whichever syzygy comes nearer, is `SYZYGY_BEFORE_INGRESS` (C1151).

**The place.** Ptolemy casts the moment for each city's own horizon
(II.10, and II.5 for an eclipse). No text fixes one capital, so the
place is the caller's and has no default (C1152).

**The eclipse.** The chart is cast for the greatest eclipse, the
instant the eclipse search answers. Ptolemy reads the eclipse's hour at
the place (II.6). That hour is a reading of the same chart, not another
moment (C1154).

**The great conjunction.** The Persian and Arabic tradition (Abu
Ma'shar) reads Jupiter and Saturn's conjunction in longitude. Where a
retrograde loop makes it triple, the SDK answers the first of the three
(C1153). The kind is marked T: the rank-1 text has not been read here.

## 3. How a moment is found

Nothing here solves on its own. An ingress is the Sun's longitude
crossing its degree through the crossing search (`astro::events`), read
in the context's zodiac through the same `ChartZodiac::searched` a
return uses, or in the tropical one. A lunation is the elongation
crossing 0° or 180° in the 32 days before its ingress. An eclipse is
`Eclipses::next_solar` or `next_lunar` under the profile's shadow rule;
a classical astronomy's eclipse is its own method and is refused. A
great conjunction is Jupiter less Saturn crossing 0°, within twenty
years and a quarter, sampled every 10 days (`CONJUNCTION_STEP_DAYS`)
rather than the search's daily step: the difference moves under half a
degree a day and a triple conjunction's passes are months apart, so no
pass falls between samples, and each is still refined to the search's
tolerance. The daily scan cost 1.37 s for one conjunction. The coarse
step is held against the daily scan over the triple conjunction of
1940–41, every pass at the same instant.

`year` starts the search at 1 January 0h UTC; `from` starts it at an
instant. The answer is the moment (`MundaneMoment`), sealed by the chart
cast there with the request as its input. `ChartArea::mundane` adds the
chart; `ts_chart_mundane` and the bindings answer the moment, and a
binding casts the chart with its own chart reader.

## 4. Acceptance

`crates/sdk/tests/mundane.rs` holds each kind to a published instant, to
two minutes:

- the U.S. Naval Observatory's equinox of 20 March 2024, 03:06 UT, its
  December solstice of 2024 and its equinox of 2025;
- its new moon of 10 March 2024 and full moon of 24 February 2024, the
  lunations before the 2024 equinox;
- NASA's total solar eclipse of 8 April 2024 and total lunar eclipse of
  14 March 2025, each at its greatest;
- the great conjunction of 21 December 2020, about 18:20 UT in
  longitude, to half an hour (the sources spread that far);
- the national panchanga committee's Mesha sankrantis of BS 2082 and
  2083 (`fixtures/official/npns-2082-2083.json`), under the
  `nepali-committee` profile.
