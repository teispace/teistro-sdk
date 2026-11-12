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
carries the source `{name: "teigeodata", version: "0.1.0", dataVersion,
tzdb}`. `dataVersion` names each pack the engine holds as
`address.component.release`, highest precedence first, joined by ` + `
(`np.places.2026.11.0`). `tzdb` is the release the packs' zone rules state
(`2026f`), or `null` without rules.

The crate stands outside the SDK workspace (ADR-0019), with its own
`Cargo.lock`, and depends on teigeodata by git at tag `v0.1.0`. teigeodata
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

## What v0.1.0 does not give

- **Zone rules:** `time_zones()` stays `None`. v0.1.0's engine answers
  `tz_offset_at` and `tz_local_candidates` for a named zone, but it lists
  no zones and does not say whether an instant falls before a zone's first
  rule. The SDK's `TimeZoneProvider` needs both (`zones`, `before_rules`,
  `current_offsets`).
- **Heights from the baseline:** v0.1.0 reads a pack's full `DEM` section
  only. The coarse terrain an embedded baseline carries (`DEM_COARSE`)
  arrived after the tag, so a baseline alone answers no height. A country
  pack with its DEM answers one.

## Tests and the gate

```sh
TEISTRO_TEIGEODATA_PACKS=/path/to/np-places-lite.tgd \
  cargo test --manifest-path adapters/geo-teigeodata/rust/Cargo.toml
```

`TEISTRO_TEIGEODATA_PACKS` names pack files, separated as a `PATH` is.
The tests need Nepal's country pack: `np-places-lite.tgd`, which
teigeodata's forge writes to `data/build/pack/np/` with
`cargo run --release --manifest-path forge/Cargo.toml -- make np --release 2026.11.0 --lite`.
No pack is checked in. A real pack is about 80 MB, and its data has its
sources' terms. When the variable is unset, the tests that need a pack
print `SKIP` and pass, so the gate job sets it. With the pack, the tests
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
