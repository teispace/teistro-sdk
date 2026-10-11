// The four facts the C smoke test prints, asserted by every consumer of a
// shipped library: the published package (`consumer.dart`, run in a
// throwaway project) and the Flutter app on a device or simulator
// (`bindings/flutter/example/integration_test`). One suite, so a library
// that loads but answers differently fails wherever it was shipped.

import 'package:teistro/teistro.dart';

/// Holds one fact: what was [found] against what was [wanted], named by
/// [what].
typedef Expect = void Function(Object? found, Object? wanted, String what);

/// Asserts the four facts on [teistro] through [expect], saying what it
/// read through [say].
void consumerFacts(
  Teistro teistro,
  Expect expect, {
  void Function(String line) say = _quiet,
}) {
  say(
    'abi ${teistro.abi}, sdk ${teistro.version}, '
    '${teistro.build.target}, ${teistro.build.profile}',
  );
  expect(teistro.build.optimised, true, 'a published library is optimised');
  expect(teistro.build.sanitizer, '', 'a published library has no sanitizer');
  expect(teistro.version, prebuiltVersion, 'the library is this package\'s');

  final context = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );
  try {
    final bs = context.calendar.convert(
      CalendarDate(
        calendar: Calendar.gregorian,
        year: 2015,
        eraYear: 0,
        month: 4,
        day: 14,
        resolution: Resolution.defined,
        computedMonth: 0,
        computedDay: 0,
      ),
      Calendar.bikramSambat,
    );
    say('14 April 2015 is ${bs.year}-${bs.month}-${bs.day} BS');
    expect(
      '${bs.year}-${bs.month}-${bs.day}',
      '2072-1-1',
      'the Bikram Sambat date',
    );

    final rendered = context.intl.render(
      'sdk.reason.grahaInBhava',
      <String, Object?>{'graha': 'graha.JUPITER', 'bhava': 7},
    );
    say('sdk.reason.grahaInBhava in ne-Deva-NP: ${rendered.text}');
    expect(rendered.text, 'गुरु ७औं भावमा', 'the rendered message');

    final positions = context.positions(
      instants: <double>[2451545.0],
      bodies: <Body>[Body.sun],
    );
    final sun = positions.at(0, 0);
    say('the Sun at J2000 is at ${sun.longitude.toStringAsFixed(4)} degrees');
    expect(sun.longitude.toStringAsFixed(4), '278.5768', 'the Sun');
  } finally {
    context.dispose();
  }
}

void _quiet(String line) {}
