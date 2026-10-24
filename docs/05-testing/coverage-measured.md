# Coverage floors, measured

Status: `generated` by `cargo xtask coverage --from FILE` from verify's `coverage` job; `check-coverage` holds this page to [`coverage.json`](coverage.json), and verify holds every run to the floors.

Each crate has two floors: the share of its lines its tests run and the share of its branches, each rounded down to a whole percent, from the last run recorded. Both are counted by `nightly-2026-10-21`, the pinned nightly `cargo llvm-cov --branch` needs; each llvm counts lines its own way, so a record another toolchain counted is re-based on the next run. A crate's generated tables are not counted, since their generators' checks hold them, and `scenario` is not measured: the fixed workload the determinism and instruction-count gates walk, run by those gates rather than by tests of its own ([`01-quality-bar.md`](01-quality-bar.md), "coverage floors"). A floor only rises: a run below it fails, recording raises it to what the run reached, and a floor lowered by hand fails against `main`'s. The quality bar asks 90% of lines and of branches of a computation crate; 11 of the 43 crates have reached both.

| crate | lines | branches | at 90% |
|---|---:|---:|---|
| `aspect` | 99% | 90% | yes |
| `astro` | 94% | 82% | no |
| `calendar` | 95% | 80% | no |
| `chart` | 94% | 81% | no |
| `core` | 97% | 84% | no |
| `dasha` | 98% | 90% | yes |
| `ephemeris-builtin` | 96% | 81% | no |
| `ephemeris-kit` | 81% | 60% | no |
| `ffi` | 96% | 79% | no |
| `geometry` | 97% | 89% | no |
| `gochar` | 98% | 92% | yes |
| `hellenistic` | 96% | 79% | no |
| `houses` | 98% | 60% | no |
| `idl` | 86% | 75% | no |
| `interpret` | 98% | 89% | no |
| `intl` | 82% | 75% | no |
| `kp` | 92% | 89% | no |
| `lalkitab` | 94% | 86% | no |
| `matching` | 98% | 97% | yes |
| `mcp` | 88% | 80% | no |
| `muhurta` | 96% | 85% | no |
| `numerology` | 93% | 94% | yes |
| `pakshi` | 96% | 83% | no |
| `panchanga` | 94% | 85% | no |
| `points` | 99% | 80% | no |
| `port-ephemeris` | 91% | 68% | no |
| `port-timezone` | 100% | 100% | yes |
| `prashna` | 90% | 92% | yes |
| `rashifal` | 93% | 86% | no |
| `rectification` | 93% | 87% | no |
| `remedies` | 83% | 82% | no |
| `render-svg` | 98% | 89% | no |
| `research` | 95% | 78% | no |
| `rules` | 97% | 84% | no |
| `sdk` | 91% | 85% | no |
| `serial` | 98% | 91% | yes |
| `siddhanta` | 94% | 68% | no |
| `state` | 97% | 95% | yes |
| `strength` | 99% | 93% | yes |
| `tajika` | 97% | 85% | no |
| `time` | 90% | 70% | no |
| `vargas` | 94% | 91% | yes |
| `western` | 98% | 83% | no |
