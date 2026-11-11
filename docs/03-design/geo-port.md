# The geo port: a place's zone, ground height and admin codes

Status: `built`, 2026-11-11 (SR-87, addendum 7, P1). Designed the same day,
before the code, and corrected by it.

A consumer resolving a birth place today supplies four facts by hand: the
coordinates, the IANA zone, the height, and which country and province it is
in. The SDK has planned a port for the middle three since the module catalogue
(`02-architecture/01-module-catalog.md`, `port-geo` beside `port-calendar`;
`08-extensibility.md`, "geo data: provider trait, coordinates to zone, place
catalogue"). SR-87 asks for it:

- the IANA zone at a point;
- the ground height at a point, with its source, resolution and datum;
- the ISO 3166-1 and 3166-2 codes of the admin units holding a point;
- optionally, a `TimeZoneProvider` whose rules match the zone names the
  provider returns (the same tzdb release).

Two rules frame it. **It is never consulted implicitly**
(`time-and-timezone.md` §2 stays): a zone is given or it is local mean time,
and a caller who wants one from the provider asks for it. **A stored chart
replays without the provider:** what the provider answered, and the data it
answered from, travel with the request, so the receipt holds them.

## 1. What the ports already do, and what this one copies

| port | trait | errors | how a context gets one | what a replay keeps |
|---|---|---|---|---|
| `port-ephemeris` | `EphemerisProvider` with `capabilities()` naming it | `ProviderError`, because it crosses the C vtable | the builder's chain; a shared library through `ts_provider_load` | the provider's `{name, version, dataVersion}` in the provenance |
| `port-timezone` | `TimeZoneProvider` with `version()` | `teistro_core::Error` | the embedded tzdb; a signed pack at build or through `ts_time_load_zones` | the `ZoneResolution` on the chart, the tzdb version in the provenance |

The geo port is in-process like the zone port, so it takes the core `Error`
and needs no vtable. It copies the zone port's shape: a `Send + Sync` trait,
blanket implementations for `&P` and `Arc<P>` (so a borrowed or shared provider
is a provider, tested method by method), one small provider of its own for
tests and for data a consumer writes, and a context that holds one behind an
`Arc` and is given it at build or loads one later. It copies the ephemeris
port's identity: a provider names itself and its data's version, and every
answer is stamped with both.

## 2. The trait

```rust
pub trait GeoProvider: Send + Sync {
    /// Who answers, and from which data: stamped on every resolution.
    fn source(&self) -> &GeoSource;
    /// The IANA zone holding the point; `None` where the data has none.
    fn zone_at(&self, at: GeoPoint) -> Result<Option<String>, Error>;
    /// The ground height at the point; `None` where the data has none (the sea).
    fn height_at(&self, at: GeoPoint) -> Result<Option<GroundHeight>, Error>;
    /// The admin units holding the point, coarse to fine; empty where none.
    fn admin_at(&self, at: GeoPoint) -> Result<Vec<AdminUnit>, Error>;
    /// The zone rules of the release `zone_at`'s names come from, when the
    /// provider carries them.
    fn time_zones(&self) -> Option<Arc<dyn TimeZoneProvider>> { None }
}
```

**Why `Option` and not a refusal for "no answer here".** A point at sea has no
ground height and may have no admin unit; a point outside a regional dataset
has none of the three. That is an answer, and the caller decides what to do
with it (state an offset, take a height of zero). A refusal is for a provider
that failed: data missing that should be there, data corrupt.

**Why three methods and not one.** A provider may answer one question from a
different layer than another (a country pack's terrain under a world zone
map), and a caller may want only the zone. The SDK's resolve asks all three
(§5); a Rust consumer may ask the provider one at a time.

**Fit to a real engine.** The engine SR-88 adapts answers `tz_at`,
`elevation_at` and `admin_at` at a point on a 1e-7° grid, each `Option` or a
list, from the highest-precedence layer that covers the point; its heights are
metres above EGM2008 from a DEM with a known step; its admin units come coarse
to fine with a level; its packs carry a release (`2026.11.0`). The trait names
none of its types: an adapter converts a `GeoPoint` to its grid and its
answers to these types. `reverse` and `search` (a place's name from a point and
the reverse) are the place catalogue's half of `08-extensibility.md`; SR-87
does not ask for them and the trait leaves them out, so adding them later is
a new method with a default, not a change.

## 3. The types

All live in `crates/port-geo`, which depends on `core` and `port-timezone`
only. Every type a document carries derives `Serialize` and `Deserialize`
camel-cased, as every request record is, and `JsonSchema` under `schema`.

| type | fields | checked |
|---|---|---|
| `GeoPoint` | `latitude: Latitude`, `longitude: Longitude` (core's quantities) | by their constructors |
| `GeoSource` | `name`, `version` (the provider's code), `dataVersion` (its data's release), `tzdb` (the release `zone_at`'s names come from, or `null`) | `name` and `dataVersion` not empty |
| `GroundHeight` | `metres`, `source` (which dataset, as the provider names it: `SRTM GL1`, `the test table`), `resolutionM` (the data's horizontal step in metres), `datum` (the vertical datum: `EGM2008`, `EGM96`, `WGS84` for the ellipsoid) | `metres` an `Altitude`, `resolutionM` finite and positive, `source` and `datum` not empty |
| `AdminUnit` | `level` (0 a country, then coarse to fine), `code` (ISO 3166-1 alpha-2 at level 0, ISO 3166-2 below; `null` for a unit with none, a district), `name` (the provider's, or `null`) | the code's shape, and that a subdivision's code starts with its country's |
| `ResolvedPlace` | `latitudeDeg`, `longitudeDeg`, `zone`, `height`, `admin`, `source` | as its parts |

**The codes are the festival regions' keys.** A code is upper case, `NP` or
`NP-P3`, exactly as the festival rules' `meta.regions` spell a region (SR-89,
`festival-metadata.md` §2a), so a resolved place's `admin` codes select its
festival sets directly, a state falling back to its country.

**Datum as text, not a closed enum.** Heights come from many models and a
closed list would be a dead end for the first provider on another
(`no-dead-ends`); the shipped values are named in the documentation, and an
empty datum is refused.

**The height is the ground's.** A chart's `altitudeM` is the observer's
height; a resolved place takes the ground's, which is right for a birth on the
ground and what the request asked for. A caller who knows better (an upper
floor) states `altitudeM` and does not send the place's height (§6).

## 4. Errors

| case | status | field |
|---|---|---|
| a resolve on a context with no provider | `CAPABILITY` | `geo`, hint: load a table or build the context with a provider |
| a point out of range | `INVALID_ARG` | `latitudeDeg` or `longitudeDeg` |
| the provider failed | its own `Error`, passed through | as it named it |
| the provider answered a malformed value (an empty zone, a height outside `Altitude`, a code of the wrong shape) | `PROVIDER` | `zone`, `height.metres`, `admin[i].code` |
| a table that does not read (§7) | `INVALID_ARG` | the path in the table |

The SDK checks every answer, whichever provider gave it, at one place
(`GeoArea::resolve`), as it checks an ephemeris's.

## 5. Resolving a place

`sdk.geo().resolve(&PlaceRequest)`, the record `geo.resolve`
(`ts_geo_resolve`): `{latitudeDeg, longitudeDeg}` in, a `ResolvedPlace` out —
the point as asked, the three answers, and the provider's `source`. It seals
no provenance: it reads no sky and no settings, and its own `source` is the
stamp. It is a record, so the MCP tool and each binding's generated request
type follow.

A context has no provider unless it is given one:

- Rust: `ContextBuilder::geo(Arc<dyn GeoProvider>)`;
- every binding: `geo.load(table)` (`ts_geo_load`), which installs a
  `GeoTable` (§7) and answers its `source`;
- the MCP server: `--geo PATH`, a table every context reads.

A context's provider is part of its changeable state, like its zone
database: a fork copies it, a context derived for a receipt
(`Context::with_settings`) keeps it, and a later load replaces it.

## 6. A chart on a resolved place, and replay

`chart.found` gains `place`, a `ResolvedPlace` as `geo.resolve` answered it.
With `place`:

- the coordinates come from it, the height from `place.height.metres` (0 when
  it has none, as an omitted `altitudeM` is), and the clock from `place.zone`;
- `latitudeDeg`, `longitudeDeg` and `zone` beside it are refused as a
  contradiction, named by the field; `altitudeM` beside it is the caller's
  own height and wins (§6a);
- a place with no zone needs `utcOffsetSeconds`; one with a zone refuses it.

`latitudeDeg` and `longitudeDeg` become optional in the request's schema, and
one of them or `place` is required, named `latitudeDeg` when neither is sent.

The request is the record of what the provider answered: a receipt
(`verify.issue`, `receipts-and-datasets.md` §1.2) carries the request whole,
so it holds the zone, the height with its source, resolution and datum, the
admin codes, and `source.dataVersion`. `verify.receipt` re-runs the request,
which reads `place` and never the provider, so it answers `HELD` on a context
with no provider at all. The chart document itself carries the zone as its
foundation's `clock` and the height in its place, as it does for a stated
zone; the provenance's `tzdb` names the zone database that read the zone.

Rust builds the same request with `ChartRequest::resolved(&place)`.

**Nothing is added to the receipt or the provenance.** Both were considered.
The receipt already carries the request, and a second copy of the place would
be a second thing to keep equal; the provenance is stamped by every answer and
the geo port reaches one of them. A record that reads a resolved place carries
it in its request, and that is where a replay reads it.

## 6a. The height into the sunrise (SR-90)

`horizon-atmosphere.md` measured the observer's height as the term that
matters for a sunrise: about half a minute at Kathmandu, through the dip and
the pressure (`Atmosphere::at`, `rise_set::horizon_refraction_arcmin`,
`centre_altitude_deg`). Both already read the place's `altitude` under the
sunrise convention that names its air (`ATMOSPHERIC`, the pressure from the
height when it is left out), so a resolved place needs no new path to the
sunrise, only a height in its place:

- `chart.found` and `almanac.days` take `place`, and the observer stands at
  `altitudeM` when the caller states one, else at the provider's ground
  height, else at sea level where the provider knows none;
- the answer's provenance gains an applied convention, `observer.height`,
  whose value is the height used (`1337 m`) and whose reason names where it
  came from: the provider, its version and data release, and the height's
  own source, resolution and datum; or "stated by the caller as `altitudeM`,
  over the provider's ground height of 1337 m"; or that the provider knows
  no height there, so sea level;
- a request written out with coordinates records nothing new, so no stored
  answer moves.

The receipt holds both: the request keeps the place with its height, and the
answer it digests carries the convention, so a replay without the provider
answers `HELD` with the same record of where the height came from. A
Kathmandu sunrise on the test table's 1337 m is the sunrise with
`altitudeM: 1337` written by hand, to the bit (`crates/sdk/tests/geo.rs`).

**The convention is the settings', not the place's.** The shipped profiles
reckon the sunrise by the almanac's fixed 34 arcminutes (`NAMED`), which
reads no height, and a resolved place does not switch it: a resolved place
must found what its parts written out found under the same settings, which
is the acceptance, and turning the convention on for every profile would
move every stored sunrise. A consumer who wants the height in its sunrise
names the air once in its settings, `{"kind": "ATMOSPHERIC", "which":
"UPPER_LIMB_REFRACTION", "air": {}}`, and every resolved height then reaches
it: at Kathmandu the test table's 1337 m moves the sunrise from sea level's,
and `altitudeM: 0` beside the place moves it back. Whether a shipped profile
should name the air is the maintainer's call, left open here.

The typed C entry points (`ts_panchanga_days`, `ts_chart_found`) take
coordinates and not a place; a binding reaches a resolved place through the
records, `record("almanac.days", ...)` and `record("chart.found", ...)`.

## 7. The table provider

`GeoTable` is the provider the SDK ships: a list of boxes, each answering any
of the three questions for the points inside it, read from JSON:

```json
{
  "source": {"name": "test", "version": "1", "dataVersion": "2026.11.0", "tzdb": "2026c"},
  "areas": [
    {"south": 26.3, "west": 80.0, "north": 30.5, "east": 88.2,
     "zone": "Asia/Kathmandu", "admin": [{"level": 0, "code": "NP", "name": "Nepal"}]},
    {"south": 27.6, "west": 85.2, "north": 27.8, "east": 85.5,
     "height": {"metres": 1337, "source": "survey", "resolutionM": 30, "datum": "EGM2008"},
     "admin": [{"level": 0, "code": "NP"}, {"level": 1, "code": "NP-P3", "name": "Bagmati"}]}
  ]
}
```

Each question is answered by the **last** box holding the point that answers
it, so a table is written coarse to fine and a finer box overrides a coarser
one for what it states and inherits the rest. A box whose `west` is east of its
`east` crosses the antimeridian. It is the test provider for every question
and every binding, and a small place catalogue a consumer can write: an app
that knows its few hundred places needs no dataset.

## 8. The optional zone rules

`time_zones()` lets a provider hand over the tzdb release its zone names come
from. The SDK does not install it on its own (the opening's first rule): a Rust
consumer who wants a chart read under the provider's own rules gives the same
data to the context as its zone database. The resolve answers
`source.tzdb` beside the zone, and the chart's provenance names the zone
database that read it, so a mismatch is visible in the stored answer. Wiring a
provider's rules into a context as its zone database is SR-88's, when there is
a provider that carries some.

## 9. The boundary, the bindings and MCP

| layer | what it gets |
|---|---|
| C | `ts_geo_resolve(context, request_json, out_json)` and `ts_geo_load(context, table_json, out_json)`, both canonical JSON, in `crates/ffi/src/geo.rs` |
| IDL | both functions, the request type `PlaceRequest`, generated into every binding |
| Node, Python, Dart, Java | `geo.load(table)` and `geo.resolve({latitudeDeg, longitudeDeg})`, a typed `ResolvedPlace`; `chart.found`'s generated request type gains `place` |
| MCP | the `geo.resolve` tool, from the record; `--geo PATH` loads a table |

Each binding's test loads the test table, asks each question, is refused
without a provider, issues a chart on the resolved place and checks the
receipt's request carries the zone, the height and the data version, and
replays it on a context with no provider: `HELD`.

## 10. Left for later

- **SR-88**, the adapter over the real engine (`adapters/geo-...`, after its
  first tag): `GeoProvider` for its engine, the grid conversion, its pack's
  release as `dataVersion`, and its zone rules through `time_zones()` if it
  carries them, with the context side of §8.
- **SR-90** is built (§6a).
- A conformance kit for geo providers like `ephemeris-kit`, when there is a
  second provider to hold to the first.
