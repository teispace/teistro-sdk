// The Dart binding end to end: the same scenario the C binding's smoke
// test and the Node binding's tests walk, through the generated
// declarations and the ergonomic layer.
//
// `cargo xtask check-dart` builds the shared library and runs this file;
// `TEISTRO_LIBRARY` names it.

import 'dart:io';

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
    expect(ctx.intl.locale, 'ne-Deva-NP');
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
      () => ctx.keys.id('graha.SUNN'),
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
      () => ctx.calendar.fixedOf(gregorian(2023, 2, 29)),
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
    final bs = ctx.calendar.convert(date, Calendar.bikramSambat);
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

    final fixed = ctx.calendar.fixedOf(date);
    expect(fixed, 735702);
    expect(ctx.calendar.weekdayOf(date), 2, reason: 'a Tuesday');
    expect(ctx.calendar.dateOf(Calendar.gregorian, fixed).era, Era.commonEra);
    expect(ctx.calendar.monthLength(Calendar.gregorian, 2024, 2), 29);
    expect(ctx.calendar.isLeap(Calendar.gregorian, 2024), isTrue);
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
    final resolved = ctx.time.resolve(civil, zone);
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

    final back = ctx.time.civilOf(
      resolved.instantJdUtc,
      zone,
      Calendar.gregorian,
    );
    expect(back.civil.date.year, 1986);
    expect(back.civil.time.minute, 20);
    expect(back.civil.time.hasTime, isTrue);
    expect(back.resolution.offsetSeconds, 20700);

    expect(
      () => ctx.time.resolve(civil, ianaZone('Asia/Kathmandou')),
      throwsA(isA<TeistroException>()),
    );
  });

  test('the time scales convert with what they applied', () {
    final ctx = context();
    final tt = ctx.time.convert(2451544.5, Scale.utc, Scale.tt);
    expect(
      tt.deltaTSeconds,
      closeTo(64.184, 1e-9),
      reason: 'exact through the leap-second table',
    );
    expect(tt.deltaTSource, DeltaTSource.leapSeconds);
    expect(tt.deltaTModel, 'TABLE_THEN_MODEL');
    expect(tt.jd, closeTo(2451544.5 + 64.184 / 86400, 1e-12));
    expect(
      ctx.time.convert(tt.jd, Scale.tt, Scale.utc).jd,
      closeTo(2451544.5, 1e-9),
    );

    final delta = ctx.time.deltaT(2451544.5);
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
    final rendered = ctx.intl.render('sdk.reason.grahaInBhava', {
      'graha': {r'$entity': 'graha.JUPITER'},
      'bhava': 7,
    });
    expect(rendered.from, 'ne-Deva-NP');
    expect(rendered.fallback, isFalse);
    expect(rendered.override, isFalse);
    expect(rendered.warningList, isEmpty);
    expect(rendered.text, contains('७'), reason: 'the Nepali numeral seven');
    expect(ctx.intl.has('sdk.reason.grahaInBhava'), isTrue);
    expect(ctx.intl.has('sdk.nope.missing'), isFalse);

    // A missing message renders as its key with a warning, never an error.
    final missing = ctx.intl.render('sdk.nope.missing');
    expect(missing.from, isNull);
    expect(missing.warningList, isNotEmpty);

    ctx.intl.locale = 'en-Latn';
    expect(ctx.intl.locale, 'en-Latn');
    expect(
      ctx.intl.render('sdk.reason.grahaInBhava', {
        'graha': {r'$entity': 'graha.JUPITER'},
        'bhava': 7,
      }).text,
      contains('Jupiter'),
    );
    expect(
      () => ctx.intl.locale = 'fr-Latn',
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
    final id = ctx.keys.id('graha.SUN');
    expect(
      id,
      1 << 16,
      reason: 'the kind in the high half, the member in the low',
    );
    expect(ctx.keys.name(id), 'graha.SUN');
    expect(
      ctx.keys.name(ctx.keys.id('nakshatra.ASHWINI')),
      'nakshatra.ASHWINI',
    );
    expect(
      () => ctx.keys.name(0xffffffff),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.status,
          'status',
          Status.unsupported,
        ),
      ),
    );
  });

  test('a disposed context says so, and disposing twice is allowed', () {
    final ctx = context();
    expect(ctx.profile, isNotEmpty);
    ctx.dispose();
    // Idempotent: the finaliser and an explicit call both run it.
    ctx.dispose();
    // Named here rather than at the boundary, which would only say
    // `invalid argument` and not which argument. The Node and Python
    // bindings answer the same way.
    expect(
      () => ctx.profile,
      throwsA(
        isA<StateError>().having(
          (e) => e.message,
          'message',
          contains('disposed'),
        ),
      ),
    );
    expect(
      () => ctx.positions(instants: [2451545.0], bodies: [Body.sun]),
      throwsA(isA<StateError>()),
    );
  });

  test('a birth with no time is refused, or reported, but never guessed', () {
    final day = Calendar.bikramSambat.date(2042, 9, 17);
    final zone = ianaZone('Asia/Kathmandu');

    // No policy: refused by name, with the hint naming the three choices.
    final strict = context(locale: null);
    addTearDown(strict.dispose);
    expect(
      () => strict.time.resolve(day.whenUnknown, zone),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.message, 'message', contains('has no time of day'))
            .having(
              (e) => e.hint,
              'hint',
              contains('NOON, MIDNIGHT or SUNRISE'),
            )
            .having((e) => e.field, 'field', 'time'),
      ),
    );

    // NOON: answered, and said twice — the resolution reports the time as
    // unknown *and* warns, so a stored chart cannot claim a time it never
    // had.
    final noon = context(
      locale: null,
      settings: const {
        'time': {'unknown_time': 'NOON'},
      },
    );
    addTearDown(noon.dispose);
    final resolved = noon.time.resolve(day.whenUnknown, zone);
    expect(resolved.timeKnown, isFalse);
    expect(
      resolved.warnings.map((w) => w.key),
      contains('time-unknown-fallback'),
    );
    expect(resolved.instantJdUtc.isFinite, isTrue);

    // A known time on the same date resolves with the time known and no
    // warning: this record sits on the day Nepal moved to +05:45.
    final exact = strict.time.resolve(day.at(hour: 0, minute: 20), zone);
    expect(exact.timeKnown, isTrue);
    expect(exact.offsetSeconds, 5 * 3600 + 45 * 60);
    expect(exact.warnings, isEmpty);
  });

  _engineTests();
}

// ── The engine's own operations ──────────────────────────────────────
// The test provider ships a two-function manifest, so these run with no
// real engine present and still exercise the whole route.

void _engineTests() {
  test('the engine names its own operations', () {
    final engine = context().engine;
    expect(engine.names, contains('tp_echo'));
    expect(engine.has('tp_sum'), isTrue);
    expect(engine.manifest['engine'], 'test-provider');
  });

  test('an operation is called by the name the engine gives it', () {
    final engine = context().engine;
    expect(engine('tp_echo', {'value': 6.0}), {'value': 6.0});
    final summed =
        engine('tp_sum', {
              'values': [1.0, 2.0, 3.5],
            })
            as Map<String, Object?>;
    expect(summed['total'], 6.5);
  });

  test('the manifest carries the role of every parameter', () {
    final engine = context().engine;
    final signature = engine.signature('tp_sum')!;
    final roles = [
      for (final param in signature['params'] as List<Object?>)
        (param as Map<String, Object?>)['role'],
    ];
    expect(roles, ['array_in', 'array_len', 'scalar_out']);
    expect(engine.signature('tm_no_such_thing'), isNull);
  });

  test("the engine's own refusal comes back", () {
    final engine = context().engine;
    expect(
      () => engine('tm_eclipse_when'),
      throwsA(predicate((e) => '$e'.contains('tm_eclipse_when'))),
    );
  });

  // An area is a **value**: built once with the context, held, and
  // passable to something that needs only that much of the SDK. That is
  // what makes the grouping worth having rather than merely tidy
  // (`03-design/surface-areas.md`).
  /// **An engine, plugged in** (ADR-0029): the 98% path, where a
  /// consumer names an adapter's platform binary and never sees a
  /// vtable.
  ///
  /// It runs only where the adapter has been built and its data is
  /// present, because a checkout has neither and a test that failed for
  /// that would fail for everyone. `TEISTRO_TEIMERIS_ADAPTER` names the
  /// library — the same variable `crates/ffi/tests/abi.rs` reads for the
  /// same reason.
  test('an ephemeris is plugged in by naming its platform binary', () {
    final plugin = Platform.environment['TEISTRO_TEIMERIS_ADAPTER'];
    if (plugin == null || plugin.isEmpty) {
      printOnFailure('set TEISTRO_TEIMERIS_ADAPTER to the adapter\'s library');
      markTestSkipped('the adapter is not built in this checkout');
      return;
    }
    final ctx = teistro.context(
      profile: 'parashari-classical',
      ephemeris: [PluginEphemeris(plugin: plugin)],
    );
    addTearDown(ctx.dispose);
    final sky = ctx.positions(instants: [2451545.0], bodies: [Body.sun]);
    // The Sun at J2000 is near 280.4°, which is astronomy rather than
    // this package: what is tested is that a real engine answered.
    expect(sky.at(0, 0).longitude, closeTo(280.37, 0.5));
    // And its own functions came with it, which no SDK operation offers.
    expect(ctx.engine.manifest['engine'], 'teimeris');
    expect(
      (ctx.engine('tm_body_name', {'body': 0}) as Map<String, Object?>)['buf'],
      'Sun',
    );
  });

  /// A chain is **ordered and explicit** (ADR-0029): tried in order, and
  /// a refusal names every entry that failed rather than only the last,
  /// which would hide the one the caller actually wanted. Needs no
  /// adapter.
  test('an ephemeris chain is tried in order and refuses naming each', () {
    // An adapter that is not there, then the built-in: the fallback the
    // caller wrote down.
    final fellBack = teistro.context(
      profile: 'parashari-classical',
      ephemeris: const [
        PluginEphemeris(plugin: '/nowhere/adapter.so'),
        NamedEphemeris(Ephemeris.builtin),
      ],
    );
    addTearDown(fellBack.dispose);
    expect(
      fellBack
          .positions(instants: [2451545.0], bodies: [Body.sun])
          .at(0, 0)
          .longitude,
      closeTo(280.37, 0.5),
    );

    // Nothing in the chain opening is one refusal that names each.
    expect(
      () => teistro.context(
        ephemeris: const [
          PluginEphemeris(plugin: '/a.so'),
          PluginEphemeris(plugin: '/b.so'),
        ],
      ),
      throwsA(
        predicate((e) => '$e'.contains('/a.so') && '$e'.contains('/b.so')),
      ),
    );

    // A chain of none names nothing, which is a mistake rather than a
    // default.
    expect(
      () => teistro.context(ephemeris: const []),
      throwsA(predicate((e) => '$e'.contains('names nothing'))),
    );
  });

  test('an area is a value that can be held and passed', () {
    final ctx = context();
    final calendar = ctx.calendar;
    expect(calendar, same(ctx.calendar), reason: 'the same object every read');
    expect(calendar.isLeap(Calendar.gregorian, 2024), isTrue);
    expect(ctx.time.deltaT(2451545.0).seconds, greaterThan(60));
    expect(ctx.keys.name(ctx.keys.id('graha.SUN')), 'graha.SUN');
  });
}
