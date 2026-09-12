/// The Teimeris ephemeris, as a Teistro adapter.
///
/// Two things, and a consumer needs both: [teimeris], the **descriptor**
/// an `ephemeris` chain takes, and the typed façade over the engine's own
/// operations, which is the extension `engine.dart` declares.
///
/// ```dart
/// import 'package:teistro/teistro.dart';
/// import 'package:teistro_ephemeris_teimeris/teistro_ephemeris_teimeris.dart';
///
/// final sdk = Teistro.open();
/// final ctx = sdk.context(ephemeris: [teimeris(dataDir: './ephe')]);
/// ctx.engine.tmBodyName(body: 0); // 'Sun'
/// ```
///
/// **This package is AGPL-3.0-only**, because the library it ships links
/// Teimeris. The SDK is Apache-2.0 and never links it: it loads this
/// adapter at run time, so the licence stays on this side of the
/// boundary (ADR-0029).
library;

import 'dart:ffi' show Abi;
import 'dart:io';

import 'package:teistro/teistro.dart' show PluginEphemeris;

/// The typed façade: an extension on the SDK's own `Engine`, generated
/// from the engine's own description.
export 'engine.dart';

/// The environment variable that names the adapter's platform binary,
/// which wins over every other place it is looked for.
///
/// The same variable `crates/ffi/tests/abi.rs` and every binding's plugin
/// test read, so a contributor sets it once.
const String pathVariable = 'TEISTRO_TEIMERIS_ADAPTER';

/// The file name this platform gives the adapter's shared library.
String get libraryFileName {
  if (Platform.isMacOS) return 'libteistro_ephemeris_teimeris.dylib';
  if (Platform.isWindows) return 'teistro_ephemeris_teimeris.dll';
  return 'libteistro_ephemeris_teimeris.so';
}

/// This host as the release names it, in the same words the SDK's own
/// installer uses so that one release page names one artefact for every
/// binding.
String get hostPlatform {
  final os =
      Platform.isMacOS
          ? 'darwin'
          : Platform.isWindows
          ? 'win32'
          : Platform.operatingSystem;
  final abi = Abi.current().toString();
  final cpu =
      abi.endsWith('arm64')
          ? 'arm64'
          : abi.endsWith('x64')
          ? 'x64'
          : abi;
  return '$os-$cpu';
}

/// Every place [binary] looks, in order.
///
/// The SDK's own `Teistro.searchPath` shape, for the same reason: a
/// consumer only ever has the installed one and a contributor only ever
/// has the build, and the order matters for whoever has both.
List<String> searchPath() {
  final named = Platform.environment[pathVariable];
  final separator = Platform.pathSeparator;
  return <String>[
    if (named != null && named.isNotEmpty) named,
    // Beside this package, which is where a published one puts it.
    '${File.fromUri(Platform.script).parent.path}$separator$libraryFileName',
    'adapters${separator}ephemeris-teimeris${separator}rust${separator}target'
        '${separator}release$separator$libraryFileName',
    'adapters${separator}ephemeris-teimeris${separator}rust${separator}target'
        '${separator}debug$separator$libraryFileName',
    '..$separator..${separator}rust${separator}target${separator}release'
        '$separator$libraryFileName',
    '..$separator..${separator}rust${separator}target${separator}debug'
        '$separator$libraryFileName',
  ];
}

/// The adapter's platform binary.
///
/// Throws a [StateError] naming every place it looked when there is
/// none, because a path a consumer cannot see is a path they cannot fix.
String binary([String? named]) {
  if (named != null) return named;
  final looked = searchPath();
  for (final candidate in looked) {
    if (File(candidate).existsSync()) return candidate;
  }
  throw StateError(
    'no Teimeris adapter for $hostPlatform. Looked in:\n  '
    '${looked.join('\n  ')}\n'
    'Build it with `cargo build --release` in '
    '`adapters/ephemeris-teimeris/rust`, or set $pathVariable to its path.',
  );
}

/// The descriptor an `ephemeris` chain takes (ADR-0029).
///
/// **It fails here** — at the call, in the line that names the engine —
/// when the adapter is not built or installed, rather than when a chart
/// is cast. That is the whole reason a descriptor is a value rather than
/// a string the SDK looks up.
///
/// `dataDir` is where the engine's data files are; it looks beside its
/// own build when omitted. `profile` is the engine's own accuracy
/// profile. `path` names the platform binary, for a caller who would
/// rather say than let this package look.
PluginEphemeris teimeris({String? dataDir, String? profile, String? path}) =>
    PluginEphemeris(
      plugin: binary(path),
      config: <String, Object?>{
        if (dataDir != null) 'data_dir': dataDir,
        if (profile != null) 'profile': profile,
      },
    );
