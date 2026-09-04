/// Spike 2, option B: the Dart measurements over Diplomat's generated
/// binding, the same rows as option A's Dart benchmark where the backend
/// can express them. There is no provider row: Diplomat 0.16's Dart
/// backend refuses traits and callbacks.
///
/// Run with `dart run bench/bench.dart` from `b-dart/`; results go to
/// `../results/b-dart.json`. The bridge's release build is loaded into the
/// process first, because the generated code binds its symbols with
/// `@Native` against the process rather than a named asset.
library;

import 'dart:ffi' as ffi;
import 'dart:io';

import 'package:spike_bench/spike_bench.dart';
import 'package:teistro_spike_b_dart/src/lib.g.dart';

const double jd = 2460000.5;
const String notExpressible = 'not expressible: the backend refuses traits and callbacks';

String workspaceBuild() {
  final name = switch (Platform.operatingSystem) {
    'macos' => 'libteistro_spike_b_bridge.dylib',
    'windows' => 'teistro_spike_b_bridge.dll',
    _ => 'libteistro_spike_b_bridge.so',
  };
  var dir = Directory.current;
  for (var up = 0; up < 8; up++) {
    final candidate = File('${dir.path}/target/release/$name');
    if (candidate.existsSync()) return candidate.path;
    dir = dir.parent;
  }
  throw StateError('no $name under a target/release directory');
}

/// A node of the tree rebuilt from the rows.
final class Node {
  Node(this.lord, this.level, this.startJd, this.endJd);
  final Body lord;
  final int level;
  final double startJd;
  final double endJd;
  final List<Node> children = [];
}

List<BodyPosition> positions(Chart chart) =>
    List<BodyPosition>.generate(chart.positionCount, (i) => chart.position(i)!, growable: false);

List<Node> tree(Chart chart) {
  final n = chart.dashaRowCount;
  final nodes = List<Node?>.filled(n, null);
  final roots = <Node>[];
  for (var i = 0; i < n; i++) {
    final r = chart.dashaRow(i)!;
    final node = Node(r.lord, r.level, r.startJd, r.endJd);
    nodes[i] = node;
    if (r.parent < 0) {
      roots.add(node);
    } else {
      nodes[r.parent]!.children.add(node);
    }
  }
  return roots;
}

void main() {
  ffi.DynamicLibrary.open(workspaceBuild());
  Settings settings(int depth) => Settings(ayanamsha: Ayanamsha.lahiri, node: NodeKind.mean, dashaDepth: depth);
  final depth3 = Context(settings(3));
  final depth4 = Context(settings(4));
  final depth5 = Context(settings(5));
  final chart3 = depth3.computeChart(jd);
  final chart4 = depth4.computeChart(jd);
  final chart5 = depth5.computeChart(jd);
  if (chart3.dashaRowCount != Info.nodeCountForDepth(3) || chart5.dashaRowCount != 66429) {
    throw StateError('node counts disagree with the library');
  }
  if (tree(chart3).length != 9 || positions(chart3)[1].sign != 0) {
    throw StateError('the accessors do not reproduce the chart');
  }

  final rows = <Row>[
    bench('chart, depth 3 (819 nodes), built-in provider: compute (opaque handle)', () => depth3.computeChart(jd)),
    unavailable('chart, depth 3, Dart provider', notExpressible),
    unavailable('one provider callback into Dart', notExpressible),
    bench('positions as 9 objects (9 accessor calls)', () => positions(chart3)),
    bench('eager tree, depth 3 (819 accessor calls)', () => tree(chart3)),
    bench('one lazy row (one accessor call)', () => chart3.dashaRow(400), iterations: 2000),
    bench('chart, depth 4 (7 380 nodes), built-in provider', () => depth4.computeChart(jd), iterations: 20),
    bench('eager tree, depth 4 (7 380 accessor calls)', () => tree(chart4), iterations: 20),
    bench('chart, depth 5 (66 429 nodes), built-in provider', () => depth5.computeChart(jd), iterations: 4),
    bench('eager tree, depth 5 (66 429 accessor calls)', () => tree(chart5), iterations: 2),
  ];

  report(rows);
  writeResults('../results', 'b-dart.json', {'option': 'B', 'binding': 'dart', 'rows': rows});
}
