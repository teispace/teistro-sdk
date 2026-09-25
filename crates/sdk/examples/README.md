# Examples

Every file here is a runnable program that `cargo xtask check-rust`
runs, so none of them can drift from what the crate does. They are meant
to be read in order: each one assumes the one before it.

They are also the **same programs** as the three bindings'
`example/` directories, and `cargo xtask check-parity` runs all four sets
and compares what they print, line for line. What a language may print
differently is asserted in the program rather than printed; the few
differences left are listed by name in `xtask/src/examples.rs`
(`EXCUSED`), each with the item that removes it.

```sh
cargo run --release -p teistro --example birth_chart
```

Release, not debug: the built-in ephemeris is a truncated VSOP87 and
ELP2000, and a debug build of it computes a year of the sky slowly
enough to notice.

| file | the scenario | what it is really teaching |
|---|---|---|
| [`quickstart.rs`](quickstart.rs) | the smallest thing that works | a context is a **value** — no handle to open, no blob to decode, no `dispose` — and one call of each kind |
| [`birth_chart.rs`](birth_chart.rs) | a Nepali birth record to the nine grahas placed by sign, nakshatra and pada | a chart is **founded**, not assembled from positions: the profile says which zodiac and which centre, so the Moon is seen from Kathmandu. Also: a zone's history matters, Rahu is a body the chart calls a graha, and an absent ayanamsha is `None` rather than nought |
| [`panchanga.rs`](panchanga.rs) | the five limbs of a day — tithi, vara, nakshatra, yoga, karana | every limb but the weekday is a function of two longitudes, so a consumer can compute a whole almanac from `positions`. The karana is not a cycle of eleven, and the rule is the SDK's own. Each limb is a **catalogue member**, so one that does not exist cannot be constructed |
| [`calendar.rs`](calendar.rs) | a Bikram Sambat year and one month as a calendar page | month lengths are decided by the Sun and vary year to year, so they are asked for and never assumed; and `CalendarResolution` is an **enum a `match` must cover**, where the other three bindings hand over a string |
| [`ephemeris.rs`](ephemeris.rs) | a year of the sky in one call | the grid is one crossing, not 366; a column is the astronomy crate's own `Vec<f64>` with no blob in between; and the answer is stamped with the provider that computed it — there is no `buildInfo` to ask for, because Cargo fixed the versions |
| [`rectification.rs`](rectification.rs) | a birth time known only to the hour, narrowed by lagna | `found_many` founds eighteen candidate charts in **one crossing**, sharing the settings, the solar model and the day's sunrise; and `found(one)` is the same crossing unwrapped, agreeing bit for bit |
| [`almanac.rs`](almanac.rs) | a week's panchangam: the five limbs of each day, and its periods | a limb is a **span**, not a name — most days have two tithis, and the SDK gives both with the instant each gives way; a span carries its own bounds as well as the clipped ones; and a value a day may not have is `Option`, which the compiler will not let you ignore |
| [`chart_reading.rs`](chart_reading.rs) | the same birth record read in full: divisional charts, houses, states, drishti and derived points | every section is **asked for** and off by default, and all of them come from one founded chart in one call; vargottama is a comparison of two signs, not a flag; a dignity and a house are different sections answering different questions; and the drishti are ragged, because relations depend on where the grahas stand. Its output is the other three bindings' `chart_reading`, line for line |
| [`interpretation.rs`](interpretation.rs) | the same birth record said in English and in Nepali | a composer returns a **plan**, not prose: an ordered list of message keys and slots that holds no words, so one plan renders in every locale and the consumer may reorder or drop items first. `sdk.chart().interpreted` founds the chart, answers its rules and composes the plans in one call. `Rendered` says which locale answered, so a fallback cannot pass unnoticed; and `readings` without rules is refused rather than answered empty |
| [`annual_chart.rs`](annual_chart.rs) | the years a 1990 birth opens, and the chart of its thirtieth | the **Varsha Pravesha**: the Sun's return to where it stood at birth, which everything in Tajika is read from. `sdk.chart().varsha` answers a `VarshaRequest` in **one call** — each year's instant and Muntha, and where it names a place, the year's own chart with its office-bearers, year lord, yogas by matter, sahams, Harsha bala and annual dashas — because whether that chart is cast for the birthplace or a residence is a question the schools answer differently, so `AnnualPlace` is a choice you write and never a default; the reading is a name (`SIDEREAL`, `TROPICAL`, `MEAN`) and never a fallback; and a refusal names the field every binding writes, `varsha.through` |
| [`readings.rs`](readings.rs) | the same birth record's yogas and doshas said in each language's own words | the reading corpus is **loaded, not embedded**: a pack is bytes, read here, as in every binding, from the files `teistro-intl build` writes (`cargo xtask check-parity` builds them into `target/packs`, or name another directory with `TEISTRO_PACKS`), and `sdk.intl().load_pack` is the one call whatever the bytes came from. Loading changes what the `readings` composer says and not how it is called, and the record holds the whole passage and its named facets beside the sentence the plan carries |
| [`phala.rs`](phala.rs) | what the chart *is*, read aloud: a graha in a bhava, the lagna's sign, each limb of the day | two corpora, one engine: a pack **merges** into a record rather than replacing it, so `nakshatra.ASHWINI` keeps its name and gains the two corpora's forms, read through `form`; the `phala` composer says only what a loaded pack has words for, and asking for it founds the chart with what it reads, the panchanga's limbs among them |
| [`your_own_ephemeris.rs`](your_own_ephemeris.rs) | putting your own engine behind the SDK | the port in full — one call per grid, a body you did not declare refused before you are asked, an instant outside your coverage **never asked for** and its cells marked, refusing a frame so the SDK completes it, a `ProviderError` whose sentence reaches the caller, and an `Arc` newtype for a provider you keep a handle on |

There is one further file, [`parity.rs`](parity.rs), and it is not an
example at all: it prints one scenario as `key<TAB>value` lines so that
`cargo xtask check-parity` can compare the four bindings value for value,
and no other gate runs it.

## Why they repeat themselves

Each file is a program a reader is invited to **copy**, so each carries
its own small helpers — a clock formatter, a sign lookup, an
entity-name-or-key fallback. A shared `support` module would make every
one of them un-copyable, which is the wrong trade here; the DRY rule
applies to what ships, and what ships is the crate.

## What these examples do not do

They all name `Ephemeris::Builtin`, the analytic ephemeris the SDK
carries, so that they run anywhere with nothing to install. It is the
fallback and not the intended path: in most cases a consumer belongs on
a real engine — Teimeris, Swiss Ephemeris — linked as its own crate
under its own licence (ADR-0029). Do that, or hand in an ephemeris of
your own as [`your_own_ephemeris.rs`](your_own_ephemeris.rs) shows, and
every one of these programs is unchanged, which is what the port is for.

A Rust consumer *can* also depend on the crates directly, which the other three bindings cannot, and
[`03-design/rust-consumer-surface.md`](../../../docs/03-design/rust-consumer-surface.md)
§6 says what the surface deliberately leaves out.
