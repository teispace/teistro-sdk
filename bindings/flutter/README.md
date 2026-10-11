# teistro_flutter

The [Teistro SDK](https://pub.dev/packages/teistro) in a Flutter app on
Android and iOS. This package holds a build hook and nothing else. When the
app is built, the hook adds the SDK's native library for each Android ABI
and iOS SDK the build targets. It fetches each one from the release this
package was cut from and checks it against the SHA-256 digest this package
carries before writing it.

```yaml
dependencies:
  teistro_flutter: ^0.1.0
```

```dart
import 'package:teistro_flutter/teistro_flutter.dart';

final teistro = Teistro.open(); // the library the hook bundled
```

`package:teistro_flutter/teistro_flutter.dart` re-exports
`package:teistro/teistro.dart`, so the whole API is available through this
one dependency. You do not need to run `dart run teistro:install`.

## What it bundles

| build | release slice | floor |
|---|---|---|
| Android arm64-v8a | `android-arm64` | API 24, 16 KB pages |
| Android armeabi-v7a | `android-arm` | API 24 |
| Android x86_64 | `android-x64` | API 24, 16 KB pages |
| iOS device | `ios-arm64` | iOS 13.0 |
| iOS simulator (Apple silicon) | `ios-sim-arm64` | iOS 14.0 |
| iOS simulator (Intel) | `ios-sim-x64` | iOS 13.0 |

The build is refused, with a sentence saying how to fix it, when:

- the app's `minSdk` is below 24 or its iOS deployment target is below 13;
- the build targets an ABI the release does not ship (ia32 or RISC-V
  Android);
- a downloaded library does not match its digest.

Desktop Flutter apps and Dart programs get nothing bundled. They find the
library the way `package:teistro` describes.

Each library is linked against the system libraries alone (no
`libc++_shared.so`). It is unsigned; Flutter signs it with the app's
identity.

## Offline builds and your own library

Set these under `hooks: user_defines: teistro_flutter:` in the app's (or
the workspace's) `pubspec.yaml`:

```yaml
hooks:
  user_defines:
    teistro_flutter:
      # The release's .gz archives, read from a directory rather than
      # fetched. Each is still checked against its digest.
      prebuilt_dir: third_party/teistro
      # Or a library you built yourself, bundled without a digest check.
      # The SDK still checks at load that it is this version's build.
      # library: ../teistro-sdk/target/aarch64-linux-android/release/libteistro_ffi.so
```

Downloads honour `HTTPS_PROXY` and `NO_PROXY`. The hook caches each slice
in Flutter's shared build directory and reuses it while its digest still
matches.
