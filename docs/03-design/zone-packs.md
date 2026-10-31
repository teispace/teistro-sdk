# The zone database from a signed pack

Status: `built`, 2026-10-30. Design for the almanac app's SR-09
(`consumers/addendum-3.md`), written before the code and corrected by it.
§6 was measured through the built call.

The SDK reads zones from a database compiled into it (`EmbeddedTzdb`), so a
government that moves its clocks waits for an SDK release. On the day this
page was written IANA's latest release was 2026f (27 October 2026) and the
embedded one was 2026c. An app must be able to take a newer database as
data, without a release, and say which one gave an answer.

## 1. What IANA ships

The primary sources are IANA's pages and RFC 9636:

- **Names.** "Since 1996, each version has been a four-digit year followed
  by lower-case letter", running "a through z, then za through zz, then zza
  through zzz, and so on" (`tz-link.html`). Those strings sort in release
  order as plain text, so `at_least` (§4) compares them as text. The version
  is the first line of the file `version` in each release (`theory.html`).
- **Files.** `tzcodeV.tar.gz`, `tzdataV.tar.gz` and `tzdb-V.tar.lz`
  (`tz-link.html`). IANA ships source text, and `zic` compiles it. The
  release directory also holds a detached PGP signature beside each tarball
  (`tzdata2026f.tar.gz.asc`).
- **Compiled form.** TZif: the magic `TZif`, a version octet, and from
  version 2 a 64-bit block and a footer (RFC 9636 §3, October 2024, which
  obsoletes RFC 8536; the media type is `application/tzif`).
- **What is stable.** Names are part of the stable interface, and a renamed
  zone's old spelling stays as a link in `backward`; boundaries are not
  (`theory.html`). The same page warns that "many, perhaps most, of the `tz`
  database's pre-1970 and future timestamps are either wrong or misleading".
  A newer pack can therefore move a historical birth's offset, which is
  why a stored chart replays from its `ZoneResolution`
  (`time-and-timezone.md` §3.2) and a pack never changes a stored instant.

## 2. What is the SDK's

As with published sets (`published-overrides.md` §1), fetching, storage,
expiry and key rotation are the transport's. The SDK takes the smallest
verifiable form: **the pack's exact bytes, a detached Ed25519 signature and
the caller's public key**, behind the same `signed-packs` feature, with no
key embedded. The check is one function, `teistro_core::signed::verify`,
which `PublishedSet::from_signed` now calls too. It moved down to the core
so the time layer could reach it, and its refusals did not change.

A pack carries IANA's own compiled TZif, read by the engine the embedded
database already uses (`jiff`'s `TimeZone::tzif`), so a pack and the embedded
database cannot disagree about what a row means.

## 3. The pack

```json
{
  "format": "teistro.zones/1",
  "version": "2026f",
  "zones": {"Asia/Kathmandu": "VFppZjIAAAA…"},
  "links": {"Asia/Katmandu": "Asia/Kathmandu"}
}
```

`zones` maps a name to its TZif, in standard base64. `links` maps a name to a
name of `zones`, so identical files are held once. JSON keeps one strict
reader and adds no binary parser to fuzz (`teistro_core::strict`, the
`published` set's), and it gzips back to about the size of the TZif.

`ZonePack::write` is the one writer, and `cargo xtask zone-pack --from DIR
--version V --out FILE` runs it over a compiled tree: `zic -b slim -d DIR` of
`rearguard` data, as the embedded database's is built (C831). The caller
signs the file with its own key.

**Checks.** Each refusal is `INVALID_ARG` on the named field: a format this
SDK does not read (`format`); a version that is not IANA's shape
(`version`); no zones (`zones`); a name that is empty or not a relative
path of `A-Za-z0-9_+-/`, or text that is not base64 or not TZif, on
`zones.<name>`; a link to a name `zones` lacks, or a name given twice
(names compare ignoring case, as the embedded database's do), on
`links.<name>`. Anything over 16 MiB is refused before it is read
(`LIMIT`, field `pack`); IANA's whole compiled set is about 0.4 MiB. Every
zone is parsed at load, so a bad row refuses the load and never a later
answer.

## 4. Into a context

`sdk.time().load_zones(pack, signature, key, at_least)` verifies, reads and
installs a pack on that context, and answers a `ZoneDatabase`: `{source,
version, zones}`. `time().zone_database()` reads the same record at any time.
In Rust, `ContextBuilder::zones(Arc<ZonePack>)` shares one parsed pack among
worker contexts, and `ZonePack::from_bytes` is the door for a consumer whose
own transport verified the bytes (no feature needed). At the C boundary and
in the bindings the only door is the signed one, `ts_time_load_zones`. An app
holds a key anyway, and a boundary whose missing signature means "skip the
check" would be a trap.

`at_least` is the newest release the app has applied, `""` for any. An older
pack is refused on `version`, so a replayed pack cannot roll a correction
back. It is deliberately not held to the embedded version: pinning an older
database is how a consumer reproduces an old answer. An app that never wants
to go below its build passes `bundled` from `zone_database()`.

**From then on every path reads it**: `resolve`, `civil_of`, a message's
`timeZone`, an almanac asked by zone (`time-and-timezone.md` §3.4), and the
provenance's `time.tzdb_version`, which is how the pack's version is
reported on the answer. `ZoneResolution.tzdb_version` says the same on a
resolution.

**Settings hash.** The database is not a setting: `settings_hash` does not
change. Two contexts with one hash and different databases are told apart
by `tzdb_version`, as a context before and after an SDK upgrade already is.

## 5. The fallback (C830)

`EmbeddedTzdb` stays. A context with no pack answers from it, and a pack
that is refused leaves the context on what it had. **The fallback is per
context and not per zone.** A zone a pack lacks is refused as an unknown
zone, with the pack's nearest names. A per-zone fallback would answer from
two databases under one `tzdb_version`, which is the one thing an answer
must not be wrong about. A subset pack is allowed, for an app that serves one
country; it knows what it left out, and `zones` counts them.

## 6. The gate

- **The same data, the same answers.** A pack written from the embedded data
  itself answers as `EmbeddedTzdb` does, for every zone, at instants from
  1800 to 2100 (offset, abbreviation, `is_dst`, `before_rules`), candidates
  included (`crates/time/tests/zone_pack.rs`).
- **A newer row wins.** A pack holding a test zone with a changed offset
  answers the changed offset through `time.resolve`, and its version is on
  the resolution and in the provenance (`crates/sdk/tests/zone_pack.rs`).
- **Refusals.** A tampered byte, another key, a replayed older version and
  each §3 check are refused by name, and the context stays as it was.
- **Every binding** loads one signed pack through its parity runner and
  reads the pack's release back from a day's provenance.

**Measured** on the release itself, 2026f compiled by `zic` beside the
embedded 2026c: see §7.

## 7. Measured

Release 2026f, rearguard form, compiled with `zic -b slim` and packed with
`xtask zone-pack`, on an Apple M2 Pro in a release build:

| Measure | Value |
|---|---|
| Pack | 597 names, 290,116 bytes, 74,478 gzipped |
| `ZonePack::from_bytes`, every zone parsed | 1.65 ms, best of 7 |
| Names beside the embedded 2026c | the same 598, less the zone below |
| Zones differing from 2026c, weekly 1900 to 2040 | 12 |

**One zone was left out, and the pack would not have loaded with it.** The
`zic` on the measuring machine is 2022g. Its output for `America/Ojinaga`, from
both the main and the rearguard data, is refused by the TZif reader: the last
transition does not agree with the footer's rule. The tool refuses the whole
set by default, which is right, so the measurement removed that one file. A
publisher should compile with the `zic` of the release; that has not been
measured here, so whether a current `zic` clears this is open.

The 12 zones, with the first and last differing week as Julian days:

| Zones | From | To |
|---|---|---|
| `America/Havana`, `Cuba` | 2461346.5 | 2461472.5 |
| `America/Inuvik`, `America/Rainy_River`, `America/Winnipeg`, `Canada/Central` | 2461346.5 | 2465994.5 |
| `CST6CDT`, `EST5EDT`, `MST7MDT`, `PST8PDT` | 2415020.5 | 2439422.5 |
| `Eire`, `Europe/Dublin` | 2424414.5 | 2424421.5 |

The first two rows begin in November 2026, after the embedded release: rows
only a newer database can answer, which is why a pack exists. The last two
rows are in the past and were not traced to their cause here; a publisher
should read `NEWS` before relying on either side for those dates.
