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

  // ── Its five office-bearers, cast where you say ───────────────────
  // Here the birthplace, the one Tajika text read casts every chart for;
  // a residence is `AnnualPlace.at(Observer(...), utcOffsetSeconds: ...)`.
  final cast =
      ctx.chart
          .found(
            instant: when.instantJdUtc,
            place: place,
            utcOffsetSeconds: when.offsetSeconds,
            varsha: const VarshaRequest(through: 30, place: AnnualPlace.birth),
          )
          .praveshas[29];
  final b = cast.annual!.officeBearers;
  final five = [
    b.muntha,
    b.janmaLagna,
    b.varshaLagna,
    b.triRashi,
    b.dinaRatri,
  ].map((lord) => lord.key).join(' ');
  final part = cast.annual!.byDay ? 'by day' : 'by night';
  print('muntha in ${cast.muntha.sign.key}; office-bearers $five, $part');

  // ── And the lord of that year, with the reason ────────────────────
  final lord = cast.annual!.yearLord;
  print(
    'year lord ${lord.graha.key} at ${lord.vishwa}, chosen ${lord.chosen.key}',
  );
  for (final claim in lord.claims) {
    final aspects = claim.aspectsLagna ? 'aspects' : 'does not aspect';
    print(
      '  ${claim.graha.key.padRight(8)} ${claim.vishwa}  '
      '${claim.portfolios} portfolio(s)  $aspects the lagna',
    );
  }

  // ── The sixteen Tajika yogas answer a matter, not a chart ─────────
  // Fourteen of them judge the lagnesha against the lord of the house you
  // ask about, so you name the houses: marriage (7) and career (10) here.
  final judged =
      ctx.chart
          .found(
            instant: when.instantJdUtc,
            place: place,
            utcOffsetSeconds: when.offsetSeconds,
            varsha: const VarshaRequest(
              through: 30,
              place: AnnualPlace.birth,
              matters: Matters.houses([7, 10]),
            ),
          )
          .praveshas[29]
          .annual!;
  for (final matter in judged.matters) {
    final held = matter.held.map((one) => one.yoga.key).join(', ');
    print(
      'house ${matter.house}: ${matter.lagnesha.key} with ${matter.karyesha.key}, '
      'held ${held.isEmpty ? 'none' : held}; '
      'not answered ${matter.unanswered.map((yoga) => yoga.key).join(', ')}',
    );
  }

  // ── The sahams: forty-one sensitive points, each a − b + c ────────
  // Name the ones you want, or `Sahams.all`; each comes back with its
  // sign, that sign's lord and the house it fell in, as the source reads
  // them.
  final points =
      ctx.chart
          .found(
            instant: when.instantJdUtc,
            place: place,
            utcOffsetSeconds: when.offsetSeconds,
            varsha: const VarshaRequest(
              through: 30,
              place: AnnualPlace.birth,
              sahams: Sahams.these([
                Saham.punya,
                Saham.vivaha,
                Saham.karyaSiddhi,
              ]),
            ),
          )
          .praveshas[29]
          .annual!;
  for (final point in points.sahams) {
    print(
      '${point.saham.key.padRight(12)} '
      '${point.longitudeDeg.toStringAsFixed(2).padLeft(6)}°  '
      '${point.sign.key}, lord ${point.lord.key}, house ${point.house}'
      '${point.addedSign ? ' (a sign added)' : ''}',
    );
    // Its strength is the source's clauses, reported and never scored.
    final strong = point.strong.map((c) => c.key).join(', ');
    final weak = point.weak.map((c) => c.key).join(', ');
    print(
      '  strong: ${strong.isEmpty ? 'none' : strong}; '
      'weak: ${weak.isEmpty ? 'none' : weak}',
    );
  }
  // And the seven's Harsha bala that year: four places each is happy in.
  print(points.harsha.map((h) => '${h.graha.key} ${h.total}').join(', '));

  // ── The annual dashas: the year divided among its lords ────────────
  // The Mudda runs round the nine from the birth nakshatra's lord, one
  // lord further each year; the Patyayini is read from the year's own
  // chart, and its lagna's share is a sign's. The Sun is read over the
  // year once for both, and each year closes on the next return.
  final divided =
      ctx.chart
          .found(
            instant: when.instantJdUtc,
            place: place,
            utcOffsetSeconds: when.offsetSeconds,
            varsha: const VarshaRequest(
              through: 30,
              place: AnnualPlace.birth,
              dashas: AnnualDashas.these([
                DashaSystem.mudda,
                DashaSystem.patyayini,
              ]),
            ),
          )
          .praveshas[29]
          .annual!;
  for (final dasha in divided.dashas) {
    final days = [
      for (final p in dasha.periods.where((p) => p.level == 1))
        '${p.sign?.key ?? p.lord.key} ${(p.to - p.from).toStringAsFixed(1)}',
    ];
    print('${dasha.system.key}: ${days.join(', ')}');
  }

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
