// Rectification: a birth time known only to the hour, narrowed by lagna.
//
// A birth record that says "some time before dawn" is the commonest hard
// case in the field. What narrows it is the **lagna** — it moves through
// all twelve signs in a day, so it changes sign every couple of hours,
// and a family that remembers the ascendant remembers something the
// clock does not.
//
// The point of this example is the shape of the call. A rectification
// pass wants many charts at one place, and `foundMany` founds them in
// **one crossing**: the settings are resolved once, the solar model is
// built once, and the day each instant belongs to is reckoned against
// the same sunrise. Founding them one at a time would give the same
// numbers and pay the setup for every one of them.
//
// `testProvider: true` selects the analytic ephemeris the SDK carries,
// so this file runs anywhere.

import 'package:teistro/teistro.dart';

String _two(int value) => value.toString().padLeft(2, '0');

void main() {
  final teistro = Teistro.open();
  // `nepali-default` is what a Nepali birth record is cast under, and
  // its frame is **topocentric**: the chart is seen from the hill the
  // record was written on rather than from the centre of the Earth. The
  // completion does that step itself over any provider
  // (`03-design/topocentric-measured.md`), so the analytic one below is
  // enough, and the steps printed at the end name it.
  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );

  // The record: a Bikram Sambat date, a place, and an hour nobody is
  // sure of. Everything below narrows the last of those.
  final born = Calendar.bikramSambat.date(2045, 9, 17);
  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );
  const fromHour = 0;
  const toHour = 3;
  const everyMinutes = 10;

  // One resolution fixes the zone and the offset; the candidates are
  // then arithmetic on the instant, which is what a Julian day is for.
  final start = ctx.resolve(
    born.at(hour: fromHour),
    ianaZone('Asia/Kathmandu'),
  );
  const step = everyMinutes / (24 * 60);
  const count = (toHour - fromHour) * 60 ~/ everyMinutes;
  final instants = <double>[
    for (var i = 0; i < count; i += 1) start.instantJdUtc + i * step,
  ];

  // ── One crossing for every candidate ─────────────────────────────────
  final charts = ctx.foundMany(
    instants: instants,
    place: place,
    utcOffsetSeconds: start.offsetSeconds,
  );

  print(
    '${charts.chartCount} candidate charts, $everyMinutes minutes apart, '
    'in one crossing',
  );
  print(
    'place  ${place.latitudeDeg}°N ${place.longitudeDeg}°E   '
    '${ChartKind.byId(charts.kind).key}',
  );
  print('');
  print('local   lagna        sign            moon         bhava');
  print('─' * 58);

  int? previous;
  for (final chart in charts.each) {
    final sign = chart.lagnaDeg ~/ 30;
    final moon = chart.grahas.firstWhere((g) => g.graha == Graha.moon);
    final rashi = ctx.entity(Rashi.byId(sign).fullKey);
    final minutes = fromHour * 60 + chart.index * everyMinutes;
    print(
      '${_two(minutes ~/ 60)}:${_two(minutes % 60)}   '
      '${chart.lagnaDeg.toStringAsFixed(4).padLeft(9)}°  '
      '${rashi.name.padRight(14)} '
      '${moon.longitudeDeg.toStringAsFixed(4).padLeft(9)}°  '
      '${moon.house.bhava.toString().padLeft(2)}'
      '${previous != null && sign != previous ? '   ← lagna changes sign' : ''}',
    );
    previous = sign;
  }

  // ── What the batch shares, and what it does not ──────────────────────
  // The place, the settings, the solar model and the completion steps
  // are one to a batch: they are what "the same chart at a different
  // minute" holds constant. The instant, the lagna, the day and the
  // timing are per chart. The provenance envelope stamps the batch as a
  // whole, so a rectification run reproduces as one thing.
  print('');
  print('model          ${charts.model}');
  print('steps applied  ${charts.stepsApplied.join(', ')}');
  print('settings hash  ${ctx.settingsHash.substring(0, 16)}…');

  // A batch of one is the ordinary case, and `found` is the same
  // crossing with the batch unwrapped: the answer is a chart, not a list
  // of one.
  final single = ctx.found(
    instant: start.instantJdUtc,
    place: place,
    utcOffsetSeconds: start.offsetSeconds,
  );
  print('');
  print(
    'found(one)     lagna ${single.lagnaDeg.toStringAsFixed(4)}°  '
    'vara ${single.vara.key}  '
    'hora lord ${single.horaLord.key}',
  );
  ctx.dispose();
}
