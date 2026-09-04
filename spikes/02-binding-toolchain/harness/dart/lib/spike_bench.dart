/// The one Dart timing harness of spike 2, shared by the option A and
/// option B benchmarks so their rows are measured the same way.
///
/// A row is the median over `iterations` samples of `batch` calls each
/// (so a sub-microsecond cost is resolved), in microseconds, with the
/// 90th percentile beside it.
library;

import 'dart:convert';
import 'dart:io';

/// Calls per sample.
const int batch = 50;

/// One measurement.
typedef Row = Map<String, Object?>;

/// Times `fn` over `rounds` independent rounds and keeps the round with the
/// lowest median, so a garbage-collection pause or a late optimisation in
/// one round does not stand for the cost of the call.
Row bench(String name, void Function() fn, {int iterations = 200, int warmup = 20, int rounds = 3}) {
  Row? best;
  for (var round = 0; round < rounds; round++) {
    for (var i = 0; i < warmup * batch; i++) {
      fn();
    }
    final times = List<double>.filled(iterations, 0);
    final watch = Stopwatch();
    for (var i = 0; i < iterations; i++) {
      watch
        ..reset()
        ..start();
      for (var j = 0; j < batch; j++) {
        fn();
      }
      watch.stop();
      times[i] = watch.elapsedTicks * 1e6 / watch.frequency / batch;
    }
    times.sort();
    double at(double q) => times[(q * iterations).floor().clamp(0, iterations - 1)];
    final row = <String, Object?>{'name': name, 'median_us': at(0.5), 'p90_us': at(0.9), 'iterations': iterations * batch, 'rounds': rounds};
    if (best == null || (row['median_us']! as double) < (best['median_us']! as double)) {
      best = row;
    }
  }
  return best!;
}

/// A row the binding cannot express, with the reason.
Row unavailable(String name, String note) =>
    {'name': name, 'median_us': null, 'p90_us': null, 'iterations': 0, 'note': note};

/// A row derived from two others: `(whole − base) / divisor`.
Row derived(String name, Row whole, Row base, int divisor) => {
      'name': name,
      'median_us': ((whole['median_us']! as double) - (base['median_us']! as double)) / divisor,
      'p90_us': ((whole['p90_us']! as double) - (base['p90_us']! as double)) / divisor,
      'iterations': whole['iterations'],
    };

/// Prints the rows as a Markdown table.
void report(List<Row> rows) {
  String value(Object? v) => v == null ? 'n/a' : (v as double).toStringAsFixed(2);
  stdout.writeln('| measurement | median µs | p90 µs |');
  stdout.writeln('|---|---:|---:|');
  for (final row in rows) {
    stdout.writeln('| ${row['name']} | ${value(row['median_us'])} | ${value(row['p90_us'])} |');
  }
}

/// Writes the rows and their context to `<dir>/<file>` as JSON.
void writeResults(String dir, String file, Map<String, Object?> payload) {
  final out = Directory(dir)..createSync(recursive: true);
  final body = {'dart': Platform.version, 'mode': 'jit', ...payload};
  File('${out.path}/$file').writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(body)}\n');
}
