# The teigeodata geo adapter

`teistro-geo-teigeodata` makes a [teigeodata](https://github.com/teispace/teigeodata)
engine a `GeoProvider` for the SDK's geo port (`docs/03-design/geo-port.md`,
SR-88, SR-141, SR-142, SR-129, SR-144). It answers from the teigeodata packs
(`.tgd` files) a consumer supplies, or from the engine teigeodata's store
fills:

| question | teigeodata | the port's answer |
|---|---|---|
| the zone | `Engine::tz_at`, the area's name; where no outline holds the point, the zone of the town it lies within | `Some("Asia/Kathmandu")`; at sea, where the world baseline outlines the land nearby, the nautical `Etc/GMT±n` marked `nautical` |
| the height | `Engine::elevation_at` | `GroundHeight { metres, source: "NASADEM", resolutionM: 30.9, datum: "EGM2008" }` |
| the admin units | `Engine::admin_at`, then `iso_code` and `name` on each | `[{level: 0, code: "NP", name: "Nepal"}, {level: 1, code: "NP-P3", ...}]` |
| a search | `Engine::search_with`, teigeodata's own ranking | `काठमाडौं`, `Kathmandu` and `Kathamandu` find the same places, best first, each labelled `Name, Parent, Country` |
| a point named | `reverse_in_country`, `mark_inside`, `admin_at` | the nearest places, each inside or not, and the units holding the point |
| a place's names | `Engine::names` | every name with its language, script, type and origin; the one chosen per language asked |
| a unit's places | `Engine::children`, `population` | ranked by population, each `COUNTED` or `MODELLED` |

A level the packs hold no outline for is absent, never a refusal: a pack
that keeps Nepal's outlines to the ward and none abroad (teigeodata's
TG-26 app pack) answers a point abroad with the world's levels 0 and 1, or
none, and a listing there with fewer places (C2209).

Every resolution carries the source `{name: "teigeodata", version:
"1.2.0", dataVersion, tzdb}`. `dataVersion` names each pack the engine
holds as `address.component.release`, highest precedence first, joined by
` + ` (`np.places.2026.11.0 + world.baseline.2026.11.0`), or `none` while
the store has installed nothing. `tzdb` is the release the packs' zone
rules state (`2026f`), or `null` without rules.

The crate stands outside the SDK workspace (ADR-0019), with its own
`Cargo.lock`, and depends on teigeodata by git at tag `v1.2.0`. teigeodata
is Apache-2.0, so the adapter is public and nothing it links changes its
licence. The packs carry their sources' own terms (`Layer::attribution`),
which an application shows wherever it shows the data.

## Plugging it in

The adapter downloads nothing. In `Cargo.toml`:

```toml
teistro = { git = "https://github.com/teispace/teistro-sdk", rev = "..." }
teistro-geo-teigeodata = { git = "https://github.com/teispace/teistro-sdk", rev = "..." }
```

**Over teigeodata's store.** A consumer that runs teigeodata's store
(`teigeodata-sync`) hands over the `Shared` engine the store fills and
refills. Packs added or replaced later answer at once, with no new
context:

```rust
use std::sync::Arc;
use teistro::{Context, PlaceSearch};
use teistro_geo_teigeodata::TeigeodataGeo;

let geo = TeigeodataGeo::from_shared(Arc::clone(&store_shared));
let sdk = Context::builder().geo(Arc::new(geo)).build()?;
let asked = PlaceSearch { query: "काठमाडौं".into(), focus: None, limit: 10, lang: Some("en".into()) };
let found = sdk.geo().search(&asked)?;
// later: store_shared.update(|engine| { engine.add(layer); });
```

An empty `Shared` is accepted: until a pack arrives every question answers
nothing and `dataVersion` is `none`. The zone rules (`time_zones()`) are
read when the context is built, from the packs installed then.

**From files or bytes.** `TeigeodataGeo::open(paths)` maps pack files
(feature `mmap`), `from_bytes` reads packs held in memory, and
`from_engine(engine)` takes an engine already built; each wraps it in its
own `Shared`, reachable through `geo.shared()`.

Then resolve places through the SDK and send the place in a request:

```rust
let place = sdk.geo().resolve(&PlaceRequest { latitude_deg: 27.7172, longitude_deg: 85.324 })?;
// Send `place` as `chart.found`'s or `almanac.days`' `place`: the receipt
// keeps it, and a replay needs no provider.
```

Only Rust reaches the adapter. A binding (Node, Python, Dart, Java) loads a
geo provider as a table (`geo.load`), whose optional `places` the
catalogue answers from. A binding consumer resolves with this crate in its
Rust core, or writes the resolved place into the request itself.

## Features and the web

| feature | gives |
|---|---|
| (always) | the zone, the admin units, reverse, names, listings, population |
| `mmap` | `open`: pack files mapped (native only) |
| `search` | `search` across scripts; without it a search is `CAPABILITY` on `search` |
| `terrain` | heights (`dem`, `dem-coarse`) |
| `zone-rules` | `time_zones()` |

The default is every feature. Lookup-only, for the web, is
`--no-default-features --features terrain,zone-rules`; it builds for
`wasm32-unknown-unknown` with packs from `from_bytes`, and so does
lookup with `search`. What each adds, measured with the adapter's `size`
profile (`examples/size.rs`; geo-port.md §13.3 has the method):

| target | features | added, as built | added, gzipped |
|---|---|---:|---:|
| aarch64-apple-darwin | lookup-only | 115,952 | 63,842 |
| aarch64-apple-darwin | lookup + `search` | 347,616 | 181,663 |
| aarch64-apple-darwin | default | 347,616 | 181,835 |
| wasm32-unknown-unknown | lookup-only | 157,001 | 45,893 |
| wasm32-unknown-unknown | lookup + `search` | 393,386 | 154,992 |

## Zone rules

`time_zones()` hands the context the packs' own zone rules
(`TeigeodataZones`, a `TimeZoneProvider`), so a place resolved through
teigeodata is reckoned by the tzdb release its packs carry (`tzdb` in the
source). It is `None` when no pack has rules. Each zone is answered by the
first pack, by precedence, whose rules hold it:

| port | engine |
|---|---|
| `zones`, `has_zone` | `tz_zones`: `zone.tab`'s zones, matched as spelled or without regard to case |
| `offset_at` | `tz_offset_at`, with `before_rules` from `tz_rules_start` (the first transition; earlier is local mean time) |
| `candidates` | `tz_local_candidates`: one offset, a gap or an overlap |
| `current_offsets` | `tz_offset_at` at noon UTC on 1 January, April, July and October of the release's year, as the SDK's own zone databases sample |

The packs hold no tzdb links (`UTC`, `US/Eastern`) and no `Etc` zones:
those are unknown here, refused with the nearest names, save the nautical
zones `Etc/GMT-12` to `Etc/GMT+12` a resolution at sea answers, which the
rules answer themselves as fixed offsets.

## Heights

Heights come from a pack's full `DEM` section, or from its coarse grid
(`DEM_COARSE`) where a pack carries only that. The lite baseline carries
neither, so a baseline alone answers no height; Nepal's packs answer one.

## Moving with teigeodata

A pack a newer format writes is refused by an older reader (the 1.0 pack
format of `data-2026.11.15` is refused by v0.3), so the tag in
`Cargo.toml`, `ENGINE_VERSION` (a test holds it to the locked engine) and
verify's `TEIGEODATA_DATA` move together.

## Tests and the gate

```sh
TEISTRO_TEIGEODATA_PACKS=/path/to/np-places-lite.tgd:/path/to/baseline-lite.tgd \
  cargo test --manifest-path adapters/geo-teigeodata/rust/Cargo.toml
```

`TEISTRO_TEIGEODATA_PACKS` names pack files, separated as a `PATH` is:
Nepal's country pack first, then the world's baseline; or teigeodata's app
pack alone (`world-np-app-lite.tgd` of `data-2026.11.17`, the world's
places and Nepal's outlines in one layer, with no DEM and no populations),
over which the height tests say `SKIP` and the listings order by name. Download them from
the data release verify's `TEIGEODATA_DATA` names (`gh release download
data-2026.11.15 -R teispace/teigeodata -p np-places-lite.tgd -p
baseline-lite.tgd`) and check them against its `SHA256SUMS`. No pack is
checked in: a real pack is about 85 MB, and its data has its sources'
terms. When the variable is unset, the tests that need a pack print
`SKIP` and pass; verify's `geo-adapter` job downloads both and sets
`TEISTRO_TEIGEODATA_REQUIRED`, under which a missing pack fails instead.
The tests open files, so they need `mmap`; a lookup-only build is checked
by building it for wasm32. With the packs, the tests check:

- Kathmandu resolves to `Asia/Kathmandu`, `NP` and `NP-P3`, and a height
  of 1,200 to 1,450 m above EGM2008;
- the middle of the Atlantic answers no unit and no height, and with the
  baseline the nautical zone `Etc/GMT+2`; a country pack alone answers no
  zone there;
- the three spellings of Kathmandu find the same places in the same
  order, a focus ranks a nearby namesake up, and `"Name, Parent"` ranks the
  parent's places first;
- a point in Kathmandu and one in London are named to their units; a pack
  with no outline for a point answers fewer units, never a refusal;
- a place's names by language, with the script fallback for `hi` and `sa`;
- Nepal's provinces, Bagmati's local levels and a district's units listed
  by population, each counted or modelled;
- packs added to the `Shared` after the context was built answer without
  a new context;
- the SDK's own geo suite (`crates/sdk/tests/geo.rs`) holds over the
  adapter.

This adapter's gate is `cargo fmt --check`, `cargo clippy --all-targets -- -D
warnings` and `cargo test` in this crate, with the variable set, and the
two wasm32 builds.
