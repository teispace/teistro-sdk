# Mutation testing, measured

Status: `generated` by `cargo xtask mutants --from DIR` from the `mutants` workflow's run; `check-mutants` holds this page to [`mutants.json`](mutants.json), and the workflow holds every run to it.

Each kernel crate's floor is the share of its mutants that its tests catch, rounded down to a whole percent; a mutant that does not build is not counted, and one listed in `.cargo/mutants.toml`, with why it cannot change what the code computes, is never made ([`01-quality-bar.md`](01-quality-bar.md), "mutation testing"). A floor only rises. Every survivor is a missing test, and a run fails on one this page does not name, so the list below only shrinks. The quality bar asks 80% of a kernel crate; 1 of the 1 have reached it.

| crate | floor | at 80% | survivors |
|---|---:|---|---:|
| `vargas` | 100% | yes | 0 |
