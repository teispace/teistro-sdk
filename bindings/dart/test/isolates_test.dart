// Contexts on isolates of their own (`docs/05-testing/01-quality-bar.md`,
// "no global state"), as Node's `workers.test.mjs` runs them on worker
// threads: each isolate opens the library, founds its own context and asks
// the same week of days and chart, all at once, and every isolate answers
// what this one answers. An isolate that saw another's state, or a library
// whose lazily built tables raced, would answer otherwise or crash the
// process.

import 'dart:isolate';

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

/// The isolates asked at once.
const isolates = 4;

/// What a context answers, as plain numbers an isolate can send back: each
/// day's sunrise and the chart's longitudes, graha by graha.
List<double> asked() {
  final ctx = Teistro.open().context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );
  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );
  final week = ctx.almanac.of(
    from: Calendar.gregorian.date(2024, 6, 17),
    to: Calendar.gregorian.date(2024, 6, 23),
    place: place,
    utcOffsetSeconds: 20700,
  );
  final chart = ctx.chart.found(
    instant: 2451545.25,
    place: place,
    utcOffsetSeconds: 20700,
  );
  final answer = [
    for (var i = 0; i < week.length; i++) week.at(i).day.sunrise,
    for (final graha in chart.grahas) graha.longitudeDeg,
  ];
  ctx.dispose();
  return answer;
}

void main() {
  test('contexts on isolates answer what this isolate answers', () async {
    final here = asked();
    final answers = await Future.wait([
      for (var i = 0; i < isolates; i++) Isolate.run(asked),
    ]);
    for (final answer in answers) {
      expect(answer, here);
    }
    expect(here.length, greaterThanOrEqualTo(7 + 9));
  });
}
