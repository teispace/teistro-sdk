# The SDK against the baseline engine

Status: `generated` by `cargo xtask ab` from `docs/05-testing/ab/*.json`, gated by `check-ab`. Do not edit. Each record is written by `cargo xtask measure-ab`, which runs `bindings/node/bench/ab.mjs` with the baseline engine's adapter (`03-design/ab-baseline.md`).

Both sides answer every birth of the golden corpus in one Node process, the SDK as `parashari-classical` over Teimeris. A birth on which the two Suns stand more than an arcminute apart is reported and kept out, and a record that kept any out is no claim. A request a side refuses is kept out of that request alone, and only at the corpus's range-edge births, where the data's years end and a day's spans run past them. A round times every birth on one side and then the other, the next round the other way round, and a figure is the median of at least 15 rounds, per birth in microseconds. **The claim** is that the SDK is not slower: the gate fails a request where its median is the larger and the two fastest-to-slowest ranges are apart. Where they meet the ratio reads `within spread`.

No request is measurably slower through the SDK.

## `macos-aarch64`

macos aarch64, commit `d9965f0d3`, Node v26.12.0; the sdk `teistro 0.0.0`, the baseline `baseline 0.6.173`.

| request | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |
|---|---:|---:|---:|---:|---:|
| chart (a birth chart with its drishti, upagrahas, houses, dignities, arudhas and Vimshottari to depth 3) | 1232 | 7277 | 0.17× | 1184 to 1813 | 7159 to 7708 |
| day (the almanac day of the birth's date at its place, its limbs, kaalas, choghadiya and horas) | 617 | 11887 | 0.05× | 601 to 624 | 11774 to 12460 |

Refused, and kept out of that request's timing:

- c047, `day`, by the sdk: MOON at JD 2378496.179673321: OutOfRange
- c048, `day`, by the sdk: MOON at JD 2597641.5517241377: OutOfRange
