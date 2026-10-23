# What an app pays

Status: `generated` by `cargo xtask app-speed` from `docs/05-testing/app-speed/*.json`, gated by `check-app-speed`. Do not edit. Each record is written by `teistro-ephemeris-teimeris-app-speed` (`adapters/ephemeris-teimeris/rust`), which times the calls an almanac or chart screen makes through the Rust façade, each on a fresh context so that the call memo starts empty.

The **budget** is the performance page's (`02-architecture/09-performance-architecture.md`), written for a native provider: the gate fails a median past 2× its budget under either setup over Teimeris, because wall time is noisy and a tight ceiling would fail on a busy runner rather than on a regression. The **target** is the faster figure a consumer's app needs on a low-end phone, scaled to a laptop core, and is reported, not gated. The built-in ephemeris needs no engine and is held to accuracy, not to the native budget; its ratio to Teimeris is the column on the right.

```text
cd adapters/ephemeris-teimeris/rust
cargo run --release --features app-speed \
  --bin teistro-ephemeris-teimeris-app-speed \
  > ../../../docs/05-testing/app-speed/desktop-$(uname -s)-$(uname -m).json
cargo xtask app-speed
```

Every gated median is inside 2× its budget, and every thread count answers alike and keeps its floor.

## `android-arm64-android-emulator-sdk-gphone64-arm64`

Android emulator, `sdk_gphone64_arm64` (Android 15, API 35, aarch64), commit `34f88dcb`, Teimeris 0.1.0, its data built in. Memo: cold: a fresh context every run.

| request | budget | target | drik, Teimeris | committee, Surya Siddhanta + Teimeris | drik, built-in | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|---:|---:|
| open (a context opened, the engine with it) | — | 3.00 ms | 0.018 | 0.021 | 0.014 | 0.8× |
| day (one almanac day) | 2.00 ms | 0.500 ms | 1.54 | 0.703 | 31.0 | 20.2× |
| month (a month grid, 42 days in one call) | 60.0 ms | 15.0 ms | 34.3 | 18.7 | 626 | 18.3× |
| year (a year of almanac days) | — | — | 295 | 170 | 5323 | 18.0× |
| festivals (a year of the Nepal pack's festivals) | — | 80.0 ms | 251 | 124 | 5320 | 21.2× |
| muhurta (a 90-day marriage muhurta search) | 200 ms | — | 182 | 90.3 | 2768 | 15.2× |
| chart (a birth chart with its Vimshottari to depth 3) | 10.0 ms | — | 1.37 | 1.38 | 2.66 | 1.9× |
| matching (Ashta Koota between two cast charts) | — | — | 0.023 | 0.024 | 0.023 | 1.0× |

Medians in milliseconds. The spread of every row:

| setup | request | runs | fastest | slowest |
|---|---|---:|---:|---:|
| drik-teimeris | open | 21 | 0.017 | 0.029 |
| drik-teimeris | day | 21 | 1.49 | 1.65 |
| drik-teimeris | month | 21 | 32.6 | 37.5 |
| drik-teimeris | year | 14 | 288 | 309 |
| drik-teimeris | festivals | 16 | 245 | 265 |
| drik-teimeris | muhurta | 21 | 175 | 187 |
| drik-teimeris | chart | 21 | 1.34 | 1.44 |
| drik-teimeris | matching | 21 | 0.023 | 0.051 |
| committee-teimeris | open | 21 | 0.019 | 0.024 |
| committee-teimeris | day | 21 | 0.693 | 0.763 |
| committee-teimeris | month | 21 | 18.4 | 21.5 |
| committee-teimeris | year | 21 | 166 | 198 |
| committee-teimeris | festivals | 21 | 121 | 131 |
| committee-teimeris | muhurta | 21 | 86.2 | 95.0 |
| committee-teimeris | chart | 21 | 1.33 | 1.88 |
| committee-teimeris | matching | 21 | 0.023 | 0.036 |
| drik-builtin | open | 21 | 0.013 | 0.088 |
| drik-builtin | day | 21 | 30.3 | 32.8 |
| drik-builtin | month | 7 | 611 | 629 |
| drik-builtin | year | 3 | 5320 | 5352 |
| drik-builtin | festivals | 3 | 5297 | 5344 |
| drik-builtin | muhurta | 3 | 2766 | 2790 |
| drik-builtin | chart | 21 | 2.58 | 3.09 |
| drik-builtin | matching | 21 | 0.023 | 0.033 |

Contexts on threads: not measured in this record.

## `desktop-Darwin-arm64`

Apple M2 Pro (Darwin 27.0.0, aarch64), commit `0c5a49b1`, Teimeris 0.1.0, its data built in. Memo: cold: a fresh context every run.

| request | budget | target | drik, Teimeris | committee, Surya Siddhanta + Teimeris | drik, built-in | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|---:|---:|
| open (a context opened, the engine with it) | — | 3.00 ms | 0.021 | 0.022 | 0.013 | 0.6× |
| day (one almanac day) | 2.00 ms | 0.500 ms | 1.09 | 0.554 | 2.24 | 2.1× |
| month (a month grid, 42 days in one call) | 60.0 ms | 15.0 ms | 16.8 | 10.8 | 34.8 | 2.1× |
| year (a year of almanac days) | — | — | 150 | 101 | 328 | 2.2× |
| festivals (a year of the Nepal pack's festivals) | — | 80.0 ms | 54.6 | 22.9 | 126 | 2.3× |
| muhurta (a 90-day marriage muhurta search) | 200 ms | — | 138 | 106 | 503 | 3.6× |
| chart (a birth chart with its Vimshottari to depth 3) | 10.0 ms | — | 0.521 | 0.522 | 0.987 | 1.9× |
| matching (Ashta Koota between two cast charts) | — | — | 0.012 | 0.012 | 0.012 | 1.0× |

Medians in milliseconds. The spread of every row:

| setup | request | runs | fastest | slowest |
|---|---|---:|---:|---:|
| drik-teimeris | open | 21 | 0.019 | 0.035 |
| drik-teimeris | day | 21 | 1.05 | 1.27 |
| drik-teimeris | month | 21 | 16.3 | 18.3 |
| drik-teimeris | year | 21 | 147 | 173 |
| drik-teimeris | festivals | 21 | 53.4 | 78.8 |
| drik-teimeris | muhurta | 21 | 136 | 166 |
| drik-teimeris | chart | 21 | 0.512 | 0.611 |
| drik-teimeris | matching | 21 | 0.012 | 0.015 |
| committee-teimeris | open | 21 | 0.021 | 0.027 |
| committee-teimeris | day | 21 | 0.526 | 0.667 |
| committee-teimeris | month | 21 | 10.4 | 11.5 |
| committee-teimeris | year | 21 | 98.9 | 116 |
| committee-teimeris | festivals | 21 | 22.0 | 23.6 |
| committee-teimeris | muhurta | 21 | 104 | 131 |
| committee-teimeris | chart | 21 | 0.506 | 0.657 |
| committee-teimeris | matching | 21 | 0.012 | 0.021 |
| drik-builtin | open | 21 | 0.013 | 0.017 |
| drik-builtin | day | 21 | 2.21 | 2.62 |
| drik-builtin | month | 21 | 34.1 | 59.1 |
| drik-builtin | year | 13 | 325 | 353 |
| drik-builtin | festivals | 21 | 124 | 149 |
| drik-builtin | muhurta | 8 | 500 | 530 |
| drik-builtin | chart | 21 | 0.946 | 1.11 |
| drik-builtin | matching | 21 | 0.012 | 0.015 |

Contexts on threads, the `month` request's grid on each, a context a thread, of the 10 threads this machine offers. **Efficiency** is the throughput over the one thread's times the count (1 is linear); up to 4 threads it is held to 0.75. **Alike** is whether every thread's month hashed as one thread's did.

| setup | threads | median trial, ms | months a second | efficiency | alike |
|---|---:|---:|---:|---:|---|
| drik-teimeris | 1 | 33.9 (5 trials of 3 months) | 88.45 | 1.000 | yes |
| drik-teimeris | 2 | 35.4 (5 trials of 3 months) | 169.46 | 0.958 | yes |
| drik-teimeris | 4 | 36.4 (5 trials of 3 months) | 329.60 | 0.932 | yes |
| drik-teimeris | 8 | 46.3 (5 trials of 3 months) | 518.67 | 0.733 | yes |
| drik-teimeris | 10 | 56.9 (5 trials of 3 months) | 527.34 | 0.596 | yes |
| committee-teimeris | 1 | 32.6 (5 trials of 3 months) | 91.89 | 1.000 | yes |
| committee-teimeris | 2 | 34.4 (5 trials of 3 months) | 174.54 | 0.950 | yes |
| committee-teimeris | 4 | 35.9 (5 trials of 3 months) | 334.07 | 0.909 | yes |
| committee-teimeris | 8 | 46.8 (5 trials of 3 months) | 513.34 | 0.698 | yes |
| committee-teimeris | 10 | 52.4 (5 trials of 3 months) | 572.01 | 0.622 | yes |
| drik-builtin | 1 | 105 (5 trials of 3 months) | 28.70 | 1.000 | yes |
| drik-builtin | 2 | 106 (5 trials of 3 months) | 56.78 | 0.989 | yes |
| drik-builtin | 4 | 108 (5 trials of 3 months) | 111.06 | 0.967 | yes |
| drik-builtin | 8 | 143 (5 trials of 3 months) | 167.68 | 0.730 | yes |
| drik-builtin | 10 | 156 (5 trials of 3 months) | 192.40 | 0.670 | yes |

## `ios-sim-arm64-ios-simulator-iphone-18-pro`

iOS simulator, iPhone 18 Pro (iOS 27.0, aarch64), commit `34f88dcb`, Teimeris 0.1.0, its data built in. Memo: cold: a fresh context every run.

| request | budget | target | drik, Teimeris | committee, Surya Siddhanta + Teimeris | drik, built-in | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|---:|---:|
| open (a context opened, the engine with it) | — | 3.00 ms | 0.024 | 0.022 | 0.013 | 0.5× |
| day (one almanac day) | 2.00 ms | 0.500 ms | 1.89 | 0.721 | 30.7 | 16.2× |
| month (a month grid, 42 days in one call) | 60.0 ms | 15.0 ms | 33.3 | 19.6 | 633 | 19.0× |
| year (a year of almanac days) | — | — | 294 | 168 | 5247 | 17.8× |
| festivals (a year of the Nepal pack's festivals) | — | 80.0 ms | 255 | 129 | 5260 | 20.7× |
| muhurta (a 90-day marriage muhurta search) | 200 ms | — | 179 | 92.5 | 2733 | 15.3× |
| chart (a birth chart with its Vimshottari to depth 3) | 10.0 ms | — | 1.40 | 1.38 | 2.75 | 2.0× |
| matching (Ashta Koota between two cast charts) | — | — | 0.016 | 0.016 | 0.017 | 1.1× |

Medians in milliseconds. The spread of every row:

| setup | request | runs | fastest | slowest |
|---|---|---:|---:|---:|
| drik-teimeris | open | 21 | 0.023 | 0.045 |
| drik-teimeris | day | 21 | 1.72 | 2.25 |
| drik-teimeris | month | 21 | 32.2 | 40.6 |
| drik-teimeris | year | 14 | 284 | 309 |
| drik-teimeris | festivals | 16 | 239 | 272 |
| drik-teimeris | muhurta | 21 | 172 | 187 |
| drik-teimeris | chart | 21 | 1.34 | 1.53 |
| drik-teimeris | matching | 21 | 0.014 | 0.093 |
| committee-teimeris | open | 21 | 0.021 | 0.028 |
| committee-teimeris | day | 21 | 0.702 | 0.894 |
| committee-teimeris | month | 21 | 18.4 | 24.7 |
| committee-teimeris | year | 21 | 163 | 184 |
| committee-teimeris | festivals | 21 | 123 | 146 |
| committee-teimeris | muhurta | 21 | 89.7 | 97.8 |
| committee-teimeris | chart | 21 | 1.32 | 1.58 |
| committee-teimeris | matching | 21 | 0.015 | 0.023 |
| drik-builtin | open | 21 | 0.013 | 0.015 |
| drik-builtin | day | 21 | 29.6 | 35.8 |
| drik-builtin | month | 7 | 607 | 650 |
| drik-builtin | year | 3 | 5226 | 5283 |
| drik-builtin | festivals | 3 | 5240 | 5314 |
| drik-builtin | muhurta | 3 | 2720 | 2762 |
| drik-builtin | chart | 21 | 2.60 | 3.04 |
| drik-builtin | matching | 21 | 0.014 | 0.030 |

Contexts on threads: not measured in this record.
