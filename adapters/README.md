# Adapters

The adapters in this repository, outside the SDK workspace by design
(ADR-0019, `CLEAN_ROOM.md`): each is a standalone crate that depends on the
SDK's port (and kit), never the reverse, and the workspace builds and passes
its tests with the analytic test provider alone. None is published.

| adapter | engine | needs | run | licence |
|---|---|---|---|---|
| [`ephemeris-sweph/rust`](ephemeris-sweph/rust) | the Swiss Ephemeris C library, compiled from sources you bring, named at build time; declares the obliquity, Delta T, ayanamsha and topocentric overrides | `SWEPH_SRC_DIR` (the library's sources), `SWEPH_DATA_DIR` (its `.se1` files; `ephe/` in the working directory by default) | `cargo run --release --manifest-path adapters/ephemeris-sweph/rust/Cargo.toml` | this crate's source Apache-2.0; a build links the Swiss Ephemeris and is under its terms (AGPL-3.0 or Astrodienst's Professional Licence), which you hold |
| [`geo-teigeodata/rust`](geo-teigeodata/rust) | a teigeodata engine, for the geo port (below) | the packs, `TEISTRO_TEIGEODATA_PACKS` | `cargo test --manifest-path adapters/geo-teigeodata/rust/Cargo.toml` | Apache-2.0, as teigeodata is |

**Any engine plugs in the same way.** An adapter is a crate with a `cdylib`
and a kit binary, and a package per binding; `cargo xtask check-adapter
--adapter DIR` holds one to what every provider is held to, wherever it
lives (`docs/06-cicd/adapter-gate.md`). Teimeris, Teispace's own engine, is
one: its adapters are Teispace-proprietary, kept in the engine's private
repository and checked against every SDK batch before it merges, and are not
part of this repository.

The Swiss Ephemeris adapter reads the `.se1` file family and declares its
coverage and content hashes for the directory it opens
(`teistro_port_ephemeris::sefile`). Its kit report and timings are quoted in
`docs/03-design/ephemeris-port-and-adapters.md`.

## Geo

[`geo-teigeodata/rust`](geo-teigeodata/rust) makes a teigeodata engine a
`GeoProvider` for the geo port (SR-88). It depends on teigeodata at tag
`v0.1.0`, by git, and opens the packs a consumer supplies. teigeodata is
Apache-2.0, so this adapter is public and its build keeps that licence. Its
tests read the packs named by `TEISTRO_TEIGEODATA_PACKS`
([README](geo-teigeodata/README.md), `docs/03-design/geo-port.md` §10).
