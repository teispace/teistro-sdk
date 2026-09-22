# Examples

Every file here is a runnable program that `cargo xtask check-dart`
runs, so none of them can drift from what the binding does. They are
meant to be read in order: each one assumes the one before it.

```sh
cargo build --release -p teistro-ffi
cd bindings/dart
TEISTRO_LIBRARY=../../target/release/libteistro_ffi.dylib \
dart run example/birth_chart.dart
```

| file | the scenario | what it is really teaching |
|---|---|---|
| [`quickstart.dart`](quickstart.dart) | the smallest thing that works | opening the library, a context, one call of each kind |
| [`birth_chart.dart`](birth_chart.dart) | a Nepali birth record to the nine grahas placed by sign, nakshatra and pada | the canonical frame is **tropical**; a Vedic chart asks for a sidereal one and the SDK completes it. Also: a zone's history matters, and Rahu is a body the chart calls a graha |
| [`panchanga.dart`](panchanga.dart) | the five limbs of a day — tithi, vara, nakshatra, yoga, karana | every limb but the weekday is a function of two longitudes, so a binding can compute a whole almanac from `positions`. The karana is not a cycle of eleven, and the rule is the SDK's own |
| [`calendar.dart`](calendar.dart) | a Bikram Sambat year and one month as a calendar page | month lengths are decided by the Sun and vary year to year, so they are asked for and never assumed; and every date says whether it came from the official table or the SDK's engine |
| [`ephemeris.dart`](ephemeris.dart) | a year of the sky in one call | the grid is one crossing, not 366; a column is a typed-data view over the blob, not a copy; and the provenance's settings hash is the cache key |
| [`rectification.dart`](rectification.dart) | a birth time known only to the hour, narrowed by lagna | `foundMany` founds a hundred candidate charts in **one crossing**, sharing the settings, the solar model and the day's sunrise; a chart is a view over the batch, not a copy; and `found(one)` is the same crossing unwrapped |
| [`almanac.dart`](almanac.dart) | a week's panchangam: the five limbs of each day, and its periods | a limb is a **span**, not a name — most days have two tithis, and the SDK gives both with the instant each gives way; a span carries its own bounds as well as the clipped ones; and a value a day may not have is `null`, never a sentinel |
| [`chart_reading.dart`](chart_reading.dart) | the same birth record read in full: divisional charts, houses, states, drishti and derived points | every section is **asked for** and off by default, and all of them come from one founded chart in one call; vargottama is a comparison of two signs, not a flag; a dignity and a house are different sections answering different questions; and the drishti are ragged, because relations depend on where the grahas stand. Its output is the other three bindings' `chart_reading`, line for line |
| [`annual_chart.dart`](annual_chart.dart) | the years a 1990 birth opens, and the chart of its thirtieth | the **Varsha Pravesha**: the Sun's return to where it stood at birth, which everything in Tajika is read from. The boundary answers the **instant** and not the chart, because whether the annual chart is cast for the birthplace or for a residence is a question the schools answer differently; the reading is a name a consumer asks for (`sidereal`, `tropical`, `mean`) and never a fallback, and the three are ten hours apart by the thirtieth year; fewer years than asked for is the answer when the ephemeris ends first |
| [`interpretation.dart`](interpretation.dart) | the same birth record said in English and in Nepali | a **composer** turns what was read into a narrative plan — message keys and their slots, no words — and the locale engine says it, so one plan says one chart in every locale. The plan comes back in the same crossing as the chart, an item's `params` are `intl.render`'s own params with no conversion between them, a reading needs the rules whose answers it says, and what a composer cannot say it leaves out rather than guessing |
| [`your_own_ephemeris.dart`](your_own_ephemeris.dart) | putting your own engine behind the SDK | the provider contract in full — one call per grid, refusing a frame so the SDK completes it, coverage checked before you are asked, and an exception that reaches the caller |

## What these examples do not do

They all name `Ephemeris.builtin`, the analytic ephemeris the SDK carries,
so that they run anywhere with nothing to install. It is the fallback and
not the intended path: in most cases a consumer belongs on a real engine —
Teimeris, Swiss Ephemeris — installed as its own package under its own
licence and named the same way. Do that, or hand in an ephemeris of your own
as [`your_own_ephemeris.dart`](your_own_ephemeris.dart) shows, and every one
of these programs is unchanged, which is what the port is for.

They also stop where the C boundary does, and it is further out than it
was: houses, divisional charts, planetary states, aspects and derived
points all cross, and [`chart_reading.dart`](chart_reading.dart) asks
for every one of them. Dashas do not, because nothing computes one yet
(Phase 5), so a binding cannot ask for them.
