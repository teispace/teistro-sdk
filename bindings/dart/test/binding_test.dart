// The Dart binding end to end: the same scenario the C binding's smoke
// test and the Node binding's tests walk, through the generated
// declarations and the ergonomic layer.
//
// `cargo xtask check-dart` builds the shared library and runs this file;
// `TEISTRO_LIBRARY` names it.

import 'dart:io';
import 'dart:typed_data';

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final Teistro teistro = Teistro.open();

/// The fixture directory `cargo xtask check-dart` names, and a reader for
/// the files in it — the convention `blob_test.dart` follows.
final String _fixtures =
    Platform.environment['TEISTRO_FIXTURES'] ?? '../../target/tsrb';
Uint8List _readFixture(String name) =>
    File('$_fixtures/$name').readAsBytesSync();

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

  test('a pack loads at runtime and lays its record over the one standing', () {
    // The fixture pack is the base locale's, so this context reads that
    // one: a record is a locale's, and loading into `en-Latn` does not
    // touch what `ne-Deva-NP` says of the same key.
    final ctx = context(locale: 'en-Latn');
    final before = ctx.intl.entity('graha.SUN');
    expect(before.name, isNotEmpty, reason: 'the engine names the Sun');
    expect(before.forms['phala'], isNull);

    final loaded = ctx.intl.loadPack(_readFixture('overlay.tpack'));
    expect(loaded.entries, 1);
    expect(loaded.replaced, 0, reason: 'nothing was thrown away');
    expect(loaded.merged, 1, reason: 'the record kept what the pack lacked');
    expect(loaded.locale, 'en-Latn');
    expect(loaded.sha256, matches(RegExp(r'^[0-9a-f]{64}$')));

    // A record's forms are an open set, so a form no locale of `i18n/`
    // carries is reached through `forms` and not by a named field.
    final after = ctx.intl.entity('graha.SUN');
    expect(after.forms['phala'], 'a reading of the Sun');
    expect(after.name, before.name);
    expect(after.forms['name'], before.name);
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
    // No context exists to keep these refusals, so the record crosses
    // whole from the call that failed (ffi-abi-and-api-description.md
    // §6.1).
    expect(
      () => context(profile: 'vedic-classic'),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.status, 'status', Status.unsupported)
            .having(
              (e) => e.message,
              'message',
              contains('no shipped profile `vedic-classic`'),
            )
            .having((e) => e.field, 'field', 'profile')
            .having((e) => e.hint, 'hint', contains('parashari-classical')),
      ),
    );
    expect(
      () => context(locale: 'xx-Latn'),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.field, 'field', 'locale')
            .having((e) => e.hint, 'hint', contains('ne-Deva-NP')),
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

    // Without an ephemeris the call is a missing capability naming the
    // option a consumer sets, and hinting at what to pass -- the same
    // field and the same hint as Node, Python and C.
    final bare = context(testProvider: false);
    expect(
      () => bare.positions(instants: [2451545.0], bodies: [Body.sun]),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.status, 'status', Status.capability)
            .having((e) => e.field, 'field', 'ephemeris')
            .having((e) => e.hint, 'hint', contains('builtin')),
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

  test('a rendered message carries its markup in parts, and plain text '
      'in one', () {
    final ctx = context();
    ctx.intl.locale = 'en-Latn';
    // `sdk.reason.lordship` is one of the two shipped messages that use
    // MF2 markup. Without the parts a renderer can only ever print the
    // text, which is why the message may as well not have had the tag.
    final rich = ctx.intl.render('sdk.reason.lordship', {
      'graha': {r'$entity': 'graha.JUPITER'},
      'bhava': 5,
    });
    expect(rich.text, 'Jupiter rules house 5');
    final parts = rich.partList;
    expect(parts.map((p) => p.toString()).toList(), [
      '<open b>',
      'Jupiter',
      '<close b>',
      ' rules house 5',
    ]);
    expect(parts.first.options, isEmpty);
    // A renderer that knows no tag joins the text parts and loses nothing.
    expect(parts.where((p) => p.isText).map((p) => p.value).join(), rich.text);

    // A message with no markup is the one text part, made here rather
    // than carried: the boundary sends nothing for it.
    final plain = ctx.intl.render('sdk.reason.grahaInBhava', {
      'graha': {r'$entity': 'graha.JUPITER'},
      'bhava': 7,
    });
    expect(plain.parts, '[]', reason: 'the boundary sent none');
    expect(plain.partList.single.value, plain.text);
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
      printOnFailure(
        'the adapter is built separately; set TEISTRO_TEIMERIS_ADAPTER to its library',
      );
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

  test('a theme writes each drawing as SVG, and a wrong one is refused by its '
      'field', () {
    final ctx = context();
    List<Drawing> found(ChartTheme? theme) =>
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              drawings: const [
                (ChartLayout.northIndian, Varga.d1),
                (ChartLayout.westernWheel, Varga.d1),
              ],
              theme: theme,
            )
            .drawings;

    expect(found(null).first.svg, isNull, reason: 'no theme, no SVG');
    final [north, wheel] = found(ChartTheme.dark);
    expect(north.svg, startsWith('<svg xmlns="http://www.w3.org/2000/svg"'));
    expect(north.svg, contains('data-body="graha.SUN">सू'));
    expect(north.svg, contains('fill="#121212"'));
    expect(wheel.svg, contains('<line '));

    final glyphs =
        found(
          ChartTheme.light.copyWith(
            style: const ThemeStyle(size: 600),
            content: const ThemeContent(
              bodyForm: BodyForm.glyph,
              cellLabel: CellLabel.house,
              retrogradeMark: '',
            ),
          ),
        ).first.svg;
    expect(glyphs, contains('viewBox="0 0 600 600"'));
    expect(glyphs, contains('data-body="graha.SUN">☉'));

    expect(
      () => found(
        ChartTheme.light.copyWith(style: const ThemeStyle(ink: 'black')),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'theme_json.style.ink',
        ),
      ),
    );
    ctx.dispose();
  });

  test('rules are answered in the same crossing, and a wrong one is refused '
      'by its field', () {
    final ctx = context();
    Map<String, Object?>? found(RuleRequest? rules) =>
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              rules: rules,
            )
            .rules;

    expect(found(null), isNull, reason: 'no rules, no answers');
    final answered =
        found(
          const RuleRequest(shipped: [ShippedRules.nabhasas], longevity: true),
        )!;
    final present = answered['present']! as List<Object?>;
    expect(present, isNotEmpty);
    for (final held in present.cast<Map<String, Object?>>()) {
      expect(held['rule'], isA<String>());
      expect((held['result']! as Map<String, Object?>)['present'], isTrue);
    }
    final longevity = answered['longevity']! as Map<String, Object?>;
    final ayurdaya = longevity['ayurdaya']! as Map<String, Object?>;
    expect((ayurdaya['pindayu']! as Map<String, Object?>)['years'], isA<num>());

    final first = (present.first! as Map<String, Object?>)['rule']! as String;
    final withMine =
        found(
          RuleRequest(
            shipped: const [ShippedRules.nabhasas],
            rules: [
              <String, Object?>{
                'key': 'MINE',
                'category': 'raja',
                'source': <String, Object?>{'text': 'BPHS'},
                'conditions': [
                  <String, Object?>{'type': 'rule', 'key': first},
                ],
              },
            ],
          ),
        )!;
    expect(
      (withMine['present']! as List<Object?>).cast<Map<String, Object?>>().any(
        (held) => held['rule'] == 'MINE',
      ),
      isTrue,
    );

    expect(
      () => found(
        const RuleRequest(
          rules: [
            <String, Object?>{'key': 'X', 'category': 'raja'},
          ],
        ),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'rules_json.rules[0]',
        ),
      ),
    );
    ctx.dispose();
  });

  test('plans compose in the same crossing, and render with nothing in '
      'between', () {
    final ctx = context();
    Map<String, Object?>? found(PlanRequest? interpret, {RuleRequest? rules}) =>
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              rules: rules,
              interpret: interpret,
            )
            .plans;

    expect(found(null), isNull, reason: 'no composer, no plans');
    final plans =
        found(
          const PlanRequest(
            placements: true,
            readings: true,
            strength: true,
            houses: true,
            positions: true,
            aspects: true,
            conditions: true,
            karakas: true,
            chalit: true,
            states: true,
            bhavaBala: true,
            vimshopaka: true,
            panchanga: true,
            dashaPhala: true,
            ashtakavarga: true,
          ),
          rules: const RuleRequest(shipped: [ShippedRules.nabhasas]),
        )!;
    final placements = plans['placements']! as List<Object?>;
    expect(placements, isNotEmpty, reason: 'every chart places its grahas');
    expect(plans['readings'], isA<List<Object?>>());
    // The strengths read the Shadbala, which this request never asked for:
    // a composer's own section is computed for it.
    final weighed = plans['strength']! as List<Object?>;
    expect(
      weighed,
      hasLength(7 * 2),
      reason: 'the seven grahas, a score and a sufficiency each',
    );
    // And the houses read the bhavas, which it never asked for either.
    final ruled = plans['houses']! as List<Object?>;
    expect(ruled, hasLength(12 * 2), reason: 'a sign and a lord each');
    // And the positions read the same states the placements do.
    final degrees = plans['positions']! as List<Object?>;
    expect(degrees, hasLength(10), reason: 'the lagna, then the nine grahas');
    // The chalit needs no section: both readings are on the grahas.
    final shifts = plans['chalit']! as List<Object?>;
    expect(shifts.length, lessThanOrEqualTo(9), reason: 'at most one a graha');
    // The drishtis read the aspects section, never asked for either.
    final looks = plans['aspects']! as List<Object?>;
    expect(looks, isNotEmpty, reason: 'every chart holds a drishti');
    // The conditions and the karakas read the same states the placements
    // do, so one section serves four composers.
    final states = plans['conditions']! as List<Object?>;
    expect(states.length, greaterThanOrEqualTo(18), reason: 'nine of each');
    final karakas = plans['karakas']! as List<Object?>;
    expect(karakas, hasLength(15), reason: 'seven of seven, eight of eight');
    // The dasha phala reads its own section, computed for it like the
    // rest: three items a graha always, and a fourth only where the
    // placement tilts the dasha one way or the other.
    final dashas = plans['dashaPhala']! as List<Object?>;
    expect(dashas.length, greaterThanOrEqualTo(9 * 3), reason: 'three each');
    expect(dashas.length, lessThanOrEqualTo(9 * 4), reason: 'and four at most');
    // The states say the half of that section a `Placement` never carried.
    final carried = plans['states']! as List<Object?>;
    expect(
      carried.length,
      greaterThanOrEqualTo(9 * 3),
      reason: 'three a graha at least',
    );
    // Each bhava is weighed and never judged, and the Vimshopaka says all
    // four schemes.
    final houses = plans['bhavaBala']! as List<Object?>;
    expect(houses, hasLength(12), reason: 'one a bhava');
    final scored = plans['vimshopaka']! as List<Object?>;
    expect(scored, hasLength(7 * 4), reason: 'seven grahas, four schemes');
    // The almanac says the five limbs and the Moon's pada.
    final almanac = plans['panchanga']! as List<Object?>;
    expect(almanac.length, greaterThanOrEqualTo(6), reason: 'limbs and pada');
    expect(almanac.length, lessThanOrEqualTo(7), reason: 'and the day');
    // The Ashtakavarga reads its own section *and* the placements: a
    // graha's bindus are the ones of the sign it stands in.
    final bindus = plans['ashtakavarga']! as List<Object?>;
    expect(bindus, hasLength(7 + 12), reason: 'a graha each, then a sign each');

    // Each item said by handing its params straight to the renderer, which
    // is the property the crossing exists for.
    var said = 0;
    for (final item
        in [
          ...placements,
          ...plans['readings']! as List<Object?>,
          ...weighed,
          ...ruled,
          ...degrees,
          ...looks,
          ...states,
          ...karakas,
          ...dashas,
          ...carried,
          ...almanac,
          ...houses,
          ...scored,
        ].cast<Map<String, Object?>>()) {
      final key = item['key']! as String;
      expect(key, startsWith('sdk.'));
      final rendered = ctx.intl.render(
        key,
        item['params']! as Map<String, Object?>,
      );
      expect(rendered.text, isNotEmpty, reason: '$key said nothing');
      expect(rendered.isFallback, 0, reason: '$key fell back');
      expect(rendered.warningCount, 0, reason: '$key warned');
      said += 1;
    }
    expect(said, greaterThan(20), reason: 'only $said items said');

    // A reading says what the rules answered, so it needs rules beside it.
    expect(
      () => found(const PlanRequest(readings: true)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'interpret_json.readings',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a layout of your own is registered, drawn by its key, and refused by '
      'its field', () {
    final base = context();
    final row = base.chart.layout(ChartLayout.southIndian);
    expect(row.key, 'SOUTH_INDIAN');
    expect(row.shape, isA<GridShape>());
    expect(
      row.toJson(),
      LayoutRow.fromJson(row.toJson()).toJson(),
      reason: 'a row reads back as it was written',
    );
    expect(
      () => base.chart.layout(ChartLayout.registered('ACME_KERALA')),
      throwsA(isA<TeistroException>().having((e) => e.field, 'field', 'key')),
    );
    base.dispose();

    final kerala = row.copyWith(key: 'ACME_KERALA');
    final own = ChartLayout.registered('ACME_KERALA');
    final ctx = teistro.context(
      profile: 'nepali-default',
      testProvider: true,
      layouts: [kerala],
    );
    expect(ctx.chart.layout(own).toJson(), kerala.toJson());
    expect(ctx.keys.name(ctx.keys.id(own.fullKey)), own.fullKey);

    final [south, drawn] =
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              drawings: [(ChartLayout.southIndian, Varga.d1), (own, Varga.d1)],
            )
            .drawings;
    expect(drawn.layout, own);
    expect(south.layout, ChartLayout.southIndian);
    expect(
      [for (final cell in drawn.cells) cell.sign],
      [for (final cell in south.cells) cell.sign],
    );
    expect(
      () => ctx.chart.found(
        instant: 2451545.0,
        place: Observer(
          latitudeDeg: Latitude(0),
          longitudeDeg: Longitude(0),
          altitudeM: Altitude(0),
        ),
        utcOffsetSeconds: 0,
        drawings: [(ChartLayout.registered('ACME_ODIA'), Varga.d1)],
      ),
      throwsArgumentError,
    );
    ctx.dispose();

    Matcher field(String name) =>
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', name));
    expect(
      () => teistro.context(testProvider: true, layouts: [kerala, row]),
      field('options.layouts_json[1].key'),
    );
    expect(
      () => teistro.context(
        testProvider: true,
        layouts: [kerala.copyWith(sources: const [])],
      ),
      field('options.layouts_json[0].sources'),
    );
  });

  /// A chart's Ashtakavarga crosses whole: each graha's bindus holding the
  /// classical totals, the sum, and each graha's reductions under the default
  /// reading; null unless asked.
  test('a chart carries its Ashtakavarga and each graha\'s reductions', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      ashtakavarga: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .ashtakavarga,
      isNull,
    );
    final av = chart.ashtakavarga!;
    expect(
      (av.shodhana, av.ekadhipatya),
      (Shodhana.eachGraha, Ekadhipatya.bphs),
    );
    expect(
      [for (final g in av.grahas) g.bindus.reduce((a, b) => a + b)],
      [48, 49, 39, 54, 56, 52, 39],
    );
    expect(av.sarva.reduce((a, b) => a + b), 337);
    expect(av.reduced, [
      for (var sign = 0; sign < 12; sign += 1)
        av.grahas.fold<int>(0, (sum, g) => sum + g.reduced![sign]),
    ]);
    expect(
      av.grahas.every((g) => g.yogaPinda == g.rashiPinda + g.grahaPinda),
      isTrue,
    );
    ctx.dispose();
  });

  /// A chart's Bhava bala crosses whole: every bhava's components under the
  /// default reading, the verses', whose totals are their parts'; null unless
  /// asked.
  test('a chart carries its Bhava bala, each bhava\'s strength', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      bhavaBala: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .bhavaBala,
      isNull,
    );
    final bhavas = chart.bhavaBala!.bhavas;
    expect([for (final b in bhavas) b.bhava], List.generate(12, (i) => i + 1));
    for (final b in bhavas) {
      expect(
        b.adhipati + b.dig + b.drishti + b.special,
        closeTo(b.virupas, 1e-9),
      );
      expect(b.dig, inInclusiveRange(0.0, 60.0));
    }
    ctx.dispose();
  });

  /// A chart's Shadbala crosses whole: every graha's six strengths under the
  /// default reading, the chapter's, whose natural strengths are 28 sevenths
  /// of a rupa and whose totals are their components'; null unless asked.
  test('a chart carries its Shadbala, each graha\'s six strengths', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      shadbala: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .shadbala,
      isNull,
    );
    final grahas = chart.shadbala!.grahas;
    expect(grahas, hasLength(7));
    expect(
      grahas.fold<double>(0, (sum, g) => sum + g.naisargika),
      closeTo(240, 1e-9),
    );
    for (final g in grahas) {
      final six =
          g.sthana.total +
          g.dig +
          g.kaala.total +
          g.cheshta +
          g.naisargika +
          g.drik;
      expect(six, closeTo(g.virupas, 1e-9), reason: '${g.graha}');
      expect(g.strong, g.rupas >= g.requiredRupas);
    }
    ctx.dispose();
  });

  /// A chart's dasha phala crosses whole: the nine grahas' Subhankas within
  /// each varga's share and complementary in total; null unless asked, and
  /// the Shadbala's rays beside the phalas.
  test('a chart carries its dasha phala, and the Shadbala its rays', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      dashaPhala: true,
      shadbala: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .dashaPhala,
      isNull,
    );
    final grahas = chart.dashaPhala!.grahas;
    expect(grahas, hasLength(9));
    expect(grahas.last.graha, Graha.ketu);
    for (final g in grahas) {
      expect(g.subhankas, hasLength(7));
      for (var k = 0; k < 7; k++) {
        expect(g.subhankas[k], inInclusiveRange(0, k == 0 ? 60 : 30));
      }
      expect(g.subhanka + g.asubhanka, closeTo(240, 1e-9));
    }
    for (final s in chart.shadbala!.grahas) {
      expect(s.subhaRashmi, inInclusiveRange(1, 7));
      expect(s.subhaRashmi + s.ashubhaRashmi, closeTo(8, 1e-9));
    }
    ctx.dispose();
  });

  /// Every graha's state carries its Sayanadi: the nine grahas a state and a
  /// sub-state under each of the five ankas, the outer planets none.
  test(
    'a graha\'s state carries its Sayanadi and a sub-state for every anka',
    () {
      final ctx = context();
      final place = Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      );
      final states =
          ctx.chart
              .found(
                instant: 2451545.0,
                place: place,
                utcOffsetSeconds: 20700,
                state: true,
              )
              .states;
      const nine = {
        Graha.sun,
        Graha.moon,
        Graha.mars,
        Graha.mercury,
        Graha.jupiter,
        Graha.venus,
        Graha.saturn,
        Graha.rahu,
        Graha.ketu,
      };
      for (final state in states) {
        final sayanadi = state.sayanadi;
        if (!nine.contains(state.graha)) {
          expect(sayanadi, isNull, reason: state.graha.fullKey);
          continue;
        }
        expect(sayanadi, isNotNull, reason: state.graha.fullKey);
        expect(sayanadi!.cheshtas, hasLength(5));
        expect(sayanadi.cheshta(3), sayanadi.cheshtas[2]);
        expect(() => sayanadi.cheshta(6), throwsRangeError);
      }
      ctx.dispose();
    },
  );

  /// A chart's Vaiseshikamsa crosses whole: each scheme's count within its
  /// vargas, a name for every count from two; null unless asked.
  test('a chart carries its Vaiseshikamsa, each scheme\'s count and name', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      vaiseshikamsa: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .vaiseshikamsa,
      isNull,
    );
    final grahas = chart.vaiseshikamsa!.grahas;
    expect(grahas, hasLength(7));
    for (final g in grahas) {
      for (final (standing, vargas) in [
        (g.shadvarga, 6),
        (g.saptavarga, 7),
        (g.dashavarga, 10),
        (g.shodashavarga, 16),
      ]) {
        expect(standing.goodVargas, lessThanOrEqualTo(vargas));
        expect(standing.name == null, standing.goodVargas < 2);
      }
    }
    ctx.dispose();
  });

  /// A chart's Vimshopaka crosses whole: every graha's four scores out of 20
  /// under the default reading, the text's, whose least in any varga is 5;
  /// null unless asked.
  test('a chart carries its Vimshopaka, each graha\'s four scores', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      vimshopaka: true,
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .vimshopaka,
      isNull,
    );
    final vs = chart.vimshopaka!;
    expect(vs.scoring, VimshopakaScoring.bphs);
    expect(
      [for (final g in vs.grahas) g.graha],
      [
        Graha.sun,
        Graha.moon,
        Graha.mars,
        Graha.mercury,
        Graha.jupiter,
        Graha.venus,
        Graha.saturn,
      ],
    );
    for (final g in vs.grahas) {
      for (final score in [
        g.shadvarga,
        g.saptavarga,
        g.dashavarga,
        g.shodashavarga,
      ]) {
        expect(score, inInclusiveRange(5.0, 20.0), reason: '${g.graha}');
      }
    }
    ctx.dispose();
  });

  /// A chart's dashas cross whole: the balance, the periods to the
  /// settings' depth with their paths, and the chain at an instant read off
  /// them.
  /// A consumer's own dasha system crosses: registered on the context, asked
  /// for by its key, named by it in the answer, and every period its
  /// catalogued twin's; a definition the checks refuse is named by its place
  /// and field.
  test('a consumer dasha system registers and reads as its twin', () {
    final twin = UduDashaDefinition(
      key: 'ACME_VIMSHOTTARI',
      lords: [
        const DashaLord(Graha.ketu, 7),
        const DashaLord(Graha.venus, 20),
        const DashaLord(Graha.sun, 6),
        const DashaLord(Graha.moon, 10),
        const DashaLord(Graha.mars, 7),
        const DashaLord(Graha.rahu, 18),
        const DashaLord(Graha.jupiter, 16),
        const DashaLord(Graha.saturn, 19),
        const DashaLord(Graha.mercury, 17),
      ],
      reference: Nakshatra.ashwini,
    );
    const signTwin = RashiDashaDefinition(key: 'ACME_CHARA');
    final ctx = teistro.context(
      testProvider: true,
      dashaSystems: [twin, signTwin],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      dashas: [
        DashaSystem.registered('ACME_VIMSHOTTARI'),
        DashaSystem.vimshottari,
      ],
    );
    final [consumer, shipped] = chart.dashas;
    expect(consumer.system, DashaSystem.registered('ACME_VIMSHOTTARI'));
    expect(shipped.system, DashaSystem.vimshottari);
    expect(consumer.periods.length, shipped.periods.length);
    for (var i = 0; i < shipped.periods.length; i++) {
      final (a, b) = (consumer.periods[i], shipped.periods[i]);
      expect((a.path, a.lord, a.from, a.to), (b.path, b.lord, b.from, b.to));
    }
    // The other kernel, the same way: a sign-based system of one's own
    // answers as the catalogued row it copies.
    final signs = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      dashas: [DashaSystem.registered('ACME_CHARA'), DashaSystem.chara],
    );
    final [own, chara] = signs.dashas;
    expect(own.system, DashaSystem.registered('ACME_CHARA'));
    expect(chara.system, DashaSystem.chara);
    expect(own.periods.length, chara.periods.length);
    for (var i = 0; i < chara.periods.length; i++) {
      final (a, b) = (own.periods[i], chara.periods[i]);
      expect((a.path, a.lord, a.from, a.to), (b.path, b.lord, b.from, b.to));
    }
    expect(
      () => ctx.chart.found(
        instant: 2451545.0,
        place: place,
        utcOffsetSeconds: 20700,
        dashas: [DashaSystem.registered('ACME_OTHER')],
      ),
      throwsArgumentError,
    );
    ctx.dispose();
    expect(
      () => teistro.context(
        testProvider: true,
        dashaSystems: [
          const RashiDashaDefinition(key: 'ACME_THIRTEEN', strongerOf: [1, 13]),
        ],
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'options.dashas_json[0].stronger_of[1]',
        ),
      ),
    );
    expect(
      () => teistro.context(
        testProvider: true,
        dashaSystems: [
          UduDashaDefinition(
            key: twin.key,
            lords: twin.lords,
            reference: twin.reference,
            span: 0,
          ),
        ],
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'options.dashas_json[0].span',
        ),
      ),
    );
  });

  /// The annual charts cross: a request's `varsha` answers each chart's
  /// returns in year order, ragged per chart, and the instant founds as a
  /// chart of its own.
  test('a chart carries the annual charts its birth opens', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
      varsha: const VarshaRequest(through: 12),
    );
    final years = chart.praveshas;
    expect(years.length, 12);
    expect(
      [for (final one in years) one.year],
      [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    );
    expect(years.first.instant, greaterThan(2447995.4895833335));
    // Eleven sidereal years between the first and the twelfth, to a day.
    final span = years.last.instant - years.first.instant;
    expect((span - 11 * 365.2564).abs(), lessThan(1));

    // The Muntha advances one sign a year from the birth lagna, which is
    // the whole of its rule; its lord is that sign's.
    final natal = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
    );
    final lagna = natal.lagnaDeg ~/ 30;
    for (final one in years) {
      expect(one.muntha.sign.id, (lagna + one.year) % 12);
      expect(one.muntha.lord, isNot(Graha.unknown));
    }
    // Twelve years is a whole circle back to the lagna's own sign.
    expect(years.last.muntha.sign, Rashi.byId(lagna % 12));

    // The two readings of the degree agree on the sign and part inside it.
    final carried = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
      varsha: const VarshaRequest(
        through: 12,
        muntha: MunthaDegree.natalDegree,
      ),
    );
    for (var i = 0; i < years.length; i += 1) {
      expect(carried.praveshas[i].muntha.sign, years[i].muntha.sign);
      expect(
        carried.praveshas[i].muntha.longitudeDeg,
        greaterThanOrEqualTo(years[i].muntha.longitudeDeg),
      );
    }

    // No place, no chart founded: the instants alone, as before.
    expect(years.every((one) => one.annual == null), isTrue);

    // At the birthplace each year's chart comes back with its five
    // office-bearers; the birth lagna's lord is shared by every year and the
    // Muntha's lord is the one already on the return.
    final cast =
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: const VarshaRequest(
                through: 12,
                place: AnnualPlace.birth,
              ),
            )
            .praveshas;
    for (final one in cast) {
      expect(one.annual, isNotNull);
      expect(
        one.annual!.officeBearers.janmaLagna,
        cast.first.annual!.officeBearers.janmaLagna,
      );
      expect(one.annual!.officeBearers.muntha, one.muntha.lord);
    }
    final again = ctx.chart.found(
      instant: cast[3].instant,
      place: place,
      utcOffsetSeconds: 20700,
    );
    expect(again.lagnaDeg, cast[3].annual!.lagnaDeg);

    // Each year names a lord, chosen among its own claimants.
    for (final one in cast) {
      final lord = one.annual!.yearLord;
      expect(lord.claims.length, inInclusiveRange(1, 5));
      expect(lord.claims.map((claim) => claim.graha), contains(lord.graha));
      final ranked = [for (final claim in lord.claims) claim.vishwa.total];
      final sorted = [...ranked]..sort((int a, int b) => b - a);
      expect(ranked, orderedEquals(sorted));
      expect(lord.vishwa.toString(), matches(r'^\d\d:\d\d:\d\d$'));
      expect(lord.vishwa.units, lord.vishwa.total ~/ 3600);
    }

    // At a residence, in the parts `found` takes, the lagnas move.
    final delhi =
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: VarshaRequest(
                through: 12,
                place: AnnualPlace.at(
                  Observer(
                    latitudeDeg: Latitude(28.6139),
                    longitudeDeg: Longitude(77.209),
                    altitudeM: Altitude(216),
                  ),
                  utcOffsetSeconds: 19800,
                ),
              ),
            )
            .praveshas;
    for (var i = 0; i < delhi.length; i += 1) {
      expect(
        (delhi[i].annual!.lagnaDeg - cast[i].annual!.lagnaDeg).abs(),
        greaterThan(0.1),
      );
    }

    // The instant founds as a chart of its own; the place is the caller's.
    final annual = ctx.chart.found(
      instant: years.last.instant,
      place: place,
      utcOffsetSeconds: 20700,
    );
    expect(annual.instant, years.last.instant);

    // Not asked for is empty, not zeroes.
    expect(
      ctx.chart
          .found(
            instant: 2447995.4895833335,
            place: place,
            utcOffsetSeconds: 20700,
          )
          .praveshas,
      isEmpty,
    );

    // The rivals are asked for by name and are not the same instant.
    final tropical =
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: const VarshaRequest(
                through: 12,
                reading: VarshaReading.tropical,
              ),
            )
            .praveshas;
    expect(tropical.last.instant, isNot(years.last.instant));

    expect(
      () => ctx.chart.found(
        instant: 2447995.4895833335,
        place: place,
        utcOffsetSeconds: 20700,
        varsha: const VarshaRequest(through: 0),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'varsha_json.through',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its dashas, their periods and the chain', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final chart = ctx.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
      dashas: [DashaSystem.vimshottari, DashaSystem.chara],
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .dashas,
      isEmpty,
    );
    final [dasha, chara] = chart.dashas;
    expect(dasha.system, DashaSystem.vimshottari);
    expect(dasha.balance!.method, Balance.spatial);
    expect(
      dasha.balance!.remaining,
      allOf(greaterThan(0), lessThanOrEqualTo(1)),
    );
    expect(dasha.moonSpan, isNull);
    expect(dasha.depth, 3);
    expect(dasha.periods, hasLength(9 + 81 + 729));
    final [first, second, ...] = dasha.periods;
    expect((first.path, first.level, first.from), ('0', 1, 2451545.0));
    expect(first.lord, dasha.firstLord);
    expect(
      (second.path, second.level, second.lord),
      ('0/0', 2, dasha.firstLord),
    );
    expect(dasha.periods.last.path, '8/8/8');
    expect(
      first.sign,
      isNull,
      reason: "a nakshatra-seeded period is its lord's",
    );

    // A sign-based dasha: no seed, no balance, twelve signs each divided in
    // twelve from its own sign.
    expect(chara.system, DashaSystem.chara);
    expect((chara.seed, chara.balance), (null, null));
    expect(chara.periods, hasLength(12 + 144 + 1728));
    final [maha, own, ...] = chara.periods;
    expect(
      (maha.path, own.path, own.sign, maha.from),
      ('0', '0/0', maha.sign, 2451545.0),
    );
    expect(maha.sign, isNotNull);
    expect(chara.firstLord, maha.lord);
    expect(
      {for (final p in chara.periods.where((p) => p.level == 1)) p.sign},
      hasLength(12),
      reason: 'every sign once',
    );
    expect(chara.at(2451545.0 + 5000), hasLength(3));

    const instant = 2451545.0 + 5000;
    final chain = dasha.at(instant);
    expect([for (final p in chain) p.level], [1, 2, 3]);
    expect(chain.every((p) => p.from <= instant && instant < p.to), isTrue);
    expect(dasha.at(2451544), isEmpty, reason: 'before birth');

    expect(
      () => ctx.chart.found(
        instant: 2451545.0,
        place: place,
        utcOffsetSeconds: 0,
        dashas: [DashaSystem.vimshottari, DashaSystem.sudarshanaChakra],
      ),
      throwsA(
        isA<TeistroException>().having((e) => e.field, 'field', 'dashas[1]'),
      ),
    );
    ctx.dispose();
  });
}
