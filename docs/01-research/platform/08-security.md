# Security and robustness

Status: `research`, 2026-09-04. Feeds
`02-architecture/10-security-architecture.md`.

## Threat model

The SDK runs inside consumers' processes: servers handling untrusted user
input (birth data, names, dates, numbers), mobile apps, browsers. Its inputs
are: user-supplied values through the API, consumer-supplied providers
(callbacks), and data packs (locale, interpretation, rules) that may come
from third parties. It performs no I/O of its own. The assets are the
consumer's process integrity and availability, and the correctness of
results.

| threat | mitigation |
|---|---|
| memory corruption from malformed inputs across FFI | Rust core; `struct_size` and capacity conventions at the C ABI; every boundary struct validated (size zero or too large refused, shorter served); no panic crosses the boundary (`catch_unwind` at every export, converted to a structured error) |
| malicious or corrupt data packs | binary packs carry magic, format version, length and CRC; parsed with bounds checks; schema validated at load; a pack cannot reference code; size caps; optional signature verification for commercial packs |
| denial of service through pathological inputs (huge batch sizes, date ranges, iteration-heavy searches) | explicit limits on batch sizes and ranges (configurable per context with documented defaults); iteration caps on every search with `NOT_CONVERGED` status; memory caps on caches; no unbounded recursion (rule trees have a depth limit) |
| provider misbehaviour (callback returns NaN, wrong length, throws) | every provider result validated (finite, in range, length matches); a provider error is a structured SDK error naming the provider; no NaN propagates |
| undefined behaviour in the core | `#![forbid(unsafe_code)]` outside the `ffi` crate; Miri on the safe crates' tests; ASan, UBSan and TSan builds of the FFI crate in CI; fuzzing of pack parsers, the MF2 engine, the JSON decoders and the C ABI entry points |
| supply chain | pinned lockfiles; `cargo-deny` (licences, advisories, duplicate crates); `cargo-audit`; `cargo-vet` for the dependency set; minimal dependency policy; SBOM (CycloneDX) per release; reproducible builds; signed artefacts with provenance (Sigstore); prebuilt binaries verified by hash at install |
| information leakage | the SDK reads no environment variables, no files and no network; logging is a callback the consumer installs, off by default; no personal data is retained in caches beyond the consumer's context lifetime |
| timing side channels | not applicable (no secrets) |
| version confusion between binding halves | each package refuses to load when its language half and native half differ (Teimeris rule) |

## Robustness rules

- Invalid input is a structured error with the field name and the accepted
  range; never a plausible number.
- Degenerate astronomy (polar day, undefined house system at a latitude,
  provider coverage exceeded) is a reported state, not an exception, and
  never an invented value.
- Every search has an iteration cap and a documented tolerance.
- Results are deterministic and carry their versions; a cache never mixes
  versions.

## Processes

- A security policy file with a reporting channel and disclosure timeline.
- Threat model reviewed at each major version.
- Fuzz targets run continuously (a scheduled job), corpus committed.
- Dependency updates are measured events with the "Numbers" changelog rule:
  a dependency bump that moves any output is a minor version with the size
  of the difference stated.
