/// The Teistro SDK for Dart and Flutter: the layer a consumer uses.
///
/// HAND-WRITTEN, and thin on purpose. Everything beneath it is generated
/// from the API description: the declarations and the value classes
/// (`src/ffi.dart`), the catalogue's enums (`src/catalogue.dart`) and the
/// result-blob decoders (`src/blob.dart`). What this file adds is what a
/// generator cannot know: where the shared library is, defaults, JSON in
/// and out, and the small conveniences a decoded result deserves.
///
/// ```dart
/// final teistro = Teistro.open();
/// final context = teistro.context(testProvider: true);
/// final sun = context.positions(
///   instants: [2451545.0],
///   bodies: [Body.sun],
/// ).at(0, 0);
/// print(sun.longitude);
/// context.dispose();
/// ```
library;

import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:io';
import 'dart:typed_data';

import 'src/blob.dart';
import 'src/catalogue.dart';
import 'src/ffi.dart';
import 'src/host.dart';
// Prefixed because the locale's names are its own: `Gender` is a word the
// catalogue uses too. A consumer that wants them imports
// `package:teistro/messages.dart`.
import 'src/messages.dart' as intl;

export 'src/blob.dart';
export 'src/catalogue.dart';
export 'src/ffi.dart';
export 'src/host.dart';

/// The SDK's shared library, opened once and shared by every context.
///
/// [open] finds it; [context] builds a context on it; the static calls of
/// the C ABI are its getters and methods, so nothing needs the generated
/// layer's [TeistroLibrary] unless you want it, and [library] hands that
/// over when you do.
final class Teistro {
  Teistro._(this.library, this.build);

  /// The generated declarations, for a call this layer does not wrap.
  final TeistroLibrary library;

  /// What the open library says about its own build.
  final BuildInfo build;

  /// The environment variable that names the shared library, which wins
  /// over every other place it is looked for.
  static const String pathVariable = 'TEISTRO_LIBRARY';

  /// Opens the shared library and checks that it is the build these
  /// declarations were generated from.
  ///
  /// `path` names the library outright. Without one, the SDK looks at
  /// `$TEISTRO_LIBRARY`, then beside this package, then in the workspace's
  /// `target/release` and `target/debug`, and finally asks the platform's
  /// loader for the bare name, which finds an installed library.
  ///
  /// Throws a [TeistroException] with `Status.unsupported` when the
  /// library is not that build ([refuseBuild] says why), and a
  /// [StateError] naming every place it looked when there is no library
  /// to open.
  factory Teistro.open({String? path}) {
    final opened = path == null ? _search() : ffi.DynamicLibrary.open(path);
    final library = TeistroLibrary(opened);
    rememberLibrary(library);
    final named = path != null || _named != null;
    final build = BuildInfo.of(buildInfo(library));
    final refusal = refuseBuild(build, named: named);
    if (refusal != null) {
      throw TeistroException(
        Status.unsupported,
        refusal,
        hint:
            'regenerate the binding with `cargo xtask gen ffi`, or build '
            'the library with `cargo build --release -p teistro-ffi`',
      );
    }
    return Teistro._(library, build);
  }

  /// The file name the platform gives the SDK's shared library.
  static String get libraryName {
    if (Platform.isMacOS) return 'libteistro_ffi.dylib';
    if (Platform.isWindows) return 'teistro_ffi.dll';
    return 'libteistro_ffi.so';
  }

  /// The library the environment names, when it names one.
  static String? get _named {
    final named = Platform.environment[pathVariable];
    return named == null || named.isEmpty ? null : named;
  }

  /// Every place [Teistro.open] looks, in order.
  static List<String> get searchPath {
    final here = File.fromUri(Platform.script).parent.path;
    final named = _named;
    return <String>[
      if (named != null) named,
      '$here/$libraryName',
      'bindings/dart/$libraryName',
      'target/release/$libraryName',
      'target/debug/$libraryName',
      '../../target/release/$libraryName',
      '../../target/debug/$libraryName',
    ];
  }

  static ffi.DynamicLibrary _search() {
    final looked = searchPath;
    for (final candidate in looked) {
      if (File(candidate).existsSync()) {
        return ffi.DynamicLibrary.open(candidate);
      }
    }
    try {
      return ffi.DynamicLibrary.open(libraryName);
    } on ArgumentError {
      throw StateError(
        'no Teistro library found. Looked in:\n  ${looked.join('\n  ')}\n'
        'Build it with `cargo build -p teistro-ffi`, or set '
        '\$$pathVariable to its path.',
      );
    }
  }

  /// The ABI the open library implements.
  int get abi => abiVersion(library);

  /// The SDK's version.
  String get version => sdkVersion(library);

  /// The catalogue's schema version, stamped in every result's provenance.
  int get catalogue => catalogueVersion(library);

  /// The profile a context uses when none is named.
  String get defaultProfileId => defaultProfile(library);

  /// The SDK's canonical frame: apparent geocentric ecliptic of date,
  /// tropical, which every chart module consumes.
  Frame get canonicalFrame => frameCanonical(library);

  /// Packs a frame's fields into the bits a position request carries.
  int packFrame(Frame frame) => framePack(library, frame);

  /// Reads packed frame bits back into their fields.
  Frame unpackFrame(int bits) => frameUnpack(library, bits);

  /// The Julian day at the UTC midnight that begins a fixed day.
  double julianDayOfFixed(int fixed) => calendarJdOfFixed(library, fixed);

  /// The fixed day a Julian day falls in, and the fraction of that day
  /// elapsed since its midnight.
  ({int value, double fraction}) fixedOfJulianDay(double jd) =>
      calendarFixedOfJd(library, jd);

  /// Builds a context: settings resolved from a profile and a patch, a
  /// locale, and an ephemeris. One context serves one thread; an isolate
  /// builds its own.
  ///
  /// `profile` names a shipped profile ([defaultProfileId] by default),
  /// `settings` is a patch over it as the settings document's own groups
  /// and knobs, `locale` is what every render resolves from, `provider`
  /// is an ephemeris of your own, and `testProvider` selects the SDK's
  /// analytic provider, which is for examples and tests only. With
  /// neither the context has no ephemeris and positions answer
  /// `Status.capability`.
  Context context({
    String? profile,
    Map<String, Object?>? settings,
    String? locale,
    EphemerisProvider? provider,
    bool testProvider = false,
  }) {
    final host = provider == null ? null : HostProvider(library, provider);
    try {
      return Context._(
        this,
        TeistroContext(
          library,
          options: ContextOptions(
            flags: testProvider && host == null ? contextTestProvider : 0,
            profile: profile,
            settingsJson: settings == null ? null : jsonEncode(settings),
            locale: locale,
          ),
          provider: host?.vtable,
          providerUserData: host?.userData,
        ),
        host,
      );
    } on Object {
      host?.dispose();
      rethrow;
    }
  }
}

/// A context: settings, a locale and an ephemeris, with the calls that use
/// them. Built by [Teistro.context].
///
/// The native context is freed when this object is collected; [dispose]
/// frees it at once, and every call after that is a [StateError].
final class Context {
  Context._(this._teistro, this._inner, this._host) {
    final host = _host;
    if (host != null) _hostFinaliser.attach(this, host, detach: this);
  }

  final Teistro _teistro;
  final TeistroContext _inner;
  final HostProvider? _host;

  /// The library this context was built on.
  Teistro get teistro => _teistro;

  /// The generated context, for a call this layer does not wrap.
  TeistroContext get inner => _inner;

  /// The id of the profile the settings came from.
  String get profile => _inner.profile();

  /// The resolved settings, as their canonical document.
  Map<String, Object?> get settings =>
      jsonDecode(settingsJson) as Map<String, Object?>;

  /// The same document as the text the library wrote, which is what the
  /// settings hash is taken over and what a stored chart keeps.
  String get settingsJson => _inner.settingsJson();

  /// The SHA-256 of the canonical settings, in hex; every result carries
  /// it, and two runs that agree on it are comparable.
  String get settingsHash => _hex(_inner.settingsHash().bytes);

  /// The locale every render resolves from.
  String get locale => _inner.intlLocale();

  set locale(String tag) => _inner.intlSetLocale(tag);

  /// Positions over a grid of instants and bodies, completed into the
  /// frame asked for; the canonical frame by default.
  ///
  /// The result's cells are instants outermost: cell `i * bodies.length +
  /// b` is body `b` at instant `i`, which [Positions.at] reads for you.
  Positions positions({
    required List<double> instants,
    required List<Body> bodies,
    TimeScale scale = TimeScale.ut1,
    Frame? frame,
    bool speeds = true,
    Observer? observer,
  }) {
    if (instants.isEmpty) {
      throw ArgumentError.value(instants, 'instants', 'expected an instant');
    }
    if (bodies.isEmpty) {
      throw ArgumentError.value(bodies, 'bodies', 'expected a body');
    }
    final bits = _teistro.packFrame(frame ?? _teistro.canonicalFrame);
    return decodePositions(
      _guarded(
        () => _inner.positions(
          PositionRequest(
            scale: scale,
            frameBits: bits,
            speeds: speeds,
            observer: observer,
            jds: instants,
            bodies: bodies,
          ),
        ),
      ),
    );
  }

  /// Runs a call that may reach a provider written in Dart, and reports
  /// what the provider said. Only a code crosses the C boundary, so the
  /// failure the provider itself raised is the one kept.
  T _guarded<T>(T Function() call) {
    _host?.thrown = null;
    try {
      return call();
    } on TeistroException catch (failure) {
      final thrown = _host?.thrown;
      if (thrown == null) rethrow;
      _host?.thrown = null;
      throw TeistroException(
        failure.status,
        'the ephemeris provider failed: $thrown',
        detail: failure.detail,
        field: failure.field,
        hint: failure.hint,
        messageKey: failure.messageKey,
        providerCode: failure.providerCode,
      );
    }
  }

  /// Renders a message of the current locale with its parameters.
  IntlRender render(String key, [Map<String, Object?>? params]) =>
      decodeIntlRender(
        _inner.intlRender(key, params == null ? null : jsonEncode(params)),
      );

  /// Whether the current locale or its fallbacks have a message.
  bool has(String key) => _inner.intlHas(key) == 1;

  /// Text from one script into another (`deva`, `iast`), for a Sanskrit
  /// or Nepali term written in the other.
  String transliterate(
    String text, {
    String from = 'deva',
    String to = 'iast',
  }) => _inner.intlTransliterate(text, from, to);

  /// An entity's forms in the current locale or its fallbacks: its name,
  /// its prose form, its transliteration, and the glyph and gender the
  /// locale gives it.
  intl.EntityForms entity(String key) =>
      intl.EntityForms.of(_inner.intlEntity(key));

  /// The typed accessors: every message of the SDK's own locale as a
  /// function of its parameters, and every catalogued entity as its
  /// forms. A key is spelled once, by the generator, and never by an
  /// application.
  ///
  /// ```dart
  /// ctx.messages.sdk.reason.grahaInBhava(
  ///   graha: GrahaKey.jupiter,
  ///   bhava: 7,
  /// );
  /// ctx.messages.sdk.entity.graha.sun.name;
  /// ```
  ///
  /// The types are `package:teistro/messages.dart`.
  intl.Messages get messages => _messages ??= intl.Messages(_Renderer(this));

  intl.Messages? _messages;

  /// Loads a `.tpack` or `.tbundle` file into the locale engine.
  IntlLoaded loadPack(Uint8List bytes) => _inner.intlLoadPack(bytes);

  /// The date a fixed day falls on in a calendar.
  CalendarDate dateOf(Calendar calendar, int fixed) =>
      _inner.calendarFromFixed(calendar, fixed);

  /// The fixed day of a date.
  int fixedOf(CalendarDate date) => _inner.calendarToFixed(date);

  /// The same date in another calendar.
  CalendarDate convert(CalendarDate date, Calendar into) =>
      _inner.calendarConvert(date, into);

  /// The weekday of a date, Monday `1` to Sunday `7`.
  int weekdayOf(CalendarDate date) => _inner.calendarWeekday(date);

  /// The length of a month.
  int monthLength(Calendar calendar, int year, int month) =>
      _inner.calendarMonthLength(calendar, year, month);

  /// Whether a year is a leap year.
  bool isLeap(Calendar calendar, int year) =>
      _inner.calendarIsLeap(calendar, year) == 1;

  /// A civil date and time in a zone, resolved to an instant with what the
  /// resolution had to decide.
  ZoneResolution resolve(CivilDateTime civil, ZoneSpec zone) =>
      _inner.timeResolve(civil, zone);

  /// The civil date and time of an instant in a zone.
  ({CivilDateTime civil, ZoneResolution resolution}) civilOf(
    double jdUtc,
    ZoneSpec zone,
    Calendar calendar,
  ) => _inner.timeCivil(jdUtc, zone, calendar);

  /// Converts an instant between the time scales.
  TimeConversion convertTime(double jd, Scale from, Scale to) =>
      _inner.timeConvert(jd, from, to);

  /// Delta T at a UT1 instant, with what produced it.
  DeltaT deltaT(double jdUt1) => _inner.timeDeltaT(jdUt1);

  /// The packed id of a catalogue key (`graha.SUN`, an alias, or a former
  /// key).
  int keyId(String key) => _inner.keyParse(key);

  /// The catalogue key of a packed id.
  String keyName(int id) => _inner.keyName(id);

  /// The provider written in Dart this context drives, null when it has
  /// none or uses the SDK's own.
  EphemerisProvider? get provider => _host?.provider;

  /// Frees the native context now rather than when this object is
  /// collected, and with it the vtable of a provider written in Dart.
  /// Calling it twice is harmless.
  void dispose() {
    _hostFinaliser.detach(this);
    _inner.dispose();
    _host?.dispose();
  }
}

/// What a library says about its own build: the SDK version, the ABI and
/// catalogue versions, the commit it came from and whether that tree was
/// clean, the profile, the target, whether it is optimised, the sanitizer
/// if any, and the compiler.
///
/// The two halves of a binding must be one build: the library carries the
/// SDK, and these declarations were generated from a description of it.
/// [Teistro.open] reads this and refuses a library that is not that
/// build.
final class BuildInfo {
  const BuildInfo({
    required this.sdk,
    required this.abi,
    required this.catalogue,
    required this.commit,
    required this.dirty,
    required this.profile,
    required this.target,
    required this.debugAssertions,
    required this.optimised,
    required this.sanitizer,
    required this.rustc,
  });

  /// Reads the document `ts_build_info` hands out.
  factory BuildInfo.of(String json) {
    final Map<String, Object?> document;
    try {
      document = jsonDecode(json) as Map<String, Object?>;
    } on FormatException catch (error) {
      throw StateError('the library did not describe its build: $error');
    }
    String text(String key) => '${document[key] ?? ''}';
    int number(String key) => (document[key] as num?)?.toInt() ?? 0;
    bool flag(String key) => document[key] == true;
    return BuildInfo(
      sdk: text('sdk'),
      abi: number('abi'),
      catalogue: number('catalogue'),
      commit: text('commit'),
      dirty: flag('dirty'),
      profile: text('profile'),
      target: text('target'),
      debugAssertions: flag('debug_assertions'),
      optimised: flag('optimised'),
      sanitizer: text('sanitizer'),
      rustc: text('rustc'),
    );
  }

  /// The SDK's version.
  final String sdk;

  /// The ABI the library implements.
  final int abi;

  /// The catalogue's schema version.
  final int catalogue;

  /// The commit it was built from, `unknown` outside a checkout.
  final String commit;

  /// Whether that tree had uncommitted changes.
  final bool dirty;

  /// The Cargo profile it was built with.
  final String profile;

  /// The target triple it was built for.
  final String target;

  /// Whether debug assertions are on.
  final bool debugAssertions;

  /// Whether it was optimised.
  final bool optimised;

  /// The sanitizer it carries, empty for none.
  final String sanitizer;

  /// The compiler that built it.
  final String rustc;

  @override
  String toString() =>
      'Teistro $sdk (ABI $abi) $profile for $target, commit '
      '${commit.length > 8 ? commit.substring(0, 8) : commit}'
      '${dirty ? '-dirty' : ''}';
}

/// Why a build may not be loaded, or null when it may.
///
/// A mismatched ABI or version is refused outright: the two halves of a
/// binding must be one build. A sanitizer build is refused because it
/// answers differently and slowly and is never chosen by accident. An
/// unoptimised one is refused only when the loader found it itself,
/// because naming a path is a deliberate act and a development build is
/// what a developer means by it.
String? refuseBuild(BuildInfo info, {required bool named}) {
  if (info.abi != generatedAbiVersion) {
    return 'the library implements ABI ${info.abi}, these declarations '
        'were generated for ABI $generatedAbiVersion';
  }
  if (info.sdk != generatedSdkVersion) {
    return 'the library is Teistro ${info.sdk}, these declarations were '
        'generated from $generatedSdkVersion';
  }
  if (info.sanitizer.isNotEmpty) {
    return 'the library is a ${info.sanitizer} sanitizer build, which is '
        'not for use';
  }
  if (!named && !info.optimised) {
    return 'the library found at a searched path is an unoptimised '
        '${info.profile} build; build it with `--release`, or set '
        '\$${Teistro.pathVariable} to load this one deliberately';
  }
  return null;
}

/// A context as the generated accessors read it: text for a message, and
/// the forms for an entity.
final class _Renderer implements intl.Renderer {
  const _Renderer(this._context);

  final Context _context;

  @override
  String render(String key, [Map<String, Object?> params = const {}]) =>
      _context.render(key, params.isEmpty ? null : params).text;

  @override
  intl.EntityForms entity(String key) => _context.entity(key);
}

/// Closes the callbacks of a provider written in Dart when the context
/// that drove it is collected, so a context nobody disposed leaks
/// nothing. The native context is freed by its own finaliser, and
/// neither call reaches the other.
final Finalizer<HostProvider> _hostFinaliser = Finalizer(
  (host) => host.dispose(),
);

/// A date in a calendar, without naming the fields a call fills in.
///
/// ```dart
/// final date = Calendar.gregorian.date(2015, 4, 14);
/// ```
extension CalendarDates on Calendar {
  /// A date in this calendar. `era` and the era year are what the call
  /// resolves them to, and the resolution is [Resolution.defined], which
  /// is what a date a caller states means.
  CalendarDate date(int year, int month, int day) => CalendarDate(
    calendar: this,
    year: year,
    eraYear: 0,
    month: month,
    day: day,
    resolution: Resolution.defined,
    computedMonth: 0,
    computedDay: 0,
  );
}

/// A date with a time of day, or with none.
extension CivilDateTimes on CalendarDate {
  /// This date at a time of day.
  ///
  /// ```dart
  /// final birth = Calendar.gregorian.date(1986, 1, 1).at(hour: 0, minute: 20);
  /// ```
  CivilDateTime at({
    int hour = 0,
    int minute = 0,
    int second = 0,
    int nanos = 0,
  }) => CivilDateTime(
    date: this,
    time: CivilTime(
      hour: hour,
      minute: minute,
      second: second,
      hasTime: true,
      nanos: nanos,
    ),
  );

  /// This date with the time of day unknown, which a resolution reports
  /// rather than guesses.
  CivilDateTime get whenUnknown => CivilDateTime(
    date: this,
    time: const CivilTime(
      hour: 0,
      minute: 0,
      second: 0,
      hasTime: false,
      nanos: 0,
    ),
  );
}

/// A zone of the embedded database, by its IANA name
/// (`Asia/Kathmandu`).
ZoneSpec ianaZone(String name) => ZoneSpec(
  kind: ZoneKind.iana,
  offsetSeconds: 0,
  longitudeDeg: Longitude(0),
  zone: name,
);

/// A fixed offset from UTC, in seconds east.
ZoneSpec fixedZone(int offsetSeconds) => ZoneSpec(
  kind: ZoneKind.fixed,
  offsetSeconds: offsetSeconds,
  longitudeDeg: Longitude(0),
);

/// Local mean time at a longitude east of Greenwich, which is what a
/// chart from before the zone existed is cast in.
ZoneSpec localMeanZone(Longitude longitudeDeg) => ZoneSpec(
  kind: ZoneKind.localMean,
  offsetSeconds: 0,
  longitudeDeg: longitudeDeg,
);

/// One cell of a position grid.
typedef Cell =
    ({
      double longitude,
      double latitude,
      double distance,
      double longitudeSpeed,
      double latitudeSpeed,
      double distanceSpeed,
      int status,
      int source,
    });

/// What a decoded position grid means, beyond the columns themselves.
extension PositionsResult on Positions {
  /// The instants of the request, in order.
  Float64List get jds => instants.jd;

  /// The bodies of the request, in order.
  List<Body> get bodyKeys =>
      List<Body>.generate(bodies.length, (i) => Body.byId(bodies.body[i]));

  /// The time scale the instants are on.
  TimeScale get timeScale => TimeScale.byId(scale);

  /// The frame the positions are in.
  Frame frame(Teistro teistro) => teistro.unpackFrame(frameBits);

  /// The completion steps the SDK applied, in order.
  List<Object?> get stepsApplied => jsonDecode(steps) as List<Object?>;

  /// Everything that reproduces this result.
  Map<String, Object?> get provenanceOf =>
      jsonDecode(provenance) as Map<String, Object?>;

  /// One cell of the grid, by the indices of its instant and its body.
  Cell at(int instant, int body) {
    if (instant < 0 || instant >= jdCount || body < 0 || body >= bodyCount) {
      throw RangeError(
        'at($instant, $body): the grid is $jdCount by $bodyCount',
      );
    }
    final i = instant * bodyCount + body;
    return (
      longitude: cells.lon[i],
      latitude: cells.lat[i],
      distance: cells.dist[i],
      longitudeSpeed: cells.lonSpeed[i],
      latitudeSpeed: cells.latSpeed[i],
      distanceSpeed: cells.distSpeed[i],
      status: cells.status[i],
      source: cells.source[i],
    );
  }
}

/// What a rendered message means, beyond its text.
extension RenderedMessage on IntlRender {
  /// Whether a fallback locale answered.
  bool get fallback => isFallback != 0;

  /// Whether a runtime override answered.
  bool get override => isOverride != 0;

  /// The locale whose message answered, null when none had it.
  String? get from => resolvedFrom.isEmpty ? null : resolvedFrom;

  /// Every problem met; rendering continues past each.
  List<String> get warningList =>
      (jsonDecode(warnings) as List<Object?>).cast<String>();
}

/// A digest as the hex every binding prints.
String _hex(Uint8List bytes) =>
    bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
