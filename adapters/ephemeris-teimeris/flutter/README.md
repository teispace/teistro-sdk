# teistro_ephemeris_teimeris_flutter

The Teimeris ephemeris in a Flutter app on Android and iOS, with **no data
files**: the adapter library this package bundles is built with the engine's
data inside it (`TM_EMBED_TIER=MODERN`, 1800 to 2400), so `teimeris()` opens
it with no path, no assets copied out at first start, and nothing hashed at
start-up. Each embedded file's SHA-256 was computed when the engine was
built, and the provenance a result carries is the same as a directory of
the same files would give.

**This package is AGPL-3.0-only**, because the library it bundles links
Teimeris (ADR-0029). The SDK it extends stays Apache-2.0 and loads the
adapter at run time.

## Use

```yaml
# pubspec.yaml
dependencies:
  teistro_flutter: ...                       # the SDK's own library
  teistro_ephemeris_teimeris_flutter: ...    # this package
```

```dart
import 'package:teistro/teistro.dart';
import 'package:teistro_ephemeris_teimeris_flutter/teistro_ephemeris_teimeris_flutter.dart';

final sdk = Teistro.open();

// The modern positions.
final drik = sdk.context(ephemeris: [teimeris()]);

// Nepal's official days: the committee's reckoning, the tithi,
// nakshatra and yoga by the Surya Siddhanta with the committee's bija,
// the sunrise and the outer planets from Teimeris.
final official = sdk.nepalOfficial(modern: [teimeris()], locale: 'ne-Deva-NP');
```

`nepalOfficial` is `profile: 'nepali-committee'` over the chain
`[NamedEphemeris(Ephemeris.suryaSiddhanta), teimeris()]`, which
`sdk.context` takes as well: the Surya Siddhanta takes the adapter after it
for its sunrise. Where the two setups differ on a day, the committee's is
the one Nepal's official calendar follows.

`teimeris(profile: 'max')` asks for the engine's corrected profile, and
`teimeris(path: ...)` opens a library you bundled another way.

## Which ephemeris

| | Teimeris (this package) | the SDK's built-in |
|---|---|---|
| licence of what ships | AGPL-3.0, or the Teimeris Professional Licence | Apache-2.0 |
| an almanac day, M2 Pro | 1.0 to 2.0 ms | 45 ms |
| a month grid (42 days) | 34 to 60 ms | 1.5 s |
| data files | none, built in | none |

Measured with a cold call cache (`docs/05-testing/app-speed-measured.md`,
which holds the current numbers and the gate on them). Choose the built-in
when the app must stay Apache-only; choose Teimeris when it needs the speed.

## How the library is found

The hook bundles the adapter the way `teistro_flutter` bundles the SDK, with
the same platforms (`android-arm64`, `android-arm`, `android-x64`,
`ios-arm64`, `ios-sim-arm64`, `ios-sim-x64`), the same floors (Android API
24, iOS 13) and the same user defines, under this package's name:

```yaml
hooks:
  user_defines:
    teistro_ephemeris_teimeris_flutter:
      # A directory of release archives, for an offline build:
      prebuilt_dir: ../archives
      # Or the libraries you built, one per platform:
      library:
        android-arm64: ../build/android-arm64/libteistro_ephemeris_teimeris.so
        ios-arm64: ../build/ios-arm64/libteistro_ephemeris_teimeris.dylib
```

At run time, Android's loader opens it by name from the app's native
library directory, and on iOS it is opened from the framework Flutter
wraps it in, inside the app's `Frameworks` directory.

Nothing is released yet, so the package carries no digests and an app
names the libraries it built with `library`. Build one with
`cargo build --release` in `adapters/ephemeris-teimeris/rust` for the
target (`--target aarch64-linux-android` with the NDK, `aarch64-apple-ios`
with Xcode).
