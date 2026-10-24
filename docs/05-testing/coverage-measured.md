# Coverage floors, measured

Status: `generated` by `cargo xtask coverage --from FILE` from verify's `coverage` job; `check-coverage` holds this page to [`coverage.json`](coverage.json), and verify holds every run to the floors.

Each crate has two floors: the share of its lines its tests run and the share of its branches, each rounded down to a whole percent, from the last run recorded. Branches are llvm's, measured on the pinned nightly `cargo llvm-cov --branch` needs. A crate's generated tables are not counted, since their generators' checks hold them, and `scenario` is not measured: the fixed workload the determinism and instruction-count gates walk, run by those gates rather than by tests of its own ([`01-quality-bar.md`](01-quality-bar.md), "coverage floors"). A floor only rises: a run below it fails, recording raises it to what the run reached, and a floor lowered by hand fails against `main`'s. The quality bar asks 90% of lines and of branches of a computation crate; 0 of the 43 crates have reached both.

| crate | lines | branches | at 90% |
|---|---:|---:|---|
| `aspect` | 99% | not yet measured | no |
| `astro` | 94% | not yet measured | no |
| `calendar` | 93% | not yet measured | no |
| `chart` | 93% | not yet measured | no |
| `core` | 94% | not yet measured | no |
| `dasha` | 96% | not yet measured | no |
| `ephemeris-builtin` | 95% | not yet measured | no |
| `ephemeris-kit` | 84% | not yet measured | no |
| `ffi` | 95% | not yet measured | no |
| `geometry` | 97% | not yet measured | no |
| `gochar` | 98% | not yet measured | no |
| `hellenistic` | 96% | not yet measured | no |
| `houses` | 98% | not yet measured | no |
| `idl` | 86% | not yet measured | no |
| `interpret` | 98% | not yet measured | no |
| `intl` | 83% | not yet measured | no |
| `kp` | 92% | not yet measured | no |
| `lalkitab` | 94% | not yet measured | no |
| `matching` | 98% | not yet measured | no |
| `mcp` | 88% | not yet measured | no |
| `muhurta` | 96% | not yet measured | no |
| `numerology` | 94% | not yet measured | no |
| `pakshi` | 96% | not yet measured | no |
| `panchanga` | 94% | not yet measured | no |
| `points` | 99% | not yet measured | no |
| `port-ephemeris` | 91% | not yet measured | no |
| `port-timezone` | 100% | not yet measured | no |
| `prashna` | 90% | not yet measured | no |
| `rashifal` | 94% | not yet measured | no |
| `rectification` | 93% | not yet measured | no |
| `remedies` | 85% | not yet measured | no |
| `render-svg` | 98% | not yet measured | no |
| `research` | 96% | not yet measured | no |
| `rules` | 97% | not yet measured | no |
| `sdk` | 92% | not yet measured | no |
| `serial` | 97% | not yet measured | no |
| `siddhanta` | 94% | not yet measured | no |
| `state` | 97% | not yet measured | no |
| `strength` | 99% | not yet measured | no |
| `tajika` | 97% | not yet measured | no |
| `time` | 90% | not yet measured | no |
| `vargas` | 94% | not yet measured | no |
| `western` | 98% | not yet measured | no |
