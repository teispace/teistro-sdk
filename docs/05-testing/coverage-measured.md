# Coverage floors, measured

Status: `generated` by `cargo xtask coverage --from FILE` from verify's `coverage` job; `check-coverage` holds this page to [`coverage.json`](coverage.json), and verify holds every run to the floors.

Each crate's floor is the share of its lines its tests run, rounded down to a whole percent, from the last run recorded; a crate's generated tables are not counted, since their generators' checks hold them, and `scenario` is not measured: the fixed workload the determinism and instruction-count gates walk, run by those gates rather than by tests of its own ([`01-quality-bar.md`](01-quality-bar.md), "coverage floors"). A floor only rises: a run below it fails, recording raises it to what the run reached, and a floor lowered by hand fails against `main`'s. The quality bar asks 90% of a computation crate; 36 of the 43 crates have reached it.

| crate | floor | at 90% |
|---|---:|---|
| `aspect` | 99% | yes |
| `astro` | 93% | yes |
| `calendar` | 93% | yes |
| `chart` | 93% | yes |
| `core` | 94% | yes |
| `dasha` | 96% | yes |
| `ephemeris-builtin` | 95% | yes |
| `ephemeris-kit` | 80% | no |
| `ffi` | 95% | yes |
| `geometry` | 97% | yes |
| `gochar` | 98% | yes |
| `hellenistic` | 96% | yes |
| `houses` | 98% | yes |
| `idl` | 54% | no |
| `interpret` | 98% | yes |
| `intl` | 79% | no |
| `kp` | 92% | yes |
| `lalkitab` | 94% | yes |
| `matching` | 98% | yes |
| `mcp` | 78% | no |
| `muhurta` | 96% | yes |
| `numerology` | 94% | yes |
| `pakshi` | 96% | yes |
| `panchanga` | 94% | yes |
| `points` | 99% | yes |
| `port-ephemeris` | 89% | no |
| `port-timezone` | 68% | no |
| `prashna` | 90% | yes |
| `rashifal` | 94% | yes |
| `rectification` | 93% | yes |
| `remedies` | 85% | no |
| `render-svg` | 98% | yes |
| `research` | 96% | yes |
| `rules` | 97% | yes |
| `sdk` | 92% | yes |
| `serial` | 97% | yes |
| `siddhanta` | 94% | yes |
| `state` | 97% | yes |
| `strength` | 99% | yes |
| `tajika` | 97% | yes |
| `time` | 90% | yes |
| `vargas` | 94% | yes |
| `western` | 98% | yes |
