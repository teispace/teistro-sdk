// The Dart binding end to end: the same scenario the C binding's smoke
// test and the Node binding's tests walk, through the generated
// declarations and the ergonomic layer.
//
// `cargo xtask check-dart` builds the shared library and runs this file;
// `TEISTRO_LIBRARY` names it.

import 'dart:io';
import 'dart:math' as math;
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

  test('the Surya Siddhanta opens by name, and its chart says which parts '
      "are the text's", () {
    // A classical astronomy's chart is the text's throughout
    // (docs/03-design/classical-chart.md), and the envelope says so, where a
    // modern chart's says nothing.
    Chart at(Context ctx) => ctx.chart.found(
      instant: 2447995.4895833335,
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
    );
    final text = teistro.context(
      profile: 'surya-siddhanta',
      ephemeris: const [NamedEphemeris(Ephemeris.suryaSiddhanta)],
    );
    final deviation = at(text).provenance.deviation!;
    expect(deviation.model, 'SURYA_SIDDHANTA');
    expect(
      deviation.detail,
      "the zodiac, the places, the angles and the day are the provider's own",
    );
    text.dispose();
    final modern = context();
    expect(at(modern).provenance.deviation, isNull);
    modern.dispose();
  });

  test('a chart handed out alone carries its own hash, and the batch the '
      "list's", () {
    // STATUS 2h: a batch's provenance hashes the list, and a chart of it the
    // value it holds, which is what a stored chart is checked against.
    final ctx = context();
    final batch = ctx.chart.foundMany(
      instants: [2451545.0, 2451546.0],
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
    );
    final first = batch.at(0).provenance;
    final second = batch.at(1).provenance;
    expect({
      first.contentHash,
      second.contentHash,
      batch.provenance.contentHash,
    }, hasLength(3));
    expect(first.settingsHash, batch.provenance.settingsHash);
    expect(first.sdkVersion.major, batch.provenance.sdkVersion.major);
    expect(() => Confidence.fromJson('MAYBE'), throwsA(isA<FormatException>()));
    ctx.dispose();
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
            .having((e) => e.toString(), 'toString', contains('UNSUPPORTED')),
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

    expect(positions.provenance.profile, 'nepali-default');
    expect(positions.provenance.calculationVersion, 1);
    expect(positions.provenance.settingsHash, ctx.settingsHash);
    expect(
      positions.provenance.provider.frame,
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
            .having((e) => e.hint, 'hint', contains('BUILTIN')),
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
      contains('TIME_UNKNOWN_FALLBACK'),
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
          'theme.style.ink',
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
    expect((ayurdaya['rules']! as Map<String, Object?>)['enmity'], 'natural');
    // The spans read as *Jataka Parijata* reads them, and a choice nothing
    // reads refused.
    final chosen =
        found(
          const RuleRequest(
            longevity: true,
            ayurdaya: AyurdayaRules.parijata,
            threePairs: ThreePairsRules(saturn: SaturnAmongPairs.raises),
          ),
        )!;
    final rules =
        ((chosen['longevity']! as Map<String, Object?>)['ayurdaya']!
                as Map<String, Object?>)['rules']!
            as Map<String, Object?>;
    expect(
      [
        rules['enemy_exempt'],
        rules['enmity'],
        rules['rising'],
        rules['combine'],
      ],
      ['mars', 'compound', 'every', 'largest'],
    );
    expect(
      () => found(const RuleRequest(ayurdaya: AyurdayaRules.parijata)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'rules.ayurdaya',
        ),
      ),
    );
    // The rays, the note's by default, and as the verses read them.
    final rays = longevity['rasmi']! as Map<String, Object?>;
    expect(rays['grahas'], hasLength(7));
    expect(rays['rules'], {'place': 'dwadasamsa'});
    expect(['short', 'medium', 'long'], contains(rays['class']));
    final sun =
        (rays['grahas']! as List<Object?>).first! as Map<String, Object?>;
    expect(
      sun['basic_years']! as num,
      greaterThanOrEqualTo(sun['basic']! as num),
    );
    final verse =
        (found(
                  const RuleRequest(longevity: true, rasmi: RasmiRules.verse),
                )!['longevity']!
                as Map<String, Object?>)['rasmi']!
            as Map<String, Object?>;
    expect(verse['rules'], {'place': 'sign'});
    // v. 33: eight candidates, the lagna weighed by its Bhava bala.
    final choice = longevity['choice']! as Map<String, Object?>;
    expect(choice['candidates'], hasLength(8));
    expect(choice['all_weighed'], isTrue);
    // v. 27: the dashas' span, a cycle of 120 years less what had run.
    final dasayus = longevity['dasayus']! as Map<String, Object?>;
    expect(dasayus['years']! as num, inInclusiveRange(100, 120));
    expect(
      () => found(const RuleRequest(rasmi: RasmiRules.verse)),
      throwsA(
        isA<TeistroException>().having((e) => e.field, 'field', 'rules.rasmi'),
      ),
    );

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
          'rules.rules[0]',
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
          'interpret.readings',
        ),
      ),
    );
    // The Sade Sati plan says what a window found, so it needs one beside
    // it; with one and no pack loaded it is present and empty, which is an
    // answer.
    expect(
      () => found(const PlanRequest(sadeSati: true)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'interpret.sadeSati',
        ),
      ),
    );
    final periods = ctx.chart.found(
      instant: 2451545.0,
      place: Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      ),
      utcOffsetSeconds: 20700,
      sadeSati: const SadeSatiRequest(from: 2451545, to: 2462502.5),
      interpret: const PlanRequest(sadeSati: true),
    );
    expect(periods.plans!['sadeSati'], isEmpty, reason: 'no pack, no words');
    expect(
      periods.sadeSati!.sadeSati,
      isNotEmpty,
      reason: 'the report is on the chart beside the plan',
    );
    ctx.dispose();
  });

  test('a layout of your own is registered, drawn by its key, and refused by '
      'its field', () {
    final base = context();
    final row = base.chart.layout(ChartLayout.southIndian);
    expect(row.key, 'SOUTH_INDIAN');
    expect(row.shape, isA<GridShape>());
    expect(row.shape.direction, LayoutDirection.clockwise);
    expect(
      (row.toJson()['shape']! as Map<String, Object?>)['direction'],
      'CLOCKWISE',
      reason: 'the words a row answers are keys',
    );
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

  /// A chart's Jaimini significators cross whole: the karakamsha with nine
  /// houses in each chart, the Atmakaraka's own navamsha the first from it,
  /// and the Brahma graha found or its absence named -- never both, never
  /// neither; null unless asked.
  test('a chart carries its hit list, the sky once for the batch', () {
    // The built-in ephemeris, whose Mercury turns retrograde in the window:
    // the test provider's planets never stand still.
    final ctx = teistro.context(
      profile: 'nepali-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    expect(
      ctx.chart
          .found(instant: 2451545, place: place, utcOffsetSeconds: 20700)
          .hits,
      isEmpty,
    );
    const asked = HitRequest(
      from: 2460676.5,
      to: 2460866.5,
      grahas: [Graha.sun, Graha.mercury, Graha.saturn],
      aspects: [0, 90, 180],
      orbDeg: 2,
    );
    final batch = ctx.chart.foundMany(
      instants: [2447995.4895833335, 2451545],
      place: place,
      utcOffsetSeconds: 20700,
      hits: asked,
    );
    String line(Hit h) => switch (h.event) {
      SignIngress(:final into, :final motion) =>
        '${h.instant} ${h.graha.key} ${into.key} ${motion.key}',
      NakshatraIngress(:final into, :final motion) =>
        '${h.instant} ${h.graha.key} ${into.key} ${motion.key}',
      Station(:final turns) => '${h.instant} ${h.graha.key} ${turns.key}',
      AspectHit(:final to, :final angle, :final phase) =>
        '${h.instant} ${h.graha.key} ${to.point} $angle ${phase.key}',
    };
    final first = batch.at(0).hits;
    final second = batch.at(1).hits;
    List<String> sky(List<Hit> hits) => [
      for (final h in hits)
        if (h.event is! AspectHit) line(h),
    ];
    expect(sky(first), sky(second));
    expect(first.map(line), isNot(second.map(line)));
    expect({for (final h in first) h.event.kind}, HitKind.values.toSet());
    for (final (k, h) in first.indexed) {
      if (k > 0) expect(first[k - 1].instant, lessThanOrEqualTo(h.instant));
      expect([Graha.sun, Graha.mercury, Graha.saturn], contains(h.graha));
      if (h.event case AspectHit(:final angle)) {
        expect([0, 90, 180], contains(angle));
      }
    }
    // What an aspect names as `to` is what a request takes back.
    final aspect = first.map((h) => h.event).whereType<AspectHit>().first;
    final again =
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              hits: HitRequest(
                from: asked.from,
                to: asked.to,
                grahas: asked.grahas,
                kinds: const [HitKind.aspect],
                points: [aspect.to],
                aspects: asked.aspects,
                orbDeg: asked.orbDeg,
              ),
            )
            .hits;
    expect(again, isNotEmpty);
    for (final h in again) {
      expect((h.event as AspectHit).to, aspect.to);
    }
    expect(
      () => ctx.chart.found(
        instant: 2451545,
        place: place,
        utcOffsetSeconds: 20700,
        hits: const HitRequest(from: 2460676.5, to: 2460600.5),
      ),
      throwsA(
        isA<TeistroException>().having((e) => e.field, 'field', 'hits.to'),
      ),
    );
    ctx.dispose();
  });

  test('a lunar return is the Moon back on her own natal place', () {
    final ctx = teistro.context(
      profile: 'nepali-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final paris = Observer(
      latitudeDeg: Latitude(48.8534),
      longitudeDeg: Longitude(2.3488),
      altitudeM: Altitude(0),
    );
    final asked = HitRequest.returns(from: 2451546, to: 2451911.25);
    expect(asked.grahas, [Graha.moon]);
    expect(asked.kinds, [HitKind.aspect]);
    expect(asked.points, [const NatalGraha(Graha.moon)]);
    expect(asked.aspects, [0]);
    Chart at(double instant, {HitRequest? hits}) => ctx.chart.found(
      instant: instant,
      place: paris,
      utcOffsetSeconds: 0,
      hits: hits,
    );
    final lunar = at(2451545, hits: asked).hits;
    expect(lunar, hasLength(13));
    for (final (k, hit) in lunar.indexed) {
      expect(hit.graha, Graha.moon);
      final event = hit.event as AspectHit;
      expect(event.to, const NatalGraha(Graha.moon));
      expect(event.angle, 0);
      if (k > 0) {
        expect(hit.instant - lunar[k - 1].instant, inInclusiveRange(27, 27.7));
      }
    }
    double moon(Chart chart) =>
        chart.grahas.firstWhere((g) => g.graha == Graha.moon).longitudeDeg;
    expect(moon(at(lunar.first.instant)), closeTo(moon(at(2451545)), 1 / 3600));
    ctx.dispose();
  });

  test('a chart carries its Sade Sati, each period whole', () {
    final ctx = teistro.context(
      profile: 'nepali-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const births = [2447995.4895833335, 2451545.2];
    SadeSatiReport? found(double instant, SadeSatiRequest? asked) =>
        ctx.chart
            .found(
              instant: instant,
              place: place,
              utcOffsetSeconds: 20700,
              sadeSati: asked,
            )
            .sadeSati;
    expect(found(births[0], null), isNull);
    for (final reckoning in Reckoning.values) {
      final asked = SadeSatiRequest(
        from: 2460676.5,
        to: 2464329,
        reckoning: reckoning,
        spells: const [4, 7, 8],
      );
      final batch = ctx.chart.foundMany(
        instants: births,
        place: place,
        utcOffsetSeconds: 20700,
        sadeSati: asked,
      );
      for (final (k, instant) in births.indexed) {
        final report = batch.at(k).sadeSati!;
        expect(report, found(instant, asked), reason: 'a batch is each alone');
        expect(report.reckoning, reckoning);
        expect(report.reference.from, GocharFrom.moon);
        expect(report.sadeSati.length + report.spells.length, greaterThan(0));
        for (final one in report.sadeSati) {
          expect([for (final spell in one.phases) spell.house], [12, 1, 2]);
          final visits = [for (final spell in one.phases) ...spell.visits]
            ..sort((a, b) => a.from!.compareTo(b.from!));
          for (var i = 1; i < visits.length; i += 1) {
            expect(visits[i - 1].to!, lessThanOrEqualTo(visits[i].from!));
          }
          // Asked at one instant inside its peak, from what the report
          // named: the same Sade Sati, whole.
          final peak = one.phases[1].visits.first;
          final now =
              found(
                instant,
                SadeSatiRequest(
                  from: (peak.from! + peak.to!) / 2,
                  countedFrom: report.reference.from,
                  reckoning: report.reckoning,
                ),
              )!;
          expect(now.sadeSati, [one], reason: '$reckoning');
        }
        for (final spell in report.spells) {
          expect([4, 7, 8], contains(spell.house));
        }
      }
    }
    for (final (bad, field) in [
      (const SadeSatiRequest(from: 2460676.5, to: 2460600.5), 'sadeSati.to'),
      (const SadeSatiRequest(from: 2460676.5, spells: [2]), 'sadeSati.spells'),
    ]) {
      expect(
        () => found(births[0], bad),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('an almanac carries the muhurta search it was asked for', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final from = gregorian(2026, 11, 25);
    Almanac days({
      CalendarDate? since,
      CalendarDate? to,
      MuhurtaRequest? muhurta,
    }) => ctx.almanac.of(
      from: since ?? from,
      to: to ?? gregorian(2026, 12, 3),
      place: place,
      utcOffsetSeconds: 20700,
      muhurta: muhurta,
    );
    final plain = days();
    expect(plain.muhurta, isNull);

    final almanac = days(
      muhurta: const MuhurtaRequest(
        rules: MuhurtaActivity.ramanMarriage,
        native: MuhurtaNative(
          star: Nakshatra.rohini,
          moonSign: Rashi.taurus,
          lagna: Rashi.leo,
        ),
        daysWithWindows: 9,
        most: 1000,
      ),
    );
    final answer = almanac.muhurta!;
    expect(answer.windows, isNotEmpty);
    expect(answer.ranking, MuhurtaRanking.texts);
    // The days are the ones asked without a search.
    for (var k = 0; k < plain.length; k += 1) {
      expect(
        almanac.at(k).provenance.contentHash,
        plain.at(k).provenance.contentHash,
      );
    }
    final kinds = [
      for (final window in answer.windows)
        for (final clause in window.clauses) clause.kind,
    ];
    expect(
      kinds.whereType<TarabalaClause>(),
      isNotEmpty,
      reason: 'the native is read',
    );
    expect(
      kinds.whereType<NakshatraClause>().every(
        (clause) => clause.nakshatra != Nakshatra.unknown,
      ),
      isTrue,
    );
    final knobs = {
      for (final convention in answer.provenance.appliedConventions)
        convention.knob,
    };
    expect(knobs, containsAll(['muhurta.asta', 'muhurta.zodiacAt']));

    // A clause answered is a bar a request may name, as it was read.
    final amrit = kinds.whereType<ChoghadiyaClause>().firstWhere(
      (clause) => clause.choghadiya == Choghadiya.amrit,
    );
    const graded = {
      'best': <Object?>[],
      'middling': <Object?>[],
      'rejected': <Object?>[],
      'otherwise': 'MIDDLING',
    };
    final rules = <String, Object?>{
      'day': {
        'tithis': graded,
        'nakshatras': graded,
        'yogas': graded,
        'karanas': graded,
        'varas': graded,
        'chandrabala': {'avoid': <Object?>[]},
      },
      'months': {'reckoning': 'ANY'},
      'lagnas': graded,
      'padas': <Object?>[],
      'heeds': <Object?>[],
      'bars': [amrit],
      'unjudged': <Object?>[],
      'baseline': null,
    };
    final barred =
        days(
          to: from,
          muhurta: MuhurtaRequest(
            rules: rules,
            daysWithWindows: 1,
            most: 100000,
          ),
        ).muhurta!;
    final struck = barred.windows.where((window) => window.barredBy.isNotEmpty);
    expect(
      struck,
      isNotEmpty,
      reason: 'the bar read back strikes the windows it names',
    );
    expect(struck.every((window) => window.barredBy.single == amrit), isTrue);

    // A closed day names its blackouts as members, and a request takes one
    // in either spelling: 1 June 2026 is in Jyeshtha's adhika month.
    final june = gregorian(2026, 6, 1);
    List<BlackoutKind> closedBy(Object rules) =>
        days(
          since: june,
          to: june,
          muhurta: MuhurtaRequest(rules: rules),
        ).muhurta!.closed.single.by;
    expect(
      closedBy(MuhurtaActivity.ramanMarriage),
      contains(BlackoutKind.adhikaMasa),
    );
    for (final heed in <Object>[BlackoutKind.adhikaMasa, 'ADHIKA_MASA']) {
      expect(
        closedBy({
          ...rules,
          'bars': <Object?>[],
          'heeds': [heed],
        }),
        [BlackoutKind.adhikaMasa],
      );
    }

    for (final (asked, field) in [
      (
        const MuhurtaRequest(rules: MuhurtaActivity.ramanMarriage, most: 0),
        'muhurta.most',
      ),
      (const MuhurtaRequest(rules: 'RAMAN_MARRIAGE'), 'muhurta.rules'),
      (
        const MuhurtaRequest(rules: <String, Object?>{'day': 1}),
        'muhurta.rules.day',
      ),
    ]) {
      expect(
        () => days(muhurta: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
        reason: field,
      );
    }

    // A rite beyond marriage, by name: its unwanted placements come back
    // typed, and the 8th the thread ceremony says must be empty bars as
    // itself.
    final thread =
        days(
          muhurta: const MuhurtaRequest(
            rules: MuhurtaActivity.ramanUpanayana,
            daysWithWindows: 9,
            most: 1000,
          ),
        ).muhurta!;
    final placed = [
      for (final window in thread.windows)
        for (final clause in window.clauses)
          if (clause.kind case final UnwantedPlacementClause placement)
            placement,
    ];
    expect(placed, isNotEmpty);
    expect(
      placed.every((p) => p.house >= 1 && p.house <= 12 && p.by.isNotEmpty),
      isTrue,
    );
    final barring = [
      for (final window in thread.windows)
        for (final bar in window.barredBy)
          if (bar case final UnwantedPlacementClause placement) placement,
    ];
    expect(barring.every((b) => b.house == 8), isTrue);
    expect(
      () => days(
        muhurta: MuhurtaRequest(
          rules: {
            ...rules,
            'unwanted': [
              {
                'grahas': ['MARS'],
                'houses': [13],
              },
            ],
          },
        ),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'muhurta.rules.unwanted[0].houses',
        ),
      ),
    );
    ctx.dispose();
  });

  test('an almanac carries the lunar years it was asked for', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    // Across Chaitra Shukla Pratipada of VS 2083, 19 March 2026.
    Almanac days({bool years = false}) => ctx.almanac.of(
      from: gregorian(2026, 3, 10),
      to: gregorian(2026, 4, 10),
      place: place,
      utcOffsetSeconds: 20700,
      years: years,
    );
    final plain = days();
    expect(plain.years, isNull);

    final almanac = days(years: true);
    final answer = almanac.years!;
    final years = answer.value;
    expect(
      [for (final year in years) year.samvatsara],
      [Samvatsara.siddharthi, Samvatsara.raudra],
    );
    expect([for (final year in years) year.vikrama], [2082, 2083]);
    expect(
      [for (final year in years) year.count],
      ['BARHASPATYA', 'BARHASPATYA'],
    );
    expect(years.first.ended, years.last.began);
    expect(years.last.opened, lessThan(years.last.began));
    expect(years.last.jovian.first.member, isNot(Samvatsara.unknown));
    expect(() => years.add(years.first), throwsUnsupportedError);
    expect(answer.provenance.contentHash, isNotEmpty);
    for (var k = 0; k < plain.length; k++) {
      expect(
        almanac.at(k).provenance.contentHash,
        plain.at(k).provenance.contentHash,
      );
    }
    ctx.dispose();
  });

  test('an almanac carries the eclipses it was asked for', () {
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    // September 2025 at Kathmandu: the total lunar eclipse of the 7th, seen
    // whole near midnight, and the partial solar eclipse of the 21st over
    // the South Pacific, which Nepal does not see.
    (Eclipses, Eclipses?) asked([Map<String, Object?>? settings]) {
      final ctx = teistro.context(
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
        settings: settings,
      );
      Almanac days({bool eclipses = false}) => ctx.almanac.of(
        from: gregorian(2025, 9, 1),
        to: gregorian(2025, 9, 30),
        place: place,
        utcOffsetSeconds: 20700,
        eclipses: eclipses,
      );
      final found = (days(eclipses: true).eclipses!, days().eclipses);
      ctx.dispose();
      return found;
    }

    final (answer, plain) = asked();
    expect(plain, isNull);
    expect(
      [for (final e in answer.value.lunar) e.eclipse.kind],
      [LunarEclipseKind.total],
    );
    expect(
      [for (final e in answer.value.solar) e.eclipse.kind],
      [SolarEclipseKind.partial],
    );
    final lunar = answer.value.lunar.single;
    expect(lunar.eclipse.shadow, EclipseShadowRule.danjon);
    expect(lunar.eclipse.contacts.u2!, lessThan(lunar.eclipse.greatest));
    expect(lunar.here.greatest.altitudeDeg, greaterThan(40));
    expect(
      lunar.here.seen,
      EclipseSeen(from: lunar.here.p1.at, to: lunar.here.p4.at),
    );
    expect(
      lunar.here.umbralSeen,
      EclipseSeen(from: lunar.here.u1!.at, to: lunar.here.u4!.at),
    );
    expect(() => answer.value.lunar.add(lunar), throwsUnsupportedError);
    final solar = answer.value.solar.single;
    expect(solar.here == null || solar.here!.seen == null, isTrue);
    expect(
      answer.provenance.appliedConventions.map((c) => c.knob),
      contains('eclipse.window'),
    );

    final (chauvenet, _) = asked({
      'panchanga': {'eclipse_shadow': 'CHAUVENET'},
    });
    final shadowed = chauvenet.value.lunar.single.eclipse;
    expect(shadowed.shadow, EclipseShadowRule.chauvenet);
    expect(
      shadowed.umbralMagnitude,
      greaterThan(lunar.eclipse.umbralMagnitude),
    );
    expect(
      chauvenet.provenance.settingsHash,
      isNot(answer.provenance.settingsHash),
    );
  });

  // Each day's Nepal Sambat date beside the days
  // (`03-design/calendar-indian-lunisolar.md` §11): one a day, the year
  // turning at Kachhala's first day, said as the committee's page header
  // prints it.
  test('an almanac carries each day\'s Nepal Sambat date', () {
    final ctx = teistro.context(
      profile: 'nepali-default',
      locale: 'ne-Deva-NP',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    Almanac days({bool nepalSambat = false}) => ctx.almanac.of(
      from: gregorian(2025, 10, 20),
      to: gregorian(2025, 10, 23),
      place: place,
      utcOffsetSeconds: 20700,
      nepalSambat: nepalSambat,
    );
    expect(days().nepalSambat, isNull);
    final almanac = days(nepalSambat: true);
    final answer = almanac.nepalSambat!;
    expect(answer.value, hasLength(almanac.length));
    expect(answer.value, const [
      NepalSambatDate(
        year: 1145,
        month: 12,
        kind: MonthKind.nija,
        paksha: Paksha.krishna,
      ),
      NepalSambatDate(
        year: 1145,
        month: 12,
        kind: MonthKind.nija,
        paksha: Paksha.krishna,
      ),
      NepalSambatDate(
        year: 1146,
        month: 1,
        kind: MonthKind.nija,
        paksha: Paksha.shukla,
      ),
      NepalSambatDate(
        year: 1146,
        month: 1,
        kind: MonthKind.nija,
        paksha: Paksha.shukla,
      ),
    ]);
    expect(() => answer.value.add(answer.value.first), throwsUnsupportedError);
    expect(answer.provenance.inputHash, almanac.provenance.inputHash);
    final first = answer.value[2];
    expect(
      ctx.intl.messages.sdk.calendar.nepalSambatDate(
        year: first.year,
        month: first.month,
        kind: first.kind.key,
        paksha: first.paksha.key,
      ),
      'ने.सं. ११४६ कछलाथ्व',
    );
    ctx.dispose();
  });

  // A limb's member naming two days or none, and its end in ghatis
  // (`03-design/nepal-day-measured.md`): Nepal's print, under the
  // committee's Surya Siddhanta.
  test('a span says which sunrises it held and when it ended in ghatis', () {
    final ctx = teistro.context(
      profile: 'nepali-committee',
      ephemeris: const [NamedEphemeris(Ephemeris.suryaSiddhanta)],
    );
    AlmanacDay day(int y, int m, int d) => ctx.almanac
        .of(
          from: gregorian(y, m, d),
          to: gregorian(y, m, d),
          place: Observer(
            latitudeDeg: Latitude(27.7172),
            longitudeDeg: Longitude(85.324),
            altitudeM: Altitude(1400),
          ),
          utcOffsetSeconds: 20700,
        )
        .at(0);
    // 13 April 2025: Krishna Pratipada day and night, a vriddhi.
    expect(
      [
        for (final span in day(2025, 4, 13).tithi)
          if (span.sunrises == Sunrises.both) span.member,
      ],
      [Tithi.krishnaPratipada],
    );
    // 26 April 2025: Krishna Chaturdashi between the sunrises, a kshaya.
    final kshaya = day(2025, 4, 26).tithi;
    expect(
      [for (final span in kshaya) span.sunrises],
      [Sunrises.opening, Sunrises.neither, Sunrises.next],
    );
    expect(kshaya[1].member, Tithi.krishnaChaturdashi);
    // 25 September 2026: Chaturdashi ended at 22:16, sunrise 05:54.
    final ends = day(2026, 9, 25).tithi[0].ends;
    expect(ends.ghati, inInclusiveRange(40, 41));
    expect(ends.pala, lessThan(60));
    expect(ends.vipala, lessThan(60));
    ctx.dispose();
  });

  // A day's season is its solar month's, and a month begins on the day
  // Nepal's calendar begins it (`03-design/ritu-measured.md`).
  test('a day\'s season turns on the first of its solar month', () {
    List<Ritu> seasons([Map<String, Object?>? settings]) {
      final ctx = teistro.context(
        profile: 'nepali-default',
        settings: settings,
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      );
      final days = ctx.almanac.of(
        from: gregorian(2026, 3, 13),
        to: gregorian(2026, 3, 16),
        place: Observer(
          latitudeDeg: Latitude(27.7172),
          longitudeDeg: Longitude(85.324),
          altitudeM: Altitude(1400),
        ),
        utcOffsetSeconds: 20700,
      );
      final named = [for (var k = 0; k < days.length; k++) days.at(k).ritu];
      ctx.dispose();
      return named;
    }

    // 15 March 2026 is 1 Chaitra 2082: Vasanta from that day.
    expect(seasons(), [
      Ritu.shishira,
      Ritu.shishira,
      Ritu.vasanta,
      Ritu.vasanta,
    ]);
    // Amanta Phalguna runs to the new moon of 19 March: Shishira.
    expect(
      seasons({
        'panchanga': {'ritu': 'LUNAR'},
      }),
      List.filled(4, Ritu.shishira),
    );
  });

  test('an almanac carries the festivals it was asked for', () {
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    Almanac days({FestivalRequest? festivals}) => ctx.almanac.of(
      from: gregorian(2026, 10, 15),
      to: gregorian(2026, 11, 10),
      place: place,
      utcOffsetSeconds: 20700,
      festivals: festivals,
    );
    final plain = days();
    expect(plain.festivals, isNull);

    final almanac = days(
      festivals: const FestivalRequest(rules: FestivalPack.dharmasindhu),
    );
    final answer = almanac.festivals!;
    expect(
      [for (final observance in answer.observances) observance.rule],
      ['VIJAYA_DASHAMI', 'LAKSHMI_PUJA', 'BALI_PRATIPADA'],
    );
    final dashami = answer.observances.first;
    expect(dashami.day.calendar, Calendar.gregorian);
    expect(dashami.day.month, 10);
    expect(dashami.extents.$1.day.calendar, Calendar.gregorian);
    expect(['GUARD', 'OTHERWISE'], contains(dashami.decidedBy.by));
    expect(answer.unjudged, isEmpty);
    // Two Ekadashis in the range, each under the pack's three observers.
    const observers = [
      'EKADASHI_VAISHNAVA',
      'EKADASHI_SMARTA',
      'EKADASHI_SMARTA_RENUNCIANT',
    ];
    expect(
      [for (final fast in answer.ekadashis) fast.rule],
      [...observers, ...observers],
    );
    final fast = answer.ekadashis.first;
    expect((fast.tithi, fast.month), (Tithi.shuklaEkadashi, Masa.ashwina));
    expect(fast.days.$1.calendar, Calendar.gregorian);
    expect(fast.day.calendar, Calendar.gregorian);
    expect(['EARLIER', 'LATER'], contains(fast.choice));
    for (var k = 0; k < plain.length; k += 1) {
      expect(
        almanac.at(k).provenance.contentHash,
        plain.at(k).provenance.contentHash,
      );
    }
    expect({
      for (final c in answer.provenance.appliedConventions) c.knob,
    }, contains('festival.days'));
    // Parsed twice, an observance is the same value.
    expect(
      days(
        festivals: const FestivalRequest(rules: [FestivalPack.dharmasindhu]),
      ).festivals!.observances,
      answer.observances,
    );

    // Lakshmi puja on whichever day holds its tithi at sunrise: a rule of
    // the consumer's own, its members as members, replacing the shipped one.
    final sunrise = <String, Object?>{
      'key': 'LAKSHMI_PUJA',
      'source': 'the tithi at sunrise',
      'month': Masa.ashwina,
      'tithi': Tithi.amavasya,
      'at': {'window': 'SUNRISE'},
      'decide': <Object?>[],
      'otherwise': 'LATER',
    };
    final moved =
        days(
          festivals: FestivalRequest(
            rules: [FestivalPack.dharmasindhu, sunrise],
          ),
        ).festivals!;
    expect(
      moved.observances[1].decidedBy,
      const FestivalDecided(by: 'OTHERWISE'),
    );
    expect(moved.provenance.inputHash, isNot(answer.provenance.inputHash));

    // Nepal's pack, and a day of the consumer's own counted two days from
    // Lakshmi puja's: it names the rule it counts from and the count.
    const following = <String, Object?>{
      'key': 'TWO_AFTER',
      'source': 'mine',
      'after': 'LAKSHMI_PUJA',
      'days': 2,
    };
    final counted =
        days(
          festivals: const FestivalRequest(
            rules: [FestivalPack.nepal, following],
          ),
        ).festivals!;
    final byRule = {for (final o in counted.observances) o.rule: o};
    final (two, lakshmi) = (byRule['TWO_AFTER']!, byRule['LAKSHMI_PUJA']!);
    expect(
      two.decidedBy,
      const FestivalDecided(by: 'AFTER', rule: 'LAKSHMI_PUJA', days: 2),
    );
    expect(two.day.day, lakshmi.day.day + 2);
    expect(
      (two.tithi.from, two.tithi.to),
      (lakshmi.tithi.from, lakshmi.tithi.to),
    );

    // Every observance says its month; Nepal's monthly full-moon fast is
    // judged at the instant of sunset, and a rule of one's own kept every
    // month leaves its month out.
    expect((dashami.month, dashami.adhika), (Masa.ashwina, false));
    final vrata = byRule['PURNIMA_VRATA']!;
    expect(vrata.month, Masa.ashwina);
    expect(vrata.extents.$1.window.from, vrata.extents.$1.window.to);
    const everyMonth = <String, Object?>{
      'key': 'EVERY_FULL_MOON',
      'source': 'mine',
      'tithi': 'tithi.PURNIMA',
      'inAdhika': true,
      'at': {'window': 'SUNSET'},
      'decide': <Object?>[],
      'otherwise': 'LATER',
    };
    final mine =
        days(festivals: const FestivalRequest(rules: [everyMonth])).festivals!;
    expect(
      [for (final o in mine.observances) (o.rule, o.month)],
      [('EVERY_FULL_MOON', Masa.ashwina)],
    );
    final hasta = <String, Object?>{
      for (final entry in everyMonth.entries)
        if (entry.key != 'tithi') entry.key: entry.value,
      'key': 'DARK_HASTA',
      'nakshatra': 'nakshatra.HASTA',
      'paksha': 'paksha.KRISHNA',
    };
    final kept = days(festivals: FestivalRequest(rules: [hasta])).festivals!;
    expect(
      [for (final o in kept.observances) (o.rule, o.month)],
      [('DARK_HASTA', Masa.ashwina)],
    );

    for (final (asked, field) in [
      (const FestivalRequest(rules: 'DHARMASINDHU'), 'festivals.rules'),
      (
        FestivalRequest(
          rules: [
            FestivalPack.dharmasindhu,
            {...sunrise, 'key': ''},
          ],
        ),
        'festivals.rules[1].key',
      ),
      (
        FestivalRequest(
          rules: [
            {
              ...sunrise,
              'at': {'window': 'DUSK'},
            },
          ],
        ),
        'festivals.rules[0].at.window',
      ),
      (
        FestivalRequest(
          rules: [
            {...sunrise, 'nakshatra': 'HASTA', 'paksha': 'SHUKLA'},
          ],
        ),
        'festivals.rules[0]',
      ),
      (
        FestivalRequest(
          rules: [
            {
              ...sunrise,
              'at': {'window': 'NIGHT_MUHURTA', 'muhurta': 16},
            },
          ],
        ),
        'festivals.rules[0].at.muhurta',
      ),
      (
        const FestivalRequest(
          rules: [
            FestivalPack.dharmasindhu,
            {
              'key': 'MINE',
              'source': '',
              'vedha': 'DUSK',
              'table': <String, Object?>{},
            },
          ],
        ),
        'festivals.rules[1].vedha',
      ),
      (
        const FestivalRequest(
          rules: [
            FestivalPack.dharmasindhu,
            {'key': 'MINE', 'source': '', 'after': 'HOLIKA', 'days': 16},
          ],
        ),
        'festivals.rules[1].days',
      ),
      (
        const FestivalRequest(
          rules: [
            FestivalPack.dharmasindhu,
            {'key': 'MINE', 'source': '', 'after': 'NOBODY', 'days': 1},
          ],
        ),
        'festivals.following[0].after',
      ),
      (const FestivalRequest(rules: [1]), 'festivals.rules[0]'),
    ]) {
      expect(
        () => days(festivals: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
        reason: field,
      );
    }
    ctx.dispose();
  });

  test('a chart carries its essential dignities', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(0),
    );
    const instants = [2460676.5, 2460676.75];
    Dignities? found(
      double instant,
      DignityRequest? asked, {
      Observer? place,
      int utcOffsetSeconds = 20700,
    }) =>
        ctx.chart
            .found(
              instant: instant,
              place: place ?? kathmandu,
              utcOffsetSeconds: utcOffsetSeconds,
              dignities: asked,
            )
            .dignities;
    expect(found(instants[0], null), isNull);

    final read = found(instants[0], const DignityRequest())!;
    expect(read.sectRule, SectRule.horizon);
    expect(
      read.rules,
      const AppliedDignityRules(
        terms: Terms.ptolemaicLilly,
        triplicities: Triplicities.lilly,
      ),
    );
    expect(read.scores, DignityScores.lilly);
    expect(read.planets.map((at) => at.planet), [
      Graha.saturn,
      Graha.jupiter,
      Graha.mars,
      Graha.sun,
      Graha.venus,
      Graha.mercury,
      Graha.moon,
    ]);
    const lilly = DignityScores.lilly;
    for (final at in read.planets) {
      final d = at.dignity;
      final held = [d.house, d.exaltation, d.triplicity, d.term, d.face];
      expect(at.peregrine, !held.contains(true), reason: '${at.planet}');
      final worth = [
        (d.house, lilly.house),
        (d.exaltation, lilly.exaltation),
        (d.triplicity, lilly.triplicity),
        (d.term, lilly.term),
        (d.face, lilly.face),
        (d.detriment, lilly.detriment),
        (d.fall, lilly.fall),
        (at.peregrine, lilly.peregrine),
      ];
      final score = worth
          .where((one) => one.$1)
          .fold<int>(0, (sum, one) => sum + one.$2);
      expect(at.score, score, reason: '${at.planet}');
    }

    // Each reception whole both ways, in the Chaldean order, and scored
    // only when mutual by house or by exaltation.
    final order = [for (final at in read.planets) at.planet];
    expect(read.receptions, isNotEmpty);
    for (final one in read.receptions) {
      final (first, second) = one.planets;
      expect(order.indexOf(first), lessThan(order.indexOf(second)));
      for (final side in [one.firstIn, one.secondIn]) {
        expect(DignityKind.values.any(side.holds), isTrue, reason: '$one');
      }
      expect(one.mutual, [
        for (final kind in DignityKind.values)
          if (one.firstIn.holds(kind) && one.secondIn.holds(kind)) kind,
      ]);
    }
    for (final at in read.planets) {
      bool by(DignityKind kind) => read.receptions.any(
        (one) =>
            (one.planets.$1 == at.planet || one.planets.$2 == at.planet) &&
            one.mutual.contains(kind),
      );
      expect(
        at.reception,
        (by(DignityKind.house) ? lilly.house : 0) +
            (by(DignityKind.exaltation) ? lilly.exaltation : 0),
        reason: '${at.planet}',
      );
    }

    // Every rule reported as asked, and a score left out stays Lilly's.
    final night =
        found(
          instants[0],
          const DignityRequest(
            sectRule: SectRule.night,
            triplicities: Triplicities.ptolemy,
            scores: DignityScores(peregrine: 0),
          ),
        )!;
    expect(night.sect, Sect.night);
    expect(night.rules.triplicities, Triplicities.ptolemy);
    expect(night.scores, const DignityScores(peregrine: 0));

    // 21 December 1988 at Tromsø: the Sun culminates under the horizon.
    final tromso = Observer(
      latitudeDeg: Latitude(69.6492),
      longitudeDeg: Longitude(18.9553),
      altitudeM: Altitude(0),
    );
    expect(
      found(
        2447516.9583333335,
        const DignityRequest(),
        place: tromso,
        utcOffsetSeconds: 3600,
      )!.sect,
      Sect.night,
    );

    // A table of the caller's own: Aries' Egyptian terms in every sign.
    const row = [
      Term(Graha.jupiter, 6),
      Term(Graha.venus, 12),
      Term(Graha.mercury, 20),
      Term(Graha.mars, 25),
      Term(Graha.saturn, 30),
    ];
    final own =
        found(instants[0], DignityRequest(table: List.filled(12, row)))!;
    expect(own.rules.terms, Terms.table);
    for (final at in own.planets) {
      final degree = at.longitudeDeg % 30;
      final lord = row.firstWhere((term) => degree < term.end).lord;
      expect(at.dignity.term, lord == at.planet, reason: '${at.planet}');
    }

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      dignities: const DignityRequest(),
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).dignities,
        found(instant, const DignityRequest()),
        reason: 'each alone',
      );
    }

    for (final (bad, field) in [
      (
        DignityRequest(table: List.filled(11, row)),
        'dignities.rules.terms.TABLE',
      ),
      // `TABLE` names a table in an answer and carries none in a request.
      (const DignityRequest(terms: Terms.table), 'dignities.rules.terms.TABLE'),
    ]) {
      expect(
        () => found(instants[0], bad),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its lots', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(0),
    );
    const instants = [2460676.5, 2460676.75];
    Chart found(
      double instant, {
      LotRequest? lots,
      FortitudeRequest? fortitudes,
    }) => ctx.chart.found(
      instant: instant,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      lots: lots,
      fortitudes: fortitudes,
    );
    double apart(double one, double other) =>
        math.min((one - other) % 360, (other - one) % 360);
    expect(found(instants[0]).lots, isNull);

    for (final instant in instants) {
      final chart = found(
        instant,
        lots: LotRequest.valens,
        fortitudes: const FortitudeRequest(),
      );
      final read = chart.lots!;
      final fortitudes = chart.fortitudes!;
      expect(read.request, LotRequest.valens);
      expect(read.fortuneReversed, read.sect == Sect.night);
      expect([for (final placed in read.lots) placed.lot], Lot.values);
      final at = {
        for (final placed in read.lots) placed.lot: placed.place.longitudeDeg,
      };
      final longitude = {
        for (final own in fortitudes.dignities.planets)
          own.planet: own.longitudeDeg,
      };
      final ascendant = fortitudes.sky.ascendantDeg;
      final (start, end) =
          read.fortuneReversed
              ? (Graha.moon, Graha.sun)
              : (Graha.sun, Graha.moon);
      final fortune = (ascendant + longitude[end]! - longitude[start]!) % 360;
      expect(apart(at[Lot.fortune]!, fortune), lessThan(1e-9));
      expect(
        apart(at[Lot.daimon]!, 2 * ascendant - fortune),
        lessThan(1e-9),
        reason: 'Daimon mirrors Fortune',
      );
      for (final placed in read.lots) {
        expect(placed.place.longitudeDeg, inInclusiveRange(0, 360));
        expect(placed.place.house, inInclusiveRange(1, 12));
      }
      // The answer's rules are a request as they stand.
      expect(found(instant, lots: read.request).lots, read);
    }

    const asked = LotRequest(
      sectRule: SectRule.daylight,
      fortune: FortuneRule.reversedWhileMoonUp,
    );
    expect(found(instants[0], lots: asked).lots!.request, asked);

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      lots: LotRequest.valens,
    );
    for (final (k, instant) in instants.indexed) {
      expect(batch.at(k).lots, found(instant, lots: LotRequest.valens).lots);
    }
    ctx.dispose();
  });

  test('a chart carries its considerations', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(-0.12),
      altitudeM: Altitude(0),
    );
    final instants = [for (var k = 0; k < 16; k += 1) 2451545 + k / 8];
    Chart found(
      double instant, {
      ConsiderationRules? considerations,
      FortitudeRequest? fortitudes,
    }) => ctx.chart.found(
      instant: instant,
      place: london,
      utcOffsetSeconds: 0,
      considerations: considerations,
      fortitudes: fortitudes,
    );
    expect(found(instants[0]).considerations, isNull);

    var voids = 0;
    for (final instant in instants) {
      final chart = found(
        instant,
        considerations: ConsiderationRules.lilly,
        fortitudes: const FortitudeRequest(),
      );
      final read = chart.considerations!;
      final sky = chart.fortitudes!.sky;
      expect(read.rules, ConsiderationRules.lilly);
      expect(read.ascendant.sign.id, sky.ascendantDeg ~/ 30);
      expect(read.seventh.cuspDeg, sky.cuspsDeg[6]);
      expect(read.ascendant.early, read.ascendant.degree < 3);
      final course = read.moon.course;
      if (course.next == null) expect(course.withinOrb, isNull);
      if (course.next == null || course.withinOrb == null) voids += 1;
      expect(
        read.radicality.grounds.contains(RadicalGround.oneLord),
        read.radicality.hourLord == read.radicality.ascendantLord,
      );
      // The answer's rules are a request as they stand.
      expect(found(instant, considerations: read.rules).considerations, read);
    }
    expect(voids, greaterThan(0), reason: 'a void Moon in the batch');

    const asked = ConsiderationRules(
      moonLateFromDeg: 25,
      orbsDeg: [9, 9, 7, 15, 7, 7, 12],
    );
    expect(
      found(instants[0], considerations: asked).considerations!.rules,
      asked,
    );

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: london,
      utcOffsetSeconds: 0,
      considerations: ConsiderationRules.lilly,
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).considerations,
        found(instant, considerations: ConsiderationRules.lilly).considerations,
      );
    }
    for (final (rules, field) in [
      (
        const ConsiderationRules(moonLateFromDeg: 31),
        'considerations.moonLateFromDeg',
      ),
      (
        const ConsiderationRules(orbsDeg: [10, 12, 7.5, 17, 8, 7, -1]),
        'considerations.orbsDeg',
      ),
    ]) {
      expect(
        () => found(instants[0], considerations: rules),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its perfection', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(-0.12),
      altitudeM: Altitude(0),
    );
    final instants = [for (var k = 0; k < 12; k += 1) 2451545 + 23 * k + k / 7];
    const seventh = PerfectionRequest.ofHouse(7);
    Chart found(
      double instant, {
      PerfectionRequest? perfection,
      FortitudeRequest? fortitudes,
    }) => ctx.chart.found(
      instant: instant,
      place: london,
      utcOffsetSeconds: 0,
      perfection: perfection,
      fortitudes: fortitudes,
    );
    expect(found(instants[0]).perfection, isNull);

    var applying = 0;
    var hindered = 0;
    for (final instant in instants) {
      final chart = found(
        instant,
        perfection: seventh,
        fortitudes: const FortitudeRequest(),
      );
      final read = chart.perfection!;
      expect(read.rules, PerfectionRules.lilly);
      expect(read.querent, isNot(read.quesited));
      final houses = {
        for (final at in chart.fortitudes!.planets) at.planet: at.house,
      };
      expect(read.ways.querent.house, houses[read.querent]);
      expect(read.ways.quesited.house, houses[read.quesited]);
      if (read.application case final application?) {
        applying += 1;
        expect(application.days, inInclusiveRange(0, read.horizonDays));
        expect([read.querent, read.quesited], contains(application.applying));
      }
      for (final way in [
        Way.conjunction,
        Way.sextileOrTrine,
        Way.square,
        Way.opposition,
      ]) {
        if (read.ways.held.contains(way)) {
          expect(read.application, isNotNull, reason: '$way');
        }
      }
      for (final impediment in read.impediments) {
        hindered += 1;
        expect(
          impediment.third == null,
          impediment.kind == ImpedimentKind.refranation,
        );
      }
      for (final translation in read.translations) {
        expect(
          {translation.from, translation.to},
          {read.querent, read.quesited},
        );
      }
      // The answer's rules are a request as they stand.
      expect(
        found(
          instant,
          perfection: PerfectionRequest.ofHouse(7, rules: read.rules),
        ).perfection,
        read,
      );
    }
    expect(applying, greaterThan(0), reason: 'the sweep applies');
    expect(hindered, greaterThan(0), reason: 'and is hindered');

    final named =
        found(
          instants[0],
          perfection: const PerfectionRequest.between(
            Graha.venus,
            Graha.mars,
            rules: PerfectionRules(horizonDays: 30),
          ),
        ).perfection!;
    expect(
      (
        named.querent,
        named.quesited,
        named.rules.horizonDays,
        named.horizonDays,
      ),
      (Graha.venus, Graha.mars, 30.0, 30.0),
    );
    final every =
        found(
          instants[0],
          perfection: const PerfectionRequest.between(
            Graha.venus,
            Graha.mars,
            rules: PerfectionRules(withinSign: false),
          ),
        ).perfection!;
    expect(every.rules.withinSign, isFalse);

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: london,
      utcOffsetSeconds: 0,
      perfection: seventh,
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).perfection,
        found(instant, perfection: seventh).perfection,
      );
    }
    for (final (asked, field) in [
      (const PerfectionRequest(), 'perfection.quesited'),
      (
        const PerfectionRequest(house: 7, quesited: Graha.mars),
        'perfection.house',
      ),
      (
        const PerfectionRequest.ofHouse(
          7,
          rules: PerfectionRules(horizonDays: -1),
        ),
        'perfection.rules.horizonDays',
      ),
    ]) {
      expect(
        () => found(instants[0], perfection: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its progressions', () {
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(0),
      altitudeM: Altitude(0),
    );
    const birth = 2400629.742361111;
    Chart found(double instant, {ProgressionsRequest? progressions}) =>
        ctx.chart.found(
          instant: instant,
          place: london,
          utcOffsetSeconds: 0,
          progressions: progressions,
        );
    expect(found(birth).progressions, isNull);

    // His forty-seventh year: the map at sidereal time 5h 54m 16s (p. 35).
    const at = birth + 46 * 365.242189;
    final read =
        found(
          birth,
          progressions: const ProgressionsRequest(at: at),
        ).progressions!;
    final progressed = read.progressed!;
    final directed = read.directed!;
    expect(progressed.sky, closeTo(birth + 46, 1e-9));
    expect(progressed.armcDeg / 15, closeTo(5 + 54 / 60 + 16 / 3600, 2 / 3600));
    expect(progressed.grahas.length, directed.planets.length);
    expect(read.contacts, isNull);
    final sun = progressed.grahas.firstWhere((g) => g.graha == Graha.sun);
    final directedSun = directed.planets.firstWhere(
      (g) => g.graha == Graha.sun,
    );
    expect(sun.longitudeDeg, closeTo(directedSun.longitudeDeg, 1e-9));

    // The Moon sesquiquadrate Mercury (p. 305): the 21st by a year, the
    // 22nd by his rule.
    const october = ProgressionContacts(
      from: 2417484.5,
      to: 2417515.5,
      grahas: [Graha.moon],
      points: [NatalGraha(Graha.mercury)],
      aspects: [135],
    );
    for (final (year, day) in [
      (YearMeasure.tropical, 21),
      (YearMeasure.noonSiderealTime, 22),
    ]) {
      final contacts =
          found(
            birth,
            progressions: ProgressionsRequest(year: year, contacts: october),
          ).progressions!;
      expect(contacts.progressed, isNull);
      final [contact] = contacts.contacts!;
      expect(
        (contact.graha, contact.to, contact.angle, contact.motion),
        (Graha.moon, const NatalGraha(Graha.mercury), 135, Motion.direct),
      );
      expect((contact.life - 2417484.5).floor() + 1, day, reason: '$year');
    }
    final none =
        found(
          birth,
          progressions: const ProgressionsRequest(
            contacts: ProgressionContacts(
              from: 2417484.5,
              to: 2417515.5,
              grahas: [Graha.moon],
              aspects: [90],
            ),
          ),
        ).progressions!;
    expect(none.contacts, isEmpty, reason: 'a window asked holding none');

    final instants = [birth, birth + 3000.25, birth + 9000.5];
    const many = ProgressionsRequest(
      at: 2430000.5,
      angles: AngleMethod.solarArcLongitude,
      direction: DirectionArc.naibod,
    );
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: london,
      utcOffsetSeconds: 0,
      progressions: many,
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).progressions,
        found(instant, progressions: many).progressions,
      );
    }
    for (final (asked, field) in [
      (const ProgressionsRequest(), 'progressions.at'),
      (
        const ProgressionsRequest(at: at, direction: DirectionArc.perYear(0)),
        'progressions.direction',
      ),
      (
        const ProgressionsRequest(
          contacts: ProgressionContacts(from: 2, to: 1),
        ),
        'progressions.contacts.to',
      ),
    ]) {
      expect(
        () => found(birth, progressions: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its Western aspects', () {
    // King Edward VII's nativity: Leo's four (*How to Judge a Nativity*,
    // pp. 295–296) under his orbs, Lilly's moieties refusing the outer
    // three, a batch the charts one at a time, and refusals named in the
    // record (`03-design/western-aspects.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final palace = Observer(
      latitudeDeg: Latitude(51.501),
      longitudeDeg: Longitude(-0.142),
      altitudeM: Altitude(0),
    );
    const birth = 2393783.95;
    Chart found(
      double instant, {
      WesternAspectRequest? asked,
      bool outerPlanets = false,
    }) => ctx.chart.found(
      instant: instant,
      place: palace,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      westernAspects: asked,
    );
    expect(found(birth).westernAspects, isNull);

    final rows =
        found(
          birth,
          asked: const WesternAspectRequest(),
          outerPlanets: true,
        ).westernAspects!;
    final held = {
      for (final row in rows) ({row.first, row.second}, row.aspect),
    };
    bool holds(Graha a, WesternAspect aspect, Graha b) =>
        held.any((h) => h.$2 == aspect && h.$1.containsAll({a, b}));
    for (final (a, aspect, b) in [
      (Graha.sun, WesternAspect.trine, Graha.uranus),
      (Graha.sun, WesternAspect.sextile, Graha.mars),
      (Graha.sun, WesternAspect.square, Graha.neptune),
      (Graha.moon, WesternAspect.square, Graha.saturn),
    ]) {
      expect(holds(a, aspect, b), isTrue, reason: '$a $aspect $b');
    }
    expect(rows.every((row) => row.fromExactDeg <= row.orbDeg), isTrue);

    // Lilly's moieties over the seven: the Moon (12½) and Saturn (10)
    // square within 11¼.
    final square = found(
      birth,
      asked: WesternAspectRequest.lilly,
    ).westernAspects!.firstWhere(
      (row) => row.first == Graha.moon && row.second == Graha.saturn,
    );
    expect((square.aspect, square.orbDeg), (WesternAspect.square, 11.25));

    final instants = [birth, birth + 3000.25, birth + 9000.5];
    const two = WesternAspectRequest(
      aspects: [WesternAspect.trine, WesternAspect.square],
    );
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: palace,
      utcOffsetSeconds: 0,
      westernAspects: two,
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).westernAspects,
        found(instant, asked: two).westernAspects,
      );
    }
    for (final (asked, field, outer) in [
      (
        const WesternAspectRequest(aspects: []),
        'westernAspects.aspects',
        false,
      ),
      (
        const WesternAspectRequest(
          aspects: [WesternAspect.trine, WesternAspect.trine],
        ),
        'westernAspects.aspects',
        false,
      ),
      (
        const WesternAspectRequest(
          aspects: [WesternAspect.trine],
          orbs: OrbModel.byAspect({WesternAspect.trine: 91}),
        ),
        'westernAspects.orbs.orbs',
        false,
      ),
      (WesternAspectRequest.lilly, 'westernAspects.orbs.orbs', true),
    ]) {
      expect(
        () => found(birth, asked: asked, outerPlanets: outer),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its declinations and parallels', () {
    // King George V (Leo, *How to Judge a Nativity*, p. 130): the recast's
    // declinations, his four parallels, a batch the charts one at a time,
    // and refusals named in the record
    // (`03-design/western-declinations.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final george = Observer(
      latitudeDeg: Latitude(51.5045),
      longitudeDeg: Longitude(-0.1366),
      altitudeM: Altitude(0),
    );
    const birth = 2402390.554166667;
    Chart found(
      double instant, {
      ParallelRequest? asked,
      bool outerPlanets = false,
    }) => ctx.chart.found(
      instant: instant,
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      parallels: asked,
    );
    expect(found(birth).declinations, isNull);
    expect(found(birth).parallels, isNull);

    final chart = found(
      birth,
      asked: const ParallelRequest(),
      outerPlanets: true,
    );
    final read = chart.declinations!;
    expect(read.grahas, hasLength(10));
    expect(read.graha(Graha.sun), closeTo(22.2997, 0.01));
    expect(read.lagnaDeg, closeTo(0.8366, 0.01));
    expect(read.midheavenDeg, closeTo(-23.452, 0.01));
    expect(
      [
        for (final row in chart.parallels!)
          (row.first, row.second, row.contrary),
      ],
      [
        (Graha.moon, Graha.neptune, true),
        (Graha.sun, Graha.jupiter, true),
        (Graha.jupiter, Graha.uranus, true),
        (Graha.mercury, Graha.venus, false),
      ],
    );

    final instants = [birth, birth - 3000.25];
    const wider = ParallelRequest(orbDeg: 1.5);
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: george,
      utcOffsetSeconds: 0,
      parallels: wider,
    );
    for (final (k, instant) in instants.indexed) {
      final one = found(instant, asked: wider);
      expect(batch.at(k).parallels, one.parallels);
      expect(batch.at(k).declinations, one.declinations);
    }
    for (final orb in [0.0, 11.0]) {
      expect(
        () => found(birth, asked: ParallelRequest(orbDeg: orb)),
        throwsA(
          isA<TeistroException>().having(
            (e) => e.field,
            'field',
            'parallels.orbDeg',
          ),
        ),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its equal distances', () {
    // King George V (Leo, *How to Judge a Nativity*, p. 130): his recast's
    // one under the default, Pluto on the far point of the Moon and
    // Jupiter, eight at 1.5°, and refusals named in the record
    // (`03-design/western-midpoints.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final george = Observer(
      latitudeDeg: Latitude(51.5045),
      longitudeDeg: Longitude(-0.1366),
      altitudeM: Altitude(0),
    );
    const birth = 2402390.554166667;
    Chart found(double instant, {MidpointRequest? asked}) => ctx.chart.found(
      instant: instant,
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      midpoints: asked,
    );
    expect(found(birth).midpoints, isNull);

    final rows = found(birth, asked: const MidpointRequest()).midpoints!;
    expect(rows, hasLength(1));
    final row = rows.single;
    expect(
      (row.first, row.second, row.middle, row.far),
      (Graha.moon, Graha.jupiter, Graha.pluto, true),
    );
    expect(row.fromAxisDeg, closeTo(0.052, 0.01));
    expect(row.distanceDeg, closeTo(137.69, 0.01));
    expect(row.orbDeg, 0.5);

    const wide = MidpointRequest(orbDeg: 1.5);
    final eight = found(birth, asked: wide).midpoints!;
    expect(eight, hasLength(8));
    for (final (n, at) in eight.indexed.skip(1)) {
      expect(eight[n - 1].fromAxisDeg, lessThanOrEqualTo(at.fromAxisDeg));
    }
    final batch = ctx.chart.foundMany(
      instants: [birth, birth - 3000.25],
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      midpoints: wide,
    );
    expect(batch.at(0).midpoints, found(birth, asked: wide).midpoints);
    expect(
      () => found(birth, asked: const MidpointRequest(orbDeg: 11)),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'midpoints.orbDeg',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its antiscia', () {
    // King George V (Leo, *How to Judge a Nativity*, p. 130): his recast's
    // one pair under Lilly's moieties, the outer three unpaired, Leo's orbs
    // pairing them, and refusals named in the record
    // (`03-design/western-antiscia.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final george = Observer(
      latitudeDeg: Latitude(51.5045),
      longitudeDeg: Longitude(-0.1366),
      altitudeM: Altitude(0),
    );
    const birth = 2402390.554166667;
    Chart found(
      double instant, {
      AntisciaRequest? asked,
      bool outerPlanets = false,
    }) => ctx.chart.found(
      instant: instant,
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      antiscia: asked,
    );
    expect(found(birth).antiscia, isNull);

    final read =
        found(
          birth,
          asked: const AntisciaRequest(),
          outerPlanets: true,
        ).antiscia!;
    final sun = read.points.firstWhere((at) => at.graha == Graha.sun);
    expect(sun.antiscionDeg, closeTo(107.5685, 0.01));
    expect(read.pairs, hasLength(1));
    final pair = read.pairs.single;
    expect(
      (pair.first, pair.second, pair.contrary),
      (Graha.mars, Graha.mercury, false),
    );
    expect(pair.apartDeg, closeTo(5.935, 0.02));
    expect(read.unpaired, [Graha.uranus, Graha.neptune, Graha.pluto]);
    expect(read.onCusps, isEmpty, reason: 'no cusps unless asked');
    expect(read.cuspSystem, isNull);

    // On the cusps, Lilly's Regiomontanus unless named: his Uranus
    // reflects 0.63° past the fourth cusp, into the next degree.
    final on =
        found(
          birth,
          asked: const AntisciaRequest(cusps: WesternHouseRequest()),
          outerPlanets: true,
        ).antiscia!;
    expect(on.onCusps, isEmpty);
    expect(on.cuspSystem, HouseSystem.regiomontanus);
    final named =
        found(
          birth,
          asked: const AntisciaRequest(
            cusps: WesternHouseRequest(system: HouseSystem.placidus),
          ),
        ).antiscia!;
    expect(named.cuspSystem, HouseSystem.placidus);

    const leo = AntisciaRequest(orbs: OrbModel.leo);
    expect(
      found(birth, asked: leo, outerPlanets: true).antiscia!.unpaired,
      isEmpty,
    );
    final instants = [birth, birth - 3000.25];
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: george,
      utcOffsetSeconds: 0,
      antiscia: leo,
    );
    for (final (k, instant) in instants.indexed) {
      expect(batch.at(k).antiscia, found(instant, asked: leo).antiscia);
    }
    expect(
      () => found(
        birth,
        asked: const AntisciaRequest(
          orbs: OrbModel.byAspect({WesternAspect.trine: 3}),
        ),
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'antiscia.orbs.orbs',
        ),
      ),
    );
    ctx.dispose();
  });

  test('a chart carries its harmonic', () {
    // Churchill's 9th harmonic as Addey reads it (*Harmonics in
    // Astrology*, pp. 97–98): the Moon on Saturn in the third, Venus
    // rising, Pluto in the tenth (`03-design/western-harmonics.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final blenheim = Observer(
      latitudeDeg: Latitude(51.8414),
      longitudeDeg: Longitude(-1.3611),
      altitudeM: Altitude(0),
    );
    const birth = 2405857.564892;
    Chart found(double instant, {HarmonicRequest? asked}) => ctx.chart.found(
      instant: instant,
      place: blenheim,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      harmonic: asked,
    );
    expect(found(birth).harmonic, isNull);

    final ninth = found(birth, asked: const HarmonicRequest(9)).harmonic!;
    expect((ninth.harmonic, ninth.points.length), (9, 12));
    int house(HarmonicPoint point) =>
        ninth.points.firstWhere((one) => one.point == point).house;
    expect(
      [
        for (final graha in [
          Graha.moon,
          Graha.saturn,
          Graha.venus,
          Graha.pluto,
        ])
          house(HarmonicGraha(graha)),
      ],
      [3, 3, 1, 10],
    );
    expect(house(HarmonicPoint.ascendant), 1);
    expect(ninth.points.last.point, HarmonicPoint.midheaven);
    final row = ninth.rows.firstWhere(
      (one) =>
          one.first == const HarmonicGraha(Graha.moon) &&
          one.second == const HarmonicGraha(Graha.saturn),
    );
    expect(row.apartDeg, lessThan(0.6));
    expect((row.multiple, row.orbDeg), (4, 12.0));

    const fifth = HarmonicRequest(5, orbDeg: 3);
    final instants = [birth, birth + 100.5];
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: blenheim,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      harmonic: fifth,
    );
    for (final (k, instant) in instants.indexed) {
      expect(batch.at(k).harmonic, found(instant, asked: fifth).harmonic);
    }
    for (final (asked, field) in [
      (const HarmonicRequest(0), 'harmonic.number'),
      (const HarmonicRequest(9, orbDeg: 31), 'harmonic.orbDeg'),
    ]) {
      expect(
        () => found(birth, asked: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its match with a partner', () {
    // A birth matched with itself: one sign and one nakshatra, so every
    // koota but Nadi takes its whole points and the shared nadi none, 28,
    // whatever the Moon (*Muhurta Chintamani* VI.21–34,
    // `03-design/matching.md`).
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    const birth = 2451545.0;
    Chart found(double instant, {MatchingRequest? asked}) => ctx.chart.found(
      instant: instant,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      matching: asked,
    );
    expect(found(birth).matching, isNull);

    final itself =
        found(
          birth,
          asked: MatchingRequest(
            Partner(instant: birth, place: kathmandu, utcOffsetSeconds: 20700),
            partnerRole: MatchRole.bride,
          ),
        ).matching!;
    expect(itself.total, 28);
    expect(
      [for (final row in itself.kootas) row.reading.koota],
      [
        Koota.varna,
        Koota.vashya,
        Koota.tara,
        Koota.yoni,
        Koota.grahaMaitri,
        Koota.gana,
        Koota.bhakoot,
        Koota.nadi,
      ],
    );
    expect(
      [for (final row in itself.kootas) row.maxPoints],
      [1, 2, 3, 4, 5, 6, 7, 8],
    );
    expect(itself.kootas[1].reading, const VashyaKoota(VashyaRelation.mutual));
    expect(
      itself.kootas[2].reading,
      const TaraKoota(brideToGroom: 1, groomToBride: 1),
    );
    final bhakoot = itself.kootas[6].reading as BhakootKoota;
    expect(
      (
        bhakoot.apart,
        bhakoot.dosha,
        bhakoot.lifted,
        bhakoot.exceptions.oneLord,
      ),
      (1, null, false, true),
    );
    final nadi = itself.kootas[7].reading as NadiKoota;
    expect((nadi.dosha, nadi.bride == nadi.groom), (true, true));
    // One star in one pada is the nadi dosha VI.36 does not lift; one gana
    // and one lord leave nothing to lift.
    final gana = itself.kootas[5].reading as GanaKoota;
    final maitri = itself.kootas[4].reading as MaitriKoota;
    expect(
      (nadi.lifted, gana.dosha, gana.lifted, maitri.lifted),
      (false, false, false, false),
    );

    // The ten considerations ride on the same request: one star in one
    // sign shares its Rajju, which the one lord lifts.
    final ten =
        found(
          birth,
          asked: MatchingRequest(
            Partner(instant: birth, place: kathmandu, utcOffsetSeconds: 20700),
            partnerRole: MatchRole.bride,
          ),
        ).porutham!;
    expect(
      [for (final row in ten.considerations) row.reading.koota],
      [
        Koota.tara,
        Koota.gana,
        Koota.mahendra,
        Koota.streeDeergha,
        Koota.yoni,
        Koota.bhakoot,
        Koota.grahaMaitri,
        Koota.vashya,
        Koota.rajju,
        Koota.vedha,
      ],
    );
    final dhinam = ten.considerations.first.reading as DhinamPorutham;
    expect(dhinam.count, 1);
    expect(dhinam.rule.key, startsWith('COMMON_'));
    final rajju = ten.considerations[8];
    final divisions = rajju.reading as RajjuPorutham;
    expect(
      (divisions.bride == divisions.groom, rajju.agrees, rajju.lifted),
      (true, true, true),
    );
    expect((ten.exception.oneLord, ten.exception.opposite), (true, false));
    expect(ten.agreeing, ten.considerations.where((row) => row.agrees).length);
    expect(found(birth).porutham, isNull);

    // The Kuja dosha rides on it too (Manasagari): one birth on both sides
    // reads Mars alike, so both carry it or neither does.
    final mars =
        found(
          birth,
          asked: MatchingRequest(
            Partner(instant: birth, place: kathmandu, utcOffsetSeconds: 20700),
            partnerRole: MatchRole.bride,
            kuja: const KujaRules(from: KujaFrom.lagnaMoonVenus),
          ),
        ).kuja!;
    expect(mars.bride, mars.groom);
    expect([for (final r in mars.bride.readings) r.from], KujaReference.values);
    for (final reading in mars.bride.readings) {
      expect(reading.inHouses, [1, 4, 7, 8, 12].contains(reading.house));
    }
    expect(mars.bride.dosha, mars.bride.readings.any((r) => r.inHouses));
    expect(mars.both, mars.bride.dosha);
    expect(found(birth).kuja, isNull);

    // The marriage doshas gather the three (C289): the shared nadi in one
    // pada first, then each consideration that disagrees or was lifted,
    // then the Kuja sides.
    final self = found(
      birth,
      asked: MatchingRequest(
        Partner(instant: birth, place: kathmandu, utcOffsetSeconds: 20700),
        partnerRole: MatchRole.bride,
      ),
    );
    final doshas = self.marriageDoshas!;
    expect(
      doshas.first,
      const MarriageDosha(
        system: DoshaSystem.ashtaKoota,
        koota: Koota.nadi,
        side: null,
        lifted: false,
      ),
    );
    expect(
      [
        for (final d in doshas)
          if (d.system == DoshaSystem.porutham) (d.koota, d.lifted),
      ],
      [
        for (final row in self.porutham!.considerations)
          if (!row.agrees || row.lifted) (row.reading.koota, row.lifted),
      ],
    );
    expect(
      [
        for (final d in doshas)
          if (d.system == DoshaSystem.kuja) d.side,
      ],
      self.kuja!.bride.dosha
          ? [MatchRole.bride, MatchRole.groom]
          : <MatchRole>[],
    );
    expect(found(birth).marriageDoshas, isNull);

    final asked = MatchingRequest(
      Partner(instant: 2447892.5, place: kathmandu, utcOffsetSeconds: 20700),
      partnerRole: MatchRole.groom,
      rules: const KootaRules(nadiDosha: NadiDosha.middleOnly),
    );
    final instants = [birth, birth + 9.5, birth + 17.25];
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      matching: asked,
    );
    for (final (k, instant) in instants.indexed) {
      final one = found(instant, asked: asked);
      final alone = one.matching!;
      expect(batch.at(k).matching, alone);
      expect(batch.at(k).porutham, one.porutham);
      expect(batch.at(k).kuja, one.kuja);
      expect(batch.at(k).marriageDoshas, one.marriageDoshas);
      final swapped =
          found(
            instant,
            asked: MatchingRequest(asked.partner, partnerRole: MatchRole.bride),
          ).matching!;
      final ours = alone.kootas.first.reading as VarnaKoota;
      final theirs = swapped.kootas.first.reading as VarnaKoota;
      expect((theirs.bride, theirs.groom), (ours.groom, ours.bride));
    }
    ctx.dispose();

    final western = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    expect(
      () => western.chart.found(
        instant: birth,
        place: kathmandu,
        utcOffsetSeconds: 20700,
        matching: asked,
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'matching.partner',
        ),
      ),
    );
    western.dispose();
  });

  test('a chart carries its Western houses', () {
    // Leo's own illustration (*How to Judge a Nativity*, p. 150), "a female
    // born at 2.42 A.M. 13th December, 1835, London", against the SDK
    // test's Moshier recast: Placidus, Saturn rising
    // (`03-design/western-houses.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(-0.1),
      altitudeM: Altitude(0),
    );
    const birth = 2391625.6125;
    bool near(double a, double b) => ((a - b + 540) % 360 - 180).abs() < 0.01;
    Chart found(double instant, {WesternHouseRequest? asked}) =>
        ctx.chart.found(
          instant: instant,
          place: london,
          utcOffsetSeconds: 0,
          outerPlanets: true,
          westernHouses: asked,
        );
    expect(found(birth).westernHouses, isNull);

    final houses =
        found(birth, asked: const WesternHouseRequest()).westernHouses!;
    expect(houses.system, HouseSystem.placidus);
    expect(houses.cuspsDeg, hasLength(12));
    expect(
      near(houses.cuspsDeg[0], 202.1436),
      isTrue,
      reason: '${houses.cuspsDeg}',
    );
    expect(
      near(houses.cuspsDeg[9], 119.3147),
      isTrue,
      reason: '${houses.cuspsDeg}',
    );
    expect(
      near(houses.reachDeg, 191.6089),
      isTrue,
      reason: '${houses.reachDeg}',
    );
    WesternHousePlacement placed(Graha graha) =>
        houses.planets.firstWhere((one) => one.graha == graha);
    expect(
      (placed(Graha.saturn).house, placed(Graha.saturn).withAscendant),
      (1, true),
    );
    expect(
      (placed(Graha.sun).house, placed(Graha.sun).withAscendant),
      (2, false),
    );
    expect(placed(Graha.mars).house, 3);

    const koch = WesternHouseRequest(system: HouseSystem.koch);
    expect(found(birth, asked: koch).westernHouses!.system, HouseSystem.koch);
    final instants = [birth, birth + 100.5];
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: london,
      utcOffsetSeconds: 0,
      outerPlanets: true,
      westernHouses: const WesternHouseRequest(),
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).westernHouses,
        found(instant, asked: const WesternHouseRequest()).westernHouses,
      );
    }
    ctx.dispose();
  });

  test('two names match star to star without a chart', () {
    // प्रि is Uttara Phalguni's 4th syllable and कृ Mrigashira's 4th
    // (Svarodaya vv. 3-8, the first consonant taking the cluster's vowel);
    // pa and ka stand 4 vargas apart, eater and eaten (VI.35).
    final ctx = teistro.context(
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final read = ctx.matching.naam('प्रिया', 'कृष्ण');
    expect(
      (read.bride.nakshatra, read.bride.quarter, read.bride.varga),
      (Nakshatra.uttaraPhalguni, 4, NameVarga.rat),
    );
    expect(
      (read.groom.nakshatra, read.groom.quarter, read.groom.varga),
      (Nakshatra.mrigashira, 4, NameVarga.cat),
    );
    expect(
      read.varga,
      const VargaKoota(
        bride: NameVarga.rat,
        groom: NameVarga.cat,
        relation: VargaRelation.enemy,
      ),
    );
    expect(read.ashta.kootas, hasLength(8));
    expect(
      read.ashta.total,
      read.ashta.kootas.fold<double>(0, (sum, row) => sum + row.points),
    );
    expect(read.porutham.considerations, hasLength(10));
    // The ten considerations count the groom's star from the bride's.
    expect(
      (read.porutham.considerations.first.reading as DhinamPorutham).count,
      21,
    );

    // IAST, when declared, is the same name.
    expect(
      ctx.matching.naam(
        'priyā',
        'kṛṣṇa',
        const NaamRules(name: NameRules(latin: LatinName.iast)),
      ),
      read,
    );

    // Abhijit's row is none of the 27: refused, unless the rules place it.
    final placed = ctx.matching.naam(
      'सीता',
      'ज़ोया',
      const NaamRules(name: NameRules(abhijit: AbhijitPada.shravana)),
    );
    expect((placed.groom.nakshatra, placed.groom.quarter), (null, 3));
    for (final (run, field) in [
      (() => ctx.matching.naam('सीता', 'ज़ोया'), 'naam.groom.abhijit'),
      (() => ctx.matching.naam('Sita', 'राम'), 'naam.bride.name'),
    ]) {
      expect(
        run,
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test(
    'a chart carries its avakahada, whose syllable names the birth pada',
    () {
      // Read as a name, in either script, the birth syllable is the birth
      // pada (C297); null unless asked; a tropical chart refuses it by name.
      final ctx = teistro.context(
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      );
      final kathmandu = Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      );
      expect(
        ctx.chart
            .found(
              instant: 2451545.0,
              place: kathmandu,
              utcOffsetSeconds: 20700,
            )
            .avakahada,
        isNull,
      );
      final padas = <(Nakshatra, int)>{};
      for (var step = 0; step < 28; step++) {
        final read =
            ctx.chart
                .found(
                  instant: 2451545.0 + step * 0.83,
                  place: kathmandu,
                  utcOffsetSeconds: 20700,
                  avakahada: true,
                )
                .avakahada!;
        final named =
            ctx.matching
                .naam(read.syllable.devanagari, read.syllable.devanagari)
                .bride;
        expect(
          (named.cell, named.nakshatra, named.quarter, named.varga),
          (read.syllable.cell, read.nakshatra, read.pada, read.syllable.varga),
          reason: read.syllable.devanagari,
        );
        expect(
          ctx.matching
              .naam(
                read.syllable.iast,
                read.syllable.iast,
                const NaamRules(name: NameRules(latin: LatinName.iast)),
              )
              .bride,
          named,
          reason: read.syllable.iast,
        );
        padas.add((read.nakshatra, read.pada));
      }
      expect(padas.length, greaterThan(20));
      ctx.dispose();

      final western = teistro.context(
        profile: 'western-tropical-default',
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      );
      expect(
        () => western.chart.found(
          instant: 2451545.0,
          place: kathmandu,
          utcOffsetSeconds: 20700,
          avakahada: true,
        ),
        throwsA(
          isA<TeistroException>().having((e) => e.field, 'field', 'avakahada'),
        ),
      );
      western.dispose();
    },
  );

  test('a chart carries its synastry with a partner', () {
    // King George V and Queen Mary (Leo, *How to Judge a Nativity*,
    // p. 130): the recast's closest contacts, the lagna left out on
    // request, a batch the charts one at a time, and refusals named in the
    // record (`03-design/western-synastry.md`).
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final george = Observer(
      latitudeDeg: Latitude(51.5045),
      longitudeDeg: Longitude(-0.1366),
      altitudeM: Altitude(0),
    );
    const birth = 2402390.554166667;
    final mary = Partner(
      instant: 2403113.499305556,
      place: Observer(
        latitudeDeg: Latitude(51.5058),
        longitudeDeg: Longitude(-0.1878),
        altitudeM: Altitude(0),
      ),
    );
    Chart found(
      double instant, {
      SynastryRequest? asked,
      bool outerPlanets = false,
    }) => ctx.chart.found(
      instant: instant,
      place: george,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      synastry: asked,
    );
    expect(found(birth).synastry, isNull);

    final rows =
        found(
          birth,
          asked: SynastryRequest(mary),
          outerPlanets: true,
        ).synastry!;
    const mars = NatalGraha(Graha.mars);
    for (final (first, aspect, second, fromExactDeg) in [
      (mars, WesternAspect.opposition, const NatalLagna(), 0.32),
      (mars, WesternAspect.sextile, const NatalGraha(Graha.sun), 0.39),
      (
        const NatalGraha(Graha.pluto),
        WesternAspect.conjunction,
        const NatalGraha(Graha.pluto),
        1.69,
      ),
    ]) {
      final row = rows.firstWhere(
        (row) =>
            row.first == first && row.aspect == aspect && row.second == second,
      );
      expect(row.fromExactDeg, closeTo(fromExactDeg, 0.01));
    }
    expect(rows.every((row) => row.fromExactDeg <= row.orbDeg), isTrue);

    final without =
        found(birth, asked: SynastryRequest(mary, lagna: false)).synastry!;
    expect(
      without.every(
        (row) => row.first is NatalGraha && row.second is NatalGraha,
      ),
      isTrue,
    );

    final instants = [birth, birth - 3000.25];
    final two = SynastryRequest(
      mary,
      table: const WesternAspectRequest(
        aspects: [WesternAspect.sextile, WesternAspect.opposition],
      ),
    );
    final batch = ctx.chart.foundMany(
      instants: instants,
      place: george,
      utcOffsetSeconds: 0,
      synastry: two,
    );
    for (final (k, instant) in instants.indexed) {
      expect(batch.at(k).synastry, found(instant, asked: two).synastry);
    }

    // The parallels across: none unless asked, then the recast's closest
    // (Uranus with Uranus, 0.05°) and a contrary pair within 0.95°.
    expect(
      found(birth, asked: SynastryRequest(mary)).synastryParallels,
      isNull,
    );
    final parallels =
        found(
          birth,
          asked: SynastryRequest(mary, parallels: const ParallelRequest()),
          outerPlanets: true,
        ).synastryParallels!;
    const uranus = NatalGraha(Graha.uranus);
    expect((parallels.first.first, parallels.first.second), (uranus, uranus));
    expect(parallels.first.apartDeg, closeTo(0.049, 0.005));
    expect(parallels.any((row) => row.contrary && row.apartDeg < 0.95), isTrue);
    expect(
      parallels.every((row) => row.apartDeg <= row.orbDeg && row.orbDeg == 1),
      isTrue,
    );
    expect(
      found(
        birth,
        asked: SynastryRequest(
          mary,
          parallels: const ParallelRequest(orbDeg: 0.000001),
        ),
      ).synastryParallels,
      isEmpty,
    );

    // The antiscia across: none unless asked, then the recast's seven under
    // Lilly's moieties, closest first (Saturn's antiscion on Jupiter, 0.09°).
    expect(found(birth, asked: SynastryRequest(mary)).synastryAntiscia, isNull);
    final reflected =
        found(
          birth,
          asked: SynastryRequest(mary, antiscia: const AntisciaRequest()),
        ).synastryAntiscia!;
    expect(
      [for (final row in reflected) (row.first, row.second, row.contrary)],
      [
        (Graha.saturn, Graha.jupiter, false),
        (Graha.saturn, Graha.moon, false),
        (Graha.mercury, Graha.mars, false),
        (Graha.mars, Graha.saturn, true),
        (Graha.venus, Graha.mars, false),
        (Graha.mars, Graha.mercury, false),
        (Graha.mars, Graha.sun, false),
      ],
    );
    expect(reflected.first.apartDeg, closeTo(0.089, 0.005));
    final far = Partner(instant: 9000000, place: mary.place);
    for (final (asked, field) in [
      (
        SynastryRequest(mary, table: WesternAspectRequest.lilly),
        'synastry.lagna',
      ),
      (SynastryRequest(far), 'synastry.partner'),
      (
        SynastryRequest(mary, parallels: const ParallelRequest(orbDeg: 11)),
        'synastry.parallels.orbDeg',
      ),
      (
        SynastryRequest(
          mary,
          antiscia: const AntisciaRequest(
            orbs: OrbModel.byAspect({WesternAspect.trine: 3}),
          ),
        ),
        'synastry.antiscia.orbs.orbs',
      ),
      (
        SynastryRequest(
          mary,
          antiscia: const AntisciaRequest(cusps: WesternHouseRequest()),
        ),
        'synastry.antiscia.cusps',
      ),
    ]) {
      expect(
        () => found(birth, asked: asked),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    // Lilly's own reading leaves the lagna out, and the outer three need
    // leaving out too.
    expect(
      found(birth, asked: SynastryRequest.lilly(mary)).synastry,
      isNotEmpty,
    );
    ctx.dispose();
  });

  test(
    'a synastry reads the equal distances and makes the composite and Davison birth',
    () {
      // King George V and Queen Mary, against the SDK test's Moshier recast
      // (`03-design/western-composites.md`).
      final ctx = teistro.context(
        profile: 'western-tropical-default',
        ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
      );
      final george = Observer(
        latitudeDeg: Latitude(51.5045),
        longitudeDeg: Longitude(-0.1366),
        altitudeM: Altitude(0),
      );
      const birth = 2402390.554166667;
      final mary = Partner(
        instant: 2403113.499305556,
        place: Observer(
          latitudeDeg: Latitude(51.5058),
          longitudeDeg: Longitude(-0.1878),
          altitudeM: Altitude(0),
        ),
      );
      Chart found(SynastryRequest asked) => ctx.chart.found(
        instant: birth,
        place: george,
        utcOffsetSeconds: 0,
        outerPlanets: true,
        synastry: asked,
      );
      bool near(double a, double b) => ((a - b + 540) % 360 - 180).abs() < 0.01;

      final plain = found(SynastryRequest(mary));
      expect(plain.synastryComposite, isNull);
      expect(plain.synastryDavison, isNull);

      // The equal distances across: none unless asked, then the SDK test's
      // recast within 1°, her Venus on his Sun and Neptune the closest.
      expect(plain.synastryMidpoints, isNull);
      final equal =
          found(
            SynastryRequest(mary, midpoints: const MidpointRequest(orbDeg: 1)),
          ).synastryMidpoints!;
      expect(equal, hasLength(10));
      final closest = equal.first;
      expect(
        (
          closest.first,
          closest.second,
          closest.middle,
          closest.partnersPair,
          closest.far,
        ),
        (Graha.sun, Graha.neptune, Graha.venus, true, false),
      );
      expect(closest.fromAxisDeg, closeTo(0.14, 0.01));

      final chart = found(
        SynastryRequest(mary, composite: true, davison: true),
      );
      final composite = chart.synastryComposite!;
      expect(composite.planets, hasLength(10));
      for (final (graha, longitude) in [
        (Graha.sun, 68.8193),
        (Graha.moon, 259.7289),
        (Graha.mars, 130.5171),
        (Graha.pluto, 44.2555),
      ]) {
        final at = composite.planets.firstWhere((one) => one.graha == graha);
        expect(
          near(at.longitudeDeg, longitude),
          isTrue,
          reason: '$graha ${at.longitudeDeg}',
        );
      }
      expect(
        near(composite.midheavenDeg, 258.1797),
        isTrue,
        reason: '${composite.midheavenDeg}',
      );
      expect(
        near(composite.lagnaDeg, 334.007),
        isTrue,
        reason: '${composite.lagnaDeg}',
      );
      expect(composite.lagnaTurned, isFalse);
      // Its Placidus cusps, the near midpoints of the two charts', the first
      // and tenth its lagna and midheaven.
      final cusps = composite.cuspsDeg!;
      expect(cusps, hasLength(12));
      expect(near(cusps[2], 57.8601), isTrue, reason: '${cusps[2]}');
      expect(
        (cusps[0], cusps[9]),
        (composite.lagnaDeg, composite.midheavenDeg),
      );

      // The Davison birth is a Partner, so it founds a chart as a birth does.
      final davison = chart.synastryDavison!;
      expect(davison.instant, closeTo(2402752.026736111, 1e-8));
      expect(davison.place.longitudeDeg, closeTo(-0.1622, 1e-9));
      expect(davison.utcOffsetSeconds, 0);
      final between = ctx.chart.found(
        instant: davison.instant,
        place: davison.place,
        utcOffsetSeconds: davison.utcOffsetSeconds,
      );
      final mars = between.grahas.firstWhere((one) => one.graha == Graha.mars);
      expect(
        near(mars.tropicalDeg, 20.1267),
        isTrue,
        reason: '${mars.tropicalDeg}',
      );
      ctx.dispose();
    },
  );

  test('a chart carries the outer planets when asked', () {
    final ctx = teistro.context(
      profile: 'western-tropical-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final london = Observer(
      latitudeDeg: Latitude(51.5),
      longitudeDeg: Longitude(0),
      altitudeM: Altitude(0),
    );
    const birth = 2400629.742361111;
    Chart found({
      bool outerPlanets = false,
      ProgressionsRequest? progressions,
    }) => ctx.chart.found(
      instant: birth,
      place: london,
      utcOffsetSeconds: 0,
      outerPlanets: outerPlanets,
      progressions: progressions,
    );
    final bare = found();
    expect(bare.outer, isEmpty);
    final asked = found(outerPlanets: true);
    expect(asked.outer.map((at) => at.graha), [
      Graha.uranus,
      Graha.neptune,
      Graha.pluto,
    ]);
    expect(
      asked.grahas.map((at) => at.longitudeDeg),
      bare.grahas.map((at) => at.longitudeDeg),
      reason: 'the nine are unmoved',
    );
    for (final at in asked.outer) {
      expect(at.distanceAu, greaterThan(15));
      expect(at.house.bhava, inInclusiveRange(1, 12));
    }

    const at = birth + 46 * 365.242189;
    final later =
        found(
          outerPlanets: true,
          progressions: const ProgressionsRequest(at: at),
        ).progressions!;
    expect(later.progressed!.grahas.length, 12);
    expect(later.directed!.planets.length, 12);
    expect(later.progressed!.grahas[11].graha, Graha.pluto);

    // Leo's progressed Moon quincunx Uranus, April 1907 (p. 41).
    const contacts = ProgressionContacts(
      from: 2417484.5,
      to: 2417941.5,
      grahas: [Graha.moon],
      points: [NatalGraha(Graha.uranus)],
    );
    final reached =
        found(
          outerPlanets: true,
          progressions: const ProgressionsRequest(contacts: contacts),
        ).progressions!.contacts!;
    expect(reached, hasLength(1));
    expect(reached.single.to, const NatalGraha(Graha.uranus));
    expect(reached.single.angle, 150);
    expect(reached.single.life, inInclusiveRange(2417635.5, 2417696.4));
    expect(
      () => found(progressions: const ProgressionsRequest(contacts: contacts)),
      throwsA(isA<TeistroException>()),
    );
    ctx.dispose();
  });

  test('a chart carries its accidental fortitudes', () {
    final ctx = teistro.context(
      profile: 'conformance-baseline',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(0),
    );
    const instants = [2460676.5, 2460676.75];
    Chart found(
      double instant, {
      FortitudeRequest? fortitudes,
      DignityRequest? dignities,
    }) => ctx.chart.found(
      instant: instant,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      fortitudes: fortitudes,
      dignities: dignities,
    );
    expect(found(instants[0]).fortitudes, isNull);

    final chart = found(instants[0], fortitudes: const FortitudeRequest());
    final read = chart.fortitudes!;
    expect(read.dignities, chart.dignities);
    expect(read.sky.houses, HouseSystem.regiomontanus);
    expect(read.sky.cuspsDeg, hasLength(12));
    expect(read.sky.speedsDegPerDay, hasLength(7));
    expect(read.rules, AccidentalRules.lilly);
    expect(read.scores, AccidentalScores.lilly);
    expect(
      read.planets.map((at) => at.planet),
      read.dignities.planets.map((at) => at.planet),
    );
    const solar = {
      Accident.cazimi,
      Accident.combust,
      Accident.underBeams,
      Accident.freeFromCombustion,
    };
    for (final (k, at) in read.planets.indexed) {
      final own = read.dignities.planets[k];
      final points = [
        read.scores.houses[at.house - 1],
        for (final line in at.accidents) line.points,
      ];
      expect(
        at.fortitude,
        points.where((n) => n > 0).fold<int>(0, (a, b) => a + b),
        reason: '${at.planet}',
      );
      expect(
        at.debility,
        points.where((n) => n < 0).fold<int>(0, (a, b) => a - b),
        reason: '${at.planet}',
      );
      expect(
        at.net,
        own.score + own.reception + at.fortitude - at.debility,
        reason: '${at.planet}',
      );
      expect(
        at.accidents.where((line) => solar.contains(line.accident)),
        hasLength(at.planet == Graha.sun ? 0 : 1),
        reason: '${at.planet}',
      );
    }

    // The almutens: Lilly's of the figure is the greatest net, Fortune the
    // ascendant plus the Moon less the Sun, and every house has one.
    final almutens = read.almutens;
    expect(almutens.rules, AlmutenRules.lilly);
    expect(
      [for (final at in almutens.figure.totals) at.total],
      [for (final at in read.planets) at.net],
    );
    final greatest = read.planets
        .map((at) => at.net)
        .reduce((a, b) => a > b ? a : b);
    expect(almutens.figure.almutens, [
      for (final at in read.planets)
        if (at.net == greatest) at.planet,
    ]);
    double longitude(Graha planet) =>
        read.dignities.planets
            .firstWhere((at) => at.planet == planet)
            .longitudeDeg;
    final fortune =
        (read.sky.ascendantDeg + longitude(Graha.moon) - longitude(Graha.sun)) %
        360;
    expect(almutens.fortuneDeg, closeTo(fortune, 1e-9));
    expect(almutens.houses, hasLength(12));
    for (final almuten in [
      almutens.figure,
      almutens.places,
      ...almutens.houses,
    ]) {
      expect(almuten.almutens, isNotEmpty);
      expect(almuten.partakers.where(almuten.almutens.contains), isEmpty);
    }

    // The answer's rules and scores are a request as they stand, and one
    // changed is obeyed.
    expect(
      found(
        instants[0],
        fortitudes: FortitudeRequest(
          rules: read.rules,
          scores: read.scores,
          almuten: almutens.rules,
        ),
      ).fortitudes,
      read,
    );
    const asked = FortitudeRequest(
      rules: AccidentalRules(
        beamsDeg: 15,
        partile: Partile.within,
        partileOrbDeg: 1,
        siege: Siege.within,
        siegeSpanDeg: 30,
      ),
      scores: AccidentalScores(regulus: 5),
      almuten: AlmutenRules(
        place: PlaceReading.sign,
        fortune: FortuneRule.reversedByNight,
      ),
    );
    final other = found(instants[0], fortitudes: asked).fortitudes!;
    expect(other.rules, asked.rules);
    expect(other.scores, asked.scores);
    expect(other.almutens.rules, asked.almuten);

    final batch = ctx.chart.foundMany(
      instants: instants,
      place: kathmandu,
      utcOffsetSeconds: 20700,
      fortitudes: const FortitudeRequest(),
    );
    for (final (k, instant) in instants.indexed) {
      expect(
        batch.at(k).fortitudes,
        found(instant, fortitudes: const FortitudeRequest()).fortitudes,
        reason: 'each alone',
      );
    }

    for (final (fortitudes, dignities, field) in [
      (
        const FortitudeRequest(rules: AccidentalRules(beamsDeg: -1)),
        null,
        'fortitudes.rules.beamsDeg',
      ),
      (const FortitudeRequest(), const DignityRequest(), 'dignities'),
    ]) {
      expect(
        () => found(instants[0], fortitudes: fortitudes, dignities: dignities),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();
  });

  test('a chart carries its KP reading', () {
    final ctx = teistro.context(
      profile: 'kp-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final place = Observer(
      latitudeDeg: Latitude(13.08),
      longitudeDeg: Longitude(80.27),
      altitudeM: Altitude(6),
    );
    const births = [2447995.4895833335, 2451545.2];
    KpReading? found(double instant, KpRequest? asked, [Context? under]) =>
        (under ?? ctx).chart
            .found(
              instant: instant,
              place: place,
              utcOffsetSeconds: 19800,
              kp: asked,
            )
            .kp;
    expect(found(births[0], null), isNull);

    final reading = found(births[0], const KpRequest())!;
    expect(reading.chart.system, HouseSystem.placidus);
    expect(reading.chart.cusps, hasLength(12));
    expect(reading.significators.houses, hasLength(12));
    for (final planet in reading.chart.planets) {
      final span = planet.lords.subSub.span;
      expect(
        span.start <= planet.longitude && planet.longitude < span.end,
        isTrue,
        reason: '${planet.graha} inside its sub-sub',
      );
    }
    expect(reading.ruling.rules.count, 'FIVE');
    expect(reading.ruling.rulers.every((r) => r.reasons.isNotEmpty), isTrue);
    expect(reading, found(births[0], const KpRequest()), reason: 'a value');

    // A horary number casts the cusps; the ruling planets stay the moment's.
    const asked = KpRequest(number: 74);
    final horary = found(births[0], asked)!;
    final lagna = horary.chart.cusps.first;
    expect(lagna.longitude, lagna.lords.sub.span.start);
    expect(horary.ruling, reading.ruling);

    final batch = ctx.chart.foundMany(
      instants: births,
      place: place,
      utcOffsetSeconds: 19800,
      kp: asked,
    );
    for (final (k, instant) in births.indexed) {
      expect(batch.at(k).kp, found(instant, asked), reason: 'each alone');
    }

    for (final (bad, field) in [
      (const KpRequest(number: 250), 'kp.number'),
      (const KpRequest(clock: 90000), 'kp.clock'),
    ]) {
      expect(
        () => found(births[0], bad),
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field)),
      );
    }
    ctx.dispose();

    final lahiri = teistro.context(
      profile: 'nepali-default',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    expect(
      () => found(births[0], const KpRequest(), lahiri),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'frame.ayanamsha',
        ),
      ),
    );
    expect(
      found(
        births[0],
        const KpRequest(anyAyanamsha: true),
        lahiri,
      )!.chart.cusps,
      hasLength(12),
    );
    lahiri.dispose();
  });

  test(
    'a chart carries its transits, each verdict its own house and vedha',
    () {
      final ctx = context();
      final place = Observer(
        latitudeDeg: Latitude(27.7172),
        longitudeDeg: Longitude(85.324),
        altitudeM: Altitude(1400),
      );
      const birth = 2447995.4895833335;
      expect(
        ctx.chart
            .found(instant: birth, place: place, utcOffsetSeconds: 20700)
            .gochar,
        isEmpty,
      );
      final instants = [for (var k = 0; k < 24; k++) 2460676.5 + 30 * k];
      final verdicts = <GocharVerdict>{};
      for (final from in GocharFrom.values) {
        final readings =
            ctx.chart
                .found(
                  instant: birth,
                  place: place,
                  utcOffsetSeconds: 20700,
                  gochar: GocharRequest(instants: instants, from: from),
                )
                .gochar;
        expect(readings, hasLength(instants.length));
        for (final (k, reading) in readings.indexed) {
          expect(reading.instant, instants[k]);
          expect(reading.reference.from, from);
          expect(reading.rules.nodeVedha, NodeVedha.likeTheSun);
          expect(reading.rules.nodeObstruction, NodeObstruction.notEachOther);
          expect(reading.grahas, hasLength(9));
          for (final g in reading.grahas) {
            expect(g.house, inInclusiveRange(1, 12));
            expect(
              g.transit.degrees,
              allOf(greaterThanOrEqualTo(0), lessThan(30)),
            );
            final expected =
                !g.goodHouse
                    ? GocharVerdict.notGood
                    : g.obstructedBy.isNotEmpty
                    ? GocharVerdict.obstructed
                    : GocharVerdict.good;
            expect(g.verdict, expected, reason: '${g.graha} in ${g.house}');
            if (!g.goodHouse) expect(g.vedhaHouse, isNull);
            verdicts.add(g.verdict);
          }
          final rahu = reading.grahas[7];
          final ketu = reading.grahas[8];
          expect(
            (ketu.house - rahu.house) % 12,
            6,
            reason: 'the nodes stand opposite',
          );
          expect(rahu.obstructedBy, isNot(contains(Graha.ketu)));
        }
      }
      expect(verdicts, GocharVerdict.values.toSet());
      final judged =
          ctx.chart
              .found(
                instant: birth,
                place: place,
                utcOffsetSeconds: 20700,
                gochar: GocharRequest(instants: instants, ashtakavarga: true),
              )
              .gochar;
      for (final reading in judged) {
        expect(reading.ashtakavarga, hasLength(7));
        for (final (k, one) in reading.ashtakavarga!.indexed) {
          final moving = reading.grahas[k];
          expect(one.graha, moving.graha);
          expect(one.good, one.bindus >= 5);
          expect(
            one.kakshya.index,
            (moving.transit.degrees / 3.75).floor() + 1,
          );
        }
      }
      expect(
        () => ctx.chart.found(
          instant: birth,
          place: place,
          utcOffsetSeconds: 20700,
          gochar: const GocharRequest(instants: []),
        ),
        throwsA(
          isA<TeistroException>().having(
            (e) => e.field,
            'field',
            'gochar.instants',
          ),
        ),
      );
      ctx.dispose();
    },
  );

  test('a chart carries its karakamsha and its Brahma graha, or why not', () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    expect(
      ctx.chart
          .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
          .jaimini,
      isNull,
    );
    final outcomes = <BrahmaOutcome>{};
    for (var step = 0; step < 40; step++) {
      final reading =
          ctx.chart
              .found(
                instant: 2451545.0 + step * 0.37,
                place: place,
                utcOffsetSeconds: 20700,
                jaimini: true,
              )
              .jaimini!;
      final k = reading.karakamsha;
      final b = reading.brahma;
      for (final houses in [k.inRasi, k.inNavamsha]) {
        expect(houses, hasLength(9));
        for (final house in houses) {
          expect(house, inInclusiveRange(1, 12));
        }
      }
      expect(k.inNavamsha[k.atmakaraka.id], 1);
      expect(b.rule, BrahmaRule.verses);
      expect(b.graha == null, b.none != null);
      expect(b.none, isNot(BrahmaOutcome.found));
      if (b.graha != null) {
        expect(b.qualified, contains(b.passedFrom ?? b.graha));
      }
      outcomes.add(b.none ?? BrahmaOutcome.found);
      expect(reading.grahaArudhas, hasLength(9));
      expect(reading.grahaArudhas.take(7), everyElement(isNotNull));
      // The default co-lordship gives the nodes no own sign, so no arudha.
      expect(reading.grahaArudhas.skip(7), everyElement(isNull));
    }
    expect(outcomes, contains(BrahmaOutcome.found));
    expect(outcomes.length, greaterThan(1));
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
  // A chart names the ayanamsha it was read under. The blob carried it and
  // no layer read it, so a reader could see an offset and not whose; and a
  // tropical chart has none, which is not an ayanamsha of nought.
  test('a chart names its ayanamsha, and a tropical one has none', () {
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final sidereal = context();
    final chart = sidereal.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
    );
    expect(chart.ayanamsha, Ayanamsha.lahiri);
    expect(chart.ayanamshaCustom, isFalse);
    expect(chart.ayanamshaOffsetDeg, isNot(0.0));
    sidereal.dispose();
    final tropical = context(
      settings: {
        'frame': {'zodiac': 'TROPICAL'},
      },
    );
    final western = tropical.chart.found(
      instant: 2451545.0,
      place: place,
      utcOffsetSeconds: 20700,
    );
    expect(western.ayanamsha, isNull);
    expect(western.ayanamshaCustom, isFalse);
    tropical.dispose();
  });

  // A chart's day and an almanac's are one record, and its date is the one
  // `calendar.convert` takes. Before, Dart flattened the weekday and the
  // sunrise onto the chart and left the rest out.
  test('a chart\'s day is the almanac\'s, and its date converts', () {
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    final ctx = context();
    final day =
        ctx.chart
            .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
            .day;
    final same =
        ctx.almanac
            .day(date: day.date, place: place, utcOffsetSeconds: 20700)
            .day;
    expect(
      [same.vara, same.sunrise, same.sunset, same.nextSunrise, same.date.day],
      [day.vara, day.sunrise, day.sunset, day.nextSunrise, day.date.day],
    );
    expect(
      [
        day.date.calendar,
        day.date.era,
        day.date.year,
        day.date.month,
        day.date.day,
      ],
      [Calendar.bikramSambat, Era.vikrama, 2056, 9, 17],
    );
    final gregorian = ctx.calendar.convert(day.date, Calendar.gregorian);
    expect([gregorian.year, gregorian.month, gregorian.day], [2000, 1, 1]);
    expect(day.vara, Vara.shanivara);
    expect(day.sunrise < day.sunset && day.sunset < 2451545.0, isTrue);
    expect(2451545.0 < day.nextSunrise, isTrue);
    expect(day.polar, isNull);
    expect(day.convention, Sunrise.centreNoRefraction);
    expect(day.customAltitudeDeg, isNull);
    ctx.dispose();

    // A custom altitude is a number and no named convention.
    final custom = context(
      settings: {
        'day': {
          'sunrise': {'kind': 'CUSTOM', 'altitude_deg': -0.5},
        },
      },
    );
    final own =
        custom.chart
            .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
            .day;
    expect(own.convention, isNull);
    expect(own.customAltitudeDeg, -0.5);
    expect(own.air, isNull);
    custom.dispose();

    // An air named for a refracted convention: what was left out comes
    // back resolved at the place, 856 hPa at 1400 m, and the thinner air
    // lifts the Sun less, so it clears the horizon later than under the
    // fixed 34′.
    LocalDay refracted(Map<String, Object?> sunrise) {
      final each = context(
        settings: {
          'day': {'sunrise': sunrise},
        },
      );
      try {
        return each.chart
            .found(instant: 2451545.0, place: place, utcOffsetSeconds: 20700)
            .day;
      } finally {
        each.dispose();
      }
    }

    final almanac = refracted({
      'kind': 'NAMED',
      'which': 'UPPER_LIMB_REFRACTION',
    });
    final standard = refracted({
      'kind': 'ATMOSPHERIC',
      'which': 'UPPER_LIMB_REFRACTION',
      'air': <String, Object?>{},
    });
    expect(almanac.air, isNull);
    expect(standard.convention, Sunrise.upperLimbRefraction);
    expect(standard.air!.pressureHpa, closeTo(855.99, 0.01));
    expect(standard.air!.temperatureC, 15.0);
    final later = (standard.sunrise - almanac.sunrise) * 86400.0;
    expect(later > 20.0 && later < 35.0, isTrue, reason: '$later s');
    final weather = refracted({
      'kind': 'ATMOSPHERIC',
      'which': 'LOWER_LIMB_REFRACTION',
      'air': {'pressure_hpa': 870.0, 'temperature_c': -4.5},
    });
    expect(
      [weather.air!.pressureHpa, weather.air!.temperatureC],
      [870.0, -4.5],
    );
    expect(
      () => refracted({
        'kind': 'ATMOSPHERIC',
        'which': 'CENTRE_NO_REFRACTION',
        'air': <String, Object?>{},
      }),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.message,
          'message',
          contains('does not refract'),
        ),
      ),
    );

    // Tromsø at midsummer: civil midnight holds the instant and says so;
    // the nearest real sunrise is weeks away, and the refusal names the
    // policy rather than the instant.
    final tromso = Observer(
      latitudeDeg: Latitude(69.65),
      longitudeDeg: Longitude(18.96),
      altitudeM: Altitude(0),
    );
    Context under(String policy) => context(
      settings: {
        'day': {'polar_day_policy': policy},
      },
    );
    final civil = under('CIVIL_MIDNIGHT');
    final polar =
        civil.chart
            .found(instant: 2451716.5, place: tromso, utcOffsetSeconds: 7200)
            .day
            .polar!;
    expect(
      [polar.kind, polar.policy],
      [PolarKind.day, PolarDayPolicy.civilMidnight],
    );
    civil.dispose();
    final nearest = under('NEAREST_EVENT');
    expect(
      () => nearest.chart.found(
        instant: 2451716.5,
        place: tromso,
        utcOffsetSeconds: 7200,
      ),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.field,
          'field',
          'day.polar_day_policy',
        ),
      ),
    );
    nearest.dispose();
  });

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
  /// A consumer's system may count over the twenty-eight nakshatras with
  /// Abhijit in groups of its own: Shashtihayani stated as BPHS states it
  /// reads as the catalogued one (crux C1).
  test('a consumer system counted with Abhijit reads as the text row', () {
    const shashti = UduDashaDefinition(
      key: 'ACME_SHASHTI',
      lords: [
        DashaLord(Graha.jupiter, 10),
        DashaLord(Graha.sun, 10),
        DashaLord(Graha.mars, 10),
        DashaLord(Graha.moon, 6),
        DashaLord(Graha.mercury, 6),
        DashaLord(Graha.venus, 6),
        DashaLord(Graha.saturn, 6),
        DashaLord(Graha.rahu, 6),
      ],
      reference: Nakshatra.ashwini,
      groups: [3, 4, 3, 4, 3, 4, 3, 4],
      wheel: DashaWheel.withAbhijit,
      repeats: false,
    );
    final ctx = teistro.context(testProvider: true, dashaSystems: [shashti]);
    addTearDown(ctx.dispose);
    final [consumer, shipped] =
        ctx.chart
            .found(
              instant: 2451545.0,
              place: Observer(
                latitudeDeg: Latitude(27.7172),
                longitudeDeg: Longitude(85.324),
                altitudeM: Altitude(1400),
              ),
              utcOffsetSeconds: 20700,
              dashas: [
                DashaSystem.registered('ACME_SHASHTI'),
                DashaSystem.shashtihayani,
              ],
            )
            .dashas;
    expect(consumer.periods.length, shipped.periods.length);
    for (var i = 0; i < shipped.periods.length; i++) {
      final (a, b) = (consumer.periods[i], shipped.periods[i]);
      expect((a.path, a.lord, a.from, a.to), (b.path, b.lord, b.from, b.to));
    }
  });

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
    // Every word a definition writes is a key, and its fields are spelt
    // as the document spells them; the SDK reads it back as written.
    const sthira = RashiDashaDefinition(
      key: 'ACME_STHIRA',
      start: RashiStart.arudhaLagna,
      order: RashiOrder.trineGroups,
      length: RashiLength.byModality(movable: 7, fixed: 8, dual: 9),
      namedLord: RashiNamedLord.first,
      strongerOf: [1, 7],
    );
    expect(sthira.toJson(), {
      'kernel': 'RASHI',
      'key': 'ACME_STHIRA',
      'start': 'ARUDHA_LAGNA',
      'order': 'TRINE_GROUPS',
      'length': {
        'BY_MODALITY': {'movable': 7, 'fixed': 8, 'dual': 9},
      },
      'named_lord': 'FIRST',
      'stronger_of': [1, 7],
    });
    teistro
        .context(
          testProvider: true,
          dashaSystems: [
            sthira,
            const RashiDashaDefinition(
              key: 'ACME_NAVA',
              length: RashiLength.fixed(9),
            ),
          ],
        )
        .dispose();
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
      // Among its own claimants, unless it succeeds the Moon, whose Ithasala
      // may be with any planet.
      final succeedsTheMoon = const [
        VarsheshaChosen.moonsIthasala,
        VarsheshaChosen.moonsSignLord,
      ].contains(lord.chosen);
      if (succeedsTheMoon) {
        expect(lord.moonPassedOver, isTrue);
      } else {
        expect(lord.claims.map((claim) => claim.graha), contains(lord.graha));
      }
      final ranked = [for (final claim in lord.claims) claim.vishwa.total];
      final sorted = [...ranked]..sort((int a, int b) => b - a);
      expect(ranked, orderedEquals(sorted));
      expect(lord.vishwa.toString(), matches(r'^\d\d:\d\d:\d\d$'));
      expect(lord.vishwa.units, lord.vishwa.total ~/ 3600);
    }

    // The pairs that make a yoga: never a neutral aspect, and each of
    // Table X-3's four kinds standing where its own degrees put it.
    final kinds = <TajikaYoga>{};
    for (final one in cast) {
      for (final pair in one.annual!.yogas) {
        expect(pair.drishti, isNot(TajikaDrishti.none));
        expect(pair.orbDeg, greaterThan(0));
        kinds.add(pair.yoga);
        if (pair.yoga == TajikaYoga.ithasalaVartamana) {
          expect(pair.apartDeg, greaterThanOrEqualTo(1));
        }
        if (pair.yoga == TajikaYoga.ithasalaPoorna) {
          expect(pair.apartDeg.abs(), lessThan(1));
        }
        if (pair.yoga == TajikaYoga.ishrafa) {
          expect(pair.apartDeg, lessThanOrEqualTo(-1));
        }
      }
    }
    // The corpus's own years reach every kind the boundary can say, so a
    // variant that stopped crossing would be caught here and not only in
    // the enum's member count.
    expect(kinds, hasLength(TajikaYoga.values.length));

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
          'varsha.through',
        ),
      ),
    );
    ctx.dispose();
  });

  test("a year's chart answers the sahams asked for", () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    List<Pravesha> years(VarshaRequest varsha) =>
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: varsha,
            )
            .praveshas;

    for (final one in years(
      const VarshaRequest(
        through: 4,
        place: AnnualPlace.birth,
        sahams: Sahams.these([Saham.karyaSiddhi, Saham.punya]),
      ),
    )) {
      final annual = one.annual!;
      expect(annual.sahams.map((p) => p.saham), [
        Saham.karyaSiddhi,
        Saham.punya,
      ]);
      final lagna = annual.lagnaDeg ~/ 30;
      for (final point in annual.sahams) {
        expect(point.longitudeDeg, inInclusiveRange(0, 360));
        final sign = point.longitudeDeg ~/ 30;
        expect(point.sign, Rashi.byId(sign));
        expect(point.house, (sign - lagna) % 12 + 1);
        expect(point.lord.id, lessThan(7));
      }
    }

    // `all` is the forty-one in the source's order; unasked is none.
    final every =
        years(
          const VarshaRequest(
            through: 1,
            place: AnnualPlace.birth,
            sahams: Sahams.all,
          ),
        ).first.annual!.sahams;
    expect(every.map((p) => p.saham), Saham.values);
    expect(
      years(
        const VarshaRequest(through: 1, place: AnnualPlace.birth),
      ).first.annual!.sahams,
      isEmpty,
    );

    // The rules cross in the boundary's casing.
    final never = years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        sahams: Sahams.all,
        sahamRules: SahamRules(addSign: AddSign.never),
      ),
    );
    expect(
      never.every((one) => one.annual!.sahams.every((p) => !p.addedSign)),
      isTrue,
    );
    years(
      const VarshaRequest(
        through: 1,
        place: AnnualPlace.birth,
        sahams: Sahams.these([Saham.mrityu]),
        sahamRules: SahamRules(
          houses: HousePoints.equal,
          roga: RogaReading.saturn,
        ),
      ),
    );

    Matcher refusedBy(String field) =>
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field));
    // Each saham carries its strength, every founded year its Harsha
    // bala, and every birth its own sahams, which need no place.
    final chart = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
      varsha: const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        sahams: Sahams.all,
        sahamStrength: SahamStrengthReadings(
          natures: SahamNatures.chapter,
          friendship: SahamFriendship.positional,
          weakBelow: 5 * 3600,
        ),
        harshaRules: HarshaRules(venus: VenusPlace.twelfth),
      ),
    );
    for (final one in chart.praveshas) {
      final annual = one.annual!;
      for (final saham in annual.sahams) {
        final near =
            saham.strong.contains(SahamStrong.lordConjoins) ||
            saham.strong.contains(SahamStrong.lordAspectsSaham);
        // The two (c) clauses negate each other: exactly one holds.
        expect(near, isNot(saham.weak.contains(SahamWeak.lordApart)));
        expect(saham.seven, hasLength(7));
        expect(
          saham.seven.firstWhere((s) => s.graha == saham.lord).company,
          saham.strong.contains(SahamStrong.lordConjoins),
        );
        expect(saham.inNodeAxis, isNotNull);
      }
      expect(annual.harsha, hasLength(7));
      for (final h in annual.harsha) {
        final parts =
            [
              h.sthana,
              h.uchchaSwakshetra,
              h.striPurusha,
              h.dinaRatri,
            ].where((held) => held).length;
        expect(h.total, 5 * parts);
      }
    }
    expect(chart.sahams, hasLength(41));
    expect(
      chart.sahams.every((s) => !s.strong.contains(SahamStrong.withYearLord)),
      isTrue,
    );
    final natal = ctx.chart.found(
      instant: 2447995.4895833335,
      place: place,
      utcOffsetSeconds: 20700,
      varsha: const VarshaRequest(
        through: 1,
        sahams: Sahams.these([Saham.punya]),
      ),
    );
    expect(natal.sahams.map((s) => s.saham), [Saham.punya]);
    expect(natal.praveshas.first.annual, isNull);
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          sahams: Sahams.these([Saham.punya, Saham.punya]),
        ),
      ),
      refusedBy('varsha.sahams'),
    );
    ctx.dispose();
  });

  test("a year's chart answers the annual dashas asked for", () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    List<Pravesha> years(VarshaRequest varsha) =>
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: varsha,
            )
            .praveshas;
    Matcher refusedBy(String field) =>
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field));

    final asked = years(
      const VarshaRequest(
        through: 3,
        place: AnnualPlace.birth,
        dashas: AnnualDashas.these([DashaSystem.mudda, DashaSystem.patyayini]),
      ),
    );
    for (final (k, one) in asked.indexed) {
      final annual = one.annual!;
      expect(annual.dashas.map((d) => d.system), [
        DashaSystem.mudda,
        DashaSystem.patyayini,
      ]);
      for (final dasha in annual.dashas) {
        expect(dasha.year.from, one.instant);
        if (k + 1 < asked.length) {
          expect(dasha.year.to, closeTo(asked[k + 1].instant, 2e-7));
        }
        expect(dasha.firstLord, dasha.ring.shares[dasha.ring.first].lord);
        // The mahadashas run end to end across the year.
        final mahas = dasha.periods.where((p) => p.level == 1).toList();
        expect(mahas.first.from, dasha.year.from);
        expect(mahas.last.to, dasha.year.to);
        for (var i = 1; i < mahas.length; i += 1) {
          expect(mahas[i].from, mahas[i - 1].to);
        }
        // Mid-year a mahadasha runs, and one of its own under it.
        final chain = dasha.at((dasha.year.from + dasha.year.to) / 2);
        expect(chain, hasLength(2));
        expect(chain[1].path, startsWith('${chain[0].path}/'));
        expect(dasha.at(dasha.year.to), isEmpty);
      }
      final [mudda, patyayini] = annual.dashas;
      expect(mudda.seed, isNotNull);
      expect(mudda.ring.shares, hasLength(9));
      expect(mudda.ring.remaining, inInclusiveRange(0, 1));
      expect(patyayini.seed, isNull);
      expect(patyayini.ring.remaining, isNull);
      expect(patyayini.ring.shares.where((s) => s.sign != null), hasLength(1));
      expect(patyayini.periods.any((p) => p.sign != null), isTrue);
    }
    final firsts = [
      for (final one in asked) one.annual!.dashas.first.ring.first,
    ];
    for (var i = 1; i < firsts.length; i += 1) {
      expect(firsts[i], (firsts[i - 1] + 1) % 9);
    }

    // `all` is the three in the catalogue's order; unasked is none.
    expect(
      years(
        const VarshaRequest(
          through: 1,
          place: AnnualPlace.birth,
          dashas: AnnualDashas.all,
        ),
      ).first.annual!.dashas.map((d) => d.system),
      [DashaSystem.patyayini, DashaSystem.mudda, DashaSystem.varshaYogini],
    );
    expect(
      years(
        const VarshaRequest(through: 1, place: AnnualPlace.birth),
      ).first.annual!.dashas,
      isEmpty,
    );

    // The rules cross in the boundary's casing.
    final days =
        years(
          const VarshaRequest(
            through: 1,
            place: AnnualPlace.birth,
            dashas: AnnualDashas.these([DashaSystem.mudda]),
            dashaRules: AnnualDashaRules(
              clock: YearClock.days(360),
              depth: 1,
              birthPeriod: BirthPeriod.elapsed,
              measure: Balance.temporal,
              balance: MuddaBalance.entryMoon,
            ),
          ),
        ).first.annual!.dashas.single;
    expect(days.year.to - days.year.from, 360);
    expect(days.periods.every((p) => p.level == 1), isTrue);
    years(
      const VarshaRequest(
        through: 1,
        place: AnnualPlace.birth,
        dashas: AnnualDashas.all,
        dashaRules: AnnualDashaRules(
          clock: YearClock.even,
          measure: Balance.spatial,
          birthPeriod: BirthPeriod.compressed,
          balance: MuddaBalance.whole,
        ),
      ),
    );

    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          dashas: AnnualDashas.these([DashaSystem.mudda]),
        ),
      ),
      refusedBy('varsha.dashas'),
    );
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          dashas: AnnualDashas.these([DashaSystem.vimshottari]),
        ),
      ),
      refusedBy('varsha.dashas'),
    );
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          dashas: AnnualDashas.all,
          dashaRules: AnnualDashaRules(clock: YearClock.days(0)),
        ),
      ),
      refusedBy('varsha.dashaRules.clock'),
    );
    ctx.dispose();
  });

  test("a year's chart answers the Tajika yogas for the matters asked", () {
    final ctx = context();
    final place = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    List<Pravesha> years(VarshaRequest varsha) =>
        ctx.chart
            .found(
              instant: 2447995.4895833335,
              place: place,
              utcOffsetSeconds: 20700,
              varsha: varsha,
            )
            .praveshas;

    for (final one in years(
      const VarshaRequest(
        through: 6,
        place: AnnualPlace.birth,
        matters: Matters.houses([7, 1]),
      ),
    )) {
      final annual = one.annual!;
      expect(annual.matters.map((matter) => matter.house), [7, 1]);
      expect(
        [...annual.retrograde, ...annual.combust].every((g) => g.id < 7),
        isTrue,
      );
      for (final matter in annual.matters) {
        // All sixteen are built and the façade reads the states, so every
        // one answers true or false.
        expect(matter.unanswered, isEmpty);
        expect(matter.holds(YearYoga.kuttha), isA<bool>());
        expect(matter.holds(YearYoga.ithasala), isA<bool>());
        expect(matter.karyesha != matter.lagnesha, !matter.sameLord);
        for (final held in matter.held) {
          if (held.between != null) expect(held.between, same(matter.between));
          if (held.legs != null) expect(held.legs!.length, 2);
          expect(
            held.afflictions != null,
            held.yoga == YearYoga.rudda || held.yoga == YearYoga.durapha,
          );
        }
        final pairYoga = matter.between?.yoga;
        expect(
          matter.holds(YearYoga.ithasala),
          pairYoga != null && pairYoga != TajikaYoga.ishrafa,
        );
      }
      final first = annual.matters[1];
      expect(first.sameLord, isTrue);
      expect(first.between, isNull);
      expect(
        first.held.every(
          (held) =>
              held.yoga == YearYoga.ikabala || held.yoga == YearYoga.induvara,
        ),
        isTrue,
      );
    }

    final every = years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        matters: Matters.all,
      ),
    );
    expect(every.first.annual!.matters.map((m) => m.house), [
      for (var house = 1; house <= 12; house += 1) house,
    ]);
    expect(
      years(
        const VarshaRequest(through: 2, place: AnnualPlace.birth),
      ).first.annual!.matters,
      isEmpty,
    );
    // The commentary's full Moon can only take a Kuttha away.
    int kutthas(Iterable<Pravesha> found) =>
        found
            .expand((one) => one.annual!.matters)
            .where((matter) => matter.holds(YearYoga.kuttha) ?? false)
            .length;
    final waxing = years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        matters: Matters.all,
        yogas: YogaRules(moonBenefic: MoonBenefic.waxing),
      ),
    );
    expect(kutthas(waxing), lessThanOrEqualTo(kutthas(every)));

    // The rule records cross in the boundary's casing.
    years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        varshesha: VarsheshaRules(
          noneAspects: NoneAspects.annualLagnaLord,
          tied: VarsheshaTied.dinaRatriPati,
          moon: MoonMayRule.ithasala,
          moonPartner: MoonPartner.officeBearer,
          subDegree: SubDegree.ishrafa,
        ),
      ),
    );
    years(
      const VarshaRequest(
        through: 2,
        place: AnnualPlace.birth,
        matters: Matters.houses([10]),
        yogas: YogaRules(
          subDegree: SubDegree.ishrafa,
          weakBelow: 4 * 3600,
          strongFrom: 12 * 3600,
          tambira: TambiraMover.eitherLord,
          moonBenefic: MoonBenefic.waxing,
        ),
      ),
    );

    Matcher refusedBy(String field) =>
        throwsA(isA<TeistroException>().having((e) => e.field, 'field', field));
    expect(
      () =>
          years(const VarshaRequest(through: 2, matters: Matters.houses([7]))),
      refusedBy('varsha.matters'),
    );
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          matters: Matters.houses([7, 7]),
        ),
      ),
      refusedBy('varsha.matters'),
    );
    expect(
      () => years(
        const VarshaRequest(
          through: 2,
          place: AnnualPlace.birth,
          matters: Matters.houses([7]),
          yogas: YogaRules(weakBelow: 12 * 3600, strongFrom: 4 * 3600),
        ),
      ),
      refusedBy('varsha.yogas.strongFrom'),
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
