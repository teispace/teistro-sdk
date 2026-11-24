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
    fn source(&self) -> GeoSource;
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
did not ask for them and the trait left them out. SR-141 and SR-129 add them
as methods with defaults that answer nothing (§12), so a provider written
before them still compiles and answers as it did.

**The source is owned (SR-142).** `source()` answers a `GeoSource` by value,
not by reference: a provider over teigeodata's store changes its packs while
a context holds it (§13), so the release it names changes too and cannot be
lent from a field. A provider with a fixed source clones it.

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

**At sea (SR-144).** Where the provider knows the whole world and no area
of it holds the point (`GeoProvider::at_sea`, `false` by default), `resolve`
answers the nautical zone in place of no zone: `n = round(longitude / 15)`,
written as the tz database writes it, its sign inverted (`Etc/GMT-6` at
88° E, `Etc/GMT+2` at 30° W, `Etc/GMT` at `n = 0`), and sets
`nautical: true` (`nautical_zone`). A land zone is never replaced, and
`nautical` is written only when true, so every land answer, stored or not,
reads as before. A provider that knows part of the world (a table, a
country pack alone) cannot tell the sea from land it lacks and answers no
zone, as before (C2216). `checked` refuses `nautical` on any zone but the
longitude's nautical one.

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

**Which conventions read the height (SR-108).** Measured at Kathmandu, a
day's sunrise, sunset, moonrise and moonset are the same to the bit at
0 m and at 8,848 m under the disc's centre with the almanac's fixed
refraction (`CENTRE_REFRACTION`, the Nepal profiles' convention) and under
every other named sunrise: a fixed 34′ or no refraction reads no height,
and the ideal horizon has no dip (`crates/sdk/tests/geo.rs`). The height is
read only by:

| What | Knob | How |
| --- | --- | --- |
| the horizon's dip | `day.horizon.dip` `GEOMETRIC` or `REFRACTED` | the visible horizon lies `acos(R / (R + h))` below the ideal one |
| the air's pressure | `day.sunrise` `ATMOSPHERIC` with `pressure_hpa` left out | the standard atmosphere's pressure at the height |
| the topocentric places | `frame.centre` or `panchanga.centre` `TOPOCENTRIC` | the station stands at the height |

So the `observer.height` convention says whether the settings read it: its
value is the height used (`1337 m`), or `1337 m, not read` with a reason
saying that nothing under these settings reads it; when something does,
the reason names each reader by its knob. A receipt never implies that a
height moved a value it did not move.

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
rules to the context as its zone database,
`Context::builder().geo(geo.clone()).time_zones(zones)`, and `sdk.time`
reports the database as source `PROVIDER` with the provider's version. The
builder takes any `TimeZoneProvider`, not only a geo provider's, and it and
`zones(pack)` each replace what the other set. The resolve answers
`source.tzdb` beside the zone, and the chart's provenance names the zone
database that read it, so a mismatch is visible in the stored answer. The
bindings' contexts keep their embedded or loaded pack: a provider is a Rust
object, and a binding hands its zone data over as a pack.

## 9. The boundary, the bindings and MCP

| layer | what it gets |
|---|---|
| C | `ts_geo_resolve(context, request_json, out_json)` and `ts_geo_load(context, table_json, out_json)`, both canonical JSON, in `crates/ffi/src/geo.rs`; the catalogue's `ts_geo_search`, `ts_geo_reverse`, `ts_geo_names`, `ts_geo_place` and `ts_geo_list` (§12), the same shape |
| IDL | the functions, the request types `PlaceRequest`, `PlaceSearch`, `PlaceReverse`, `PlaceNamesRequest`, `PlaceLookup` and `PlaceListing`, generated into every binding |
| Node, Python, Dart, Java | `geo.load(table)` and `geo.resolve({latitudeDeg, longitudeDeg})`, a typed `ResolvedPlace`; `chart.found`'s generated request type gains `place`; `geo.search`, `geo.reverse`, `geo.names`, `geo.place` and `geo.list`, each answering its decoded JSON (typed in Node's declarations) |
| MCP | the `geo.resolve`, `geo.search`, `geo.reverse`, `geo.names`, `geo.place` and `geo.list` tools, from the records; `--geo PATH` loads a table |

Each binding's test loads the test table, asks each question, is refused
without a provider, issues a chart on the resolved place and checks the
receipt's request carries the zone, the height and the data version, and
replays it on a context with no provider: `HELD`.

## 10. The teigeodata adapter (SR-88)

`adapters/geo-teigeodata/rust` (`teistro-geo-teigeodata`) is the first
provider over a real engine: teigeodata at its tag `v1.2.0` (§13.1; it was
`v0.3.1` until SR-142 and `v1.1.0` until SR-164), by git, in a
standalone crate with its own lock, outside the workspace like the
ephemeris adapters (ADR-0019). It is public, since teigeodata is
Apache-2.0. `TeigeodataGeo` opens pack files (`open`), packs in memory
(`from_bytes`) or an engine the consumer already runs (`from_engine`), and
asks the engine as a whole, so teigeodata's own precedence picks the layer
that answers:

| port | engine |
|---|---|
| `zone_at` | `tz_at`, then the area's canonical name, which is its tzdb name (as `tgd` reads it); where no area outline holds the point, the zone of the settlement or locality it lies within (`mark_inside`, `tz_of`; C2218) |
| `height_at` | `elevation_at`: metres above EGM2008, the DEM product's name as `source` (`NASADEM`), its north-south spacing as `resolutionM` |
| `admin_at` | `admin_at`, coarse to fine, each unit's `admin_level`, `iso_code` (`NP`, `NP-P3`) and canonical name |
| `source` | `teigeodata`, `1.2.0`, every pack as `address.component.release` joined by ` + ` (`np.places.2026.11.0 + world.baseline.2026.11.0`), and `tz_version()` as `tzdb`; `none` while the store has installed nothing (§13.2) |
| `at_sea` | a `world` layer installed, neither `tz_at` nor `admin_at` holding the point, and the nearest settlement, locality or unit lying inside a unit the packs hold (§5, C2216) |

A point no pack covers answers `None` or an empty list. An engine error
crosses as `PROVIDER` on the question's field (`zone`, `height`, `admin`),
with a hint for a component that was not installed. A pack that does not
open is `PACK` on `packs[i]`. An engine with no pack is `INVALID_ARG` on
`packs`.

**How a consumer plugs it in.** A Rust consumer such as Tithipatra opens
the packs it ships or installed, or hands over the engine it already
runs, and builds its context with the provider:
`Context::builder().geo(Arc::new(TeigeodataGeo::from_engine(engine)?))`.
It then resolves with `sdk.geo().resolve` and sends the place in its
requests (§6). The adapter fetches nothing. A binding reaches a provider
only as a table (`geo.load`), so a binding consumer resolves in its Rust
core or writes the place into the request itself.

**Zone rules.** `time_zones()` answers `TeigeodataZones`, the packs' own
rules as a `TimeZoneProvider` (teigeodata v0.3.0's `tz_zones` and
`tz_rules_start`, asked for on 2026-11-13 and added the same day, with
`tz_offset_at` and `tz_local_candidates`), so a place teigeodata resolves
is reckoned by the tzdb release its packs carry; `None` without rules.
`before_rules` is an instant before the zone's first transition, as the
SDK's own databases say it, and `current_offsets` samples the release's
year as they do. The packs hold `zone.tab`'s zones and no links (`UTC`,
`US/Eastern`) and no `Etc` zones: those are unknown, save the nautical
zones `Etc/GMT-12` to `Etc/GMT+12` a resolution at sea answers (§5), which
`TeigeodataZones` answers itself, each a fixed offset with tzdb's
abbreviation (`+06`, `GMT`), so a chart on a place at sea reckons through
the packs' own rules too (C2217). Heights come from the full `DEM` section or, where a pack carries only
that, the coarse grid (`dem-coarse`). The lite baseline of
release verify reads carries neither, so a baseline alone answers no height.

**Moving with teigeodata.** A pack a newer format writes is refused by an
older reader (from v0.2.0 the packs split names and outlines into parts), so the
engine tag in the adapter's `Cargo.toml` and the data release verify reads
(`TEIGEODATA_DATA`) move together, with each teigeodata release.
`ENGINE_VERSION` is held to the locked engine by a test.

**Tests.** No pack is checked in: Nepal's country pack is about 80 MB, and
its data has its sources' terms. The adapter's tests read the packs
`TEISTRO_TEIGEODATA_PACKS` names and print `SKIP` when the variable is
unset. Verify's `geo-adapter` job downloads `np-places-lite.tgd` and
`baseline-lite.tgd` (a country pack carries only its own zones' rules;
the baseline carries every zone's) from the pinned data release, checks
them against `SHA256SUMS`, and sets
`TEISTRO_TEIGEODATA_REQUIRED`, under which a missing pack fails instead of
skipping. With the pack, Kathmandu resolves to
`Asia/Kathmandu`, `NP` and `NP-P3`, and a valley-floor height above
EGM2008. The Atlantic answers no unit and no height, and, with the
baseline installed, the nautical zone `Etc/GMT+2` (§5). The SDK's own geo suite holds over
the adapter: a receipt keeps the place and replays `HELD` without the
provider, the place founds the chart its written-out parts found, and
the sunrise names the height's source (`adapters/geo-teigeodata/README.md`).

## 12. The place catalogue: search, reverse, names and listings (SR-141, SR-129, SR-143)

Status: `built`, 2026-11-17. Designed before the code, from a probe of
teigeodata v1.1.0 over its `data-2026.11.15` packs (Nepal's main pack and
the world baseline).

Tithipatra's apps need the other direction from §2's three questions: a
city found by name in any script (the weather, the clock and panchang city
pickers), a tapped point named (to the ward in Nepal, the city elsewhere),
each place's names in Nepali, English, Hindi and Sanskrit, and a unit's
places listed for a site's city pages. The core never calls teigeodata
itself (the layering is teigeodata, then this port, then the core, then the
apps), so the port carries them.

### 12.1 The methods

Four methods join the trait, each with a default that answers nothing, so
`GeoTable` without places, and any provider written before them, answers an
empty list:

```rust
fn search(&self, asked: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> { Ok(Vec::new()) }
fn reverse(&self, asked: &PlaceReverse) -> Result<NearbyPlaces, Error> { Ok(NearbyPlaces::default()) }
fn names(&self, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> { Ok(PlaceNames::none(asked)) }
fn place(&self, asked: &PlaceLookup) -> Result<Option<NamedPlace>, Error> { Ok(None) }
fn list(&self, asked: &PlaceListing) -> Result<Vec<NamedPlace>, Error> { Ok(Vec::new()) }
```

The requests are records of their own, read strictly as every request is,
so the SDK's records, the C boundary, the bindings and the MCP tools take
them as JSON unchanged:

| request | fields |
|---|---|
| `PlaceSearch` | `query` (the text as typed; `"Name, Parent"` names a parent), `focus` (`{latitudeDeg, longitudeDeg, scaleM}` or `null`; `scaleM` defaults to 50 km), `limit` (1 to 100, default 10), `lang` (or `null`) |
| `PlaceReverse` | `latitudeDeg`, `longitudeDeg`, `limit` (1 to 100, default 5), `lang` |
| `PlaceNamesRequest` | `id`, `langs` (the languages to pick a name in; empty for none) |
| `PlaceLookup` | `id` (as an answer gave it), `lang` (or `null`): a saved place read back (§12.9) |
| `PlaceListing` | `within` (a place's id or an ISO 3166 code: `NP`, `NP-P3`), `level` (an admin level, or `null`), `kinds` (empty for every kind), `depth` (how far below `within`: 1 its children, `null` any depth; default 1), `limit` (1 to 1,000, default 100), `lang` |

The SDK checks what a provider cannot be trusted to: a limit out of range
and an empty query or id are `INVALID_ARG` named by the field before any
provider is asked; a `lang` is a BCP 47 tag (`ne`, `ne-Deva`, `ne-Deva-NP`)
and the provider reads its language and script subtags.

### 12.2 The answers

A **`NamedPlace`** is one place of the catalogue:

| field | what |
|---|---|
| `id` | the provider's stable id (`tgd:np/326`); a listing's `within` and a names request take it |
| `name` | its name in `lang` (§12.4), else its canonical name |
| `nameLang` | the language tag of `name` as the data states it (`ne`), `null` where undetermined |
| `label` | `"Name, Parent, Country"` (§12.3), for a picker |
| `kind` | `COUNTRY`, `ADMIN`, `SETTLEMENT`, `LOCALITY`, `NATURAL` or `POSTAL_AREA` |
| `level` | its admin level (0 a country, then coarse to fine) for a country or unit, `null` for any other place |
| `code` | its ISO 3166 code, `null` where none |
| `latitudeDeg`, `longitudeDeg` | its point |
| `population` | `{people, kind}`, `kind` `COUNTED` (a source stated it) or `MODELLED` (a population grid summed over it), `null` where the data has none (§12.6) |
| `parents` | its admin units coarse to fine, each `{id, name, level, code}`, with `name` in `lang` |
| `line` | the id line the id belongs to where a provider numbers more than one (teigeodata's `odbl` main and `cc-by-4.0` lite, its D-368), `null` otherwise (§12.9) |
| `zone` | its IANA zone (`Asia/Kathmandu`), from its own timezone area, `null` where the data gives none: a clock needs no second question |
| `class` | what a landmark is (`temple`, `palace`, `airport`), the provider's word, `null` for any other place |

`search` answers `FoundPlace {place, score}`, best first; `score` is the
provider's and compares within one query only. `reverse` answers
`NearbyPlaces {near, admin}`: `near` the nearest named places, each
`{place, distanceM, bearingDeg, inside}` (the bearing from the place to the
point, `inside` whether the point lies within it where the provider can
say, else `null`), and `admin` the units holding the point coarse to fine,
as `NamedPlace`s, so the name of "my location" is the finest unit or the
nearest settlement inside which it falls. `names` answers `PlaceNames {id,
best, all}`: `all` every name the place has, each `{text, lang, script,
type, origin, machine}`, and `best` one per asked language that has one
(§12.4). `list` answers `NamedPlace`s ranked by population (§12.6).

Timezone areas, which teigeodata stores as places, are not places of the
catalogue: they are left out of every answer, and a place's own zone is
its `zone` (a point's is `zone_at`'s).

### 12.3 Ranking and the label

**Search ranks as teigeodata ranks** (its D-052, D-119, D-226): one
additive score per hit, exact 60, loose 55, prefix 30, one edit 25,
substring 20, two edits 15, phonetic 5, each plus the place's importance
(0–255), plus a focus bonus of up to 30 halving every `scaleM` from the
focus; after a comma the query names a parent, and a place whose ancestor
matches it gains 1,000, so `Kathmandu, Bagmati` puts Bagmati's Kathmandus
first. The adapter passes the query whole and keeps the engine's order and
score; the port adds no rank of its own. The default focus is none: a
caller with a location sends it.

**Across scripts.** teigeodata folds a query and every name the same way
and indexes each name's loose and phonetic keys besides, and Nepal's packs
carry each place's Devanagari and Latin names, so `काठमाडौं`, `Kathmandu` and
`Kathamandu` (one edit) find the same three Kathmandus first, in the same
order (§12.7). The port does no transliteration of its own: a script the
pack holds no name in is found only through the engine's phonetic key.

**The label** is the name, then the finest parent whose name differs from
it, then the country when it is not already said: `Kathmandu, Bagmati,
Nepal` for the metropolitan city, `Pokhara, Kaski, Nepal`, `London,
England, United Kingdom`. It is in `lang` as `name` is. A place with no
parent is its name alone.

### 12.4 Names by language and script

A language is asked as a BCP 47 tag; the provider reads its language
(`ne`) and, when given, its script (`ne-Deva`). A place's name in a
language is chosen from two groups, the first that has one:

1. a name the data tags with the language (and the script, when asked), or
   a name in the language's script whose language the data leaves
   undetermined (Nepal's packs tag most Devanagari names with the script
   only);
2. else a name in the language's script tagged with another language:
   `काठमाडौं` for `hi` and `sa` where only a Nepali name is written;
3. else the canonical name, and `nameLang` says what it is.

Within a group: a **reviewed** name first (teigeodata's origin
`reviewed`, a person's correction against a cited source, D-359), then a
name a person wrote before one a program wrote, then by type (official,
local, untyped, an alternative or colloquial one, a historical one), then
a tagged name before an undetermined one, then the data's order.

The script of a language with none asked is its usual one: Devanagari for
`ne`, `hi`, `sa`, `mr`, `mai`, `bho` and `new`, Latin for `en`; any other
language takes step 1 alone. Step 2 is a choice (C2206): a Devanagari name
tagged with no language is offered as Hindi or Sanskrit rather than falling
back to Latin, and `nameLang` says it was not tagged so.

### 12.5 Reverse: the nearest places and the admin chain

`reverse` asks the engine for the `limit` nearest places (timezone areas
left out, so it asks for a few more), marks each inside or not
(`Engine::mark_inside`: by its outline for a unit, by teigeodata's 750 m
and 700 m Voronoi radii for a settlement or a locality), and answers the
units holding the point from `admin_at`. In Nepal that chain ends at the
finest level the pack holds outlines for: the ward (level 4) in a pack
carrying them (teigeodata's TG-26 app pack); the municipality (level 3) in
`data-2026.11.15`, whose main pack holds no ward outlines. Elsewhere it
ends where the pack's outlines do: the world baseline holds countries and
first-level units, and the nearest settlement names the city.

**Missing levels are fewer answers, never a refusal.** A pack that keeps
outlines in Nepal and none abroad (TG-26), or only the world's levels 0
and 1, answers a point abroad with a shorter chain or none, and `near`
still names the nearest places. The same rule holds for `admin_at`, the
listing and a place's `parents`: a level the data does not hold is absent
(C2209).

### 12.6 Listings and population

`list` walks the admin hierarchy below `within` (an ISO 3166 code or a
place id; `Engine::children`, level by level, descending through units
only, at most 200,000 places visited) and keeps the places matching `level`
(a unit of exactly that level) and `kinds`, at most `depth` levels down
(default 1, the children; `null` for any depth). Units the data marks
former (teigeodata's `FLAG_FORMER`: Nepal's development regions and zones)
are left out, so a level-1 listing of Nepal is its seven provinces. It
answers them ranked by population, most first, a place with none after
every place with one, then by canonical name, then by id, so the order is
the same on every run (SR-129, restated as SR-143). Population is teigeodata's (D-351): a count a source
stated is `COUNTED`, a WorldPop grid summed over a unit `MODELLED` (most of
Nepal's local levels in the main build; the lite build carries counts
only). A listing ranks by the number whatever its kind, and says the kind,
so a page can show "about" for a modelled one.

### 12.7 What the probe measured

On `np-places-main.tgd` and `baseline-main.tgd` of `data-2026.11.15`
(SHA-256 as the release's `SHA256SUMS`), through the adapter:

| query | first three, as the adapter answers (its label, kind and score) |
|---|---|
| `काठमाडौं` | Kathmandu (settlement, 143), Kathmandu (district, 136), Kathmandu (metropolitan city, 134) |
| `Kathmandu` | the same three, the same scores |
| `Kathamandu` | the same three, 35 lower each (one edit) |
| `Kathmandu, Bagmati` | the same three, 1,000 higher each |
| reverse 27.7172, 85.324 | near: Kholagaal 109 m, inside; admin: Nepal (`NP`), Bagmati (`NP-P3`), Kathmandu district, Kathmandu metropolitan city |
| reverse 51.5074, −0.1278 | near: London 190 m, inside; admin: United Kingdom (`GB`), England (`GB-ENG`) |
| list `within: tgd:np/410` (Kathmandu district) | its 11 municipalities, Kathmandu first (863,151, modelled) |

The adapter's tests hold these (`adapters/geo-teigeodata/rust/tests/catalogue.rs`).

On teigeodata's app pack alone (`world-np-app-lite.tgd` of
`data-2026.11.17`, the pack Tithipatra's apps ship first; C2219):

| query | as the adapter answers |
|---|---|
| `काठमाडौं` / `Kathmandu` | Kathmandu settlement 144, district 136, metropolitan city 131, each `Kathmandu, Bagmati, Nepal` |
| `Kathamandu` | the same three, 109, 101, 96 |
| `Kathmandu, Bagmati` | the same three, 1,000 higher each |
| reverse 27.7172, 85.324 | near: Narayanhiti Royal Palace 331 m, inside; admin: Nepal (`NP`), Bagmati (`NP-P3`), Kathmandu district, Kathmandu metropolitan city; zone `Asia/Kathmandu`; no height |
| reverse 51.5074, −0.1278 | near: London 190 m, inside; no unit; zone `Europe/London` from the town (C2218) |
| 0, −30 (Atlantic) | no zone, no unit, not called sea (C2216) |
| list `NP`, level 1 | the seven provinces, no populations, by name |
| list Bagmati, level 3, any depth | 124 local levels, no populations, by name |

### 12.8 The table provider

`GeoTable` gains an optional `places` list, so a binding's tests, the
records' examples and a consumer with a few hundred places get answers
without a dataset. Each place is `{id, names, kind, level, code,
latitudeDeg, longitudeDeg, parent, population}`, `names` a list of
`{text, lang, script}`, the first the canonical one. The table answers:
search by a case- and accent-folded match of any name (exact before
prefix, then population), with a named parent after a comma; reverse by
great-circle distance, its `admin` the nearest place's chain of units;
names as written; listings as §12.6. It ranks more simply than
teigeodata does and makes no claim to its scores.

### 12.9 A saved place read back (SR-164)

A city a person saved is kept by its id, and read back later against
newer data. Two things can have happened to the id: the place merged into
another and its id was retired (teigeodata keeps a redirect, D-382), or it
belongs to the other id line (teigeodata numbers its `odbl` main and its
`cc-by-4.0` lite builds apart, D-368, so an id is looked up only in data of
its line). `place` answers the first, and `line` names the second:

```rust
fn place(&self, asked: &PlaceLookup) -> Result<Option<NamedPlace>, Error>;
```

`sdk.geo().place(&asked)` (`geo.place`, `ts_geo_place`) answers
`LookedUpPlace {asked, place, moved}`: `place` the record under its current
id, `null` for an id the provider does not know or holds as removed, and
`moved` true when that id is not the one asked, which the caller stores in
its place. The provider follows the redirect (teigeodata's `Engine::get`,
at most four hops); the SDK checks the record as every place is checked
and sets `moved` itself, so no provider can claim a move it did not make.
The answer is a record rather than a bare nullable place, so every binding
reads it with the shapes it already has.

`line` stays `null` from teigeodata 1.2.0: a pack's licence is in its
manifest, which the engine's `Layer` does not expose (it exposes the
address, component, release and attribution). It is asked for upstream;
the adapter fills it when the engine says it.

## 13. Over teigeodata's store, and on the web (SR-142)

### 13.1 teigeodata 1.1.0, then 1.2.0

SR-164 moves the adapter to `v1.2.0`: the live signed catalog's entries
name their files under a release folder, which readers before 1.2 refuse
(teigeodata D-384), and a place's landmark class is 1.2's (D-381). The
engine kept every call the adapter makes. Verify's `TEIGEODATA_DATA` stays
`data-2026.11.15`, whose packs 1.2 reads, since `data-2026.11.22.2` carries
no baseline; a retired id answers its survivor only over packs that carry
the redirects (from `data-2026.11.22.2`), and the adapter's test of it says
`SKIP` over older ones.


The adapter moves from the `v0.3.1` tag to `v1.1.0`, the stable release
(`gh release list -R teispace/teigeodata`, 2026-11-17). Its packs are
format 1.0, which a v0.3 reader refuses; `data-2026.11.15` is the first
data release in it, so verify's `TEIGEODATA_DATA` moves with the tag. The
engine's API kept every call §10 makes; 1.0 made `PlaceKind` and the other
vocabularies `#[non_exhaustive]`, which the adapter matches with a
wildcard arm.

### 13.2 An engine the store fills

teigeodata's store (`teigeodata-sync`) installs, updates and repairs packs
on the device, and `Shared::follow` (feature `store`) opens what its journal
lists into a `Shared` engine, publishing each new set atomically while
queries run on the old one (teigeodata's D-141, D-172). The adapter holds a
`Shared` rather than an `Engine`:

- `TeigeodataGeo::from_shared(Arc<Shared>)` takes the application's own
  `Shared`; the application keeps it and calls `follow` (or `update`) after
  each install, and every later question reads the new set. No context is
  rebuilt: the context holds the provider, the provider holds the `Shared`.
- `from_engine(Engine)`, `open` and `from_bytes` wrap a `Shared` of their
  own, reachable through `shared()`, so packs can be added to them later
  too.
- Each question loads one snapshot (`Shared::load`, lock-free) and answers
  wholly from it. `source()` names the snapshot's packs, so a resolution
  is stamped with the set that answered.
- An empty `Shared` is accepted by `from_shared` (the store may not have
  installed anything yet): every question answers nothing and the source's
  `dataVersion` is `none` until a pack arrives (C2203). `open`,
  `from_bytes` and `from_engine` still refuse an empty set, since nothing
  will fill it.
- The zone rules are read when `time_zones()` is called: a context takes
  them once, at build, from the snapshot then current (C2204).
- `geo.resolve` asks its three questions one after another, so a swap
  between them could stamp the newer set on an answer partly read from the
  older; the window is one resolution, and a stored place is replayed from
  the request, never re-asked.

The adapter depends on neither `teigeodata-sync` nor the engine's `store`
feature: the application brings its store, so a build that reads only
shipped packs, and the web build, carry no HTTP or TLS stack.

### 13.3 Feature sets and the web

| adapter feature | engine features | gives |
|---|---|---|
| (always) | `admin`, `admin-index`, `names`, `timezone`, `timezone-index`, `extra`, `shared` | the zone, the admin units, reverse, names, listings, population |
| `mmap` | `mmap` | `open`: pack files mapped (native only) |
| `search` | `search`, `substring` | `search` across scripts; without it a search is `CAPABILITY` on `search` (C2207) |
| `terrain` | `dem`, `dem-coarse` | `height_at`; without it no height |
| `zone-rules` | `tz-rules` | `time_zones()`; without it none |

The default is every feature. **Lookup-only** is no `mmap` and no
`search`, as teigeodata's own `teigeodata-wasm-lookup` leaves out search:
`--no-default-features --features terrain,zone-rules`, which builds for
`wasm32-unknown-unknown` with packs from `from_bytes` and is checked by
verify's `geo-adapter` job. A web build that wants search adds `search`;
it builds for wasm32 too.

Sizes, as built (stripped) and gzipped, of the adapter's probe binary
(`examples/size.rs`: a provider from bytes asked every question through
`dyn GeoProvider`) minus the same binary with the provider left out, so
the number is what the adapter and its engine add to a consumer:


| target | features | as built | gzipped | added, as built | added, gzipped |
|---|---|---:|---:|---:|---:|
| aarch64-apple-darwin | none (probe without the adapter) | 452,960 | 236,883 | — | — |
| aarch64-apple-darwin | `--no-default-features` | 535,504 | 269,421 | 82,544 | 32,538 |
| aarch64-apple-darwin | lookup-only (`terrain,zone-rules`) | 568,912 | 300,725 | 115,952 | 63,842 |
| aarch64-apple-darwin | lookup + `search` | 800,576 | 418,546 | 347,616 | 181,663 |
| aarch64-apple-darwin | default (+ `mmap`) | 800,576 | 418,718 | 347,616 | 181,835 |
| wasm32-unknown-unknown | none (probe without the adapter) | 251,978 | 102,574 | — | — |
| wasm32-unknown-unknown | `--no-default-features` | 348,122 | 123,178 | 96,144 | 20,604 |
| wasm32-unknown-unknown | lookup-only (`terrain,zone-rules`) | 408,979 | 148,467 | 157,001 | 45,893 |
| wasm32-unknown-unknown | lookup + `search` | 645,364 | 257,566 | 393,386 | 154,992 |

Bytes, built with the adapter's `size` profile (`opt-level = "z"`, fat
LTO, one codegen unit, `panic = "abort"`, stripped) by rustc 1.100 on
2026-11-17, gzipped with `gzip -9`; wasm32 not passed through `wasm-opt`.
Search is the heaviest part, about 230 KB as built (118 KB gzipped)
natively and 236 KB (109 KB) on wasm32: the fold and phonetic tables
and the trigram matcher. `mmap` adds nothing measurable natively; it is
not offered on wasm32.

## 14. Left for later

- **SR-88** is built (§10), the zone rules and the context side of §8
  included.
- **SR-90** is built (§6a).
- A conformance kit for geo providers like `ephemeris-kit`, when there is a
  second provider to hold to the first.
