# The teigeodata geo adapter

`teistro-geo-teigeodata` makes a [teigeodata](https://github.com/teispace/teigeodata)
engine a `GeoProvider` for the SDK's geo port (`docs/03-design/geo-port.md`,
SR-88). It answers three questions at a point from the teigeodata packs
(`.tgd` files) a consumer supplies:

| question | teigeodata | the port's answer |
|---|---|---|
| the zone | `Engine::tz_at`, the area's name | `Some("Asia/Kathmandu")` |
| the height | `Engine::elevation_at` | `GroundHeight { metres, source: "NASADEM", resolutionM: 30.9, datum: "EGM2008" }` |
| the admin units | `Engine::admin_at`, then `iso_code` and `name` on each | `[{level: 0, code: "NP", name: "Nepal"}, {level: 1, code: "NP-P3", ...}]` |

A point no pack covers answers `None` or an empty list. Every resolution
carries the source `{name: "teigeodata", version: "0.3.1", dataVersion,
tzdb}`. `dataVersion` names each pack the engine holds as
`address.component.release`, highest precedence first, joined by ` + `
(`np.places.2026.11.0`). `tzdb` is the release the packs' zone rules state
(`2026f`), or `null` without rules.

The crate stands outside the SDK workspace (ADR-0019), with its own
`Cargo.lock`, and depends on teigeodata by git at tag `v0.2.0`. teigeodata
is Apache-2.0, so the adapter is public and nothing it links changes its
licence. The packs carry their sources' own terms (`Layer::attribution`),
which an application shows wherever it shows the data.

## Plugging it in

The adapter downloads nothing. The consumer opens packs it already has:
files it shipped or installed through teigeodata's store, or bytes it
embedded. In `Cargo.toml`:

```toml
teistro = { git = "https://github.com/teispace/teistro-sdk", rev = "..." }
teistro-geo-teigeodata = { git = "https://github.com/teispace/teistro-sdk", rev = "..." }
```

Then build the context with it and resolve places through the SDK:

```rust
use std::sync::Arc;
use teistro::{Context, PlaceRequest};
use teistro_geo_teigeodata::TeigeodataGeo;

let geo = TeigeodataGeo::open(["/data/packs/np.places.2026.11.0.tgd"])?;
let sdk = Context::builder().geo(Arc::new(geo)).build()?;
let place = sdk.geo().resolve(&PlaceRequest { latitude_deg: 27.7172, longitude_deg: 85.324 })?;
// Send `place` as `chart.found`'s or `almanac.days`' `place`: the receipt
// keeps it, and a replay needs no provider.
```

A consumer that already runs a teigeodata engine, as Tithipatra does with
its embedded baseline and installed packs, hands that engine over:
`TeigeodataGeo::from_engine(engine)` (a snapshot from `Shared::load()`
works, since an `Engine` clone shares its layers). After the packs change,
it builds a context with a new provider. Packs held in memory open with
`TeigeodataGeo::from_bytes`.

Only Rust reaches the adapter. A binding (Node, Python, Dart, Java) loads a
geo provider as a table (`geo.load`). No plugin ABI for geo providers exists
yet, so a binding consumer resolves with this crate in its Rust core, or
writes the resolved place into the request itself.

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

The packs hold no tzdb links (`UTC`, `Etc/*`, `US/Eastern`): a link is an
unknown zone here, refused with the nearest names.

## Heights

Heights come from a pack's full `DEM` section, or from its coarse grid
(`DEM_COARSE`, feature `dem-coarse`) where a pack carries only that. The
lite baseline of the data release verify reads carries neither, so a
baseline alone still answers no height; Nepal's lite pack answers one.

## Moving with teigeodata

Packs built by v0.2.0 and later split names and outlines into parts, and a
v0.1.0 reader refuses them, so the adapter moves with each teigeodata
release: the tag in `Cargo.toml`, `ENGINE_VERSION` (a test holds it to the
locked engine) and verify's `TEIGEODATA_DATA` move together.

## Tests and the gate

```sh
TEISTRO_TEIGEODATA_PACKS=/path/to/np-places-lite.tgd:/path/to/baseline-lite.tgd \
  cargo test --manifest-path adapters/geo-teigeodata/rust/Cargo.toml
```

`TEISTRO_TEIGEODATA_PACKS` names pack files, separated as a `PATH` is.
The tests need Nepal's country pack, `np-places-lite.tgd`, and the zone
tests the world's baseline, `baseline-lite.tgd` (a country pack carries
only its own zones' rules; the baseline carries every zone's): download
them from teigeodata's data release, the one verify's `TEIGEODATA_DATA`
names (`gh release download data-2026.11.14 -R teispace/teigeodata -p
np-places-lite.tgd -p baseline-lite.tgd`), or build them with
teigeodata's forge (`make np --lite`). Name Nepal's first, so it answers
Nepal before the baseline; without the baseline the world-zone tests say
`SKIP`. No pack is checked in. A real pack is about 80 MB, and its data
has its sources' terms. When the variable is unset, the tests that need a
pack print `SKIP` and pass; verify's `geo-adapter` job downloads both,
checks them against the release's `SHA256SUMS`, and sets
`TEISTRO_TEIGEODATA_REQUIRED`, under which a missing pack, or packs
without the world's zones, fail instead.
The engine tag and the data release move together: a pack a newer format
writes is refused by an older reader. With the pack, the tests
check four things:

- Kathmandu resolves to `Asia/Kathmandu`, `NP` and `NP-P3`, and a height
  of 1,200 to 1,450 m above EGM2008;
- the middle of the Atlantic answers nothing, and that is not a refusal;
- every answer passes `teistro_port_geo::resolve`'s check;
- the SDK's own geo suite (`crates/sdk/tests/geo.rs`) holds over the
  adapter. A chart on the resolved place keeps the place in its receipt.
  The receipt replays `HELD` without the provider, matches the chart
  from the written-out coordinates, and its sunrise names the height's
  source.

`cargo xtask check-adapter` gates ephemeris adapters: it runs a provider
kit and loads a plugin library, and a geo adapter has neither. This
adapter's gate is `cargo fmt --check`, `cargo clippy --all-targets -- -D
warnings` and `cargo test` in this crate, with the variable set.
