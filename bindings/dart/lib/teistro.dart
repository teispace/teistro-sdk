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
import 'src/install.dart';
// Prefixed because the locale's names are its own: `Gender` is a word the
// catalogue uses too. A consumer that wants them imports
// `package:teistro/messages.dart`.
import 'src/messages.dart' as intl;

export 'src/blob.dart';
export 'src/catalogue.dart';
export 'src/ffi.dart';
export 'src/host.dart';
export 'src/install.dart'
    show hostPlatform, install, installedLibrary, InstallException, Installed;
// The version this package expects its library to be, which is the
// release its installer fetches from.
export 'src/prebuilt.dart' show prebuiltVersion;

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
  ///
  /// The installer needs the same name and cannot import this library, so
  /// the name is defined beside it and read back here.
  static String get libraryName => libraryFileName;

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
      // What `dart run teistro:install` fetched for this project, which a
      // consumer has and a contributor does not.
      installedLibrary(),
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
  /// is an ephemeris of your own, and `ephemeris` names one of the SDK's:
  /// `Ephemeris.builtin` is the analytic ephemeris the SDK carries, which
  /// needs no files, no network and no licence beyond the SDK's own, and
  /// is what lets a chart compute with nothing else installed;
  /// `Ephemeris.test` is the test provider, whose positions are **not
  /// astronomy**. `testProvider` is the older spelling of the latter and
  /// still works, with `ephemeris` winning when both are given
  /// (ADR-0028). With none of them the context has no ephemeris and
  /// positions answer `Status.capability`.
  /// A context: settings, a locale and an ephemeris.
  ///
  /// `ephemeris` is an **ordered chain**, tried in order (ADR-0029). An
  /// entry is an adapter's own descriptor — [PluginEphemeris], what a
  /// package like `teistro_ephemeris_teimeris` exports, carrying the
  /// platform binary it ships and its own configuration — or one of the
  /// SDK's own by name, [NamedEphemeris].
  ///
  /// **A list even for one**, which Dart needs because it has no
  /// untagged union, and which says the thing the ADR wants said: a
  /// chain is a caller *saying* they will accept the fallback. A context
  /// asked for an engine and given the built-in without being told is
  /// the silence this refuses.
  Context context({
    String? profile,
    Map<String, Object?>? settings,
    String? locale,
    EphemerisProvider? provider,
    List<EphemerisChoice>? ephemeris,
    bool testProvider = false,
  }) {
    // Two ways to answer one question, so both together is a refusal
    // rather than one silently winning.
    if (provider != null && ephemeris != null) {
      throw ArgumentError(
        'provider and ephemeris each name the ephemeris to compute with; '
        'give one of them',
      );
    }
    if (ephemeris != null && ephemeris.isEmpty) {
      throw ArgumentError(
        'an ephemeris chain of none names nothing; give an entry or omit it',
      );
    }
    final host = provider == null ? null : HostProvider(library, provider);
    // One rule, written once: a named ephemeris wins, and the older flag
    // decides only when none was named (ADR-0028).
    final chain =
        ephemeris ??
        <EphemerisChoice>[
          NamedEphemeris(
            testProvider && host == null ? Ephemeris.test : Ephemeris.none,
          ),
        ];
    try {
      // A chain of one is not a chain, so nothing is caught for it.
      // Found by an existing test: a bad *profile* is not an ephemeris
      // failure, and catching it to try the next entry replaced a
      // refusal carrying its status, its field and its hint with a bare
      // "nothing could be opened". With one entry there is no next
      // entry, so the refusal is the refusal.
      if (chain.length == 1) {
        return Context._(
          this,
          _open(chain.first, profile, settings, locale, host),
          host,
        );
      }
      // With more than one, every refusal is kept and reported together,
      // because a chain that said only why its last entry failed would
      // hide the one the caller actually wanted.
      final refusals = <String>[];
      for (final entry in chain) {
        try {
          return Context._(
            this,
            _open(entry, profile, settings, locale, host),
            host,
          );
        } on Object catch (refusal) {
          refusals.add('${_names(entry)}: $refusal');
        }
      }
      throw StateError(
        'no ephemeris in the chain could be opened:\n  ${refusals.join('\n  ')}',
      );
    } on Object {
      host?.dispose();
      rethrow;
    }
  }

  /// What one entry of the chain is called, for a refusal that names it.
  static String _names(EphemerisChoice entry) => switch (entry) {
    NamedEphemeris(:final name) => name.key,
    PluginEphemeris(:final plugin) => plugin,
  };

  /// Opens the context on one entry of the chain.
  TeistroContext _open(
    EphemerisChoice entry,
    String? profile,
    Map<String, Object?>? settings,
    String? locale,
    HostProvider? host,
  ) {
    ContextOptions options(Ephemeris named) => ContextOptions(
      flags: 0,
      ephemeris: named,
      profile: profile,
      settingsJson: settings == null ? null : jsonEncode(settings),
      locale: locale,
    );
    switch (entry) {
      case NamedEphemeris(:final name):
        return TeistroContext(
          library,
          options: options(name),
          provider: host?.vtable,
          providerUserData: host?.userData,
        );
      case PluginEphemeris(:final plugin, :final config):
        // The context takes its own reference to the adapter, so the
        // handle this loads is disposed at once: what keeps the library
        // loaded is the context, and a consumer holds neither.
        final loaded = TeistroProvider(
          library,
          path: plugin,
          configJson: jsonEncode(config ?? const <String, Object?>{}),
        );
        try {
          return TeistroContext.newWithProvider(
            library,
            options: options(Ephemeris.none),
            provider: loaded,
          );
        } finally {
          loaded.dispose();
        }
    }
  }
}

/// One entry of an ephemeris chain (ADR-0029).
///
/// Sealed, so the two kinds are the two kinds: the switch that opens one
/// is exhaustive and a third would not compile until it was handled.
sealed class EphemerisChoice {
  const EphemerisChoice();
}

/// One of the SDK's own ephemerides, by name.
///
/// `Ephemeris.builtin` is the analytic ephemeris the SDK carries, which
/// needs no files, no network and no licence beyond the SDK's own;
/// `Ephemeris.test` is the test provider, whose positions are **not
/// astronomy**.
final class NamedEphemeris extends EphemerisChoice {
  const NamedEphemeris(this.name);

  /// Which of the SDK's own.
  final Ephemeris name;
}

/// An adapter's descriptor: the platform binary its package ships, and
/// that adapter's own configuration.
///
/// What the configuration means is the adapter's to say and its
/// package's to type; the SDK hands it over as JSON and reads none of
/// it.
final class PluginEphemeris extends EphemerisChoice {
  const PluginEphemeris({required this.plugin, this.config});

  /// The adapter's platform binary.
  final String plugin;

  /// That adapter's own options.
  final Map<String, Object?>? config;
}

/// The engine's own operations, reached by the names it gives them.
///
/// The SDK names eight operations. An engine names far more, and what it
/// names beyond them is reached through here rather than around the SDK.
/// **Nothing in this class is a list of an engine's operations**: it asks
/// the engine what it offers and calls what the answer names, so a
/// function the engine gains after this package ships is callable without
/// a new release of it.
///
/// ```dart
/// final engine = context.ephemeris;
/// final answer = engine('tp_echo', {'value': 6.0});
/// ```
///
/// The operations are called by name rather than reached as members.
/// Dart spells its members in camel case and an engine spells its
/// functions as C does, so a member proxy would have to guess at the
/// mapping between `tm_eclipse_when` and `tmEclipseWhen` — and a guess
/// that is wrong for one engine's spelling would make that operation
/// unreachable, which is the dead end this whole route exists to close.
/// The engine's own spelling is what its manifest says and what its
/// documentation calls it, so that is what this takes. The Node and
/// Python bindings reach them as members because in those languages the
/// engine's own spelling *is* the idiomatic one.
final class Engine {
  Engine._(this._context);

  final Context _context;
  Map<String, Object?>? _manifest;

  /// The manifest as the engine wrote it.
  String get manifestJson => _context._inner.ephemerisManifest();

  /// The manifest, parsed and remembered.
  ///
  /// Read once per engine: it changes when the engine does, and an engine
  /// does not change under a live context.
  Map<String, Object?> get manifest =>
      _manifest ??= jsonDecode(manifestJson) as Map<String, Object?>;

  /// Every operation the engine offers, in its own order.
  List<String> get names => [
    for (final function in (manifest['functions'] as List<Object?>? ?? []))
      (function as Map<String, Object?>)['name'] as String,
  ];

  /// What the manifest says about one operation, or `null`.
  ///
  /// Its parameters carry the role of each, which says which of them a
  /// caller supplies and which the engine fills.
  Map<String, Object?>? signature(String name) {
    for (final function in (manifest['functions'] as List<Object?>? ?? [])) {
      final candidate = function as Map<String, Object?>;
      if (candidate['name'] == name) return candidate;
    }
    return null;
  }

  /// Whether the engine names an operation.
  bool has(String name) => names.contains(name);

  /// Calls an operation by name, with its parameters keyed by the names
  /// the manifest gives, answering with the engine's own answer, parsed.
  Object? call(String name, [Map<String, Object?> arguments = const {}]) =>
      jsonDecode(callJson(name, jsonEncode(arguments)));

  /// Calls an operation with arguments already written as JSON, and
  /// answers with the engine's own JSON: the form to use when the answer
  /// is being handed on rather than read.
  String callJson(String name, String argumentsJson) =>
      _context._inner.ephemerisCall(name, argumentsJson);

  @override
  String toString() =>
      'Engine(${manifest['engine']} ${manifest['version']}, ${names.length} operations)';
}

/// What every area is.
///
/// A **value**: one object per context, built on first read of the field
/// that holds it and kept, so a consumer may hold it and pass it. That is
/// what makes the areas worth having rather than merely tidy
/// (`03-design/surface-areas.md`).
///
/// Dart has extension methods and they were rejected for this: an
/// extension resolves statically and cannot be held or passed, and the
/// requirement is that an area is a value.
///
/// Each holds the context and nothing else. The context's own members are
/// library-private, so an area reaches the boundary through it rather
/// than keeping a handle of its own.
sealed class _Area {
  const _Area(this._context);

  final Context _context;
}

/// `sdk.calendar` — the calendars, and the fixed day they share.
final class CalendarArea extends _Area {
  const CalendarArea._(super.context);

  /// The date a fixed day falls on in a calendar.
  CalendarDate dateOf(Calendar calendar, int fixed) =>
      _context._inner.calendarFromFixed(calendar, fixed);

  /// The fixed day of a date.
  int fixedOf(CalendarDate date) => _context._inner.calendarToFixed(date);

  /// The same date in another calendar.
  CalendarDate convert(CalendarDate date, Calendar into) =>
      _context._inner.calendarConvert(date, into);

  /// The weekday of a date, Monday `1` to Sunday `7`.
  int weekdayOf(CalendarDate date) => _context._inner.calendarWeekday(date);

  /// The length of a month.
  int monthLength(Calendar calendar, int year, int month) =>
      _context._inner.calendarMonthLength(calendar, year, month);

  /// Whether a year is a leap year.
  bool isLeap(Calendar calendar, int year) =>
      _context._inner.calendarIsLeap(calendar, year) == 1;
}

/// `sdk.time` — the scales, the zones and what separates them.
final class TimeArea extends _Area {
  const TimeArea._(super.context);

  /// A civil date and time in a zone, resolved to an instant with what the
  /// resolution had to decide.
  ZoneResolution resolve(CivilDateTime civil, ZoneSpec zone) =>
      _context._inner.timeResolve(civil, zone);

  /// The civil date and time of an instant in a zone.
  ({CivilDateTime civil, ZoneResolution resolution}) civilOf(
    double jdUtc,
    ZoneSpec zone,
    Calendar calendar,
  ) => _context._inner.timeCivil(jdUtc, zone, calendar);

  /// Converts an instant between the time scales.
  TimeConversion convert(double jd, Scale from, Scale to) =>
      _context._inner.timeConvert(jd, from, to);

  /// Delta T at a UT1 instant, with what produced it.
  DeltaT deltaT(double jdUt1) => _context._inner.timeDeltaT(jdUt1);
}

/// `sdk.intl` — the locale, its messages and the scripts they are in.
final class IntlArea extends _Area {
  IntlArea._(super.context);

  /// The locale every render resolves from.
  String get locale => _context._inner.intlLocale();

  set locale(String tag) => _context._inner.intlSetLocale(tag);

  /// Renders a message of the current locale with its parameters.
  IntlRender render(String key, [Map<String, Object?>? params]) =>
      decodeIntlRender(
        _context._inner.intlRender(
          key,
          params == null ? null : jsonEncode(params),
        ),
      );

  /// Whether the current locale or its fallbacks have a message.
  bool has(String key) => _context._inner.intlHas(key) == 1;

  /// Text from one script into another (`deva`, `iast`), for a Sanskrit
  /// or Nepali term written in the other.
  String transliterate(
    String text, {
    String from = 'deva',
    String to = 'iast',
  }) => _context._inner.intlTransliterate(text, from, to);

  /// An entity's forms in the current locale or its fallbacks: its name,
  /// its prose form, its transliteration, and the glyph and gender the
  /// locale gives it.
  intl.EntityForms entity(String key) =>
      intl.EntityForms.of(_context._inner.intlEntity(key));

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
  intl.Messages get messages =>
      _messages ??= intl.Messages(_Renderer(_context));

  intl.Messages? _messages;

  /// Loads a `.tpack` or `.tbundle` file into the locale engine.
  IntlLoaded loadPack(Uint8List bytes) => _context._inner.intlLoadPack(bytes);
}

/// `sdk.keys` — the catalogue's keys and their packed ids.
final class KeysArea extends _Area {
  const KeysArea._(super.context);

  /// The packed id of a catalogue key (`graha.SUN`, an alias, or a former
  /// key).
  int id(String key) => _context._inner.keyParse(key);

  /// The catalogue key of a packed id.
  String name(int id) => _context._inner.keyName(id);
}

/// `sdk.frame` — the coordinate conventions a request is expressed in.
final class FrameArea extends _Area {
  const FrameArea._(super.context);

  /// The SDK's canonical frame: apparent geocentric ecliptic of date,
  /// tropical.
  Frame get canonical => _context._teistro.canonicalFrame;

  /// Packs a frame's fields into the bits a position request carries.
  int pack(Frame frame) => _context._teistro.packFrame(frame);

  /// The frame a packed set of bits describes.
  Frame unpack(int bits) => _context._teistro.unpackFrame(bits);
}

/// `sdk.chart` — a chart founded at an instant and a place.
final class ChartArea extends _Area {
  const ChartArea._(super.context);

  /// Founds a chart at an instant and a place.
  ///
  /// Everything but this is the context's settings, so two charts
  /// founded under one context are comparable and the settings hash says
  /// why. The clock is here because nothing else knows it: a chart's day
  /// runs from a local sunrise and its date is a civil date, and a
  /// longitude gives local *mean* time rather than a civil offset.
  ///
  /// A profile whose frame is topocentric needs a provider that answers
  /// topocentric natively; the completion's centre step is Phase 3's
  /// (`03-design/chart-at-the-boundary.md` §8).
  Chart found({
    required double instant,
    required Observer place,
    required int utcOffsetSeconds,
    ChartKind kind = ChartKind.natal,
  }) => foundMany(
    instants: <double>[instant],
    place: place,
    utcOffsetSeconds: utcOffsetSeconds,
    kind: kind,
  ).at(0);

  /// Founds a chart at each of many instants, at one place, in one
  /// crossing.
  ///
  /// The founder shares the settings and the solar model across the
  /// batch, so a hundred instants cost one setup rather than a hundred —
  /// which is what a rectification pass wants. A batch of none is an
  /// empty result rather than an error.
  Charts foundMany({
    required List<double> instants,
    required Observer place,
    required int utcOffsetSeconds,
    ChartKind kind = ChartKind.natal,
  }) => decodeCharts(
    _context._guarded(
      () => _context._inner.chartFound(
        ChartRequest(
          kind: kind,
          instants: instants,
          latitudeDeg: place.latitudeDeg,
          longitudeDeg: place.longitudeDeg,
          altitudeM: place.altitudeM,
          utcOffsetSeconds: utcOffsetSeconds,
        ),
      ),
    ),
  );
}

/// `sdk.almanac` — a day, or a run of days, with its limbs.
///
/// The boundary calls this `panchanga`; the area takes the consumer's
/// word, because an almanac is what the operation answers and a panchanga
/// is one tradition's name for five of its limbs
/// (`03-design/surface-areas.md`).
final class AlmanacArea extends _Area {
  const AlmanacArea._(super.context);

  /// The almanac of every day in a range, at one place.
  ///
  /// A **range** rather than a list of dates, because consecutive days
  /// share a boundary — day *n*'s next sunrise is day *n+1*'s sunrise —
  /// so a month of days costs much less than thirty days computed
  /// separately. A range holding more than a year and a day is refused
  /// by name.
  Almanac of({
    required CalendarDate from,
    required CalendarDate to,
    required Observer place,
    required int utcOffsetSeconds,
  }) => Almanac(
    decodePanchanga(
      _context._guarded(
        () => _context._inner.panchangaDays(
          PanchangaRequest(
            calendar: from.calendar,
            fromYear: from.year,
            fromMonth: from.month,
            fromDay: from.day,
            toYear: to.year,
            toMonth: to.month,
            toDay: to.day,
            latitudeDeg: place.latitudeDeg,
            longitudeDeg: place.longitudeDeg,
            altitudeM: place.altitudeM,
            utcOffsetSeconds: utcOffsetSeconds,
          ),
        ),
      ),
    ),
  );

  /// The almanac of one day, which is the range of one unwrapped.
  AlmanacDay day({
    required CalendarDate date,
    required Observer place,
    required int utcOffsetSeconds,
  }) => of(
    from: date,
    to: date,
    place: place,
    utcOffsetSeconds: utcOffsetSeconds,
  ).at(0);
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

  /// The engine's own operations, beyond the eight the SDK names.
  ///
  /// **Not `ephemeris`**: `engine` says *this particular engine, not the
  /// portable contract*, so a consumer reading their own code sees the
  /// difference between a call that survives changing provider and one
  /// that does not (ADR-0030).
  ///
  /// Throws when the context has no ephemeris, or when the one it has
  /// describes nothing of its own — asked now rather than at the first
  /// call, so a caller learns it where they can act on it.
  Engine get engine {
    final engine = _engine ??= Engine._(this);
    engine.manifestJson;
    return engine;
  }

  Engine? _engine;

  /// The calendars, and the fixed day they share.
  ///
  /// An **area**: one object per context, built on first read and kept, so
  /// a consumer may hold it (`final calendar = sdk.calendar`). A field
  /// rather than a getter, because a getter that rebuilt on every read
  /// would make holding one a lie
  /// (`03-design/surface-areas.md`).
  late final CalendarArea calendar = CalendarArea._(this);

  /// The scales, the zones and what separates them.
  late final TimeArea time = TimeArea._(this);

  /// The locale, its messages and the scripts they are in.
  late final IntlArea intl = IntlArea._(this);

  /// The catalogue's keys and their packed ids.
  late final KeysArea keys = KeysArea._(this);

  /// The coordinate conventions a request is expressed in.
  late final FrameArea frame = FrameArea._(this);

  /// A chart founded at an instant and a place.
  late final ChartArea chart = ChartArea._(this);

  /// A day, or a run of days, with its limbs.
  late final AlmanacArea almanac = AlmanacArea._(this);

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

  /// Runs a call that may reach a provider written in Dart, and rethrows
  /// what the provider itself threw.
  ///
  /// Only a code crosses the C boundary, so without this the provider's
  /// own sentence would be lost and the caller would see the port's
  /// summary of it instead. The original object is rethrown rather than
  /// wrapped, so a caller catches the type it wrote — which is what the
  /// Node and Python bindings do as well.
  T _guarded<T>(T Function() call) {
    _host?.thrown = null;
    try {
      return call();
    } on TeistroException {
      final thrown = _host?.thrown;
      if (thrown == null) rethrow;
      _host?.thrown = null;
      throw thrown;
    }
  }

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
      _context.intl.render(key, params.isEmpty ? null : params).text;

  @override
  intl.EntityForms entity(String key) => _context.intl.entity(key);
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

  /// This date with its time of day unknown.
  ///
  /// Nothing guesses one. Unless the profile sets `time.unknown_time`, a
  /// resolution refuses it by name and the hint says what to choose;
  /// under `NOON` it resolves with [ZoneResolution.timeKnown] false and a
  /// `time-unknown-fallback` warning, and under `SUNRISE` it needs the
  /// place and a solar model.
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

/// Where a graha sits in a set of bhavas.
final class Placement {
  const Placement({
    required this.bhava,
    required this.method,
    required this.through,
    required this.fromMadhyaDeg,
  });

  /// The bhava, 1 to 12.
  final int bhava;

  /// The house system that produced it.
  final HouseSystem method;

  /// How far through the bhava it is, 0 to 1.
  final double through;

  /// Its distance from the bhava's centre, degrees.
  final double fromMadhyaDeg;
}

/// One graha of a chart, read out of the batch's columns.
final class PlacedGraha {
  const PlacedGraha({
    required this.graha,
    required this.longitudeDeg,
    required this.tropicalDeg,
    required this.latitudeDeg,
    required this.distanceAu,
    required this.speedDegPerDay,
    required this.house,
    required this.placement,
  });

  /// Which graha.
  final Graha graha;

  /// Its longitude in the chart's zodiac, degrees.
  final double longitudeDeg;

  /// Its tropical longitude, degrees.
  final double tropicalDeg;

  /// Its ecliptic latitude, degrees.
  final double latitudeDeg;

  /// Its distance, astronomical units.
  final double distanceAu;

  /// Its longitude speed, degrees per day.
  final double speedDegPerDay;

  /// Whether that speed is negative.
  bool get retrograde => speedDegPerDay < 0;

  /// The bhava for "which house is it in".
  final Placement house;

  /// The bhava of the chart's chalit, which is a different question and
  /// often a different answer.
  final Placement placement;
}

/// One of the twelve bhavas.
final class Bhava {
  const Bhava({required this.madhyaDeg, required this.sandhiDeg});

  /// The bhava's centre, degrees.
  final double madhyaDeg;

  /// The bhava's opening cusp, degrees.
  final double sandhiDeg;
}

/// One founded chart: a view over its batch, not a copy.
///
/// Every getter reads the batch's columns at this chart's index, so a
/// chart costs nothing until something is asked of it and holding one
/// holds the whole blob rather than a copy of a slice of it.
final class Chart {
  const Chart(this.batch, this.index);

  /// The batch this chart belongs to.
  final Charts batch;

  /// Where in that batch it sits.
  final int index;

  /// The instant the chart is cast for, as a Julian day (UTC).
  double get instant => batch.cast.instant[index];

  /// The lagna at the instant, in the chart's zodiac, degrees.
  double get lagnaDeg => batch.cast.lagnaDeg[index];

  /// The lagna at the sunrise that opened the day, degrees.
  double get dayLagnaDeg => batch.cast.dayLagnaDeg[index];

  /// The ayanamsha applied at this instant, degrees; zero if tropical.
  double get ayanamshaOffsetDeg => batch.cast.ayanamshaOffsetDeg[index];

  /// Which arc of its day the instant falls in.
  ///
  /// This and [dayElapsed] belong to the **instant**, not to the day, so
  /// they are the chart's rather than the day section's — which is what
  /// the panchanga blob's arrival settled.
  DayPart get dayPart => DayPart.byId(batch.cast.dayPart[index]);

  /// How far through that arc the instant is, 0 to 1.
  double get dayElapsed => batch.cast.dayElapsed[index];

  /// What kind of chart this is.
  ChartKind get kind => ChartKind.byId(batch.kind);

  /// The weekday the chart's day carries.
  Vara get vara => Vara.byId(batch.day.vara[index]);

  /// The sunrise that opened the chart's day, as a Julian day (UTC).
  double get sunrise => batch.day.sunrise[index];

  /// The graha that rules the hora holding the instant.
  Graha get horaLord => Graha.byId(batch.timing.horaLord[index]);

  /// The grahas, in the catalogue's order, one object each.
  ///
  /// The columns underneath are views over the blob's bytes, charts
  /// outermost; this reads this chart's stride out of them into the
  /// shape an application wants, which is a row.
  List<PlacedGraha> get grahas {
    final g = batch.grahas;
    final base = index * batch.grahaCount;
    return List<PlacedGraha>.generate(batch.grahaCount, (j) {
      final i = base + j;
      return PlacedGraha(
        graha: Graha.byId(g.graha[i]),
        longitudeDeg: g.longitudeDeg[i],
        tropicalDeg: g.tropicalDeg[i],
        latitudeDeg: g.latitudeDeg[i],
        distanceAu: g.distanceAu[i],
        speedDegPerDay: g.speedDegPerDay[i],
        house: Placement(
          bhava: g.houseBhava[i],
          method: HouseSystem.byId(g.houseMethod[i]),
          through: g.houseThrough[i],
          fromMadhyaDeg: g.houseFromMadhyaDeg[i],
        ),
        placement: Placement(
          bhava: g.placementBhava[i],
          method: HouseSystem.byId(g.placementMethod[i]),
          through: g.placementThrough[i],
          fromMadhyaDeg: g.placementFromMadhyaDeg[i],
        ),
      );
    });
  }

  /// The twelve bhavas for "which house is it in", first to twelfth.
  List<Bhava> get houses =>
      _bhavas(batch.houses.madhyaDeg, batch.houses.sandhiDeg);

  /// The twelve bhavas of the chart's chalit.
  List<Bhava> get chalit =>
      _bhavas(batch.chalit.madhyaDeg, batch.chalit.sandhiDeg);

  List<Bhava> _bhavas(Float64List madhya, Float64List sandhi) {
    final base = index * 12;
    return List<Bhava>.generate(
      12,
      (j) => Bhava(madhyaDeg: madhya[base + j], sandhiDeg: sandhi[base + j]),
    );
  }
}

/// Reading a batch of founded charts one chart at a time.
extension ChartsByIndex on Charts {
  /// One chart of the batch, by index.
  Chart at(int index) {
    if (index < 0 || index >= chartCount) {
      throw RangeError.index(index, this, 'index', null, chartCount);
    }
    return Chart(this, index);
  }

  /// The completion steps the SDK applied, in order, each
  /// `name:Implementation`.
  ///
  /// The blob carries them as JSON text, as it carries the provenance
  /// envelope; this is the parsed form the Node and Python bindings
  /// hand back.
  List<String> get stepsApplied =>
      (jsonDecode(steps) as List<dynamic>).cast<String>();

  /// Every chart, in the order the instants were asked for.
  Iterable<Chart> get each sync* {
    for (var i = 0; i < chartCount; i += 1) {
      yield Chart(this, i);
    }
  }
}

/// A span of time, as every almanac row carries one.
final class Interval {
  const Interval({required this.from, required this.to});

  /// When it begins, as a Julian day (UTC).
  final double from;

  /// When it ends.
  final double to;
}

/// One member of a limb, with its own bounds and the clipped ones.
final class Span<T> {
  const Span({required this.member, required this.whole, required this.inside});

  /// Which member ran.
  final T member;

  /// When the member itself began and ended, inside the day or not.
  final Interval whole;

  /// The part inside the day: what an almanac row prints.
  final Interval inside;
}

/// The lunar month a day falls in, under both conventions.
final class Month {
  const Month({
    required this.month,
    required this.amanta,
    required this.purnimanta,
    required this.paksha,
    required this.convention,
    required this.kind,
  });

  /// The month under the profile's own convention.
  final Masa month;

  /// The amanta month: new moon to new moon.
  final Masa amanta;

  /// The purnimanta month: full moon to full moon.
  final Masa purnimanta;

  /// Which fortnight the day opens in.
  final Paksha paksha;

  /// Which convention [month] leads with.
  final LunarMonth convention;

  /// Whether the month is ordinary, intercalary or omitted.
  ///
  /// The name above needs no case for the intercalary one — an adhika
  /// month and the nija month after it take the same name — so this is
  /// the mark beside the name.
  final MonthKind kind;
}

/// One inauspicious eighth of the daylight.
final class KaalaPeriod {
  const KaalaPeriod({required this.kaala, required this.at});

  /// Which one.
  final Kaala kaala;

  /// When it runs.
  final Interval at;
}

/// One choghadiya, of the daylight or of the night.
final class ChoghadiyaPeriod {
  const ChoghadiyaPeriod({
    required this.choghadiya,
    required this.lord,
    required this.at,
    required this.daytime,
  });

  /// Which choghadiya.
  final Choghadiya choghadiya;

  /// The graha that rules it.
  final Graha lord;

  /// When it runs.
  final Interval at;

  /// Whether it is one of the eight of the daylight.
  final bool daytime;
}

/// One hora, from sunrise.
final class Hora {
  const Hora({
    required this.number,
    required this.lord,
    required this.start,
    required this.end,
  });

  /// Its number, 1 to 24.
  final int number;

  /// The graha that rules it.
  final Graha lord;

  /// When it begins, as a Julian day (UTC).
  final double start;

  /// When it ends.
  final double end;
}

/// One of the thirty muhurtas.
final class Muhurta {
  const Muhurta({required this.at, required this.daylight});

  /// When it runs.
  final Interval at;

  /// Whether it is one of the fifteen of the daylight.
  final bool daylight;
}

/// A moonrise or a moonset.
final class MoonEvent {
  const MoonEvent({required this.rise, required this.instant});

  /// True for a rise, false for a set.
  final bool rise;

  /// When, as a Julian day (UTC).
  final double instant;
}

/// A muhurta yoga that held, and what made it hold.
final class HeldYoga {
  const HeldYoga({
    required this.yoga,
    required this.at,
    required this.vara,
    required this.tithi,
    required this.nakshatra,
  });

  /// Which yoga.
  final MuhurtaYoga yoga;

  /// While it held, clipped to the day.
  final Interval at;

  /// The vara that makes it; every cause has one.
  final Vara vara;

  /// The tithi that makes it, or `null` when the cause has none.
  final Tithi? tithi;

  /// The nakshatra that makes it.
  final Nakshatra nakshatra;
}

/// Abhijit, with whether it is effective.
final class Abhijit {
  const Abhijit({required this.at, required this.effective});

  /// When it runs.
  final Interval at;

  /// True on every day but a Wednesday.
  final bool effective;
}

/// A batch of daily panchangas at one place.
///
/// Every per-day list is concatenated across the batch, so a day's rows
/// are found by adding up every earlier day's count. That sum is done
/// **once**, when the batch is built, rather than per access: the
/// alternative is quadratic over a year of days, which is the shape an
/// almanac is actually asked for.
final class Almanac {
  Almanac(this.decoded) : _starts = _prefixSums(decoded.counts);

  /// The blob as its generated decoder read it.
  final Panchanga decoded;
  final Map<String, Uint32List> _starts;

  /// How many days the batch holds.
  int get length => decoded.dayCount;

  /// The place they were all founded at.
  Observer get place => Observer(
    latitudeDeg: Latitude(decoded.latitudeDeg),
    longitudeDeg: Longitude(decoded.longitudeDeg),
    altitudeM: Altitude(decoded.altitudeM),
  );

  /// The civil calendar the days' dates are read in.
  Calendar get calendar => Calendar.byId(decoded.calendar);

  /// The solar model that reckoned the days, as it describes itself.
  String get model => decoded.model;

  /// One day of the batch, by index.
  AlmanacDay at(int index) {
    if (index < 0 || index >= length) {
      throw RangeError.index(index, this, 'index', null, length);
    }
    return AlmanacDay(this, index);
  }

  /// Every day, in the order the range runs.
  Iterable<AlmanacDay> get each sync* {
    for (var i = 0; i < length; i += 1) {
      yield AlmanacDay(this, i);
    }
  }

  /// Where day [index]'s rows of a per-day list begin and end.
  (int, int) range(String list, int index) {
    final starts = _starts[list] ?? Uint32List(length + 1);
    return (starts[index], starts[index + 1]);
  }

  static Map<String, Uint32List> _prefixSums(PanchangaCounts counts) {
    final columns = <String, List<int>>{
      'tithi': counts.tithi,
      'nakshatra': counts.nakshatra,
      'yoga': counts.yoga,
      'karana': counts.karana,
      'panchaka': counts.panchaka,
      'moonSigns': counts.moonSigns,
      'sunSigns': counts.sunSigns,
      'kaalas': counts.kaalas,
      'choghadiya': counts.choghadiya,
      'horas': counts.horas,
      'muhurtas': counts.muhurtas,
      'moonEvents': counts.moonEvents,
      'muhurtaYogas': counts.muhurtaYogas,
    };
    return columns.map((name, column) {
      final starts = Uint32List(column.length + 1);
      for (var i = 0; i < column.length; i += 1) {
        starts[i + 1] = starts[i] + column[i];
      }
      return MapEntry(name, starts);
    });
  }
}

/// One day of an almanac: a view over its batch, not a copy.
final class AlmanacDay {
  const AlmanacDay(this.batch, this.index);

  /// The batch this day belongs to.
  final Almanac batch;

  /// Where in that batch it sits.
  final int index;

  /// The weekday the day carries.
  Vara get vara => Vara.byId(batch.decoded.day.vara[index]);

  /// The sunrise that opened the day, as a Julian day (UTC).
  double get sunrise => batch.decoded.day.sunrise[index];

  /// The sunset that closed its daylight.
  double get sunset => batch.decoded.day.sunset[index];

  /// What the spans are clipped to.
  Interval get window => Interval(
    from: batch.decoded.days.windowFrom[index],
    to: batch.decoded.days.windowTo[index],
  );

  /// The lunar month, under both conventions.
  Month get month {
    final d = batch.decoded.days;
    return Month(
      month: Masa.byId(d.month[index]),
      amanta: Masa.byId(d.amanta[index]),
      purnimanta: Masa.byId(d.purnimanta[index]),
      paksha: Paksha.byId(d.paksha[index]),
      convention: LunarMonth.byId(batch.decoded.lunarMonth),
      kind: MonthKind.byId(d.monthKind[index]),
    );
  }

  /// Which half of the year the day falls in.
  Ayana get ayana => Ayana.byId(batch.decoded.days.ayana[index]);

  /// The direction not to travel in, which is the vara's.
  Direction get dishaShool =>
      Direction.byId(batch.decoded.days.dishaShool[index]);

  /// When the Sun entered a new sign inside the day, or `null`.
  double? get sankranti =>
      batch.decoded.days.hasSankranti[index] == 1
          ? batch.decoded.days.sankranti[index]
          : null;

  /// Abhijit; `null` on a day with no daylight.
  Abhijit? get abhijit {
    final d = batch.decoded.days;
    if (d.hasAbhijit[index] != 1) return null;
    return Abhijit(
      at: Interval(from: d.abhijitFrom[index], to: d.abhijitTo[index]),
      effective: d.abhijitEffective[index] == 1,
    );
  }

  /// Brahma muhurta; `null` when the night before is not known.
  Interval? get brahma {
    final d = batch.decoded.days;
    return d.hasBrahma[index] == 1
        ? Interval(from: d.brahmaFrom[index], to: d.brahmaTo[index])
        : null;
  }

  /// The tithis that touch the day.
  List<Span<Tithi>> get tithi {
    final c = batch.decoded.tithi;
    return _spans(
      'tithi',
      c.member,
      c.wholeFrom,
      c.wholeTo,
      c.insideFrom,
      c.insideTo,
      Tithi.byId,
    );
  }

  /// The nakshatras the Moon was in.
  List<Span<Nakshatra>> get nakshatra {
    final c = batch.decoded.nakshatra;
    return _spans(
      'nakshatra',
      c.member,
      c.wholeFrom,
      c.wholeTo,
      c.insideFrom,
      c.insideTo,
      Nakshatra.byId,
    );
  }

  /// The nitya yogas.
  List<Span<Yoga>> get yoga {
    final c = batch.decoded.yoga;
    return _spans(
      'yoga',
      c.member,
      c.wholeFrom,
      c.wholeTo,
      c.insideFrom,
      c.insideTo,
      Yoga.byId,
    );
  }

  /// The karanas: half-tithis, so three or four on an ordinary day.
  List<Span<Karana>> get karana {
    final c = batch.decoded.karana;
    return _spans(
      'karana',
      c.member,
      c.wholeFrom,
      c.wholeTo,
      c.insideFrom,
      c.insideTo,
      Karana.byId,
    );
  }

  /// Panchaka, while the Moon is in the last five nakshatras.
  List<Span<Panchaka>> get panchaka {
    final c = batch.decoded.panchaka;
    return _spans(
      'panchaka',
      c.member,
      c.wholeFrom,
      c.wholeTo,
      c.insideFrom,
      c.insideTo,
      Panchaka.byId,
    );
  }

  /// The signs the Moon stood in.
  List<Span<Rashi>> get moonSigns {
    final c = batch.decoded.moonSigns;
    return _spans(
      'moonSigns',
      c.member,
      c.wholeFrom,
      c.wholeTo,
      c.insideFrom,
      c.insideTo,
      Rashi.byId,
    );
  }

  /// The signs the Sun stood in; two only on a sankranti day.
  List<Span<Rashi>> get sunSigns {
    final c = batch.decoded.sunSigns;
    return _spans(
      'sunSigns',
      c.member,
      c.wholeFrom,
      c.wholeTo,
      c.insideFrom,
      c.insideTo,
      Rashi.byId,
    );
  }

  /// The inauspicious eighths of the daylight.
  List<KaalaPeriod> get kaalas {
    final c = batch.decoded.kaalas;
    return _rows(
      'kaalas',
      (i) => KaalaPeriod(
        kaala: Kaala.byId(c.kaala[i]),
        at: Interval(from: c.from[i], to: c.to[i]),
      ),
    );
  }

  /// Eight choghadiya of the daylight and eight of the night.
  List<ChoghadiyaPeriod> get choghadiya {
    final c = batch.decoded.choghadiya;
    return _rows(
      'choghadiya',
      (i) => ChoghadiyaPeriod(
        choghadiya: Choghadiya.byId(c.choghadiya[i]),
        lord: Graha.byId(c.lord[i]),
        at: Interval(from: c.from[i], to: c.to[i]),
        daytime: c.daytime[i] == 1,
      ),
    );
  }

  /// The twenty-four horas, from sunrise.
  List<Hora> get horas {
    final c = batch.decoded.horas;
    return _rows(
      'horas',
      (i) => Hora(
        number: c.number[i],
        lord: Graha.byId(c.lord[i]),
        start: c.start[i],
        end: c.end[i],
      ),
    );
  }

  /// The thirty muhurtas: fifteen of the daylight, then fifteen of the night.
  List<Muhurta> get muhurtas {
    final c = batch.decoded.muhurtas;
    return _rows(
      'muhurtas',
      (i) => Muhurta(
        at: Interval(from: c.from[i], to: c.to[i]),
        daylight: c.daylight[i] == 1,
      ),
    );
  }

  /// Every moonrise and moonset inside the day's moon window.
  List<MoonEvent> get moonEvents {
    final c = batch.decoded.moonEvents;
    return _rows(
      'moonEvents',
      (i) => MoonEvent(rise: c.kind[i] == 0, instant: c.instant[i]),
    );
  }

  /// The muhurta yogas that held, with what made each hold.
  List<HeldYoga> get muhurtaYogas {
    final c = batch.decoded.muhurtaYogas;
    return _rows(
      'muhurtaYogas',
      (i) => HeldYoga(
        yoga: MuhurtaYoga.byId(c.yoga[i]),
        at: Interval(from: c.from[i], to: c.to[i]),
        vara: Vara.byId(c.becauseVara[i]),
        // A `VARA_NAKSHATRA` cause has no tithi, and the blob leaves the
        // column at nought rather than at a tithi that did not make it.
        tithi: c.becauseKind[i] == 0 ? null : Tithi.byId(c.becauseTithi[i]),
        nakshatra: Nakshatra.byId(c.becauseNakshatra[i]),
      ),
    );
  }

  List<T> _rows<T>(String list, T Function(int) build) {
    final (from, to) = batch.range(list, index);
    return List<T>.generate(to - from, (k) => build(from + k));
  }

  List<Span<T>> _spans<T>(
    String list,
    Uint16List member,
    Float64List wholeFrom,
    Float64List wholeTo,
    Float64List insideFrom,
    Float64List insideTo,
    T Function(int) byId,
  ) => _rows(
    list,
    (i) => Span<T>(
      member: byId(member[i]),
      whole: Interval(from: wholeFrom[i], to: wholeTo[i]),
      inside: Interval(from: insideFrom[i], to: insideTo[i]),
    ),
  );
}
