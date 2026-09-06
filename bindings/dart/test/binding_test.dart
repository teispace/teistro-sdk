// The Dart binding end to end: the same scenario the C binding's smoke
// test and the Node binding's tests walk, through the generated
// declarations and the ergonomic layer.
//
// `cargo xtask check-dart` builds the shared library and runs this file;
// `TEISTRO_LIBRARY` names it.

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final Teistro teistro = Teistro.open();

/// A context with the analytic test provider; every test builds its own.
Context context({
  String? profile = 'nepali-default',
  Map<String, Object?>? settings,
  String? locale = 'ne-Deva-NP',
  bool testProvider = true,
}) => teistro.context(
  profile: profile,
  settings: settings,
  locale: locale,
  testProvider: testProvider,
);

/// A Gregorian date as the boundary takes one, the long way, which the
/// layer's `Calendar.date` shortens.
CalendarDate gregorian(int year, int month, int day) => CalendarDate(
  calendar: Calendar.gregorian,
  year: year,
  eraYear: 0,
  month: month,
  day: day,
  resolution: Resolution.defined,
  computedMonth: 0,
  computedDay: 0,
);

void main() {
  test('the library and the declarations were generated for the same ABI', () {
    expect(teistro.abi, generatedAbiVersion);
    expect(teistro.catalogue, 1);
    expect(teistro.defaultProfileId, 'parashari-classical');
    expect(teistro.version, matches(r'^\d+\.\d+\.\d+$'));
  });

  test('a context resolves its settings and reports them', () {
    final ctx = context();
    expect(ctx.profile, 'nepali-default');
    expect(ctx.locale, 'ne-Deva-NP');
    expect(ctx.settingsHash, matches(r'^[0-9a-f]{64}$'));
    expect(
      (ctx.settings['frame']! as Map<String, Object?>)['zodiac'],
      'SIDEREAL',
    );
    expect(ctx.settings['schema'], 1);

    // A patch over the profile changes the settings and therefore the hash.
    final patched = context(
      settings: {
        'frame': {'zodiac': 'TROPICAL'},
      },
    );
    expect(
      (patched.settings['frame']! as Map<String, Object?>)['zodiac'],
      'TROPICAL',
    );
    expect(patched.settingsHash, isNot(ctx.settingsHash));

    // The default profile is the one the library names.
    expect(
      context(profile: null, locale: null).profile,
      teistro.defaultProfileId,
    );
    ctx.dispose();
    expect(
      () => ctx.profile,
      throwsStateError,
      reason: 'a freed context is closed',
    );
  });

  test('a refusal carries its status, its field and its hint', () {
    final ctx = context();
    expect(
      () => ctx.keyId('graha.SUNN'),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.status, 'status', Status.unsupported)
            .having((e) => e.status.id, 'code', -6)
            .having((e) => e.detail, 'detail', 'UNKNOWN_KEY')
            .having((e) => e.hint, 'hint', contains('did you mean `SUN`'))
            .having((e) => e.toString(), 'toString', contains('unsupported')),
      ),
    );
    expect(
      () => context(profile: 'vedic-classic'),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.message,
          'message',
          contains('no shipped profile `vedic-classic`'),
        ),
      ),
    );
    expect(
      () => context(locale: 'xx-Latn'),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.message,
          'message',
          contains('ne-Deva-NP'),
        ),
      ),
    );
    expect(
      () => ctx.fixedOf(gregorian(2023, 2, 29)),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.detail, 'detail', 'NONEXISTENT_DATE')
            .having((e) => e.status, 'status', Status.invalidArg),
      ),
    );
  });

  test('a date converts into Bikram Sambat with its era and its '
      'resolution', () {
    final ctx = context();
    final date = gregorian(2015, 4, 14);
    final bs = ctx.convert(date, Calendar.bikramSambat);
    expect(bs.year, 2072);
    expect(bs.month, 1);
    expect(bs.day, 1);
    expect(bs.era, Era.vikrama);
    expect(bs.eraYear, 2072);
    expect(
      bs.resolution,
      Resolution.tabular,
      reason: 'inside the official table',
    );

    final fixed = ctx.fixedOf(date);
    expect(fixed, 735702);
    expect(ctx.weekdayOf(date), 2, reason: 'a Tuesday');
    expect(ctx.dateOf(Calendar.gregorian, fixed).era, Era.commonEra);
    expect(ctx.monthLength(Calendar.gregorian, 2024, 2), 29);
    expect(ctx.isLeap(Calendar.gregorian, 2024), isTrue);
    expect(teistro.julianDayOfFixed(fixed), 2457126.5);
    expect(teistro.fixedOfJulianDay(2457126.75), (
      value: fixed,
      fraction: 0.25,
    ));
  });

  test('a Nepali birth time resolves with the metadata a stored chart '
      'keeps', () {
    final ctx = context();
    final civil = Calendar.gregorian.date(1986, 1, 1).at(hour: 0, minute: 20);
    expect(
      civil.date,
      isA<CalendarDate>().having((d) => d.year, 'year', 1986),
      reason: 'the layer builds the value the generated class holds',
    );
    final zone = ianaZone('Asia/Kathmandu');
    final resolved = ctx.resolve(civil, zone);
    expect(resolved.instantJdUtc, closeTo(2446431.2743056, 1e-6));
    expect(
      resolved.offsetSeconds,
      20700,
      reason: '+05:45, the offset that began that midnight',
    );
    expect(resolved.era, ZoneEra.current);
    expect(resolved.source, ZoneSource.iana);
    expect(resolved.timeKnown, isTrue);
    expect(resolved.warnings, isEmpty, reason: 'nothing had to be guessed');
    expect(resolved.tzdbVersion, matches(r'^20\d\d[a-z]$'));

    final back = ctx.civilOf(resolved.instantJdUtc, zone, Calendar.gregorian);
    expect(back.civil.date.year, 1986);
    expect(back.civil.time.minute, 20);
    expect(back.civil.time.hasTime, isTrue);
    expect(back.resolution.offsetSeconds, 20700);

    expect(
      () => ctx.resolve(civil, ianaZone('Asia/Kathmandou')),
      throwsA(isA<TeistroException>()),
    );
  });

  test('the time scales convert with what they applied', () {
    final ctx = context();
    final tt = ctx.convertTime(2451544.5, Scale.utc, Scale.tt);
    expect(
      tt.deltaTSeconds,
      closeTo(64.184, 1e-9),
      reason: 'exact through the leap-second table',
    );
    expect(tt.deltaTSource, DeltaTSource.leapSeconds);
    expect(tt.deltaTModel, 'TABLE_THEN_MODEL');
    expect(tt.jd, closeTo(2451544.5 + 64.184 / 86400, 1e-12));
    expect(
      ctx.convertTime(tt.jd, Scale.tt, Scale.utc).jd,
      closeTo(2451544.5, 1e-9),
    );

    final delta = ctx.deltaT(2451544.5);
    expect(delta.seconds, closeTo(63.83, 0.02));
    expect(delta.source, DeltaTSource.table);
  });

  test('positions come back in the frame asked for', () {
    final ctx = context();
    final frame = teistro.canonicalFrame;
    expect(frame.centre, Centre.geocentric);
    expect(frame.coordinates, Coordinates.ecliptic);
    expect(frame.sidereal, isFalse);
    expect(frame.ayanamsha, isNull, reason: 'a tropical frame carries none');
    final again = teistro.unpackFrame(teistro.packFrame(frame));
    expect(again.centre, frame.centre);
    expect(again.equinox, frame.equinox);
    expect(again.sidereal, frame.sidereal);

    final positions = ctx.positions(
      instants: [2451545.0, 2451546.0],
      bodies: [Body.sun, Body.moon, Body.mars],
    );
    expect(positions.bodyKeys, [Body.sun, Body.moon, Body.mars]);
    expect(positions.jds, [2451545.0, 2451546.0]);
    expect(positions.timeScale, TimeScale.ut1);
    expect(positions.cells.length, 6, reason: 'two instants by three bodies');

    final sun = positions.at(0, 0);
    expect(sun.longitude, inInclusiveRange(0, 360));
    expect(sun.status, 0);
    expect(
      positions.at(0, 1).longitudeSpeed.abs(),
      greaterThan(sun.longitudeSpeed.abs()),
      reason: 'the Moon moves faster than the Sun',
    );
    expect(() => positions.at(2, 0), throwsRangeError);

    expect(positions.provenanceOf['profile'], 'nepali-default');
    expect(positions.provenanceOf['calculation_version'], 1);
    expect(positions.provenanceOf['settings_hash'], ctx.settingsHash);
    expect(
      (positions.provenanceOf['provider']! as Map<String, Object?>)['frame'],
      'GEOCENTRIC/OF_DATE/ECLIPTIC/TROPICAL/APPARENT',
    );

    // Without an ephemeris the call is a missing capability, named.
    final bare = context(testProvider: false);
    expect(
      () => bare.positions(instants: [2451545.0], bodies: [Body.sun]),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.status, 'status', Status.capability)
            .having((e) => e.field, 'field', 'provider'),
      ),
    );
    expect(
      () => ctx.positions(instants: [], bodies: [Body.sun]),
      throwsArgumentError,
    );
    expect(
      () => ctx.positions(instants: [2451545.0], bodies: []),
      throwsArgumentError,
    );
  });

  test('the locale engine renders typed parameters, and says where '
      'from', () {
    final ctx = context();
    final rendered = ctx.render('sdk.reason.grahaInBhava', {
      'graha': {r'$entity': 'graha.JUPITER'},
      'bhava': 7,
    });
    expect(rendered.from, 'ne-Deva-NP');
    expect(rendered.fallback, isFalse);
    expect(rendered.override, isFalse);
    expect(rendered.warningList, isEmpty);
    expect(rendered.text, contains('७'), reason: 'the Nepali numeral seven');
    expect(ctx.has('sdk.reason.grahaInBhava'), isTrue);
    expect(ctx.has('sdk.nope.missing'), isFalse);

    // A missing message renders as its key with a warning, never an error.
    final missing = ctx.render('sdk.nope.missing');
    expect(missing.from, isNull);
    expect(missing.warningList, isNotEmpty);

    ctx.locale = 'en-Latn';
    expect(ctx.locale, 'en-Latn');
    expect(
      ctx.render('sdk.reason.grahaInBhava', {
        'graha': {r'$entity': 'graha.JUPITER'},
        'bhava': 7,
      }).text,
      contains('Jupiter'),
    );
    expect(
      () => ctx.locale = 'fr-Latn',
      throwsA(
        isA<TeistroException>()
            .having((e) => e.field, 'field', 'locale')
            .having((e) => e.hint, 'hint', contains('sa-Deva')),
      ),
    );
  });

  test('a quantity is its own type, and its constructor checks the '
      'range', () {
    final ctx = context();
    addTearDown(ctx.dispose);
    expect(
      Latitude(27.7172),
      27.7172,
      reason: 'a branded number is a double at run time',
    );
    expect(() => Latitude(91), throwsRangeError);
    expect(() => Longitude(-181), throwsRangeError);
    expect(() => Altitude(20000), throwsRangeError);

    // The place reaches the boundary and comes back through the frame.
    final positions = ctx.positions(
      instants: [2451545.0],
      bodies: [Body.sun],
      observer: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
    );
    expect(positions.cells.length, 1);
  });

  test('a catalogue key packs to an id and back', () {
    final ctx = context();
    final id = ctx.keyId('graha.SUN');
    expect(
      id,
      1 << 16,
      reason: 'the kind in the high half, the member in the low',
    );
    expect(ctx.keyName(id), 'graha.SUN');
    expect(ctx.keyName(ctx.keyId('nakshatra.ASHWINI')), 'nakshatra.ASHWINI');
    expect(
      () => ctx.keyName(0xffffffff),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.status,
          'status',
          Status.unsupported,
        ),
      ),
    );
  });
}
