# Adapters

The ephemeris adapters for licensed engines, outside the SDK workspace
by design (ADR-0019, `CLEAN_ROOM.md`): each is a standalone crate that
depends on the SDK's port and kit, never the reverse, and the workspace
builds and passes its tests with the analytic test provider alone. A
binary that links an engine is distributed under the engine's terms; the
adapters' own source is Apache-2.0.

| adapter | engine | needs | run |
|---|---|---|---|
| [`ephemeris-teimeris/rust`](ephemeris-teimeris/rust) | Teimeris, through its Rust binding; declares the obliquity, Delta T, ayanamsha, topocentric, rise-and-set, crossings and stations overrides | `TEIMERIS_LIB_DIR` (the engine's release build), `TEIMERIS_DATA_DIR` (the `.se1` files; the Teimeris checkout's `data/` by default) | `cargo run --release --manifest-path adapters/ephemeris-teimeris/rust/Cargo.toml --bin teistro-ephemeris-teimeris-kit`; `--bin teistro-ephemeris-teimeris-bs-fit` for the Bikram Sambat measurement |
| [`ephemeris-sweph/rust`](ephemeris-sweph/rust) | the Swiss Ephemeris C library, compiled from sources named at build time; declares the obliquity, Delta T, ayanamsha and topocentric overrides | `SWEPH_SRC_DIR` (the library's sources), `SWEPH_DATA_DIR` | `cargo run --release --manifest-path adapters/ephemeris-sweph/rust/Cargo.toml` |

Both read the same `.se1` file family and declare the same coverage and
content hashes for the same directory (`teistro_port_ephemeris::sefile`).
Their kit reports and timings are quoted in
`docs/03-design/ephemeris-port-and-adapters.md`; the Teimeris adapter's
tests also compare the SDK's rise and set solver with the engine's own
search and with the baseline's 55 fixture charts
(`docs/03-design/astro-events-and-crossings.md`).
