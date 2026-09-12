// The five limbs of a day, computed from the boundary alone.
//
// A panchanga is the Hindu almanac's five parts — tithi, vara,
// nakshatra, yoga and karana — and every one of them except the weekday
// is a function of **two longitudes**: the Sun's and the Moon's, in the
// sidereal zodiac. So a binding can compute a whole panchanga from
// `positions` and `weekdayOf`, without any part of the chart layer.
//
// The arithmetic is the SDK's own (`crates/panchanga/src/limb.rs`):
//
//   tithi      Moon − Sun    30 divisions of 12°
//   karana     Moon − Sun    60 divisions of 6°
//   nakshatra  Moon          27 divisions of 360/27°
//   yoga       Moon + Sun    27 divisions of 360/27°
//   vara       the weekday    7
//
// The karana is the one that is not a plain division: sixty half-tithis
// make a lunar month and they are **not** a cycle of eleven. Kimstughna
// opens the month, Shakuni, Chatushpada and Naga close it, and the seven
// movable karanas repeat through everything between. `karanaOf` below is
// the SDK's rule, transcribed, and the SDK holds it to the corpus's own
// successor relation over 109 consecutive pairs.
//
// What this example is honest about: a limb here is the one holding **at
// the instant asked for**. A printed almanac gives the limb at sunrise
// and the time it ends, which needs a boundary search over the Moon's
// motion — that is what `crates/panchanga` does with a real ephemeris,
// and what a binding cannot yet ask for.

import 'package:teistro/teistro.dart';

const double nakshatraDeg = 360.0 / 27.0;
const double yogaDeg = 360.0 / 27.0;
const double tithiDeg = 12.0;
const double karanaDeg = 6.0;

/// The karana that a half-tithi of the lunar month is.
///
/// Transcribed from `crates/panchanga/src/limb.rs`. Sixty of them make a
/// month: Kimstughna opens it, Shakuni, Chatushpada and Naga close it,
/// and the seven movable karanas fill everything between, Bava first
/// from the second half of the first tithi.
Karana karanaOf(int halfTithi) {
  final half = halfTithi % 60;
  return switch (half) {
    0 => Karana.kimstughna,
    57 => Karana.shakuni,
    58 => Karana.chatushpada,
    59 => Karana.naga,
    _ => Karana.byId((half - 1) % 7),
  };
}

/// The five limbs at one instant, with the numbers behind them.
final class Panchanga {
  const Panchanga({
    required this.sun,
    required this.moon,
    required this.tithi,
    required this.vara,
    required this.nakshatra,
    required this.yoga,
    required this.karana,
  });

  final double sun;
  final double moon;
  final Tithi tithi;
  final Vara vara;
  final Nakshatra nakshatra;
  final Yoga yoga;
  final Karana karana;

  /// How far the Moon stands ahead of the Sun, 0 to 360.
  double get elongation => (moon - sun) % 360.0;

  /// The fortnight: waxing to the full moon, waning after it.
  String get paksha => elongation < 180.0 ? 'shukla' : 'krishna';

  /// How far through the tithi the Moon has come, as a fraction.
  double get tithiElapsed => (elongation % tithiDeg) / tithiDeg;
}

/// The five limbs at an instant.
///
/// `weekday` is the ISO weekday of the **civil day** the instant belongs
/// to, which the caller has because it asked the calendar for it: a vara
/// is a property of the day, not of the moment.
Panchanga panchangaAt(
  Teistro teistro,
  Context ctx,
  double instant,
  int weekday,
) {
  final canonical = teistro.canonicalFrame;
  final frame = Frame(
    ayanamsha: Ayanamsha.lahiri,
    centre: canonical.centre,
    equinox: canonical.equinox,
    coordinates: canonical.coordinates,
    sidereal: true,
    lightTime: canonical.lightTime,
    aberration: canonical.aberration,
    deflection: canonical.deflection,
    nutation: canonical.nutation,
  );
  final sky = ctx.positions(
    instants: [instant],
    bodies: [Body.sun, Body.moon],
    frame: frame,
  );
  final sun = sky.at(0, 0).longitude;
  final moon = sky.at(0, 1).longitude;
  final elongation = (moon - sun) % 360.0;
  return Panchanga(
    sun: sun,
    moon: moon,
    tithi: Tithi.byId(elongation ~/ tithiDeg),
    // The boundary's weekday is ISO (Monday 1 … Sunday 7) and a vara
    // counts from Sunday, so the one becomes the other by `% 7`.
    vara: Vara.byId(weekday % 7),
    nakshatra: Nakshatra.byId(moon ~/ nakshatraDeg),
    yoga: Yoga.byId(((moon + sun) % 360.0) ~/ yogaDeg),
    karana: karanaOf(elongation ~/ karanaDeg),
  );
}

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );

  // Nepali New Year: the first day of Baisakh, BS 2082.
  final day = Calendar.bikramSambat.date(2082, 1, 1);
  final gregorian = ctx.calendar.convert(day, Calendar.gregorian);
  // Six in the morning stands in for sunrise, which the almanac would
  // use and which needs the rise-and-set solver.
  final when = ctx.time.resolve(day.at(hour: 6), ianaZone('Asia/Kathmandu'));
  final found = panchangaAt(
    teistro,
    ctx,
    when.instantJdUtc,
    ctx.calendar.weekdayOf(day),
  );

  print(
    'BS ${day.year}-${_two(day.month)}-${_two(day.day)}'
    '  (${gregorian.year}-${_two(gregorian.month)}-${_two(gregorian.day)})'
    '  06:00 Kathmandu',
  );
  print(
    '  sun ${found.sun.toStringAsFixed(4).padLeft(8)}°'
    '   moon ${found.moon.toStringAsFixed(4).padLeft(8)}°'
    '   elongation ${found.elongation.toStringAsFixed(4).padLeft(8)}°',
  );
  print('');

  // Each generated enum carries `key` and `fullKey`, but Dart's enums
  // share no supertype, so a heterogeneous list holds the two strings
  // rather than the members themselves.
  final limbs = <(String, String, String)>[
    ('tithi', found.tithi.fullKey, found.tithi.key),
    ('vara', found.vara.fullKey, found.vara.key),
    ('nakshatra', found.nakshatra.fullKey, found.nakshatra.key),
    ('yoga', found.yoga.fullKey, found.yoga.key),
    ('karana', found.karana.fullKey, found.karana.key),
  ];
  for (final (label, fullKey, key) in limbs) {
    final entity = ctx.intl.entity(fullKey);
    print(
      '  ${label.padRight(10)} ${entity.name.padRight(14)} '
      '${entity.iast.padRight(18)} ($key)',
    );
  }
  print('');
  print('  paksha     ${found.paksha}');
  print(
    '  tithi is   ${(found.tithiElapsed * 100).toStringAsFixed(1)}%'
    ' elapsed at this instant',
  );

  // The Sun on this day is the reason the year turns: BS begins at
  // the **Mesha Sankranti**, the instant the Sun enters Aries, and the
  // year's first day is the civil day that instant is reckoned into.
  // So the number worth printing is how far *past* the crossing this
  // moment is -- which is why the almanac's year-start is an instant
  // and not a date.
  //
  // A Rust example of the same scenario is what found this wrong. This
  // file said the Sun "has not quite arrived" and printed 359.9023°
  // short of Aries, when it had entered Aries two and a half hours
  // earlier: `(360 - sun) % 360` of a longitude just past zero is just
  // under 360, and reads as nearly a whole circle still to go.
  final intoSign = found.sun % 30.0;
  print(
    '  the Sun stands ${intoSign.toStringAsFixed(4)}° into Aries, so the Mesha'
    ' Sankranti is about ${(intoSign / 0.9856 * 24).toStringAsFixed(1)} hours past --',
  );
  print('  which is what BS 2082 is reckoned from, and why it opens today');

  ctx.dispose();
}

String _two(int value) => value.toString().padLeft(2, '0');
