# The transit hit list: every event of a window, against one chart

Status: `draft`, 2026-09-27; §6 steps 1 and 2 **built** the same day. Written
before any code; the building is expected to correct it.

Derives from `gochar.md` §6 step 4 and the research page's P0 row "transit
hit list over a date range: every exact aspect, ingress, station, with
entering, exact and leaving times, sorted"
(`01-research/feature-universe/11-transits-gochar.md`, whose closing
checklist asks for it "as a first-class batch API over the crossing search
of the ephemeris port"). Gochar answers what the sky is at an instant; the
hit list answers **when** it changes, which is what a transit calendar, a
Sade Sati's phases and an alert all read.

## 1. What is searched

Every event is a crossing the SDK already searches for
(`astro-events-and-crossings.md`): a quantity of time reaching a lattice of
boundaries, found by the provider's own search where it declares one and
by the SDK's sample-and-bisect otherwise.

| event | quantity | lattice | notes |
|---|---|---|---|
| sign ingress | a body's longitude in the chart's zodiac | `SIGNS` | each crossing names the sign entered and the direction, so a retrograde re-entry is its own event |
| nakshatra ingress | the same | `NAKSHATRAS` | |
| station | a body's speed | zero | retrograde and direct, `events::stations` |
| exact aspect to a natal point | the same longitude | origin the natal longitude, step 30° | one search catches every multiple of 30°, filtered to the angles asked for |
| orb entered and left | the same | the aspect's line ± the orb | the two edges of an aspect's window; asked for separately |

Ketu is Rahu's opposite point in every chart, so its crossings are Rahu's
lattice shifted by 180° rather than a second search. The Moon crosses
every lattice many times a month; it is searched like any body and a
caller who does not want its events leaves it out.

## 2. The zodiac

A hit's sign must be the sign a chart founded at that instant would give,
so the search reads the **chart's** zodiac — the nutated ayanamsha under
`frame.ayanamsha_basis` — and not a `Frame`'s sidereal reading, which
applies the mean one and lands up to 18″ away
(`read-the-answer-back-elsewhere`, the annual chart's measurement). The
annual charts' search already reads it so (`LimbZodiac`); the hit list
reuses that source rather than keeping a second. The acceptance test is
the same: a chart founded at each ingress instant a moment before and
after stands in the sign the event says it left and entered.

## 3. The forks (cruxes)

| crux | question | readings | default | why |
|---|---|---|---|---|
| C145 | which aspects a hit list reports | the conjunction and opposition, which every tradition counts; the Western five (0, 60, 90, 120, 180); any set of multiples of 30° | **conjunction and opposition**, the request naming others | a Vedic aspect is a sign's (graha drishti) and not a degree's; degree-exact hits beyond the two both traditions share are the caller's choice, not the SDK's |
| C146 | the orb of an aspect's window | none (exact only); a fixed orb; an orb per body | **exact only**, the request naming an orb | no text read here sets one; the research page's hit lists (Kala, Solar Fire) take it as a setting |

## 4. The design

- **`teistro-astro`** keeps the searches; nothing new there if the
  crossing search already accepts a lattice with an origin (it does:
  `Lattice { origin_deg, step_deg }`).
- **`teistro-gochar`** gains `hits`: the event types and the filtering and
  sorting — `Hit { instant, body, event }`, `HitEvent::{ SignIngress,
  NakshatraIngress, Station, Aspect }`, an aspect naming the natal point,
  the angle and whether the instant is the exact line, the orb's entry or
  its exit. Pure data over the crossings; it depends on nothing new.
- **The façade**: `sdk.chart().hits(&natal, &HitRequest)`, where the
  request names the window, the bodies, the natal points, the events and
  the aspects and orb (C145, C146). One search per body and lattice, in
  the chart's zodiac; events sorted by instant, ties by body then kind so
  the order is total and the output reproducible.
- **A batch shape from the start**: the natal chart's points are read
  once, and a request over many natal charts founds nothing twice.

## 5. Tests

- Every sign ingress's instant, read back through a founded chart a
  second either side, crosses the boundary it names, in the direction it
  names (the consumer, not the search, as the acceptance).
- A retrograde passage over a natal point gives three exact hits, the
  middle one falling, and a station between each pair.
- Ketu's ingresses are Rahu's shifted by six signs, to the tolerance.
- The order is total: two runs give identical lists, and ties are ordered.
- Through a generated page over the recorded births: the counts of each
  kind over a year, and the search's cost per body, priced on CI before
  merging.

## 6. Order of work

1. The data model and the façade over ingresses and stations, with the
   read-back test: **done**. `Founder::transit_events` searches each
   graha over each lattice, and its stations, in the chart's zodiac
   through `Sidereal::over` — which the panchanga's and Tajika's eight
   hand-built copies of the same five fields now call too — and stamps
   the result; `teistro-gochar::hits` names the events and orders them;
   `sdk.chart().hits(&natal, &HitRequest)` joins them. Over a year every
   sign ingress reads back through a chart founded a second either side,
   under the default profile and the conformance one, and forcing the
   mean basis turns that red (Mercury entering Sagittarius read as
   Scorpio both sides). Over a classical astronomy the search reads the
   catalogue's zodiac and not the text's, as the annual charts' does; a
   hit list over one is not yet held to its own chart.

   **Found building it:** a chart founded on a clock far behind the
   place's own time could not be founded for part of every day. The day
   is chosen from the clock's civil date, and a Kathmandu instant at
   23:53 UTC is 05:38 the next morning, after that day's sunrise; the
   founder looked only for an instant **before** the civil date's
   sunrise and refused the other side with an `INTERNAL` error. It now
   takes the next day's arc too, with a test on a clock ten hours behind.
2. Aspects to natal points, with orbs (C145, C146): **done**. Each
   natal point gets one lattice from its own longitude at 30° steps,
   which holds every aspect's line on both sides at once, and with an
   orb two more shifted by it; a crossing is an aspect only at an angle
   asked for, and its edge and direction say whether the window opened
   or closed (a retrograde transit enters by the edge past the line).
   The natal points are the nine grahas and the lagna, the lagna being
   the one other point every chart has. Every exact hit of a year is
   exact in a chart founded at its instant, every edge stands the orb
   from it, and no window opens twice before it closes.
3. The measured pass and its price; then the boundary and the bindings.
4. Sade Sati's phases over the ingresses, once its source is in hand.
