/// Spike 2, option A: the Dart measurements, the same rows as the Node
/// benchmark so the two columns of the result table compare like for like.
///
/// Run with `dart run bench/bench.dart` (JIT) from `a-dart/`; results go to
/// `../results/a-dart.json`.
library;

import 'package:spike_bench/spike_bench.dart';
import 'package:teistro_spike_a_dart/teistro_spike.dart';

const double jd = 2460000.5;

/// A Dart provider as cheap as the native one, so the boundary is what is measured.
Position dartProvider(double jdUt, int body) =>
    Position(longitudeDeg: (jdUt % 360) + body * 10, latitudeDeg: 0.5, speedDegPerDay: 1);

void main() {
  final lib = openLibrary();
  final depth3 = TeistroContext(lib);
  final depth3Host = TeistroContext(lib, provider: dartProvider);
  final depth4 = TeistroContext(lib, dashaDepth: DashaDepth(4));
  final depth5 = TeistroContext(lib, dashaDepth: DashaDepth(5));
  final at = JulianDay(jd);

  final chart3 = depth3.computeChart(at);
  final chart4 = depth4.computeChart(at);
  final chart5 = depth5.computeChart(at);
  if (chart3.nodeCount != chartNodeCount(lib, 3) || chart5.nodeCount != 66429) {
    throw StateError('node counts disagree with the library');
  }
  if (depth3Host.computeChart(at).positions()[0].longitudeDeg == chart3.positions()[0].longitudeDeg) {
    throw StateError('the host provider was not used');
  }

  final native3 = bench('chart, depth 3 (819 nodes), native provider: compute + blob copy', () => depth3.computeChart(at));
  final host3 = bench('chart, depth 3, Dart provider: 9 callbacks + compute + blob copy', () => depth3Host.computeChart(at));
  final rows = <Row>[
    native3,
    host3,
    derived('one provider callback into Dart (derived)', host3, native3, 9),
    bench('decode columns, depth 3 (views)', () => Chart(chart3.bytes).decoded),
    bench('positions as 9 objects', chart3.positions),
    bench('eager tree, depth 3 (819 objects)', chart3.tree),
    bench('chart, depth 4 (7 380 nodes), native provider', () => depth4.computeChart(at), iterations: 20),
    bench('eager tree, depth 4 (7 380 objects)', chart4.tree, iterations: 20),
    bench('chart, depth 5 (66 429 nodes), native provider', () => depth5.computeChart(at), iterations: 4),
    bench('decode columns, depth 5', () => Chart(chart5.bytes).decoded, iterations: 4),
    bench('eager tree, depth 5 (66 429 objects)', chart5.tree, iterations: 2),
  ];
  final blobBytes = {'depth3': chart3.bytes.length, 'depth4': chart4.bytes.length, 'depth5': chart5.bytes.length};

  report(rows);
  print('\nblob bytes: depth 3 ${blobBytes['depth3']}, depth 4 ${blobBytes['depth4']}, depth 5 ${blobBytes['depth5']}');
  writeResults('../results', 'a-dart.json', {'option': 'A', 'binding': 'dart', 'blob_bytes': blobBytes, 'rows': rows});
  for (final context in [depth3, depth3Host, depth4, depth5]) {
    context.dispose();
  }
}
