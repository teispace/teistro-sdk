// The Dart row of `docs/03-design/ffi-cost.md`: the scenario every binding
// times, printed as one JSON line a key. `cargo xtask measure-ffi-cost`
// runs it with `TEISTRO_LIBRARY` naming the release library.
//
// `call-*` asks for 1 instant and 3 bodies, `grid-*` for 366 instants and
// 10 bodies. `*-native` is a context over the library's test provider and
// `*-hosted` one over a provider written here, answering fixed columns.
// Each call reads the answer's grid size. `decode` is the decoder alone,
// on the grid's bytes.

import 'dart:convert';
import 'dart:typed_data';

import 'package:teistro/teistro.dart';

const _runMicroseconds = 50000;
const _runs = 5;

const _bodies = <Body>[
  Body.sun,
  Body.moon,
  Body.mercury,
  Body.venus,
  Body.mars,
  Body.jupiter,
  Body.saturn,
  Body.meanNode,
  Body.uranus,
  Body.neptune,
];

/// A provider written in Dart: every cell a fixed longitude.
final class _Fixed extends EphemerisProvider {
  const _Fixed();

  @override
  String get name => 'ffi-cost-in-dart';

  @override
  List<Body> get bodies => _bodies;

  @override
  String get version => '1';

  @override
  PositionAnswer positions(PositionQuery query) {
    final cells = query.cellCount;
    return PositionAnswer(
      lon: Float64List(cells)..fillRange(0, cells, 123.25),
      lat: Float64List(cells),
      dist: Float64List(cells)..fillRange(0, cells, 1),
      lonSpeed: Float64List(cells)..fillRange(0, cells, 1),
    );
  }
}

void _time(String key, int Function() call) {
  var sink = 0;
  for (var i = 0; i < 100; i++) {
    sink += call();
  }
  final clock = Stopwatch();
  var calls = 1;
  for (;;) {
    clock
      ..reset()
      ..start();
    for (var i = 0; i < calls; i++) {
      sink += call();
    }
    clock.stop();
    if (clock.elapsedMicroseconds >= _runMicroseconds || calls >= 1 << 24) {
      break;
    }
    calls *= 2;
  }
  final runs = <double>[];
  for (var r = 0; r < _runs; r++) {
    clock
      ..reset()
      ..start();
    for (var i = 0; i < calls; i++) {
      sink += call();
    }
    clock.stop();
    runs.add(clock.elapsedTicks * 1e6 / clock.frequency / calls);
  }
  if (sink == 0) throw StateError('nothing was answered');
  runs.sort();
  print(
    jsonEncode({
      'binding': 'dart',
      'key': key,
      'runs': _runs,
      'calls': calls,
      'median_us': runs[_runs >> 1],
      'min_us': runs.first,
      'max_us': runs.last,
    }),
  );
}

void main() {
  final teistro = Teistro.open();
  final native = teistro.context(profile: 'nepali-drik', testProvider: true);
  final hosted = teistro.context(
    profile: 'nepali-drik',
    provider: const _Fixed(),
  );
  try {
    const one = <double>[2451545];
    const three = <Body>[Body.sun, Body.moon, Body.mars];
    final year = <double>[for (var day = 0; day < 366; day++) 2451545.0 + day];
    // The hosted provider's answer is what it sent, so a figure is never
    // taken over a call that failed or answered something else.
    final answered = hosted.positions(instants: year, bodies: _bodies);
    if (answered.jdCount * answered.bodyCount != 3660 ||
        answered.cells.lon[3659] != 123.25) {
      throw StateError(
        'the hosted provider did not answer the grid it was asked',
      );
    }
    _time(
      'call-native',
      () => native.positions(instants: one, bodies: three).jdCount,
    );
    _time(
      'call-hosted',
      () => hosted.positions(instants: one, bodies: three).jdCount,
    );
    _time(
      'grid-native',
      () => native.positions(instants: year, bodies: _bodies).jdCount,
    );
    _time(
      'grid-hosted',
      () => hosted.positions(instants: year, bodies: _bodies).jdCount,
    );
    // A decoded grid's columns are views over the blob it was read from,
    // which the binding copied into a buffer of its own.
    final bytes =
        native
            .positions(instants: year, bodies: _bodies)
            .instants
            .jd
            .buffer
            .asUint8List();
    if (decodePositions(bytes).jdCount != 366) {
      throw StateError('the grid\'s buffer is not its blob');
    }
    _time('decode', () => decodePositions(bytes).jdCount);
  } finally {
    native.dispose();
    hosted.dispose();
  }
}
