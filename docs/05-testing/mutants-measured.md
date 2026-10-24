# Mutation testing, measured

Status: `generated` by `cargo xtask mutants --from DIR` from the `mutants` workflow's run; `check-mutants` holds this page to [`mutants.json`](mutants.json), and the workflow holds every run to it.

Each kernel crate's floor is the share of its mutants that its tests catch, rounded down to a whole percent; a mutant that does not build is not counted, and one listed in `.cargo/mutants.toml`, with why it cannot change what the code computes, is never made ([`01-quality-bar.md`](01-quality-bar.md), "mutation testing"). A floor only rises. Every survivor is a missing test, and a run fails on one this page does not name, so the list below only shrinks. The quality bar asks 80% of a kernel crate; 4 of the 4 have reached it.

| crate | floor | at 80% | survivors |
|---|---:|---|---:|
| `dasha` | 99% | yes | 8 |
| `rules` | 99% | yes | 8 |
| `strength` | 100% | yes | 0 |
| `vargas` | 100% | yes | 0 |

## `dasha`

- `crates/dasha/src/firdaria.rs: replace > with < in <impl Timeline for FirdariaDasha>::child`
- `crates/dasha/src/firdaria.rs: replace || with && in <impl Timeline for FirdariaDasha>::child`
- `crates/dasha/src/kalachakra.rs: replace > with < in <impl Timeline for KalachakraDasha>::mahadasha`
- `crates/dasha/src/kalachakra.rs: replace || with && in <impl Timeline for KalachakraDasha>::mahadasha`
- `crates/dasha/src/rashi.rs: replace || with && in <impl Timeline for RashiDasha>::mahadasha_at`
- `crates/dasha/src/tree.rs: replace + with - in <impl Timeline for Dasha>::mahadasha_at`
- `crates/dasha/src/tree.rs: replace match guard s == total with false in <impl Timeline for Dasha>::child`
- `crates/dasha/src/tree.rs: replace || with && in <impl Timeline for Dasha>::mahadasha_at`

## `rules`

- `crates/rules/src/longevity/ashtakavarga.rs: replace > with >= in Evaluator<'_>::ashtakavarga_ayus`
- `crates/rules/src/longevity/choice.rs: delete match arm Ayus::Bhinnashtakavarga in span_choice`
- `crates/rules/src/longevity/choice.rs: delete match arm Ayus::Samudaya in span_choice`
- `crates/rules/src/longevity/maraka.rs: delete ! in Evaluator<'_>::marakas`
- `crates/rules/src/longevity/maraka.rs: replace && with || in Evaluator<'_>::marakas`
- `crates/rules/src/longevity/rasmi.rs: replace / with % in graha_rays`
- `crates/rules/src/longevity/rasmi.rs: replace / with * in graha_rays`
- `crates/rules/src/shipped.rs: replace packs -> [&'static[Rule]; 6] with [Vec::leak(Vec::new()); 6]`
