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

Every gated median is inside 2× its budget.

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

## `desktop-Darwin-arm64`

Apple M2 Pro (Darwin 27.0.0, aarch64), commit `0d579058`, Teimeris 0.1.0, its data built in. Memo: cold: a fresh context every run.

| request | budget | target | drik, Teimeris | committee, Surya Siddhanta + Teimeris | drik, built-in | built-in ÷ Teimeris |
|---|---:|---:|---:|---:|---:|---:|
| open (a context opened, the engine with it) | — | 3.00 ms | 0.021 | 0.018 | 0.013 | 0.6× |
| day (one almanac day) | 2.00 ms | 0.500 ms | 1.72 | 0.651 | 32.7 | 19.0× |
| month (a month grid, 42 days in one call) | 60.0 ms | 15.0 ms | 21.5 | 13.6 | 463 | 21.5× |
| year (a year of almanac days) | — | — | 193 | 129 | 3806 | 19.7× |
| festivals (a year of the Nepal pack's festivals) | — | 80.0 ms | 158 | 90.2 | 3902 | 24.6× |
| muhurta (a 90-day marriage muhurta search) | 200 ms | — | 189 | 88.2 | 3056 | 16.2× |
| chart (a birth chart with its Vimshottari to depth 3) | 10.0 ms | — | 1.22 | 1.18 | 2.60 | 2.1× |
| matching (Ashta Koota between two cast charts) | — | — | 0.012 | 0.012 | 0.013 | 1.1× |

Medians in milliseconds. The spread of every row:

| setup | request | runs | fastest | slowest |
|---|---|---:|---:|---:|
| drik-teimeris | open | 21 | 0.017 | 0.209 |
| drik-teimeris | day | 21 | 1.60 | 1.98 |
| drik-teimeris | month | 21 | 20.7 | 22.5 |
| drik-teimeris | year | 21 | 190 | 198 |
| drik-teimeris | festivals | 21 | 154 | 164 |
| drik-teimeris | muhurta | 21 | 185 | 213 |
| drik-teimeris | chart | 21 | 1.19 | 1.36 |
| drik-teimeris | matching | 21 | 0.012 | 0.019 |
| committee-teimeris | open | 21 | 0.018 | 0.021 |
| committee-teimeris | day | 21 | 0.606 | 0.795 |
| committee-teimeris | month | 21 | 13.2 | 14.5 |
| committee-teimeris | year | 21 | 125 | 136 |
| committee-teimeris | festivals | 21 | 87.9 | 117 |
| committee-teimeris | muhurta | 21 | 86.0 | 110 |
| committee-teimeris | chart | 21 | 1.15 | 1.36 |
| committee-teimeris | matching | 21 | 0.012 | 0.025 |
| drik-builtin | open | 21 | 0.013 | 0.016 |
| drik-builtin | day | 21 | 31.9 | 33.9 |
| drik-builtin | month | 9 | 455 | 477 |
| drik-builtin | year | 3 | 3804 | 3809 |
| drik-builtin | festivals | 3 | 3889 | 3923 |
| drik-builtin | muhurta | 3 | 3055 | 3068 |
| drik-builtin | chart | 21 | 2.56 | 2.66 |
| drik-builtin | matching | 21 | 0.012 | 0.038 |

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
