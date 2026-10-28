# Coverage floors, measured

Status: `generated` by `cargo xtask coverage --from FILE` from verify's `coverage` job; `check-coverage` holds this page to [`coverage.json`](coverage.json), and verify holds every run to the floors.

Each crate has two floors: the share of its lines its tests run and the share of its branches, each rounded down to a whole percent, from the last run recorded. Both are counted by `nightly-2026-10-21`, the pinned nightly `cargo llvm-cov --branch` needs, over shipped code; each llvm counts lines its own way, so a record another toolchain counted, or one that counted other code, is re-based on the next run. A test module is `coverage(off)` under the nightly, since its own assertions are branches its passing takes one way. A crate's generated tables are not counted, since their generators' checks hold them, and `scenario` is not measured: the fixed workload the determinism and instruction-count gates walk, run by those gates rather than by tests of its own ([`01-quality-bar.md`](01-quality-bar.md), "coverage floors"). A floor only rises: a run below it fails, recording raises it to what the run reached, and a floor lowered by hand fails against `main`'s. The quality bar asks 90% of lines and of branches of a computation crate; 18 of the 43 crates have reached both.

| crate | lines | branches | at 90% |
|---|---:|---:|---|
| `aspect` | 98% | 96% | yes |
| `astro` | 92% | 85% | no |
| `calendar` | 93% | 92% | yes |
| `chart` | 90% | 81% | no |
| `core` | 97% | 88% | no |
| `dasha` | 97% | 91% | yes |
| `ephemeris-builtin` | 95% | 81% | no |
| `ephemeris-kit` | 81% | 65% | no |
| `ffi` | 96% | 81% | no |
| `geometry` | 97% | 89% | no |
| `gochar` | 97% | 91% | yes |
| `hellenistic` | 93% | 83% | no |
| `houses` | 97% | 100% | yes |
| `idl` | 84% | 76% | no |
| `interpret` | 95% | 90% | yes |
| `intl` | 78% | 75% | no |
| `kp` | 92% | 89% | no |
| `lalkitab` | 94% | 86% | no |
| `matching` | 98% | 97% | yes |
| `mcp` | 87% | 80% | no |
| `muhurta` | 94% | 86% | no |
| `numerology` | 93% | 94% | yes |
| `pakshi` | 96% | 83% | no |
| `panchanga` | 94% | 85% | no |
| `points` | 98% | 100% | yes |
| `port-ephemeris` | 88% | 70% | no |
| `port-timezone` | 100% | 100% | yes |
| `prashna` | 90% | 92% | yes |
| `rashifal` | 93% | 86% | no |
| `rectification` | 92% | 87% | no |
| `remedies` | 83% | 82% | no |
| `render-svg` | 98% | 90% | yes |
| `research` | 95% | 78% | no |
| `rules` | 96% | 91% | yes |
| `sdk` | 91% | 85% | no |
| `serial` | 97% | 91% | yes |
| `siddhanta` | 91% | 84% | no |
| `state` | 94% | 95% | yes |
| `strength` | 98% | 94% | yes |
| `tajika` | 95% | 91% | yes |
| `time` | 91% | 84% | no |
| `vargas` | 90% | 94% | yes |
| `western` | 96% | 89% | no |

## The bindings

Each binding's own layer, its generated readers and its ergonomic code, under its own tool (`03-design/binding-coverage.md`), recorded by `cargo xtask coverage --bindings DIR` from the reports the binding checks write when `TEISTRO_BINDING_COVERAGE` names a directory. A floor rises as a crate's does, and a tool's new version re-bases it. The quality bar asks 80% of a binding's lines, and of its branches where its tool counts them.

| binding | lines | branches | counted by | at 80% |
|---|---:|---:|---|---|
| dart | 65% | 48% | `dart 3.13.7` | no |
| java | 59% | 50% | `jacoco 0.8.15` | no |
| node | 98% | 83% | `node 22.23.4` | yes |
| python | 92% | 77% | `coverage.py 7.16.2` | no |
