# `teistro-research`

The statistics under `sdk.research()`, and nothing that reads a sky.
A study's charts are evaluated once into a chart-by-predicate matrix;
this crate permutes the study's labels over that matrix and answers,
for every predicate, a permutation p-value with its Monte Carlo
interval, the family's adjusted p-values (Westfall–Young max-T, Holm,
Bonferroni, Benjamini–Hochberg, Benjamini–Yekutieli) and the effect
sizes with their intervals.

The generator (SplitMix64, keyed per permutation) and the shuffle
(Fisher–Yates with Lemire's bounded integer) are written here rather
than taken from a dependency, because they are part of the answer: the
same request gives the same bits on every platform and at any thread
count, and the answer names the shuffle's version.

The design is `docs/03-design/research.md`.
