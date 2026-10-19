/// The Teimeris ephemeris in a Flutter app.
///
/// The build hook bundles the adapter, built with the engine's data inside
/// it, for the ABI or SDK the app is built for; [teimeris] opens it with no
/// path, no files copied out and nothing hashed at start.
///
/// ```dart
/// import 'package:teistro/teistro.dart';
/// import 'package:teistro_ephemeris_teimeris_flutter/teistro_ephemeris_teimeris_flutter.dart';
///
/// final sdk = Teistro.open();
/// // The modern positions:
/// final drik = sdk.context(ephemeris: [teimeris()]);
/// // The Nepali committee's reckoning: the limbs by the Surya Siddhanta,
/// // sunrise and the outer planets from Teimeris.
/// final committee = sdk.context(
///   profile: 'nepali-committee',
///   ephemeris: [const NamedEphemeris(Ephemeris.suryaSiddhanta), teimeris()],
/// );
/// ```
///
/// **This package is AGPL-3.0-only**, because the library it bundles links
/// Teimeris (ADR-0029); the SDK stays Apache-2.0 and loads it at run time.
library;

import 'dart:io';

import 'package:teistro/teistro.dart' show PluginEphemeris;

import 'package:teistro_ephemeris_teimeris/teistro_ephemeris_teimeris.dart'
    as desktop;

import 'src/release.dart' show adapterStem;

/// The engine's own operations, typed, as the desktop package offers them.
export 'package:teistro_ephemeris_teimeris/engine.dart';

/// The Teimeris engine over the data built into the adapter.
///
/// `profile` is the engine's own accuracy profile (`compatible`, `max`).
/// `path` names the adapter's library outright, for a caller who bundles it
/// another way; otherwise the one this package's hook bundled is opened.
PluginEphemeris teimeris({String? profile, String? path}) => PluginEphemeris(
  plugin: path ?? bundledAdapter(),
  config: <String, Object?>{'data': 'embedded', 'profile': ?profile},
);

/// The adapter's library as the platform's loader opens it from inside the
/// app, for [operatingSystem] (`Platform.operatingSystem`'s spelling).
///
/// On Android, the bare file name: the hook bundles it into the app's
/// native library directory, which the loader searches for the SDK's own
/// library and therefore for a library the SDK opens. On iOS, Flutter wraps
/// it in a framework inside the app's `Frameworks` directory; the absolute
/// path is given when it is there, because a relative one would be read
/// from the working directory, and the framework's own name otherwise. On
/// any other system, the desktop package's search (`TEISTRO_TEIMERIS_ADAPTER`,
/// then beside the program).
String bundledAdapter({
  String? operatingSystem,
  String? executable,
  bool Function(String path)? exists,
}) {
  final os = operatingSystem ?? Platform.operatingSystem;
  final present = exists ?? (String path) => File(path).existsSync();
  switch (os) {
    case 'android':
      return 'lib$adapterStem.so';
    case 'ios':
      final framework = '$adapterStem.framework/$adapterStem';
      final app = File(executable ?? Platform.resolvedExecutable).parent.path;
      final absolute = '$app/Frameworks/$framework';
      return present(absolute) ? absolute : framework;
    default:
      return desktop.binary();
  }
}
