// The annual chart: the one instant every Tajika judgement is made from.
//
// A birth chart is cast for a birth. An **annual** chart is cast for the
// moment the Sun comes back to the longitude it held then — once a year,
// about twenty minutes earlier than the clock would say, and never on the
// birthday itself (`docs/03-design/annual-chart.md`).
//
// What it teaches:
//
// 1. **The boundary answers the instant, not the chart.** Whether the
//    annual chart is cast for the birthplace or for where you live now is
//    a question the schools answer differently, so the SDK hands you the
//    instant and you found the chart with the place you mean.
// 2. **Which longitude is a choice with a name.** `sidereal` is the
//    tradition's; `tropical` is the Western solar return and is most of a
//    circle of lagna away by the fortieth year; `mean` is the older
//    arithmetic and needs no ephemeris at all. None of them is a fallback
//    for another.
// 3. **Fewer than you asked for is the answer**, not a refusal: an
//    ephemeris that ends before your hundredth year says so by giving you
//    the years it has.
//
// The record is `birth_chart.dart`'s own, so the two can be read side by
// side.

import 'package:teistro/teistro.dart';

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );

  final birthDay = Calendar.gregorian.date(1990, 4, 14);
  final when = ctx.time.resolve(
    birthDay.at(hour: 5, minute: 30),
    ianaZone('Asia/Kathmandu'),
  );
  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );

  // ── The years a birth opens ───────────────────────────────────────
  final chart = ctx.chart.found(
    instant: when.instantJdUtc,
    place: place,
    utcOffsetSeconds: when.offsetSeconds,
    varsha: const VarshaRequest(through: 40),
  );
  final years = chart.praveshas;
  print('returns computed: ${years.length}');
  final thirtieth = years.firstWhere((one) => one.year == 30);
  print(
    'the thirtieth year opens at jd ${thirtieth.instant.toStringAsFixed(6)}',
  );

  // A return is about a sidereal year after the last, never a calendar one.
  final gaps = <double>[
    for (var i = 1; i < years.length; i += 1)
      years[i].instant - years[i - 1].instant,
  ];
  gaps.sort();
  print(
    'between returns: ${gaps.first.toStringAsFixed(4)} '
    'to ${gaps.last.toStringAsFixed(4)} days',
  );

  // ── The chart of that year, cast where you choose ─────────────────
  final annual = ctx.chart.found(
    instant: thirtieth.instant,
    place: place,
    utcOffsetSeconds: when.offsetSeconds,
  );
  print(
    'natal lagna ${chart.lagnaDeg.toStringAsFixed(3)}°, '
    'annual lagna ${annual.lagnaDeg.toStringAsFixed(3)}°',
  );

  // ── The readings are named, and they are not each other ───────────
  for (final reading in VarshaReading.values) {
    final one =
        ctx.chart
            .found(
              instant: when.instantJdUtc,
              place: place,
              utcOffsetSeconds: when.offsetSeconds,
              varsha: VarshaRequest(through: 30, reading: reading),
            )
            .praveshas;
    final apart = (one[29].instant - years[29].instant) * 24;
    print(
      '${reading.key.padRight(9)} thirtieth year, '
      '${apart.toStringAsFixed(2)} hours from the sidereal one',
    );
  }

  // ── What it refuses, and by which field ───────────────────────────
  try {
    ctx.chart.found(
      instant: when.instantJdUtc,
      place: place,
      utcOffsetSeconds: when.offsetSeconds,
      varsha: const VarshaRequest(through: 0),
    );
  } on TeistroException catch (error) {
    print('refused  ${error.field}: ${error.message}');
  }

  ctx.dispose();
}
