# Examples

Every file here is a runnable program that `cargo xtask check-java`
compiles at release 22 with every lint an error and runs, and
`cargo xtask check-parity` compares line for line with the other
bindings' programs of the same name (`BirthChart` is `birth_chart`).
They are meant to be read in order: each one assumes the one before it.
They use only what the module exports, as a consumer's code would.

```sh
cargo build --release -p teistro-ffi
cargo xtask check-java    # compiles the module and these into target/java-examples
cd bindings/java
TEISTRO_LIBRARY=../../target/release/libteistro_ffi.dylib \
java -Dstdout.encoding=UTF-8 --enable-native-access=com.teispace.teistro \
  --module-path ../../target/java-examples/main --add-modules com.teispace.teistro \
  -cp ../../target/java-examples/example BirthChart
```

`-Dstdout.encoding=UTF-8` because the examples print Devanagari and
degree signs, and `System.out` writes the console's encoding: on Windows
that is a code page with neither, and every one of them prints as `?`.

| file | the scenario | what it is really teaching |
|---|---|---|
| [`Quickstart.java`](Quickstart.java) | the smallest thing that works | opening the library, a context, one call of each kind |
| [`BirthChart.java`](BirthChart.java) | a Nepali birth record to the nine grahas placed by sign, nakshatra and pada | the canonical frame is **tropical**; a Vedic chart asks for a sidereal one and the SDK completes it. Also: a zone's history matters, and Rahu is a body the chart calls a graha |
| [`Panchanga.java`](Panchanga.java) | the five limbs of a day — tithi, vara, nakshatra, yoga, karana | every limb but the weekday is a function of two longitudes, so a binding can compute a whole almanac from `positions`. The karana is not a cycle of eleven, and the rule is the SDK's own |
| [`Calendars.java`](Calendars.java) | a Bikram Sambat year and one month as a calendar page | month lengths are decided by the Sun and vary year to year, so they are asked for and never assumed; and every date says whether it came from the official table or the SDK's engine |
| [`Ephemeris.java`](Ephemeris.java) | a year of the sky in one call | the grid is one crossing, not 366; a column is read in place from the blob, never copied; and the provenance's settings hash is the cache key |
| [`Rectification.java`](Rectification.java) | a birth time known only to the hour, narrowed by lagna | `foundMany` founds a hundred candidate charts in **one crossing**, sharing the settings, the solar model and the day's sunrise; a chart is a view over the batch, not a copy; and `found` is the same crossing unwrapped |
| [`Almanac.java`](Almanac.java) | a week's panchangam: the five limbs of each day, and its periods | a limb is a **span**, not a name — most days have two tithis, and the SDK gives both with the instant each gives way; a span carries its own bounds as well as the clipped ones; and a value a day may not have is an empty `Optional`, never a sentinel |
| [`ChartReading.java`](ChartReading.java) | the same birth record read in full: divisional charts, houses, states, drishti and derived points | every section is **asked for** and off by default, and all of them come from one founded chart in one call; vargottama is a comparison of two signs, not a flag; a dignity and a house are different sections answering different questions; and the drishti are ragged, because relations depend on where the grahas stand. Its output is the other bindings' `chart_reading`, line for line |
| [`AnnualChart.java`](AnnualChart.java) | the years a 1990 birth opens, and the chart of its thirtieth | the **Varsha Pravesha**: the Sun's return to where it stood at birth, which everything in Tajika is read from. The SDK answers the **instant**, and casts the year's own chart — its office-bearers, year lord, yogas by matter, sahams, Harsha bala and annual dashas — only where the request names a place, because whether it is cast for the birthplace or for a residence is a question the schools answer differently; the reading is a name a consumer asks for (`SIDEREAL`, `TROPICAL`, `MEAN`) and never a fallback, and the three are ten hours apart by the thirtieth year; fewer years than asked for is the answer when the ephemeris ends first |
| [`Interpretation.java`](Interpretation.java) | the same birth record said in English and in Nepali | a **composer** turns what was read into a narrative plan — message keys and their slots, no words — and the locale engine says it, so one plan says one chart in every locale. The plan comes back in the same crossing as the chart, an item's `params` are `intl.render`'s own params with no conversion between them, a reading needs the rules whose answers it says, and what a composer cannot say it leaves out rather than guessing |
| [`Readings.java`](Readings.java) | the same birth record's yogas and doshas said in each language's own words | the reading corpus is **loaded, not embedded**: a pack is bytes, read here from the files `teistro-intl build` writes (`cargo xtask check-parity` builds them into `target/packs`, or name another directory with `TEISTRO_PACKS`), and `ctx.intl().loadPack` is the one call whatever the bytes came from. Loading changes what the `readings` composer says and not how it is called, and the record holds the whole passage and its named facets beside the sentence the plan carries |
| [`Phala.java`](Phala.java) | what the chart *is*, read aloud: a graha in a bhava, the lagna's sign, each limb of the day | two corpora, one engine: a pack **merges** into a record rather than replacing it, so `nakshatra.ASHWINI` keeps its name and gains the two corpora's forms, read through `forms`; the `phala` composer says only what a loaded pack has words for, and asking for it founds the chart with what it reads, the panchanga's limbs among them |
| [`YourOwnEphemeris.java`](YourOwnEphemeris.java) | putting your own engine behind the SDK | the provider contract in full — one call per grid, refusing a frame so the SDK completes it, coverage checked before you are asked, and an exception that reaches the caller as itself, or as a `ProviderException`'s cause when it is checked |

## What these examples do not do

They all name `Ephemeris.BUILTIN`, the analytic ephemeris the SDK carries,
so that they run anywhere with nothing to install. It is the fallback and
not the intended path: in most cases a consumer belongs on a real engine —
Teimeris, Swiss Ephemeris — installed as its own package under its own
licence and named the same way. Do that, or hand in an ephemeris of your
own as [`YourOwnEphemeris.java`](YourOwnEphemeris.java) shows, and every
one of these programs is unchanged, which is what the port is for.
