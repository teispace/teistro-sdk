# Security architecture

Status: `draft`, 2026-09-04. The threat model is in
`01-research/platform/08-security.md`.

## Boundaries and their defences

| boundary | defence |
|---|---|
| C ABI entry | `struct_size` validation (zero and oversize refused, shorter served), capacity checks, enum range checks, finite-number checks, `catch_unwind` converting panics to `INTERNAL` with no unwinding across the boundary |
| provider callbacks | every returned buffer length checked, every value finite and in range, timeouts not applicable (synchronous) but iteration caps on searches that call providers |
| packs | magic, version, length, CRC; schema validation; string arena bounds; key ids resolved against the catalogue; size caps; optional Ed25519 signature check for signed packs |
| rule packs | predicate depth and count limits; no arbitrary code; compiled form validated |
| limits | per-context defaults for batch sizes (for example 10,000 instants), date ranges, candidate counts, cache bytes; exceeding is `LIMIT` |
| memory | Rust ownership; the ffi crate is the only `unsafe`; audited and fuzzed |
| logging | consumer callback, off by default, never receives personal data unless the consumer passes it |

## Build and supply chain

- `cargo-deny` (licences, advisories, bans), `cargo-audit`, `cargo-vet`
  in CI; lockfiles committed; dependency count reviewed per release.
- Reproducible builds; SBOM (CycloneDX) per artefact; artefacts signed
  with Sigstore and carrying SLSA-style provenance; prebuilds verified by
  hash at install.
- Sanitizers (ASan, UBSan, TSan) on the ffi crate; Miri on safe crates;
  fuzz targets for pack parsers, the MF2 engine, the blob decoders and
  every C ABI entry point with a committed corpus.

## Policy

- `SECURITY.md` with a private reporting channel and a disclosure
  timeline.
- Threat model review at each major version.
- No environment variables, files or network access in the core; adapters
  that read files (ephemeris data) document exactly what they read.
