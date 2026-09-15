# `teistro-dasha`

Status: `building`, 2026-09-15: the nine nakshatra-seeded systems built and
measured — Vimshottari, Ashtottari, Dwadashottari, Panchottari, Shatabdika,
Chaturashiti-sama, Dwisaptati-sama, Yogini and Tribhagi. The design is
[`docs/03-design/dasha-kernels.md`](../../docs/03-design/dasha-kernels.md),
measured in [`dasha-measured.md`](../../docs/03-design/dasha-measured.md) and
[`dasha-systems-measured.md`](../../docs/03-design/dasha-systems-measured.md).

Dashas as rows over a kernel, the balance at birth, and the period tree read
without building it.

| module | what it settles |
|---|---|
| [`row`](src/row.rs) | a nakshatra-seeded system as data: its lords and years, the map from the Moon's nakshatra to its first lord (reference, direction, window, offset, whether the lords repeat round the nakshatras), and a scale on the mahadashas with the rounds a cycle runs; the nine shipped rows |
| [`balance`](src/balance.rs) | what remains of the lord's window, by how far into its own nakshatra the Moon is spatially or temporally, and its written form with the minutes rounded |
| [`tree`](src/tree.rs) | the dasha of a birth: periods by path, children computed when asked for, and the chain running at an instant without allocating |
| [`reading`](src/reading.rs) | a dasha as a chart document carries it: its rules, seed, balance and Moon span, and its periods as rows to the settings' depth |

## What the corpus settled

- **The birth period's sub-periods are compressed** into its balance, on
  every recorded answer. The other reading is `dasha.birth_period =
  ELAPSED` (crux C48).
- **The cycle ends** after its last lord. `dasha.after_cycle = REPEAT`
  begins it again.
- **Boundaries are exact shares of their parent.** Children partition their
  parent by construction, and nothing accumulates down the tree.
- **Every other system is a row.** The seat, the balance, the compressed
  birth period, the children and the cycle's end are Vimshottari's in all
  eight; only the data differs.
- **A temporal balance reads a window through the Moon's own nakshatra**:
  the whole nakshatras behind it are gone and its own is gone by time, which
  every recorded Ashtottari answer takes.
- **Tribhagi is Vimshottari at two thirds, twice round**; a third three times
  round is refused by every answer.

## What proves it

- Every recorded Vimshottari is reproduced from the recorded Moon: 148
  method answers, 53 415 tree rows, 14 841 sampled deeper children and 296
  running chains, worst boundary 1.4e-9 days, against a corpus tolerance of
  1e-3 (`tests/baseline.rs`).
- Every other system is reproduced from the same recorded inputs: 1184
  answers, 94 128 tree rows and 2368 chains, worst boundary 1.4e-9 days, and
  each row's data equal to the data the corpus states (`tests/systems.rs`).
- Reading the chain at an instant allocates nothing, at the deepest level,
  in a later cycle, and for every row (`tests/allocations.rs`).
- Every row passes its checks; a wrong row is refused by the field; seats,
  balances, partitions, both birth-period readings and both cycle ends are
  unit-tested.
