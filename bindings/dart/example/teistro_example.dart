// The README's example, kept honest: `cargo xtask check-dart` runs this
// file, so the README cannot drift from what the binding does.

import 'package:teistro/teistro.dart';

void main() {
  final teistro = Teistro.open();
  print('Teistro ${teistro.version}, ABI ${teistro.abi}');

  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );

  // 14 April 2015 is 1 Baisakh 2072 BS.
  final bs = ctx.convert(
    Calendar.gregorian.date(2015, 4, 14),
    Calendar.bikramSambat,
  );
  print('${bs.year}-${bs.month}-${bs.day} ${bs.era?.key}');

  // A Kathmandu birth time, with the metadata a stored chart keeps.
  final resolved = ctx.resolve(
    Calendar.gregorian.date(1986, 1, 1).at(hour: 0, minute: 20),
    ianaZone('Asia/Kathmandu'),
  );
  print(
    'JD ${resolved.instantJdUtc.toStringAsFixed(6)} UTC, '
    '${resolved.offsetSeconds} s, tzdb ${resolved.tzdbVersion}',
  );

  // The Sun and the Moon at J2000, in the SDK's canonical frame.
  final sky = ctx.positions(
    instants: [2451545.0],
    bodies: [Body.sun, Body.moon],
  );
  print('the Sun at ${sky.at(0, 0).longitude.toStringAsFixed(4)} degrees');

  // A message in the context's locale.
  print(
    ctx.render('sdk.reason.grahaInBhava', {
      'graha': {r'$entity': 'graha.JUPITER'},
      'bhava': 7,
    }).text,
  );

  ctx.dispose();
}
