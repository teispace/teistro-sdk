# `teistro-dasha`

Status: `building`, 2026-09-15: Vimshottari built and measured. The design
is [`docs/03-design/dasha-kernels.md`](../../docs/03-design/dasha-kernels.md),
measured in [`docs/03-design/dasha-measured.md`](../../docs/03-design/dasha-measured.md).

Dashas as rows over a kernel, the balance at birth, and the period tree read
without building it.

| module | what it settles |
|---|---|
| [`row`](src/row.rs) | a nakshatra-seeded system as data: its lords and years, and the map from the Moon's nakshatra to its first lord, with the window a lord covers and whether the lords repeat round the nakshatras |
| [`balance`](src/balance.rs) | what remains of the first period, spatially from the exact nakshatra position or temporally from the Moon's span, and its written form with the minutes rounded |
| [`tree`](src/tree.rs) | the dasha of a birth: periods by path, children computed when asked for, and the chain running at an instant without allocating |

## What the corpus settled

- **The birth period's sub-periods are compressed** into its balance, on
  every recorded answer. The other reading is `dasha.birth_period =
  ELAPSED` (crux C48).
- **The cycle ends** after its last lord. `dasha.after_cycle = REPEAT`
  begins it again.
- **Boundaries are exact shares of their parent.** Children partition their
  parent by construction, and nothing accumulates down the tree.

## What proves it

- Every recorded Vimshottari is reproduced from the recorded Moon: 148
  method answers, 53 415 tree rows, 14 841 sampled deeper children and 296
  running chains, worst boundary 1.4e-9 days, against a corpus tolerance of
  1e-3 (`tests/baseline.rs`).
- Reading the chain at an instant allocates nothing, at the deepest level
  and in a later cycle (`tests/allocations.rs`).
- Every row passes its checks; a wrong row is refused by the field; seats,
  balances, partitions, both birth-period readings and both cycle ends are
  unit-tested.
