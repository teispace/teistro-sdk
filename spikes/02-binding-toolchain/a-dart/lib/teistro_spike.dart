/// Spike 2, option A: the ergonomic layer of the Dart binding.
///
/// HAND-WRITTEN over `src/generated.dart`, which is the mechanical
/// contract generated from the same description as the Node glue. This
/// file adds what a generator cannot know yet: where the library is,
/// validated extension types (ADR-0023), defaults, and a `Chart` that
/// decodes its blob lazily.
library;

import 'dart:ffi' as ffi;
import 'dart:io';
import 'dart:typed_data';

import 'src/generated.dart';

export 'src/generated.dart';

/// Opens the C ABI library and checks its ABI version against the one this
/// layer was generated for. Without `path`, the workspace's release build
/// is found by walking up from the working directory.
TspLibrary openLibrary({String? path}) {
  final lib = TspLibrary(ffi.DynamicLibrary.open(path ?? _workspaceBuild()));
  final version = abiVersion(lib);
  if (version != generatedAbiVersion) {
    throw StateError('library ABI $version, this layer expects $generatedAbiVersion');
  }
  return lib;
}

String _workspaceBuild() {
  final name = switch (Platform.operatingSystem) {
    'macos' => 'libteistro_spike_a_ffi.dylib',
    'windows' => 'teistro_spike_a_ffi.dll',
    _ => 'libteistro_spike_a_ffi.so',
  };
  var dir = Directory.current;
  for (var up = 0; up < 8; up++) {
    final candidate = File('${dir.path}/target/release/$name');
    if (candidate.existsSync()) return candidate.path;
    final parent = dir.parent;
    if (parent.path == dir.path) break;
    dir = parent;
  }
  throw StateError('no $name under a target/release directory above ${Directory.current.path}');
}

/// A Julian Day in UT that is finite; the type says so, the factory checks.
extension type const JulianDay._(double value) {
  /// Validates: finite, or an [ArgumentError].
  factory JulianDay(double value) {
    if (!value.isFinite) throw ArgumentError.value(value, 'value', 'must be finite');
    return JulianDay._(value);
  }
}

/// A dasha depth from 1 to 5; the type says so, the factory checks.
extension type const DashaDepth._(int value) {
  /// Validates: 1 to 5, or a [RangeError].
  factory DashaDepth(int value) {
    if (value < 1 || value > 5) throw RangeError.range(value, 1, 5, 'value');
    return DashaDepth._(value);
  }

  /// Three levels, the default.
  static const DashaDepth three = DashaDepth._(3);
}

/// One classified position as a plain object.
final class BodyPosition {
  const BodyPosition({
    required this.body,
    required this.longitudeDeg,
    required this.longitudeNas,
    required this.latitudeDeg,
    required this.speedDegPerDay,
    required this.sign,
    required this.nakshatra,
    required this.pada,
    required this.retrograde,
  });

  final Body body;
  final double longitudeDeg;
  final int longitudeNas;
  final double latitudeDeg;
  final double speedDegPerDay;
  final int sign;
  final int nakshatra;
  final int pada;
  final bool retrograde;
}

/// A computed chart: the bytes, decoded on first use.
final class Chart {
  Chart(this.bytes);

  /// The bytes the native side returned.
  final Uint8List bytes;
  DecodedChart? _decoded;

  /// The columns, decoded once as views over [bytes].
  DecodedChart get decoded => _decoded ??= decodeChart(bytes);

  /// The instant the chart was computed for.
  double get jdUt => decoded.jdUt;

  /// The number of tree nodes.
  int get nodeCount => decoded.nodeCount;

  /// The nine positions as plain objects, built on demand.
  List<BodyPosition> positions() {
    final p = decoded.positions;
    return List<BodyPosition>.generate(
      p.length,
      (i) => BodyPosition(
        body: Body.fromValue(p.body[i]),
        longitudeDeg: p.longitudeDeg[i],
        longitudeNas: p.longitudeNas[i],
        latitudeDeg: p.latitudeDeg[i],
        speedDegPerDay: p.speedDegPerDay[i],
        sign: p.sign[i],
        nakshatra: p.nakshatra[i],
        pada: p.pada[i],
        retrograde: p.retrograde[i] == 1,
      ),
      growable: false,
    );
  }

  /// The tree as nested objects, all of it.
  List<DashaNode> tree() => dashaTree(decoded);
}

/// A context with defaults and validated inputs over the generated [Context].
final class TeistroContext {
  TeistroContext(
    TspLibrary lib, {
    Ayanamsha ayanamsha = Ayanamsha.lahiri,
    NodeKind node = NodeKind.mean,
    DashaDepth dashaDepth = DashaDepth.three,
    PositionProvider? provider,
  }) : _inner = Context(
          lib,
          Settings(ayanamsha: ayanamsha, node: node, dashaDepth: dashaDepth.value),
          provider: provider,
        );

  final Context _inner;

  /// The one batch call.
  Chart computeChart(JulianDay jdUt) => Chart(_inner.chartCompute(jdUt.value));

  /// The settings the context was built with.
  Settings settings() => _inner.settings();

  /// The provider's code from the last failure, `0` when there was none.
  int lastProviderCode() => _inner.lastProviderCode();

  /// Frees the native context.
  void dispose() => _inner.dispose();
}
