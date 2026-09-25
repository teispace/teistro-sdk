# Examples

Every file here is a runnable program that `cargo xtask check-node`
runs, so none of them can drift from what the binding does. They are
meant to be read in order: each one assumes the one before it.

```sh
cargo build --release -p teistro-node
cd bindings/node
node example/birth_chart.mjs
```

| file | the scenario | what it is really teaching |
|---|---|---|
| [`quickstart.mjs`](quickstart.mjs) | the smallest thing that works | opening the library, a context, one call of each kind |
| [`birth_chart.mjs`](birth_chart.mjs) | a Nepali birth record to the nine grahas placed by sign, nakshatra and pada | the canonical frame is **tropical**; a Vedic chart asks for a sidereal one and the SDK completes it. Also: a zone's history matters, and Rahu is a body the chart calls a graha |
| [`panchanga.mjs`](panchanga.mjs) | the five limbs of a day — tithi, vara, nakshatra, yoga, karana | every limb but the weekday is a function of two longitudes, so a binding can compute a whole almanac from `positions`. The karana is not a cycle of eleven, and the rule is the SDK's own |
| [`calendar.mjs`](calendar.mjs) | a Bikram Sambat year and one month as a calendar page | month lengths are decided by the Sun and vary year to year, so they are asked for and never assumed; and every date says whether it came from the official table or the SDK's engine |
| [`ephemeris.mjs`](ephemeris.mjs) | a year of the sky in one call | the grid is one crossing, not 366; a column is a `Float64Array` over the blob, not an array of objects; and the provenance's settings hash is the cache key |
| [`rectification.mjs`](rectification.mjs) | a birth time known only to the hour, narrowed by lagna | `foundMany` founds a hundred candidate charts in **one crossing**, sharing the settings, the solar model and the day's sunrise; a chart is a view over the batch, not a copy; and `found(one)` is the same crossing unwrapped |
| [`almanac.mjs`](almanac.mjs) | a week's panchangam: the five limbs of each day, and its periods | a limb is a **span**, not a name — most days have two tithis, and the SDK gives both with the instant each gives way; a span carries its own bounds as well as the clipped ones; and a value a day may not have is `null`, never a sentinel |
| [`chart_reading.mjs`](chart_reading.mjs) | the same birth record read in full: divisional charts, houses, states, drishti and derived points | every section is **asked for** and off by default, and all of them come from one founded chart in one call; vargottama is a comparison of two signs, not a flag; a dignity and a house are different sections answering different questions; and the drishti are ragged, because relations depend on where the grahas stand. Its output is the other three bindings' `chart_reading`, line for line |
| [`annual_chart.mjs`](annual_chart.mjs) | the years a 1990 birth opens, and the chart of its thirtieth | the **Varsha Pravesha**: the Sun's return to where it stood at birth, which everything in Tajika is read from. The SDK answers the **instant**, and casts the year's own chart — its office-bearers, year lord, yogas by matter, sahams, Harsha bala and annual dashas — only where the request names a place, because whether it is cast for the birthplace or for a residence is a question the schools answer differently; the reading is a name a consumer asks for (`SIDEREAL`, `TROPICAL`, `MEAN`) and never a fallback, and the three are ten hours apart by the thirtieth year; fewer years than asked for is the answer when the ephemeris ends first |
| [`interpretation.mjs`](interpretation.mjs) | the same birth record said in English and in Nepali | a **composer** turns what was read into a narrative plan — message keys and their slots, no words — and the locale engine says it, so one plan says one chart in every locale. The plan comes back in the same crossing as the chart, an item's `params` are `intl.render`'s own params with no conversion between them, a reading needs the rules whose answers it says, and what a composer cannot say it leaves out rather than guessing |
| [`readings.mjs`](readings.mjs) | the same birth record's yogas and doshas said in each language's own words | the reading corpus is **loaded, not embedded**: a pack is bytes, read here from the files `teistro-intl build` writes (`cargo xtask check-parity` builds them into `target/packs`, or name another directory with `TEISTRO_PACKS`), and `ctx.intl.loadPack` is the one call whatever the bytes came from. Loading changes what the `readings` composer says and not how it is called, and the record holds the whole passage and its named facets beside the sentence the plan carries |
| [`phala.mjs`](phala.mjs) | what the chart *is*, read aloud: a graha in a bhava, the lagna's sign, each limb of the day | two corpora, one engine: a pack **merges** into a record rather than replacing it, so `nakshatra.ASHWINI` keeps its name and gains the two corpora's forms, read through `forms`; the `phala` composer says only what a loaded pack has words for, and asking for it founds the chart with what it reads, the panchanga's limbs among them |
| [`your_own_ephemeris.mjs`](your_own_ephemeris.mjs) | putting your own engine behind the SDK | the provider contract in full — one call per grid, refusing a frame so the SDK completes it, coverage checked before you are asked, and an exception that reaches the caller |

## What these examples do not do

They all name `ephemeris: 'BUILTIN'`, the analytic ephemeris the SDK
carries, so that they run anywhere with nothing to install. It is the
fallback and not the intended path: in most cases a consumer belongs on a
real engine — Teimeris, Swiss Ephemeris — installed as its own package under
its own licence and named the same way. Do that, or hand in an ephemeris of
your own as [`your_own_ephemeris.mjs`](your_own_ephemeris.mjs) shows, and
every one of these programs is unchanged, which is what the port is for.

They also stop where the C boundary does, and it is further out than it
was: houses, divisional charts, planetary states, aspects and derived
points all cross, and [`chart_reading.mjs`](chart_reading.mjs) asks for
every one of them, as it asks for the dashas; and the annual chart
crosses in the same call as the birth it is read from.
