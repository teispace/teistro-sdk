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
/// final context = teistro.context(
///   ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
/// );
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
import 'src/records.dart';

export 'src/blob.dart';
export 'src/catalogue.dart';
export 'src/ffi.dart';
export 'src/records.dart';
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
    List<LayoutRow> layouts = const <LayoutRow>[],
    List<DashaDefinition> dashaSystems = const <DashaDefinition>[],
  }) {
    // Serialised once, however many entries of the chain are tried.
    final layoutsJson =
        layouts.isEmpty
            ? null
            : jsonEncode([for (final row in layouts) row.toJson()]);
    final dashasJson =
        dashaSystems.isEmpty
            ? null
            : jsonEncode([for (final system in dashaSystems) system.toJson()]);
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
          _open(
            chain.first,
            profile,
            settings,
            locale,
            layoutsJson,
            dashasJson,
            host,
          ),
          host,
          layouts,
          dashaSystems,
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
            _open(
              entry,
              profile,
              settings,
              locale,
              layoutsJson,
              dashasJson,
              host,
            ),
            host,
            layouts,
            dashaSystems,
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
    String? layoutsJson,
    String? dashasJson,
    HostProvider? host,
  ) {
    ContextOptions options(Ephemeris named) => ContextOptions(
      flags: 0,
      ephemeris: named,
      profile: profile,
      settingsJson: settings == null ? null : jsonEncode(settings),
      locale: locale,
      layoutsJson: layoutsJson,
      dashasJson: dashasJson,
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

  /// A layout this context can draw in, shipped or registered, as its row:
  /// copy it with [LayoutRow.copyWith], give it a key of its own, and pass
  /// it to `Teistro.context(layouts: ...)` (`03-design/chart-geometry.md`
  /// §7f).
  LayoutRow layout(KeyOf<ChartLayout> layout) => LayoutRow.fromJson(
    jsonDecode(
          _context._guarded(
            () => _context._inner.chartLayoutRow(layout.fullKey),
          ),
        )
        as Map<String, Object?>,
  );

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
    List<Varga> vargas = const <Varga>[],
    List<KeyOf<DashaSystem>> dashas = const <KeyOf<DashaSystem>>[],
    List<(KeyOf<ChartLayout>, Varga)> drawings =
        const <(KeyOf<ChartLayout>, Varga)>[],
    ChartTheme? theme,
    RuleRequest? rules,
    PlanRequest? interpret,
    VarshaRequest? varsha,
    GocharRequest? gochar,
    HitRequest? hits,
    SadeSatiRequest? sadeSati,
    KpRequest? kp,
    DignityRequest? dignities,
    FortitudeRequest? fortitudes,
    LotRequest? lots,
    ConsiderationRules? considerations,
    PerfectionRequest? perfection,
    ProgressionsRequest? progressions,
    WesternAspectRequest? westernAspects,
    SynastryRequest? synastry,
    ParallelRequest? parallels,
    AntisciaRequest? antiscia,
    MidpointRequest? midpoints,
    WesternHouseRequest? westernHouses,
    HarmonicRequest? harmonic,
    MatchingRequest? matching,
    bool aspects = false,
    bool points = false,
    bool houses = false,
    bool ashtakavarga = false,
    bool vimshopaka = false,
    bool vaiseshikamsa = false,
    bool dashaPhala = false,
    bool jaimini = false,
    bool avakahada = false,
    bool outerPlanets = false,
    bool shadbala = false,
    bool bhavaBala = false,
    bool state = false,
  }) => foundMany(
    instants: <double>[instant],
    place: place,
    utcOffsetSeconds: utcOffsetSeconds,
    kind: kind,
    vargas: vargas,
    dashas: dashas,
    drawings: drawings,
    theme: theme,
    rules: rules,
    interpret: interpret,
    varsha: varsha,
    gochar: gochar,
    hits: hits,
    sadeSati: sadeSati,
    kp: kp,
    dignities: dignities,
    fortitudes: fortitudes,
    lots: lots,
    considerations: considerations,
    perfection: perfection,
    progressions: progressions,
    westernAspects: westernAspects,
    synastry: synastry,
    parallels: parallels,
    antiscia: antiscia,
    midpoints: midpoints,
    westernHouses: westernHouses,
    harmonic: harmonic,
    matching: matching,
    aspects: aspects,
    points: points,
    houses: houses,
    ashtakavarga: ashtakavarga,
    vimshopaka: vimshopaka,
    vaiseshikamsa: vaiseshikamsa,
    dashaPhala: dashaPhala,
    jaimini: jaimini,
    avakahada: avakahada,
    outerPlanets: outerPlanets,
    shadbala: shadbala,
    bhavaBala: bhavaBala,
    state: state,
  ).at(0);

  /// Founds a chart at each of many instants, at one place, in one
  /// crossing.
  ///
  /// The founder shares the settings and the solar model across the
  /// batch, so a hundred instants cost one setup rather than a hundred —
  /// which is what a rectification pass wants. A batch of none is an
  /// empty result rather than an error.
  /// `vargas` names the divisional charts to compute, in the order to
  /// answer them; none by default, because a caller who wants a birth
  /// chart should not pay for twenty-one of them
  /// (`03-design/chart-reading.md` §4). `drawings` names charts to draw, each
  /// a `(ChartLayout, Varga)` pair with `Varga.d1` the founded chart, in the
  /// order to answer them. `theme` writes each drawing as SVG in the
  /// context's locale, read back as `Drawing.svg`; none by default.
  /// `dashas` names the dasha systems to compute, their periods to the
  /// settings' `dasha.depth` (`03-design/dasha-kernels.md`).
  /// `aspects` asks for the drishti.
  Charts foundMany({
    required List<double> instants,
    required Observer place,
    required int utcOffsetSeconds,
    ChartKind kind = ChartKind.natal,
    List<Varga> vargas = const <Varga>[],
    List<KeyOf<DashaSystem>> dashas = const <KeyOf<DashaSystem>>[],
    List<(KeyOf<ChartLayout>, Varga)> drawings =
        const <(KeyOf<ChartLayout>, Varga)>[],
    ChartTheme? theme,
    RuleRequest? rules,
    PlanRequest? interpret,
    VarshaRequest? varsha,
    GocharRequest? gochar,
    HitRequest? hits,
    SadeSatiRequest? sadeSati,
    KpRequest? kp,
    DignityRequest? dignities,
    FortitudeRequest? fortitudes,
    LotRequest? lots,
    ConsiderationRules? considerations,
    PerfectionRequest? perfection,
    ProgressionsRequest? progressions,
    WesternAspectRequest? westernAspects,
    SynastryRequest? synastry,
    ParallelRequest? parallels,
    AntisciaRequest? antiscia,
    MidpointRequest? midpoints,
    WesternHouseRequest? westernHouses,
    HarmonicRequest? harmonic,
    MatchingRequest? matching,
    bool aspects = false,
    bool points = false,
    bool houses = false,
    bool ashtakavarga = false,
    bool vimshopaka = false,
    bool vaiseshikamsa = false,
    bool dashaPhala = false,
    bool jaimini = false,
    bool avakahada = false,
    bool outerPlanets = false,
    bool shadbala = false,
    bool bhavaBala = false,
    bool state = false,
  }) => _named(
    decodeCharts(
      _context._guarded(
        () => _context._inner.chartFound(
          ChartRequest(
            kind: kind,
            instants: instants,
            latitudeDeg: place.latitudeDeg,
            longitudeDeg: place.longitudeDeg,
            altitudeM: place.altitudeM,
            utcOffsetSeconds: utcOffsetSeconds,
            // The sections beside the foundation, which the SDK takes as
            // a bit set and nothing here writes as one
            // (`03-design/chart-reading.md` §5): a named argument each,
            // and one more as each crosses.
            sections:
                (aspects ? chartAspects : 0) |
                (points ? chartPoints : 0) |
                (houses ? chartHouses : 0) |
                (ashtakavarga ? chartAshtakavarga : 0) |
                (vimshopaka ? chartVimshopaka : 0) |
                (vaiseshikamsa ? chartVaiseshikamsa : 0) |
                (dashaPhala ? chartDashaPhala : 0) |
                (jaimini ? chartJaimini : 0) |
                (avakahada ? chartAvakahada : 0) |
                (outerPlanets ? chartOuter : 0) |
                (shadbala ? chartShadbala : 0) |
                (bhavaBala ? chartBhavaBala : 0) |
                (state ? chartState : 0),
            vargas: vargas,
            dashas: _dashaIds(dashas, _context._registeredDashas),
            drawings: _drawingBits(drawings, _context._registeredLayouts),
            themeJson: theme?._json,
            rulesJson: rules?._json,
            interpretJson: interpret?._json,
            varshaJson: varsha?._json,
            gocharJson: gochar?._json,
            hitsJson: hits?._json,
            sadeSatiJson: sadeSati?._json,
            kpJson: kp?._json,
            dignitiesJson: dignities?._json,
            fortitudesJson: fortitudes?._json,
            lotsJson: lots?._json,
            considerationsJson: considerations?._json,
            perfectionJson: perfection?._json,
            progressionsJson: progressions?._json,
            westernAspectsJson: westernAspects?._json,
            synastryJson: synastry?._json,
            parallelsJson: parallels?._json,
            antisciaJson: antiscia?._json,
            midpointsJson: midpoints?._json,
            westernHousesJson: westernHouses?._json,
            harmonicJson: harmonic?._json,
            matchingJson: matching?._json,
          ),
        ),
      ),
    ),
    _context._registeredDashas,
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
  ///
  /// [muhurta] runs a search over the same days, answered as
  /// [Almanac.muhurta], and [festivals] the rules whose days fall in them,
  /// answered as [Almanac.festivals]; the days are founded once for both.
  /// [years] answers the lunar years the days fall in, as [Almanac.years],
  /// [eclipses] the eclipses whose greatest moment falls in them with how
  /// the place sees each, as [Almanac.eclipses], and [nepalSambat] each
  /// day's Nepal Sambat date, as [Almanac.nepalSambat].
  Almanac of({
    required CalendarDate from,
    required CalendarDate to,
    required Observer place,
    required int utcOffsetSeconds,
    MuhurtaRequest? muhurta,
    FestivalRequest? festivals,
    bool years = false,
    bool eclipses = false,
    bool nepalSambat = false,
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
            muhurtaJson: muhurta?._json,
            festivalsJson: festivals?._json,
            sections:
                (years ? panchangaYears : 0) |
                (eclipses ? panchangaEclipses : 0) |
                (nepalSambat ? panchangaNepalSambat : 0),
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

/// `sdk.matching` — what matches without a chart: two names, star to star
/// (`03-design/matching.md`, C291 to C296). A match of two births is asked
/// of the charts, through [MatchingRequest] beside a chart request.
final class MatchingArea extends _Area {
  const MatchingArea._(super.context);

  /// Two names matched star to star (naam milan): each name's first
  /// syllable in the śatapada cakra, the varga koota of *Muhurta
  /// Chintamani* VI.35, and the Ashta Koota and the ten considerations
  /// read from the two name stars, as a chart's match reads two Moons.
  ///
  /// A name is read in Devanagari, or in IAST when [NameRules.latin] is
  /// [LatinName.iast]; an English spelling is refused rather than guessed.
  /// A name in Abhijit's row is refused unless [NameRules.abhijit] places
  /// it. Each refusal is named, `naam.groom.abhijit` and the like.
  ///
  /// ```dart
  /// final read = sdk.matching.naam('सीता', 'राम');
  /// print('${read.bride.nakshatra} ${read.varga.relation} ${read.ashta.total}');
  /// ```
  NaamMilan naam(
    String bride,
    String groom, [
    NaamRules rules = const NaamRules(),
  ]) {
    final d = decodeNaam(
      _context._inner.naamMilan(
        jsonEncode(<String, Object?>{
          'bride': bride,
          'groom': groom,
          'rules': rules._record,
        }),
      ),
    );
    final n = d.naamNames;
    NameSyllable name(int at) => NameSyllable(
      cell: n.cell[at],
      nakshatra: n.abhijit[at] == 1 ? null : Nakshatra.byId(n.nakshatra[at]),
      quarter: n.quarter[at],
      varga: NameVarga.byId(n.varga[at]),
    );
    final (brideName, groomName) = (name(0), name(1));
    return NaamMilan(
      bride: brideName,
      groom: groomName,
      varga: VargaKoota(
        bride: brideName.varga,
        groom: groomName.varga,
        relation: VargaRelation.byId(d.relation),
      ),
      ashta: _matchingsIn(d.matchings, d.matchingKootas, 1).single,
      porutham: _poruthamsIn(d.poruthams, d.poruthamRows, 1).single,
    );
  }
}

/// A context: settings, a locale and an ephemeris, with the calls that use
/// them. Built by [Teistro.context].
///
/// The native context is freed when this object is collected; [dispose]
/// frees it at once, and every call after that is a [StateError].
final class Context {
  Context._(
    this._teistro,
    this._inner,
    this._host,
    List<LayoutRow> layouts,
    List<DashaDefinition> dashaSystems,
  )
    // The member id of each layout and dasha system this context registered,
    // by its full key: asked once, here, so a request resolves a consumer's
    // own without crossing the boundary again (`03-design/chart-geometry.md`
    // §7f).
    : _registeredLayouts = {
        for (final row in layouts)
          'chart_layout.${row.key}':
              _inner.keyParse('chart_layout.${row.key}') & 0xFFFF,
      },
      _registeredDashas = {
        for (final system in dashaSystems)
          'dasha_system.${system.key}':
              _inner.keyParse('dasha_system.${system.key}') & 0xFFFF,
      } {
    final host = _host;
    if (host != null) _hostFinaliser.attach(this, host, detach: this);
  }

  final Map<String, int> _registeredLayouts;
  final Map<String, int> _registeredDashas;

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

  /// What matches without a chart: two names, star to star.
  late final MatchingArea matching = MatchingArea._(this);

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
  /// `TIME_UNKNOWN_FALLBACK` warning, and under `SUNRISE` it needs the
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
  List<Step> get stepsApplied => [
    for (final step in jsonDecode(steps) as List<Object?>)
      Step.fromJson(step! as Map<String, Object?>),
  ];

  /// Everything that reproduces this result: what computed it, and under
  /// what.
  Provenance get provenance => _provenanceOf(this, provenanceJson);

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

/// One part of a rendered message: its text, or a markup tag standing in
/// the text.
///
/// MF2 markup (`{#b}…{/b}`) is how a message says that part of it is a
/// link, a name or emphasis, **without saying what that looks like** —
/// the message stays free of markup languages and the renderer decides.
/// A Flutter renderer walks the parts and builds a `TextSpan` per tag.
final class MessagePart {
  const MessagePart.text(this.value)
    : isText = true,
      kind = '',
      name = '',
      options = const {};

  const MessagePart.markup({
    required this.kind,
    required this.name,
    required this.options,
  }) : isText = false,
       value = '';

  /// Whether this is text rather than a tag.
  final bool isText;

  /// The text, already formatted and localised; empty for a tag.
  final String value;

  /// `open`, `close` or `standalone`; empty for text.
  final String kind;

  /// The tag's name, as the message wrote it: `b`, `link`, …
  final String name;

  /// Its options, each already resolved to a string.
  final Map<String, String> options;

  static MessagePart _of(Map<String, Object?> written) =>
      written['type'] == 'text'
          ? MessagePart.text(written['value']! as String)
          : MessagePart.markup(
            kind: written['kind']! as String,
            name: written['name']! as String,
            options: (written['options']! as Map<String, Object?>).map(
              (name, value) => MapEntry(name, value! as String),
            ),
          );

  @override
  String toString() => isText ? value : '<$kind $name>';
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

  /// The message in parts, its markup kept: what a rich renderer walks.
  ///
  /// Joining the text parts gives exactly [text], so a renderer that does
  /// not know a tag can ignore it and lose nothing. The boundary sends
  /// nothing when the message has no markup, because the parts would then
  /// be the text written twice; the one part is made here rather than
  /// carried.
  List<MessagePart> get partList {
    final written =
        (jsonDecode(parts.isEmpty ? '[]' : parts) as List<Object?>)
            .cast<Map<String, Object?>>();
    return written.isEmpty
        ? [MessagePart.text(text)]
        : written.map(MessagePart._of).toList();
  }
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
/// How near a body stands to a boundary, which is what an ayanamsha
/// that moved would change.
final class EdgeDistance {
  const EdgeDistance({
    required this.signDeg,
    required this.nakshatraDeg,
    required this.padaDeg,
  });

  /// To the nearer edge of its sign, degrees.
  final double signDeg;

  /// To the nearer edge of its nakshatra, degrees.
  final double nakshatraDeg;

  /// To the nearer edge of its pada, degrees.
  final double padaDeg;
}

/// What the Sun does to a body.
final class Combustion {
  const Combustion({
    required this.burning,
    required this.fromSunDeg,
    required this.orbDeg,
    required this.deepOrbDeg,
  });

  /// How badly it burns.
  final Burning burning;

  /// How far from the Sun it stands, degrees, or null when the chart
  /// carries no Sun — in which case nothing is burnt and this says why
  /// rather than claiming the sky is clear.
  final double? fromSunDeg;

  /// Combust inside this, degrees, or null for a body that does not burn.
  final double? orbDeg;

  /// Deeply combust inside this, degrees, where the table gives one.
  final double? deepOrbDeg;
}

/// How a body stands to its dispositor, three ways.
final class Friendship {
  const Friendship({
    required this.natural,
    required this.temporary,
    required this.compound,
    required this.dispositor,
  });

  /// The table's own reading.
  final Relationship natural;

  /// Where the dispositor stands.
  final Relationship temporary;

  /// The five-fold compound of the two.
  final Relationship compound;

  /// The lord of the sign, which all three are with; null only for a
  /// body the catalogue gives no sign.
  final Graha? dispositor;
}

/// The lajjitadi a body holds, is ruled out of, and nothing decides.
final class Lajjitadi {
  const Lajjitadi({
    required this.holding,
    required this.ruledOut,
    required this.undecided,
  });

  /// The states that hold.
  final List<AvasthaLajjitadi> holding;

  /// The states that certainly do not hold.
  final List<AvasthaLajjitadi> ruledOut;

  /// The states nothing decides: the necessary condition holds and what
  /// narrows it further is not in the chart.
  final List<AvasthaLajjitadi> undecided;
}

/// A planetary war a body is in.
final class War {
  const War({
    required this.opponent,
    required this.isWinner,
    required this.apartDeg,
  });

  /// The other body.
  final Graha opponent;

  /// Whether this body won it.
  final bool isWinner;

  /// How far apart they stand, degrees.
  final double apartDeg;
}

/// What one graha **is**, as opposed to where it is.
final class GrahaState {
  const GrahaState({
    required this.graha,
    required this.sign,
    required this.house,
    required this.dignity,
    required this.friendship,
    required this.combustion,
    required this.age,
    required this.wakefulness,
    required this.deeptadi,
    required this.lajjitadi,
    required this.war,
    required this.sayanadi,
    required this.boundaries,
  });

  /// Which graha.
  final Graha graha;

  /// The sign it stands in.
  final Rashi sign;

  /// The bhava it falls in, under the chart's placement system.
  final int house;

  /// Its dignity.
  final Dignity dignity;

  /// How it stands to its dispositor.
  final Friendship friendship;

  /// What the Sun does to it.
  final Combustion combustion;

  /// Which fifth of its sign it stands in.
  final AvasthaBaladi age;

  /// Awake, dreaming or asleep.
  final AvasthaJagradadi wakefulness;

  /// The bright state, where the SDK can decide one.
  final AvasthaDeeptadi? deeptadi;

  /// The lajjitadi that hold, and the ones nothing decides.
  final Lajjitadi lajjitadi;

  /// The war it is in, if it is in one.
  final War? war;

  /// The Sayanadi state and its sub-states, or `null` for a body the verses
  /// give no number.
  final Sayanadi? sayanadi;

  /// How near it stands to a classification boundary.
  final EdgeDistance boundaries;
}

/// A graha's Sayanadi state, with its sub-state under a name of each anka
/// (BPHS ch. 45 vv. 30 to 37).
final class Sayanadi {
  const Sayanadi({required this.avastha, required this.cheshtas});

  /// The state, Shayana to Nidra.
  final AvasthaSayanadi avastha;

  /// The sub-state under a name whose first syllable's anka is 1 to 5, in
  /// that order.
  final List<AvasthaCheshta> cheshtas;

  /// The sub-state under a name of this anka.
  ///
  /// ```dart
  /// final cheshta = state.sayanadi?.cheshta(3);
  /// ```
  ///
  /// Throws an [ArgumentError] outside 1 to 5.
  AvasthaCheshta cheshta(int anka) {
    RangeError.checkValueInInterval(anka, 1, 5, 'anka');
    return cheshtas[anka - 1];
  }
}

/// One bhava as the houses service reads it.
final class ServiceBhava {
  const ServiceBhava({
    required this.number,
    required this.sign,
    required this.lord,
    required this.quadrant,
  });

  /// The bhava, 1 to 12.
  final int number;

  /// The sign its **middle** falls in.
  final Rashi sign;

  /// The lord of that sign.
  final Graha lord;

  /// Which third of the wheel it stands in.
  final Quadrant quadrant;
}

/// One derived point: an upagraha or a special lagna.
final class DerivedPoint {
  const DerivedPoint({
    required this.point,
    required this.longitudeDeg,
    required this.sign,
    required this.boundaries,
  });

  /// Which point.
  final Point point;

  /// Its longitude in the chart's zodiac, degrees.
  final double longitudeDeg;

  /// The sign it falls in.
  final Rashi sign;

  /// How near it stands to a sign, nakshatra or pada edge.
  final EdgeDistance boundaries;
}

/// One body looking at another.
final class Drishti {
  const Drishti({
    required this.from,
    required this.to,
    required this.houses,
    required this.strength,
    required this.fromEdge,
    required this.toEdge,
  });

  /// The body looking.
  final Graha from;

  /// The body looked at.
  final Graha to;

  /// Which house of the first's sign the second stands in, counting
  /// inclusively from one.
  final int houses;

  /// How strongly.
  final Strength strength;

  /// How near the looking body stands to a boundary.
  final EdgeDistance fromEdge;

  /// How near the body looked at stands to one.
  final EdgeDistance toEdge;
}

/// One graha's Ashtakavarga.
final class GrahaAshtakavarga {
  const GrahaAshtakavarga({
    required this.graha,
    required this.bindus,
    required this.reduced,
    required this.rashiPinda,
    required this.grahaPinda,
    required this.yogaPinda,
  });

  /// Which graha, Sun to Saturn.
  final Graha graha;

  /// Its bindus by sign, Aries to Pisces, 0 to 8.
  final List<int> bindus;

  /// The same after both reductions, when they were made in each graha's own
  /// Ashtakavarga; null otherwise.
  final List<int>? reduced;

  /// Its rashi pinda.
  final int rashiPinda;

  /// Its graha pinda.
  final int grahaPinda;

  /// Its yoga pinda, the two together.
  final int yogaPinda;
}

/// One bhava's Bhava bala, in virupas.
final class BhavaStrength {
  const BhavaStrength({
    required this.bhava,
    required this.lord,
    required this.adhipati,
    required this.dig,
    required this.drishti,
    required this.special,
    required this.virupas,
  });

  /// Which bhava, 1 to 12.
  final int bhava;

  /// The lord of the sign its madhya falls in.
  final Graha lord;

  /// The lord's Shadbala.
  final double adhipati;

  /// From its direction, 0 to 60.
  final double dig;

  /// From the drishtis it receives, which may be negative.
  final double drishti;

  /// From its occupants and its sign's rising, under BPHS's special rules.
  final double special;

  /// The four together.
  final double virupas;
}

/// A chart's Bhava bala, read under the context's `strength.bhava_*` settings
/// (`03-design/bhava-bala-measured.md`).
final class BhavaBala {
  const BhavaBala({required this.bhavas});

  /// Each bhava's, the first to the twelfth.
  final List<BhavaStrength> bhavas;
}

/// A graha's Sthana bala by component, virupas.
final class SthanaBala {
  const SthanaBala({
    required this.uchcha,
    required this.saptavargaja,
    required this.ojayugma,
    required this.kendradi,
    required this.drekkana,
  });

  /// From its distance to its debilitation point, 0 to 60.
  final double uchcha;

  /// From its dignity in the seven vargas.
  final double saptavargaja;

  /// From its rasi's and navamsha's parity, 0, 15 or 30.
  final double ojayugma;

  /// From its house: 60, 30 or 15.
  final double kendradi;

  /// From its decanate: 0 or 15.
  final double drekkana;

  /// The five together.
  double get total => uchcha + saptavargaja + ojayugma + kendradi + drekkana;
}

/// A graha's Kaala bala by component, virupas.
final class KaalaBala {
  const KaalaBala({
    required this.nathonnatha,
    required this.paksha,
    required this.tribhaga,
    required this.abda,
    required this.masa,
    required this.vara,
    required this.hora,
    required this.ayana,
    required this.yuddha,
  });

  /// From the hour, 0 to 60.
  final double nathonnatha;

  /// From the Moon's elongation, the Moon's doubled.
  final double paksha;

  /// 60 to the lord of the third of the day or night, and to Jupiter.
  final double tribhaga;

  /// 15 to the year's lord.
  final double abda;

  /// 30 to the month's lord.
  final double masa;

  /// 45 to the weekday's lord.
  final double vara;

  /// 60 to the hour's lord.
  final double hora;

  /// From its declination.
  final double ayana;

  /// Gained by the victor and lost by the vanquished of a planetary war.
  final double yuddha;

  /// The nine together.
  double get total =>
      nathonnatha +
      paksha +
      tribhaga +
      vara +
      hora +
      ayana +
      abda +
      masa +
      yuddha;
}

/// One graha's Shadbala, in virupas.
final class GrahaShadbala {
  const GrahaShadbala({
    required this.graha,
    required this.sthana,
    required this.dig,
    required this.kaala,
    required this.cheshta,
    required this.naisargika,
    required this.drik,
    required this.virupas,
    required this.rupas,
    required this.requiredRupas,
    required this.strong,
    required this.ishta,
    required this.kashta,
    required this.subhaRashmi,
    required this.ashubhaRashmi,
  });

  /// Which graha, Sun to Saturn.
  final Graha graha;

  /// Positional strength by component.
  final SthanaBala sthana;

  /// Directional strength, 0 to 60.
  final double dig;

  /// Temporal strength by component.
  final KaalaBala kaala;

  /// Motional strength.
  final double cheshta;

  /// Natural strength.
  final double naisargika;

  /// Aspectual strength, which may be negative.
  final double drik;

  /// The six together.
  final double virupas;

  /// The six together, in rupas.
  final double rupas;

  /// The rupas it must reach to be strong.
  final double requiredRupas;

  /// Whether it reaches them.
  final bool strong;

  /// How far it tends to good, 0 to 60 (BPHS ch. 28).
  final double ishta;

  /// How far it tends to harm, 0 to 60.
  final double kashta;

  /// Its auspicious rays, 1 to 7: the mean of its Uchcha and Cheshta rays
  /// (BPHS ch. 28 v. 5).
  final double subhaRashmi;

  /// Its inauspicious rays, 8 less the auspicious.
  final double ashubhaRashmi;
}

/// A chart's Shadbala, read under the context's `strength.*` settings
/// (`03-design/shadbala-measured.md`).
final class Shadbala {
  const Shadbala({required this.grahas});

  /// Each graha's, Sun to Saturn.
  final List<GrahaShadbala> grahas;
}

/// One graha's dasha phala (BPHS ch. 28 vv. 7 to 10, ch. 47 vv. 3 to 6).
final class GrahaDashaPhala {
  const GrahaDashaPhala({
    required this.graha,
    required this.subhankas,
    required this.subhanka,
    required this.asubhanka,
    required this.nature,
    required this.phase,
    required this.favourable,
    required this.unfavourable,
  });

  /// Which graha, Sun to Ketu.
  final Graha graha;

  /// Its Subhanka in the D1, D2, D3, D7, D9, D12 and D30: out of 60 in the
  /// first and 30 in the rest.
  final List<double> subhankas;

  /// The seven together, out of 240.
  final double subhanka;

  /// Their complements together, out of 240.
  final double asubhanka;

  /// Whether its rasi place is auspicious (benefic), neutral or inauspicious
  /// (malefic).
  final Nature nature;

  /// Where in its dasha its effects come.
  final DashaPhase phase;

  /// Whether its placement makes its dasha favourable.
  final bool favourable;

  /// Whether its placement makes its dasha unfavourable; both can hold.
  final bool unfavourable;
}

/// What a gochar reading counted its houses from, and that point's sign.
final class GocharReference {
  const GocharReference({required this.from, required this.sign});

  /// The natal Moon (v. 1) or, asked, the lagna (C139).
  final GocharFrom from;

  /// Its sign.
  final Rashi sign;
}

/// The readings of the nodes a transit was judged under, the settings'
/// `gochar` group.
final class GocharRules {
  const GocharRules({
    required this.nodeVedha,
    required this.nodeObstruction,
    required this.ashtakavargaGoodFrom,
  });

  /// The nodes' vedha (C136).
  final NodeVedha nodeVedha;

  /// Whom the nodes obstruct (C137, C140).
  final NodeObstruction nodeObstruction;

  /// How many bindus make a transit good by the Ashtakavarga (C141).
  final AshtakavargaGoodFrom ashtakavargaGoodFrom;
}

/// The eighth of a sign a transit stands in, 3°45′ each (Phaladeepika ch.
/// 23 v. 16), and its lord in the orbits' order (vv. 18 and 19).
final class Kakshya {
  const Kakshya({required this.index, required this.lord});

  /// Which eighth, 1 to 8.
  final int index;

  /// Its lord.
  final KakshyaLord lord;
}

/// One graha's transit judged by the natal Ashtakavarga.
final class AshtakavargaTransit {
  const AshtakavargaTransit({
    required this.graha,
    required this.bindus,
    required this.good,
    required this.kakshya,
    required this.kakshyaBindu,
    required this.sarva,
    required this.sarvaStanding,
  });

  /// Which graha, Sun to Saturn.
  final Graha graha;

  /// The bindus its own Ashtakavarga put in the sign it transits, 0 to 8
  /// (v. 11), unreduced.
  final int bindus;

  /// Whether they reach `gochar.ashtakavarga_good_from`.
  final bool good;

  /// The eighth of the sign it stands in.
  final Kakshya kakshya;

  /// Whether that eighth's lord gave a bindu there, so that a bindu bears
  /// its fruit now.
  final bool kakshyaBindu;

  /// The sign's sarvashtakavarga.
  final int sarva;

  /// Where it stands against 28 (v. 20).
  final SarvaStanding sarvaStanding;
}

/// A graha in transit: the sign it stands in and its degrees within it.
final class Transit {
  const Transit({required this.sign, required this.degrees});

  /// The sign.
  final Rashi sign;

  /// Degrees within the sign, 0 to 30.
  final double degrees;
}

/// One graha's transit, read from the reference sign.
final class GrahaGochar {
  const GrahaGochar({
    required this.graha,
    required this.transit,
    required this.house,
    required this.goodHouse,
    required this.vedhaHouse,
    required this.obstructedBy,
    required this.verdict,
    required this.fruition,
    required this.fruitfulNow,
  });

  /// Which graha.
  final Graha graha;

  /// Where it stands.
  final Transit transit;

  /// Its house from the reference sign, 1 to 12.
  final int house;

  /// Whether v. 2 makes a transit of this house good.
  final bool goodHouse;

  /// The house whose occupant obstructs it (vv. 3 to 8); null where
  /// nothing can.
  final int? vedhaHouse;

  /// The grahas standing in the vedha house that obstruct it, the verses'
  /// exemptions left out, in id order.
  final List<Graha> obstructedBy;

  /// What the transit comes to.
  final GocharVerdict verdict;

  /// The decanate in which its transit bears fruit (v. 25).
  final Fruition fruition;

  /// Whether it stands in that decanate now.
  final bool fruitfulNow;
}

/// Every graha's transit at one instant, read from the natal chart.
final class GocharReading {
  const GocharReading({
    required this.instant,
    required this.reference,
    required this.rules,
    required this.grahas,
    required this.ashtakavarga,
  });

  /// The instant, a UTC Julian day.
  final double instant;

  /// What the houses are counted from.
  final GocharReference reference;

  /// The readings of the nodes it was judged under.
  final GocharRules rules;

  /// Each graha's, the Sun to Ketu.
  final List<GrahaGochar> grahas;

  /// The seven judged by the natal Ashtakavarga, Sun to Saturn; null unless
  /// `ashtakavarga` asked.
  final List<AshtakavargaTransit>? ashtakavarga;
}

/// A chart's karakamsha: the Atmakaraka's navamsha sign (BPHS ch. 33 v. 1).
final class Karakamsha {
  const Karakamsha({
    required this.atmakaraka,
    required this.sign,
    required this.inRasi,
    required this.inNavamsha,
  });

  /// The Atmakaraka, under `jaimini.chara_karakas`.
  final Graha atmakaraka;

  /// The karakamsha, the Atmakaraka's navamsha sign.
  final Rashi sign;

  /// Each graha's house from it in the rasi chart, 1 to 12, the Sun to Ketu.
  final List<int> inRasi;

  /// Each graha's house from it in the navamsha, 1 to 12, the Sun to Ketu
  /// (C130).
  final List<int> inNavamsha;
}

/// A chart's Brahma graha, and how it was found (BPHS ch. 46 vv. 170 to 173).
final class Brahma {
  const Brahma({
    required this.rule,
    required this.countedFrom,
    required this.qualified,
    required this.graha,
    required this.passedFrom,
    required this.none,
  });

  /// The rule it was sought under, `jaimini.brahma`.
  final BrahmaRule rule;

  /// The stronger of the lagna and the 7th, which the rule counts from.
  final Rashi countedFrom;

  /// The planets that met the rule's marks, in id order.
  final List<Graha> qualified;

  /// The Brahma graha; null where the rule finds none.
  final Graha? graha;

  /// Saturn or the node that passed Brahma-hood to the planet in the 6th
  /// from it (C127).
  final Graha? passedFrom;

  /// Why there is none; null when there is one, and never
  /// [BrahmaOutcome.found].
  final BrahmaOutcome? none;
}

/// A chart's Jaimini significators, read under the settings' `jaimini`
/// group.
///
/// ```dart
/// final chart = ctx.chart.found(/* … */ jaimini: true);
/// final brahma = chart.jaimini!.brahma;
/// final why = brahma.graha == null ? brahma.none : null;
/// ```
final class JaiminiReading {
  const JaiminiReading({
    required this.karakamsha,
    required this.brahma,
    required this.grahaArudhas,
  });

  /// The karakamsha, with every graha's house from it in both charts.
  final Karakamsha karakamsha;

  /// The Brahma graha, or why there is none.
  final Brahma brahma;

  /// Each graha's arudha, the Sun to Ketu (BPHS ch. 29 vv. 6 and 7), under
  /// `jaimini.graha_arudha_exception`; null for a node that owns no sign
  /// under `jaimini.node_co_lordship`.
  final List<Rashi?> grahaArudhas;
}

/// A chart's dasha phala, read under `dasha.shanta_sign`.
///
/// ```dart
/// final chart = ctx.chart.found(/* … */ dashaPhala: true);
/// final saturn = chart.dashaPhala!.grahas.firstWhere((g) => g.graha == Graha.saturn);
/// ```
final class DashaPhalaReading {
  const DashaPhalaReading({required this.grahas});

  /// Each graha's, Sun to Ketu.
  final List<GrahaDashaPhala> grahas;
}

/// A graha's standing in one scheme of vargas.
final class VaiseshikamsaStanding {
  const VaiseshikamsaStanding({required this.goodVargas, required this.name});

  /// How many of the scheme's vargas are good for it.
  final int goodVargas;

  /// The name that count earns, from two good vargas; null below.
  final Vaiseshikamsa? name;
}

/// One graha's Vaiseshikamsa (BPHS ch. 6 vv. 42 to 53).
final class GrahaVaiseshikamsa {
  const GrahaVaiseshikamsa({
    required this.graha,
    required this.shadvarga,
    required this.saptavarga,
    required this.dashavarga,
    required this.shodashavarga,
    required this.impaired,
  });

  /// Which graha, Sun to Saturn.
  final Graha graha;

  /// Over the six vargas.
  final VaiseshikamsaStanding shadvarga;

  /// Over the seven.
  final VaiseshikamsaStanding saptavarga;

  /// Over the ten.
  final VaiseshikamsaStanding dashavarga;

  /// Over the sixteen.
  final VaiseshikamsaStanding shodashavarga;

  /// Whether it is combust, defeated in war or in Shayana, its names then not auspicious.
  final bool impaired;
}

/// A chart's Vaiseshikamsa.
final class VaiseshikamsaReading {
  const VaiseshikamsaReading({required this.grahas});

  /// Each graha's, Sun to Saturn.
  final List<GrahaVaiseshikamsa> grahas;
}

/// One graha's Vimshopaka, each score out of 20.
final class GrahaVimshopaka {
  const GrahaVimshopaka({
    required this.graha,
    required this.shadvarga,
    required this.saptavarga,
    required this.dashavarga,
    required this.shodashavarga,
  });

  /// Which graha, Sun to Saturn.
  final Graha graha;

  /// Over the six vargas.
  final double shadvarga;

  /// Over the seven.
  final double saptavarga;

  /// Over the ten.
  final double dashavarga;

  /// Over the sixteen.
  final double shodashavarga;
}

/// A chart's Vimshopaka: each graha's strength across the divisional charts
/// under the four schemes (`03-design/vimshopaka-measured.md`).
final class Vimshopaka {
  const Vimshopaka({required this.scoring, required this.grahas});

  /// How each varga was scored.
  final VimshopakaScoring scoring;

  /// Each graha's, Sun to Saturn.
  final List<GrahaVimshopaka> grahas;
}

/// A chart's Ashtakavarga: each graha's, the sarvashtakavarga, and their
/// reductions and pindas (`03-design/ashtakavarga-measured.md`).
final class Ashtakavarga {
  const Ashtakavarga({
    required this.shodhana,
    required this.ekadhipatya,
    required this.grahas,
    required this.sarva,
    required this.trikona,
    required this.reduced,
  });

  /// Where the reductions and pindas were made.
  final Shodhana shodhana;

  /// How a co-ruled sign beside an occupied one was reduced.
  final Ekadhipatya ekadhipatya;

  /// Each graha's, Sun to Saturn.
  final List<GrahaAshtakavarga> grahas;

  /// The seven grahas' bindus by sign, 337 in all.
  final List<int> sarva;

  /// The sum after the trine reduction.
  final List<int> trikona;

  /// The sum after both reductions.
  final List<int> reduced;
}

/// One period of a dasha.
final class DashaPeriod {
  const DashaPeriod({
    required this.path,
    required this.level,
    required this.sign,
    required this.lord,
    required this.from,
    required this.to,
  });

  /// Its place at each level from the mahadasha down, joined by `/`:
  /// `2/5/3`.
  final String path;

  /// How deep: 1 for a mahadasha.
  final int level;

  /// The sign it is the period of, in a sign-based dasha; null otherwise.
  final Rashi? sign;

  /// Its lord.
  final Graha lord;

  /// When it begins, a Julian day (UTC).
  final double from;

  /// When it ends, a Julian day (UTC).
  final double to;
}

/// A balance written as a reader writes it.
final class WrittenBalance {
  const WrittenBalance({
    required this.years,
    required this.months,
    required this.days,
    required this.hours,
    required this.minutes,
  });

  /// Whole years of the year length.
  final int years;

  /// Whole months of a twelfth of it.
  final int months;

  /// Whole days.
  final int days;

  /// Hours.
  final int hours;

  /// Minutes, rounded.
  final int minutes;
}

/// What remained of a dasha's first period at birth.
final class DashaBalance {
  const DashaBalance({
    required this.method,
    required this.remaining,
    required this.days,
    required this.written,
  });

  /// How it was measured.
  final Balance method;

  /// The fraction still to run, 0 to 1.
  final double remaining;

  /// That fraction of the first lord's years, in days.
  final double days;

  /// The same in years, months, days, hours and minutes.
  final WrittenBalance written;
}

/// A dasha of a founded chart: its periods, and for a nakshatra-seeded one
/// its seed and balance at birth. A sign-based dasha has neither, and its
/// periods name their signs.
final class Dasha {
  const Dasha({
    required this.system,
    required this.seed,
    required this.firstLord,
    required this.overflow,
    required this.balance,
    required this.moonSpan,
    required this.depth,
    required this.periods,
  });

  /// Which system: a [DashaSystem], or one a context registered, as
  /// `DashaSystem.registered('ACME_SAPTAKA')`.
  final KeyOf<DashaSystem> system;

  /// The nakshatra the Moon stood in, which seeds it; null for a sign-based
  /// dasha.
  final Nakshatra? seed;

  /// The lord it starts with.
  final Graha firstLord;

  /// Whether the seed lay outside a conditional system's nakshatras.
  final bool overflow;

  /// What remained of the first period at birth; null for a sign-based
  /// dasha, whose first period runs whole from birth.
  final DashaBalance? balance;

  /// The Moon's stay in its nakshatra, when the balance read one.
  final Interval? moonSpan;

  /// How many levels the periods go down.
  final int depth;

  /// Every period of the birth cycle to [depth], depth first in time
  /// order: a mahadasha, then its antardashas and theirs, then the next.
  final List<DashaPeriod> periods;

  /// The periods running at a Julian day (UTC), from the mahadasha down to
  /// [depth]; empty before birth and past the end of the cycle.
  List<DashaPeriod> at(double jd) => _chainAt(periods, jd);
}

/// The periods running at a Julian day (UTC), from the mahadasha down.
///
/// Depth first order means a period's children follow it, so one walk that
/// takes the next level's running period finds the chain.
List<DashaPeriod> _chainAt(List<DashaPeriod> periods, double jd) {
  final chain = <DashaPeriod>[];
  for (final period in periods) {
    if (period.level == chain.length + 1 &&
        period.from <= jd &&
        jd < period.to) {
      chain.add(period);
    }
  }
  return chain;
}

/// A dasha's periods from the columns of a period section — the births'
/// `dasha_periods` or the years' `year_dasha_periods`, which share a
/// layout — so a period is decoded in one place. A period's path is its
/// index below the nearest earlier period one level up.
List<DashaPeriod> _periodsOf({
  required List<int> level,
  required List<int> index,
  required List<int> sign,
  required List<int> lord,
  required List<double> from,
  required List<double> to,
  required int start,
  required int count,
  required bool Function(int i) signed,
}) {
  final path = <int>[];
  return List<DashaPeriod>.unmodifiable([
    for (var i = start; i < start + count; i += 1)
      DashaPeriod(
        path: (path
              ..length = level[i] - 1
              ..add(index[i]))
            .join('/'),
        level: level[i],
        sign: signed(i) ? Rashi.byId(sign[i]) : null,
        lord: Graha.byId(lord[i]),
        from: from[i],
        to: to[i],
      ),
  ]);
}

/// One lord of the ring a year's dasha runs round.
final class AnnualDashaShare {
  const AnnualDashaShare({
    required this.lord,
    required this.sign,
    required this.weight,
  });

  /// Its lord: the graha, or the sign's lord when the share is a sign's.
  final Graha lord;

  /// The sign, when the share is one's: the Patyayini's lagna; null for a
  /// planet's.
  final Rashi? sign;

  /// Its weight, of which a lord's share of the year is its weight over the
  /// ring's: a nakshatra year's lord's natal years, or a Patyayini share's
  /// patyamsha in nanoarcseconds. 0 for a lord that runs for no time.
  final double weight;
}

/// The lords a year's dasha runs round, and where it opens.
final class DashaRing {
  const DashaRing({
    required this.shares,
    required this.first,
    required this.remaining,
  });

  /// In the order the ring runs.
  final List<AnnualDashaShare> shares;

  /// The place in [shares] the year opens with, from 0.
  final int first;

  /// How much of the first lord's share was still to run at the return, 0
  /// to 1, the rest closing the year; null when it runs whole from the
  /// return: the Patyayini, and [MuddaBalance.whole].
  final double? remaining;
}

/// One annual dasha of a year: its ring, the year it divides, and its
/// periods (`03-design/annual-dashas.md`).
final class AnnualDasha {
  const AnnualDasha({
    required this.system,
    required this.seed,
    required this.ring,
    required this.year,
    required this.periods,
  });

  /// Which of the three: [DashaSystem.patyayini], [DashaSystem.mudda] or
  /// [DashaSystem.varshaYogini].
  final DashaSystem system;

  /// The birth Moon's nakshatra, which seeds a nakshatra year; null for the
  /// Patyayini.
  final Nakshatra? seed;

  /// The lords the year runs round.
  final DashaRing ring;

  /// The year: from its return to where the clock closes it, under the
  /// default clock the next return.
  final Interval year;

  /// Every period to the rules' depth, depth first in time order; a period
  /// that runs for no time is not listed.
  final List<DashaPeriod> periods;

  /// The lord the year opens with.
  Graha get firstLord => ring.shares[ring.first].lord;

  /// The periods running at a Julian day (UTC), from the mahadasha down;
  /// empty outside the year.
  List<DashaPeriod> at(double jd) => _chainAt(periods, jd);
}

/// Where one body stands in a divisional chart.
///
/// `sign == rashi` is the body keeping the sign it was already in, which
/// in the navamsha is **vargottama** and in another chart is the same
/// fact without the name.
final class VargaPlacement {
  const VargaPlacement({
    required this.rashi,
    required this.part,
    required this.sign,
  });

  /// The sign the body stands in, in the rashi chart.
  final Rashi rashi;

  /// Which part of that sign it falls in, counted from zero.
  final int part;

  /// The sign the divisional chart puts it in.
  final Rashi sign;

  /// Whether the divisional chart leaves the body in the sign it was in.
  bool get keepsItsSign => sign == rashi;
}

/// Where one graha stands in a divisional chart.
final class PlacedInVarga {
  const PlacedInVarga({required this.graha, required this.at});

  /// Which graha.
  final Graha graha;

  /// Where it stands.
  final VargaPlacement at;
}

/// A point in a drawing's unit square, y downwards.
final class UnitPoint {
  const UnitPoint(this.x, this.y);

  /// From the left edge, 0 to 1.
  final double x;

  /// From the top edge, 0 to 1.
  final double y;

  factory UnitPoint._of(Map<String, Object?> raw) =>
      UnitPoint((raw['x']! as num).toDouble(), (raw['y']! as num).toDouble());

  Map<String, Object?> _json() => {'x': x, 'y': y};
}

/// One step of an outline, from wherever the previous step ended.
sealed class Segment {
  const Segment(this.to);

  /// Where the step ends.
  final UnitPoint to;

  factory Segment._of(Map<String, Object?> raw) {
    final to = UnitPoint._of(raw['to']! as Map<String, Object?>);
    return switch (raw['kind']) {
      'LINE' => LineSegment(to),
      'QUAD' => QuadSegment(
        UnitPoint._of(raw['control']! as Map<String, Object?>),
        to,
      ),
      'ARC' => ArcSegment(
        UnitPoint._of(raw['centre']! as Map<String, Object?>),
        raw['clockwise']! as bool,
        to,
      ),
      // A step the SDK does not write is refused, never read as a line.
      final kind => throw StateError('an outline step of kind $kind'),
    };
  }

  Map<String, Object?> _json() => switch (this) {
    LineSegment() => {'kind': 'LINE', 'to': to._json()},
    QuadSegment(:final control) => {
      'kind': 'QUAD',
      'control': control._json(),
      'to': to._json(),
    },
    ArcSegment(:final centre, :final clockwise) => {
      'kind': 'ARC',
      'centre': centre._json(),
      'clockwise': clockwise,
      'to': to._json(),
    },
  };
}

/// A straight line to a point.
final class LineSegment extends Segment {
  const LineSegment(super.to);
}

/// A quadratic curve to a point, pulled towards its control.
final class QuadSegment extends Segment {
  const QuadSegment(this.control, super.to);

  /// The control point.
  final UnitPoint control;
}

/// A circular arc about a centre to a point the same distance from it.
final class ArcSegment extends Segment {
  const ArcSegment(this.centre, this.clockwise, super.to);

  /// The circle's centre.
  final UnitPoint centre;

  /// Which way the arc runs, as a reader sees it.
  final bool clockwise;
}

/// A closed outline: a start and the steps back to it.
final class Outline {
  const Outline({required this.start, required this.segments});

  /// Where the outline starts.
  final UnitPoint start;

  /// The steps around it.
  final List<Segment> segments;

  factory Outline._of(Map<String, Object?> raw) => Outline(
    start: UnitPoint._of(raw['start']! as Map<String, Object?>),
    segments: [
      for (final step in raw['segments']! as List<Object?>)
        Segment._of(step! as Map<String, Object?>),
    ],
  );
  Map<String, Object?> _json() => {
    'start': start._json(),
    'segments': [for (final step in segments) step._json()],
  };
}

/// Which way a layout's signs or houses run, as a reader sees it.
enum LayoutDirection {
  /// With the hands of a clock.
  clockwise('CLOCKWISE'),

  /// Against them.
  anticlockwise('ANTICLOCKWISE');

  const LayoutDirection(this.key);

  /// The key a layout row spells it with.
  final String key;

  static LayoutDirection _byKey(Object? key) => values.firstWhere(
    (direction) => direction.key == key,
    orElse: () => throw StateError('a layout running $key'),
  );
}

/// What a grid cell always carries: a sign, or a house.
sealed class CellHolds {
  const CellHolds();

  factory CellHolds._of(Map<String, Object?> raw) => switch (raw['kind']) {
    'SIGN' => HoldsSign(Rashi.byKey(raw['value']! as String) ?? Rashi.unknown),
    'HOUSE' => HoldsHouse(raw['value']! as int),
    final kind => throw StateError('a cell holding a $kind'),
  };

  Map<String, Object?> _json() => switch (this) {
    HoldsSign(:final sign) => {'kind': 'SIGN', 'value': sign.key},
    HoldsHouse(:final house) => {'kind': 'HOUSE', 'value': house},
  };
}

/// The cell is always this sign; its house moves with the lagna.
final class HoldsSign extends CellHolds {
  const HoldsSign(this.sign);

  /// The sign.
  final Rashi sign;
}

/// The cell is always this house, 1 to 12; its sign moves with the lagna.
final class HoldsHouse extends CellHolds {
  const HoldsHouse(this.house);

  /// The house.
  final int house;
}

/// One region of a grid layout.
final class LayoutCell {
  const LayoutCell({
    required this.outline,
    required this.holds,
    required this.label,
    required this.bodies,
  });

  /// The region's outline, in the unit square.
  final Outline outline;

  /// The sign or house the cell always carries.
  final CellHolds holds;

  /// Where the sign or house number is drawn.
  final UnitPoint label;

  /// Where the cell's bodies are stacked about.
  final UnitPoint bodies;

  Map<String, Object?> _json() => {
    'outline': outline._json(),
    'holds': holds._json(),
    'label': label._json(),
    'bodies': bodies._json(),
  };
}

/// What a ring of a radial layout counts its first house from.
enum RingReference {
  /// The lagna's sign.
  lagna('LAGNA'),

  /// The Moon's sign.
  moon('MOON'),

  /// The Sun's sign.
  sun('SUN'),

  /// The chart's cusps, each house as wide as it is.
  cusps('CUSPS'),

  /// Twelve signs of 30°, turned so the lagna's degree sits at the start.
  zodiac('ZODIAC');

  const RingReference(this.key);

  /// The key a layout row spells it with.
  final String key;

  static RingReference _byKey(Object? key) => values.firstWhere(
    (reference) => reference.key == key,
    orElse: () => throw StateError('a ring counting from $key'),
  );
}

/// One ring of a radial layout.
final class LayoutRing {
  const LayoutRing({
    required this.inner,
    required this.outer,
    required this.countsFrom,
  });

  /// The inner radius, a fraction of the square's side; 0 makes wedges.
  final double inner;

  /// The outer radius, at most a half.
  final double outer;

  /// What the ring counts its first house from.
  final RingReference countsFrom;

  Map<String, Object?> _json() => {
    'inner': inner,
    'outer': outer,
    'counts_from': countsFrom.key,
  };
}

/// A layout's shape: twelve cells fixed in the row, or rings computed per
/// chart.
sealed class LayoutShape {
  const LayoutShape(this.direction);

  /// Which way the signs or houses run.
  final LayoutDirection direction;

  factory LayoutShape._of(Map<String, Object?> raw) {
    final direction = LayoutDirection._byKey(raw['direction']);
    List<Map<String, Object?>> objects(String name) => [
      for (final item in raw[name]! as List<Object?>)
        item! as Map<String, Object?>,
    ];
    return switch (raw['kind']) {
      'RADIAL' => RadialShape(
        rings: [
          for (final ring in objects('rings'))
            LayoutRing(
              inner: (ring['inner']! as num).toDouble(),
              outer: (ring['outer']! as num).toDouble(),
              countsFrom: RingReference._byKey(ring['counts_from']),
            ),
        ],
        startsAt: raw['starts_at']! as int,
        direction: direction,
      ),
      'GRID' => GridShape(
        cells: [
          for (final cell in objects('cells'))
            LayoutCell(
              outline: Outline._of(cell['outline']! as Map<String, Object?>),
              holds: CellHolds._of(cell['holds']! as Map<String, Object?>),
              label: UnitPoint._of(cell['label']! as Map<String, Object?>),
              bodies: UnitPoint._of(cell['bodies']! as Map<String, Object?>),
            ),
        ],
        frame: [for (final path in objects('frame')) Outline._of(path)],
        direction: direction,
      ),
      final kind => throw StateError('a layout shaped as $kind'),
    };
  }

  Map<String, Object?> _json() => switch (this) {
    GridShape(:final cells, :final frame) => {
      'kind': 'GRID',
      'cells': [for (final cell in cells) cell._json()],
      'frame': [for (final path in frame) path._json()],
      'direction': direction.key,
    },
    RadialShape(:final rings, :final startsAt) => {
      'kind': 'RADIAL',
      'rings': [for (final ring in rings) ring._json()],
      'starts_at': startsAt,
      'direction': direction.key,
    },
  };
}

/// Twelve cells fixed in the row.
final class GridShape extends LayoutShape {
  const GridShape({
    required this.cells,
    required this.frame,
    required LayoutDirection direction,
  }) : super(direction);

  /// The twelve cells, in the order the row lists them.
  final List<LayoutCell> cells;

  /// Lines drawn that hold nothing: the border, a divider.
  final List<Outline> frame;
}

/// Rings of sectors computed from the chart, innermost first.
final class RadialShape extends LayoutShape {
  const RadialShape({
    required this.rings,
    required this.startsAt,
    required LayoutDirection direction,
  }) : super(direction);

  /// The rings, innermost first.
  final List<LayoutRing> rings;

  /// The clock hour house 1 starts at, 1 to 12.
  final int startsAt;
}

/// One lord of a consumer's dasha system and its whole years.
final class DashaLord {
  const DashaLord(this.graha, this.years);

  /// The graha.
  final Graha graha;

  /// Its whole years in the cycle.
  final int years;

  /// The lord as the JSON a definition crosses as.
  Map<String, Object?> toJson() => {'graha': graha.key, 'years': years};
}

/// A dasha system of your own, of either kernel, as `dashaSystems` takes
/// it (`03-design/dasha-kernels.md`).
///
/// Which kernel runs it is **stated** and never guessed from the fields
/// present, so a typo is refused by the field you wrote rather than by one
/// you did not.
///
/// ```dart
/// final ctx = teistro.context(dashaSystems: [
///   UduDashaDefinition(
///     key: 'ACME_SAPTAKA',
///     lords: [for (final g in [Graha.sun, Graha.moon, Graha.mars]) DashaLord(g, 10)],
///     reference: Nakshatra.krittika,
///   ),
///   RashiDashaDefinition(key: 'ACME_STHIRA', length: RashiLength.byModality(movable: 7, fixed: 8, dual: 9)),
/// ]);
/// ctx.chart.found(/* … */ dashas: [DashaSystem.registered('ACME_SAPTAKA')]);
/// ```
sealed class DashaDefinition {
  const DashaDefinition();

  /// Its key: `[A-Z][A-Z0-9_]`, at most 48 characters, and not one the
  /// catalogue has.
  String get key;

  /// The definition as the JSON a context's `dashaSystems` crosses as,
  /// naming its kernel.
  Map<String, Object?> toJson();
}

/// A nakshatra-seeded dasha system of your own: its key, its lords in order
/// and the reference nakshatra, every other field defaulting to
/// Vimshottari's shape.
final class UduDashaDefinition extends DashaDefinition {
  const UduDashaDefinition({
    required this.key,
    required this.lords,
    required this.reference,
    this.sources = const <String>[],
    this.count,
    this.span,
    this.groups = const <int>[],
    this.wheel,
    this.offset,
    this.repeats,
    this.yearLength,
    this.depth,
  });

  @override
  final String key;

  /// The lords, in the order they run.
  final List<DashaLord> lords;

  /// The nakshatra that maps to the first lord.
  final Nakshatra reference;

  /// Where the table comes from.
  final List<String> sources;

  /// `FROM_REFERENCE` (the default) or `TO_REFERENCE`.
  final String? count;

  /// How many nakshatras each lord covers; one by default.
  final int? span;

  /// How many nakshatras each lord covers, lord by lord, when they differ:
  /// BPHS's Ashtottari is `[4, 3, 4, 3, 4, 3, 4, 3]`. None by default.
  final List<int> groups;

  /// The circle the seed is counted round; the 27 nakshatras by default.
  final DashaWheel? wheel;

  /// What is added after the division, before the modulo; none by default.
  final int? offset;

  /// Whether the lords run round the nakshatras again; true by default.
  final bool? repeats;

  /// The length of its year (`JULIAN_365_25` by default, `SAVANA_360`, …).
  final String? yearLength;

  /// How many levels of periods a reading carries, 1 to 6; three by default.
  final int? depth;

  @override
  Map<String, Object?> toJson() => {
    'kernel': 'UDU',
    'key': key,
    'lords': [for (final lord in lords) lord.toJson()],
    'reference': reference.key,
    if (sources.isNotEmpty) 'sources': sources,
    if (count != null) 'count': count,
    if (span != null) 'span': span,
    if (groups.isNotEmpty) 'groups': groups,
    if (wheel case final wheel?) 'wheel': wheel.key,
    if (offset != null) 'offset': offset,
    if (repeats != null) 'repeats': repeats,
    if (yearLength != null) 'year_length': yearLength,
    if (depth != null) 'depth': depth,
  };
}

/// A sign-based (Jaimini) dasha system of your own: its key, and optionally
/// where it starts, the order it visits the signs in, how long a sign runs,
/// which lord a mahadasha names, and the houses to start from the strongest
/// of. Everything unsaid is Chara's.
final class RashiDashaDefinition extends DashaDefinition {
  const RashiDashaDefinition({
    required this.key,
    this.sources = const <String>[],
    this.start,
    this.order,
    this.length,
    this.namedLord,
    this.strongerOf = const <int>[],
    this.yearLength,
    this.depth,
  });

  @override
  final String key;

  /// Where the table comes from.
  final List<String> sources;

  /// Where it starts; the lagna by default.
  final RashiStart? start;

  /// The order it visits the signs in; every sign in turn by default.
  final RashiOrder? order;

  /// How long a sign's period runs; the count to its stronger lord by
  /// default.
  final RashiLength? length;

  /// Which lord a mahadasha names; the stronger of a dual-lorded sign's two
  /// by default.
  final RashiNamedLord? namedLord;

  /// The houses from the lagna to start from the strongest of; none by
  /// default, which starts from `start` itself.
  final List<int> strongerOf;

  /// The length of its year (`JULIAN_365_25` by default, `SAVANA_360`, …).
  final String? yearLength;

  /// How many levels of periods a reading carries, 1 to 6; three by default.
  final int? depth;

  @override
  Map<String, Object?> toJson() => {
    'kernel': 'RASHI',
    'key': key,
    if (sources.isNotEmpty) 'sources': sources,
    if (start case final start?) 'start': start.key,
    if (order case final order?) 'order': order.key,
    if (length case final length?) 'length': length._json,
    if (namedLord case final lord?) 'named_lord': lord.key,
    if (strongerOf.isNotEmpty) 'stronger_of': strongerOf,
    if (yearLength != null) 'year_length': yearLength,
    if (depth != null) 'depth': depth,
  };
}

/// The circle a nakshatra-seeded system counts its seed round.
enum DashaWheel {
  /// The twenty-seven nakshatras.
  nakshatras('NAKSHATRAS'),

  /// The twenty-eight with Abhijit, cut from Uttarashadha's last pada and
  /// Shravana's first fifteenth, as BPHS counts Ashtottari and
  /// Shashtihayani.
  withAbhijit('WITH_ABHIJIT');

  const DashaWheel(this.key);

  /// The key a definition spells it with.
  final String key;
}

/// Where a sign-based system starts.
enum RashiStart {
  /// The lagna.
  lagna('LAGNA'),

  /// The arudha lagna.
  arudhaLagna('ARUDHA_LAGNA'),

  /// The navamsa lagna.
  navamsaLagna('NAVAMSA_LAGNA');

  const RashiStart(this.key);

  /// The key a definition spells it with.
  final String key;
}

/// The order a sign-based system visits the signs in.
enum RashiOrder {
  /// Every sign in turn, forward from an odd start and back from an even.
  consecutive('CONSECUTIVE'),

  /// The trine groups from the start's.
  trineGroups('TRINE_GROUPS'),

  /// The ninth, tenth and eleventh from the start, each with the signs it
  /// aspects.
  drishtiChain('DRISHTI_CHAIN'),

  /// Back two signs at a time from the start.
  leap('LEAP');

  const RashiOrder(this.key);

  /// The key a definition spells it with.
  final String key;
}

/// Which lord a sign-based system's mahadasha names.
enum RashiNamedLord {
  /// The stronger of a dual-lorded sign's two.
  stronger('STRONGER'),

  /// The first, Ketu or Rahu.
  first('FIRST');

  const RashiNamedLord(this.key);

  /// The key a definition spells it with.
  final String key;
}

/// How long a sign's period runs, in a sign-based system.
sealed class RashiLength {
  const RashiLength._();

  /// The count to the sign's stronger lord.
  static const RashiLength countToLord = _KeyedLength('COUNT_TO_LORD');

  /// The same, a year more when that lord is exalted and a year less when
  /// debilitated.
  static const RashiLength countToLordByDignity = _KeyedLength(
    'COUNT_TO_LORD_BY_DIGNITY',
  );

  /// The same [years] for every sign.
  const factory RashiLength.fixed(int years) = _FixedLength;

  /// By the sign's modality.
  const factory RashiLength.byModality({
    required int movable,
    required int fixed,
    required int dual,
  }) = _ModalLength;

  Object get _json;
}

final class _KeyedLength extends RashiLength {
  const _KeyedLength(this.key) : super._();
  final String key;
  @override
  Object get _json => key;
}

final class _FixedLength extends RashiLength {
  const _FixedLength(this.years) : super._();
  final int years;
  @override
  Object get _json => {'FIXED': years};
}

final class _ModalLength extends RashiLength {
  const _ModalLength({
    required this.movable,
    required this.fixed,
    required this.dual,
  }) : super._();
  final int movable;
  final int fixed;
  final int dual;
  @override
  Object get _json => {
    'BY_MODALITY': {'movable': movable, 'fixed': fixed, 'dual': dual},
  };
}

/// A chart layout as a row: its key, what cites it, and its shape
/// (`03-design/chart-geometry.md` §7f).
final class LayoutRow {
  const LayoutRow({
    required this.key,
    required this.sources,
    required this.shape,
  });

  /// The key, in the key grammar: `[A-Z][A-Z0-9_]`, at most 48 characters.
  final String key;

  /// The sources the row comes from; at least one.
  final List<String> sources;

  /// Its cells or its rings.
  final LayoutShape shape;

  /// A row read from the JSON `ChartArea.layout` answers.
  factory LayoutRow.fromJson(Map<String, Object?> raw) => LayoutRow(
    key: raw['key']! as String,
    sources: [
      for (final source in raw['sources']! as List<Object?>) source! as String,
    ],
    shape: LayoutShape._of(raw['shape']! as Map<String, Object?>),
  );

  /// This row with what is named changed: a copy of a shipped row under a
  /// key of its own is a layout of your own.
  LayoutRow copyWith({
    String? key,
    List<String>? sources,
    LayoutShape? shape,
  }) => LayoutRow(
    key: key ?? this.key,
    sources: sources ?? this.sources,
    shape: shape ?? this.shape,
  );

  /// The row as the JSON a context's `layouts` crosses as.
  Map<String, Object?> toJson() => {
    'key': key,
    'sources': sources,
    'shape': shape._json(),
  };
}

/// One region of a drawn chart.
final class DrawnCell {
  const DrawnCell({
    required this.outline,
    required this.sign,
    required this.house,
    required this.lagna,
    required this.ring,
    required this.label,
    required this.anchor,
    required this.bodies,
  });

  /// The region's outline in the unit square.
  final Outline outline;

  /// The sign the cell shows; for a house between cusps, its cusp's sign.
  final Rashi sign;

  /// The house the cell shows, 1 to 12.
  final int house;

  /// Whether the lagna stands in this cell.
  final bool lagna;

  /// The ring, innermost 0; a grid's cells are all 0.
  final int ring;

  /// Where the sign or house number is drawn.
  final UnitPoint label;

  /// Where the cell's bodies are stacked about.
  final UnitPoint anchor;

  /// The bodies in the cell, as catalogue keys (`graha.SUN`).
  final List<String> bodies;
}

/// A body drawn at its own degree on a wheel.
final class DrawnMark {
  const DrawnMark({
    required this.body,
    required this.ring,
    required this.at,
    required this.longitudeDeg,
  });

  /// The body, as a catalogue key.
  final String body;

  /// The ring it is drawn in.
  final int ring;

  /// Where it is drawn.
  final UnitPoint at;

  /// The longitude that put it there, degrees.
  final double longitudeDeg;
}

/// A chart drawn in a layout (`03-design/chart-geometry.md`).
/// How a drawing looks: every field optional, over the theme it extends
/// (`03-design/render-svg.md`).
final class ThemeStyle {
  const ThemeStyle({
    this.size,
    this.background,
    this.ink,
    this.cell,
    this.lagnaCell,
    this.accent,
    this.stroke,
    this.fontFamily,
    this.bodySize,
    this.labelSize,
    this.markSize,
    this.advance,
    this.lineHeight,
    this.baselineShift,
  });

  /// The drawing's width and height, in SVG user units.
  final double? size;

  /// The page behind the chart, as `#rrggbb`.
  final String? background;

  /// Lines and text, as `#rrggbb`.
  final String? ink;

  /// A cell's fill, as `#rrggbb`.
  final String? cell;

  /// The fill of the cell the lagna stands in, as `#rrggbb`.
  final String? lagnaCell;

  /// The lagna's own label and mark, as `#rrggbb`.
  final String? accent;

  /// Line width, as a fraction of the size.
  final double? stroke;

  /// The font family every text asks for.
  final String? fontFamily;

  /// The largest a body's label is drawn, as a fraction of the size.
  final double? bodySize;

  /// A cell's label, as a fraction of the size.
  final double? labelSize;

  /// A body at its degree on a wheel, as a fraction of the size.
  final double? markSize;

  /// The width one character is estimated at, in ems.
  final double? advance;

  /// The distance between two lines of a stack, in ems.
  final double? lineHeight;

  /// How far below a line's centre its baseline sits, in ems.
  final double? baselineShift;

  // Only what is named: an absent field is the extended theme's.
  Map<String, Object?> _json() => <String, Object?>{
    'size': size,
    'background': background,
    'ink': ink,
    'cell': cell,
    'lagna_cell': lagnaCell,
    'accent': accent,
    'stroke': stroke,
    'font_family': fontFamily,
    'body_size': bodySize,
    'label_size': labelSize,
    'mark_size': markSize,
    'advance': advance,
    'line_height': lineHeight,
    'baseline_shift': baselineShift,
  }..removeWhere((_, value) => value == null);
}

/// The locale form a drawn body is written in.
enum BodyForm {
  /// The locale's abbreviation: `Su`, `सू`.
  short('SHORT'),

  /// The symbol: `☉`.
  glyph('GLYPH');

  const BodyForm(this.key);

  /// The key the theme record spells it with.
  final String key;
}

/// What a drawn cell's label shows.
enum CellLabel {
  /// The sign's number, or on a wheel the house and the sign's glyph.
  auto('AUTO'),

  /// The sign's number, 1 for Aries.
  signNumber('SIGN_NUMBER'),

  /// The sign's abbreviation.
  signShort('SIGN_SHORT'),

  /// The sign's symbol.
  signGlyph('SIGN_GLYPH'),

  /// The house's number.
  house('HOUSE'),

  /// Nothing.
  nothing('NOTHING');

  const CellLabel(this.key);

  /// The key the theme record spells it with.
  final String key;
}

/// What a drawing says: every field optional, over the theme it extends.
final class ThemeContent {
  const ThemeContent({
    this.bodyForm,
    this.cellLabel,
    this.lagnaMark,
    this.retrogradeMark,
    this.degrees,
  });

  /// The locale form a body is written in.
  final BodyForm? bodyForm;

  /// What a cell's label shows.
  final CellLabel? cellLabel;

  /// Whether the lagna is written first in the cell it stands in.
  final bool? lagnaMark;

  /// What is written after a retrograde graha's name; `''` for nothing.
  final String? retrogradeMark;

  /// Whether a graha's degree follows its name, on the founded chart.
  final bool? degrees;

  // Only what is named: an absent field is the extended theme's.
  Map<String, Object?> _json() => <String, Object?>{
    'body_form': bodyForm?.key,
    'cell_label': cellLabel?.key,
    'lagna_mark': lagnaMark,
    'retrograde_mark': retrogradeMark,
    'degrees': degrees,
  }..removeWhere((_, value) => value == null);
}

/// The theme a request writes its drawings as SVG in: a shipped one, or one
/// naming only what it changes over a shipped one (`03-design/render-svg.md`).
final class ChartTheme {
  const ChartTheme._(this._base, this._style, this._content);

  /// Dark ink on white, as a printed patrika.
  static const light = ChartTheme._('LIGHT', ThemeStyle(), ThemeContent());

  /// Light ink on a dark page.
  static const dark = ChartTheme._('DARK', ThemeStyle(), ThemeContent());

  /// This theme with what [style] and [content] name changed.
  ChartTheme copyWith({
    ThemeStyle style = const ThemeStyle(),
    ThemeContent content = const ThemeContent(),
  }) => ChartTheme._(_base, style, content);

  final String _base;
  final ThemeStyle _style;
  final ThemeContent _content;

  String get _json => jsonEncode({
    'extends': _base,
    'style': _style._json(),
    'content': _content._json(),
  });
}

final class Drawing {
  const Drawing({
    required this.layout,
    required this.varga,
    required this.cells,
    required this.frame,
    required this.marks,
    this.svg,
  });

  /// The drawing as SVG, in the request's theme and the context's locale;
  /// null when the request gave no theme.
  final String? svg;

  /// The layout it is drawn in: a [ChartLayout], or one the context
  /// registered.
  final KeyOf<ChartLayout> layout;

  /// Which chart: `Varga.d1` for the founded chart, or a divisional one.
  final Varga varga;

  /// The cells, in the layout's order.
  final List<DrawnCell> cells;

  /// The lines drawn that hold nothing.
  final List<Outline> frame;

  /// Each body at its own degree, on a wheel; empty for a grid.
  final List<DrawnMark> marks;

  factory Drawing._of(Map<String, Object?> raw, String? svg) {
    final placed = raw['placed']! as Map<String, Object?>;
    Map<String, Object?> object(Object? value) =>
        value! as Map<String, Object?>;
    return Drawing(
      svg: svg,
      layout:
          ChartLayout.byKey(placed['layout']! as String) ??
          ChartLayout.registered(placed['layout']! as String),
      varga: Varga.byKey(raw['varga']! as String) ?? Varga.unknown,
      cells: [
        for (final cell in (placed['cells']! as List<Object?>).map(object))
          DrawnCell(
            outline: Outline._of(object(cell['outline'])),
            sign: Rashi.byKey(cell['sign']! as String) ?? Rashi.unknown,
            house: cell['house']! as int,
            lagna: cell['lagna']! as bool,
            ring: cell['ring']! as int,
            label: UnitPoint._of(object(cell['label'])),
            anchor: UnitPoint._of(object(cell['anchor'])),
            bodies: [
              for (final body in cell['bodies']! as List<Object?>)
                body! as String,
            ],
          ),
      ],
      frame: [
        for (final path in (placed['frame']! as List<Object?>).map(object))
          Outline._of(path),
      ],
      marks: [
        for (final mark in (placed['marks']! as List<Object?>).map(object))
          DrawnMark(
            body: mark['body']! as String,
            ring: mark['ring']! as int,
            at: UnitPoint._of(object(mark['at'])),
            longitudeDeg: (mark['longitude_deg']! as num).toDouble(),
          ),
      ],
    );
  }
}

/// Each batch's Ashtakavargas, decoded once however many charts read them.
final Expando<List<Ashtakavarga>> _ashtakavargas = Expando<List<Ashtakavarga>>(
  'ashtakavargas',
);

List<Ashtakavarga> _ashtakavargasOf(Charts batch) =>
    _ashtakavargas[batch] ??= _decodeAshtakavargas(batch);

List<Ashtakavarga> _decodeAshtakavargas(Charts batch) {
  final rows = batch.ashtakavarga;
  final bins = batch.ashtakavargaBindus;
  final sums = batch.sarvashtakavarga;
  List<int> twelve(List<int> column, int from) =>
      List<int>.unmodifiable(column.sublist(from, from + 12));
  return List<Ashtakavarga>.generate(rows.length ~/ 7, (chart) {
    final grahas = List<GrahaAshtakavarga>.generate(7, (g) {
      final row = chart * 7 + g;
      final each = Shodhana.byId(rows.shodhana[row]) == Shodhana.eachGraha;
      return GrahaAshtakavarga(
        graha: Graha.byId(rows.graha[row]),
        bindus: twelve(bins.bindus, row * 12),
        reduced: each ? twelve(bins.reduced, row * 12) : null,
        rashiPinda: rows.rashiPinda[row],
        grahaPinda: rows.grahaPinda[row],
        yogaPinda: rows.yogaPinda[row],
      );
    }, growable: false);
    return Ashtakavarga(
      shodhana: Shodhana.byId(rows.shodhana[chart * 7]),
      ekadhipatya: Ekadhipatya.byId(rows.ekadhipatya[chart * 7]),
      grahas: grahas,
      sarva: twelve(sums.sarva, chart * 12),
      trikona: twelve(sums.trikona, chart * 12),
      reduced: twelve(sums.reduced, chart * 12),
    );
  }, growable: false);
}

/// Each batch's Bhava balas, decoded once however many charts read them.
final Expando<List<BhavaBala>> _bhavaBalas = Expando<List<BhavaBala>>(
  'bhavaBalas',
);

List<BhavaBala> _bhavaBalasOf(Charts batch) =>
    _bhavaBalas[batch] ??= _decodeBhavaBalas(batch);

List<BhavaBala> _decodeBhavaBalas(Charts batch) {
  final c = batch.bhavaBala;
  return List<BhavaBala>.generate(
    c.length ~/ 12,
    (chart) => BhavaBala(
      bhavas: List<BhavaStrength>.generate(12, (h) {
        final row = chart * 12 + h;
        return BhavaStrength(
          bhava: h + 1,
          lord: Graha.byId(c.lord[row]),
          adhipati: c.adhipati[row],
          dig: c.dig[row],
          drishti: c.drishti[row],
          special: c.special[row],
          virupas: c.virupas[row],
        );
      }, growable: false),
    ),
    growable: false,
  );
}

/// Each batch's Shadbalas, decoded once however many charts read them.
final Expando<List<Shadbala>> _shadbalas = Expando<List<Shadbala>>('shadbalas');

List<Shadbala> _shadbalasOf(Charts batch) =>
    _shadbalas[batch] ??= _decodeShadbalas(batch);

List<Shadbala> _decodeShadbalas(Charts batch) {
  final c = batch.shadbala;
  GrahaShadbala graha(int row) => GrahaShadbala(
    graha: Graha.byId(c.graha[row]),
    sthana: SthanaBala(
      uchcha: c.uchcha[row],
      saptavargaja: c.saptavargaja[row],
      ojayugma: c.ojayugma[row],
      kendradi: c.kendradi[row],
      drekkana: c.drekkana[row],
    ),
    dig: c.dig[row],
    kaala: KaalaBala(
      nathonnatha: c.nathonnatha[row],
      paksha: c.paksha[row],
      tribhaga: c.tribhaga[row],
      abda: c.abda[row],
      masa: c.masa[row],
      vara: c.vara[row],
      hora: c.hora[row],
      ayana: c.ayana[row],
      yuddha: c.yuddha[row],
    ),
    cheshta: c.cheshta[row],
    naisargika: c.naisargika[row],
    drik: c.drik[row],
    virupas: c.virupas[row],
    rupas: c.rupas[row],
    requiredRupas: c.requiredRupas[row],
    strong: c.strong[row] == 1,
    ishta: c.ishta[row],
    kashta: c.kashta[row],
    subhaRashmi: c.subhaRashmi[row],
    ashubhaRashmi: c.ashubhaRashmi[row],
  );
  return List<Shadbala>.generate(
    c.length ~/ 7,
    (chart) => Shadbala(
      grahas: List<GrahaShadbala>.generate(
        7,
        (g) => graha(chart * 7 + g),
        growable: false,
      ),
    ),
    growable: false,
  );
}

/// Each batch's dasha phalas, decoded once however many charts read them.
final Expando<List<DashaPhalaReading>> _dashaPhalas =
    Expando<List<DashaPhalaReading>>('dashaPhalas');

List<DashaPhalaReading> _dashaPhalasOf(Charts batch) =>
    _dashaPhalas[batch] ??= _decodeDashaPhalas(batch);

List<DashaPhalaReading> _decodeDashaPhalas(Charts batch) {
  final c = batch.dashaPhala;
  final subhankas = [
    c.subhankaD1,
    c.subhankaD2,
    c.subhankaD3,
    c.subhankaD7,
    c.subhankaD9,
    c.subhankaD12,
    c.subhankaD30,
  ];
  return List<DashaPhalaReading>.generate(
    c.length ~/ 9,
    (chart) => DashaPhalaReading(
      grahas: List<GrahaDashaPhala>.generate(9, (g) {
        final row = chart * 9 + g;
        return GrahaDashaPhala(
          graha: Graha.byId(c.graha[row]),
          subhankas: List<double>.unmodifiable([
            for (final column in subhankas) column[row],
          ]),
          subhanka: c.subhanka[row],
          asubhanka: c.asubhanka[row],
          nature: Nature.byId(c.nature[row]),
          phase: DashaPhase.byId(c.phase[row]),
          favourable: c.favourable[row] == 1,
          unfavourable: c.unfavourable[row] == 1,
        );
      }, growable: false),
    ),
    growable: false,
  );
}

/// Each batch's transits, decoded once however many charts read them.
/// A natal point from a `to_lagna` and a `to_graha` column's cells.
NatalPoint _pointAt(int toLagna, int toGraha) =>
    toLagna != 0 ? const NatalLagna() : NatalGraha(Graha.byId(toGraha));

/// One row of the `hits` section as the Rust `Hit` spells it.
Hit _hitAt(ChartsHits h, int row) {
  final motion = Motion.byId(h.motion[row]);
  final HitEvent event = switch (HitKind.byId(h.kind[row])) {
    HitKind.signIngress => SignIngress(
      into: Rashi.byId(h.into[row]),
      motion: motion,
    ),
    HitKind.nakshatraIngress => NakshatraIngress(
      into: Nakshatra.byId(h.into[row]),
      motion: motion,
    ),
    HitKind.station => Station(turns: motion),
    HitKind.aspect => AspectHit(
      to: _pointAt(h.toLagna[row], h.toGraha[row]),
      angle: h.angle[row],
      phase: AspectPhase.byId(h.phase[row]),
      motion: motion,
    ),
  };
  return Hit(
    instant: h.instant[row],
    graha: Graha.byId(h.graha[row]),
    event: event,
  );
}

final Expando<List<Dignities>> _dignities = Expando<List<Dignities>>(
  'dignities',
);

List<Dignities> _dignitiesOf(Charts batch) =>
    _dignities[batch] ??= _decodeDignities(batch);

/// `dignities` holds a row a chart, or none when none was asked, and
/// `dignity_planets` seven a chart, in the Chaldean order.
List<Dignities> _decodeDignities(Charts batch) {
  final c = batch.dignities;
  final p = batch.dignityPlanets;
  final r = batch.dignityReceptions;
  final charts = batch.cast.instant.length;
  if (c.length == 0) return const <Dignities>[];
  if (c.length != charts || p.length != 7 * charts) {
    throw StateError(
      'dignities has ${c.length} rows and dignity_planets ${p.length} for '
      '$charts charts; they are one and seven a chart',
    );
  }
  final starts = [0];
  for (var chart = 0; chart < charts; chart += 1) {
    starts.add(starts.last + c.receptionCount[chart]);
  }
  if (starts.last != r.length) {
    throw StateError(
      'dignity_receptions has ${r.length} rows and the charts count '
      '${starts.last}',
    );
  }
  // One row's seven flags, from columns in `EssentialDignity`'s order.
  EssentialDignity flags(List<List<int>> columns, int row) {
    final [house, exaltation, triplicity, term, face, detriment, fall] = [
      for (final column in columns) column[row] == 1,
    ];
    return EssentialDignity(
      house: house,
      exaltation: exaltation,
      triplicity: triplicity,
      term: term,
      face: face,
      detriment: detriment,
      fall: fall,
    );
  }

  final planetFlags = [
    p.house,
    p.exaltation,
    p.triplicity,
    p.term,
    p.face,
    p.detriment,
    p.fall,
  ];
  final firstFlags = [
    r.firstInHouse,
    r.firstInExaltation,
    r.firstInTriplicity,
    r.firstInTerm,
    r.firstInFace,
    r.firstInDetriment,
    r.firstInFall,
  ];
  final secondFlags = [
    r.secondInHouse,
    r.secondInExaltation,
    r.secondInTriplicity,
    r.secondInTerm,
    r.secondInFace,
    r.secondInDetriment,
    r.secondInFall,
  ];
  PlanetDignity planet(int row) {
    final dignity = flags(planetFlags, row);
    return PlanetDignity(
      planet: Graha.byId(p.planet[row]),
      longitudeDeg: p.longitude[row],
      dignity: dignity,
      peregrine: !DignityKind.values.any(dignity.holds),
      score: p.score[row],
      reception: p.reception[row],
    );
  }

  Reception reception(int row) => Reception(
    planets: (Graha.byId(r.first[row]), Graha.byId(r.second[row])),
    firstIn: flags(firstFlags, row),
    secondIn: flags(secondFlags, row),
  );

  return List<Dignities>.generate(
    charts,
    (chart) => Dignities(
      sect: Sect.byId(c.sect[chart]),
      sectRule: SectRule.byId(c.sectRule[chart]),
      rules: AppliedDignityRules(
        terms: Terms.byId(c.terms[chart]),
        triplicities: Triplicities.byId(c.triplicities[chart]),
      ),
      scores: DignityScores(
        house: c.scoreHouse[chart],
        exaltation: c.scoreExaltation[chart],
        triplicity: c.scoreTriplicity[chart],
        term: c.scoreTerm[chart],
        face: c.scoreFace[chart],
        detriment: c.scoreDetriment[chart],
        fall: c.scoreFall[chart],
        peregrine: c.scorePeregrine[chart],
      ),
      planets: List<PlanetDignity>.unmodifiable([
        for (var row = 7 * chart; row < 7 * chart + 7; row += 1) planet(row),
      ]),
      receptions: List<Reception>.unmodifiable([
        for (var row = starts[chart]; row < starts[chart + 1]; row += 1)
          reception(row),
      ]),
    ),
  );
}

final Expando<List<Lots>> _lots = Expando<List<Lots>>('lots');

List<Lots> _lotsOf(Charts batch) => _lots[batch] ??= _decodeLots(batch);

/// `lots` holds a row a chart, or none when none was asked, and
/// `lot_places` the catalogue's lots for each chart, in its order.
List<Lots> _decodeLots(Charts batch) {
  final c = batch.lots;
  final p = batch.lotPlaces;
  final charts = batch.cast.instant.length;
  if (c.length == 0) return const <Lots>[];
  final per = Lot.values.length;
  if (c.length != charts || p.length != per * charts) {
    throw StateError(
      'lots has ${c.length} rows and lot_places ${p.length} for $charts '
      'charts; they are one and $per a chart',
    );
  }
  PlacedLot placed(int row) => PlacedLot(
    lot: Lot.byId(p.lot[row]),
    place: LotPlace(
      longitudeDeg: p.longitudeDeg[row],
      sign: Rashi.byId(p.sign[row]),
      lord: Graha.byId(p.lord[row]),
      house: p.house[row],
    ),
  );
  return List<Lots>.generate(
    charts,
    (chart) => Lots(
      sect: Sect.byId(c.sect[chart]),
      request: LotRequest(
        sectRule: SectRule.byId(c.sectRule[chart]),
        fortune: FortuneRule.byId(c.fortune[chart]),
      ),
      fortuneReversed: c.fortuneReversed[chart] == 1,
      lots: List<PlacedLot>.unmodifiable([
        for (var row = per * chart; row < per * chart + per; row += 1)
          placed(row),
      ]),
    ),
  );
}

final Expando<List<Considerations>> _considerations =
    Expando<List<Considerations>>('considerations');

List<Considerations> _considerationsOf(Charts batch) =>
    _considerations[batch] ??= _decodeConsiderations(batch);

/// `considerations` holds a row a chart, or none when none was asked,
/// `consideration_perfections` two a chart (the Moon's next aspect, then
/// the first within the moieties), and `consideration_orbs` seven a chart
/// in the Chaldean order.
List<Considerations> _decodeConsiderations(Charts batch) {
  final c = batch.considerations;
  final p = batch.considerationPerfections;
  final o = batch.considerationOrbs;
  final charts = batch.cast.instant.length;
  if (c.length == 0) return const <Considerations>[];
  if (c.length != charts || p.length != 2 * charts || o.length != 7 * charts) {
    throw StateError(
      'considerations has ${c.length} rows, consideration_perfections '
      '${p.length} and consideration_orbs ${o.length} for $charts charts; '
      'they are one, two and seven a chart',
    );
  }
  Perfection? perfection(int row) =>
      p.present[row] == 1
          ? Perfection(
            planet: Graha.byId(p.planet[row]),
            aspect: PtolemaicAspect.byId(p.aspect[row]),
            days: p.days[row],
            gapDeg: p.gapDeg[row],
          )
          : null;
  return List<Considerations>.generate(
    charts,
    (k) => Considerations(
      radicality: Radicality(
        hourLord: Graha.byId(c.hourLord[k]),
        ascendantLord: Graha.byId(c.ascendantLord[k]),
        grounds: List<RadicalGround>.unmodifiable(
          _members<RadicalGround>(
            c.radicalGrounds[k],
            RadicalGround.values,
            (g) => g.id,
          ),
        ),
      ),
      ascendant: AscendantClause(
        sign: Rashi.byId(c.ascendantSign[k]),
        degree: c.ascendantDegree[k],
        early: c.ascendantEarly[k] == 1,
        late: c.ascendantLate[k] == 1,
        shortAscension: c.shortAscension[k] == 1,
      ),
      moon: MoonClause(
        sign: Rashi.byId(c.moonSign[k]),
        degree: c.moonDegree[k],
        late: c.moonLate[k] == 1,
        lateSign: c.moonLateSign[k] == 1,
        viaCombusta: c.viaCombusta[k] == 1,
        course: MoonCourse(
          next: perfection(2 * k),
          withinOrb: perfection(2 * k + 1),
          daysInSign: c.daysInSign[k],
          eased: c.eased[k] == 1,
        ),
      ),
      seventh: SeventhClause(
        cuspDeg: c.seventhCuspDeg[k],
        lord: Graha.byId(c.seventhLord[k]),
        infortunesInHouse: List<Graha>.unmodifiable(
          _seven(c.seventhInfortunes[k]),
        ),
        lordRetrograde: c.seventhLordRetrograde[k] == 1,
        lordCombust: c.seventhLordCombust[k] == 1,
        lordInFall: c.seventhLordInFall[k] == 1,
        lordInInfortuneTerm: c.seventhLordInInfortuneTerm[k] == 1,
        lordNet: c.seventhLordNet[k],
      ),
      saturnHouse: c.saturnHouse[k],
      saturnRetrograde: c.saturnRetrograde[k] == 1,
      ascendantLordCombust: c.ascendantLordCombust[k] == 1,
      rules: ConsiderationRules(
        moonLateFromDeg: c.moonLateFromDeg[k],
        orbsDeg: List<double>.unmodifiable(o.orbDeg.sublist(7 * k, 7 * k + 7)),
      ),
    ),
  );
}

final Expando<List<Progressions>> _progressions = Expando<List<Progressions>>(
  'progressions',
);

List<Progressions> _progressionsOf(Charts batch) =>
    _progressions[batch] ??= _decodeProgressions(batch);

/// `progressions` holds a row a chart, or none when none was asked; the
/// progressed and directed planets are the same number of rows a chart
/// when an instant was asked (the nine, and the outer three when the
/// birth placed them), and the contacts are ragged by the row's count.
List<Progressions> _decodeProgressions(Charts batch) {
  final p = batch.progressions;
  final g = batch.progressedGrahas;
  final d = batch.directedGrahas;
  final c = batch.progressedContacts;
  final charts = batch.cast.instant.length;
  if (p.length == 0) return const <Progressions>[];
  final asked = !p.life[0].isNaN;
  final perChart = asked ? g.length ~/ charts : 0;
  final contacts = p.contactCount.fold<int>(0, (sum, n) => sum + n);
  if (p.length != charts ||
      d.length != g.length ||
      g.length != perChart * charts ||
      c.length != contacts) {
    throw StateError(
      'progressions has ${p.length} rows, progressed_grahas ${g.length}, '
      'directed_grahas ${d.length} and progressed_contacts ${c.length} for '
      '$charts charts',
    );
  }
  var contact = 0;
  return List<Progressions>.unmodifiable([
    for (var k = 0; k < charts; k++)
      () {
        final first = contact;
        contact += p.contactCount[k];
        final planets = [
          for (var at = k * perChart; at < (k + 1) * perChart; at++) at,
        ];
        return Progressions(
          progressed:
              asked
                  ? Progressed(
                    life: p.life[k],
                    sky: p.sky[k],
                    armcDeg: p.armcDeg[k],
                    angles: ProgressedAngles(
                      ascendantDeg: p.ascendantDeg[k],
                      midheavenDeg: p.midheavenDeg[k],
                    ),
                    grahas: List<ProgressedPlanet>.unmodifiable([
                      for (final at in planets)
                        ProgressedPlanet(
                          graha: Graha.byId(g.graha[at]),
                          longitudeDeg: g.longitudeDeg[at],
                          tropicalDeg: g.tropicalDeg[at],
                          speedDegPerDay: g.speedDegPerDay[at],
                        ),
                    ]),
                  )
                  : null,
          directed:
              asked
                  ? Directed(
                    life: p.life[k],
                    arcDeg: p.arcDeg[k],
                    ascendantDeg: p.directedAscendantDeg[k],
                    midheavenDeg: p.directedMidheavenDeg[k],
                    planets: List<DirectedPlanet>.unmodifiable([
                      for (final at in planets)
                        DirectedPlanet(
                          graha: Graha.byId(d.graha[at]),
                          longitudeDeg: d.longitudeDeg[at],
                        ),
                    ]),
                  )
                  : null,
          contacts:
              p.contactsAsked[k] == 1
                  ? List<ProgressedContact>.unmodifiable([
                    for (var at = first; at < contact; at++)
                      ProgressedContact(
                        life: c.life[at],
                        sky: c.sky[at],
                        graha: Graha.byId(c.graha[at]),
                        to: _pointAt(c.toLagna[at], c.toGraha[at]),
                        angle: c.angle[at],
                        motion: Motion.byId(c.motion[at]),
                      ),
                  ])
                  : null,
        );
      }(),
  ]);
}

final Expando<List<List<WesternAspectRow>>> _westernAspects =
    Expando<List<List<WesternAspectRow>>>('westernAspects');

List<List<WesternAspectRow>> _westernAspectsOf(Charts batch) =>
    _westernAspects[batch] ??= _decodeWesternAspects(batch);

/// A per-chart table from a count section's [counts] and the [rows] it is
/// ragged by: each chart's rows, or none at all when nothing was asked.
List<List<T>> _ragged<T>(
  Charts batch,
  List<int> counts,
  int rows,
  String names,
  T Function(int at) read,
) => _raggedIn(batch.cast.instant.length, counts, rows, names, read);

/// A per-match table from a count column and the rows it is ragged by, for
/// [charts] rows: a chart batch's, or a naam blob's one match.
List<List<T>> _raggedIn<T>(
  int charts,
  List<int> counts,
  int rows,
  String names,
  T Function(int at) read,
) {
  if (counts.isEmpty) return List<List<T>>.unmodifiable(const []);
  final total = counts.fold<int>(0, (sum, n) => sum + n);
  if (counts.length != charts || rows != total) {
    throw StateError(
      '$names: ${counts.length} counts and $rows rows for $charts charts',
    );
  }
  final tables = <List<T>>[];
  var start = 0;
  for (final count in counts) {
    tables.add(
      List<T>.unmodifiable([
        for (var at = start; at < start + count; at++) read(at),
      ]),
    );
    start += count;
  }
  return List<List<T>>.unmodifiable(tables);
}

/// `western_aspects` holds a row a chart, or none when none was asked, and
/// `western_aspect_rows` is ragged by its count.
List<List<WesternAspectRow>> _decodeWesternAspects(Charts batch) {
  final r = batch.westernAspectRows;
  return _ragged(
    batch,
    batch.westernAspects.count,
    r.length,
    'western_aspects and western_aspect_rows',
    (at) => WesternAspectRow(
      first: Graha.byId(r.first[at]),
      second: Graha.byId(r.second[at]),
      aspect: WesternAspect.byId(r.aspect[at]),
      apartDeg: r.apartDeg[at],
      fromExactDeg: r.fromExactDeg[at],
      orbDeg: r.orbDeg[at],
      applying: r.applying[at] == 1,
    ),
  );
}

final Expando<(List<Declinations>, List<List<ParallelRow>>)> _declinations =
    Expando<(List<Declinations>, List<List<ParallelRow>>)>('declinations');

(List<Declinations>, List<List<ParallelRow>>) _declinationsOf(Charts batch) =>
    _declinations[batch] ??= _decodeDeclinations(batch);

/// `declinations` holds a row a chart, or none when none was asked, and
/// `declination_rows` and `parallel_rows` are ragged by its two counts.
(List<Declinations>, List<List<ParallelRow>>) _decodeDeclinations(
  Charts batch,
) {
  final row = batch.declinations;
  final rows = batch.declinationRows;
  final p = batch.parallelRows;
  final grahas = _ragged(
    batch,
    row.grahaCount,
    rows.length,
    'declinations and declination_rows',
    (at) => Declined(
      graha: Graha.byId(rows.graha[at]),
      declinationDeg: rows.declinationDeg[at],
    ),
  );
  return (
    List<Declinations>.unmodifiable([
      for (final (k, planets) in grahas.indexed)
        Declinations(
          obliquityDeg: row.obliquityDeg[k],
          grahas: planets,
          lagnaDeg: row.lagnaDeg[k],
          midheavenDeg: row.midheavenDeg[k],
        ),
    ]),
    _ragged(
      batch,
      row.parallelCount,
      p.length,
      'declinations and parallel_rows',
      (at) => ParallelRow(
        first: Graha.byId(p.first[at]),
        second: Graha.byId(p.second[at]),
        contrary: p.contrary[at] == 1,
        apartDeg: p.apartDeg[at],
        orbDeg: p.orbDeg[at],
      ),
    ),
  );
}

final Expando<List<List<SynastryRow>>> _synastries =
    Expando<List<List<SynastryRow>>>('synastries');

List<List<SynastryRow>> _synastriesOf(Charts batch) =>
    _synastries[batch] ??= _decodeSynastries(batch);

/// `synastry` holds a row a chart, or none when none was asked, and
/// `synastry_rows` is ragged by its count.
List<List<SynastryRow>> _decodeSynastries(Charts batch) {
  final r = batch.synastryRows;
  return _ragged(
    batch,
    batch.synastry.count,
    r.length,
    'synastry and synastry_rows',
    (at) => SynastryRow(
      first: _pointAt(r.firstLagna[at], r.firstGraha[at]),
      second: _pointAt(r.secondLagna[at], r.secondGraha[at]),
      aspect: WesternAspect.byId(r.aspect[at]),
      apartDeg: r.apartDeg[at],
      fromExactDeg: r.fromExactDeg[at],
      orbDeg: r.orbDeg[at],
    ),
  );
}

/// The pairs in antiscion read off their columns, a chart's own
/// (`antiscion_rows`) or across a synastry (`synastry_antiscion_rows`):
/// the two sections share columns.
AntiscionRow Function(int) _antiscionRows(
  Uint16List first,
  Uint16List second,
  Uint8List contrary,
  Float64List apartDeg,
  Float64List orbDeg,
) =>
    (at) => AntiscionRow(
      first: Graha.byId(first[at]),
      second: Graha.byId(second[at]),
      contrary: contrary[at] == 1,
      apartDeg: apartDeg[at],
      orbDeg: orbDeg[at],
    );

final Expando<List<List<AntiscionRow>>> _synastryAntiscia =
    Expando<List<List<AntiscionRow>>>('synastry antiscia');

List<List<AntiscionRow>> _synastryAntisciaOf(Charts batch) =>
    _synastryAntiscia[batch] ??= _decodeSynastryAntiscia(batch);

/// `synastry_antiscia` holds a row a chart, or none when none was asked,
/// and `synastry_antiscion_rows` is ragged by its count.
List<List<AntiscionRow>> _decodeSynastryAntiscia(Charts batch) {
  final r = batch.synastryAntiscionRows;
  return _ragged(
    batch,
    batch.synastryAntiscia.count,
    r.length,
    'synastry_antiscia and synastry_antiscion_rows',
    _antiscionRows(r.first, r.second, r.contrary, r.apartDeg, r.orbDeg),
  );
}

final Expando<List<List<MidpointRow>>> _midpoints =
    Expando<List<List<MidpointRow>>>('midpoints');

List<List<MidpointRow>> _midpointsOf(Charts batch) =>
    _midpoints[batch] ??= _decodeMidpoints(batch);

/// `midpoints` holds a row a chart, or none when none was asked, and
/// `midpoint_rows` is ragged by its count.
List<List<MidpointRow>> _decodeMidpoints(Charts batch) {
  final r = batch.midpointRows;
  return _ragged(
    batch,
    batch.midpoints.count,
    r.length,
    'midpoints and midpoint_rows',
    _midpointRows(
      r.first,
      r.second,
      r.middle,
      r.far,
      r.distanceDeg,
      r.fromAxisDeg,
      r.orbDeg,
    ),
  );
}

/// The equal distances read off their columns, a chart's own
/// (`midpoint_rows`) or across a synastry (`synastry_midpoint_rows`): the
/// two sections share these columns.
MidpointRow Function(int) _midpointRows(
  Uint16List first,
  Uint16List second,
  Uint16List middle,
  Uint8List far,
  Float64List distanceDeg,
  Float64List fromAxisDeg,
  Float64List orbDeg,
) =>
    (at) => MidpointRow(
      first: Graha.byId(first[at]),
      second: Graha.byId(second[at]),
      middle: Graha.byId(middle[at]),
      far: far[at] == 1,
      distanceDeg: distanceDeg[at],
      fromAxisDeg: fromAxisDeg[at],
      orbDeg: orbDeg[at],
    );

final Expando<List<List<SynastryMidpointRow>>> _synastryMidpoints =
    Expando<List<List<SynastryMidpointRow>>>('synastry midpoints');

List<List<SynastryMidpointRow>> _synastryMidpointsOf(Charts batch) =>
    _synastryMidpoints[batch] ??= _decodeSynastryMidpoints(batch);

/// `synastry_midpoints` holds a row a chart, or none when none was asked,
/// and `synastry_midpoint_rows` is ragged by its count.
List<List<SynastryMidpointRow>> _decodeSynastryMidpoints(Charts batch) {
  final r = batch.synastryMidpointRows;
  final row = _midpointRows(
    r.first,
    r.second,
    r.middle,
    r.far,
    r.distanceDeg,
    r.fromAxisDeg,
    r.orbDeg,
  );
  return _ragged(
    batch,
    batch.synastryMidpoints.count,
    r.length,
    'synastry_midpoints and synastry_midpoint_rows',
    (at) =>
        SynastryMidpointRow._of(row(at), partnersPair: r.partnersPair[at] == 1),
  );
}

final Expando<List<Composite>> _synastryComposites = Expando<List<Composite>>(
  'synastry composites',
);

List<Composite> _synastryCompositesOf(Charts batch) =>
    _synastryComposites[batch] ??= _decodeSynastryComposites(batch);

/// `synastry_composites` holds a row a chart, or none when none was asked,
/// and `synastry_composite_rows` is ragged by its count.
List<Composite> _decodeSynastryComposites(Charts batch) {
  final c = batch.synastryComposites;
  final r = batch.synastryCompositeRows;
  final planets = _ragged(
    batch,
    c.count,
    r.length,
    'synastry_composites and synastry_composite_rows',
    (at) => CompositePlanet(
      graha: Graha.byId(r.graha[at]),
      longitudeDeg: r.longitudeDeg[at],
      speedDegPerDay: r.speedDegPerDay[at],
    ),
  );
  final u = batch.synastryCompositeCusps;
  final cusps = _ragged(
    batch,
    c.cuspCount,
    u.length,
    'synastry_composites and synastry_composite_cusps',
    (at) => u.cuspDeg[at],
  );
  return List<Composite>.unmodifiable([
    for (final (k, rows) in planets.indexed)
      Composite(
        planets: rows,
        lagnaDeg: c.lagnaDeg[k],
        midheavenDeg: c.midheavenDeg[k],
        lagnaTurned: c.lagnaTurned[k] == 1,
        cuspsDeg: cusps[k].isEmpty ? null : cusps[k],
      ),
  ]);
}

final Expando<List<Partner>> _synastryDavisons = Expando<List<Partner>>(
  'synastry davisons',
);

List<Partner> _synastryDavisonsOf(Charts batch) =>
    _synastryDavisons[batch] ??= _decodeSynastryDavisons(batch);

/// `synastry_davisons` holds a Davison birth a chart, or none when none was
/// asked.
List<Partner> _decodeSynastryDavisons(Charts batch) {
  final b = batch.synastryDavisons;
  final charts = batch.cast.instant.length;
  if (b.length != 0 && b.length != charts) {
    throw StateError(
      'synastry_davisons has ${b.length} rows for $charts charts; it is one a chart or none',
    );
  }
  return List<Partner>.unmodifiable([
    for (var k = 0; k < b.length; k++)
      Partner(
        instant: b.instant[k],
        place: Observer(
          latitudeDeg: Latitude(b.latitudeDeg[k]),
          longitudeDeg: Longitude(b.longitudeDeg[k]),
          altitudeM: Altitude(b.altitudeM[k]),
        ),
        utcOffsetSeconds: b.utcOffsetSeconds[k],
      ),
  ]);
}

final Expando<List<Antiscia>> _antiscia = Expando<List<Antiscia>>('antiscia');

List<Antiscia> _antisciaOf(Charts batch) =>
    _antiscia[batch] ??= _decodeAntiscia(batch);

/// `antiscia` holds a row a chart, or none when none was asked, and
/// `antiscion_points` and `antiscion_rows` are ragged by its two counts.
List<Antiscia> _decodeAntiscia(Charts batch) {
  final row = batch.antiscia;
  final p = batch.antiscionPoints;
  final r = batch.antiscionRows;
  final points = _ragged(
    batch,
    row.pointCount,
    p.length,
    'antiscia and antiscion_points',
    (at) => at,
  );
  final pairs = _ragged(
    batch,
    row.pairCount,
    r.length,
    'antiscia and antiscion_rows',
    _antiscionRows(r.first, r.second, r.contrary, r.apartDeg, r.orbDeg),
  );
  final c = batch.antiscionCuspRows;
  final onCusps = _ragged(
    batch,
    row.cuspCount,
    c.length,
    'antiscia and antiscion_cusp_rows',
    (at) => CuspAntiscion(
      graha: Graha.byId(c.graha[at]),
      house: c.house[at],
      contrary: c.contrary[at] == 1,
    ),
  );
  return [
    for (final (k, rows) in points.indexed)
      Antiscia(
        points: [
          for (final at in rows)
            Antiscion(
              graha: Graha.byId(p.graha[at]),
              antiscionDeg: p.antiscionDeg[at],
              contrantiscionDeg: p.contrantiscionDeg[at],
            ),
        ],
        pairs: pairs[k],
        unpaired: [
          for (final at in rows)
            if (p.paired[at] == 0) Graha.byId(p.graha[at]),
        ],
        onCusps: onCusps[k],
        cuspSystem:
            row.cuspSystem[k] == _noHouseSystem
                ? null
                : HouseSystem.byId(row.cuspSystem[k]),
      ),
  ];
}

/// `antiscia.cusp_system` where no cusps were asked.
const int _noHouseSystem = 0xFFFF;

final Expando<List<AshtaKoota>> _matchings = Expando<List<AshtaKoota>>(
  'matchings',
);

List<AshtaKoota> _matchingsOf(Charts batch) =>
    _matchings[batch] ??= _matchingsIn(
      batch.matchings,
      batch.matchingKootas,
      batch.cast.instant.length,
    );

/// The Ashta Koota read from the `matchings` and `matching_kootas` shapes
/// of any blob carrying them, [charts] rows: a chart batch's, or a naam
/// blob's one match. `matchings` holds a row a match with what each koota
/// read, or none when none was asked, and `matching_kootas` eight rows a
/// match, each koota's points in the verse's order
/// (`03-design/matching.md`).
List<AshtaKoota> _matchingsIn(Matchings m, MatchingKootas k, int charts) {
  Map<Koota, KootaReading> readings(int at) {
    final dosha = BhakootDosha.byId(m.bhakootDosha[at]);
    final read = <KootaReading>[
      VarnaKoota(
        bride: Varna.byId(m.brideVarna[at]),
        groom: Varna.byId(m.groomVarna[at]),
      ),
      VashyaKoota(VashyaRelation.byId(m.vashya[at])),
      TaraKoota(
        brideToGroom: m.taraBrideToGroom[at],
        groomToBride: m.taraGroomToBride[at],
      ),
      YoniKoota(
        bride: Yoni.byId(m.brideYoni[at]),
        groom: Yoni.byId(m.groomYoni[at]),
        relation: YoniRelation.byId(m.yoni[at]),
      ),
      MaitriKoota(
        bride: Graha.byId(m.brideLord[at]),
        groom: Graha.byId(m.groomLord[at]),
        relation: MaitriRelation.byId(m.maitri[at]),
        lifted: m.maitriLifted[at] == 1,
      ),
      GanaKoota(
        bride: Gana.byId(m.brideGana[at]),
        groom: Gana.byId(m.groomGana[at]),
        dosha: m.ganaDosha[at] == 1,
        lifted: m.ganaLifted[at] == 1,
      ),
      BhakootKoota(
        apart: m.bhakootApart[at],
        dosha: dosha == BhakootDosha.none ? null : dosha,
        exceptions: BhakootExceptions(
          oneLord: m.bhakootOneLord[at] == 1,
          lordsFriends: m.bhakootLordsFriends[at] == 1,
          navamshaLordsFriends: m.bhakootNavamshaLordsFriends[at] == 1,
          taraPure: m.bhakootTaraPure[at] == 1,
          vashya: m.bhakootVashya[at] == 1,
        ),
        lifted: m.bhakootLifted[at] == 1,
      ),
      NadiKoota(
        bride: Nadi.byId(m.brideNadi[at]),
        groom: Nadi.byId(m.groomNadi[at]),
        dosha: m.nadiDosha[at] == 1,
        lifted: m.nadiLifted[at] == 1,
      ),
    ];
    return {for (final one in read) one.koota: one};
  }

  final kootas = _raggedIn(
    charts,
    List<int>.filled(m.total.length, 8),
    k.length,
    'matchings and matching_kootas',
    (row) => (Koota.byId(k.koota[row]), k.points[row], k.maxPoints[row]),
  );
  return List<AshtaKoota>.unmodifiable([
    for (final (at, rows) in kootas.indexed)
      AshtaKoota(
        kootas: List<KootaRow>.unmodifiable([
          for (final (koota, points, most) in rows)
            KootaRow(
              points: points,
              maxPoints: most,
              reading: readings(at)[koota]!,
            ),
        ]),
        total: m.total[at],
      ),
  ]);
}

final Expando<List<Porutham>> _poruthams = Expando<List<Porutham>>('poruthams');

List<Porutham> _poruthamsOf(Charts batch) =>
    _poruthams[batch] ??= _poruthamsIn(
      batch.poruthams,
      batch.poruthamRows,
      batch.cast.instant.length,
    );

/// The ten considerations read from the `poruthams` and `porutham_rows`
/// shapes of any blob carrying them, [charts] rows. `poruthams` holds a
/// row a match with what each read, or none when none was asked, and
/// `porutham_rows` ten rows a match, whether each agrees in the chapter's
/// order (`03-design/matching.md`).
List<Porutham> _poruthamsIn(Poruthams p, PoruthamRows r, int charts) {
  Map<Koota, PoruthamReading> readings(int at) {
    final read = <PoruthamReading>[
      DhinamPorutham(
        count: p.count[at],
        rule: DhinamRule.byId(p.dhinamRule[at]),
      ),
      GanamPorutham(
        bride: Gana.byId(p.brideGana[at]),
        groom: Gana.byId(p.groomGana[at]),
        diminished: p.ganaDiminished[at] == 1,
      ),
      MahendraPorutham(p.count[at]),
      DeerghaPorutham(p.count[at]),
      YoniPorutham(
        bride: Yoni.byId(p.brideYoni[at]),
        groom: Yoni.byId(p.groomYoni[at]),
        hostile: p.yoniHostile[at] == 1,
      ),
      RasiPorutham(p.apart[at]),
      RasyadhipathiPorutham(
        bride: Graha.byId(p.brideLord[at]),
        groom: Graha.byId(p.groomLord[at]),
        brideCallsFriend: p.brideCallsFriend[at] == 1,
        groomCallsFriend: p.groomCallsFriend[at] == 1,
      ),
      VasyamPorutham(
        brideToGroom: p.brideToGroom[at] == 1,
        groomToBride: p.groomToBride[at] == 1,
      ),
      RajjuPorutham(
        bride: Rajju.byId(p.brideRajju[at]),
        groom: Rajju.byId(p.groomRajju[at]),
      ),
      VedhaiPorutham(pierced: p.pierced[at] == 1),
    ];
    return {for (final one in read) one.koota: one};
  }

  final rows = _raggedIn(
    charts,
    List<int>.filled(p.agreeing.length, 10),
    r.length,
    'poruthams and porutham_rows',
    (row) => (Koota.byId(r.koota[row]), r.agrees[row] == 1, r.lifted[row] == 1),
  );
  return List<Porutham>.unmodifiable([
    for (final (at, ten) in rows.indexed)
      Porutham(
        considerations: List<PoruthamRow>.unmodifiable([
          for (final (koota, agrees, lifted) in ten)
            PoruthamRow(
              agrees: agrees,
              lifted: lifted,
              reading: readings(at)[koota]!,
            ),
        ]),
        agreeing: p.agreeing[at],
        chiefAgreeing: p.chiefAgreeing[at],
        exception: PoruthamException(
          oneLord: p.oneLord[at] == 1,
          lordsFriendly: p.lordsFriendly[at] == 1,
          opposite: p.opposite[at] == 1,
        ),
      ),
  ]);
}

final Expando<List<Kuja>> _kujas = Expando<List<Kuja>>('kujas');

List<Kuja> _kujasOf(Charts batch) => _kujas[batch] ??= _decodeKujas(batch);

/// `kujas` holds a row a chart, or none when none was asked
/// (`03-design/matching.md`).
List<Kuja> _decodeKujas(Charts batch) {
  final k = batch.kujas;
  KujaSide side(
    List<Uint8List> house,
    List<Uint8List> inHouses,
    Uint8List dosha,
    int at,
  ) => KujaSide(
    readings: List.unmodifiable([
      for (final (n, from) in KujaReference.values.indexed)
        KujaReading(
          from: from,
          house: house[n][at],
          inHouses: inHouses[n][at] == 1,
        ),
    ]),
    dosha: dosha[at] == 1,
  );
  return List.unmodifiable([
    for (var at = 0; at < k.both.length; at++)
      Kuja(
        bride: side(
          [k.brideLagnaHouse, k.brideMoonHouse, k.brideVenusHouse],
          [k.brideLagnaInHouses, k.brideMoonInHouses, k.brideVenusInHouses],
          k.brideDosha,
          at,
        ),
        groom: side(
          [k.groomLagnaHouse, k.groomMoonHouse, k.groomVenusHouse],
          [k.groomLagnaInHouses, k.groomMoonInHouses, k.groomVenusInHouses],
          k.groomDosha,
          at,
        ),
        both: k.both[at] == 1,
      ),
  ]);
}

final Expando<List<List<MarriageDosha>>> _marriageDoshas =
    Expando<List<List<MarriageDosha>>>('marriageDoshas');

List<List<MarriageDosha>> _marriageDoshasOf(Charts batch) =>
    _marriageDoshas[batch] ??= _decodeMarriageDoshas(batch);

/// `marriage_doshas` holds a count a chart, or none when none was asked,
/// and `marriage_dosha_rows` the entries ragged under it
/// (`03-design/matching.md`).
List<List<MarriageDosha>> _decodeMarriageDoshas(Charts batch) {
  final r = batch.marriageDoshaRows;
  return _ragged(
    batch,
    batch.marriageDoshas.count,
    r.system.length,
    'marriage_doshas and marriage_dosha_rows',
    (row) {
      final system = DoshaSystem.byId(r.system[row]);
      final kuja = system == DoshaSystem.kuja;
      return MarriageDosha(
        system: system,
        koota: kuja ? null : Koota.byId(r.koota[row]),
        side: kuja ? MatchRole.byId(r.side[row]) : null,
        lifted: r.lifted[row] == 1,
      );
    },
  );
}

final Expando<List<HarmonicChart>> _harmonics = Expando<List<HarmonicChart>>(
  'harmonics',
);

List<HarmonicChart> _harmonicsOf(Charts batch) =>
    _harmonics[batch] ??= _decodeHarmonics(batch);

/// A harmonic chart's point from its two cells: 0 and a graha's id for a
/// planet, 1 for the ascendant and 2 for the midheaven.
HarmonicPoint _harmonicPoint(int angle, int graha) => switch (angle) {
  1 => HarmonicPoint.ascendant,
  2 => HarmonicPoint.midheaven,
  _ => HarmonicGraha(Graha.byId(graha)),
};

/// `harmonics` holds a row a chart, or none when none was asked, and
/// `harmonic_points` and `harmonic_rows` are ragged by its two counts.
List<HarmonicChart> _decodeHarmonics(Charts batch) {
  final h = batch.harmonics;
  final p = batch.harmonicPoints;
  final r = batch.harmonicRows;
  final points = _ragged(
    batch,
    h.pointCount,
    p.length,
    'harmonics and harmonic_points',
    (at) => HarmonicPlaced(
      point: _harmonicPoint(p.angle[at], p.graha[at]),
      longitudeDeg: p.longitudeDeg[at],
      house: p.house[at],
    ),
  );
  final rows = _ragged(
    batch,
    h.rowCount,
    r.length,
    'harmonics and harmonic_rows',
    (at) => HarmonicRow(
      first: _harmonicPoint(r.firstAngle[at], r.firstGraha[at]),
      second: _harmonicPoint(r.secondAngle[at], r.secondGraha[at]),
      apartDeg: r.apartDeg[at],
      multiple: r.multiple[at],
      orbDeg: r.orbDeg[at],
    ),
  );
  return List<HarmonicChart>.unmodifiable([
    for (final (k, placed) in points.indexed)
      HarmonicChart(harmonic: h.number[k], points: placed, rows: rows[k]),
  ]);
}

final Expando<List<WesternHouses>> _westernHouses =
    Expando<List<WesternHouses>>('western houses');

List<WesternHouses> _westernHousesOf(Charts batch) =>
    _westernHouses[batch] ??= _decodeWesternHouses(batch);

/// `western_houses` holds a row a chart, or none when none was asked,
/// `western_house_cusps` twelve a chart, and `western_house_planets` is
/// ragged by its count.
List<WesternHouses> _decodeWesternHouses(Charts batch) {
  final h = batch.westernHouses;
  final u = batch.westernHouseCusps;
  final g = batch.westernHousePlanets;
  final cusps = _ragged(
    batch,
    List<int>.filled(h.length, 12),
    u.length,
    'western_houses and western_house_cusps',
    (at) => u.cuspDeg[at],
  );
  final planets = _ragged(
    batch,
    h.planetCount,
    g.length,
    'western_houses and western_house_planets',
    (at) => WesternHousePlacement(
      graha: Graha.byId(g.graha[at]),
      house: g.house[at],
      withAscendant: g.withAscendant[at] == 1,
    ),
  );
  return List<WesternHouses>.unmodifiable([
    for (final (k, rows) in planets.indexed)
      WesternHouses(
        system: HouseSystem.byId(h.system[k]),
        cuspsDeg: cusps[k],
        ascendantDeg: h.ascendantDeg[k],
        reachDeg: h.reachDeg[k],
        planets: rows,
      ),
  ]);
}

final Expando<List<List<SynastryParallelRow>>> _synastryParallels =
    Expando<List<List<SynastryParallelRow>>>('synastry parallels');

List<List<SynastryParallelRow>> _synastryParallelsOf(Charts batch) =>
    _synastryParallels[batch] ??= _decodeSynastryParallels(batch);

/// `synastry_parallels` holds a row a chart, or none when none was asked,
/// and `synastry_parallel_rows` is ragged by its count.
List<List<SynastryParallelRow>> _decodeSynastryParallels(Charts batch) {
  final p = batch.synastryParallelRows;
  return _ragged(
    batch,
    batch.synastryParallels.count,
    p.length,
    'synastry_parallels and synastry_parallel_rows',
    (at) => SynastryParallelRow(
      first: _pointAt(p.firstLagna[at], p.firstGraha[at]),
      second: _pointAt(p.secondLagna[at], p.secondGraha[at]),
      contrary: p.contrary[at] == 1,
      apartDeg: p.apartDeg[at],
      orbDeg: p.orbDeg[at],
    ),
  );
}

final Expando<List<Matter>> _perfections = Expando<List<Matter>>('perfections');

List<Matter> _perfectionsOf(Charts batch) =>
    _perfections[batch] ??= _decodePerfections(batch);

/// A dignity bit set, bit `n` the `n`th of [EssentialDignity]'s flags in
/// its order: house, exaltation, triplicity, term, face, detriment, fall.
EssentialDignity _dignityOf(int bits) {
  bool bit(int n) => (bits >> n) & 1 == 1;
  return EssentialDignity(
    house: bit(0),
    exaltation: bit(1),
    triplicity: bit(2),
    term: bit(3),
    face: bit(4),
    detriment: bit(5),
    fall: bit(6),
  );
}

/// `perfection` holds a row a chart, or none when none was asked; its
/// impediments, translations and collections are ragged by that row's
/// counts, and `perfection_orbs` holds seven a chart in the Chaldean order.
List<Matter> _decodePerfections(Charts batch) {
  final m = batch.perfection;
  final i = batch.perfectionImpediments;
  final t = batch.perfectionTranslations;
  final c = batch.perfectionCollections;
  final o = batch.perfectionOrbs;
  final charts = batch.cast.instant.length;
  if (m.length == 0) return const <Matter>[];
  if (m.length != charts || o.length != 7 * charts) {
    throw StateError(
      'perfection has ${m.length} rows and perfection_orbs ${o.length} for '
      '$charts charts; they are one and seven a chart',
    );
  }
  final impediments = _Starts._running(m.impedimentCount);
  final translations = _Starts._running(m.translationCount);
  final collections = _Starts._running(m.collectionCount);
  for (final (name, rows, counted) in [
    ('perfection_impediments', i.length, impediments.last),
    ('perfection_translations', t.length, translations.last),
    ('perfection_collections', c.length, collections.last),
  ]) {
    if (rows != counted) {
      throw StateError('$name has $rows rows and the charts count $counted');
    }
  }
  List<T> rows<T>(List<int> starts, int k, T Function(int) read) =>
      List<T>.unmodifiable([
        for (var at = starts[k]; at < starts[k + 1]; at += 1) read(at),
      ]);
  return List<Matter>.generate(charts, (k) {
    final horizonRule = m.horizonRuleDays[k];
    return Matter(
      querent: Graha.byId(m.querent[k]),
      quesited: Graha.byId(m.quesited[k]),
      application:
          m.applicationPresent[k] == 1
              ? Application(
                aspect: PtolemaicAspect.byId(m.applicationAspect[k]),
                days: m.applicationDays[k],
                applying: Graha.byId(m.applying[k]),
                kind: ApplicationKind.byId(m.applicationKind[k]),
                gapDeg: m.gapDeg[k],
                withinMoieties: m.withinMoieties[k] == 1,
              )
              : null,
      separation:
          m.separationPresent[k] == 1
              ? Separation(
                aspect: PtolemaicAspect.byId(m.separationAspect[k]),
                pastDeg: m.separationPastDeg[k],
              )
              : null,
      impediments: rows(
        impediments,
        k,
        (at) => Impediment(
          kind: ImpedimentKind.byId(i.kind[at]),
          significator: Graha.byId(i.significator[at]),
          third: i.thirdPresent[at] == 1 ? Graha.byId(i.third[at]) : null,
          aspect: PtolemaicAspect.byId(i.aspect[at]),
          days: i.days[at],
        ),
      ),
      translations: rows(
        translations,
        k,
        (at) => Translation(
          translator: Graha.byId(t.translator[at]),
          from: Graha.byId(t.from[at]),
          to: Graha.byId(t.to[at]),
          separating: Separation(
            aspect: PtolemaicAspect.byId(t.separatingAspect[at]),
            pastDeg: t.separatingPastDeg[at],
          ),
          aspect: PtolemaicAspect.byId(t.aspect[at]),
          days: t.days[at],
          received: _dignityOf(t.received[at]),
        ),
      ),
      collections: rows(
        collections,
        k,
        (at) => Collection(
          collector: Graha.byId(c.collector[at]),
          fromQuerent: ContactAhead(
            aspect: PtolemaicAspect.byId(c.fromQuerentAspect[at]),
            days: c.fromQuerentDays[at],
          ),
          fromQuesited: ContactAhead(
            aspect: PtolemaicAspect.byId(c.fromQuesitedAspect[at]),
            days: c.fromQuesitedDays[at],
          ),
          collectorInQuerent: _dignityOf(c.collectorInQuerent[at]),
          collectorInQuesited: _dignityOf(c.collectorInQuesited[at]),
          querentInCollector: _dignityOf(c.querentInCollector[at]),
          quesitedInCollector: _dignityOf(c.quesitedInCollector[at]),
        ),
      ),
      ways: Ways(
        querent: SignificatorPlace(
          planet: Graha.byId(m.querent[k]),
          house: m.querentHouse[k],
          dignity: _dignityOf(m.querentDignity[k]),
        ),
        quesited: SignificatorPlace(
          planet: Graha.byId(m.quesited[k]),
          house: m.quesitedHouse[k],
          dignity: _dignityOf(m.quesitedDignity[k]),
        ),
        mutualByHouse: m.mutualByHouse[k] == 1,
        infortunesBetween: List<Graha>.unmodifiable(
          _seven(m.infortunesBetween[k]),
        ),
        moonRelays: m.moonRelays[k] == 1,
        quesitedInAscendant: m.quesitedInAscendant[k] == 1,
        held: List<Way>.unmodifiable(
          _members<Way>(m.waysHeld[k], Way.values, (w) => w.id),
        ),
      ),
      horizonDays: m.horizonDays[k],
      rules: PerfectionRules(
        orbsDeg: List<double>.unmodifiable(o.orbDeg.sublist(7 * k, 7 * k + 7)),
        horizonDays: horizonRule.isNaN ? null : horizonRule,
        withinSign: m.withinSignRule[k] == 1,
      ),
    );
  });
}

final Expando<List<Fortitudes>> _fortitudes = Expando<List<Fortitudes>>(
  'fortitudes',
);

List<Fortitudes> _fortitudesOf(Charts batch) =>
    _fortitudes[batch] ??= _decodeFortitudes(batch);

/// `fortitudes` holds a row a chart, or none when none was asked,
/// `fortitude_houses` twelve a chart, `fortitude_planets` seven in the
/// Chaldean order, and `fortitude_accidents` each planet's lines, ragged by
/// its `accidentCount`. The essential half is the batch's dignities.
List<Fortitudes> _decodeFortitudes(Charts batch) {
  final c = batch.fortitudes;
  final h = batch.fortitudeHouses;
  final p = batch.fortitudePlanets;
  final a = batch.fortitudeAccidents;
  final charts = batch.cast.instant.length;
  if (c.length == 0) return const <Fortitudes>[];
  if (c.length != charts || h.length != 12 * charts || p.length != 7 * charts) {
    throw StateError(
      'fortitudes has ${c.length} rows, fortitude_houses ${h.length} and '
      'fortitude_planets ${p.length} for $charts charts; they are one, '
      'twelve and seven a chart',
    );
  }
  final starts = [0];
  for (var row = 0; row < p.length; row += 1) {
    starts.add(starts.last + p.accidentCount[row]);
  }
  if (starts.last != a.length) {
    throw StateError(
      'fortitude_accidents has ${a.length} rows and the planets count '
      '${starts.last}',
    );
  }
  final essential = _dignitiesOf(batch);
  List<T> rows<T>(int count, int chart, T Function(int row) at) =>
      List<T>.unmodifiable([
        for (var row = count * chart; row < count * chart + count; row += 1)
          at(row),
      ]);
  final lines = [
    c.scoreDirect,
    c.scoreRetrograde,
    c.scoreSwift,
    c.scoreSlow,
    c.scoreSuperiorOriental,
    c.scoreSuperiorOccidental,
    c.scoreInferiorOriental,
    c.scoreInferiorOccidental,
    c.scoreIncreasing,
    c.scoreDecreasing,
    c.scoreFreeFromCombustion,
    c.scoreCazimi,
    c.scoreCombust,
    c.scoreUnderBeams,
    c.scoreConjunctBenefic,
    c.scoreConjunctNorthNode,
    c.scoreTrineBenefic,
    c.scoreSextileBenefic,
    c.scoreConjunctMalefic,
    c.scoreConjunctSouthNode,
    c.scoreOpposedMalefic,
    c.scoreSquareMalefic,
    c.scoreBesieged,
    c.scoreRegulus,
    c.scoreSpica,
    c.scoreAlgol,
  ];
  // Each house's almuten totals, a column a planet in the Chaldean order.
  final houseAlmutens = [
    h.almutenSaturn,
    h.almutenJupiter,
    h.almutenMars,
    h.almutenSun,
    h.almutenVenus,
    h.almutenMercury,
    h.almutenMoon,
  ];
  PlanetAccidents planet(int chart, int row) {
    final own = essential[chart].planets[row - 7 * chart];
    return PlanetAccidents(
      planet: Graha.byId(p.planet[row]),
      house: p.house[row],
      accidents: List<AccidentLine>.unmodifiable([
        for (var at = starts[row]; at < starts[row + 1]; at += 1)
          AccidentLine(
            accident: Accident.byId(a.accident[at]),
            points: a.points[at],
          ),
      ]),
      fortitude: p.fortitude[row],
      debility: p.debility[row],
      net: own.score + own.reception + p.fortitude[row] - p.debility[row],
    );
  }

  return List<Fortitudes>.generate(charts, (chart) {
    final [
      direct,
      retrograde,
      swift,
      slow,
      superiorOriental,
      superiorOccidental,
      inferiorOriental,
      inferiorOccidental,
      increasing,
      decreasing,
      freeFromCombustion,
      cazimi,
      combust,
      underBeams,
      conjunctBenefic,
      conjunctNorthNode,
      trineBenefic,
      sextileBenefic,
      conjunctMalefic,
      conjunctSouthNode,
      opposedMalefic,
      squareMalefic,
      besieged,
      regulus,
      spica,
      algol,
    ] = [for (final line in lines) line[chart]];
    final planets = rows(7, chart, (row) => planet(chart, row));
    final seven = [for (final at in planets) at.planet];
    return Fortitudes(
      dignities: essential[chart],
      sky: AccidentalSky(
        houses: HouseSystem.byId(c.houses[chart]),
        cuspsDeg: rows(12, chart, (row) => h.cusp[row]),
        ascendantDeg: c.ascendant[chart],
        midheavenDeg: c.midheaven[chart],
        speedsDegPerDay: rows(7, chart, (row) => p.speed[row]),
        northNodeDeg: c.northNode[chart],
        regulusDeg: c.regulus[chart],
        spicaDeg: c.spica[chart],
        algolDeg: c.algol[chart],
      ),
      rules: AccidentalRules(
        combustionDeg: c.combustionOrb[chart],
        combustionInSign: c.combustionInSign[chart] == 1,
        beamsDeg: c.beamsOrb[chart],
        cazimiDeg: c.cazimiOrb[chart],
        cuspOrbDeg: c.cuspOrb[chart],
        starOrbDeg: c.starOrb[chart],
        partile: Partile.byId(c.partile[chart]),
        partileOrbDeg: c.partileOrb[chart],
        siege: Siege.byId(c.siege[chart]),
        siegeSpanDeg: c.siegeSpan[chart],
        meanMotionDeg: rows(7, chart, (row) => p.meanMotion[row]),
      ),
      scores: AccidentalScores(
        houses: rows(12, chart, (row) => h.score[row]),
        direct: direct,
        retrograde: retrograde,
        swift: swift,
        slow: slow,
        superiorOriental: superiorOriental,
        superiorOccidental: superiorOccidental,
        inferiorOriental: inferiorOriental,
        inferiorOccidental: inferiorOccidental,
        increasing: increasing,
        decreasing: decreasing,
        freeFromCombustion: freeFromCombustion,
        cazimi: cazimi,
        combust: combust,
        underBeams: underBeams,
        conjunctBenefic: conjunctBenefic,
        conjunctNorthNode: conjunctNorthNode,
        trineBenefic: trineBenefic,
        sextileBenefic: sextileBenefic,
        conjunctMalefic: conjunctMalefic,
        conjunctSouthNode: conjunctSouthNode,
        opposedMalefic: opposedMalefic,
        squareMalefic: squareMalefic,
        besieged: besieged,
        regulus: regulus,
        spica: spica,
        algol: algol,
      ),
      planets: planets,
      almutens: Almutens(
        rules: AlmutenRules(
          place: PlaceReading.byId(c.almutenPlace[chart]),
          fortune: FortuneRule.byId(c.almutenFortune[chart]),
        ),
        fortuneDeg: c.fortune[chart],
        figure: Almuten._of(seven, [for (final at in planets) at.net]),
        places: Almuten._of(seven, rows(7, chart, (row) => p.places[row])),
        houses: rows(
          12,
          chart,
          (row) => Almuten._of(seven, [
            for (final column in houseAlmutens) column[row],
          ]),
        ),
      ),
    );
  });
}

final Expando<List<SadeSatiReport>> _sadeSatis = Expando<List<SadeSatiReport>>(
  'sadeSatis',
);

List<SadeSatiReport> _sadeSatisOf(Charts batch) =>
    _sadeSatis[batch] ??= _decodeSadeSatis(batch);

/// `sade_sati` holds a row a chart, or none when none was asked, and
/// `sade_sati_visits` each chart's visits, ragged by
/// `cast.sade_sati_visit_count` and numbered by `period`: its Sade Satis
/// first (houses 12, 1 and 2), then its smaller spells.
List<SadeSatiReport> _decodeSadeSatis(Charts batch) {
  final c = batch.sadeSati;
  final v = batch.sadeSatiVisits;
  final counts = batch.cast.sadeSatiVisitCount;
  if (c.length == 0) return const <SadeSatiReport>[];
  final total = counts.fold<int>(0, (sum, count) => sum + count);
  if (c.length != counts.length || total != v.length) {
    throw StateError(
      'sade_sati has ${c.length} rows and sade_sati_visits ${v.length} over '
      '${counts.length} charts whose counts sum to $total; it is a row a '
      "chart and every chart's visits",
    );
  }
  double? bound(double jd) => jd.isNaN ? null : jd;
  // The Sade Sati's houses are 12, 1 and 2; a smaller spell is 3 to 11.
  bool isSadeSati(List<SadeSatiSpell> spells) =>
      spells.first.house == 12 || spells.first.house <= 2;
  var start = 0;
  return List<SadeSatiReport>.generate(c.length, (chart) {
    // A period's rows are adjacent and share `period`; a spell's are the
    // run of one house inside it.
    final periods = <List<(int, List<SadeSatiVisit>)>>[];
    for (var row = start; row < start + counts[chart]; row += 1) {
      if (v.period[row] == periods.length) periods.add([]);
      final spells = periods[v.period[row]];
      if (spells.isEmpty || spells.last.$1 != v.house[row]) {
        spells.add((v.house[row], <SadeSatiVisit>[]));
      }
      spells.last.$2.add(
        SadeSatiVisit(from: bound(v.from[row]), to: bound(v.to[row])),
      );
    }
    start += counts[chart];
    final built = [
      for (final spells in periods)
        List<SadeSatiSpell>.unmodifiable([
          for (final (house, visits) in spells)
            SadeSatiSpell(
              house: house,
              visits: List<SadeSatiVisit>.unmodifiable(visits),
            ),
        ]),
    ];
    return SadeSatiReport(
      reference: GocharReference(
        from: GocharFrom.byId(c.countedFrom[chart]),
        sign: Rashi.byId(c.reference[chart]),
      ),
      reckoning: Reckoning.byId(c.reckoning[chart]),
      sadeSati: List<SadeSati>.unmodifiable([
        for (final spells in built)
          if (isSadeSati(spells)) SadeSati(phases: spells),
      ]),
      spells: List<SadeSatiSpell>.unmodifiable([
        for (final spells in built)
          if (!isSadeSati(spells)) spells.first,
      ]),
    );
  });
}

final Expando<List<List<GocharReading>>> _gochars =
    Expando<List<List<GocharReading>>>('gochars');

List<List<GocharReading>> _gocharsOf(Charts batch) =>
    _gochars[batch] ??= _decodeGochars(batch);

/// Fixed rather than ragged: the request settles how many instants every
/// chart gets, so each chart holds the section's rows over the chart count.
List<List<GocharReading>> _decodeGochars(Charts batch) {
  final c = batch.gochar;
  final g = batch.gocharGrahas;
  final a = batch.gocharAshtakavarga;
  final charts = batch.chartCount;
  final perChart = charts == 0 ? 0 : c.length ~/ charts;
  if (perChart * charts != c.length || g.length != c.length * 9) {
    throw StateError(
      'gochar has ${c.length} rows and ${g.length} grahas over $charts '
      'charts; it is every chart at every instant, nine grahas each',
    );
  }
  final judged = a.length > 0;
  if (judged && a.length != c.length * 7) {
    throw StateError(
      'gochar_ashtakavarga has ${a.length} rows under ${c.length} transits; '
      'it is seven under every one or none',
    );
  }
  List<AshtakavargaTransit> byBindus(int row) =>
      List<AshtakavargaTransit>.unmodifiable(
        List<AshtakavargaTransit>.generate(7, (k) {
          final at = row * 7 + k;
          return AshtakavargaTransit(
            graha: Graha.byId(a.graha[at]),
            bindus: a.bindus[at],
            good: a.good[at] == 1,
            kakshya: Kakshya(
              index: a.kakshya[at],
              lord: KakshyaLord.byId(a.kakshyaLord[at]),
            ),
            kakshyaBindu: a.kakshyaBindu[at] == 1,
            sarva: a.sarva[at],
            sarvaStanding: SarvaStanding.byId(a.sarvaStanding[at]),
          );
        }),
      );
  GrahaGochar graha(int at) => GrahaGochar(
    graha: Graha.byId(g.graha[at]),
    transit: Transit(sign: Rashi.byId(g.sign[at]), degrees: g.degrees[at]),
    house: g.house[at],
    goodHouse: g.goodHouse[at] == 1,
    vedhaHouse: g.vedhaHouse[at] == 0 ? null : g.vedhaHouse[at],
    obstructedBy: List<Graha>.unmodifiable(_nine(g.obstructedBy[at])),
    verdict: GocharVerdict.byId(g.verdict[at]),
    fruition: Fruition.byId(g.fruition[at]),
    fruitfulNow: g.fruitfulNow[at] == 1,
  );
  GocharReading reading(int row) => GocharReading(
    instant: c.instant[row],
    reference: GocharReference(
      from: GocharFrom.byId(c.countedFrom[row]),
      sign: Rashi.byId(c.reference[row]),
    ),
    rules: GocharRules(
      nodeVedha: NodeVedha.byId(c.nodeVedha[row]),
      nodeObstruction: NodeObstruction.byId(c.nodeObstruction[row]),
      ashtakavargaGoodFrom: AshtakavargaGoodFrom.byId(
        c.ashtakavargaGoodFrom[row],
      ),
    ),
    ashtakavarga: judged ? byBindus(row) : null,
    grahas: List<GrahaGochar>.unmodifiable(
      List<GrahaGochar>.generate(9, (k) => graha(row * 9 + k)),
    ),
  );
  return List<List<GocharReading>>.generate(
    charts,
    (chart) => List<GocharReading>.unmodifiable(
      List<GocharReading>.generate(
        perChart,
        (k) => reading(chart * perChart + k),
      ),
    ),
    growable: false,
  );
}

/// Each batch's Jaimini significators, decoded once however many charts read
/// them.
final Expando<List<JaiminiReading>> _jaiminis = Expando<List<JaiminiReading>>(
  'jaiminis',
);

List<JaiminiReading> _jaiminisOf(Charts batch) =>
    _jaiminis[batch] ??= _decodeJaiminis(batch);

List<JaiminiReading> _decodeJaiminis(Charts batch) {
  final c = batch.jaimini;
  final h = batch.jaiminiGrahas;
  List<int> houses(List<int> column, int chart) =>
      List<int>.unmodifiable(column.sublist(chart * 9, chart * 9 + 9));
  return List<JaiminiReading>.generate(c.length, (chart) {
    final outcome = BrahmaOutcome.byId(c.brahmaOutcome[chart]);
    final found = outcome == BrahmaOutcome.found;
    return JaiminiReading(
      karakamsha: Karakamsha(
        atmakaraka: Graha.byId(c.atmakaraka[chart]),
        sign: Rashi.byId(c.karakamsha[chart]),
        inRasi: houses(h.inRasi, chart),
        inNavamsha: houses(h.inNavamsha, chart),
      ),
      brahma: Brahma(
        rule: BrahmaRule.byId(c.brahmaRule[chart]),
        countedFrom: Rashi.byId(c.countedFrom[chart]),
        qualified: List<Graha>.unmodifiable(_nine(c.qualified[chart])),
        graha: found ? Graha.byId(c.brahma[chart]) : null,
        passedFrom:
            c.passedFromPresent[chart] == 1
                ? Graha.byId(c.passedFrom[chart])
                : null,
        none: found ? null : outcome,
      ),
      grahaArudhas: List<Rashi?>.unmodifiable([
        for (var row = chart * 9; row < chart * 9 + 9; row++)
          h.arudhaPresent[row] == 1 ? Rashi.byId(h.arudha[row]) : null,
      ]),
    );
  }, growable: false);
}

/// Each batch's avakahadas, decoded once however many charts read them.
final Expando<List<Avakahada>> _avakahadas = Expando<List<Avakahada>>(
  'avakahadas',
);

List<Avakahada> _avakahadasOf(Charts batch) =>
    _avakahadas[batch] ??= _decodeAvakahadas(batch);

List<Avakahada> _decodeAvakahadas(Charts batch) {
  final c = batch.avakahada;
  final syllables =
      batch.avakahadaSyllables.isEmpty
          ? const <Object?>[]
          : jsonDecode(batch.avakahadaSyllables) as List<Object?>;
  return List<Avakahada>.generate(c.length, (chart) {
    final text = syllables[chart] as List<Object?>;
    return Avakahada(
      nakshatra: Nakshatra.byId(c.nakshatra[chart]),
      pada: c.pada[chart],
      rashi: Rashi.byId(c.rashi[chart]),
      nakshatraLord: Graha.byId(c.nakshatraLord[chart]),
      rashiLord: Graha.byId(c.rashiLord[chart]),
      varna: Varna.byId(c.varna[chart]),
      yoni: Yoni.byId(c.yoni[chart]),
      gana: Gana.byId(c.gana[chart]),
      nadi: Nadi.byId(c.nadi[chart]),
      syllable: BirthSyllable(
        cell: c.cell[chart],
        devanagari: text[0]! as String,
        iast: text[1]! as String,
        varga: NameVarga.byId(c.varga[chart]),
      ),
    );
  }, growable: false);
}

/// Each batch's Vaiseshikamsas, decoded once however many charts read them.
final Expando<List<VaiseshikamsaReading>> _vaiseshikamsas =
    Expando<List<VaiseshikamsaReading>>('vaiseshikamsas');

List<VaiseshikamsaReading> _vaiseshikamsasOf(Charts batch) =>
    _vaiseshikamsas[batch] ??= _decodeVaiseshikamsas(batch);

List<VaiseshikamsaReading> _decodeVaiseshikamsas(Charts batch) {
  final c = batch.vaiseshikamsa;
  VaiseshikamsaStanding standing(List<int> good, List<int> names, int row) =>
      VaiseshikamsaStanding(
        goodVargas: good[row],
        name: good[row] >= 2 ? Vaiseshikamsa.byId(names[row]) : null,
      );
  return List<VaiseshikamsaReading>.generate(
    c.length ~/ 7,
    (chart) => VaiseshikamsaReading(
      grahas: List<GrahaVaiseshikamsa>.generate(7, (g) {
        final row = chart * 7 + g;
        return GrahaVaiseshikamsa(
          graha: Graha.byId(c.graha[row]),
          shadvarga: standing(c.shadvargaGood, c.shadvargaName, row),
          saptavarga: standing(c.saptavargaGood, c.saptavargaName, row),
          dashavarga: standing(c.dashavargaGood, c.dashavargaName, row),
          shodashavarga: standing(
            c.shodashavargaGood,
            c.shodashavargaName,
            row,
          ),
          impaired: c.impaired[row] == 1,
        );
      }, growable: false),
    ),
    growable: false,
  );
}

/// Each batch's Vimshopakas, decoded once however many charts read them.
final Expando<List<Vimshopaka>> _vimshopakas = Expando<List<Vimshopaka>>(
  'vimshopakas',
);

List<Vimshopaka> _vimshopakasOf(Charts batch) =>
    _vimshopakas[batch] ??= _decodeVimshopakas(batch);

List<Vimshopaka> _decodeVimshopakas(Charts batch) {
  final rows = batch.vimshopaka;
  return List<Vimshopaka>.generate(
    rows.length ~/ 7,
    (chart) => Vimshopaka(
      scoring: VimshopakaScoring.byId(rows.scoring[chart * 7]),
      grahas: List<GrahaVimshopaka>.generate(7, (g) {
        final row = chart * 7 + g;
        return GrahaVimshopaka(
          graha: Graha.byId(rows.graha[row]),
          shadvarga: rows.shadvarga[row],
          saptavarga: rows.saptavarga[row],
          dashavarga: rows.dashavarga[row],
          shodashavarga: rows.shodashavarga[row],
        );
      }, growable: false),
    ),
    growable: false,
  );
}

/// Each batch's dashas, decoded once however many charts read them.
final Expando<List<List<Dasha>>> _dashas = Expando<List<List<Dasha>>>('dashas');

List<List<Dasha>> _dashasOf(Charts batch) =>
    _dashas[batch] ??= _decodeDashas(batch);

/// Every chart's dashas: the periods are **ragged** by each dasha's
/// `periodCount`, so a chart's begin where the one before it ends.
List<List<Dasha>> _decodeDashas(Charts batch) {
  final per = batch.dashaCount;
  if (per == 0) return const <List<Dasha>>[];
  var start = 0;
  return List<List<Dasha>>.generate(
    batch.dashas.length ~/ per,
    (chart) => List<Dasha>.generate(per, (j) {
      final row = chart * per + j;
      final count = batch.dashas.periodCount[row];
      final dasha = _dashaOf(batch, row, start, count);
      start += count;
      return dasha;
    }),
  );
}

/// The full key of each dasha system a batch's context registered, by its id.
final Expando<Map<int, String>> _dashaNames = Expando<Map<int, String>>(
  'dashaNames',
);

/// A batch, remembering the names of the dasha systems its context
/// registered so a registered id reads as its key.
Charts _named(Charts batch, Map<String, int> registered) {
  if (registered.isNotEmpty) {
    _dashaNames[batch] = {for (final e in registered.entries) e.value: e.key};
  }
  return batch;
}

/// A dasha row's system: the catalogue's member, or a registered one.
KeyOf<DashaSystem> _dashaSystem(Charts batch, int id) {
  final full = _dashaNames[batch]?[id];
  return full == null
      ? DashaSystem.byId(id)
      : DashaSystem.registered(full.substring('dasha_system.'.length));
}

/// The dashas asked for, as the ids the boundary takes: a [DashaSystem], or a
/// system this context registered (`03-design/dasha-kernels.md`).
List<int> _dashaIds(
  List<KeyOf<DashaSystem>> dashas,
  Map<String, int> registered,
) => [
  for (final (index, system) in dashas.indexed)
    switch (system) {
      DashaSystem(:final id) => id,
      _ =>
        registered[system.fullKey] ??
            (throw ArgumentError.value(
              system.fullKey,
              'dashas[$index]',
              'not a dasha system this context registered',
            )),
    },
];

/// One dasha row and its periods, in this layer's shape. A period's path is
/// its index below the nearest earlier period one level up.
Dasha _dashaOf(Charts batch, int row, int start, int count) {
  final d = batch.dashas;
  final p = batch.dashaPeriods;
  final seeded = d.seeded[row] != 0;
  final signed = d.signed[row] != 0;
  final periods = _periodsOf(
    level: p.level,
    index: p.index,
    sign: p.sign,
    lord: p.lord,
    from: p.fromJd,
    to: p.toJd,
    start: start,
    count: count,
    signed: (_) => signed,
  );
  final spanFrom = d.moonSpanFrom[row];
  return Dasha(
    system: _dashaSystem(batch, d.system[row]),
    seed: seeded ? Nakshatra.byId(d.seed[row]) : null,
    firstLord: Graha.byId(d.firstLord[row]),
    overflow: d.overflow[row] != 0,
    balance:
        seeded
            ? DashaBalance(
              method: Balance.byId(d.balance[row]),
              remaining: d.remaining[row],
              days: d.balanceDays[row],
              written: WrittenBalance(
                years: d.balanceYears[row],
                months: d.balanceMonths[row],
                days: d.balanceDayCount[row],
                hours: d.balanceHours[row],
                minutes: d.balanceMinutes[row],
              ),
            )
            : null,
    moonSpan:
        spanFrom.isNaN ? null : Interval(from: spanFrom, to: d.moonSpanTo[row]),
    depth: d.depth[row],
    periods: periods,
  );
}

/// Each batch's answers by rule, parsed once however many charts read them.
final Expando<List<Map<String, Object?>>> _rules =
    Expando<List<Map<String, Object?>>>('rules');

List<Map<String, Object?>> _rulesOf(Charts batch) =>
    _rules[batch] ??= _sectionOf(batch.rules);

/// Each batch's narrative plans, parsed once however many charts read them.
final Expando<List<Map<String, Object?>>> _plans =
    Expando<List<Map<String, Object?>>>('plans');

List<Map<String, Object?>> _plansOf(Charts batch) =>
    _plans[batch] ??= _sectionOf(batch.plans);

/// A blob section of canonical JSON, one object a chart; empty where the
/// request did not ask for the section.
List<Map<String, Object?>> _sectionOf(String json) =>
    json.isEmpty
        ? const <Map<String, Object?>>[]
        : [
          for (final chart in jsonDecode(json) as List<Object?>)
            chart! as Map<String, Object?>,
        ];

/// A set of rules the SDK ships.
enum ShippedRules {
  /// The recording engine's doshas the SDK computes.
  doshas('DOSHAS'),

  /// The recording engine's yogas the SDK computes.
  yogas('YOGAS'),

  /// The gandantas of BPHS ch. 92.
  gandantas('GANDANTAS'),

  /// The evils at birth and their cancellations.
  arishtas('ARISHTAS'),

  /// The generated readings of a graha, a pair and a rising part.
  readings('READINGS'),

  /// The yogas, doshas and classes of life written from the texts.
  nabhasas('NABHASAS');

  const ShippedRules(this.key);

  /// The key a rule request names it with.
  final String key;
}

/// The readings a request evaluates rules under.
enum RuleReadings {
  /// The texts' readings wherever a text settles one.
  texts('TEXTS'),

  /// The recording engine's.
  recordingEngine('RECORDING_ENGINE');

  const RuleReadings(this.key);

  /// The key a rule request names it with.
  final String key;
}

/// The rules a request asks a chart to answer
/// (`03-design/rules-at-the-boundary.md`): shipped sets by name and a
/// consumer's own rules in the SDK's rule format.
final class RuleRequest {
  /// A request for [shipped] sets and a consumer's own [rules].
  const RuleRequest({
    this.shipped = const <ShippedRules>[],
    this.rules = const <Map<String, Object?>>[],
    this.readings = RuleReadings.texts,
    this.houses = false,
    this.longevity = false,
  });

  /// The shipped sets to evaluate.
  final List<ShippedRules> shipped;

  /// A consumer's own rules, which may name shipped rules by key.
  final List<Map<String, Object?>> rules;

  /// The readings to evaluate under.
  final RuleReadings readings;

  /// Whether to add the twelve house readings.
  final bool houses;

  /// Whether to add the three pairs, the three spans and the marakas.
  final bool longevity;

  String get _json => jsonEncode(<String, Object?>{
    'shipped': [for (final set in shipped) set.key],
    'rules': rules,
    'readings': readings.key,
    'houses': houses,
    'longevity': longevity,
  });
}

/// The narrative plans a request asks a chart for
/// (`03-design/plans-at-the-boundary.md`). Each composer is off by default,
/// and [readings] needs a [RuleRequest] beside it, since it says what the
/// rules a chart held answered.
/// Which longitude an annual chart's Sun returns to
/// (`03-design/annual-chart.md`).
enum VarshaReading {
  /// The natal sidereal longitude, read on the chart's own ayanamsha
  /// basis: the tradition's, and the default.
  sidereal('SIDEREAL'),

  /// The natal tropical longitude: the Western solar return. Forty years
  /// on it is most of a circle of lagna from the sidereal one, so it is a
  /// choice and never a fallback.
  tropical('TROPICAL'),

  /// A whole sidereal year for each year of life, from birth: the older
  /// arithmetic, and the only reading that needs no ephemeris.
  mean('MEAN');

  const VarshaReading(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Sade Sati and Saturn's smaller spells to find for every chart of a
/// request (`03-design/sade-sati.md`): the window, and optionally what the
/// houses are counted from (the natal Moon by default; C139), what they are
/// reckoned in (whole signs by default; C147) and the smaller spells (the
/// 4th and the 8th by default; C149). Every period reaching into the window
/// comes back whole; Saturn is searched once for the whole batch.
///
/// ```dart
/// final chart = ctx.chart.found(
///   /* … */ sadeSati: const SadeSatiRequest(from: 2460676.5, to: 2464329),
/// );
/// final peak = chart.sadeSati?.sadeSati.firstOrNull?.phases[1];
/// ```
final class SadeSatiRequest {
  const SadeSatiRequest({
    required this.from,
    this.to,
    this.countedFrom = GocharFrom.moon,
    this.reckoning = Reckoning.sign,
    this.spells,
  });

  /// The window's start, a UTC Julian day.
  final double from;

  /// The window's end, not before the start, or the SDK refuses it by
  /// `sadeSati.to`; [from] when absent, one instant.
  final double? to;

  /// The natal point the houses are counted from.
  final GocharFrom countedFrom;

  /// What the houses are reckoned in.
  final Reckoning reckoning;

  /// The smaller spells, houses 3 to 11 each named once; the 4th and the
  /// 8th when absent.
  final List<int>? spells;

  String get _json => jsonEncode(<String, Object?>{
    'from': from,
    if (to case final to?) 'to': to,
    'countedFrom': countedFrom.key,
    'reckoning': reckoning.key,
    if (spells case final spells?) 'spells': spells,
  });
}

/// One stay of Saturn's in a house, half-open.
final class SadeSatiVisit {
  const SadeSatiVisit({required this.from, required this.to});

  /// When Saturn entered, a UTC Julian day; null before the ephemeris's
  /// coverage.
  final double? from;

  /// When it left; null after the ephemeris's coverage.
  final double? to;

  @override
  bool operator ==(Object other) =>
      other is SadeSatiVisit && other.from == from && other.to == to;

  @override
  int get hashCode => Object.hash(from, to);
}

/// Every stay of Saturn's in one house of one period, a retrograde re-entry
/// a visit of its own (C148).
final class SadeSatiSpell {
  const SadeSatiSpell({required this.house, required this.visits});

  /// The house from the reference, 1 to 12.
  final int house;

  final List<SadeSatiVisit> visits;

  @override
  bool operator ==(Object other) =>
      other is SadeSatiSpell &&
      other.house == house &&
      _sameList(other.visits, visits);

  @override
  int get hashCode => Object.hash(house, Object.hashAll(visits));
}

/// One Sade Sati: the rising (12th), peak (1st) and setting (2nd) spells, in
/// order.
final class SadeSati {
  const SadeSati({required this.phases});

  final List<SadeSatiSpell> phases;

  @override
  bool operator ==(Object other) =>
      other is SadeSati && _sameList(other.phases, phases);

  @override
  int get hashCode => Object.hashAll(phases);
}

/// A chart's Sade Satis and smaller spells, each period **whole** however far
/// its bounds fall outside the window asked about.
final class SadeSatiReport {
  const SadeSatiReport({
    required this.reference,
    required this.reckoning,
    required this.sadeSati,
    required this.spells,
  });

  /// What the houses were counted from, and that point's sign.
  final GocharReference reference;

  final Reckoning reckoning;

  /// Every Sade Sati reaching into the window, in time order.
  final List<SadeSati> sadeSati;

  /// The smaller spells asked for, in time order.
  final List<SadeSatiSpell> spells;

  @override
  bool operator ==(Object other) =>
      other is SadeSatiReport &&
      other.reference.from == reference.from &&
      other.reference.sign == reference.sign &&
      other.reckoning == reckoning &&
      _sameList(other.sadeSati, sadeSati) &&
      _sameList(other.spells, spells);

  @override
  int get hashCode => Object.hash(
    reference.from,
    reference.sign,
    reckoning,
    Object.hashAll(sadeSati),
    Object.hashAll(spells),
  );
}

/// Whether two lists hold equal members in the same order.
bool _sameList<T>(List<T> a, List<T> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i += 1) {
    final (x, y) = (a[i], b[i]);
    if (x is List && y is List ? !_sameList(x, y) : x != y) return false;
  }
  return true;
}

/// A value compared field by field, lists by their members: two are equal
/// when they are the same type and every field is.
abstract base class _Value {
  const _Value();

  /// The fields that make the value, in declaration order.
  List<Object?> get _fields;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is _Value &&
          other.runtimeType == runtimeType &&
          _sameList(other._fields, _fields));

  @override
  int get hashCode => Object.hashAll(
    _fields.map((field) => field is List ? Object.hashAll(field) : field),
  );
}

/// One term of a table of the caller's own: its lord, and the degree within
/// the sign it ends at, exclusive.
final class Term {
  const Term(this.lord, this.end);

  final Graha lord;
  final int end;

  Map<String, Object?> get _json => {'lord': lord.key, 'end': end};
}

/// What each dignity and debility is worth; every one left out is Lilly's
/// (p. 115), so `DignityScores(peregrine: 0)` changes that one alone.
final class DignityScores extends _Value {
  const DignityScores({
    this.house = 5,
    this.exaltation = 4,
    this.triplicity = 3,
    this.term = 2,
    this.face = 1,
    this.detriment = -5,
    this.fall = -4,
    this.peregrine = -5,
  });

  /// Lilly's "ready Table" (p. 115).
  static const DignityScores lilly = DignityScores();

  final int house;
  final int exaltation;
  final int triplicity;
  final int term;
  final int face;
  final int detriment;
  final int fall;

  /// In none of its five dignities.
  final int peregrine;

  @override
  List<Object?> get _fields => [
    house,
    exaltation,
    triplicity,
    term,
    face,
    detriment,
    fall,
    peregrine,
  ];

  Map<String, Object?> get _json => {
    'house': house,
    'exaltation': exaltation,
    'triplicity': triplicity,
    'term': term,
    'face': face,
    'detriment': detriment,
    'fall': fall,
    'peregrine': peregrine,
  };
}

/// How to read every chart's essential dignities
/// (`03-design/essential-dignities.md`). Each chart's come back as its
/// `dignities`, every rule applied reported beside them.
///
/// The sect is the Sun's centre above the true horizon by default,
/// Valens's hemisphere (C209), and the rest is Lilly's. A [table] of the
/// caller's own, twelve signs of five [Term]s from Aries, stands in for
/// [terms]; a malformed one is refused by `dignities.rules.terms.TABLE`.
///
/// ```dart
/// final chart = ctx.chart.found(
///   /* … */ dignities: const DignityRequest(terms: Terms.egyptian),
/// );
/// final sect = chart.dignities?.sect;
/// ```
final class DignityRequest {
  const DignityRequest({
    this.sectRule = SectRule.horizon,
    this.terms = Terms.ptolemaicLilly,
    this.table,
    this.triplicities = Triplicities.lilly,
    this.scores = DignityScores.lilly,
  });

  final SectRule sectRule;

  /// The system of terms, unless [table] gives one.
  final Terms terms;

  /// A table of terms of the caller's own, which [terms] then does not
  /// name.
  final List<List<Term>>? table;

  final Triplicities triplicities;
  final DignityScores scores;

  String get _json => jsonEncode(_record);

  Map<String, Object?> get _record => <String, Object?>{
    'sectRule': sectRule.key,
    'rules': {
      'terms': switch (table) {
        final table? => {
          'TABLE': [
            for (final sign in table) [for (final term in sign) term._json],
          ],
        },
        null => terms.key,
      },
      'triplicities': triplicities.key,
    },
    'scores': scores._json,
  };
}

/// The terms and triplicities a reading used; [Terms.table] for the
/// request's own table.
final class AppliedDignityRules extends _Value {
  const AppliedDignityRules({required this.terms, required this.triplicities});

  final Terms terms;
  final Triplicities triplicities;

  @override
  List<Object?> get _fields => [terms, triplicities];
}

/// The dignities and debilities a planet holds where it stands.
final class EssentialDignity extends _Value {
  const EssentialDignity({
    required this.house,
    required this.exaltation,
    required this.triplicity,
    required this.term,
    required this.face,
    required this.detriment,
    required this.fall,
  });

  final bool house;
  final bool exaltation;
  final bool triplicity;
  final bool term;
  final bool face;
  final bool detriment;
  final bool fall;

  /// Whether this dignity holds.
  bool holds(DignityKind kind) => switch (kind) {
    DignityKind.house => house,
    DignityKind.exaltation => exaltation,
    DignityKind.triplicity => triplicity,
    DignityKind.term => term,
    DignityKind.face => face,
  };

  @override
  List<Object?> get _fields => [
    house,
    exaltation,
    triplicity,
    term,
    face,
    detriment,
    fall,
  ];
}

/// One of the five essential dignities, strongest first, as Lilly scores
/// them (p. 115).
enum DignityKind { house, exaltation, triplicity, term, face }

/// Two planets each standing in at least one of the other's five
/// dignities (Lilly, p. 112), each side reported whole.
///
/// ```dart
/// final byHouse = chart.dignities?.receptions
///     .where((one) => one.mutual.contains(DignityKind.house));
/// ```
final class Reception extends _Value {
  const Reception({
    required this.planets,
    required this.firstIn,
    required this.secondIn,
  });

  /// The two, in the Chaldean order.
  final (Graha, Graha) planets;

  /// The second's dignities where the first stands.
  final EssentialDignity firstIn;

  /// The first's dignities where the second stands.
  final EssentialDignity secondIn;

  /// The kinds each stands in of the other's, strongest first; empty for a
  /// mixed reception.
  List<DignityKind> get mutual => [
    for (final kind in DignityKind.values)
      if (firstIn.holds(kind) && secondIn.holds(kind)) kind,
  ];

  @override
  List<Object?> get _fields => [planets, firstIn, secondIn];
}

/// One planet's dignities and its score.
final class PlanetDignity extends _Value {
  const PlanetDignity({
    required this.planet,
    required this.longitudeDeg,
    required this.dignity,
    required this.peregrine,
    required this.score,
    required this.reception,
  });

  final Graha planet;

  /// Degrees of the chart's zodiac.
  final double longitudeDeg;

  final EssentialDignity dignity;

  /// In none of its five dignities, whatever its debilities.
  final bool peregrine;

  /// Its score from its own dignities alone.
  final int score;

  /// What Lilly's table adds for mutual reception (p. 115): the house's
  /// score when received by house, the exaltation's when by exaltation,
  /// nothing for a mixed reception or one by a lesser dignity (C210). A
  /// total is `score + reception`.
  final int reception;

  @override
  List<Object?> get _fields => [
    planet,
    longitudeDeg,
    dignity,
    peregrine,
    score,
    reception,
  ];
}

/// A chart's essential dignities, with everything that made them: the
/// sect, the rule that chose it, the rules and the scores
/// (`03-design/essential-dignities.md`).
final class Dignities extends _Value {
  const Dignities({
    required this.sect,
    required this.sectRule,
    required this.rules,
    required this.scores,
    required this.planets,
    required this.receptions,
  });

  final Sect sect;
  final SectRule sectRule;
  final AppliedDignityRules rules;
  final DignityScores scores;

  /// The seven in the Chaldean order, Saturn first.
  final List<PlanetDignity> planets;

  /// Every pair in reception, in the Chaldean order of the first and then
  /// the second.
  final List<Reception> receptions;

  @override
  List<Object?> get _fields => [
    sect,
    sectRule,
    rules,
    scores,
    planets,
    receptions,
  ];
}

/// The orbs and limits Lilly's accidental fortitudes are judged by
/// (`03-design/essential-dignities.md` §Accidental fortitudes); every one
/// left out is Lilly's, so `AccidentalRules(beamsDeg: 15)` changes that one
/// alone. An answer's rules are a request as they stand.
final class AccidentalRules extends _Value {
  const AccidentalRules({
    this.combustionDeg = 8.5,
    this.combustionInSign = true,
    this.beamsDeg = 17,
    this.cazimiDeg = 17 / 60,
    this.cuspOrbDeg = 5,
    this.starOrbDeg = 5,
    this.partile = Partile.sameDegree,
    this.partileOrbDeg = 0,
    this.siege = Siege.sameSign,
    this.siegeSpanDeg = 0,
    this.meanMotionDeg = lillysMeanMotions,
  });

  /// Lilly's (pp. 113–115).
  static const AccidentalRules lilly = AccidentalRules();

  /// Lilly's mean daily motions, the seven in the Chaldean order: Saturn
  /// 2′01″, Jupiter 4′59″, Mars 31′27″, the Sun, Venus and Mercury 59′08″,
  /// the Moon 13°10′36″.
  static const List<double> lillysMeanMotions = [
    0.0 + 2 / 60 + 1 / 3600,
    0.0 + 4 / 60 + 59 / 3600,
    0.0 + 31 / 60 + 27 / 3600,
    0.0 + 59 / 60 + 8 / 3600,
    0.0 + 59 / 60 + 8 / 3600,
    0.0 + 59 / 60 + 8 / 3600,
    13.0 + 10 / 60 + 36 / 3600,
  ];

  /// Combust within this many degrees of the Sun.
  final double combustionDeg;

  /// Whether combustion also asks for the Sun's sign (C211).
  final bool combustionInSign;

  /// Under the beams within this many degrees (C212).
  final double beamsDeg;

  /// Cazimi within this many degrees.
  final double cazimiDeg;

  /// A planet this near the next cusp is in its house (p. 33, C214).
  final double cuspOrbDeg;

  /// With a star within this many degrees.
  final double starOrbDeg;

  /// By the same degree, Lilly's, or within [partileOrbDeg] of the exact
  /// aspect (C216).
  final Partile partile;

  /// The orb of [Partile.within]; 0 for [Partile.sameDegree].
  final double partileOrbDeg;

  /// Within one sign, Lilly's example, or on an arc no wider than
  /// [siegeSpanDeg] (C215).
  final Siege siege;

  /// The span of [Siege.within]; 0 for [Siege.sameSign].
  final double siegeSpanDeg;

  /// The mean daily motions swift and slow are judged against, the seven
  /// in the Chaldean order.
  final List<double> meanMotionDeg;

  @override
  List<Object?> get _fields => [
    combustionDeg,
    combustionInSign,
    beamsDeg,
    cazimiDeg,
    cuspOrbDeg,
    starOrbDeg,
    partile,
    partileOrbDeg,
    siege,
    siegeSpanDeg,
    meanMotionDeg,
  ];

  Map<String, Object?> get _json => {
    'combustionDeg': combustionDeg,
    'combustionInSign': combustionInSign,
    'beamsDeg': beamsDeg,
    'cazimiDeg': cazimiDeg,
    'cuspOrbDeg': cuspOrbDeg,
    'starOrbDeg': starOrbDeg,
    'partile':
        partile == Partile.within
            ? {
              'WITHIN': {'orbDeg': partileOrbDeg},
            }
            : partile.key,
    'siege':
        siege == Siege.within
            ? {
              'WITHIN': {'spanDeg': siegeSpanDeg},
            }
            : siege.key,
    'meanMotionDeg': meanMotionDeg,
  };
}

/// What each of Lilly's accidental lines is worth (p. 115): positive for a
/// fortitude, negative for a debility; every one left out is Lilly's. An
/// answer's scores are a request as they stand.
final class AccidentalScores extends _Value {
  const AccidentalScores({
    this.houses = const [5, 3, 1, 4, 3, -2, 4, -2, 2, 5, 4, -5],
    this.direct = 4,
    this.retrograde = -5,
    this.swift = 2,
    this.slow = -2,
    this.superiorOriental = 2,
    this.superiorOccidental = -2,
    this.inferiorOriental = -2,
    this.inferiorOccidental = 2,
    this.increasing = 2,
    this.decreasing = -2,
    this.freeFromCombustion = 5,
    this.cazimi = 5,
    this.combust = -5,
    this.underBeams = -4,
    this.conjunctBenefic = 5,
    this.conjunctNorthNode = 4,
    this.trineBenefic = 4,
    this.sextileBenefic = 3,
    this.conjunctMalefic = -5,
    this.conjunctSouthNode = -4,
    this.opposedMalefic = -4,
    this.squareMalefic = -3,
    this.besieged = -5,
    this.regulus = 6,
    this.spica = 5,
    this.algol = -5,
  });

  /// Lilly's "ready Table" (p. 115).
  static const AccidentalScores lilly = AccidentalScores();

  /// The first house to the twelfth.
  final List<int> houses;
  final int direct;
  final int retrograde;
  final int swift;
  final int slow;

  /// Saturn, Jupiter or Mars oriental.
  final int superiorOriental;

  /// Saturn, Jupiter or Mars occidental.
  final int superiorOccidental;

  /// Venus or Mercury oriental.
  final int inferiorOriental;

  /// Venus or Mercury occidental.
  final int inferiorOccidental;
  final int increasing;
  final int decreasing;
  final int freeFromCombustion;
  final int cazimi;
  final int combust;
  final int underBeams;
  final int conjunctBenefic;
  final int conjunctNorthNode;
  final int trineBenefic;
  final int sextileBenefic;
  final int conjunctMalefic;
  final int conjunctSouthNode;
  final int opposedMalefic;
  final int squareMalefic;
  final int besieged;
  final int regulus;
  final int spica;
  final int algol;

  /// Each line by the name a request spells it, in Lilly's order.
  Map<String, int> get lines => {
    'direct': direct,
    'retrograde': retrograde,
    'swift': swift,
    'slow': slow,
    'superiorOriental': superiorOriental,
    'superiorOccidental': superiorOccidental,
    'inferiorOriental': inferiorOriental,
    'inferiorOccidental': inferiorOccidental,
    'increasing': increasing,
    'decreasing': decreasing,
    'freeFromCombustion': freeFromCombustion,
    'cazimi': cazimi,
    'combust': combust,
    'underBeams': underBeams,
    'conjunctBenefic': conjunctBenefic,
    'conjunctNorthNode': conjunctNorthNode,
    'trineBenefic': trineBenefic,
    'sextileBenefic': sextileBenefic,
    'conjunctMalefic': conjunctMalefic,
    'conjunctSouthNode': conjunctSouthNode,
    'opposedMalefic': opposedMalefic,
    'squareMalefic': squareMalefic,
    'besieged': besieged,
    'regulus': regulus,
    'spica': spica,
    'algol': algol,
  };

  @override
  List<Object?> get _fields => [houses, ...lines.values];

  Map<String, Object?> get _json => {'houses': houses, ...lines};
}

/// How to read every chart's fortitudes, both halves of Lilly's table
/// (`03-design/essential-dignities.md` §Accidental fortitudes). Each
/// chart's come back as its `fortitudes`, and the essential half as its
/// `dignities` too, so a request asks for one or the other: both are
/// refused by `dignities`.
///
/// ```dart
/// final chart = ctx.chart.found(
///   /* … */ fortitudes: const FortitudeRequest(
///     rules: AccidentalRules(partile: Partile.within, partileOrbDeg: 1),
///   ),
/// );
/// final strongest = chart.fortitudes?.planets
///     .reduce((a, b) => a.net >= b.net ? a : b);
/// ```
final class FortitudeRequest {
  const FortitudeRequest({
    this.dignities = const DignityRequest(),
    this.rules = AccidentalRules.lilly,
    this.scores = AccidentalScores.lilly,
    this.almuten = AlmutenRules.lilly,
  });

  /// How the essential half is read.
  final DignityRequest dignities;
  final AccidentalRules rules;
  final AccidentalScores scores;

  /// How the almutens are read.
  final AlmutenRules almuten;

  String get _json => jsonEncode(<String, Object?>{
    'dignities': dignities._record,
    'rules': rules._json,
    'scores': scores._json,
    'almuten': almuten._json,
  });
}

/// How a chart's almutens are read, Lilly's by default
/// (`03-design/essential-dignities.md` §The almuten). An answer's
/// `almutens.rules` is one, handed back as it stands.
///
/// ```dart
/// const sign = AlmutenRules(place: PlaceReading.sign);
/// ```
final class AlmutenRules extends _Value {
  const AlmutenRules({
    this.place = PlaceReading.degree,
    this.fortune = FortuneRule.dayAndNight,
  });

  /// Lilly's: the degree, and Fortune the same by day and night.
  static const AlmutenRules lilly = AlmutenRules();

  /// What of a place its dignities are counted from: the degree (all five)
  /// or the sign (house, exaltation, triplicity), C218.
  final PlaceReading place;

  /// How Fortune is taken by night: Lilly's, reversed, or reversed while
  /// the Moon is up (C220, C221).
  final FortuneRule fortune;

  Map<String, Object?> get _json => {
    'place': place.key,
    'fortune': fortune.key,
  };

  @override
  List<Object?> get _fields => [place, fortune];
}

/// One planet's total in an almuten's ranking.
final class AlmutenTotal extends _Value {
  const AlmutenTotal({required this.planet, required this.total});

  final Graha planet;
  final int total;

  @override
  List<Object?> get _fields => [planet, total];
}

/// An almuten as a ranking. Lilly breaks no tie, so every planet holding
/// the greatest total is an almuten (C219).
final class Almuten extends _Value {
  const Almuten({
    required this.totals,
    required this.almutens,
    required this.partakers,
  });

  /// The ranking of `planets` by `totals`, both in the Chaldean order.
  factory Almuten._of(List<Graha> planets, List<int> totals) {
    int greatest(int a, int b) => a > b ? a : b;
    final top = totals.reduce(greatest);
    final below = [
      for (final total in totals)
        if (total < top) total,
    ];
    List<Graha> holding(int total) => List<Graha>.unmodifiable([
      for (var k = 0; k < planets.length; k += 1)
        if (totals[k] == total) planets[k],
    ]);
    return Almuten(
      totals: List<AlmutenTotal>.unmodifiable([
        for (var k = 0; k < planets.length; k += 1)
          AlmutenTotal(planet: planets[k], total: totals[k]),
      ]),
      almutens: holding(top),
      partakers:
          below.isEmpty ? const <Graha>[] : holding(below.reduce(greatest)),
    );
  }

  /// The seven's totals, in the Chaldean order.
  final List<AlmutenTotal> totals;

  /// Every planet holding the greatest total: one unless they tie.
  final List<Graha> almutens;

  /// Every planet holding the next total down, Chapter CV's partakers;
  /// empty when all seven tie.
  final List<Graha> partakers;

  @override
  List<Object?> get _fields => [totals, almutens, partakers];
}

/// A chart's almutens three ways, with the rules that made them.
///
/// ```dart
/// final lord = chart.fortitudes?.almutens.figure.almutens; // Lilly's
/// ```
final class Almutens extends _Value {
  const Almutens({
    required this.rules,
    required this.fortuneDeg,
    required this.figure,
    required this.places,
    required this.houses,
  });

  final AlmutenRules rules;

  /// The Part of Fortune, one of the five places.
  final double fortuneDeg;

  /// Lilly's almuten of the figure: each planet's `net`.
  final Almuten figure;

  /// Chapter CV's: essential dignities over the ascendant, midheaven, Sun,
  /// Moon and Fortune.
  final Almuten places;

  /// Each house's, of its cusp, the first to the twelfth.
  final List<Almuten> houses;

  @override
  List<Object?> get _fields => [rules, fortuneDeg, figure, places, houses];
}

/// What a chart's accidental fortitudes were read from, in the chart's
/// zodiac.
final class AccidentalSky extends _Value {
  const AccidentalSky({
    required this.houses,
    required this.cuspsDeg,
    required this.ascendantDeg,
    required this.midheavenDeg,
    required this.speedsDegPerDay,
    required this.northNodeDeg,
    required this.regulusDeg,
    required this.spicaDeg,
    required this.algolDeg,
  });

  /// Regiomontanus, Lilly's, unless a profile names another division for
  /// the `hellenistic` module.
  final HouseSystem houses;

  /// The twelve cusps, the first to the twelfth.
  final List<double> cuspsDeg;

  /// The ascendant, from the chart's angles: whole-sign and equal houses do
  /// not put it on a cusp.
  final double ascendantDeg;

  /// The midheaven, from the chart's angles.
  final double midheavenDeg;

  /// The seven's daily motions in the Chaldean order, negative when
  /// retrograde.
  final List<double> speedsDegPerDay;

  final double northNodeDeg;

  /// The star's apparent place of date, as are Spica's and Algol's.
  final double regulusDeg;
  final double spicaDeg;
  final double algolDeg;

  @override
  List<Object?> get _fields => [
    houses,
    cuspsDeg,
    ascendantDeg,
    midheavenDeg,
    speedsDegPerDay,
    northNodeDeg,
    regulusDeg,
    spicaDeg,
    algolDeg,
  ];
}

/// One accidental line a planet meets, and what it scores for that planet:
/// orientality scores Saturn, Jupiter and Mars one way and Venus and
/// Mercury the other.
final class AccidentLine extends _Value {
  const AccidentLine({required this.accident, required this.points});

  final Accident accident;
  final int points;

  @override
  List<Object?> get _fields => [accident, points];
}

/// One planet's accidental fortitudes and debilities.
final class PlanetAccidents extends _Value {
  const PlanetAccidents({
    required this.planet,
    required this.house,
    required this.accidents,
    required this.fortitude,
    required this.debility,
    required this.net,
  });

  final Graha planet;

  /// Its house, 1 to 12, under the five-degree rule.
  final int house;

  /// Every line beyond its house, in Lilly's order.
  final List<AccidentLine> accidents;

  /// The sum of its fortitudes, its house's included.
  final int fortitude;

  /// The sum of its debilities, its house's included, as a positive number.
  final int debility;

  /// Lilly's net over the whole table: its essential `score + reception`
  /// and `fortitude - debility`.
  final int net;

  @override
  List<Object?> get _fields => [
    planet,
    house,
    accidents,
    fortitude,
    debility,
    net,
  ];
}

/// Both halves of Lilly's table in one chart, with everything that made
/// them (`03-design/essential-dignities.md` §Accidental fortitudes).
final class Fortitudes extends _Value {
  const Fortitudes({
    required this.dignities,
    required this.sky,
    required this.rules,
    required this.scores,
    required this.planets,
    required this.almutens,
  });

  /// The essential half, which the chart's `dignities` also reads.
  final Dignities dignities;
  final AccidentalSky sky;
  final AccidentalRules rules;
  final AccidentalScores scores;

  /// The seven in the Chaldean order, Saturn first.
  final List<PlanetAccidents> planets;

  /// The almutens: of the figure, of the five places, and of each house.
  final Almutens almutens;

  @override
  List<Object?> get _fields => [
    dignities,
    sky,
    rules,
    scores,
    planets,
    almutens,
  ];
}

/// How to read every chart's lots, Valens's by default
/// (`03-design/hellenistic-lots.md`). Each chart's come back as its
/// `lots`, and an answer's `request` is one, handed back as it stands.
///
/// ```dart
/// final chart = ctx.chart.found(
///   /* … */ lots: const LotRequest(fortune: FortuneRule.reversedWhileMoonUp),
/// );
/// final fortune = chart.lots?.lots.first.place.sign;
/// ```
final class LotRequest extends _Value {
  const LotRequest({
    this.sectRule = SectRule.horizon,
    this.fortune = FortuneRule.reversedByNight,
  });

  /// Valens's: his hemisphere (C209), and Fortune reversed by night
  /// (II.22, C221).
  static const LotRequest valens = LotRequest();

  final SectRule sectRule;

  /// How Fortune is taken by night (C221).
  final FortuneRule fortune;

  String get _json => jsonEncode(<String, Object?>{
    'sectRule': sectRule.key,
    'fortune': fortune.key,
  });

  @override
  List<Object?> get _fields => [sectRule, fortune];
}

/// Where a lot fell, in the chart's zodiac.
final class LotPlace extends _Value {
  const LotPlace({
    required this.longitudeDeg,
    required this.sign,
    required this.lord,
    required this.house,
  });

  /// Degrees in [0, 360).
  final double longitudeDeg;
  final Rashi sign;

  /// The sign's lord, the lot's ruler.
  final Graha lord;

  /// 1 to 12, counted in whole signs from the ascendant's sign.
  final int house;

  @override
  List<Object?> get _fields => [longitudeDeg, sign, lord, house];
}

/// One lot and where it fell.
final class PlacedLot extends _Value {
  const PlacedLot({required this.lot, required this.place});

  final Lot lot;
  final LotPlace place;

  @override
  List<Object?> get _fields => [lot, place];
}

/// A chart's lots, with its sect and the rules they were read under
/// (`03-design/hellenistic-lots.md`).
final class Lots extends _Value {
  const Lots({
    required this.sect,
    required this.request,
    required this.fortuneReversed,
    required this.lots,
  });

  final Sect sect;

  /// The rules they were read under, every field filled.
  final LotRequest request;

  /// Whether Fortune was counted from the Moon to the Sun, and Daimon the
  /// other way.
  final bool fortuneReversed;

  /// All fourteen, in the catalogue's order.
  final List<PlacedLot> lots;

  @override
  List<Object?> get _fields => [sect, request, fortuneReversed, lots];
}

/// How to read every chart's considerations before judgement, Lilly's by
/// default (`03-design/hellenistic-considerations.md`). Each chart's come
/// back as its `considerations`, and an answer's `rules` is one, handed
/// back as it stands.
///
/// ```dart
/// final chart = ctx.chart.found(
///   /* … */ considerations: const ConsiderationRules(moonLateFromDeg: 25),
/// );
/// final voidOfCourse = chart.considerations?.moon.course.next == null;
/// ```
final class ConsiderationRules extends _Value {
  const ConsiderationRules({
    this.moonLateFromDeg = 27,
    this.orbsDeg = const <double>[10, 12, 7.5, 17, 8, 7, 12.5],
  });

  /// Lilly's: the Moon late from 27°, and his orbs (*Christian Astrology*
  /// p. 107).
  static const ConsiderationRules lilly = ConsiderationRules();

  /// From what degree of her sign the Moon is late (C229): Lilly gives no
  /// number, so 27, his late Ascendant's. 0 to 30, or the SDK refuses it
  /// by `considerations.moonLateFromDeg`.
  final double moonLateFromDeg;

  /// Each planet's orb in the Chaldean order, Saturn to the Moon; a
  /// perfection is within the moieties when the gap is under half the sum
  /// of the two planets' (C230). Seven, none negative.
  final List<double> orbsDeg;

  String get _json => jsonEncode(<String, Object?>{
    'moonLateFromDeg': moonLateFromDeg,
    'orbsDeg': orbsDeg,
  });

  @override
  List<Object?> get _fields => [moonLateFromDeg, orbsDeg];
}

/// Lilly's rules a chart's perfection is read under
/// (`03-design/hellenistic-perfection.md`); an answer's `rules` is one,
/// handed back as it stands.
final class PerfectionRules extends _Value {
  const PerfectionRules({
    this.orbsDeg = const <double>[10, 12, 7.5, 17, 8, 7, 12.5],
    this.horizonDays,
    this.withinSign = true,
  });

  /// Lilly's orbs (p. 107), looking until the swifter significator leaves
  /// its sign, a third planet's contacts bounded by the applier's sign.
  static const PerfectionRules lilly = PerfectionRules();

  /// Each planet's whole orb in the Chaldean order, Saturn to the Moon;
  /// half of each counts toward an application. Seven, none negative.
  final List<double> orbsDeg;

  /// How many days ahead to look; null, until the swifter significator
  /// leaves its sign (C232). Positive, or the SDK refuses it by
  /// `perfection.rules.horizonDays`.
  final double? horizonDays;

  /// Whether a third planet's contact counts only before the planet
  /// applying leaves its sign (C234); false counts every contact inside
  /// the horizon.
  final bool withinSign;

  Map<String, Object?> get _record => <String, Object?>{
    'orbsDeg': orbsDeg,
    if (horizonDays != null) 'horizonDays': horizonDays,
    'withinSign': withinSign,
  };

  @override
  List<Object?> get _fields => [orbsDeg, horizonDays, withinSign];
}

/// A span of time a progression's rate is stated in.
final class ProgressionSpan extends _Value {
  const ProgressionSpan._(this._key) : days = null;

  /// Any number of days, finite and above zero.
  const ProgressionSpan.days(double this.days) : _key = null;

  /// A day.
  static const ProgressionSpan day = ProgressionSpan._('DAY');

  /// A mean synodic month, new Moon to new Moon.
  static const ProgressionSpan synodicMonth = ProgressionSpan._(
    'SYNODIC_MONTH',
  );

  /// A mean sidereal month.
  static const ProgressionSpan siderealMonth = ProgressionSpan._(
    'SIDEREAL_MONTH',
  );

  /// A year, as long as the request's [YearMeasure] makes it.
  static const ProgressionSpan year = ProgressionSpan._('YEAR');

  final String? _key;

  /// The days of a [ProgressionSpan.days] span; null for a named one.
  final double? days;

  Object get _record => _key ?? <String, Object?>{'DAYS': days};

  @override
  List<Object?> get _fields => [_key, days];
}

/// How much sky measures how much life.
final class ProgressionRate extends _Value {
  const ProgressionRate(this.sky, this.life);

  /// A day for a year: Leo's progressed horoscope, the secondary progression.
  static const ProgressionRate secondary = ProgressionRate(
    ProgressionSpan.day,
    ProgressionSpan.year,
  );

  /// A day for a synodic month: the tertiary progression (C238).
  static const ProgressionRate tertiary = ProgressionRate(
    ProgressionSpan.day,
    ProgressionSpan.synodicMonth,
  );

  /// A synodic month for a year: the minor progression.
  static const ProgressionRate minor = ProgressionRate(
    ProgressionSpan.synodicMonth,
    ProgressionSpan.year,
  );

  final ProgressionSpan sky;
  final ProgressionSpan life;

  Map<String, Object?> get _record => {
    'sky': sky._record,
    'life': life._record,
  };

  @override
  List<Object?> get _fields => [sky, life];
}

/// How long a year of life is, against the calendar (C236).
enum YearMeasure {
  /// The mean tropical year, every modern implementation's.
  tropical('TROPICAL'),

  /// The Julian year of 365.25 days.
  julian('JULIAN'),

  /// Leo's rule by sidereal time at noon (Appendix V), a day for a year only.
  noonSiderealTime('NOON_SIDEREAL_TIME');

  const YearMeasure(this.key);

  /// The key a request spells it with.
  final String key;
}

/// How the progressed midheaven moves (C237).
enum AngleMethod {
  /// The mean Sun in right ascension: Leo's own map.
  naibodRightAscension('NAIBOD_RIGHT_ASCENSION'),

  /// The mean Sun along the ecliptic.
  naibodLongitude('NAIBOD_LONGITUDE'),

  /// The true Sun along the ecliptic: the solar arc.
  solarArcLongitude('SOLAR_ARC_LONGITUDE'),

  /// The true Sun in right ascension.
  solarArcRightAscension('SOLAR_ARC_RIGHT_ASCENSION'),

  /// The chart at the progressed instant itself.
  quotidian('QUOTIDIAN');

  const AngleMethod(this.key);

  /// The key a request spells it with.
  final String key;
}

/// The arc a direction moves every point by.
final class DirectionArc extends _Value {
  const DirectionArc._(this._key) : degreesPerYear = null;

  /// Any degrees a year, finite and above zero.
  const DirectionArc.perYear(double this.degreesPerYear) : _key = null;

  /// The Sun's arc under the request's measure.
  static const DirectionArc solar = DirectionArc._('SOLAR');

  /// Naibod's measure, the mean Sun's daily motion a year.
  static const DirectionArc naibod = DirectionArc._('NAIBOD');

  /// Ptolemy's measure, a degree a year.
  static const DirectionArc ptolemy = DirectionArc._('PTOLEMY');

  final String? _key;

  /// The degrees a year of a [DirectionArc.perYear] arc; null otherwise.
  final double? degreesPerYear;

  Object get _record => _key ?? <String, Object?>{'PER_YEAR': degreesPerYear};

  @override
  List<Object?> get _fields => [_key, degreesPerYear];
}

/// A window of life to find a birth's progressed contacts in, spelled as a
/// [HitRequest] spells its planets, points and aspects.
final class ProgressionContacts {
  const ProgressionContacts({
    required this.from,
    required this.to,
    this.grahas,
    this.points,
    this.aspects,
  });

  /// The window's start, an instant of life, a UTC Julian day.
  final double from;

  /// Its end, after the start.
  final double to;

  /// The progressed planets; the seven by default.
  final List<Graha>? grahas;

  /// The radical points; the seven and the lagna by default.
  final List<NatalPoint>? points;

  /// The aspects' angles, whole degrees to 180; Leo's table (p. 48) by
  /// default.
  final List<int>? aspects;

  Map<String, Object?> get _record => {
    'from': from,
    'to': to,
    if (grahas case final grahas?) 'grahas': [for (final g in grahas) g.key],
    if (points case final points?) 'points': [for (final p in points) p._json],
    if (aspects case final aspects?) 'aspects': aspects,
  };
}

/// What to read every chart's birth through
/// (`03-design/western-progressions.md`): the progressed chart and the
/// direction at an instant of life [at], the [contacts] over a window, or
/// both. Every field left out is Leo's default.
///
/// ```dart
/// const leo = ProgressionsRequest(at: 2460676.5, year: YearMeasure.noonSiderealTime);
/// ```
final class ProgressionsRequest {
  const ProgressionsRequest({
    this.at,
    this.rate = ProgressionRate.secondary,
    this.year = YearMeasure.tropical,
    this.angles = AngleMethod.naibodRightAscension,
    this.direction = DirectionArc.solar,
    this.contacts,
  });

  /// The instant of life, a UTC Julian day.
  final double? at;
  final ProgressionRate rate;
  final YearMeasure year;
  final AngleMethod angles;
  final DirectionArc direction;
  final ProgressionContacts? contacts;

  String get _json => jsonEncode(<String, Object?>{
    if (at case final at?) 'at': at,
    'rate': rate._record,
    'year': year.key,
    'angles': angles.key,
    'direction': direction._record,
    if (contacts case final contacts?) 'contacts': contacts._record,
  });
}

/// The orbs a Western aspect table is read under
/// (`03-design/western-aspects.md`, C240): Leo's by aspect, a moiety a
/// planet, or a caller's own orb an aspect.
///
/// ```dart
/// const tight = OrbModel.byAspect({WesternAspect.trine: 6, WesternAspect.square: 6});
/// ```
final class OrbModel extends _Value {
  const OrbModel._(this._model) : moieties = null, byAspect = null;

  /// Each planet's whole orb; a pair is within half the sum of theirs.
  const OrbModel.moieties(Map<Graha, double> this.moieties)
    : _model = 'MOIETIES',
      byAspect = null;

  /// Each aspect's orb, whatever the pair.
  const OrbModel.byAspect(Map<WesternAspect, double> this.byAspect)
    : _model = 'BY_ASPECT',
      moieties = null;

  /// Leo's orbs by aspect, the luminaries' wider (*How to Judge a
  /// Nativity*, pp. 43–47); the default.
  static const OrbModel leo = OrbModel._('LEO');

  /// Lilly's moieties (*Christian Astrology*, p. 107), which give the outer
  /// three no orb.
  static const OrbModel lilly = OrbModel.moieties({
    Graha.saturn: 10,
    Graha.jupiter: 12,
    Graha.mars: 7.5,
    Graha.sun: 17,
    Graha.venus: 8,
    Graha.mercury: 7,
    Graha.moon: 12.5,
  });

  final String _model;

  /// The moieties of an [OrbModel.moieties]; null otherwise.
  final Map<Graha, double>? moieties;

  /// The orbs of an [OrbModel.byAspect]; null otherwise.
  final Map<WesternAspect, double>? byAspect;

  Map<String, Object?> get _record => <String, Object?>{
    'model': _model,
    if (moieties case final moieties?)
      'orbs': [
        for (final MapEntry(:key, :value) in moieties.entries)
          <String, Object?>{'graha': key.key, 'orbDeg': value},
      ],
    if (byAspect case final byAspect?)
      'orbs': [
        for (final MapEntry(:key, :value) in byAspect.entries)
          <String, Object?>{'aspect': key.key, 'orbDeg': value},
      ],
  };

  @override
  List<Object?> get _fields => [
    _model,
    ...?moieties?.keys,
    ...?moieties?.values,
    ...?byAspect?.keys,
    ...?byAspect?.values,
  ];
}

/// Which Western aspects to look for in every chart, and under which
/// [orbs] (`03-design/western-aspects.md`); Leo's nine under his orbs by
/// default (C240).
///
/// ```dart
/// const two = WesternAspectRequest(aspects: [WesternAspect.trine, WesternAspect.square]);
/// ```
final class WesternAspectRequest {
  const WesternAspectRequest({this.aspects, this.orbs = OrbModel.leo});

  /// Lilly's reading: the Ptolemaic five under his moieties.
  static const WesternAspectRequest lilly = WesternAspectRequest(
    aspects: [
      WesternAspect.conjunction,
      WesternAspect.sextile,
      WesternAspect.square,
      WesternAspect.trine,
      WesternAspect.opposition,
    ],
    orbs: OrbModel.lilly,
  );

  /// The aspects looked for, each once; Leo's nine when null.
  final List<WesternAspect>? aspects;
  final OrbModel orbs;

  Map<String, Object?> get _record => <String, Object?>{
    if (aspects case final aspects?)
      'aspects': [for (final a in aspects) a.key],
    'orbs': orbs._record,
  };

  String get _json => jsonEncode(_record);
}

/// The point an equal varna earns (C259).
enum EqualVarna {
  /// One point, the verse's own; the default.
  whole('WHOLE'),

  /// Half a point.
  half('HALF');

  const EqualVarna(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// The points a Deva bride and a Manushya groom earn in Gana (C262).
enum DevaBride {
  /// Four; the default.
  four('FOUR'),

  /// Three.
  three('THREE');

  const DevaBride(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// How a bad Bhakoot is lifted (C263).
enum BhakootLift {
  /// Any one of the five exceptions, the nadi pure; the default.
  anyOne('ANY_ONE'),

  /// Garga's count.
  garga('GARGA');

  const BhakootLift(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// Which shared nadi is a dosha (C264).
enum NadiDosha {
  /// Any of the three; the default.
  any('ANY'),

  /// The middle nadi only.
  middleOnly('MIDDLE_ONLY');

  const NadiDosha(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// The readings an Ashta Koota is computed under; each default is the
/// source's own (`03-design/matching.md`).
///
/// ```dart
/// const middle = KootaRules(nadiDosha: NadiDosha.middleOnly);
/// ```
final class KootaRules {
  const KootaRules({
    this.equalVarna = EqualVarna.whole,
    this.devaBride = DevaBride.four,
    this.bhakootLift = BhakootLift.anyOne,
    this.nadiDosha = NadiDosha.any,
  });

  final EqualVarna equalVarna;
  final DevaBride devaBride;
  final BhakootLift bhakootLift;
  final NadiDosha nadiDosha;

  Map<String, Object?> get _record => <String, Object?>{
    'equalVarna': equalVarna.key,
    'devaBride': devaBride.key,
    'bhakootLift': bhakootLift.key,
    'nadiDosha': nadiDosha.key,
  };
}

/// Whose quarter comes first when one star, spanning two signs, is both
/// natives' (C270).
enum TwoSignStar {
  /// The groom's quarter is the earlier (p. 71); the default.
  groomEarlier('GROOM_EARLIER'),

  /// For a star with one quarter in the first sign, that quarter is the
  /// bride's (pp. 71–72).
  brideFirstSign('BRIDE_FIRST_SIGN');

  const TwoSignStar(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// How far the groom's star must stand from the bride's for
/// Sthree-Dheergham (C272).
enum DeerghaBeyond {
  /// Beyond the 13th (p. 72); the default.
  thirteenth('THIRTEENTH'),

  /// Beyond the 7th, as "some writers hold".
  seventh('SEVENTH');

  const DeerghaBeyond(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// Which friendship of the lords agrees on Rasyadhipathi and lifts by the
/// p. 76 exception (C273).
enum LordsFriendship {
  /// Each lord calls the other a friend; the default.
  mutual('MUTUAL'),

  /// Either calls the other a friend.
  oneWay('ONE_WAY');

  const LordsFriendship(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// The readings the ten considerations are computed under; each default
/// is the chapter's own (`03-design/matching.md`).
///
/// ```dart
/// const seventh = PoruthamRules(deerghaBeyond: DeerghaBeyond.seventh);
/// ```
final class PoruthamRules {
  const PoruthamRules({
    this.twoSignStar = TwoSignStar.groomEarlier,
    this.deerghaBeyond = DeerghaBeyond.thirteenth,
    this.lordsFriendship = LordsFriendship.mutual,
  });

  final TwoSignStar twoSignStar;
  final DeerghaBeyond deerghaBeyond;
  final LordsFriendship lordsFriendship;

  Map<String, Object?> get _record => <String, Object?>{
    'twoSignStar': twoSignStar.key,
    'deerghaBeyond': deerghaBeyond.key,
    'lordsFriendship': lordsFriendship.key,
  };
}

/// Which houses of Mars make the Kuja dosha (C285).
enum KujaHouses {
  /// The 1st, 4th, 7th, 8th and 12th (*Manasagari*, jāyābhāva v. 4); the
  /// default.
  manasagari('MANASAGARI'),

  /// The same with the 2nd, as modern practice has it (rank 3).
  withSecond('WITH_SECOND');

  const KujaHouses(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// From where Mars's house makes the Kuja dosha (C286).
enum KujaFrom {
  /// The lagna alone, as the verse counts; the default.
  lagna('LAGNA'),

  /// The lagna, the Moon or Venus, as modern practice counts (rank 3).
  lagnaMoonVenus('LAGNA_MOON_VENUS');

  const KujaFrom(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// The readings the Kuja dosha is computed under; each default is the
/// verse's own (`03-design/matching.md`).
///
/// ```dart
/// const everywhere = KujaRules(from: KujaFrom.lagnaMoonVenus);
/// ```
final class KujaRules {
  const KujaRules({
    this.houses = KujaHouses.manasagari,
    this.from = KujaFrom.lagna,
  });

  final KujaHouses houses;
  final KujaFrom from;

  Map<String, Object?> get _record => <String, Object?>{
    'houses': houses.key,
    'from': from.key,
  };
}

/// A place Mars's house is counted from.
enum KujaReference {
  /// The lagna.
  lagna('LAGNA'),

  /// The Moon.
  moon('MOON'),

  /// Venus.
  venus('VENUS');

  const KujaReference(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// Mars's house from one reference.
final class KujaReading extends _Value {
  const KujaReading({
    required this.from,
    required this.house,
    required this.inHouses,
  });

  /// The place the house is counted from.
  final KujaReference from;

  /// Mars's house from it by sign, 1 to 12 (C287).
  final int house;

  /// Whether the house is one of the rules' houses; it makes the dosha
  /// only from a reference the rules count.
  final bool inHouses;

  @override
  List<Object?> get _fields => [from, house, inHouses];
}

/// One native's Kuja dosha.
final class KujaSide extends _Value {
  const KujaSide({required this.readings, required this.dosha});

  /// Mars's house from the lagna, the Moon and Venus, whatever the rules
  /// count.
  final List<KujaReading> readings;

  /// Whether Mars stands in one of the rules' houses from a reference the
  /// rules count.
  final bool dosha;

  @override
  List<Object?> get _fields => [...readings, dosha];
}

/// One marriage dosha a match carries (C289): its reading, its koota or
/// consideration (null for the Kuja dosha, which is no koota), the side
/// carrying it for the Kuja dosha (null for the rest), and whether an
/// exception the source names lifts it. No severity (C290).
final class MarriageDosha extends _Value {
  const MarriageDosha({
    required this.system,
    required this.koota,
    required this.side,
    required this.lifted,
  });

  final DoshaSystem system;
  final Koota? koota;
  final MatchRole? side;
  final bool lifted;

  @override
  List<Object?> get _fields => [system, koota, side, lifted];
}

/// The Kuja dosha of a bride and a groom (*Manasagari*, jāyābhāva v. 4),
/// as clauses: nothing is lifted (C288).
final class Kuja extends _Value {
  const Kuja({required this.bride, required this.groom, required this.both});

  final KujaSide bride;
  final KujaSide groom;

  /// Whether both carry it, the fact the popular cancellation reads.
  final bool both;

  @override
  List<Object?> get _fields => [bride, groom, both];
}

/// A match with a partner's birth (`03-design/matching.md`): the
/// [partner], founded once for the whole batch under the context's
/// sidereal profile; [partnerRole], the side the partner stands on, every
/// chart standing on the other; the Ashta Koota's [rules]; the ten
/// considerations' [porutham] rules; and the Kuja dosha's [kuja] rules.
///
/// ```dart
/// final asked = MatchingRequest(
///   Partner(instant: 2447892.5, place: kathmandu, utcOffsetSeconds: 20700),
///   partnerRole: MatchRole.bride,
/// );
/// ```
final class MatchingRequest {
  const MatchingRequest(
    this.partner, {
    required this.partnerRole,
    this.rules = const KootaRules(),
    this.porutham = const PoruthamRules(),
    this.kuja = const KujaRules(),
  });

  /// Whose birth every chart is matched with.
  final Partner partner;

  /// The side the partner stands on.
  final MatchRole partnerRole;

  /// The readings the kootas are computed under.
  final KootaRules rules;

  /// The readings the ten considerations are computed under.
  final PoruthamRules porutham;

  /// The readings the Kuja dosha is computed under.
  final KujaRules kuja;

  Map<String, Object?> get _record => <String, Object?>{
    'partner': partner._record,
    'partnerRole': partnerRole.key,
    'rules': rules._record,
    'porutham': porutham._record,
    'kuja': kuja._record,
  };

  String get _json => jsonEncode(_record);
}

/// The five exceptions of VI.32–33 that lift a bad Bhakoot, each a clause
/// that holds or not.
final class BhakootExceptions extends _Value {
  const BhakootExceptions({
    required this.oneLord,
    required this.lordsFriends,
    required this.navamshaLordsFriends,
    required this.taraPure,
    required this.vashya,
  });

  /// One lord rules both signs.
  final bool oneLord;

  /// The sign lords are each other's friends.
  final bool lordsFriends;

  /// The navamsha lords are one or each other's friends.
  final bool navamshaLordsFriends;

  /// The tara is pure both ways.
  final bool taraPure;

  /// One sign is vashya to the other.
  final bool vashya;

  @override
  List<Object?> get _fields => [
    oneLord,
    lordsFriends,
    navamshaLordsFriends,
    taraPure,
    vashya,
  ];
}

/// What a koota read, one class a koota, each naming its [koota].
sealed class KootaReading extends _Value {
  const KootaReading();

  /// The koota this is the reading of.
  Koota get koota;
}

/// The varnas of the two Moon signs (VI.22).
final class VarnaKoota extends KootaReading {
  const VarnaKoota({required this.bride, required this.groom});

  final Varna bride;
  final Varna groom;

  @override
  Koota get koota => Koota.varna;

  @override
  List<Object?> get _fields => [bride, groom];
}

/// How the two Moon signs stand in Vashya (VI.23, C260).
final class VashyaKoota extends KootaReading {
  const VashyaKoota(this.relation);

  final VashyaRelation relation;

  @override
  Koota get koota => Koota.vashya;

  @override
  List<Object?> get _fields => [relation];
}

/// The taras each way, 1 to 9 (VI.24); the 3rd, 5th and 7th are bad.
final class TaraKoota extends KootaReading {
  const TaraKoota({required this.brideToGroom, required this.groomToBride});

  /// Counted from the bride's nakshatra to the groom's.
  final int brideToGroom;

  /// Counted from the groom's nakshatra to the bride's.
  final int groomToBride;

  @override
  Koota get koota => Koota.tara;

  @override
  List<Object?> get _fields => [brideToGroom, groomToBride];
}

/// The two yonis and how they stand (VI.25–26, C261).
final class YoniKoota extends KootaReading {
  const YoniKoota({
    required this.bride,
    required this.groom,
    required this.relation,
  });

  final Yoni bride;
  final Yoni groom;
  final YoniRelation relation;

  @override
  Koota get koota => Koota.yoni;

  @override
  List<Object?> get _fields => [bride, groom, relation];
}

/// The two Moon signs' lords and how they stand by the natural
/// friendships (VI.27–28).
final class MaitriKoota extends KootaReading {
  const MaitriKoota({
    required this.bride,
    required this.groom,
    required this.relation,
    required this.lifted,
  });

  final Graha bride;
  final Graha groom;
  final MaitriRelation relation;

  /// Whether a good Bhakoot lifts the lords' enmity (VI.33); false with no
  /// enmity.
  final bool lifted;

  @override
  Koota get koota => Koota.grahaMaitri;

  @override
  List<Object?> get _fields => [bride, groom, relation, lifted];
}

/// The two ganas (VI.29–30).
final class GanaKoota extends KootaReading {
  const GanaKoota({
    required this.bride,
    required this.groom,
    required this.dosha,
    required this.lifted,
  });

  final Gana bride;
  final Gana groom;

  /// Whether a Rakshasa stands beside another gana.
  final bool dosha;

  /// Whether the dosha is lifted: the sign lords or the navamsha lords
  /// befriended (VI.33), or one sign or one star between the two (VI.36);
  /// false with no dosha.
  final bool lifted;

  @override
  Koota get koota => Koota.gana;

  @override
  List<Object?> get _fields => [bride, groom, dosha, lifted];
}

/// How far the groom's Moon sign stands from the bride's (VI.31–33).
final class BhakootKoota extends KootaReading {
  const BhakootKoota({
    required this.apart,
    required this.dosha,
    required this.exceptions,
    required this.lifted,
  });

  /// The groom's sign counted from the bride's, 1 to 12.
  final int apart;

  /// The bad Bhakoot the signs stand at, or null.
  final BhakootDosha? dosha;

  final BhakootExceptions exceptions;

  /// Whether the exceptions lift the dosha under the rules; false with no
  /// dosha.
  final bool lifted;

  @override
  Koota get koota => Koota.bhakoot;

  @override
  List<Object?> get _fields => [apart, dosha, exceptions, lifted];
}

/// The two nadis (VI.34).
final class NadiKoota extends KootaReading {
  const NadiKoota({
    required this.bride,
    required this.groom,
    required this.dosha,
    required this.lifted,
  });

  final Nadi bride;
  final Nadi groom;

  /// Whether the shared nadi is a dosha under the rules.
  final bool dosha;

  /// Whether the dosha is lifted by one sign with two stars, one star across
  /// two signs or one star in two padas (VI.36); false with no dosha.
  final bool lifted;

  @override
  Koota get koota => Koota.nadi;

  @override
  List<Object?> get _fields => [bride, groom, dosha, lifted];
}

/// One koota's points and what it read.
final class KootaRow extends _Value {
  const KootaRow({
    required this.points,
    required this.maxPoints,
    required this.reading,
  });

  /// Its points, a multiple of a half.
  final double points;

  /// The most it gives, 1 for Varna to 8 for Nadi.
  final double maxPoints;

  final KootaReading reading;

  @override
  List<Object?> get _fields => [points, maxPoints, reading];
}

/// The Ashta Koota of a bride and a groom (*Muhurta Chintamani* VI.21–34).
/// Never a verdict: the doshas and their exceptions are clauses.
final class AshtaKoota extends _Value {
  const AshtaKoota({required this.kootas, required this.total});

  /// The eight, in the verse's order.
  final List<KootaRow> kootas;

  /// Their points, out of 36.
  final double total;

  @override
  List<Object?> get _fields => [...kootas, total];
}

/// What one of the ten considerations read, one class each, each naming
/// its catalogue [koota] (C282).
sealed class PoruthamReading extends _Value {
  const PoruthamReading();

  /// The catalogue koota this consideration is.
  Koota get koota;
}

/// Dhinam: the count and the rule of *Kalaprakasika* XIII that decided it
/// (pp. 69–72).
final class DhinamPorutham extends PoruthamReading {
  const DhinamPorutham({required this.count, required this.rule});

  /// The groom's nakshatra counted from the bride's, 1 to 27.
  final int count;
  final DhinamRule rule;

  @override
  Koota get koota => Koota.tara;

  @override
  List<Object?> get _fields => [count, rule];
}

/// Ganam: the two ganas (p. 72).
final class GanamPorutham extends PoruthamReading {
  const GanamPorutham({
    required this.bride,
    required this.groom,
    required this.diminished,
  });

  final Gana bride;
  final Gana groom;

  /// A Rakshasa beside another gana, the bride's star beyond the 14th from
  /// the groom's: the evil "diminishes", the disagreement stands (C279).
  final bool diminished;

  @override
  Koota get koota => Koota.gana;

  @override
  List<Object?> get _fields => [bride, groom, diminished];
}

/// Mahendra: the count, which agrees at the 4th, 7th and every third to
/// the 25th (p. 72).
final class MahendraPorutham extends PoruthamReading {
  const MahendraPorutham(this.count);

  final int count;

  @override
  Koota get koota => Koota.mahendra;

  @override
  List<Object?> get _fields => [count];
}

/// Sthree-Dheergham: the count, which agrees beyond the 13th (p. 72,
/// C272).
final class DeerghaPorutham extends PoruthamReading {
  const DeerghaPorutham(this.count);

  final int count;

  @override
  Koota get koota => Koota.streeDeergha;

  @override
  List<Object?> get _fields => [count];
}

/// Yoni on the chapter's own table, Uttarashadha the cow (p. 73, C278).
final class YoniPorutham extends PoruthamReading {
  const YoniPorutham({
    required this.bride,
    required this.groom,
    required this.hostile,
  });

  final Yoni bride;
  final Yoni groom;

  /// Whether they are among the chapter's eight enmities.
  final bool hostile;

  @override
  Koota get koota => Koota.yoni;

  @override
  List<Object?> get _fields => [bride, groom, hostile];
}

/// Rasi: how far the groom's Moon sign stands from the bride's (pp.
/// 73–74).
final class RasiPorutham extends PoruthamReading {
  const RasiPorutham(this.apart);

  /// The groom's sign counted from the bride's, 1 to 12.
  final int apart;

  @override
  Koota get koota => Koota.bhakoot;

  @override
  List<Object?> get _fields => [apart];
}

/// Rasyadhipathi: the two Moon signs' lords on the chapter's own
/// friendships (pp. 74–75).
final class RasyadhipathiPorutham extends PoruthamReading {
  const RasyadhipathiPorutham({
    required this.bride,
    required this.groom,
    required this.brideCallsFriend,
    required this.groomCallsFriend,
  });

  final Graha bride;
  final Graha groom;

  /// Whether the bride's lord calls the groom's a friend; a lord is its
  /// own.
  final bool brideCallsFriend;
  final bool groomCallsFriend;

  @override
  Koota get koota => Koota.grahaMaitri;

  @override
  List<Object?> get _fields => [
    bride,
    groom,
    brideCallsFriend,
    groomCallsFriend,
  ];
}

/// Vasyam on p. 75's table, never a sign to itself (C274).
final class VasyamPorutham extends PoruthamReading {
  const VasyamPorutham({
    required this.brideToGroom,
    required this.groomToBride,
  });

  /// Whether the bride's sign is concordant to the groom's.
  final bool brideToGroom;
  final bool groomToBride;

  @override
  Koota get koota => Koota.vashya;

  @override
  List<Object?> get _fields => [brideToGroom, groomToBride];
}

/// Rajju: the two divisions (p. 75, C275).
final class RajjuPorutham extends PoruthamReading {
  const RajjuPorutham({required this.bride, required this.groom});

  final Rajju bride;
  final Rajju groom;

  @override
  Koota get koota => Koota.rajju;

  @override
  List<Object?> get _fields => [bride, groom];
}

/// Vedhai: whether the two nakshatras pierce each other (p. 76, C276).
final class VedhaiPorutham extends PoruthamReading {
  const VedhaiPorutham({required this.pierced});

  final bool pierced;

  @override
  Koota get koota => Koota.vedha;

  @override
  List<Object?> get _fields => [pierced];
}

/// One consideration: whether it agrees, and what it read.
final class PoruthamRow extends _Value {
  const PoruthamRow({
    required this.agrees,
    required this.lifted,
    required this.reading,
  });

  /// Whether it agrees, a lift included.
  final bool agrees;

  /// Whether it agrees only by the p. 76 exception.
  final bool lifted;

  final PoruthamReading reading;

  @override
  List<Object?> get _fields => [agrees, lifted, reading];
}

/// The p. 76 exception's clauses, any one of which lifts Ganam, Rasi,
/// Rajju and Vedhai (C277).
final class PoruthamException extends _Value {
  const PoruthamException({
    required this.oneLord,
    required this.lordsFriendly,
    required this.opposite,
  });

  final bool oneLord;
  final bool lordsFriendly;
  final bool opposite;

  @override
  List<Object?> get _fields => [oneLord, lordsFriendly, opposite];
}

/// The ten considerations of a bride and a groom (*Kalaprakasika* XIII).
/// Never a verdict: "at least five" is the reader's to apply.
final class Porutham extends _Value {
  const Porutham({
    required this.considerations,
    required this.agreeing,
    required this.chiefAgreeing,
    required this.exception,
  });

  /// The ten, in the chapter's order.
  final List<PoruthamRow> considerations;

  /// How many agree.
  final int agreeing;

  /// How many of the chief five agree: Dhinam, Ganam, Yoni, Rasi and
  /// Rajju.
  final int chiefAgreeing;

  final PoruthamException exception;

  @override
  List<Object?> get _fields => [
    ...considerations,
    agreeing,
    chiefAgreeing,
    exception,
  ];
}

/// How a name in Latin letters is read (C293).
enum LatinName {
  /// Refused; the default, since English "ch" is IAST "c" and a guess
  /// would match the wrong star.
  refuse('REFUSE'),

  /// Read as IAST.
  iast('IAST');

  const LatinName(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// Where a syllable in Abhijit's row is placed, Abhijit being none of the
/// 27 (C294).
enum AbhijitPada {
  /// Refused; the default.
  refuse('REFUSE'),

  /// Uttara Ashadha's 4th quarter.
  uttaraAshadha('UTTARA_ASHADHA'),

  /// Shravana's 1st quarter.
  shravana('SHRAVANA');

  const AbhijitPada(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// How a name is read for naam milan; each default is the source's own
/// (`03-design/matching.md`).
///
/// ```dart
/// const iast = NameRules(latin: LatinName.iast);
/// ```
final class NameRules {
  const NameRules({
    this.latin = LatinName.refuse,
    this.abhijit = AbhijitPada.refuse,
  });

  final LatinName latin;
  final AbhijitPada abhijit;

  Map<String, Object?> get _record => <String, Object?>{
    'latin': latin.key,
    'abhijit': abhijit.key,
  };
}

/// The readings naam milan is computed under; each default is the
/// source's own.
///
/// ```dart
/// const placed = NaamRules(name: NameRules(abhijit: AbhijitPada.shravana));
/// ```
final class NaamRules {
  const NaamRules({
    this.name = const NameRules(),
    this.koota = const KootaRules(),
    this.porutham = const PoruthamRules(),
  });

  /// How the two names are read.
  final NameRules name;

  /// The readings the Ashta Koota of the name stars is computed under.
  final KootaRules koota;

  /// The readings the ten considerations of the name stars are computed
  /// under.
  final PoruthamRules porutham;

  Map<String, Object?> get _record => <String, Object?>{
    'name': name._record,
    'koota': koota._record,
    'porutham': porutham._record,
  };
}

/// A name's first syllable in the śatapada cakra (*Svarodaya* vv. 3–8).
final class NameSyllable extends _Value {
  const NameSyllable({
    required this.cell,
    required this.nakshatra,
    required this.quarter,
    required this.varga,
  });

  /// Its place among the cakra's 112 cells, 0 for a, Krittika's first.
  final int cell;

  /// Its star, or null for Abhijit, which is none of the 27;
  /// [NameRules.abhijit] decides the star it is matched as.
  final Nakshatra? nakshatra;

  /// Which of the star's four syllables it is, 1 to 4: the pada, for one
  /// of the 27.
  final int quarter;

  /// The letter group the name begins in, as written (VI.35).
  final NameVarga varga;

  @override
  List<Object?> get _fields => [cell, nakshatra, quarter, varga];
}

/// The syllable a child is named by: the birth pada's own cell in the
/// śatapada cakra (C297 to C299).
final class BirthSyllable extends _Value {
  const BirthSyllable({
    required this.cell,
    required this.devanagari,
    required this.iast,
    required this.varga,
  });

  /// Its place among the cakra's 112 cells, 0 for a, Krittika's first.
  final int cell;

  /// As *Muhurta Chintamani* p. 173 prints it, e.g. `चू`.
  final String devanagari;

  /// Its IAST, e.g. `cū`.
  final String iast;

  /// The letter group it begins in (VI.35).
  final NameVarga varga;

  @override
  List<Object?> get _fields => [cell, devanagari, iast, varga];
}

/// What a janma-patrika prints of the Moon: its star and pada, the
/// syllable the child is named by, and the readings the Ashta Koota takes
/// of the same Moon (C301). Vashya, paya, disha and tatwa are not here
/// (C300).
///
/// ```dart
/// final chart = ctx.chart.found(/* … */ avakahada: true);
/// final nameBy = chart.avakahada!.syllable.devanagari;
/// ```
final class Avakahada extends _Value {
  const Avakahada({
    required this.nakshatra,
    required this.pada,
    required this.rashi,
    required this.nakshatraLord,
    required this.rashiLord,
    required this.varna,
    required this.yoni,
    required this.gana,
    required this.nadi,
    required this.syllable,
  });

  /// The Moon's nakshatra.
  final Nakshatra nakshatra;

  /// Its pada, 1 to 4.
  final int pada;

  /// The Moon's sign.
  final Rashi rashi;

  /// The nakshatra's lord, the Vimshottari dasha's.
  final Graha nakshatraLord;

  /// The sign's lord, the one Graha Maitri reads.
  final Graha rashiLord;

  /// The sign's varna, as Varna koota reads it (VI.22).
  final Varna varna;

  /// The nakshatra's yoni.
  final Yoni yoni;

  /// The nakshatra's gana.
  final Gana gana;

  /// The nakshatra's nadi.
  final Nadi nadi;

  /// The syllable the child is named by.
  final BirthSyllable syllable;

  @override
  List<Object?> get _fields => [
    nakshatra,
    pada,
    rashi,
    nakshatraLord,
    rashiLord,
    varna,
    yoni,
    gana,
    nadi,
    syllable,
  ];
}

/// Two names' vargas and how they stand (*Muhurta Chintamani* VI.35,
/// C295).
final class VargaKoota extends _Value {
  const VargaKoota({
    required this.bride,
    required this.groom,
    required this.relation,
  });

  final NameVarga bride;
  final NameVarga groom;

  /// One varga, enemies (each the 5th from the other), or neither.
  final VargaRelation relation;

  @override
  List<Object?> get _fields => [bride, groom, relation];
}

/// Two names matched star to star (naam milan).
final class NaamMilan extends _Value {
  const NaamMilan({
    required this.bride,
    required this.groom,
    required this.varga,
    required this.ashta,
    required this.porutham,
  });

  final NameSyllable bride;
  final NameSyllable groom;
  final VargaKoota varga;

  /// The Ashta Koota of the two name stars, as a chart's match reads two
  /// Moons.
  final AshtaKoota ashta;

  /// The ten considerations of the two name stars, as a chart's match
  /// reads two Moons.
  final Porutham porutham;

  @override
  List<Object?> get _fields => [bride, groom, varga, ashta, porutham];
}

/// What a chart's harmonic is asked (`03-design/western-harmonics.md`):
/// [number], a whole number from 1 to 360 every longitude is multiplied by
/// (Addey), and [orbDeg], how close two points meet in the harmonic chart,
/// 12° by default (C252), at most 30°.
///
/// ```dart
/// const ninth = HarmonicRequest(9);
/// const tight = HarmonicRequest(5, orbDeg: 3);
/// ```
final class HarmonicRequest {
  const HarmonicRequest(this.number, {this.orbDeg = 12});

  /// Which harmonic.
  final int number;

  /// The orb of a meeting in the harmonic chart, degrees.
  final double orbDeg;

  Map<String, Object?> get _record => {'number': number, 'orbDeg': orbDeg};

  String get _json => jsonEncode(_record);
}

/// A point of a harmonic chart: a planet ([HarmonicGraha]), or the
/// ascendant or midheaven ([HarmonicPoint.ascendant],
/// [HarmonicPoint.midheaven]).
sealed class HarmonicPoint extends _Value {
  const HarmonicPoint();

  /// The ascendant.
  static const HarmonicPoint ascendant = HarmonicAngle._('ASCENDANT');

  /// The midheaven.
  static const HarmonicPoint midheaven = HarmonicAngle._('MIDHEAVEN');

  /// `'GRAHA'`, `'ASCENDANT'` or `'MIDHEAVEN'`, as every binding spells it.
  String get point;
}

/// A planet, as a point of a harmonic chart.
final class HarmonicGraha extends HarmonicPoint {
  const HarmonicGraha(this.graha);

  /// Which.
  final Graha graha;

  @override
  String get point => 'GRAHA';

  @override
  List<Object?> get _fields => [graha];
}

/// The ascendant or the midheaven, as a point of a harmonic chart.
final class HarmonicAngle extends HarmonicPoint {
  const HarmonicAngle._(this.point);

  @override
  final String point;

  @override
  List<Object?> get _fields => [point];
}

/// A point's place in a harmonic chart.
final class HarmonicPlaced extends _Value {
  const HarmonicPlaced({
    required this.point,
    required this.longitudeDeg,
    required this.house,
  });

  final HarmonicPoint point;

  /// Its longitude multiplied by the harmonic, degrees in `[0, 360)`.
  final double longitudeDeg;

  /// Its equal house from the harmonic ascendant, 1 to 12 (C254).
  final int house;

  @override
  List<Object?> get _fields => [point, longitudeDeg, house];
}

/// Two points meeting in a harmonic chart, within the orb of each other
/// there: the planets in the catalogue's order first, then the ascendant,
/// then the midheaven.
final class HarmonicRow extends _Value {
  const HarmonicRow({
    required this.first,
    required this.second,
    required this.apartDeg,
    required this.multiple,
    required this.orbDeg,
  });

  final HarmonicPoint first;
  final HarmonicPoint second;

  /// How far apart they stand in the harmonic chart, degrees.
  final double apartDeg;

  /// Which multiple k of the harmonic's aspect, k × 360° / n, they stand at
  /// in the chart itself.
  final int multiple;

  /// The orb the request allowed, degrees.
  final double orbDeg;

  @override
  List<Object?> get _fields => [first, second, apartDeg, multiple, orbDeg];
}

/// A chart's harmonic chart (Addey, *Harmonics in Astrology*), in the
/// chart's own zodiac (C253).
final class HarmonicChart extends _Value {
  const HarmonicChart({
    required this.harmonic,
    required this.points,
    required this.rows,
  });

  /// Which harmonic.
  final int harmonic;

  /// The planets in the catalogue's order, then the ascendant and the
  /// midheaven.
  final List<HarmonicPlaced> points;

  /// The pairs meeting within the orb, closest first.
  final List<HarmonicRow> rows;

  @override
  List<Object?> get _fields => [harmonic, ...points, null, ...rows];
}

/// What a chart's Western houses are asked (`03-design/western-houses.md`):
/// the division, [system]; the profile's `houses.module_overrides.western`
/// when null, else Placidus, the division Leo's figures are cast in (C249).
///
/// ```dart
/// const leo = WesternHouseRequest();
/// const koch = WesternHouseRequest(system: HouseSystem.koch);
/// ```
final class WesternHouseRequest {
  const WesternHouseRequest({this.system});

  final HouseSystem? system;

  Map<String, Object?> get _record => {
    if (system case final system?) 'system': system.fullKey,
  };

  String get _json => jsonEncode(_record);
}

/// What the antiscia are asked (`03-design/western-antiscia.md`): the
/// [orbs] a pair is read under, at the conjunction; Lilly's moieties by
/// default (C244), which give the outer three none. [cusps], the houses
/// whose cusps a reflection is read upon at its very degree (C251),
/// Lilly's Regiomontanus unless it names another division.
///
/// ```dart
/// const leo = AntisciaRequest(orbs: OrbModel.leo);
/// const onCusps = AntisciaRequest(cusps: WesternHouseRequest());
/// ```
final class AntisciaRequest {
  const AntisciaRequest({this.orbs = OrbModel.lilly, this.cusps});

  final OrbModel orbs;
  final WesternHouseRequest? cusps;

  Map<String, Object?> get _record => {
    'orbs': orbs._record,
    if (cusps case final cusps?) 'cusps': cusps._record,
  };

  String get _json => jsonEncode(_record);
}

/// A planet's two reflections, tropical degrees
/// (`03-design/western-antiscia.md`).
final class Antiscion extends _Value {
  const Antiscion({
    required this.graha,
    required this.antiscionDeg,
    required this.contrantiscionDeg,
  });

  final Graha graha;

  /// Its reflection about the solstices: 180° less its longitude.
  final double antiscionDeg;

  /// Its reflection about the equinoxes: 360° less its longitude.
  final double contrantiscionDeg;

  @override
  List<Object?> get _fields => [graha, antiscionDeg, contrantiscionDeg];
}

/// Two planets in antiscion within the orb: in a chart's own pair the two
/// in catalogue order, across a synastry the chart's [first] and the
/// partner's [second].
final class AntiscionRow extends _Value {
  const AntiscionRow({
    required this.first,
    required this.second,
    required this.contrary,
    required this.apartDeg,
    required this.orbDeg,
  });

  final Graha first;
  final Graha second;

  /// Whether it is the contrantiscion, the reflection about the equinoxes.
  final bool contrary;

  /// How far the one's reflection stands from the other, degrees.
  final double apartDeg;

  /// The orb the request allowed the pair, degrees.
  final double orbDeg;

  @override
  List<Object?> get _fields => [first, second, contrary, apartDeg, orbDeg];
}

/// A chart's antiscia (Lilly, *Christian Astrology*, pp. 90–92): each
/// planet's reflections, the pairs within the orb closest first, and the
/// planets the orbs give none.
final class Antiscia extends _Value {
  const Antiscia({
    required this.points,
    required this.pairs,
    required this.unpaired,
    this.onCusps = const [],
    this.cuspSystem,
  });

  final List<Antiscion> points;
  final List<AntiscionRow> pairs;
  final List<Graha> unpaired;

  /// The reflections upon a cusp's very degree (C251); empty unless the
  /// request asked [AntisciaRequest.cusps].
  final List<CuspAntiscion> onCusps;

  /// The division [onCusps] was read in; null unless asked.
  final HouseSystem? cuspSystem;

  @override
  List<Object?> get _fields => [
    ...points,
    null,
    ...pairs,
    null,
    ...unpaired,
    null,
    ...onCusps,
    cuspSystem,
  ];
}

/// A planet's reflection upon a cusp's very degree, its own sign and whole
/// degree (Lilly, *Christian Astrology*, p. 165; C251).
final class CuspAntiscion extends _Value {
  const CuspAntiscion({
    required this.graha,
    required this.house,
    required this.contrary,
  });

  final Graha graha;

  /// The house whose cusp it falls upon, 1 to 12.
  final int house;

  /// Whether it is the contrantiscion, the reflection about the equinoxes.
  final bool contrary;

  @override
  List<Object?> get _fields => [graha, house, contrary];
}

/// Where a planet is counted among a chart's Western houses.
final class WesternHousePlacement extends _Value {
  const WesternHousePlacement({
    required this.graha,
    required this.house,
    required this.withAscendant,
  });

  final Graha graha;

  /// The house whose cusp it has passed and whose next cusp it has not.
  final int house;

  /// Whether Leo counts it with the ascendant (C250): in the first house,
  /// or above the ascendant no further than the degree that rose one
  /// sidereal hour before; its house is never moved for it.
  final bool withAscendant;

  @override
  List<Object?> get _fields => [graha, house, withAscendant];
}

/// A chart's Western houses (`03-design/western-houses.md`), in the
/// chart's own zodiac.
final class WesternHouses extends _Value {
  const WesternHouses({
    required this.system,
    required this.cuspsDeg,
    required this.ascendantDeg,
    required this.reachDeg,
    required this.planets,
  });

  /// The division the cusps are of: the one asked, or the one a polar
  /// policy fell back to.
  final HouseSystem system;

  /// The twelve cusps, first to twelfth, degrees.
  final List<double> cuspsDeg;

  final double ascendantDeg;

  /// The degree that rose one sidereal hour before the birth (C250).
  final double reachDeg;

  final List<WesternHousePlacement> planets;

  @override
  List<Object?> get _fields => [
    system,
    ...cuspsDeg,
    ascendantDeg,
    reachDeg,
    ...planets,
  ];
}

/// One pair of planets within an aspect's orb, the pair in catalogue
/// order (`03-design/western-aspects.md`).
final class WesternAspectRow extends _Value {
  const WesternAspectRow({
    required this.first,
    required this.second,
    required this.aspect,
    required this.apartDeg,
    required this.fromExactDeg,
    required this.orbDeg,
    required this.applying,
  });

  final Graha first;
  final Graha second;
  final WesternAspect aspect;

  /// The shorter arc between them, degrees 0 to 180.
  final double apartDeg;

  /// How far that arc is from the aspect's exact angle, degrees.
  final double fromExactDeg;

  /// The orb the model allowed the pair at this aspect, degrees.
  final double orbDeg;

  /// Whether the faster planet is closing on the exact angle.
  final bool applying;

  @override
  List<Object?> get _fields => [
    first,
    second,
    aspect,
    apartDeg,
    fromExactDeg,
    orbDeg,
    applying,
  ];
}

/// How close two distances from the equator must stand to be a parallel
/// (`03-design/western-declinations.md`): Leo's 1° (*How to Judge a
/// Nativity*, p. 47) by default, at most 10°.
///
/// ```dart
/// const leo = ParallelRequest();
/// const wider = ParallelRequest(orbDeg: 1.5);
/// ```
final class ParallelRequest {
  const ParallelRequest({this.orbDeg = 1});

  /// The orb, degrees.
  final double orbDeg;

  Map<String, Object?> get _record => <String, Object?>{'orbDeg': orbDeg};

  String get _json => jsonEncode(_record);
}

/// How far from the axis through two planets' midpoint a third may stand
/// to be equally distant from them (`03-design/western-midpoints.md`):
/// 0.5° by default (C245), at most 10°.
///
/// ```dart
/// const standard = MidpointRequest();
/// const wider = MidpointRequest(orbDeg: 1.5);
/// ```
final class MidpointRequest {
  const MidpointRequest({this.orbDeg = 0.5});

  /// The orb from the nearer point of the axis, degrees.
  final double orbDeg;

  Map<String, Object?> get _record => {'orbDeg': orbDeg};

  String get _json => jsonEncode(_record);
}

/// A planet equally distant from two others (Leo, *How to Judge a
/// Nativity*, pp. 47–48), within the orb of the axis through their
/// midpoint: the pair in catalogue order, and the planet between.
final class MidpointRow extends _Value {
  const MidpointRow({
    required this.first,
    required this.second,
    required this.middle,
    required this.far,
    required this.distanceDeg,
    required this.fromAxisDeg,
    required this.orbDeg,
  });

  final Graha first;
  final Graha second;

  /// The planet equally distant from the two.
  final Graha middle;

  /// Whether it stands opposite the midpoint of the pair's shorter arc, on
  /// the longer arc's midpoint (C246).
  final bool far;

  /// How far it stands from each of the two, the mean of the two arcs,
  /// degrees.
  final double distanceDeg;

  /// How far it stands from the nearer point of the axis, degrees.
  final double fromAxisDeg;

  /// The orb the request allowed, degrees.
  final double orbDeg;

  @override
  List<Object?> get _fields => [
    first,
    second,
    middle,
    far,
    distanceDeg,
    fromAxisDeg,
    orbDeg,
  ];
}

/// An equal distance across a synastry (`03-design/western-midpoints.md`,
/// decision 9): a planet of one chart on the axis through two of the
/// other's. [partnersPair] is true when the pair is the partner's and
/// [middle] the chart's planet.
final class SynastryMidpointRow extends MidpointRow {
  const SynastryMidpointRow({
    required super.first,
    required super.second,
    required super.middle,
    required this.partnersPair,
    required super.far,
    required super.distanceDeg,
    required super.fromAxisDeg,
    required super.orbDeg,
  });

  SynastryMidpointRow._of(MidpointRow row, {required this.partnersPair})
    : super(
        first: row.first,
        second: row.second,
        middle: row.middle,
        far: row.far,
        distanceDeg: row.distanceDeg,
        fromAxisDeg: row.fromAxisDeg,
        orbDeg: row.orbDeg,
      );

  /// Whether the pair is the partner's and the planet between it the
  /// chart's.
  final bool partnersPair;

  @override
  List<Object?> get _fields => [...super._fields, partnersPair];
}

/// A planet's distance from the equator.
final class Declined extends _Value {
  const Declined({required this.graha, required this.declinationDeg});

  final Graha graha;

  /// Degrees north of the equator.
  final double declinationDeg;

  @override
  List<Object?> get _fields => [graha, declinationDeg];
}

/// A chart's distances from the equator, degrees north
/// (`03-design/western-declinations.md`).
final class Declinations extends _Value {
  const Declinations({
    required this.obliquityDeg,
    required this.grahas,
    required this.lagnaDeg,
    required this.midheavenDeg,
  });

  /// The true obliquity at the chart's instant, which turned every one.
  final double obliquityDeg;

  /// The planets, in the catalogue's order.
  final List<Declined> grahas;

  /// The lagna's: the Sun's at that degree (Leo, p. 141).
  final double lagnaDeg;

  /// The midheaven's, read the same way.
  final double midheavenDeg;

  /// One planet's declination, when the chart placed it.
  double? graha(Graha graha) {
    for (final one in grahas) {
      if (one.graha == graha) return one.declinationDeg;
    }
    return null;
  }

  @override
  List<Object?> get _fields => [
    obliquityDeg,
    ...grahas,
    lagnaDeg,
    midheavenDeg,
  ];
}

/// One pair of planets the same distance from the equator within the orb,
/// the pair in catalogue order.
final class ParallelRow extends _Value {
  const ParallelRow({
    required this.first,
    required this.second,
    required this.contrary,
    required this.apartDeg,
    required this.orbDeg,
  });

  final Graha first;
  final Graha second;

  /// Whether the two stand on opposite sides of the equator (C243).
  final bool contrary;

  /// How far apart their distances from the equator are, degrees.
  final double apartDeg;

  /// The orb the request allowed, degrees.
  final double orbDeg;

  @override
  List<Object?> get _fields => [first, second, contrary, apartDeg, orbDeg];
}

/// The birth every chart of a batch is read against in a synastry: its
/// instant, where it happened and the clock kept there.
///
/// ```dart
/// final mary = Partner(
///   instant: 2403113.4993,
///   place: Observer(latitudeDeg: Latitude(51.5058), longitudeDeg: Longitude(-0.1878), altitudeM: Altitude(0)),
/// );
/// ```
final class Partner {
  const Partner({
    required this.instant,
    required this.place,
    this.utcOffsetSeconds = 0,
  });

  /// The birth's instant, a Julian day in UTC.
  final double instant;

  /// Where it happened.
  final Observer place;

  /// The partner's clock, seconds east of UTC; UTC by default.
  final int utcOffsetSeconds;

  Map<String, Object?> get _record => <String, Object?>{
    'instant': instant,
    'place': <String, Object?>{
      'latitude': place.latitudeDeg,
      'longitude': place.longitudeDeg,
      'altitude': place.altitudeM,
    },
    'utcOffsetSeconds': utcOffsetSeconds,
  };
}

/// Which zodiac a synastry compares two charts in (C241).
enum SynastryZodiac {
  /// Both charts' tropical longitudes, Leo's frame; the default.
  tropical('TROPICAL'),

  /// Each chart's longitudes as founded, for a sidereal reader; two charts
  /// founded in different zodiacs are refused.
  charts('CHARTS');

  const SynastryZodiac(this.key);

  /// The member's key, as every binding spells it.
  final String key;
}

/// A synastry: every chart of a batch read against one [partner]'s birth
/// (`03-design/western-synastry.md`), under the aspects and orbs a chart's
/// own [table] reads, each chart's [lagna] beside its planets, in a
/// [zodiac]: Leo's nine under his orbs, the lagna read, tropically by
/// default (C240–C242). [parallels] asks for the parallels across the two
/// charts too ([Chart.synastryParallels],
/// `03-design/western-declinations.md`), [antiscia] the antiscia across
/// them ([Chart.synastryAntiscia], `03-design/western-antiscia.md`),
/// [midpoints] the equal distances across them ([Chart.synastryMidpoints],
/// `03-design/western-midpoints.md`), [composite] the composite of the two
/// ([Chart.synastryComposite], C247)
/// and [davison] each chart's Davison birth with the partner
/// ([Chart.synastryDavison], C248; `03-design/western-composites.md`).
///
/// ```dart
/// final asked = SynastryRequest(mary, lagna: false);
/// final lilly = SynastryRequest.lilly(mary);
/// final level = SynastryRequest(mary, parallels: const ParallelRequest());
/// final mirrored = SynastryRequest(mary, antiscia: const AntisciaRequest());
/// final oneChart = SynastryRequest(mary, composite: true, davison: true);
/// ```
final class SynastryRequest {
  const SynastryRequest(
    this.partner, {
    this.table = const WesternAspectRequest(),
    this.lagna = true,
    this.zodiac = SynastryZodiac.tropical,
    this.parallels,
    this.antiscia,
    this.midpoints,
    this.composite = false,
    this.davison = false,
  });

  /// Lilly's reading: the Ptolemaic five under his moieties, which give the
  /// lagna no orb, so it is left out.
  const SynastryRequest.lilly(
    this.partner, {
    this.zodiac = SynastryZodiac.tropical,
    this.parallels,
    this.antiscia,
    this.midpoints,
    this.composite = false,
    this.davison = false,
  }) : table = WesternAspectRequest.lilly,
       lagna = false;

  /// Whose birth every chart is read against.
  final Partner partner;

  /// The aspects, and the orbs they are read under.
  final WesternAspectRequest table;

  /// Whether each chart's lagna joins its planets; it stands as a planet in
  /// Leo's orbs (C242), and the moieties give it none.
  final bool lagna;

  /// The zodiac the two are compared in.
  final SynastryZodiac zodiac;

  /// The orb the parallels across are read under; none are read when null.
  final ParallelRequest? parallels;

  /// The orbs the antiscia across are read under; none are read when null.
  final AntisciaRequest? antiscia;

  /// The orb the equal distances across are read under; none are read when
  /// null.
  final MidpointRequest? midpoints;

  /// Whether the composite of the two charts is made too.
  final bool composite;

  /// Whether each chart's Davison birth with the partner is given too.
  final bool davison;

  String get _json => jsonEncode(<String, Object?>{
    'partner': partner._record,
    ...table._record,
    'lagna': lagna,
    'zodiac': zodiac.key,
    if (parallels case final asked?) 'parallels': asked._record,
    if (antiscia case final asked?) 'antiscia': asked._record,
    if (midpoints case final asked?) 'midpoints': asked._record,
    if (composite) 'composite': true,
    if (davison) 'davison': true,
  });
}

/// A planet of a composite chart: the near midpoint of its two places,
/// degrees in the synastry's zodiac, moving at the mean of its two speeds.
final class CompositePlanet extends _Value {
  const CompositePlanet({
    required this.graha,
    required this.longitudeDeg,
    required this.speedDegPerDay,
  });

  final Graha graha;

  /// The near midpoint of its two places, degrees.
  final double longitudeDeg;

  /// The mean of its two speeds, degrees a day; negative when retrograde.
  final double speedDegPerDay;

  @override
  List<Object?> get _fields => [graha, longitudeDeg, speedDegPerDay];
}

/// The composite of a chart and a synastry's partner
/// (`03-design/western-composites.md`, C247): its [planets] in the
/// chart's order, the midheaven at the near midpoint of the two, and the
/// lagna at the near midpoint of the two lagnas, turned by 180° when that
/// stood before the midheaven, as [lagnaTurned] says.
final class Composite extends _Value {
  const Composite({
    required this.planets,
    required this.lagnaDeg,
    required this.midheavenDeg,
    required this.lagnaTurned,
    this.cuspsDeg,
  });

  final List<CompositePlanet> planets;

  /// The composite lagna, degrees.
  final double lagnaDeg;

  /// The composite midheaven, degrees.
  final double midheavenDeg;

  /// Whether the lagnas' near midpoint stood before the midheaven and was
  /// turned by 180°.
  final bool lagnaTurned;

  /// The twelve composite cusps, each the near midpoint of the two charts'
  /// same cusp, turned when more than 90° from where the midheaven puts it
  /// (Astrolog); null when either chart's cusps could not be read, as at a
  /// polar place.
  final List<double>? cuspsDeg;

  @override
  List<Object?> get _fields => [
    planets,
    lagnaDeg,
    midheavenDeg,
    lagnaTurned,
    cuspsDeg,
  ];
}

/// One point of a chart and one of the partner's within an aspect's orb
/// (`03-design/western-synastry.md`): [first] is the chart's, [second] the
/// partner's.
final class SynastryRow extends _Value {
  const SynastryRow({
    required this.first,
    required this.second,
    required this.aspect,
    required this.apartDeg,
    required this.fromExactDeg,
    required this.orbDeg,
  });

  final NatalPoint first;
  final NatalPoint second;
  final WesternAspect aspect;

  /// The shorter arc between them, degrees 0 to 180.
  final double apartDeg;

  /// How far that arc is from the aspect's exact angle, degrees.
  final double fromExactDeg;

  /// The orb the model allowed the pair at this aspect, degrees.
  final double orbDeg;

  @override
  List<Object?> get _fields => [
    first,
    second,
    aspect,
    apartDeg,
    fromExactDeg,
    orbDeg,
  ];
}

/// One point of a chart and one of the partner's the same distance from the
/// equator within the orb (`03-design/western-declinations.md`): [first]
/// is the chart's, [second] the partner's.
final class SynastryParallelRow extends _Value {
  const SynastryParallelRow({
    required this.first,
    required this.second,
    required this.contrary,
    required this.apartDeg,
    required this.orbDeg,
  });

  final NatalPoint first;
  final NatalPoint second;

  /// Whether the two stand on opposite sides of the equator (C243).
  final bool contrary;

  /// How far apart their distances from the equator are, degrees.
  final double apartDeg;

  /// The orb the request allowed, degrees.
  final double orbDeg;

  @override
  List<Object?> get _fields => [first, second, contrary, apartDeg, orbDeg];
}

/// A planet of the progressed chart.
final class ProgressedPlanet extends _Value {
  const ProgressedPlanet({
    required this.graha,
    required this.longitudeDeg,
    required this.tropicalDeg,
    required this.speedDegPerDay,
  });

  final Graha graha;

  /// Its longitude in the chart's zodiac, degrees.
  final double longitudeDeg;
  final double tropicalDeg;

  /// Degrees a day at the instant of sky; below zero when retrograde.
  final double speedDegPerDay;

  @override
  List<Object?> get _fields => [
    graha,
    longitudeDeg,
    tropicalDeg,
    speedDegPerDay,
  ];
}

/// The progressed angles, by the request's [AngleMethod] (C237).
final class ProgressedAngles extends _Value {
  const ProgressedAngles({
    required this.ascendantDeg,
    required this.midheavenDeg,
  });

  final double ascendantDeg;
  final double midheavenDeg;

  @override
  List<Object?> get _fields => [ascendantDeg, midheavenDeg];
}

/// The chart at the instant of sky that measures an instant of life.
final class Progressed extends _Value {
  const Progressed({
    required this.life,
    required this.sky,
    required this.armcDeg,
    required this.angles,
    required this.grahas,
  });

  final double life;
  final double sky;

  /// The progressed meridian's right ascension, degrees.
  final double armcDeg;
  final ProgressedAngles angles;
  final List<ProgressedPlanet> grahas;

  @override
  List<Object?> get _fields => [life, sky, armcDeg, angles, grahas];
}

/// A birth's planet moved by the direction's arc.
final class DirectedPlanet extends _Value {
  const DirectedPlanet({required this.graha, required this.longitudeDeg});

  final Graha graha;
  final double longitudeDeg;

  @override
  List<Object?> get _fields => [graha, longitudeDeg];
}

/// A birth's points moved by one arc.
final class Directed extends _Value {
  const Directed({
    required this.life,
    required this.arcDeg,
    required this.ascendantDeg,
    required this.midheavenDeg,
    required this.planets,
  });

  final double life;

  /// The arc, degrees; a solar arc is signed.
  final double arcDeg;
  final double ascendantDeg;
  final double midheavenDeg;
  final List<DirectedPlanet> planets;

  @override
  List<Object?> get _fields => [
    life,
    arcDeg,
    ascendantDeg,
    midheavenDeg,
    planets,
  ];
}

/// One exact aspect a progressed planet makes to a radical point.
final class ProgressedContact extends _Value {
  const ProgressedContact({
    required this.life,
    required this.sky,
    required this.graha,
    required this.to,
    required this.angle,
    required this.motion,
  });

  /// The instant of life it falls due, a UTC Julian day.
  final double life;

  /// The instant of sky it is exact at.
  final double sky;
  final Graha graha;
  final NatalPoint to;

  /// A whole degree 0 to 180.
  final int angle;
  final Motion motion;

  @override
  List<Object?> get _fields => [life, sky, graha, to, angle, motion];
}

/// A birth read through its progressions: [progressed] and [directed] are
/// null without [ProgressionsRequest.at], [contacts] null without a window.
final class Progressions extends _Value {
  const Progressions({
    required this.progressed,
    required this.directed,
    required this.contacts,
  });

  final Progressed? progressed;
  final Directed? directed;
  final List<ProgressedContact>? contacts;

  @override
  List<Object?> get _fields => [progressed, directed, contacts];
}

/// Whether a horary matter is brought to pass (Lilly, *Christian
/// Astrology* pp. 107–113 and 125–127,
/// `03-design/hellenistic-perfection.md`): the quesited's significator
/// named, or the house of the matter, whose cusp's lord signifies it; the
/// querent's is the Ascendant's lord unless named.
///
/// ```dart
/// final chart = ctx.chart.found(
///   /* … */ perfection: const PerfectionRequest.ofHouse(7),
/// );
/// final perfects = chart.perfection?.ways.held.isNotEmpty ?? false;
/// ```
final class PerfectionRequest extends _Value {
  /// Every field as written; the SDK refuses neither or both of `quesited`
  /// and `house`, by the field it names.
  const PerfectionRequest({
    this.querent,
    this.quesited,
    this.house,
    this.rules = PerfectionRules.lilly,
  });

  /// The matter of house [house], 1 to 12.
  const PerfectionRequest.ofHouse(
    int this.house, {
    this.querent,
    this.rules = PerfectionRules.lilly,
  }) : quesited = null;

  /// The matter between two named significators.
  const PerfectionRequest.between(
    Graha this.querent,
    Graha this.quesited, {
    this.rules = PerfectionRules.lilly,
  }) : house = null;

  /// The querent's significator; null, the Ascendant's lord.
  final Graha? querent;

  /// The quesited's significator, one of the seven.
  final Graha? quesited;

  /// The house of the matter, 1 to 12.
  final int? house;

  final PerfectionRules rules;

  String get _json => jsonEncode(<String, Object?>{
    if (querent case final querent?) 'querent': querent.fullKey,
    if (quesited case final quesited?) 'quesited': quesited.fullKey,
    if (house != null) 'house': house,
    'rules': rules._record,
  });

  @override
  List<Object?> get _fields => [querent, quesited, house, rules];
}

/// The significators coming to an aspect (p. 107).
final class Application extends _Value {
  const Application({
    required this.aspect,
    required this.days,
    required this.applying,
    required this.kind,
    required this.gapDeg,
    required this.withinMoieties,
  });

  final PtolemaicAspect aspect;

  /// Days until it is exact.
  final double days;

  /// The significator whose motion closes it.
  final Graha applying;
  final ApplicationKind kind;

  /// How far it is from exact now, degrees.
  final double gapDeg;

  /// Whether the gap is already within the two planets' moieties of orb.
  final bool withinMoieties;

  @override
  List<Object?> get _fields => [
    aspect,
    days,
    applying,
    kind,
    gapDeg,
    withinMoieties,
  ];
}

/// Two planets past an aspect and still within their moieties (p. 110).
final class Separation extends _Value {
  const Separation({required this.aspect, required this.pastDeg});

  final PtolemaicAspect aspect;

  /// How far past exact, degrees.
  final double pastDeg;

  @override
  List<Object?> get _fields => [aspect, pastDeg];
}

/// What stops or hinders the application (pp. 110–113).
final class Impediment extends _Value {
  const Impediment({
    required this.kind,
    required this.significator,
    required this.third,
    required this.aspect,
    required this.days,
  });

  final ImpedimentKind kind;

  /// The significator it falls on.
  final Graha significator;

  /// The third planet; null for a refranation.
  final Graha? third;

  /// The aspect the third perfects, or the one refrained from.
  final PtolemaicAspect aspect;

  /// Days until the contact, or the station.
  final double days;

  @override
  List<Object?> get _fields => [kind, significator, third, aspect, days];
}

/// A lighter planet carrying one significator's light to the other
/// (p. 111).
final class Translation extends _Value {
  const Translation({
    required this.translator,
    required this.from,
    required this.to,
    required this.separating,
    required this.aspect,
    required this.days,
    required this.received,
  });

  final Graha translator;

  /// The significator it separates from.
  final Graha from;

  /// The significator it applies to next.
  final Graha to;
  final Separation separating;

  /// The aspect it applies to [to] by.
  final PtolemaicAspect aspect;

  /// Days until that is exact.
  final double days;

  /// The dignities of [from] the translator stands in: how it is received
  /// (p. 126).
  final EssentialDignity received;

  @override
  List<Object?> get _fields => [
    translator,
    from,
    to,
    separating,
    aspect,
    days,
    received,
  ];
}

/// A significator's application to a collector.
final class ContactAhead extends _Value {
  const ContactAhead({required this.aspect, required this.days});

  final PtolemaicAspect aspect;

  /// Days until it is exact.
  final double days;

  @override
  List<Object?> get _fields => [aspect, days];
}

/// A heavier planet both significators apply to (p. 112); who must
/// receive whom is C233.
final class Collection extends _Value {
  const Collection({
    required this.collector,
    required this.fromQuerent,
    required this.fromQuesited,
    required this.collectorInQuerent,
    required this.collectorInQuesited,
    required this.querentInCollector,
    required this.quesitedInCollector,
  });

  final Graha collector;
  final ContactAhead fromQuerent;
  final ContactAhead fromQuesited;
  final EssentialDignity collectorInQuerent;
  final EssentialDignity collectorInQuesited;
  final EssentialDignity querentInCollector;
  final EssentialDignity quesitedInCollector;

  @override
  List<Object?> get _fields => [
    collector,
    fromQuerent,
    fromQuesited,
    collectorInQuerent,
    collectorInQuesited,
    querentInCollector,
    quesitedInCollector,
  ];
}

/// Where a significator stands.
final class SignificatorPlace extends _Value {
  const SignificatorPlace({
    required this.planet,
    required this.house,
    required this.dignity,
  });

  final Graha planet;

  /// 1 to 12.
  final int house;

  /// Its own dignities at its degree.
  final EssentialDignity dignity;

  @override
  List<Object?> get _fields => [planet, house, dignity];
}

/// The ways of perfection (pp. 125–127): what they weigh, and which hold.
final class Ways extends _Value {
  const Ways({
    required this.querent,
    required this.quesited,
    required this.mutualByHouse,
    required this.infortunesBetween,
    required this.moonRelays,
    required this.quesitedInAscendant,
    required this.held,
  });

  final SignificatorPlace querent;
  final SignificatorPlace quesited;

  /// Each stands in the other's house.
  final bool mutualByHouse;

  /// Saturn and Mars among the thirds that come between the significators
  /// before they perfect.
  final List<Graha> infortunesBetween;

  /// The Moon, neither significator, separating from the quesited's and
  /// coming next to the querent's.
  final bool moonRelays;

  /// The quesited's significator in the first house.
  final bool quesitedInAscendant;

  /// The ways the figure holds, in [Way]'s order.
  final List<Way> held;

  @override
  List<Object?> get _fields => [
    querent,
    quesited,
    mutualByHouse,
    infortunesBetween,
    moonRelays,
    quesitedInAscendant,
    held,
  ];
}

/// Whether a horary matter is brought to pass: the relations between two
/// significators with the facts each rests on, never a verdict
/// (`03-design/hellenistic-perfection.md`).
final class Matter extends _Value {
  const Matter({
    required this.querent,
    required this.quesited,
    required this.application,
    required this.separation,
    required this.impediments,
    required this.translations,
    required this.collections,
    required this.ways,
    required this.horizonDays,
    required this.rules,
  });

  final Graha querent;
  final Graha quesited;

  /// Their application within the horizon; null when none.
  final Application? application;

  /// Their separation at the figure; null when none.
  final Separation? separation;
  final List<Impediment> impediments;
  final List<Translation> translations;
  final List<Collection> collections;
  final Ways ways;

  /// How many days ahead it was read.
  final double horizonDays;

  /// The rules it was read under, as asked.
  final PerfectionRules rules;

  @override
  List<Object?> get _fields => [
    querent,
    quesited,
    application,
    separation,
    impediments,
    translations,
    collections,
    ways,
    horizonDays,
    rules,
  ];
}

/// A Ptolemaic aspect the Moon perfects with another planet, and how far
/// off it is.
final class Perfection extends _Value {
  const Perfection({
    required this.planet,
    required this.aspect,
    required this.days,
    required this.gapDeg,
  });

  final Graha planet;
  final PtolemaicAspect aspect;

  /// Days until it is exact, at the motions of the moment.
  final double days;

  /// How far it is from exact now, degrees: the arc the two close, which
  /// a reading by the moieties weighs (C230).
  final double gapDeg;

  @override
  List<Object?> get _fields => [planet, aspect, days, gapDeg];
}

/// The Moon's course to the end of her sign, read both ways Lilly's
/// figures support (C230).
final class MoonCourse extends _Value {
  const MoonCourse({
    required this.next,
    required this.withinOrb,
    required this.daysInSign,
    required this.eased,
  });

  /// Her first perfection before she leaves her sign; null when she makes
  /// none, void by the modern reading.
  final Perfection? next;

  /// Her first perfection already within the two planets' moieties; null
  /// when none is, void by the moieties.
  final Perfection? withinOrb;

  /// Days until she leaves her sign.
  final double daysInSign;

  /// Taurus, Cancer, Sagittarius or Pisces, where void "somewhat she
  /// performes" (p. 122).
  final bool eased;

  @override
  List<Object?> get _fields => [next, withinOrb, daysInSign, eased];
}

/// Whether the question is radical: the lord of the hour against the lord
/// of the Ascendant, with every ground that holds.
final class Radicality extends _Value {
  const Radicality({
    required this.hourLord,
    required this.ascendantLord,
    required this.grounds,
  });

  final Graha hourLord;
  final Graha ascendantLord;

  /// Every ground that holds, in id order; empty when none does.
  final List<RadicalGround> grounds;

  @override
  List<Object?> get _fields => [hourLord, ascendantLord, grounds];
}

/// The Ascendant's clause: too early, too late, or in a sign of short
/// ascension.
final class AscendantClause extends _Value {
  const AscendantClause({
    required this.sign,
    required this.degree,
    required this.early,
    required this.late,
    required this.shortAscension,
  });

  final Rashi sign;

  /// Degrees within the sign.
  final double degree;

  /// Under 3°.
  final bool early;

  /// 27° or more.
  final bool late;

  /// Capricorn to Gemini.
  final bool shortAscension;

  @override
  List<Object?> get _fields => [sign, degree, early, late, shortAscension];
}

/// The Moon's clause: late in her sign, in a sign Lilly names, in the via
/// combusta, and her course.
final class MoonClause extends _Value {
  const MoonClause({
    required this.sign,
    required this.degree,
    required this.late,
    required this.lateSign,
    required this.viaCombusta,
    required this.course,
  });

  final Rashi sign;

  /// Degrees within the sign.
  final double degree;

  /// At or past the rules' `moonLateFromDeg` (C229).
  final bool late;

  /// In Gemini, Scorpio or Capricorn.
  final bool lateSign;

  /// Between Libra 15° and Scorpio 15°.
  final bool viaCombusta;

  final MoonCourse course;

  @override
  List<Object?> get _fields => [
    sign,
    degree,
    late,
    lateSign,
    viaCombusta,
    course,
  ];
}

/// The seventh house's clause, which Lilly says reads the astrologer
/// (C231).
final class SeventhClause extends _Value {
  const SeventhClause({
    required this.cuspDeg,
    required this.lord,
    required this.infortunesInHouse,
    required this.lordRetrograde,
    required this.lordCombust,
    required this.lordInFall,
    required this.lordInInfortuneTerm,
    required this.lordNet,
  });

  /// The seventh cusp's longitude, degrees in [0, 360).
  final double cuspDeg;
  final Graha lord;

  /// Saturn or Mars counted in the seventh house, in id order: what
  /// afflicts the cusp, by one reading of C231.
  final List<Graha> infortunesInHouse;
  final bool lordRetrograde;
  final bool lordCombust;
  final bool lordInFall;

  /// In a term of Saturn or Mars.
  final bool lordInInfortuneTerm;

  /// The lord's net strength over Lilly's table, which says whether he is
  /// "unfortunate".
  final int lordNet;

  @override
  List<Object?> get _fields => [
    cuspDeg,
    lord,
    infortunesInHouse,
    lordRetrograde,
    lordCombust,
    lordInFall,
    lordInInfortuneTerm,
    lordNet,
  ];
}

/// A chart's considerations before judgement (Lilly, *Christian
/// Astrology* I.XIX), each clause with the facts it rests on and never a
/// verdict (`03-design/hellenistic-considerations.md`).
final class Considerations extends _Value {
  const Considerations({
    required this.radicality,
    required this.ascendant,
    required this.moon,
    required this.seventh,
    required this.saturnHouse,
    required this.saturnRetrograde,
    required this.ascendantLordCombust,
    required this.rules,
  });

  final Radicality radicality;
  final AscendantClause ascendant;
  final MoonClause moon;
  final SeventhClause seventh;

  /// Saturn's house, 1 to 12; Lilly's cautions name the first and seventh.
  final int saturnHouse;

  /// Saturn retrograde, which makes him in the Ascendant the worse.
  final bool saturnRetrograde;

  /// Whether the lord of the Ascendant is combust.
  final bool ascendantLordCombust;

  /// The rules they were read under, every field filled.
  final ConsiderationRules rules;

  @override
  List<Object?> get _fields => [
    radicality,
    ascendant,
    moon,
    seventh,
    saturnHouse,
    saturnRetrograde,
    ascendantLordCombust,
    rules,
  ];
}

/// A KP reading to make of every chart of a request (`03-design/kp.md`),
/// every field optional. Each chart's reading comes back as its `kp`, under
/// the settings' `kp` group.
///
/// ```dart
/// final chart = ctx.chart.found(/* … */ kp: const KpRequest(number: 74));
/// final standing = chart.kp?.ruling.accepted;
/// ```
final class KpRequest {
  const KpRequest({this.number, this.clock, this.anyAyanamsha = false});

  /// The querent's horary number, 1 to 249, or the SDK refuses it by
  /// `kp.number`: the cusps are cast from it, the ruling planets stay the
  /// moment's (C156).
  final int? number;

  /// Seconds east of UT that the civil day lord is the weekday on; the
  /// request's own `utcOffsetSeconds` when absent (C151).
  final int? clock;

  /// Whether to read a chart whose zodiac is not Krishnamurti's, which is
  /// refused by `frame.ayanamsha` otherwise (C157).
  final bool anyAyanamsha;

  String get _json => jsonEncode(<String, Object?>{
    if (number != null) 'number': number,
    if (clock != null) 'clock': clock,
    'anyAyanamsha': anyAyanamsha,
  });
}

/// An arc of the zodiac, half-open, in nanoarcseconds (divide by `3.6e12`
/// for degrees).
final class KpSpan extends _Value {
  const KpSpan({required this.start, required this.end});

  final int start;
  final int end;

  @override
  List<Object?> get _fields => [start, end];
}

/// One level of a point's lords below the sign: its lord, and the arc it
/// rules.
final class KpLevel extends _Value {
  const KpLevel({required this.lord, required this.span});

  final Graha lord;
  final KpSpan span;

  @override
  List<Object?> get _fields => [lord, span];
}

/// A point's lords: of its sign, its star, its sub and its sub-sub.
final class KpLords extends _Value {
  const KpLords({
    required this.sign,
    required this.star,
    required this.sub,
    required this.subSub,
  });

  final Graha sign;
  final KpLevel star;
  final KpLevel sub;
  final KpLevel subSub;

  @override
  List<Object?> get _fields => [sign, star, sub, subSub];
}

/// A cusp, its longitude in nanoarcseconds of the sidereal zodiac.
final class KpCusp extends _Value {
  const KpCusp({
    required this.house,
    required this.longitude,
    required this.lords,
  });

  /// 1 to 12.
  final int house;
  final int longitude;
  final KpLords lords;

  @override
  List<Object?> get _fields => [house, longitude, lords];
}

/// A planet, its longitude in nanoarcseconds of the sidereal zodiac.
final class KpPlanet extends _Value {
  const KpPlanet({
    required this.graha,
    required this.longitude,
    required this.retrograde,
    required this.house,
    required this.lords,
  });

  final Graha graha;
  final int longitude;
  final bool retrograde;

  /// The house whose cusp arc holds it, 1 to 12.
  final int house;
  final KpLords lords;

  @override
  List<Object?> get _fields => [graha, longitude, retrograde, house, lords];
}

/// A chart as KP reads it: its cusps, the horary number's when one was
/// named, and its planets.
final class KpChart extends _Value {
  const KpChart({
    required this.system,
    required this.cusps,
    required this.planets,
  });

  final HouseSystem system;
  final List<KpCusp> cusps;
  final List<KpPlanet> planets;

  @override
  List<Object?> get _fields => [system, cusps, planets];
}

/// A house's significators in KP Reader VI's order, strongest first (C154).
final class KpHouseSignificators extends _Value {
  const KpHouseSignificators({
    required this.house,
    required this.inOccupantsStars,
    required this.occupants,
    required this.inLordsStar,
    required this.lord,
    required this.conjoined,
    required this.aspected,
    required this.intercepted,
  });

  final int house;

  /// (a) Planets in the stars of the house's occupants.
  final List<Graha> inOccupantsStars;

  /// (b) The occupants.
  final List<Graha> occupants;

  /// (c) Planets in the star of the house's lord.
  final List<Graha> inLordsStar;

  /// (d) The house's lord.
  final Graha lord;

  /// (e) Planets joined to a significator above.
  final List<Graha> conjoined;

  /// (f) Planets aspecting the house under the settings' node aspects.
  final List<Graha> aspected;

  /// Signs wholly inside the house.
  final List<Rashi> intercepted;

  @override
  List<Object?> get _fields => [
    house,
    inOccupantsStars,
    occupants,
    inLordsStar,
    lord,
    conjoined,
    aspected,
    intercepted,
  ];
}

/// What a node stands for, in Reader VI's order (C155).
final class KpNodeAgency extends _Value {
  const KpNodeAgency({
    required this.node,
    required this.conjoined,
    required this.starLord,
    required this.aspecting,
    required this.signLord,
  });

  final Graha node;
  final List<Graha> conjoined;
  final Graha starLord;
  final List<Graha> aspecting;
  final Graha signLord;

  @override
  List<Object?> get _fields => [node, conjoined, starLord, aspecting, signLord];
}

/// A chart's significators: the twelve houses, and the nodes' agency.
final class KpSignificators extends _Value {
  const KpSignificators({required this.houses, required this.nodes});

  final List<KpHouseSignificators> houses;
  final List<KpNodeAgency> nodes;

  @override
  List<Object?> get _fields => [houses, nodes];
}

/// Why a planet is a ruling planet: [kind] is `LAGNA_STAR`, `LAGNA_SIGN`,
/// `LAGNA_SUB`, `MOON_STAR`, `MOON_SIGN`, `MOON_SUB`, `DAY_LORD` or `AGENT`,
/// a node standing for the ruler [of], [by] being `IN_ITS_SIGN` or
/// `CONJOINED` (C152).
final class KpReason extends _Value {
  const KpReason({required this.kind, this.of, this.by});

  final String kind;
  final Graha? of;
  final String? by;

  @override
  List<Object?> get _fields => [kind, of, by];
}

/// A retrograde planet rejecting a ruler through its star, or its sub
/// (C153).
final class KpRejection extends _Value {
  const KpRejection({required this.retrograde, required this.byStar});

  final Graha retrograde;
  final bool byStar;

  @override
  List<Object?> get _fields => [retrograde, byStar];
}

/// One ruling planet, every reason it rules, and what rejects it.
final class KpRuler extends _Value {
  const KpRuler({
    required this.graha,
    required this.reasons,
    required this.retrograde,
    required this.rejectedBy,
    required this.rejectedBySub,
  });

  final Graha graha;

  /// Every reason it rules, the first the strongest.
  final List<KpReason> reasons;

  /// Itself retrograde, which the Reader reads as delay and not rejection.
  final bool retrograde;

  /// What rejects it under the settings; null when it stands.
  final KpRejection? rejectedBy;

  /// What would reject it under the other reading of C153.
  final KpRejection? rejectedBySub;

  @override
  List<Object?> get _fields => [
    graha,
    reasons,
    retrograde,
    rejectedBy,
    rejectedBySub,
  ];
}

/// The settings the ruling planets were read under: [count] (`FIVE` or
/// `WITH_SUBS`, C150), [nodeRulers] (`SIGN_OR_CONJOINED` or `SIGN`, C152)
/// and [retrogradeRejection] (`STAR` or `STAR_OR_SUB`, C153).
final class KpRulingRules extends _Value {
  const KpRulingRules({
    required this.count,
    required this.nodeRulers,
    required this.retrogradeRejection,
  });

  final String count;
  final String nodeRulers;
  final String retrogradeRejection;

  @override
  List<Object?> get _fields => [count, nodeRulers, retrogradeRejection];
}

/// The ruling planets of a moment, and the settings they were read under.
final class KpRuling extends _Value {
  const KpRuling({required this.rulers, required this.rules});

  final List<KpRuler> rulers;
  final KpRulingRules rules;

  /// The rulers that stand, in order.
  List<Graha> get accepted => [
    for (final ruler in rulers)
      if (ruler.rejectedBy == null) ruler.graha,
  ];

  @override
  List<Object?> get _fields => [rulers, rules];
}

/// A chart read as KP: the chart, its significators and the ruling planets
/// of its moment (`03-design/kp.md`). For a horary number the cusps are
/// the number's and the ruling planets still the moment's own.
final class KpReading extends _Value {
  const KpReading({
    required this.chart,
    required this.significators,
    required this.ruling,
  });

  final KpChart chart;
  final KpSignificators significators;
  final KpRuling ruling;

  @override
  List<Object?> get _fields => [chart, significators, ruling];
}

/// A set of rules the SDK ships, which a [MuhurtaRequest] may name.
enum MuhurtaActivity {
  /// A marriage by Raman's *Muhurtha*.
  ramanMarriage('RAMAN_MARRIAGE'),

  /// A marriage by the baseline engine's gates and weights, which the
  /// [MuhurtaRanking.baseline] ranking reads.
  baselineMarriage('BASELINE_MARRIAGE'),

  /// Naming the child, Nepal's nwaran, by Raman.
  ramanNamakarana('RAMAN_NAMAKARANA'),

  /// The first feeding on rice, Nepal's pasni, by Raman.
  ramanAnnaprasana('RAMAN_ANNAPRASANA'),

  /// The thread ceremony, Nepal's bratabandha, by Raman.
  ramanUpanayana('RAMAN_UPANAYANA'),

  /// Entering a new house, Nepal's griha pravesh, by Raman.
  ramanGrihaPravesha('RAMAN_GRIHA_PRAVESHA');

  const MuhurtaActivity(this.key);

  /// The name a request writes.
  final String key;
}

/// How a search orders its windows (C162).
enum MuhurtaRanking {
  /// By the texts' clauses: fewest uncancelled doshas, then most favourable.
  texts('TEXTS'),

  /// By the baseline engine's weights.
  baseline('BASELINE');

  const MuhurtaRanking(this.key);

  /// The name a request writes and an answer reads back.
  final String key;
}

/// A visibility criterion the SDK names, for how Venus's and Jupiter's
/// combustion is seen (C164).
enum AstaCriterion {
  /// The Surya Siddhanta's degrees of time; the default.
  suryaSiddhanta('SURYA_SIDDHANTA'),

  /// The tradition's combustion orbs.
  combustionOrb('COMBUSTION_ORB'),

  /// Ptolemy's arcus visionis.
  ptolemy('PTOLEMY');

  const AstaCriterion(this.key);

  /// The name a request writes.
  final String key;
}

/// Whose day a search reads: the birth star and Moon sign, and the birth
/// lagna when the time is known, whose eighth the search avoids.
final class MuhurtaNative {
  const MuhurtaNative({required this.star, required this.moonSign, this.lagna});

  final Nakshatra star;
  final Rashi moonSign;
  final Rashi? lagna;

  Map<String, Object?> get _json => {
    'star': star.fullKey,
    'moonSign': moonSign.fullKey,
    if (lagna != null) 'lagna': lagna!.fullKey,
  };
}

/// A muhurta search to run over an almanac's days
/// (`03-design/muhurta-at-the-boundary.md`), answered as [Almanac.muhurta].
///
/// `rules` is a [MuhurtaActivity], a set the SDK ships, or the rules spelt
/// out as the JSON record reads them, in which a catalogue member may stand
/// as itself and a clause an answer gave may stand as a bar; `asta` is an
/// [AstaCriterion] or a criterion spelt out. Anything else is refused by
/// name.
///
/// ```dart
/// final almanac = ctx.almanac.of(/* … */
///     muhurta: const MuhurtaRequest(rules: MuhurtaActivity.ramanMarriage));
/// final best = almanac.muhurta?.windows.first;
/// ```
final class MuhurtaRequest {
  const MuhurtaRequest({
    required this.rules,
    this.native,
    this.ranking = MuhurtaRanking.texts,
    this.daysWithWindows = 7,
    this.most = 50,
    this.asta,
  });

  /// A [MuhurtaActivity], or the rules spelt out.
  final Object rules;

  /// The native whose tarabala, chandrabala and ashtama lagna are read.
  final MuhurtaNative? native;

  /// How the windows are ordered.
  final MuhurtaRanking ranking;

  /// How many of the best days are cut into windows.
  final int daysWithWindows;

  /// How many windows are answered at most.
  final int most;

  /// An [AstaCriterion], or a criterion spelt out; the Surya Siddhanta's
  /// when absent.
  final Object? asta;

  String get _json => jsonEncode(<String, Object?>{
    'rules': _named(rules, 'rules', (MuhurtaActivity a) => a.key),
    if (native != null) 'native': native!._json,
    'ranking': ranking.key,
    'daysWithWindows': daysWithWindows,
    'most': most,
    if (asta != null) 'asta': _named(asta!, 'asta', (AstaCriterion a) => a.key),
  });

  /// A field that is a name or a record spelt out, written down; anything
  /// else refused by the field's name, as the SDK names its own refusals.
  static Object? _named<N>(
    Object value,
    String field,
    String Function(N) key,
  ) => switch (value) {
    N named => key(named),
    Map<String, Object?> spelt => _written(spelt),
    _ =>
      throw TeistroException(
        Status.invalidArg,
        'muhurta.$field is a name or a record spelt out, not ${value.runtimeType}',
        field: 'muhurta.$field',
      ),
  };
}

/// A request value as JSON writes it: a member as its full key, a clause as
/// its tag and fields, a record's values written in turn.
Object? _written(Object? value) => switch (value) {
  KeyOf<Object> member => member.fullKey,
  MuhurtaBar bar => bar._json,
  Map<String, Object?> record => {
    for (final MapEntry(:key, value: inner) in record.entries)
      key: _written(inner),
  },
  List<Object?> list => [for (final inner in list) _written(inner)],
  _ => value,
};

T _key<T>(Object? raw, T? Function(String) byKey, T unknown) =>
    byKey(raw! as String) ?? unknown;

List<T> _keys<T>(Object? raw, T? Function(String) byKey, T unknown) =>
    List<T>.unmodifiable([
      for (final key in raw! as List<Object?>) _key(key, byKey, unknown),
    ]);

/// A nakshatra's quarter.
final class MuhurtaPada extends _Value {
  const MuhurtaPada({required this.nakshatra, required this.pada});

  factory MuhurtaPada._read(Map<String, Object?> raw) => MuhurtaPada(
    nakshatra: _key(raw['nakshatra'], Nakshatra.byKey, Nakshatra.unknown),
    pada: raw['pada']! as int,
  );

  final Nakshatra nakshatra;

  /// 1 to 4.
  final int pada;

  Map<String, Object?> get _json => {
    'nakshatra': nakshatra.fullKey,
    'pada': pada,
  };

  @override
  List<Object?> get _fields => [nakshatra, pada];
}

/// The birth star's count to the day's, and the tara it gives: `JANMA`,
/// `SAMPAT`, `VIPAT`, `KSHEMA`, `PRATYAK`, `SADHANA`, `NAIDHANA`, `MITRA` or
/// `PARAMA_MITRA`.
final class TaraReading extends _Value {
  const TaraReading({
    required this.count,
    required this.tara,
    required this.cycle,
  });

  factory TaraReading._read(Map<String, Object?> raw) => TaraReading(
    count: raw['count']! as int,
    tara: raw['tara']! as String,
    cycle: raw['cycle']! as int,
  );

  final int count;
  final String tara;
  final int cycle;

  Map<String, Object?> get _json => {
    'count': count,
    'tara': tara,
    'cycle': cycle,
  };

  @override
  List<Object?> get _fields => [count, tara, cycle];
}

/// What bars a time outright: every clause of a kind ([BarAllOf]), or one
/// clause exactly (a [MuhurtaClauseKind]). A window's `barredBy` holds them,
/// and a request's rules take them back.
sealed class MuhurtaBar extends _Value {
  const MuhurtaBar();

  /// The tag of the clauses it bars.
  String get clause;

  Object get _json;
}

/// A bar on every clause of a kind, by its tag (`KAALA` bars all three
/// kaalas).
final class BarAllOf extends MuhurtaBar {
  const BarAllOf(this.clause);

  @override
  final String clause;

  @override
  Object get _json => clause;

  @override
  List<Object?> get _fields => [clause];
}

/// One named condition from a source, without when it held: a subclass a
/// kind, named for its tag, so a `switch` reads it; handed back as a bar it
/// bars exactly that clause. A `grade` is `BEST`, `MIDDLING` or `REJECTED`.
sealed class MuhurtaClauseKind extends MuhurtaBar {
  const MuhurtaClauseKind();

  @override
  Map<String, Object?> get _json;
}

/// The day's tithi, as the rules grade it.
final class TithiClause extends MuhurtaClauseKind {
  const TithiClause({required this.tithi, required this.grade});

  final Tithi tithi;
  final String grade;

  @override
  String get clause => 'TITHI';

  @override
  List<Object?> get _fields => [tithi, grade];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'tithi': tithi.fullKey,
    'grade': grade,
  };
}

/// The Moon's nakshatra, as the rules grade it.
final class NakshatraClause extends MuhurtaClauseKind {
  const NakshatraClause({required this.nakshatra, required this.grade});

  final Nakshatra nakshatra;
  final String grade;

  @override
  String get clause => 'NAKSHATRA';

  @override
  List<Object?> get _fields => [nakshatra, grade];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'nakshatra': nakshatra.fullKey,
    'grade': grade,
  };
}

/// The nitya yoga, as the rules grade it.
final class YogaClause extends MuhurtaClauseKind {
  const YogaClause({required this.yoga, required this.grade});

  final Yoga yoga;
  final String grade;

  @override
  String get clause => 'YOGA';

  @override
  List<Object?> get _fields => [yoga, grade];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'yoga': yoga.fullKey,
    'grade': grade,
  };
}

/// The karana, as the rules grade it.
final class KaranaClause extends MuhurtaClauseKind {
  const KaranaClause({required this.karana, required this.grade});

  final Karana karana;
  final String grade;

  @override
  String get clause => 'KARANA';

  @override
  List<Object?> get _fields => [karana, grade];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'karana': karana.fullKey,
    'grade': grade,
  };
}

/// The weekday, as the rules grade it.
final class VaraClause extends MuhurtaClauseKind {
  const VaraClause({required this.vara, required this.grade});

  final Vara vara;
  final String grade;

  @override
  String get clause => 'VARA';

  @override
  List<Object?> get _fields => [vara, grade];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'vara': vara.fullKey,
    'grade': grade,
  };
}

/// The lunar month, as the rules grade it (C161).
final class MonthClause extends MuhurtaClauseKind {
  const MonthClause({required this.masa, required this.grade});

  final Masa masa;
  final String grade;

  @override
  String get clause => 'MONTH';

  @override
  List<Object?> get _fields => [masa, grade];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'masa': masa.fullKey,
    'grade': grade,
  };
}

/// The Sun's sign, as the rules grade it (C161).
final class SolarMonthClause extends MuhurtaClauseKind {
  const SolarMonthClause({required this.sign, required this.grade});

  final Rashi sign;
  final String grade;

  @override
  String get clause => 'SOLAR_MONTH';

  @override
  List<Object?> get _fields => [sign, grade];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'sign': sign.fullKey,
    'grade': grade,
  };
}

/// The rising sign, as the rules grade it.
final class LagnaClause extends MuhurtaClauseKind {
  const LagnaClause({required this.sign, required this.grade});

  final Rashi sign;
  final String grade;

  @override
  String get clause => 'LAGNA';

  @override
  List<Object?> get _fields => [sign, grade];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'sign': sign.fullKey,
    'grade': grade,
  };
}

/// A quarter of the Moon's star the rules reject.
final class PadaClause extends MuhurtaClauseKind {
  const PadaClause({required this.pada});

  final MuhurtaPada pada;

  @override
  String get clause => 'PADA';

  @override
  List<Object?> get _fields => [pada];

  @override
  Map<String, Object?> get _json => {'clause': clause, 'pada': pada._json};
}

/// Rahu kaala, Yamaghanda or Gulika kaala.
final class KaalaClause extends MuhurtaClauseKind {
  const KaalaClause({required this.kaala});

  final Kaala kaala;

  @override
  String get clause => 'KAALA';

  @override
  List<Object?> get _fields => [kaala];

  @override
  Map<String, Object?> get _json => {'clause': clause, 'kaala': kaala.fullKey};
}

/// The choghadiya.
final class ChoghadiyaClause extends MuhurtaClauseKind {
  const ChoghadiyaClause({required this.choghadiya});

  final Choghadiya choghadiya;

  @override
  String get clause => 'CHOGHADIYA';

  @override
  List<Object?> get _fields => [choghadiya];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'choghadiya': choghadiya.fullKey,
  };
}

/// Abhijit muhurta.
final class AbhijitClause extends MuhurtaClauseKind {
  const AbhijitClause();

  @override
  String get clause => 'ABHIJIT';

  @override
  List<Object?> get _fields => [];

  @override
  Map<String, Object?> get _json => {'clause': clause};
}

/// A special yoga of vara, tithi and nakshatra (Raman ch. VI).
final class MuhurtaYogaClause extends MuhurtaClauseKind {
  const MuhurtaYogaClause({required this.yoga});

  final MuhurtaYoga yoga;

  @override
  String get clause => 'MUHURTA_YOGA';

  @override
  List<Object?> get _fields => [yoga];

  @override
  Map<String, Object?> get _json => {'clause': clause, 'yoga': yoga.fullKey};
}

/// The native's tarabala.
final class TarabalaClause extends MuhurtaClauseKind {
  const TarabalaClause({required this.reading});

  final TaraReading reading;

  @override
  String get clause => 'TARABALA';

  @override
  List<Object?> get _fields => [reading];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'reading': reading._json,
  };
}

/// The native's chandrabala: the Moon's house from the birth sign.
final class ChandrabalaClause extends MuhurtaClauseKind {
  const ChandrabalaClause({required this.house, required this.holds});

  final int house;
  final bool holds;

  @override
  String get clause => 'CHANDRABALA';

  @override
  List<Object?> get _fields => [house, holds];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'house': house,
    'holds': holds,
  };
}

/// Malefics either side of the lagna.
final class KartariClause extends MuhurtaClauseKind {
  const KartariClause({required this.second, required this.twelfth});

  final List<Graha> second;
  final List<Graha> twelfth;

  @override
  String get clause => 'KARTARI';

  @override
  List<Object?> get _fields => [second, twelfth];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'second': [for (final member in second) member.fullKey],
    'twelfth': [for (final member in twelfth) member.fullKey],
  };
}

/// The Moon in the 6th, 8th or 12th from the lagna.
final class MoonInDusthanaClause extends MuhurtaClauseKind {
  const MoonInDusthanaClause({required this.house});

  final int house;

  @override
  String get clause => 'MOON_IN_DUSTHANA';

  @override
  List<Object?> get _fields => [house];

  @override
  Map<String, Object?> get _json => {'clause': clause, 'house': house};
}

/// The Moon with another graha.
final class MoonJoinedClause extends MuhurtaClauseKind {
  const MoonJoinedClause({required this.joined});

  /// Which JSON writes as `with`.
  final List<Graha> joined;

  @override
  String get clause => 'MOON_JOINED';

  @override
  List<Object?> get _fields => [joined];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'with': [for (final member in joined) member.fullKey],
  };
}

/// Venus in the 6th (Bhrigu shatka).
final class VenusInSixthClause extends MuhurtaClauseKind {
  const VenusInSixthClause();

  @override
  String get clause => 'VENUS_IN_SIXTH';

  @override
  List<Object?> get _fields => [];

  @override
  Map<String, Object?> get _json => {'clause': clause};
}

/// Mars in the 8th (Kujashtama).
final class MarsInEighthClause extends MuhurtaClauseKind {
  const MarsInEighthClause();

  @override
  String get clause => 'MARS_IN_EIGHTH';

  @override
  List<Object?> get _fields => [];

  @override
  Map<String, Object?> get _json => {'clause': clause};
}

/// The lagna eighth from the native's birth lagna.
final class AshtamaLagnaClause extends MuhurtaClauseKind {
  const AshtamaLagnaClause();

  @override
  String get clause => 'ASHTAMA_LAGNA';

  @override
  List<Object?> get _fields => [];

  @override
  Map<String, Object?> get _json => {'clause': clause};
}

/// The lagna in a malefic's navamsa.
final class KunavamsaClause extends MuhurtaClauseKind {
  const KunavamsaClause({required this.navamsa, required this.lord});

  final Rashi navamsa;
  final Graha lord;

  @override
  String get clause => 'KUNAVAMSA';

  @override
  List<Object?> get _fields => [navamsa, lord];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'navamsa': navamsa.fullKey,
    'lord': lord.fullKey,
  };
}

/// The panchaka the remainder by nine names (C159).
final class PanchakaRemainderClause extends MuhurtaClauseKind {
  const PanchakaRemainderClause({required this.panchaka});

  final Panchaka panchaka;

  @override
  String get clause => 'PANCHAKA_REMAINDER';

  @override
  List<Object?> get _fields => [panchaka];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'panchaka': panchaka.fullKey,
  };
}

/// The lagna in its rasi visha ghatika.
final class LagnaTyajyaClause extends MuhurtaClauseKind {
  const LagnaTyajyaClause({required this.sign});

  final Rashi sign;

  @override
  String get clause => 'LAGNA_TYAJYA';

  @override
  List<Object?> get _fields => [sign];

  @override
  Map<String, Object?> get _json => {'clause': clause, 'sign': sign.fullKey};
}

/// A graha in the 7th.
final class SeventhOccupiedClause extends MuhurtaClauseKind {
  const SeventhOccupiedClause({required this.by});

  final List<Graha> by;

  @override
  String get clause => 'SEVENTH_OCCUPIED';

  @override
  List<Object?> get _fields => [by];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'by': [for (final member in by) member.fullKey],
  };
}

/// A malefic in the lagna.
final class MaleficInLagnaClause extends MuhurtaClauseKind {
  const MaleficInLagnaClause({required this.grahas});

  final List<Graha> grahas;

  @override
  String get clause => 'MALEFIC_IN_LAGNA';

  @override
  List<Object?> get _fields => [grahas];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'grahas': [for (final member in grahas) member.fullKey],
  };
}

/// Venus, Mercury or Jupiter in the lagna (neutralisation 6).
final class BeneficInLagnaClause extends MuhurtaClauseKind {
  const BeneficInLagnaClause({required this.grahas});

  final List<Graha> grahas;

  @override
  String get clause => 'BENEFIC_IN_LAGNA';

  @override
  List<Object?> get _fields => [grahas];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'grahas': [for (final member in grahas) member.fullKey],
  };
}

/// An exalted graha in the lagna (neutralisation 10).
final class ExaltedInLagnaClause extends MuhurtaClauseKind {
  const ExaltedInLagnaClause({required this.grahas});

  final List<Graha> grahas;

  @override
  String get clause => 'EXALTED_IN_LAGNA';

  @override
  List<Object?> get _fields => [grahas];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'grahas': [for (final member in grahas) member.fullKey],
  };
}

/// The Sun or the Moon in the 11th (neutralisation 8).
final class LuminaryInEleventhClause extends MuhurtaClauseKind {
  const LuminaryInEleventhClause({required this.grahas});

  final List<Graha> grahas;

  @override
  String get clause => 'LUMINARY_IN_ELEVENTH';

  @override
  List<Object?> get _fields => [grahas];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'grahas': [for (final member in grahas) member.fullKey],
  };
}

/// Jupiter or Venus in a kendra with the Sun, Mars and Saturn in the 3rd, 6th or 11th (neutralisation 11, C167).
final class KendraBeneficsClause extends MuhurtaClauseKind {
  const KendraBeneficsClause({required this.grahas});

  final List<Graha> grahas;

  @override
  String get clause => 'KENDRA_BENEFICS';

  @override
  List<Object?> get _fields => [grahas];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'grahas': [for (final member in grahas) member.fullKey],
  };
}

/// Grahas standing in a house the rite's rules want them out of, as their
/// `unwanted` list names them: one clause a house.
final class UnwantedPlacementClause extends MuhurtaClauseKind {
  const UnwantedPlacementClause({required this.house, required this.by});

  /// The house, 1 to 12 from the lagna.
  final int house;
  final List<Graha> by;

  @override
  String get clause => 'UNWANTED_PLACEMENT';

  @override
  List<Object?> get _fields => [house, by];

  @override
  Map<String, Object?> get _json => {
    'clause': clause,
    'house': house,
    'by': [for (final member in by) member.fullKey],
  };
}

/// A clause kind from its tagged JSON: a switch over every tag, so a tag
/// this build does not know is refused by name rather than guessed.
MuhurtaClauseKind _clauseKind(
  Map<String, Object?> raw,
) => switch (raw['clause']) {
  'TITHI' => TithiClause(
    tithi: _key(raw['tithi'], Tithi.byKey, Tithi.unknown),
    grade: raw['grade']! as String,
  ),
  'NAKSHATRA' => NakshatraClause(
    nakshatra: _key(raw['nakshatra'], Nakshatra.byKey, Nakshatra.unknown),
    grade: raw['grade']! as String,
  ),
  'YOGA' => YogaClause(
    yoga: _key(raw['yoga'], Yoga.byKey, Yoga.unknown),
    grade: raw['grade']! as String,
  ),
  'KARANA' => KaranaClause(
    karana: _key(raw['karana'], Karana.byKey, Karana.unknown),
    grade: raw['grade']! as String,
  ),
  'VARA' => VaraClause(
    vara: _key(raw['vara'], Vara.byKey, Vara.unknown),
    grade: raw['grade']! as String,
  ),
  'MONTH' => MonthClause(
    masa: _key(raw['masa'], Masa.byKey, Masa.unknown),
    grade: raw['grade']! as String,
  ),
  'SOLAR_MONTH' => SolarMonthClause(
    sign: _key(raw['sign'], Rashi.byKey, Rashi.unknown),
    grade: raw['grade']! as String,
  ),
  'LAGNA' => LagnaClause(
    sign: _key(raw['sign'], Rashi.byKey, Rashi.unknown),
    grade: raw['grade']! as String,
  ),
  'PADA' => PadaClause(
    pada: MuhurtaPada._read(raw['pada']! as Map<String, Object?>),
  ),
  'KAALA' => KaalaClause(kaala: _key(raw['kaala'], Kaala.byKey, Kaala.unknown)),
  'CHOGHADIYA' => ChoghadiyaClause(
    choghadiya: _key(raw['choghadiya'], Choghadiya.byKey, Choghadiya.unknown),
  ),
  'ABHIJIT' => AbhijitClause(),
  'MUHURTA_YOGA' => MuhurtaYogaClause(
    yoga: _key(raw['yoga'], MuhurtaYoga.byKey, MuhurtaYoga.unknown),
  ),
  'TARABALA' => TarabalaClause(
    reading: TaraReading._read(raw['reading']! as Map<String, Object?>),
  ),
  'CHANDRABALA' => ChandrabalaClause(
    house: raw['house']! as int,
    holds: raw['holds']! as bool,
  ),
  'KARTARI' => KartariClause(
    second: _keys(raw['second'], Graha.byKey, Graha.unknown),
    twelfth: _keys(raw['twelfth'], Graha.byKey, Graha.unknown),
  ),
  'MOON_IN_DUSTHANA' => MoonInDusthanaClause(house: raw['house']! as int),
  'MOON_JOINED' => MoonJoinedClause(
    joined: _keys(raw['with'], Graha.byKey, Graha.unknown),
  ),
  'VENUS_IN_SIXTH' => VenusInSixthClause(),
  'MARS_IN_EIGHTH' => MarsInEighthClause(),
  'ASHTAMA_LAGNA' => AshtamaLagnaClause(),
  'KUNAVAMSA' => KunavamsaClause(
    navamsa: _key(raw['navamsa'], Rashi.byKey, Rashi.unknown),
    lord: _key(raw['lord'], Graha.byKey, Graha.unknown),
  ),
  'PANCHAKA_REMAINDER' => PanchakaRemainderClause(
    panchaka: _key(raw['panchaka'], Panchaka.byKey, Panchaka.unknown),
  ),
  'LAGNA_TYAJYA' => LagnaTyajyaClause(
    sign: _key(raw['sign'], Rashi.byKey, Rashi.unknown),
  ),
  'SEVENTH_OCCUPIED' => SeventhOccupiedClause(
    by: _keys(raw['by'], Graha.byKey, Graha.unknown),
  ),
  'MALEFIC_IN_LAGNA' => MaleficInLagnaClause(
    grahas: _keys(raw['grahas'], Graha.byKey, Graha.unknown),
  ),
  'BENEFIC_IN_LAGNA' => BeneficInLagnaClause(
    grahas: _keys(raw['grahas'], Graha.byKey, Graha.unknown),
  ),
  'EXALTED_IN_LAGNA' => ExaltedInLagnaClause(
    grahas: _keys(raw['grahas'], Graha.byKey, Graha.unknown),
  ),
  'LUMINARY_IN_ELEVENTH' => LuminaryInEleventhClause(
    grahas: _keys(raw['grahas'], Graha.byKey, Graha.unknown),
  ),
  'KENDRA_BENEFICS' => KendraBeneficsClause(
    grahas: _keys(raw['grahas'], Graha.byKey, Graha.unknown),
  ),
  'UNWANTED_PLACEMENT' => UnwantedPlacementClause(
    house: raw['house']! as int,
    by: _keys(raw['by'], Graha.byKey, Graha.unknown),
  ),
  final tag =>
    throw StateError(
      'the library drew a clause this build does not know: $tag',
    ),
};

/// A clause and the interval it held over.
final class MuhurtaClause extends _Value {
  const MuhurtaClause({required this.kind, required this.at});

  final MuhurtaClauseKind kind;
  final Interval at;

  @override
  List<Object?> get _fields => [kind, at.from, at.to];
}

/// One of the baseline engine's weights: what it measured (a dimension such
/// as `TARA_BALA`), by how much, and the graha it read, if one.
final class MuhurtaFactor extends _Value {
  const MuhurtaFactor({
    required this.dimension,
    required this.weight,
    this.graha,
  });

  final String dimension;
  final int weight;
  final Graha? graha;

  @override
  List<Object?> get _fields => [dimension, weight, graha];
}

/// The baseline engine's score for a window, under its ranking.
final class MuhurtaScore extends _Value {
  const MuhurtaScore({
    required this.value,
    required this.factors,
    this.cappedAt,
  });

  final int value;
  final List<MuhurtaFactor> factors;

  /// The cap a Mahadosha put on it, if one did.
  final int? cappedAt;

  @override
  List<Object?> get _fields => [value, factors, cappedAt];
}

/// A window judged: when, by which clauses, what barred it, and the
/// baseline's score under that ranking.
final class MuhurtaWindow extends _Value {
  const MuhurtaWindow({
    required this.at,
    required this.clauses,
    required this.barredBy,
    this.score,
  });

  final Interval at;
  final List<MuhurtaClause> clauses;

  /// The bars that struck it; empty when the rite may be held in it.
  final List<MuhurtaBar> barredBy;

  /// `null` under the texts' ranking.
  final MuhurtaScore? score;

  @override
  List<Object?> get _fields => [at.from, at.to, clauses, barredBy, score];
}

/// A day the season closed, and the blackouts that closed it.
final class ClosedDay {
  const ClosedDay({required this.date, required this.by});

  final CalendarDate date;
  final List<BlackoutKind> by;
}

/// Something the rules ask that the SDK does not judge yet, and why.
final class MuhurtaUnjudged extends _Value {
  const MuhurtaUnjudged({required this.what, required this.why});

  final String what;
  final String why;

  @override
  List<Object?> get _fields => [what, why];
}

/// A muhurta search's answer (`03-design/muhurta-at-the-boundary.md` §4):
/// the windows judged, best first under the ranking, the days the season
/// closed, and what computed it.
final class MuhurtaAnswer {
  const MuhurtaAnswer({
    required this.windows,
    required this.closed,
    required this.daysJudged,
    required this.daysCut,
    required this.windowsBlackedOut,
    required this.ranking,
    required this.unjudged,
    required this.provenance,
  });

  final List<MuhurtaWindow> windows;
  final List<ClosedDay> closed;
  final int daysJudged;

  /// How many of the days judged were cut into windows.
  final int daysCut;

  /// How many windows fell in a blackout that did not cover their whole
  /// day, and were left out.
  final int windowsBlackedOut;
  final MuhurtaRanking ranking;
  final List<MuhurtaUnjudged> unjudged;

  /// What computed it and under what: the asta criterion and the zodiac's
  /// instant among the applied conventions, and the hash of the value.
  final Provenance provenance;
}

/// The `muhurta` section: the envelope's value, its members resolved, with
/// the provenance beside it.
MuhurtaAnswer _muhurtaAnswer(String json) {
  final envelope = jsonDecode(json) as Map<String, Object?>;
  final value = envelope['value']! as Map<String, Object?>;
  Map<String, Object?> at(Object? raw) => raw! as Map<String, Object?>;
  List<Map<String, Object?>> each(Object? raw) => [
    for (final item in raw! as List<Object?>) at(item),
  ];
  Interval interval(Object? raw) => Interval(
    from: (at(raw)['from']! as num).toDouble(),
    to: (at(raw)['to']! as num).toDouble(),
  );
  MuhurtaBar bar(Object? raw) =>
      raw is String ? BarAllOf(raw) : _clauseKind(at(raw));

  MuhurtaWindow window(Map<String, Object?> raw) {
    final score = raw['score'] as Map<String, Object?>?;
    return MuhurtaWindow(
      at: interval(raw['at']),
      clauses: List.unmodifiable([
        for (final clause in each(raw['clauses']))
          MuhurtaClause(kind: _clauseKind(clause), at: interval(clause['at'])),
      ]),
      barredBy: List.unmodifiable([
        for (final raw in raw['barredBy']! as List<Object?>) bar(raw),
      ]),
      score:
          score == null
              ? null
              : MuhurtaScore(
                value: score['value']! as int,
                factors: List.unmodifiable([
                  for (final factor in each(score['factors']))
                    MuhurtaFactor(
                      dimension: factor['dimension']! as String,
                      weight: factor['weight']! as int,
                      graha:
                          factor['graha'] == null
                              ? null
                              : _key(
                                factor['graha'],
                                Graha.byKey,
                                Graha.unknown,
                              ),
                    ),
                ]),
                cappedAt: score['cappedAt'] as int?,
              ),
    );
  }

  return MuhurtaAnswer(
    windows: List.unmodifiable([
      for (final raw in each(value['windows'])) window(raw),
    ]),
    closed: List.unmodifiable([
      for (final day in each(value['closed']))
        ClosedDay(
          date: _serdeDate(at(day['date'])),
          by: _keys(day['by'], BlackoutKind.byKey, BlackoutKind.unknown),
        ),
    ]),
    daysJudged: value['daysJudged']! as int,
    daysCut: value['daysCut']! as int,
    windowsBlackedOut: value['windowsBlackedOut']! as int,
    ranking: MuhurtaRanking.values.firstWhere(
      (ranking) => ranking.key == value['ranking'],
    ),
    unjudged: List.unmodifiable([
      for (final raw in each(value['unjudged']))
        MuhurtaUnjudged(
          what: raw['what']! as String,
          why: raw['why']! as String,
        ),
    ]),
    provenance: Provenance.fromJson(at(envelope['provenance'])),
  );
}

/// A date as the Rust types serialise it inside a JSON section (a muhurta
/// answer's closed day, a festival's day), in this binding's own shape: the
/// era beside its year, the resolution by name with a divergent one's
/// computed day.
CalendarDate _serdeDate(Map<String, Object?> raw) {
  final resolution = raw['resolution']! as Map<String, Object?>;
  final era = raw['era'] as Map<String, Object?>?;
  final computed =
      resolution['kind'] == 'DIVERGENT'
          ? resolution['computed']! as Map<String, Object?>
          : null;
  return CalendarDate(
    calendar: _key(raw['calendar'], Calendar.byKey, Calendar.unknown),
    era: era == null ? null : _key(era['era'], Era.byKey, Era.unknown),
    year: raw['year']! as int,
    eraYear: era == null ? 0 : era['year']! as int,
    month: raw['month']! as int,
    day: raw['day']! as int,
    resolution:
        Resolution.byKey(resolution['kind']! as String) ?? Resolution.defined,
    computedMonth: computed == null ? 0 : computed['month']! as int,
    computedDay: computed == null ? 0 : computed['day']! as int,
  );
}

/// A pack of festival rules the SDK ships, which a [FestivalRequest] may
/// name.
enum FestivalPack {
  /// *Dharmasindhu*'s rules (`03-design/festival-rules.md` §1).
  dharmasindhu('DHARMASINDHU'),

  /// *Dharmasindhu*'s rules as Nepal's national panchanga keeps them (a
  /// rite of the daylight on the day whose sunrise holds its tithi), and
  /// the days it counts from them (`03-design/festival-rules.md` §9.4–9.5).
  nepal('NEPAL');

  const FestivalPack(this.key);

  /// Its key, as the record names it.
  final String key;
}

/// Festival rules to reckon over an almanac's days
/// (`03-design/festival-rules.md` §7), answered as [Almanac.festivals].
///
/// `rules` is a [FestivalPack], or a list whose items are each a
/// [FestivalPack] or a rule spelt out as the JSON record reads it, in which
/// a catalogue member may stand as itself; a later rule replaces an earlier
/// one with its key. Anything else is refused by name.
///
/// ```dart
/// final almanac = ctx.almanac.of(/* … */
///     festivals: const FestivalRequest(rules: FestivalPack.dharmasindhu));
/// final first = almanac.festivals?.observances.first;
/// ```
final class FestivalRequest {
  const FestivalRequest({required this.rules});

  /// A [FestivalPack], or a list of packs and rules spelt out.
  final Object rules;

  String get _json => jsonEncode(<String, Object?>{
    'rules': switch (rules) {
      List<Object?> items => [
        for (final (index, item) in items.indexed)
          _item(item, 'festivals.rules[$index]'),
      ],
      final one => _item(one, 'festivals.rules'),
    },
  });

  static Object? _item(Object? item, String field) => switch (item) {
    FestivalPack pack => pack.key,
    Map<String, Object?> spelt when field != 'festivals.rules' => _written(
      spelt,
    ),
    _ =>
      throw TeistroException(
        Status.invalidArg,
        '$field is a pack, or a list of packs and rules spelt out, not ${item.runtimeType}',
        field: field,
      ),
  };
}

/// One of an observance's two days: its window for the rite, and the
/// fraction of it the tithi held (0 to 1; an instant's is 0 or 1).
final class FestivalExtent extends _Value {
  const FestivalExtent({
    required this.day,
    required this.window,
    required this.held,
  });

  final CalendarDate day;
  final Interval window;
  final double held;

  @override
  List<Object?> get _fields => [
    ..._dateFields(day),
    window.from,
    window.to,
    held,
  ];
}

/// What decided an observance's day: a guard, by its index in the rule's
/// list, or the rule's `otherwise`, whose [index] is `null`.
final class FestivalDecided extends _Value {
  const FestivalDecided({required this.by, this.index, this.rule, this.days});

  /// `GUARD`, `OTHERWISE`, or `AFTER` for a following rule.
  final String by;

  /// The guard's index; `null` unless a guard decided.
  final int? index;

  /// The rule a following rule counts from; `null` unless [by] is `AFTER`.
  final String? rule;

  /// The days a following rule counts; `null` unless [by] is `AFTER`.
  final int? days;

  @override
  List<Object?> get _fields => [by, index, rule, days];
}

/// The day a rule falls on, and why.
final class FestivalObservance extends _Value {
  const FestivalObservance({
    required this.rule,
    required this.day,
    required this.tithi,
    required this.month,
    required this.adhika,
    required this.case_,
    required this.extents,
    required this.decidedBy,
    required this.choice,
  });

  /// The rule's key.
  final String rule;
  final CalendarDate day;

  /// The occurrence judged: the tithi's, or the nakshatra's for a rule
  /// kept on a nakshatra in a paksha.
  final Interval tithi;

  /// Its amanta month, as an Ekadashi fast's is: which month's occurrence a
  /// rule kept every month decided, and whether it is the adhika one.
  final Masa month;
  final bool adhika;

  /// How the tithi held the rite's time on its two days: `EARLIER_ONLY`,
  /// `LATER_ONLY`, `BOTH`, `NEITHER`, `EQUAL_PARTS` or `UNEQUAL_PARTS`;
  /// `case_` because `case` is a keyword.
  final String case_;

  /// The earlier day's extent and the later's.
  final (FestivalExtent, FestivalExtent) extents;
  final FestivalDecided decidedBy;

  /// The choice that decided, `EARLIER`, `LATER` or `BY_YUGMA`, which [day]
  /// resolves.
  final String choice;

  @override
  List<Object?> get _fields => [
    rule,
    ..._dateFields(day),
    tithi.from,
    tithi.to,
    month,
    adhika,
    case_,
    extents.$1,
    extents.$2,
    decidedBy,
    choice,
  ];
}

/// An occurrence no day could be given to, and why.
final class FestivalUnjudged extends _Value {
  const FestivalUnjudged({
    required this.rule,
    required this.tithi,
    required this.why,
  });

  final String rule;
  final Interval tithi;
  final String why;

  @override
  List<Object?> get _fields => [rule, tithi.from, tithi.to, why];
}

/// An Ekadashi fast: the day a rule gives the 11th, by its vedha and the
/// excess the 11th and the 12th hold (`03-design/festival-rules.md` §8).
final class EkadashiFast extends _Value {
  const EkadashiFast({
    required this.rule,
    required this.tithi,
    required this.month,
    required this.adhika,
    required this.tithis,
    required this.days,
    required this.piercedAt,
    required this.pierced,
    required this.excess,
    required this.choice,
    required this.day,
  });

  /// The rule's key.
  final String rule;

  /// `Tithi.shuklaEkadashi` or `Tithi.krishnaEkadashi`.
  final Tithi tithi;

  /// The lunar month the 11th falls in, and whether it is the adhika one.
  final Masa month;
  final bool adhika;

  /// The 10th, the 11th and the 12th.
  final (Interval, Interval, Interval) tithis;

  /// The 11th's own day and the day after, the two the rule chooses from.
  final (CalendarDate, CalendarDate) days;

  /// Where the 10th reached into the 11th's day, `SUNRISE` or
  /// `ARUNODAYA`, whatever the rule's vedha; null when it did not.
  final String? piercedAt;

  /// Whether that piercing counts under the rule's vedha.
  final bool pierced;

  /// Which of the 11th and the 12th holds the next sunrise: `ELEVENTH`,
  /// `TWELFTH`, `BOTH` or `NEITHER`.
  final String excess;

  /// `EARLIER` or `LATER`, which [day] resolves.
  final String choice;
  final CalendarDate day;

  @override
  List<Object?> get _fields => [
    rule,
    tithi,
    month,
    adhika,
    for (final t in [tithis.$1, tithis.$2, tithis.$3]) ...[t.from, t.to],
    ..._dateFields(days.$1),
    ..._dateFields(days.$2),
    piercedAt,
    pierced,
    excess,
    choice,
    ..._dateFields(day),
  ];
}

/// What a set of festival rules gives over an almanac's days
/// (`03-design/festival-rules.md` §7.3).
final class FestivalAnswer {
  const FestivalAnswer({
    required this.observances,
    required this.ekadashis,
    required this.unjudged,
    required this.provenance,
  });

  final List<FestivalObservance> observances;

  /// The Ekadashi fasts, each under each Ekadashi rule asked for.
  final List<EkadashiFast> ekadashis;
  final List<FestivalUnjudged> unjudged;

  /// What computed it: the widened days among the applied conventions as
  /// `festival.days`, and the hash of the value.
  final Provenance provenance;
}

/// The `festivals` section: the envelope's value, its dates in this
/// binding's shape, with the provenance beside it.
FestivalAnswer _festivalAnswer(String json) {
  final envelope = jsonDecode(json) as Map<String, Object?>;
  final value = envelope['value']! as Map<String, Object?>;
  Map<String, Object?> at(Object? raw) => raw! as Map<String, Object?>;
  List<Map<String, Object?>> each(Object? raw) => [
    for (final item in raw! as List<Object?>) at(item),
  ];
  Interval interval(Object? raw) => Interval(
    from: (at(raw)['from']! as num).toDouble(),
    to: (at(raw)['to']! as num).toDouble(),
  );
  FestivalExtent extent(Map<String, Object?> raw) => FestivalExtent(
    day: _serdeDate(at(raw['day'])),
    window: interval(raw['window']),
    held: (raw['held']! as num).toDouble(),
  );

  return FestivalAnswer(
    observances: List.unmodifiable([
      for (final raw in each(value['observances']))
        FestivalObservance(
          rule: raw['rule']! as String,
          day: _serdeDate(at(raw['day'])),
          tithi: interval(raw['tithi']),
          month: _key(raw['month'], Masa.byKey, Masa.unknown),
          adhika: raw['adhika']! as bool,
          case_: raw['case']! as String,
          extents: switch (each(raw['extents'])) {
            [final earlier, final later] => (extent(earlier), extent(later)),
            final other =>
              throw StateError(
                'an observance has two extents, not ${other.length}',
              ),
          },
          decidedBy: FestivalDecided(
            by: at(raw['decidedBy'])['by']! as String,
            index: at(raw['decidedBy'])['index'] as int?,
            rule: at(raw['decidedBy'])['rule'] as String?,
            days: at(raw['decidedBy'])['days'] as int?,
          ),
          choice: raw['choice']! as String,
        ),
    ]),
    ekadashis: List.unmodifiable([
      for (final raw in each(value['ekadashis']))
        EkadashiFast(
          rule: raw['rule']! as String,
          tithi: _key(raw['tithi'], Tithi.byKey, Tithi.unknown),
          month: _key(raw['month'], Masa.byKey, Masa.unknown),
          adhika: raw['adhika']! as bool,
          tithis: switch (raw['tithis']! as List<Object?>) {
            [final tenth, final eleventh, final twelfth] => (
              interval(tenth),
              interval(eleventh),
              interval(twelfth),
            ),
            final other =>
              throw StateError('a fast has three tithis, not ${other.length}'),
          },
          days: switch (each(raw['days'])) {
            [final own, final after] => (_serdeDate(own), _serdeDate(after)),
            final other =>
              throw StateError('a fast has two days, not ${other.length}'),
          },
          piercedAt: raw['piercedAt'] as String?,
          pierced: raw['pierced']! as bool,
          excess: raw['excess']! as String,
          choice: raw['choice']! as String,
          day: _serdeDate(at(raw['day'])),
        ),
    ]),
    unjudged: List.unmodifiable([
      for (final raw in each(value['unjudged']))
        FestivalUnjudged(
          rule: raw['rule']! as String,
          tithi: interval(raw['tithi']),
          why: raw['why']! as String,
        ),
    ]),
    provenance: Provenance.fromJson(at(envelope['provenance'])),
  );
}

/// One Jovian year of the Surya Siddhanta's count (I.55).
final class JovianYear extends _Value {
  const JovianYear({
    required this.member,
    required this.count,
    required this.from,
    required this.to,
  });

  /// The year's name.
  final Samvatsara member;

  /// The signs mean Jupiter had crossed since the Kali age began, from 0.
  final int count;

  /// When mean Jupiter entered the sign, a UTC Julian day.
  final double from;

  /// When it entered the next, a UTC Julian day.
  final double to;

  @override
  List<Object?> get _fields => [member, count, from, to];
}

/// One lunar year: the name it carries, its numbers and bounds, and the
/// Jovian years that ran in it (`03-design/calendar-indian-lunisolar.md`
/// §10).
final class LunarYear extends _Value {
  const LunarYear({
    required this.samvatsara,
    required this.count,
    required this.vikrama,
    required this.shaka,
    required this.opened,
    required this.began,
    required this.ended,
    required this.jovian,
    required this.lupta,
  });

  /// The name the year carries under `calendars.samvatsara`.
  final Samvatsara samvatsara;

  /// Which count named it: `BARHASPATYA`, `BARHASPATYA_RUNNING` or
  /// `CHANDRAMANA`.
  final String count;

  /// The Vikrama year.
  final int vikrama;

  /// The Shaka year, whose number the southern count reads.
  final int shaka;

  /// The new moon that opened the year's first Chaitra, a UTC Julian day.
  final double opened;

  /// The sunrise of Chaitra Shukla Pratipada, where the name is read, a
  /// UTC Julian day.
  final double began;

  /// The next year's first sunrise, which ends this one, a UTC Julian day.
  final double ended;

  /// The Jovian years running between [began] and [ended], in order.
  final List<JovianYear> jovian;

  /// The Jovian year that began and ended inside this one and so names no
  /// year, or `null`.
  final Samvatsara? lupta;

  @override
  List<Object?> get _fields => [
    samvatsara,
    count,
    vikrama,
    shaka,
    opened,
    began,
    ended,
    jovian,
    lupta,
  ];
}

/// The lunar years an almanac's days fall in, in order and abutting.
///
/// ```dart
/// final almanac = sdk.almanac.of(
///     from: from, to: to, place: place, utcOffsetSeconds: 20700,
///     years: true);
/// final name = almanac.years?.value.first.samvatsara;
/// ```
final class LunarYears {
  const LunarYears({required this.value, required this.provenance});

  /// The years, each from one Chaitra Shukla Pratipada's sunrise to the
  /// next.
  final List<LunarYear> value;

  /// What computed them, and the hash of [value].
  final Provenance provenance;
}

/// A day's Nepal Sambat date, the committee's "ने.सं. ११४६ (कछलाथ्व)"
/// (`03-design/calendar-indian-lunisolar.md` §11).
///
/// `sdk.calendar.nepalSambatDate` says one.
final class NepalSambatDate {
  const NepalSambatDate({
    required this.year,
    required this.month,
    required this.kind,
    required this.paksha,
  });

  /// The year, which opens at Kachhala's first day: 1146 from 2025-10-22.
  final int year;

  /// The month, 1 for Kachhala (amanta Kartika) to 12 for Kaula (amanta
  /// Ashwina); an adhika month keeps the number of the month it repeats.
  final int month;

  /// Whether the month is ordinary, intercalary (Anala) or omitted.
  final MonthKind kind;

  /// The half: [Paksha.shukla] is thwa and [Paksha.krishna] ga.
  final Paksha paksha;

  @override
  bool operator ==(Object other) =>
      other is NepalSambatDate &&
      other.year == year &&
      other.month == month &&
      other.kind == kind &&
      other.paksha == paksha;

  @override
  int get hashCode => Object.hash(year, month, kind, paksha);
}

/// Each day of an almanac's Nepal Sambat date, in the days' order.
///
/// ```dart
/// final almanac = sdk.almanac.of(
///     from: from, to: to, place: place, utcOffsetSeconds: 20700,
///     nepalSambat: true);
/// final year = almanac.nepalSambat?.value.first.year;
/// ```
final class NepalSambatDates {
  const NepalSambatDates({required this.value, required this.provenance});

  /// One date a day, in the days' order.
  final List<NepalSambatDate> value;

  /// The days' own provenance, and the hash of [value].
  final Provenance provenance;
}

/// The `nepal_sambat` section: one date a day, with the provenance beside
/// them.
NepalSambatDates _nepalSambat(String json) {
  final envelope = jsonDecode(json) as Map<String, Object?>;
  NepalSambatDate date(Map<String, Object?> raw) {
    final kind = raw['kind']! as String;
    return NepalSambatDate(
      year: raw['year']! as int,
      month: raw['month']! as int,
      kind:
          MonthKind.byKey(kind) ??
          (throw ArgumentError.value(kind, 'kind', 'not a MonthKind')),
      paksha: _key(raw['paksha'], Paksha.byKey, Paksha.unknown),
    );
  }

  return NepalSambatDates(
    value: List.unmodifiable([
      for (final item in envelope['value']! as List<Object?>)
        date(item! as Map<String, Object?>),
    ]),
    provenance: Provenance.fromJson(
      envelope['provenance']! as Map<String, Object?>,
    ),
  );
}

/// The `years` section: the envelope's years as values, with the
/// provenance beside them.
LunarYears _lunarYears(String json) {
  final envelope = jsonDecode(json) as Map<String, Object?>;
  Map<String, Object?> at(Object? raw) => raw! as Map<String, Object?>;
  double jd(Object? raw) => (raw! as num).toDouble();
  Samvatsara named(Object? raw) =>
      _key(raw, Samvatsara.byKey, Samvatsara.unknown);
  JovianYear jovian(Map<String, Object?> raw) => JovianYear(
    member: named(raw['member']),
    count: raw['count']! as int,
    from: jd(raw['from']),
    to: jd(raw['to']),
  );
  LunarYear year(Map<String, Object?> raw) => LunarYear(
    samvatsara: named(raw['samvatsara']),
    count: raw['count']! as String,
    vikrama: raw['vikrama']! as int,
    shaka: raw['shaka']! as int,
    opened: jd(raw['opened']),
    began: jd(raw['began']),
    ended: jd(raw['ended']),
    jovian: List.unmodifiable([
      for (final item in raw['jovian']! as List<Object?>) jovian(at(item)),
    ]),
    lupta: raw['lupta'] == null ? null : named(raw['lupta']),
  );
  return LunarYears(
    value: List.unmodifiable([
      for (final item in envelope['value']! as List<Object?>) year(at(item)),
    ]),
    provenance: Provenance.fromJson(at(envelope['provenance'])),
  );
}

/// The rule that sized the Earth's shadow, `panchanga.eclipse_shadow`.
enum EclipseShadowRule {
  /// Danjon's, 1951: both radii grown by 1% of the Moon's horizontal
  /// parallax.
  danjon('DANJON'),

  /// Chauvenet's, 1891: both radii multiplied by 1.02.
  chauvenet('CHAUVENET');

  const EclipseShadowRule(this.key);

  /// The key the boundary spells it with.
  final String key;

  static EclipseShadowRule _byKey(Object? key) => values.firstWhere(
    (rule) => rule.key == key,
    orElse: () => throw StateError('a shadow rule $key'),
  );
}

/// One moment of an eclipse at the place: when, and the body's topocentric
/// geometric altitude there.
final class EclipseMoment extends _Value {
  const EclipseMoment({required this.at, required this.altitudeDeg});

  /// A UT1 Julian day.
  final double at;

  /// The eclipsed body's centre above the horizon, before refraction.
  final double altitudeDeg;

  @override
  List<Object?> get _fields => [at, altitudeDeg];
}

/// The stretch of an eclipse the place sees, the body above its horizon.
final class EclipseSeen extends _Value {
  const EclipseSeen({required this.from, required this.to});

  /// A UT1 Julian day.
  final double from;

  /// A UT1 Julian day.
  final double to;

  @override
  List<Object?> get _fields => [from, to];
}

/// A lunar eclipse's contacts with the penumbra and umbra, UT1 Julian
/// days; an umbral contact the eclipse never reaches is `null`.
final class LunarContacts extends _Value {
  const LunarContacts({
    required this.p1,
    required this.u1,
    required this.u2,
    required this.u3,
    required this.u4,
    required this.p4,
  });

  final double p1;
  final double? u1;
  final double? u2;
  final double? u3;
  final double? u4;
  final double p4;

  @override
  List<Object?> get _fields => [p1, u1, u2, u3, u4, p4];
}

/// A lunar eclipse: its kind, gamma, magnitudes and contacts under a rule
/// for the Earth's shadow.
final class LunarEclipse extends _Value {
  const LunarEclipse({
    required this.greatest,
    required this.kind,
    required this.gamma,
    required this.umbralMagnitude,
    required this.penumbralMagnitude,
    required this.contacts,
    required this.shadow,
  });

  /// The greatest eclipse, a UT1 Julian day.
  final double greatest;

  final LunarEclipseKind kind;

  /// The Moon's centre from the shadow's axis at the greatest eclipse, in
  /// Earth radii, positive when the Moon passes north of it.
  final double gamma;

  /// Negative for a penumbral eclipse, 1 or more for a total one.
  final double umbralMagnitude;

  final double penumbralMagnitude;
  final LunarContacts contacts;

  /// The rule that sized the shadow.
  final EclipseShadowRule shadow;

  @override
  List<Object?> get _fields => [
    greatest,
    kind,
    gamma,
    umbralMagnitude,
    penumbralMagnitude,
    contacts,
    shadow,
  ];
}

/// A solar eclipse: its kind at greatest, gamma, magnitude and where on the
/// Earth it is greatest.
final class SolarEclipse extends _Value {
  const SolarEclipse({
    required this.greatest,
    required this.kind,
    required this.gamma,
    required this.magnitude,
    required this.latitude,
    required this.longitude,
  });

  /// The greatest eclipse, a UT1 Julian day.
  final double greatest;

  final SolarEclipseKind kind;

  /// The shadow's axis from the Earth's centre at the greatest eclipse, in
  /// Earth radii, positive when it passes north.
  final double gamma;

  final double magnitude;

  /// Where the eclipse is greatest: geodetic latitude, degrees.
  final double latitude;

  /// Where the eclipse is greatest: longitude, degrees east.
  final double longitude;

  @override
  List<Object?> get _fields => [
    greatest,
    kind,
    gamma,
    magnitude,
    latitude,
    longitude,
  ];
}

/// A lunar eclipse at the place: each contact with the Moon's altitude,
/// and the stretch seen, or `null` when the Moon was down throughout.
final class LunarEclipseView extends _Value {
  const LunarEclipseView({
    required this.p1,
    required this.u1,
    required this.u2,
    required this.greatest,
    required this.u3,
    required this.u4,
    required this.p4,
    required this.seen,
    required this.umbralSeen,
  });

  final EclipseMoment p1;
  final EclipseMoment? u1;
  final EclipseMoment? u2;
  final EclipseMoment greatest;
  final EclipseMoment? u3;
  final EclipseMoment? u4;
  final EclipseMoment p4;
  final EclipseSeen? seen;

  /// The stretch of the umbral phase seen, the part the eye sees, or
  /// `null` (always for a penumbral eclipse).
  final EclipseSeen? umbralSeen;

  @override
  List<Object?> get _fields => [
    p1,
    u1,
    u2,
    greatest,
    u3,
    u4,
    p4,
    seen,
    umbralSeen,
  ];
}

/// A solar eclipse at the place: its own contacts, maximum and magnitude,
/// and the stretch seen, or `null` when the Sun was down throughout.
final class SolarEclipseView extends _Value {
  const SolarEclipseView({
    required this.kind,
    required this.magnitude,
    required this.obscuration,
    required this.first,
    required this.second,
    required this.third,
    required this.fourth,
    required this.maximum,
    required this.seen,
  });

  /// What the place sees at its maximum: never [SolarEclipseKind.hybrid].
  final SolarEclipseKind kind;

  final double magnitude;

  /// The fraction of the Sun's disc covered at the maximum.
  final double obscuration;

  final EclipseMoment first;
  final EclipseMoment? second;
  final EclipseMoment? third;
  final EclipseMoment fourth;
  final EclipseMoment maximum;
  final EclipseSeen? seen;

  @override
  List<Object?> get _fields => [
    kind,
    magnitude,
    obscuration,
    first,
    second,
    third,
    fourth,
    maximum,
    seen,
  ];
}

/// A lunar eclipse and how the place sees it.
final class LunarEclipseHere extends _Value {
  const LunarEclipseHere({required this.eclipse, required this.here});

  final LunarEclipse eclipse;
  final LunarEclipseView here;

  @override
  List<Object?> get _fields => [eclipse, here];
}

/// A solar eclipse and how the place sees it, `null` where the penumbra
/// never reaches.
final class SolarEclipseHere extends _Value {
  const SolarEclipseHere({required this.eclipse, required this.here});

  final SolarEclipse eclipse;
  final SolarEclipseView? here;

  @override
  List<Object?> get _fields => [eclipse, here];
}

/// The eclipses whose greatest moment falls in an almanac's days, each kind
/// in order.
final class EclipsesFound extends _Value {
  const EclipsesFound({required this.lunar, required this.solar});

  final List<LunarEclipseHere> lunar;
  final List<SolarEclipseHere> solar;

  @override
  List<Object?> get _fields => [lunar, solar];
}

/// The eclipses an almanac's days hold (`03-design/eclipses.md`).
///
/// ```dart
/// final almanac = sdk.almanac.of(
///     from: from, to: to, place: place, utcOffsetSeconds: 20700,
///     eclipses: true);
/// final seen = almanac.eclipses?.value.lunar.where((e) => e.here.seen != null);
/// ```
final class Eclipses {
  const Eclipses({required this.value, required this.provenance});

  /// The eclipses and how the place sees each.
  final EclipsesFound value;

  /// What computed them, the window searched among its conventions, and
  /// the hash of [value].
  final Provenance provenance;
}

/// The `eclipses` section: the envelope's eclipses as values, with the
/// provenance beside them.
Eclipses _eclipses(String json) {
  final envelope = jsonDecode(json) as Map<String, Object?>;
  Map<String, Object?> at(Object? raw) => raw! as Map<String, Object?>;
  double jd(Object? raw) => (raw! as num).toDouble();
  double? reachedJd(Object? raw) => raw == null ? null : jd(raw);
  EclipseMoment moment(Object? raw) {
    final m = at(raw);
    return EclipseMoment(at: jd(m['at']), altitudeDeg: jd(m['altitudeDeg']));
  }

  EclipseMoment? reached(Object? raw) => raw == null ? null : moment(raw);
  EclipseSeen? seen(Object? raw) {
    if (raw == null) return null;
    final s = at(raw);
    return EclipseSeen(from: jd(s['from']), to: jd(s['to']));
  }

  LunarEclipseHere lunar(Map<String, Object?> raw) {
    final e = at(raw['eclipse']);
    final c = at(e['contacts']);
    final h = at(raw['here']);
    return LunarEclipseHere(
      eclipse: LunarEclipse(
        greatest: jd(e['greatest']),
        kind: _key(e['kind'], LunarEclipseKind.byKey, LunarEclipseKind.unknown),
        gamma: jd(e['gamma']),
        umbralMagnitude: jd(e['umbralMagnitude']),
        penumbralMagnitude: jd(e['penumbralMagnitude']),
        contacts: LunarContacts(
          p1: jd(c['p1']),
          u1: reachedJd(c['u1']),
          u2: reachedJd(c['u2']),
          u3: reachedJd(c['u3']),
          u4: reachedJd(c['u4']),
          p4: jd(c['p4']),
        ),
        shadow: EclipseShadowRule._byKey(e['shadow']),
      ),
      here: LunarEclipseView(
        p1: moment(h['p1']),
        u1: reached(h['u1']),
        u2: reached(h['u2']),
        greatest: moment(h['greatest']),
        u3: reached(h['u3']),
        u4: reached(h['u4']),
        p4: moment(h['p4']),
        seen: seen(h['seen']),
        umbralSeen: seen(h['umbralSeen']),
      ),
    );
  }

  SolarEclipseHere solar(Map<String, Object?> raw) {
    final e = at(raw['eclipse']);
    final point = at(e['point']);
    final h = raw['here'] == null ? null : at(raw['here']);
    return SolarEclipseHere(
      eclipse: SolarEclipse(
        greatest: jd(e['greatest']),
        kind: _key(e['kind'], SolarEclipseKind.byKey, SolarEclipseKind.unknown),
        gamma: jd(e['gamma']),
        magnitude: jd(e['magnitude']),
        latitude: jd(point['latitude']),
        longitude: jd(point['longitude']),
      ),
      here:
          h == null
              ? null
              : SolarEclipseView(
                kind: _key(
                  h['kind'],
                  SolarEclipseKind.byKey,
                  SolarEclipseKind.unknown,
                ),
                magnitude: jd(h['magnitude']),
                obscuration: jd(h['obscuration']),
                first: moment(h['first']),
                second: reached(h['second']),
                third: reached(h['third']),
                fourth: moment(h['fourth']),
                maximum: moment(h['maximum']),
                seen: seen(h['seen']),
              ),
    );
  }

  final value = at(envelope['value']);
  return Eclipses(
    value: EclipsesFound(
      lunar: List.unmodifiable([
        for (final item in value['lunar']! as List<Object?>) lunar(at(item)),
      ]),
      solar: List.unmodifiable([
        for (final item in value['solar']! as List<Object?>) solar(at(item)),
      ]),
    ),
    provenance: Provenance.fromJson(at(envelope['provenance'])),
  );
}

/// A date's fields, for a value that holds one to compare by: the
/// generated [CalendarDate] is a plain record without equality of its own.
List<Object?> _dateFields(CalendarDate date) => [
  date.calendar,
  date.era,
  date.year,
  date.eraYear,
  date.month,
  date.day,
  date.resolution,
  date.computedMonth,
  date.computedDay,
];

/// Each batch's KP readings, parsed once however many charts read them.
final Expando<List<KpReading>> _kps = Expando<List<KpReading>>('kp');

List<KpReading> _kpsOf(Charts batch) =>
    _kps[batch] ??= [for (final raw in _sectionOf(batch.kp)) _kpReading(raw)];

/// A chart's KP reading from the `kp` section's JSON, its keys made members.
KpReading _kpReading(Map<String, Object?> raw) {
  Map<String, Object?> at(Object? value) => value! as Map<String, Object?>;
  List<Object?> each(Object? value) => value! as List<Object?>;
  Graha graha(Object? key) => Graha.byKey(key! as String) ?? Graha.unknown;
  List<Graha> grahas(Object? keys) => [
    for (final key in each(keys)) graha(key),
  ];
  KpLevel level(Object? value) {
    final level = at(value);
    final span = at(level['span']);
    return KpLevel(
      lord: graha(level['lord']),
      span: KpSpan(start: span['start']! as int, end: span['end']! as int),
    );
  }

  KpLords lords(Object? value) {
    final lords = at(value);
    return KpLords(
      sign: graha(lords['sign']),
      star: level(lords['star']),
      sub: level(lords['sub']),
      subSub: level(lords['subSub']),
    );
  }

  KpRejection? rejection(Object? value) =>
      value == null
          ? null
          : KpRejection(
            retrograde: graha(at(value)['retrograde']),
            byStar: at(value)['byStar']! as bool,
          );

  final chart = at(raw['chart']);
  final significators = at(raw['significators']);
  final ruling = at(raw['ruling']);
  final rules = at(ruling['rules']);
  return KpReading(
    chart: KpChart(
      system:
          HouseSystem.byKey(chart['system']! as String) ?? HouseSystem.unknown,
      cusps: List<KpCusp>.unmodifiable([
        for (final cusp in each(chart['cusps']).map(at))
          KpCusp(
            house: cusp['house']! as int,
            longitude: cusp['longitude']! as int,
            lords: lords(cusp['lords']),
          ),
      ]),
      planets: List<KpPlanet>.unmodifiable([
        for (final planet in each(chart['planets']).map(at))
          KpPlanet(
            graha: graha(planet['graha']),
            longitude: planet['longitude']! as int,
            retrograde: planet['retrograde']! as bool,
            house: planet['house']! as int,
            lords: lords(planet['lords']),
          ),
      ]),
    ),
    significators: KpSignificators(
      houses: List<KpHouseSignificators>.unmodifiable([
        for (final house in each(significators['houses']).map(at))
          KpHouseSignificators(
            house: house['house']! as int,
            inOccupantsStars: grahas(house['inOccupantsStars']),
            occupants: grahas(house['occupants']),
            inLordsStar: grahas(house['inLordsStar']),
            lord: graha(house['lord']),
            conjoined: grahas(house['conjoined']),
            aspected: grahas(house['aspected']),
            intercepted: [
              for (final key in each(house['intercepted']))
                Rashi.byKey(key! as String) ?? Rashi.unknown,
            ],
          ),
      ]),
      nodes: List<KpNodeAgency>.unmodifiable([
        for (final node in each(significators['nodes']).map(at))
          KpNodeAgency(
            node: graha(node['node']),
            conjoined: grahas(node['conjoined']),
            starLord: graha(node['starLord']),
            aspecting: grahas(node['aspecting']),
            signLord: graha(node['signLord']),
          ),
      ]),
    ),
    ruling: KpRuling(
      rulers: List<KpRuler>.unmodifiable([
        for (final ruler in each(ruling['rulers']).map(at))
          KpRuler(
            graha: graha(ruler['graha']),
            reasons: List<KpReason>.unmodifiable([
              for (final why in each(ruler['reasons']).map(at))
                KpReason(
                  kind: why['kind']! as String,
                  of: why['of'] == null ? null : graha(why['of']),
                  by: why['by'] as String?,
                ),
            ]),
            retrograde: ruler['retrograde']! as bool,
            rejectedBy: rejection(ruler['rejectedBy']),
            rejectedBySub: rejection(ruler['rejectedBySub']),
          ),
      ]),
      rules: KpRulingRules(
        count: rules['count']! as String,
        nodeRulers: rules['nodeRulers']! as String,
        retrogradeRejection: rules['retrogradeRejection']! as String,
      ),
    ),
  );
}

/// The transit hit list to search against every chart of a request
/// (`03-design/transit-hit-list.md`): the window, and optionally the grahas,
/// the kinds of event, the natal points aspected, the aspects' angles and an
/// orb. Anything left out is the default: every graha, every kind, the nine
/// grahas and the lagna, the conjunction and opposition (C145), exact only
/// (C146). The sky is searched once for the whole batch.
///
/// ```dart
/// final chart = ctx.chart.found(
///   /* … */ hits: const HitRequest(from: 2460676.5, to: 2461041.5, grahas: [Graha.saturn]),
/// );
/// final ingresses = chart.hits.where((h) => h.event is SignIngress);
/// ```
final class HitRequest {
  const HitRequest({
    required this.from,
    required this.to,
    this.grahas,
    this.kinds,
    this.points,
    this.aspects,
    this.orbDeg,
  });

  /// The hit list that is each graha's returns: its conjunction, at 0°,
  /// with its own natal place (`03-design/western-returns.md`), the Moon's
  /// by default (the lunar return); the Sun's is the solar. Asked for
  /// several grahas, the list also holds each one's crossing of another's
  /// natal place, its [AspectHit.to] naming the place.
  ///
  /// ```dart
  /// final lunar = ctx.chart.found(/* … */ hits: HitRequest.returns(from: 2451546, to: 2451911.25)).hits;
  /// // The figure, erected where the native is (C258).
  /// final figure = ctx.chart.found(instant: lunar.first.instant, /* … */);
  /// ```
  factory HitRequest.returns({
    required double from,
    required double to,
    List<Graha> grahas = const [Graha.moon],
  }) => HitRequest(
    from: from,
    to: to,
    grahas: grahas,
    kinds: const [HitKind.aspect],
    points: [for (final g in grahas) NatalGraha(g)],
    aspects: const [0],
  );

  /// The window's start, a UTC Julian day.
  final double from;

  /// The window's end, after the start, or the SDK refuses it by `hits.to`.
  final double to;

  /// The grahas to search; the nine by default.
  final List<Graha>? grahas;

  /// The kinds of event to report; all four by default.
  final List<HitKind>? kinds;

  /// The natal points aspected; the nine grahas and the lagna by default.
  final List<NatalPoint>? points;

  /// The aspects' angles, whole degrees from 0 to 180; 0 and 180 by default.
  final List<int>? aspects;

  /// An orb in degrees, more than 0, under 15 and under half the step
  /// between the aspects' lines, for each window's opening and closing;
  /// exact only by default.
  final double? orbDeg;

  String get _json => jsonEncode(<String, Object?>{
    'from': from,
    'to': to,
    if (grahas case final grahas?) 'grahas': [for (final g in grahas) g.key],
    if (kinds case final kinds?) 'kinds': [for (final k in kinds) k.key],
    if (points case final points?) 'points': [for (final p in points) p._json],
    if (aspects case final aspects?) 'aspects': aspects,
    if (orbDeg case final orbDeg?) 'orbDeg': orbDeg,
  });
}

/// A natal point a transit aspects: a natal graha ([NatalGraha]) or the
/// lagna ([NatalLagna]). It goes into [HitRequest.points] as it comes back
/// in an aspect's [AspectHit.to].
sealed class NatalPoint {
  const NatalPoint();

  /// `'GRAHA'` or `'LAGNA'`, as every binding spells it.
  String get point;

  Map<String, Object?> get _json;
}

/// A natal graha, as a point a transit aspects.
final class NatalGraha extends NatalPoint {
  const NatalGraha(this.graha);

  /// Which.
  final Graha graha;

  @override
  String get point => 'GRAHA';

  @override
  Map<String, Object?> get _json => {'point': point, 'graha': graha.key};

  @override
  bool operator ==(Object other) => other is NatalGraha && other.graha == graha;

  @override
  int get hashCode => graha.hashCode;
}

/// The natal lagna, as a point a transit aspects.
final class NatalLagna extends NatalPoint {
  const NatalLagna();

  @override
  String get point => 'LAGNA';

  @override
  Map<String, Object?> get _json => {'point': point};

  @override
  bool operator ==(Object other) => other is NatalLagna;

  @override
  int get hashCode => point.hashCode;
}

/// What a hit was: [SignIngress], [NakshatraIngress], [Station] or
/// [AspectHit], each carrying only its own fields.
sealed class HitEvent {
  const HitEvent();

  /// Which kind, as the event's tag.
  HitKind get kind;
}

/// The graha entered a sign; a retrograde ingress enters the one before
/// the line it crossed.
final class SignIngress extends HitEvent {
  const SignIngress({required this.into, required this.motion});

  /// The sign entered.
  final Rashi into;

  /// Which way it was moving.
  final Motion motion;

  @override
  HitKind get kind => HitKind.signIngress;
}

/// The graha entered a nakshatra.
final class NakshatraIngress extends HitEvent {
  const NakshatraIngress({required this.into, required this.motion});

  /// The nakshatra entered.
  final Nakshatra into;

  /// Which way it was moving.
  final Motion motion;

  @override
  HitKind get kind => HitKind.nakshatraIngress;
}

/// The graha stood still in longitude.
final class Station extends HitEvent {
  const Station({required this.turns});

  /// The motion it turned to.
  final Motion turns;

  @override
  HitKind get kind => HitKind.station;
}

/// The graha aspected a natal point, or came within or left its orb.
final class AspectHit extends HitEvent {
  const AspectHit({
    required this.to,
    required this.angle,
    required this.phase,
    required this.motion,
  });

  /// The natal point aspected.
  final NatalPoint to;

  /// The angle, 0 to 180 degrees, either side of the natal point (C145).
  final int angle;

  /// Where in the orb's window (C146); always exact without an orb.
  final AspectPhase phase;

  /// Which way the transit was moving.
  final Motion motion;

  @override
  HitKind get kind => HitKind.aspect;
}

/// One event of the transit hit list.
final class Hit {
  const Hit({required this.instant, required this.graha, required this.event});

  /// When, a UTC Julian day.
  final double instant;

  /// The transiting graha.
  final Graha graha;

  /// What happened.
  final HitEvent event;
}

/// The transits to read against every chart of a request
/// (`03-design/gochar.md`): at least one instant, and what to count the
/// houses from — the natal Moon by default (Phaladeepika ch. 26 v. 1).
///
/// ```dart
/// final chart = ctx.chart.found(
///   /* … */ gochar: const GocharRequest(instants: [2460676.5]),
/// );
/// final good = chart.gochar.first.grahas
///     .where((g) => g.verdict == GocharVerdict.good)
///     .map((g) => g.graha);
/// ```
final class GocharRequest {
  const GocharRequest({
    required this.instants,
    this.from = GocharFrom.moon,
    this.ashtakavarga = false,
  });

  /// The instants, UTC Julian days: at least one, or the SDK refuses the
  /// request by `gochar.instants`.
  final List<double> instants;

  /// What to count the houses from.
  final GocharFrom from;

  /// Whether to judge the seven by the natal Ashtakavarga too
  /// (`03-design/gochar-ashtakavarga.md`).
  final bool ashtakavarga;

  String get _json => jsonEncode(<String, Object?>{
    'instants': instants,
    'from': from.key,
    'ashtakavarga': ashtakavarga,
  });
}

/// The annual charts a request asks for: how many years, and which
/// longitude the Sun returns to.
///
/// ```dart
/// final chart = ctx.chart.found(
///   /* … */ varsha: const VarshaRequest(through: 40),
/// );
/// final thirtieth = chart.praveshas.firstWhere((one) => one.year == 30);
/// ```
final class VarshaRequest {
  const VarshaRequest({
    required this.through,
    this.reading = VarshaReading.sidereal,
    this.muntha = MunthaDegree.signStart,
    this.place,
    this.varshesha = const VarsheshaRules(),
    this.matters,
    this.yogas = const YogaRules(),
    this.sahams,
    this.sahamRules = const SahamRules(),
    this.sahamStrength = const SahamStrengthReadings(),
    this.harshaRules = const HarshaRules(),
    this.dashas,
    this.dashaRules = const AnnualDashaRules(),
  });

  /// The last year of life wanted, 1 to 200.
  final int through;

  /// Which longitude the Sun returns to.
  final VarshaReading reading;

  /// Where the Muntha stands inside the sign it has reached.
  final MunthaDegree muntha;

  /// Where each year's own chart is cast, when you want the charts and not
  /// only their instants. **Null, none is founded** — the SDK does not
  /// choose between the birthplace and a residence for you, because the
  /// schools differ.
  final AnnualPlace? place;

  /// The readings the year lord's chain parts on, where authorities
  /// differ; the source's own by default.
  final VarsheshaRules varshesha;

  /// The matters each year's sixteen Tajika yogas are judged for. Fourteen
  /// of the sixteen are judgements about the lagnesha and the lord of the
  /// house asked about, so they answer a matter and not a chart. **Needs
  /// [place]**; null, none is judged.
  ///
  /// ```dart
  /// const VarshaRequest(
  ///   through: 40, place: AnnualPlace.birth, matters: Matters.houses([7, 10]));
  /// ```
  final Matters? matters;

  /// The readings the sixteen part on, where the source leaves a choice.
  final YogaRules yogas;

  /// The sahams each year's chart is read for. **Needs [place]**; null,
  /// none is read.
  ///
  /// ```dart
  /// const VarshaRequest(
  ///   through: 40,
  ///   place: AnnualPlace.birth,
  ///   sahams: Sahams.these([Saham.punya, Saham.vivaha]),
  /// );
  /// ```
  final Sahams? sahams;

  /// The readings the sahams part on, where the sources differ.
  final SahamRules sahamRules;

  /// The readings a saham's strength parts on.
  final SahamStrengthReadings sahamStrength;

  /// The Harsha bala's reading of Venus's house of joy.
  final HarshaRules harshaRules;

  /// The annual dashas each year is divided by. The Sun is read over a year
  /// once however many are asked for. **Needs [place]**; null, none is
  /// read.
  ///
  /// ```dart
  /// const VarshaRequest(
  ///   through: 40,
  ///   place: AnnualPlace.birth,
  ///   dashas: AnnualDashas.these([DashaSystem.mudda]),
  /// );
  /// ```
  final AnnualDashas? dashas;

  /// The readings the annual dashas part on, where the sources differ.
  final AnnualDashaRules dashaRules;

  String get _json => jsonEncode(<String, Object?>{
    'reading': reading.key,
    'through': through,
    'muntha': muntha.key,
    if (place case final place?) 'place': place._json,
    'varshesha': varshesha._json,
    if (matters case final matters?) 'matters': matters._json,
    'yogas': yogas._json,
    if (sahams case final sahams?) 'sahams': sahams._json,
    'sahamRules': sahamRules._json,
    'sahamStrength': sahamStrength._json,
    'harshaRules': harshaRules._json,
    if (dashas case final dashas?) 'dashas': dashas._json,
    'dashaRules': dashaRules._json,
  });
}

/// The annual dashas a request asks for: the three, or these in the order
/// you want them answered (`03-design/annual-dashas.md`).
sealed class AnnualDashas {
  const AnnualDashas._();

  /// The Patyayini, the Mudda and the Varsha Yogini, the catalogue's order.
  static const AnnualDashas all = _AllAnnualDashas();

  /// These, in this order: [DashaSystem.patyayini], [DashaSystem.mudda] or
  /// [DashaSystem.varshaYogini]. Another system, or one named twice, is
  /// refused.
  const factory AnnualDashas.these(List<DashaSystem> systems) =
      _TheseAnnualDashas;

  Object get _json;
}

final class _AllAnnualDashas extends AnnualDashas {
  const _AllAnnualDashas() : super._();

  @override
  Object get _json => 'all';
}

final class _TheseAnnualDashas extends AnnualDashas {
  const _TheseAnnualDashas(this.systems) : super._();

  final List<DashaSystem> systems;

  // A system crosses as its full key, the spelling it is read back in.
  @override
  Object get _json => [for (final one in systems) one.fullKey];
}

/// What a unit of the year is (crux C122).
sealed class YearClock {
  const YearClock._();

  /// The Sun's motion through one degree from where it stood at the return:
  /// the source's own, so the year closes on the next return.
  static const YearClock sunDegrees = _NamedClock('SUN_DEGREES');

  /// An equal share of the time from this return to the next.
  static const YearClock even = _NamedClock('EVEN');

  /// The whole year as this many civil days from the return: the printed
  /// durations, 360 for the Mudda and the Yogini and 365 for the Patyayini.
  const factory YearClock.days(double days) = _DaysClock;

  Object get _json;
}

final class _NamedClock extends YearClock {
  const _NamedClock(this.key) : super._();

  final String key;

  @override
  Object get _json => key;
}

final class _DaysClock extends YearClock {
  const _DaysClock(this.days) : super._();

  final double days;

  @override
  Object get _json => {'DAYS': days};
}

/// Where the balance a nakshatra year opens with comes from (crux C123).
enum MuddaBalance {
  /// What remained of the birth Moon's nakshatra: the source's own.
  natalMoon('NATAL_MOON'),

  /// How far the Moon at the return is through its own nakshatra.
  entryMoon('ENTRY_MOON'),

  /// None: the first lord runs its whole share from the return.
  whole('WHOLE');

  const MuddaBalance(this.key);

  /// The key the boundary reads.
  final String key;
}

/// How a dasha's birth period — an annual dasha's first lord's two pieces —
/// is divided among its sub-periods.
enum BirthPeriod {
  /// Each sub-period its share of the balance: the default.
  compressed('COMPRESSED'),

  /// Each its share of the whole period, those already over dropped.
  elapsed('ELAPSED');

  const BirthPeriod(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Where the sources differ on an annual dasha
/// (`03-design/annual-dashas.md`). A reading left null is the SDK's own
/// default.
final class AnnualDashaRules {
  const AnnualDashaRules({
    this.clock,
    this.balance,
    this.measure,
    this.birthPeriod,
    this.depth,
  });

  /// What a unit of the year is; [YearClock.sunDegrees] by default.
  final YearClock? clock;

  /// Where a nakshatra year's balance comes from; [MuddaBalance.natalMoon]
  /// by default.
  final MuddaBalance? balance;

  /// How the balance is measured; null, each balance's source's own: by arc
  /// for the birth Moon, by time for the Moon at the return.
  final Balance? measure;

  /// How the first lord's two pieces are divided among sub-lords.
  final BirthPeriod? birthPeriod;

  /// How many levels the periods go down: 2, mahadashas and antardashas,
  /// by default.
  final int? depth;

  Map<String, Object?> get _json => <String, Object?>{
    if (clock case final clock?) 'clock': clock._json,
    if (balance case final balance?) 'balance': balance.key,
    // The measure is a settings knob, which the boundary reads by its own
    // key; an exhaustive switch, so a new member stops this compiling.
    if (measure case final measure?)
      'measure': switch (measure) {
        Balance.spatial => 'SPATIAL',
        Balance.temporal => 'TEMPORAL',
      },
    if (birthPeriod case final birthPeriod?) 'birthPeriod': birthPeriod.key,
    if (depth case final depth?) 'depth': depth,
  };
}

/// Which planets a saham's strength calls benefic and malefic.
enum SahamNatures {
  /// The chapter's own: the Sun a malefic among them.
  chapter('CHAPTER'),

  /// The catalogue's Parashari natures.
  parashari('PARASHARI');

  const SahamNatures(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Whose friendship a saham's "friend" and "inimical" clauses read.
enum SahamFriendship {
  /// Tajika's positional friendship, the only one the source defines.
  positional('POSITIONAL'),

  /// The catalogue's natural friendships.
  natural('NATURAL');

  const SahamFriendship(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Where the sources differ on a saham's strength
/// (`03-design/tajika-saham-strength.md`). A reading left null is the
/// SDK's own default.
final class SahamStrengthReadings {
  const SahamStrengthReadings({this.natures, this.friendship, this.weakBelow});

  /// Which planets are benefic and malefic.
  final SahamNatures? natures;

  /// Whose friendship is read.
  final SahamFriendship? friendship;

  /// The Vishwa bala below which a saham's lord is weak, in **sub-sub
  /// units**: `5 * 3600` by default.
  final int? weakBelow;

  Map<String, Object?> get _json => <String, Object?>{
    if (natures case final natures?) 'natures': natures.key,
    if (friendship case final friendship?) 'friendship': friendship.key,
    if (weakBelow case final weakBelow?) 'weakBelow': weakBelow,
  };
}

/// Venus's house of joy, which the Harsha bala's first part reads.
enum VenusPlace {
  /// The fifth: the verse's, and the default.
  fifth('FIFTH'),

  /// The twelfth, as a widely used program reads it.
  twelfth('TWELFTH');

  const VenusPlace(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Where the sources differ on the Harsha bala
/// (`03-design/tajika-harsha.md`).
final class HarshaRules {
  const HarshaRules({this.venus});

  /// Venus's house of joy.
  final VenusPlace? venus;

  Map<String, Object?> get _json => <String, Object?>{
    if (venus case final venus?) 'venus': venus.key,
  };
}

/// The sahams a request asks for: all forty-one, or these in the order you
/// want them answered.
sealed class Sahams {
  const Sahams._();

  /// The forty-one, in the source's order.
  static const Sahams all = _AllSahams();

  /// These, in this order; a saham named twice is refused.
  const factory Sahams.these(List<Saham> sahams) = _TheseSahams;

  Object get _json;
}

final class _AllSahams extends Sahams {
  const _AllSahams() : super._();

  @override
  Object get _json => 'all';
}

final class _TheseSahams extends Sahams {
  const _TheseSahams(this.sahams) : super._();

  final List<Saham> sahams;

  // A saham crosses as its key, the spelling it is read back in.
  @override
  Object get _json => [for (final one in sahams) one.key];
}

/// When a saham is carried a sign further (`03-design/tajika-sahams.md`).
enum AddSign {
  /// When c does not fall between b and a by degrees: the source's own.
  degrees('DEGREES'),

  /// By whole signs, as a widely used program reads it.
  signs('SIGNS'),

  /// Never: a − b + c alone.
  never('NEVER');

  const AddSign(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Where a house's point stands, for the sahams that read one.
enum HousePoints {
  /// Sripati's mid-point, built from the angles: the source's own.
  sripati('SRIPATI'),

  /// The chart's own chalit middles, under whatever its profile names.
  chalit('CHALIT'),

  /// Equal houses from the lagna's degree.
  equal('EQUAL');

  const HousePoints(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Roga's formula: the source gives two.
enum RogaReading {
  /// Lagna − Moon + lagna: the saham as given.
  lagna('LAGNA'),

  /// Saturn − Moon + lagna: the other authority's.
  saturn('SATURN');

  const RogaReading(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Where the sources differ on a saham. A reading left null is the SDK's
/// own default, which lives in one place and is not repeated here.
final class SahamRules {
  const SahamRules({this.addSign, this.houses, this.roga});

  /// When a saham is carried a sign further.
  final AddSign? addSign;

  /// Where a house's point stands.
  final HousePoints? houses;

  /// Roga's formula.
  final RogaReading? roga;

  Map<String, Object?> get _json => <String, Object?>{
    if (addSign case final addSign?) 'addSign': addSign.key,
    if (houses case final houses?) 'houses': houses.key,
    if (roga case final roga?) 'roga': roga.key,
  };
}

/// The matters a request asks the sixteen Tajika yogas about: all twelve,
/// or houses by number in the order you want them answered.
sealed class Matters {
  const Matters._();

  /// The twelve, first to twelfth.
  static const Matters all = _AllMatters();

  /// These houses, 1 to 12, in this order; a house named twice is refused.
  const factory Matters.houses(List<int> houses) = _Houses;

  Object get _json;
}

final class _AllMatters extends Matters {
  const _AllMatters() : super._();

  @override
  Object get _json => 'all';
}

final class _Houses extends Matters {
  const _Houses(this.houses) : super._();

  final List<int> houses;

  @override
  Object get _json => houses;
}

/// How the Tajika aspects read a pair less than a degree past (crux C112).
enum SubDegree {
  /// Poorna, the default: an Ithasala fulfilled.
  poorna('POORNA'),

  /// Ishrafa: already drawing apart.
  ishrafa('ISHRAFA');

  const SubDegree(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Which lord a Tambira lets reach the next sign.
enum TambiraMover {
  /// The karyesha: the definition's, and the default.
  karyesha('KARYESHA'),

  /// Either lord: the source's "some authorities".
  eitherLord('EITHER_LORD');

  const TambiraMover(this.key);

  /// The key the boundary reads.
  final String key;
}

/// When the Moon counts among Kuttha's benefics (crux C117).
enum MoonBenefic {
  /// Always: Charak's list, and the default.
  always('ALWAYS'),

  /// Waxing only: the commentary's "full Moon", read as the bright half.
  waxing('WAXING');

  const MoonBenefic(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Where the source leaves the sixteen Tajika yogas a choice
/// (`03-design/tajika-yogas.md`). A reading left null is the SDK's own
/// default, which lives in one place and is not repeated here.
final class YogaRules {
  const YogaRules({
    this.subDegree,
    this.weakBelow,
    this.strongFrom,
    this.tambira,
    this.moonBenefic,
  });

  /// How a pair less than a degree past reads.
  final SubDegree? subDegree;

  /// The strength below which a planet with no dignity is weak, in
  /// **sub-sub units**, 3600 to a unit: `5 * 3600` by default (crux C116).
  final int? weakBelow;

  /// The strength from which a planet is strong, sub-sub units; `10 * 3600`
  /// by default.
  final int? strongFrom;

  /// Which lord a Tambira lets reach the next sign.
  final TambiraMover? tambira;

  /// When the Moon counts among Kuttha's benefics.
  final MoonBenefic? moonBenefic;

  Map<String, Object?> get _json => <String, Object?>{
    if (subDegree case final subDegree?)
      'drishti': {'subDegree': subDegree.key},
    if (weakBelow case final weakBelow?) 'weakBelow': weakBelow,
    if (strongFrom case final strongFrom?) 'strongFrom': strongFrom,
    if (tambira case final tambira?) 'tambira': tambira.key,
    if (moonBenefic case final moonBenefic?) 'moonBenefic': moonBenefic.key,
  };
}

/// Where a year's own chart is cast: the birthplace, or a residence in the
/// parts `found` itself takes.
sealed class AnnualPlace {
  const AnnualPlace._();

  /// The birth chart's own place and clock.
  static const AnnualPlace birth = _Birthplace();

  /// A residence, under the clock kept there.
  const factory AnnualPlace.at(
    Observer observer, {
    required int utcOffsetSeconds,
  }) = _Residence;

  Object get _json;
}

final class _Birthplace extends AnnualPlace {
  const _Birthplace() : super._();

  @override
  Object get _json => 'birth';
}

final class _Residence extends AnnualPlace {
  const _Residence(this.observer, {required this.utcOffsetSeconds}) : super._();

  final Observer observer;
  final int utcOffsetSeconds;

  @override
  Object get _json => <String, Object>{
    'latitudeDeg': observer.latitudeDeg,
    'longitudeDeg': observer.longitudeDeg,
    'altitudeM': observer.altitudeM,
    'utcOffsetSeconds': utcOffsetSeconds,
  };
}

/// The annual chart's five office-bearers, one of whom becomes the lord of
/// the year.
final class OfficeBearers {
  const OfficeBearers({
    required this.muntha,
    required this.janmaLagna,
    required this.varshaLagna,
    required this.triRashi,
    required this.dinaRatri,
  });

  /// The lord of the Muntha's sign.
  final Graha muntha;

  /// The lord of the birth lagna.
  final Graha janmaLagna;

  /// The lord of the annual lagna.
  final Graha varshaLagna;

  /// The annual lagna's triplicity lord for the part of the day.
  final Graha triRashi;

  /// The lord of the Sun's sign by day, of the Moon's by night.
  final Graha dinaRatri;
}

/// Who takes the year when no office-bearer aspects the lagna.
enum NoneAspects {
  /// The Muntha's lord: Charak's rule, and the default.
  munthaLord('MUNTHA_LORD'),

  /// The annual lagna's lord, which "some authorities" give.
  annualLagnaLord('ANNUAL_LAGNA_LORD'),

  /// The strongest of the five: the Nilakanthi's Varshatantra v. 11.
  strongest('STRONGEST');

  const NoneAspects(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Who takes the year when the office-bearers tie outright.
enum VarsheshaTied {
  /// The Muntha's lord: the source's rule, and the default.
  munthaLord('MUNTHA_LORD'),

  /// The Dina-Ratri Pati, which "still others" give.
  dinaRatriPati('DINA_RATRI_PATI');

  const VarsheshaTied(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Whether the Moon may hold the year, and who takes it when it may not.
enum MoonMayRule {
  /// Passed over for the next claimant that aspects, and else for its
  /// Ithasala successor: Charak's two steps, and the default.
  passedOver('PASSED_OVER'),

  /// Its Ithasala successor at once: the Nilakanthi's own view.
  ithasala('ITHASALA'),

  /// It holds the year like any other office-bearer.
  likeAnyOther('LIKE_ANY_OTHER');

  const MoonMayRule(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Which planets may succeed the Moon through an Ithasala.
enum MoonPartner {
  /// Any of the seven: the default.
  anyPlanet('ANY_PLANET'),

  /// Only an office-bearer, as one commentary reads the verse.
  officeBearer('OFFICE_BEARER');

  const MoonPartner(this.key);

  /// The key the boundary reads.
  final String key;
}

/// Where the sources differ on the lord of the year, each a named reading
/// (`03-design/varshesha.md`). A reading left null is the SDK's own
/// default, which lives in one place and is not repeated here.
final class VarsheshaRules {
  const VarsheshaRules({
    this.noneAspects,
    this.tied,
    this.moon,
    this.moonPartner,
    this.subDegree,
  });

  /// Who takes the year when nobody aspects the lagna.
  final NoneAspects? noneAspects;

  /// Who takes it on an outright tie.
  final VarsheshaTied? tied;

  /// Whether the Moon may hold it.
  final MoonMayRule? moon;

  /// Who may succeed the Moon.
  final MoonPartner? moonPartner;

  /// How a pair less than a degree past reads, for the Ithasala the
  /// Moon's successor needs.
  final SubDegree? subDegree;

  Map<String, Object?> get _json => <String, Object?>{
    if (noneAspects case final noneAspects?) 'noneAspects': noneAspects.key,
    if (tied case final tied?) 'tied': tied.key,
    if (moon case final moon?) 'moon': moon.key,
    if (moonPartner case final moonPartner?) 'moonPartner': moonPartner.key,
    if (subDegree case final subDegree?)
      'drishti': {'subDegree': subDegree.key},
  };
}

/// A Tajika strength, exact. The boundary carries it as an integer count
/// of **sub-sub units**, 3600 to a unit, because two office-bearers a
/// sub-sub unit apart decide a year between them.
final class Bala {
  /// A strength from the sub-sub units the boundary carries.
  factory Bala(int subSub) {
    final units = subSub ~/ 3600;
    final rest = subSub - units * 3600;
    return Bala._(units, rest ~/ 60, rest % 60, subSub);
  }

  const Bala._(this.units, this.subUnits, this.subSub, this.total);

  /// Whole units, at most twenty: the figure a reader compares.
  final int units;

  /// The sub-units after those, 0 to 59.
  final int subUnits;

  /// The sub-sub units after those, 0 to 59.
  final int subSub;

  /// The whole of it in sub-sub units: what to compare and sum.
  final int total;

  /// `14:20:15`, as the sources write one.
  @override
  String toString() =>
      '${units.toString().padLeft(2, '0')}:'
      '${subUnits.toString().padLeft(2, '0')}:'
      '${subSub.toString().padLeft(2, '0')}';
}

/// One office-bearer's claim on the year's lordship.
final class YearClaim {
  const YearClaim({
    required this.graha,
    required this.vishwa,
    required this.portfolios,
    required this.aspectsLagna,
  });

  /// Whose claim it is.
  final Graha graha;

  /// Its five-fold strength.
  final Bala vishwa;

  /// How many of the five offices it holds, 1 to 5: the tie-break.
  final int portfolios;

  /// Whether it gives the Tajika aspect to the annual lagna, which it must
  /// to hold the year.
  final bool aspectsLagna;
}

/// The lord of the year, and the reckoning it came out of.
final class YearLord {
  const YearLord({
    required this.graha,
    required this.chosen,
    required this.vishwa,
    required this.moonPassedOver,
    required this.claims,
  });

  /// The lord of the year.
  final Graha graha;

  /// Which step of the chain decided it.
  final VarsheshaChosen chosen;

  /// Its five-fold strength.
  final Bala vishwa;

  /// Whether the Moon led on strength and stepped aside, being "unable to
  /// govern".
  final bool moonPassedOver;

  /// Every claimant, strongest first, so the decision can be read rather
  /// than trusted.
  final List<YearClaim> claims;
}

/// Two planets of an annual chart, and what they make.
final class TajikaPair {
  const TajikaPair({
    required this.faster,
    required this.slower,
    required this.drishti,
    required this.yoga,
    required this.orbDeg,
    required this.apartDeg,
  });

  /// The faster of the two by the tradition's ranking.
  final Graha faster;

  /// The slower.
  final Graha slower;

  /// The aspect between the signs they stand in.
  final TajikaDrishti drishti;

  /// What they are doing: coming together or drawing apart.
  final TajikaYoga yoga;

  /// The orb governing them, degrees: the mean of their deeptamshas.
  final double orbDeg;

  /// How far apart within their signs, degrees; positive when the faster
  /// is behind the slower and coming to it.
  final double apartDeg;
}

/// Two planets of an annual chart, and what they make — which may be
/// nothing.
final class TajikaBetween {
  const TajikaBetween({
    required this.faster,
    required this.slower,
    required this.drishti,
    required this.yoga,
    required this.orbDeg,
    required this.apartDeg,
  });

  /// The faster of the two by the tradition's ranking.
  final Graha faster;

  /// The slower.
  final Graha slower;

  /// The aspect between the signs they stand in.
  final TajikaDrishti drishti;

  /// What they are doing; null when they make neither an Ithasala nor an
  /// Ishrafa.
  final TajikaYoga? yoga;

  /// The orb governing them, degrees: the mean of their deeptamshas.
  final double orbDeg;

  /// How far apart within their signs, degrees; positive when the faster
  /// is behind the slower and coming to it.
  final double apartDeg;
}

/// The two lords' afflictions, clause by clause: what made a Rudda or a
/// Durapha.
final class Afflictions {
  const Afflictions({required this.lagnesha, required this.karyesha});

  /// The lagnesha's.
  final List<Affliction> lagnesha;

  /// The karyesha's.
  final List<Affliction> karyesha;
}

/// One of the sixteen holding, with what made it hold.
final class HeldYearYoga {
  const HeldYearYoga({
    required this.yoga,
    required this.between,
    required this.through,
    required this.entering,
    required this.legs,
    required this.afflictions,
  });

  /// Which of the sixteen.
  final YearYoga yoga;

  /// The lords' own relation, where that is what made it.
  final TajikaBetween? between;

  /// The third planet it turns on, where one does.
  final Graha? through;

  /// The planet judged on entering the next sign: Gairi-Kamboola's Moon,
  /// Tambira's lord.
  final Graha? entering;

  /// How the third planet stands to each of the pair, read from the next
  /// sign for [entering]; two, or null.
  final List<TajikaBetween>? legs;

  /// The lords' afflictions, where those made it: Rudda and Durapha.
  final Afflictions? afflictions;
}

/// The sixteen Tajika yogas for one matter of a year: the question it asked
/// as well as the answer, because a list of yogas whose pair a reader
/// cannot see is not checkable.
final class TajikaMatter {
  const TajikaMatter({
    required this.house,
    required this.sign,
    required this.lagnesha,
    required this.karyesha,
    required this.sameLord,
    required this.between,
    required this.held,
    required this.unanswered,
  });

  /// The house asked about, 1 to 12, counted from the annual lagna.
  final int house;

  /// The sign that house falls in.
  final Rashi sign;

  /// The lord of the annual lagna.
  final Graha lagnesha;

  /// The lord of the house asked about.
  final Graha karyesha;

  /// One planet is both — always so of the first house — so there is no
  /// pair to judge.
  final bool sameLord;

  /// How the two lords stand to each other; null when they are one.
  final TajikaBetween? between;

  /// Every yoga that holds, once for each third planet that makes it.
  final List<HeldYearYoga> held;

  /// The yogas this call could not answer for. A yoga absent from [held]
  /// did not hold **only** if it is not listed here.
  final List<YearYoga> unanswered;

  /// Whether [yoga] holds: null where this call could not say, which is
  /// not the same answer as false.
  bool? holds(YearYoga yoga) =>
      unanswered.contains(yoga) ? null : held.any((one) => one.yoga == yoga);
}

/// A return's own chart, read down to what Tajika reads from it.
final class AnnualChart {
  const AnnualChart({
    required this.lagnaDeg,
    required this.byDay,
    required this.officeBearers,
    required this.yearLord,
    required this.yogas,
    required this.retrograde,
    required this.combust,
    required this.matters,
    required this.sahams,
    required this.harsha,
    required this.dashas,
  });

  /// The annual chart's lagna, sidereal degrees, at the place it was cast
  /// for.
  final double lagnaDeg;

  /// Whether the return fell between sunrise and sunset there.
  final bool byDay;

  /// The five office-bearers.
  final OfficeBearers officeBearers;

  /// The lord of the year, chosen among them.
  final YearLord yearLord;

  /// The pairs of the seven that make a Tajika yoga in this chart. The
  /// pairs that make none do not cross; Rust's `sdk.chart().drishtis` has
  /// all twenty-one.
  final List<TajikaPair> yogas;

  /// The seven retrograde in this chart: what the matters were judged on.
  final List<Graha> retrograde;

  /// The seven combust in this chart, under the context's combustion
  /// table.
  final List<Graha> combust;

  /// The sixteen yogas for each matter [VarshaRequest.matters] asked
  /// about, in its order; empty otherwise.
  final List<TajikaMatter> matters;

  /// Each saham [VarshaRequest.sahams] asked for, in its order, with its
  /// strength under the year's lord; empty otherwise.
  final List<TajikaSaham> sahams;

  /// The seven's Harsha bala in this year's chart, in the catalogue's
  /// order.
  final List<HarshaBala> harsha;

  /// Each annual dasha [VarshaRequest.dashas] asked for, in its order,
  /// under [VarshaRequest.dashaRules]; empty otherwise.
  final List<AnnualDasha> dashas;
}

/// One planet's Harsha bala: four places it is happy in, five units each.
final class HarshaBala {
  const HarshaBala({
    required this.graha,
    required this.house,
    required this.sthana,
    required this.uchchaSwakshetra,
    required this.striPurusha,
    required this.dinaRatri,
    required this.total,
    required this.grade,
  });

  /// Whose.
  final Graha graha;

  /// The house it stands in, whole signs from the annual lagna.
  final int house;

  /// In its house of joy.
  final bool sthana;

  /// In its exaltation or own sign.
  final bool uchchaSwakshetra;

  /// In a house of its own gender, Tajika's genders.
  final bool striPurusha;

  /// In a year opening at its own part of the day.
  final bool dinaRatri;

  /// The parts held, five units each: 0 to 20.
  final int total;

  /// What the source calls that total.
  final HarshaGrade grade;
}

/// How one of the seven stands to a saham.
final class SahamSeven {
  const SahamSeven({
    required this.graha,
    required this.drishti,
    required this.relation,
    required this.company,
  });

  /// Which planet.
  final Graha graha;

  /// The Tajika aspect its sign casts on the saham's.
  final TajikaDrishti drishti;

  /// How it stands to the saham's lord, under the friendship read.
  final TajikaRelation relation;

  /// Whether it keeps the saham company, in the saham's sign.
  final bool company;
}

/// Where a saham fell in a year's chart, and what it fell in.
final class TajikaSaham {
  const TajikaSaham({
    required this.saham,
    required this.longitudeDeg,
    required this.sign,
    required this.lord,
    required this.house,
    required this.addedSign,
    required this.strong,
    required this.weak,
    required this.lordVishwa,
    required this.lordHarsha,
    required this.inNodeAxis,
    required this.seven,
  });

  /// Which of the forty-one.
  final Saham saham;

  /// Where it fell, sidereal degrees in [0, 360).
  final double longitudeDeg;

  /// The sign it fell in.
  final Rashi sign;

  /// That sign's lord: the saham's lord, by whose strength the source
  /// judges it.
  final Graha lord;

  /// The house it fell in, 1 to 12, by whole signs from the annual lagna.
  final int house;

  /// Whether it was carried a sign further because c did not fall between
  /// b and a.
  final bool addedSign;

  /// The clauses of the source's strong list that hold. Reported and never
  /// weighed: the source gives no score, and three sahams in five meet
  /// clauses on both lists.
  final List<SahamStrong> strong;

  /// The clauses of the source's weak list that hold.
  final List<SahamWeak> weak;

  /// The saham lord's Panchavargiya Vishwa bala.
  final Bala lordVishwa;

  /// The saham lord's Harsha bala grade.
  final HarshaGrade lordHarsha;

  /// Whether it stands in the Rahu-Ketu axis; null when the chart placed
  /// no nodes.
  final bool? inNodeAxis;

  /// How each of the seven stands to it, in the catalogue's order.
  final List<SahamSeven> seven;

  /// In the 6th, 8th or 12th, where the source calls a saham handicapped.
  bool get handicapped => house == 6 || house == 8 || house == 12;
}

/// A saham section's columns and the seven rows under it, from the years'
/// sections or the births': the generated classes share no type, and one
/// decoder must read both so they cannot drift.
final class _SahamCols {
  _SahamCols.year(Charts batch)
    : saham = batch.yearSahams.saham,
      longitudeDeg = batch.yearSahams.longitudeDeg,
      sign = batch.yearSahams.sign,
      lord = batch.yearSahams.lord,
      house = batch.yearSahams.house,
      addedSign = batch.yearSahams.addedSign,
      strong = batch.yearSahams.strong,
      weak = batch.yearSahams.weak,
      lordVishwa = batch.yearSahams.lordVishwa,
      lordHarsha = batch.yearSahams.lordHarsha,
      nodeAxis = batch.yearSahams.nodeAxis,
      sevenGraha = batch.yearSahamSeven.graha,
      sevenDrishti = batch.yearSahamSeven.drishti,
      sevenRelation = batch.yearSahamSeven.relation,
      sevenCompany = batch.yearSahamSeven.company;

  _SahamCols.natal(Charts batch)
    : saham = batch.natalSahams.saham,
      longitudeDeg = batch.natalSahams.longitudeDeg,
      sign = batch.natalSahams.sign,
      lord = batch.natalSahams.lord,
      house = batch.natalSahams.house,
      addedSign = batch.natalSahams.addedSign,
      strong = batch.natalSahams.strong,
      weak = batch.natalSahams.weak,
      lordVishwa = batch.natalSahams.lordVishwa,
      lordHarsha = batch.natalSahams.lordHarsha,
      nodeAxis = batch.natalSahams.nodeAxis,
      sevenGraha = batch.natalSahamSeven.graha,
      sevenDrishti = batch.natalSahamSeven.drishti,
      sevenRelation = batch.natalSahamSeven.relation,
      sevenCompany = batch.natalSahamSeven.company;

  final List<int> saham;
  final List<double> longitudeDeg;
  final List<int> sign;
  final List<int> lord;
  final List<int> house;
  final List<int> addedSign;
  final List<int> strong;
  final List<int> weak;
  final List<int> lordVishwa;
  final List<int> lordHarsha;
  final List<int> nodeAxis;
  final List<int> sevenGraha;
  final List<int> sevenDrishti;
  final List<int> sevenRelation;
  final List<int> sevenCompany;

  /// Row [k]: where it fell, its strength clause by clause, and the seven
  /// rows under it.
  TajikaSaham at(int k) => TajikaSaham(
    saham: Saham.byId(saham[k]),
    longitudeDeg: longitudeDeg[k],
    sign: Rashi.byId(sign[k]),
    lord: Graha.byId(lord[k]),
    house: house[k],
    addedSign: addedSign[k] == 1,
    strong: _members(strong[k], SahamStrong.values, (c) => c.id),
    weak: _members(weak[k], SahamWeak.values, (c) => c.id),
    lordVishwa: Bala(lordVishwa[k]),
    lordHarsha: HarshaGrade.byId(lordHarsha[k]),
    // 2 is the boundary's "the chart placed no nodes to read".
    inNodeAxis: nodeAxis[k] == 2 ? null : nodeAxis[k] == 1,
    seven: [
      for (var at = 7 * k; at < 7 * k + 7; at += 1)
        SahamSeven(
          graha: Graha.byId(sevenGraha[at]),
          drishti: TajikaDrishti.byId(sevenDrishti[at]),
          relation: TajikaRelation.byId(sevenRelation[at]),
          company: sevenCompany[at] == 1,
        ),
    ],
  );
}

/// Where each row's block starts in each ragged section under the annual
/// charts: `starts[row]` to `starts[row + 1]`. Computed once for a chart's
/// years rather than once a year, so reading them is linear.
final class _Starts {
  _Starts(Charts batch)
    : claims = _running(batch.annualCharts.claimCount),
      yogas = _running(batch.annualCharts.yogaCount),
      matters = _running(batch.annualCharts.matterCount),
      held = _running(batch.yearMatters.heldCount),
      legs = _running(batch.matterYogas.legCount),
      sahams = _running(batch.annualCharts.sahamCount),
      dashas = _running(batch.annualCharts.dashaCount),
      shares = _running(batch.yearDashas.shareCount),
      periods = _running(batch.yearDashas.periodCount);

  final List<int> claims;
  final List<int> yogas;
  final List<int> matters;
  final List<int> held;
  final List<int> legs;
  final List<int> sahams;
  final List<int> dashas;
  final List<int> shares;
  final List<int> periods;

  static List<int> _running(List<int> counts) {
    final starts = List<int>.filled(counts.length + 1, 0);
    for (var i = 0; i < counts.length; i += 1) {
      starts[i + 1] = starts[i] + counts[i];
    }
    return starts;
  }
}

/// The members of a bit set over a small closed enum, in id order: bit `n`
/// is the member with id `n`.
List<T> _members<T>(int bits, List<T> values, int Function(T) id) => [
  for (final member in values)
    if (bits & (1 << id(member)) != 0) member,
];

/// The seven the Tajika bit sets range over, Sun to Saturn: ids 0 to 6.
final List<Graha> _theSeven = List<Graha>.generate(7, Graha.byId);

/// The seven a year's bit set names, in graha id order.
List<Graha> _seven(int bits) => _members(bits, _theSeven, (g) => g.id);

/// The nine a Jaimini bit set ranges over, Sun to Ketu: ids 0 to 8.
final List<Graha> _theNine = List<Graha>.generate(9, Graha.byId);

/// The nine a Jaimini bit set names, in graha id order.
List<Graha> _nine(int bits) => _members(bits, _theNine, (g) => g.id);

/// Where the Muntha stands inside the sign it has reached (crux C107).
///
/// Both readings give the same sign at the return and part over the
/// course of the year, so they differ for a Tajika aspect taken to the
/// Muntha and for nothing else.
enum MunthaDegree {
  /// It enters each year at its sign's first degree and crosses the whole
  /// sign during the year: the source's own reading.
  signStart('SIGN_START'),

  /// It carries the natal lagna's degree into each new sign.
  natalDegree('NATAL_DEGREE');

  const MunthaDegree(this.key);

  /// The key the boundary takes.
  final String key;
}

/// The Muntha at one return: the birth lagna's sign advanced one sign for
/// each completed year, and that sign's lord.
final class Muntha {
  const Muntha({
    required this.sign,
    required this.lord,
    required this.longitudeDeg,
  });

  /// The sign it has reached; the same under either reading.
  final Rashi sign;

  /// The lord of that sign: the Munthesha, first of the annual chart's
  /// five office-bearers and the one that takes the year's lordship when
  /// no other qualifies.
  final Graha lord;

  /// Its longitude at the return, degrees, under the reading asked for.
  final double longitudeDeg;
}

/// One annual chart's instant.
final class Pravesha {
  const Pravesha({
    required this.year,
    required this.instant,
    required this.muntha,
    this.annual,
  });

  /// The Muntha standing at it, progressed by this year's own count.
  final Muntha muntha;

  /// The year's own chart, or null unless `varsha:` named a `place`.
  final AnnualChart? annual;

  /// How many years the native has completed at this instant: 1 is the
  /// first return, a year after birth. Counted in returns and not in
  /// years of life, because the two namings differ by one and both are
  /// in use.
  final int year;

  /// The instant, a Julian day (UTC), to pass to `found`.
  final double instant;
}

final class PlanRequest {
  /// A request for the composers named.
  const PlanRequest({
    this.placements = false,
    this.readings = false,
    this.strength = false,
    this.houses = false,
    this.positions = false,
    this.aspects = false,
    this.conditions = false,
    this.karakas = false,
    this.chalit = false,
    this.phala = false,
    this.bhavaBala = false,
    this.vimshopaka = false,
    this.panchanga = false,
    this.states = false,
    this.dashaPhala = false,
    this.ashtakavarga = false,
    this.sadeSati = false,
  });

  /// Where each of the nine grahas stands and who shares a sign.
  final bool placements;

  /// What each rule the chart held says.
  final bool readings;

  /// Each graha's Shadbala in rupas, the strongest first.
  final bool strength;

  /// The lord of each of the twelve bhavas, first house first.
  final bool houses;

  /// Where each graha stands to the degree, which `placements` rounds away.
  final bool positions;

  /// Which graha looks at which, and how strongly.
  final bool aspects;

  /// What each graha is where it stands: its dignity, its navamsha and the
  /// vargottama it may make, its retrogression and its combustion.
  final bool conditions;

  /// Which chara karaka each graha holds, under both schemes.
  final bool karakas;

  /// Where the placement system and the chalit put a graha in different
  /// bhavas. It says nothing of a chart whose readings agree.
  final bool chalit;

  /// What a loaded corpus of state readings says of this chart's subjects:
  /// a graha in a bhava, the lagna's sign, each limb of the panchanga, and
  /// what the birth nakshatra is. It says nothing until a pack carrying
  /// those readings is loaded.
  final bool phala;

  /// Each bhava's strength in virupas, the first house first. It says the
  /// weight and never a verdict: a bhava carries no requirement.
  final bool bhavaBala;

  /// Each graha's Vimshopaka under all four schemes, each item naming the
  /// scheme it belongs to.
  final bool vimshopaka;

  /// The almanac of the chart's day: the tithi with its paksha, the vara,
  /// the nakshatra and the Moon's pada in it, the yoga and the karana, and
  /// whether the birth fell by day where the chart says.
  final bool panchanga;

  /// The other half of a graha's state: how it stands to its dispositor
  /// under all three friendships, and the four avasthas — the fifth of its
  /// sign, its wakefulness, its brightness where the chart decides one,
  /// and the lajjitadi that hold beside the ones nothing decides.
  final bool states;

  /// What each graha's placement says of its dasha: when in the dasha its
  /// effects come, whether its place is auspicious, the points its dignity
  /// earns and whether the placement makes the dasha favourable — with the
  /// reading a loaded corpus carries of that graha as a dasha lord. It
  /// reads the dasha phala section, computed for you when asked.
  final bool dashaPhala;

  /// What the Ashtakavarga says: each graha's bindus in the sign it
  /// stands in, and each sign's sarvashtakavarga. The two numbers a text
  /// quotes and no verdict, because the reading carries no threshold. It
  /// reads the Ashtakavarga section and the chart's own placements, both
  /// computed for you when asked.
  final bool ashtakavarga;

  /// What a loaded corpus says of Saturn's periods from the natal Moon: the
  /// reading of each Sade Sati phase and smaller spell the chart's report
  /// holds, each house once, in the order Saturn first reaches it. It says
  /// the report the request's `sadeSati` window finds, so it needs that
  /// window beside it, and like `phala` it says nothing until a pack of
  /// state readings is loaded.
  final bool sadeSati;

  String get _json => jsonEncode(<String, Object?>{
    'placements': placements,
    'readings': readings,
    'strength': strength,
    'houses': houses,
    'positions': positions,
    'aspects': aspects,
    'conditions': conditions,
    'karakas': karakas,
    'chalit': chalit,
    'phala': phala,
    'bhavaBala': bhavaBala,
    'vimshopaka': vimshopaka,
    'panchanga': panchanga,
    'states': states,
    'dashaPhala': dashaPhala,
    'ashtakavarga': ashtakavarga,
    'sadeSati': sadeSati,
  });
}

/// Each batch's drawings, parsed once however many charts read them.
final Expando<List<List<Drawing>>> _drawings = Expando<List<List<Drawing>>>(
  'drawings',
);

List<List<Drawing>> _drawingsOf(Charts batch) =>
    _drawings[batch] ??= _parseDrawings(batch);

List<List<Drawing>> _parseDrawings(Charts batch) {
  if (batch.drawings.isEmpty) return const <List<Drawing>>[];
  final written =
      batch.svgs.isEmpty
          ? const <Object?>[]
          : jsonDecode(batch.svgs) as List<Object?>;
  final charts = jsonDecode(batch.drawings) as List<Object?>;
  return [
    for (var chart = 0; chart < charts.length; chart += 1)
      [
        for (final (index, raw) in (charts[chart]! as List<Object?>).indexed)
          Drawing._of(
            raw! as Map<String, Object?>,
            chart < written.length
                ? (written[chart]! as List<Object?>)[index]! as String
                : null,
          ),
      ],
  ];
}

/// The drawings asked for, as the packed ids the boundary takes: `layout << 16
/// | varga` each, so a caller names pairs and nothing else writes bits
/// (`03-design/chart-geometry.md`).
List<int> _drawingBits(
  List<(KeyOf<ChartLayout>, Varga)> drawings,
  Map<String, int> registered,
) => [
  for (final (index, (layout, varga)) in drawings.indexed)
    ((switch (layout) {
              ChartLayout(:final id) => id,
              // A consumer's own, from the ids its context resolved when it
              // was made (§7f).
              _ =>
                registered[layout.fullKey] ??
                    (throw ArgumentError.value(
                      layout.fullKey,
                      'drawings[$index].layout',
                      'not a layout this context registered',
                    )),
            }) <<
            16) |
        varga.id,
];

/// One divisional chart of one founded moment.
final class VargaChart {
  const VargaChart({
    required this.varga,
    required this.lagna,
    required this.grahas,
  });

  /// Which divisional chart.
  final Varga varga;

  /// Where the lagna falls in it.
  final VargaPlacement lagna;

  /// Every graha, in the order the foundation carries them.
  final List<PlacedInVarga> grahas;
}

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

/// A day with no sunrise or no sunset: which, and what the policy did
/// about it.
final class PolarDay {
  const PolarDay({required this.kind, required this.policy});

  /// Whether the Sun stayed up or stayed down.
  final PolarKind kind;

  /// The policy that put bounds on the day.
  final PolarDayPolicy policy;
}

/// A local day, as a chart and an almanac both read it: the civil date,
/// its weekday, the sunrise that opened it, its sunset and the sunrise
/// that closes it, whether it had a sunrise at all, and by which
/// convention. The same record in every binding.
final class LocalDay {
  const LocalDay({
    required this.date,
    required this.vara,
    required this.sunrise,
    required this.sunset,
    required this.nextSunrise,
    required this.polar,
    required this.convention,
    required this.customAltitudeDeg,
    required this.air,
  });

  /// One row of a `day` section -- a chart's or an almanac's, which share
  /// it -- read into the record both layers hand back.
  factory LocalDay._of(Day section, int i) {
    final custom = section.conventionKind[i] == _customSunrise;
    final era = section.era[i];
    return LocalDay(
      date: CalendarDate(
        calendar: Calendar.byId(section.calendar[i]),
        era: era == noMember ? null : Era.byId(era),
        year: section.year[i],
        eraYear: section.eraYear[i],
        month: section.month[i],
        day: section.dayOfMonth[i],
        resolution: Resolution.byId(section.resolution[i]),
        computedMonth: section.computedMonth[i],
        computedDay: section.computedDay[i],
      ),
      vara: Vara.byId(section.vara[i]),
      sunrise: section.sunrise[i],
      sunset: section.sunset[i],
      nextSunrise: section.nextSunrise[i],
      polar:
          DayState.byId(section.stateKind[i]) == DayState.polar
              ? PolarDay(
                kind: PolarKind.byId(section.statePolarKind[i]),
                policy: PolarDayPolicy.byId(section.statePolarPolicy[i]),
              )
              : null,
      convention: custom ? null : Sunrise.byId(section.conventionKind[i]),
      customAltitudeDeg: custom ? section.conventionValue[i] : null,
      // No air has a pressure of zero, so a zero says the convention named
      // none.
      air:
          section.airPressureHpa[i] > 0
              ? Air(
                pressureHpa: section.airPressureHpa[i],
                temperatureC: section.airTemperatureC[i],
              )
              : null,
    );
  }

  /// The convention column's value for a custom sunrise altitude.
  static const int _customSunrise = 0xFF;

  /// The civil date, as `calendar.convert` returns one, so it can be
  /// handed back to it.
  final CalendarDate date;

  /// The weekday, which the sunrise-anchored reckoning keeps from sunrise
  /// to sunrise.
  final Vara vara;

  /// The sunrise that opened the day, or what the polar policy put in its
  /// place, as a Julian day (UTC).
  final double sunrise;

  /// The sunset that closed its daylight, as a Julian day (UTC).
  final double sunset;

  /// The sunrise that closes it, as a Julian day (UTC).
  final double nextSunrise;

  /// `null` for a day the Sun rose and set on; what happened instead, for
  /// one it did not.
  final PolarDay? polar;

  /// The named sunrise convention the day was reckoned by; `null` for a
  /// custom altitude.
  final Sunrise? convention;

  /// The custom altitude of the Sun's centre, degrees, when [convention]
  /// is `null`; `null` otherwise.
  final double? customAltitudeDeg;

  /// The air the horizon was refracted through, resolved at the place,
  /// when the settings named one (`{'kind': 'ATMOSPHERIC', ...}`); `null`
  /// for the almanac's fixed 34′ or no refraction.
  final Air? air;
}

/// An air as it was applied: a part the settings left out is the engines'
/// standard at the place, the ICAO atmosphere's pressure at its height and
/// 15 °C. The same record in every binding.
final class Air {
  const Air({required this.pressureHpa, required this.temperatureC});

  /// The pressure at the observer, hectopascals.
  final double pressureHpa;

  /// The temperature at the observer, degrees Celsius.
  final double temperatureC;
}

/// Where in its day a chart's moment falls: the ishtakaal and the hora.
/// The same record in every binding.
final class ChartTiming {
  const ChartTiming({
    required this.ghati,
    required this.pala,
    required this.vipala,
    required this.ghatiReckoning,
    required this.horaNumber,
    required this.horaLord,
    required this.horaStart,
    required this.horaEnd,
  });

  /// The ishtakaal's ghatis since sunrise, 0 to 59.
  final int ghati;

  /// Its palas, 0 to 59.
  final int pala;

  /// Its vipalas, 0 to 59.
  final int vipala;

  /// How the ghatis were measured.
  final GhatiReckoning ghatiReckoning;

  /// Which hora of the day holds the instant, 1 to 24.
  final int horaNumber;

  /// The graha that rules it.
  final Graha horaLord;

  /// When that hora began, as a Julian day (UTC).
  final double horaStart;

  /// When it ends, as a Julian day (UTC).
  final double horaEnd;
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

  /// What computed this chart, and under what: the batch's provenance
  /// stamped with this chart's own `contentHash`.
  Provenance get provenance =>
      _memberProvenance(batch.provenanceJson, batch.contentHashes, index);

  /// The instant the chart is cast for, as a Julian day (UTC).
  double get instant => batch.cast.instant[index];

  /// The lagna at the instant, in the chart's zodiac, degrees.
  double get lagnaDeg => batch.cast.lagnaDeg[index];

  /// The lagna at the sunrise that opened the day, degrees.
  double get dayLagnaDeg => batch.cast.dayLagnaDeg[index];

  /// The ayanamsha applied at this instant, degrees; zero if tropical.
  double get ayanamshaOffsetDeg => batch.cast.ayanamshaOffsetDeg[index];

  /// The catalogued ayanamsha this chart was read under, or `null` when
  /// none was applied -- a tropical chart -- or the settings defined their
  /// own, which [ayanamshaCustom] says. One for the batch, since the frame
  /// is the request's.
  ///
  /// Not on [Charts], whose generated `ayanamsha` is the raw id and would
  /// win over an extension member of the same name.
  Ayanamsha? get ayanamsha =>
      batch.ayanamshaKind == 1 ? Ayanamsha.byId(batch.ayanamsha) : null;

  /// Whether the ayanamsha is one the settings define rather than a
  /// catalogued one.
  bool get ayanamshaCustom => batch.ayanamshaKind == 2;

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

  /// The day the chart's moment belongs to, which may be the civil date
  /// before the instant's: its date, weekday and sunrises.
  LocalDay get day => LocalDay._of(batch.day, index);

  /// Where in its day the moment falls, in the reckonings the settings
  /// named.
  ChartTiming get timing {
    final t = batch.timing;
    return ChartTiming(
      ghati: t.ghati[index],
      pala: t.pala[index],
      vipala: t.vipala[index],
      ghatiReckoning: GhatiReckoning.byId(t.ghatiReckoning[index]),
      horaNumber: t.horaNumber[index],
      horaLord: Graha.byId(t.horaLord[index]),
      horaStart: t.horaStart[index],
      horaEnd: t.horaEnd[index],
    );
  }

  /// What each graha **is**, as opposed to where it is — or an empty
  /// list unless `state: true` asked for it.
  ///
  /// The motion is not here: `grahas[j].retrograde` already says it.
  List<GrahaState> get states {
    final st = batch.states;
    if (st.length == 0) {
      return const <GrahaState>[];
    }
    final count = batch.grahaCount;
    final base = index * count;
    // `id >= 0` skips the generated unknown sentinel every catalogue
    // enum carries: a bit set is over the members the catalogue has.
    List<AvasthaLajjitadi> set(int bits) => <AvasthaLajjitadi>[
      for (final member in AvasthaLajjitadi.values)
        if (member.id >= 0 && bits & (1 << member.id) != 0) member,
    ];
    return List<GrahaState>.generate(count, (j) {
      final i = base + j;
      return GrahaState(
        graha: Graha.byId(st.graha[i]),
        sign: Rashi.byId(st.sign[i]),
        house: st.house[i],
        dignity: Dignity.byId(st.dignity[i]),
        friendship: Friendship(
          natural: Relationship.byId(st.natural[i]),
          temporary: Relationship.byId(st.temporary[i]),
          compound: Relationship.byId(st.compound[i]),
          dispositor:
              st.hasDispositor[i] != 0 ? Graha.byId(st.dispositor[i]) : null,
        ),
        combustion: Combustion(
          burning: Burning.byId(st.burning[i]),
          fromSunDeg: st.hasFromSun[i] != 0 ? st.fromSunDeg[i] : null,
          orbDeg: st.hasOrbs[i] != 0 ? st.orbDeg[i] : null,
          deepOrbDeg: st.hasDeepOrb[i] != 0 ? st.deepOrbDeg[i] : null,
        ),
        age: AvasthaBaladi.byId(st.age[i]),
        wakefulness: AvasthaJagradadi.byId(st.wakefulness[i]),
        deeptadi:
            st.hasDeeptadi[i] != 0
                ? AvasthaDeeptadi.byId(st.deeptadi[i])
                : null,
        lajjitadi: Lajjitadi(
          holding: set(st.lajjitadiHolding[i]),
          ruledOut: set(st.lajjitadiRuledOut[i]),
          undecided: set(st.lajjitadiUndecided[i]),
        ),
        war:
            st.hasWar[i] != 0
                ? War(
                  opponent: Graha.byId(st.warOpponent[i]),
                  isWinner: st.warWon[i] != 0,
                  apartDeg: st.warApartDeg[i],
                )
                : null,
        sayanadi:
            st.hasSayanadi[i] != 0
                ? Sayanadi(
                  avastha: AvasthaSayanadi.byId(st.sayanadi[i]),
                  cheshtas: List.unmodifiable([
                    for (final column in [
                      st.cheshta1,
                      st.cheshta2,
                      st.cheshta3,
                      st.cheshta4,
                      st.cheshta5,
                    ])
                      AvasthaCheshta.byId(column[i]),
                  ]),
                )
                : null,
        boundaries: EdgeDistance(
          signDeg: st.signDeg[i],
          nakshatraDeg: st.nakshatraDeg[i],
          padaDeg: st.padaDeg[i],
        ),
      );
    });
  }

  /// The twelve bhavas as the houses service reads them, or an empty
  /// list unless `houses: true` asked for them.
  ///
  /// `sign` is the sign the bhava's **middle** falls in, which under an
  /// unequal division is not the sign it begins in.
  List<ServiceBhava> get bhavas {
    final b = batch.bhavas;
    if (b.length == 0) {
      return const <ServiceBhava>[];
    }
    final base = index * 12;
    return List<ServiceBhava>.generate(
      12,
      (j) => ServiceBhava(
        number: j + 1,
        sign: Rashi.byId(b.sign[base + j]),
        lord: Graha.byId(b.lord[base + j]),
        quadrant: Quadrant.byId(b.quadrant[base + j]),
      ),
    );
  }

  /// The derived points — the upagrahas and the special lagnas — or an
  /// empty list unless `points: true` asked for them.
  ///
  /// Gulika and Mandi are Saturn's eighth of the day's arc, so they are
  /// the two a chart with no arc to divide cannot have — which is why
  /// the section is ragged.
  List<DerivedPoint> get points {
    final counts = batch.cast.pointCount;
    var from = 0;
    for (var i = 0; i < index; i += 1) {
      from += counts[i];
    }
    final p = batch.points;
    return List<DerivedPoint>.generate(counts[index], (k) {
      final i = from + k;
      return DerivedPoint(
        point: Point.byId(p.point[i]),
        longitudeDeg: p.longitudeDeg[i],
        sign: Rashi.byId(p.sign[i]),
        boundaries: EdgeDistance(
          signDeg: p.signDeg[i],
          nakshatraDeg: p.nakshatraDeg[i],
          padaDeg: p.padaDeg[i],
        ),
      );
    });
  }

  /// The drishti this chart casts, strongest first among those a body
  /// casts; empty unless `aspects: true` asked for them.
  ///
  /// The section is **ragged**: a chart's relations depend on where the
  /// bodies stand rather than on how many there are, so two charts of
  /// the same nine grahas hold different numbers of them, and
  /// `cast.aspect_count` is what says where each chart's begin.
  /// The annual charts' instants: the Sun's returns to where it stood at
  /// birth, in year order; empty unless `varsha:` asked for them
  /// (`03-design/annual-chart.md`).
  ///
  /// The section is **ragged** for a reason of its own: the request
  /// settles how many returns are *wanted* and the ephemeris settles how
  /// many there *are*, so read the length rather than the number you
  /// asked for.
  ///
  /// The place is yours. A return is an instant, and whether the annual
  /// chart is cast for the birthplace or for a residence is a choice the
  /// schools differ on, so pass the instant to `found` yourself.
  /// Row [row] of `annual_charts`, which runs beside `praveshas` row for
  /// row or is empty; anything between is a layout this layer cannot pair,
  /// and it says so rather than giving a year another year's chart.
  AnnualChart? _annualOf(int row, _Starts starts) {
    final charts = batch.annualCharts;
    final returns = batch.praveshas;
    if (charts.lagnaDeg.isEmpty) return null;
    if (charts.lagnaDeg.length != returns.year.length) {
      throw StateError(
        'annual_charts has ${charts.lagnaDeg.length} rows beside '
        '${returns.year.length} returns; it is all of them or none',
      );
    }
    // The claims are ragged by `claimCount`, as the returns are by
    // `praveshaCount`: this year's block starts where the ones before end.
    final from = starts.claims[row];
    final count = charts.claimCount[row];
    final claims = batch.yearClaims;
    final fromYoga = starts.yogas[row];
    final pairs = batch.yearYogas;
    return AnnualChart(
      lagnaDeg: charts.lagnaDeg[row],
      byDay: charts.daylight[row] == 1,
      officeBearers: OfficeBearers(
        muntha: Graha.byId(returns.munthaLord[row]),
        janmaLagna: Graha.byId(charts.janmaLagnaLord[row]),
        varshaLagna: Graha.byId(charts.varshaLagnaLord[row]),
        triRashi: Graha.byId(charts.triRashiLord[row]),
        dinaRatri: Graha.byId(charts.dinaRatriLord[row]),
      ),
      yearLord: YearLord(
        graha: Graha.byId(charts.yearLord[row]),
        chosen: VarsheshaChosen.byId(charts.yearLordChosen[row]),
        vishwa: Bala(charts.yearLordVishwa[row]),
        moonPassedOver: charts.moonPassedOver[row] == 1,
        claims: List<YearClaim>.generate(
          count,
          (k) => YearClaim(
            graha: Graha.byId(claims.graha[from + k]),
            vishwa: Bala(claims.vishwa[from + k]),
            portfolios: claims.portfolios[from + k],
            aspectsLagna: claims.aspectsLagna[from + k] == 1,
          ),
        ),
      ),
      yogas: List<TajikaPair>.generate(
        charts.yogaCount[row],
        (k) => TajikaPair(
          faster: Graha.byId(pairs.faster[fromYoga + k]),
          slower: Graha.byId(pairs.slower[fromYoga + k]),
          drishti: TajikaDrishti.byId(pairs.drishti[fromYoga + k]),
          yoga: TajikaYoga.byId(pairs.yoga[fromYoga + k]),
          orbDeg: pairs.orbDeg[fromYoga + k],
          apartDeg: pairs.apartDeg[fromYoga + k],
        ),
      ),
      retrograde: _seven(charts.retrograde[row]),
      combust: _seven(charts.combust[row]),
      matters: _mattersOf(row, starts),
      sahams: _sahamsOf(row, starts),
      harsha: _harshaOf(row),
      dashas: _annualDashasOf(row, starts),
    );
  }

  /// A year's annual dashas, ragged by `dashaCount`, each with its ring and
  /// its periods ragged under it by `shareCount` and `periodCount`
  /// (`03-design/annual-dashas.md`).
  List<AnnualDasha> _annualDashasOf(int row, _Starts starts) {
    final d = batch.yearDashas;
    final shares = batch.yearDashaShares;
    final p = batch.yearDashaPeriods;
    return [
      for (var k = starts.dashas[row]; k < starts.dashas[row + 1]; k += 1)
        AnnualDasha(
          system: DashaSystem.byId(d.system[k]),
          seed: d.seeded[k] == 1 ? Nakshatra.byId(d.seed[k]) : null,
          ring: DashaRing(
            shares: List<AnnualDashaShare>.unmodifiable([
              for (var i = starts.shares[k]; i < starts.shares[k + 1]; i += 1)
                AnnualDashaShare(
                  lord: Graha.byId(shares.lord[i]),
                  sign:
                      shares.hasSign[i] == 1
                          ? Rashi.byId(shares.sign[i])
                          : null,
                  weight: shares.weight[i],
                ),
            ]),
            first: d.first[k],
            remaining: d.remaining[k].isNaN ? null : d.remaining[k],
          ),
          year: Interval(from: d.fromJd[k], to: d.toJd[k]),
          periods: _periodsOf(
            level: p.level,
            index: p.index,
            sign: p.sign,
            lord: p.lord,
            from: p.fromJd,
            to: p.toJd,
            start: starts.periods[k],
            count: d.periodCount[k],
            signed: (i) => p.hasSign[i] == 1,
          ),
        ),
    ];
  }

  /// A year's sahams, ragged by `sahamCount`
  /// (`03-design/tajika-sahams.md`).
  List<TajikaSaham> _sahamsOf(int row, _Starts starts) {
    final cols = _SahamCols.year(batch);
    return [
      for (var k = starts.sahams[row]; k < starts.sahams[row + 1]; k += 1)
        cols.at(k),
    ];
  }

  /// A founded year's Harsha bala: seven rows a year, fixed, in the
  /// catalogue's order (`03-design/tajika-harsha.md`).
  List<HarshaBala> _harshaOf(int row) {
    final h = batch.yearHarsha;
    return [
      for (var at = 7 * row; at < 7 * row + 7; at += 1)
        HarshaBala(
          graha: Graha.byId(h.graha[at]),
          house: h.house[at],
          sthana: h.sthana[at] == 1,
          uchchaSwakshetra: h.uchchaSwakshetra[at] == 1,
          striPurusha: h.striPurusha[at] == 1,
          dinaRatri: h.dinaRatri[at] == 1,
          total: h.total[at],
          grade: HarshaGrade.byId(h.grade[at]),
        ),
    ];
  }

  /// How two planets stand, from the pair columns of a matter's lords.
  TajikaBetween _matterPair(int m) {
    final cols = batch.yearMatters;
    return TajikaBetween(
      faster: Graha.byId(cols.pairFaster[m]),
      slower: Graha.byId(cols.pairSlower[m]),
      drishti: TajikaDrishti.byId(cols.pairDrishti[m]),
      yoga:
          cols.pairYogaPresent[m] == 1
              ? TajikaYoga.byId(cols.pairYoga[m])
              : null,
      orbDeg: cols.pairOrbDeg[m],
      apartDeg: cols.pairApartDeg[m],
    );
  }

  /// How two planets stand, from one of a held yoga's legs.
  TajikaBetween _leg(int l) {
    final cols = batch.matterLegs;
    return TajikaBetween(
      faster: Graha.byId(cols.faster[l]),
      slower: Graha.byId(cols.slower[l]),
      drishti: TajikaDrishti.byId(cols.drishti[l]),
      yoga: cols.yogaPresent[l] == 1 ? TajikaYoga.byId(cols.yoga[l]) : null,
      orbDeg: cols.orbDeg[l],
      apartDeg: cols.apartDeg[l],
    );
  }

  /// A year's matters, each with its question, the lords' pair, what it
  /// could not answer and every yoga that held — ragged three deep
  /// (`03-design/tajika-yogas.md`, "Crossing the boundary").
  List<TajikaMatter> _mattersOf(int row, _Starts starts) => [
    for (var m = starts.matters[row]; m < starts.matters[row + 1]; m += 1)
      _matterAt(m, starts),
  ];

  /// Row [m] of `year_matters`, with the yogas and legs under it.
  TajikaMatter _matterAt(int m, _Starts starts) {
    final matters = batch.yearMatters;
    final held = batch.matterYogas;
    final between = matters.sameLord[m] == 1 ? null : _matterPair(m);
    return TajikaMatter(
      house: matters.house[m],
      sign: Rashi.byId(matters.sign[m]),
      lagnesha: Graha.byId(matters.lagnesha[m]),
      karyesha: Graha.byId(matters.karyesha[m]),
      sameLord: matters.sameLord[m] == 1,
      between: between,
      held: [
        for (var h = starts.held[m]; h < starts.held[m + 1]; h += 1)
          HeldYearYoga(
            yoga: YearYoga.byId(held.yoga[h]),
            between: held.byPair[h] == 1 ? between : null,
            through:
                held.throughPresent[h] == 1
                    ? Graha.byId(held.through[h])
                    : null,
            entering:
                held.enteringPresent[h] == 1
                    ? Graha.byId(held.entering[h])
                    : null,
            legs:
                held.legCount[h] == 0
                    ? null
                    : [
                      for (
                        var l = starts.legs[h];
                        l < starts.legs[h + 1];
                        l += 1
                      )
                        _leg(l),
                    ],
            afflictions:
                held.afflictionsPresent[h] == 1
                    ? Afflictions(
                      lagnesha: _members(
                        held.lagneshaAfflictions[h],
                        Affliction.values,
                        (a) => a.id,
                      ),
                      karyesha: _members(
                        held.karyeshaAfflictions[h],
                        Affliction.values,
                        (a) => a.id,
                      ),
                    )
                    : null,
          ),
      ],
      unanswered: _members(matters.unanswered[m], YearYoga.values, (y) => y.id),
    );
  }

  /// The birth chart's own sahams, each with its strength clause by
  /// clause — which has no year lord — in the order
  /// [VarshaRequest.sahams] named them; empty unless it asked. The source
  /// reads a year's sahams beside these, and they need no place
  /// (`03-design/tajika-saham-strength.md`).
  List<TajikaSaham> get sahams {
    final counts = batch.cast.natalSahamCount;
    var from = 0;
    for (var i = 0; i < index; i += 1) {
      from += counts[i];
    }
    final cols = _SahamCols.natal(batch);
    return [for (var k = from; k < from + counts[index]; k += 1) cols.at(k)];
  }

  List<Pravesha> get praveshas {
    final counts = batch.cast.praveshaCount;
    var from = 0;
    for (var i = 0; i < index; i += 1) {
      from += counts[i];
    }
    final p = batch.praveshas;
    final starts = _Starts(batch);
    return List<Pravesha>.generate(
      counts[index],
      (k) => Pravesha(
        year: p.year[from + k],
        instant: p.jd[from + k],
        muntha: Muntha(
          sign: Rashi.byId(p.munthaSign[from + k]),
          lord: Graha.byId(p.munthaLord[from + k]),
          longitudeDeg: p.munthaDeg[from + k],
        ),
        annual: _annualOf(from + k, starts),
      ),
    );
  }

  List<Drishti> get aspects {
    final counts = batch.cast.aspectCount;
    var from = 0;
    for (var i = 0; i < index; i += 1) {
      from += counts[i];
    }
    final count = counts[index];
    final a = batch.aspects;
    return List<Drishti>.generate(count, (k) {
      final i = from + k;
      return Drishti(
        from: Graha.byId(a.from[i]),
        to: Graha.byId(a.to[i]),
        houses: a.houses[i],
        strength: Strength.byId(a.strength[i]),
        fromEdge: EdgeDistance(
          signDeg: a.fromSignDeg[i],
          nakshatraDeg: a.fromNakshatraDeg[i],
          padaDeg: a.fromPadaDeg[i],
        ),
        toEdge: EdgeDistance(
          signDeg: a.toSignDeg[i],
          nakshatraDeg: a.toNakshatraDeg[i],
          padaDeg: a.toPadaDeg[i],
        ),
      );
    });
  }

  /// The Ashtakavarga, when `ashtakavarga: true` asked for it.
  Ashtakavarga? get ashtakavarga {
    final all = _ashtakavargasOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The Bhava bala, when `bhavaBala: true` asked for it.
  BhavaBala? get bhavaBala {
    final all = _bhavaBalasOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The Shadbala, when `shadbala: true` asked for it.
  Shadbala? get shadbala {
    final all = _shadbalasOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The dasha phala, when `dashaPhala: true` asked for it.
  DashaPhalaReading? get dashaPhala {
    final all = _dashaPhalasOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// Jaimini's significators, when `jaimini: true` asked for them.
  JaiminiReading? get jaimini {
    final all = _jaiminisOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The Moon's avakahada, when `avakahada: true` asked for it; a tropical
  /// chart refuses it, named `avakahada`.
  Avakahada? get avakahada {
    final all = _avakahadasOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The transits read against this chart, one reading an instant in the
  /// order `gochar:` asked; empty unless asked for.
  List<GocharReading> get gochar {
    final all = _gocharsOf(batch);
    return index < all.length ? all[index] : const <GocharReading>[];
  }

  /// The transit hit list, sorted by instant, then graha, then kind; empty
  /// unless `hits` asked for it (`03-design/transit-hit-list.md`). The
  /// section is ragged: a chart's aspects depend on where its points stand.
  List<Hit> get hits {
    final counts = batch.cast.hitCount;
    final h = batch.hits;
    final total = counts.fold<int>(0, (sum, count) => sum + count);
    if (total != h.length) {
      throw StateError(
        'hits has ${h.length} rows and cast.hit_count sums to $total; '
        "it is every chart's list, concatenated",
      );
    }
    var from = 0;
    for (var i = 0; i < index; i += 1) {
      from += counts[i];
    }
    return List<Hit>.generate(counts[index], (k) => _hitAt(h, from + k));
  }

  /// Sade Sati and Saturn's smaller spells, every period reaching into the
  /// window **whole**; null unless `sadeSati` asked for it
  /// (`03-design/sade-sati.md`). Saturn is searched once for the whole
  /// batch.
  SadeSatiReport? get sadeSati {
    final all = _sadeSatisOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The chart read as KP — its cusps and planets to the sub-sub lord, its
  /// significators in Reader VI's order and the ruling planets of its
  /// moment, under the settings' `kp` group; null unless `kp` asked for it
  /// (`03-design/kp.md`). A longitude and a lord's span are integers in
  /// nanoarcseconds, exact.
  KpReading? get kp {
    final all = _kpsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The seven planets' essential dignities and the chart's sect, with
  /// everything that made them; null unless `dignities` asked for them
  /// (`03-design/essential-dignities.md`).
  Dignities? get dignities {
    final all = _dignitiesOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// Both halves of Lilly's table, the essential dignities and the
  /// accidental fortitudes, with everything that made them; null unless
  /// `fortitudes` asked for them (`03-design/essential-dignities.md`
  /// §Accidental fortitudes).
  Fortitudes? get fortitudes {
    final all = _fortitudesOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// Valens's fourteen lots, with the chart's sect and the rules they were
  /// read under; null unless `lots` asked for them
  /// (`03-design/hellenistic-lots.md`).
  Lots? get lots {
    final all = _lotsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// Lilly's considerations before judgement, each clause with the facts
  /// it rests on; null unless `considerations` asked for them
  /// (`03-design/hellenistic-considerations.md`).
  Considerations? get considerations {
    final all = _considerationsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// Whether a horary matter is brought to pass, weighed on the chart's
  /// fortitudes and searched on the ephemeris; null unless `perfection`
  /// asked (`03-design/hellenistic-perfection.md`).
  Matter? get perfection {
    final all = _perfectionsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The birth read through its progressions: the progressed chart and the
  /// direction at [ProgressionsRequest.at], the contacts in its window;
  /// null unless `progressions` asked (`03-design/western-progressions.md`).
  Progressions? get progressions {
    final all = _progressionsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The Western aspect table, closest first: Leo's nine under his orbs by
  /// default (C240); null unless `westernAspects` asked
  /// (`03-design/western-aspects.md`).
  List<WesternAspectRow>? get westernAspects {
    final all = _westernAspectsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The chart's distances from the equator, its planets' and its angles';
  /// null unless `parallels` asked (`03-design/western-declinations.md`).
  Declinations? get declinations {
    final all = _declinationsOf(batch).$1;
    return index < all.length ? all[index] : null;
  }

  /// The chart's antiscia: each planet's reflection about the solstices
  /// and the equinoxes, and the pairs standing in one within the orbs,
  /// Lilly's moieties by default (C244); null unless `antiscia` asked
  /// (`03-design/western-antiscia.md`).
  Antiscia? get antiscia {
    final all = _antisciaOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The chart matched with a partner's birth by the Ashta Koota of
  /// *Muhurta Chintamani* VI.21–34 (`matching` asks for it, the chart on
  /// the side the partner leaves): each koota's points and what it read, in
  /// the verse's order, and the total out of 36. Never a verdict: the
  /// doshas and their exceptions are clauses; null unless asked
  /// (`03-design/matching.md`).
  AshtaKoota? get matching {
    final all = _matchingsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The chart matched with the same partner by the ten considerations of
  /// *Kalaprakasika* XIII (`matching` asks for both systems): whether each
  /// agrees and what it read, in the chapter's order, how many agree, how
  /// many of the chief five, and the p. 76 exception's clauses. Never a
  /// verdict: "at least five" is the reader's to apply; null unless asked
  /// (`03-design/matching.md`).
  Porutham? get porutham {
    final all = _poruthamsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The chart's Kuja dosha beside the same partner's (*Manasagari*,
  /// jāyābhāva v. 4; `matching` asks for it with both systems): Mars's house
  /// by sign from the lagna, the Moon and Venus on each side, whether each
  /// side carries the dosha under the rules and whether both do. Nothing is
  /// lifted; null unless asked (`03-design/matching.md`).
  Kuja? get kuja {
    final all = _kujasOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// Every marriage dosha the chart's match with the same partner carries,
  /// as one list in the answers' own order: the Ashta Koota's Bhakoot,
  /// Nadi, Gana and the lords' enmity, each of the ten that disagrees or
  /// agrees only by the p. 76 exception, and each side's Kuja dosha, each
  /// with whether it is lifted. No severity; null unless `matching` asked
  /// (`03-design/matching.md`).
  List<MarriageDosha>? get marriageDoshas {
    final all = _marriageDoshasOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The chart's harmonic chart: each planet, the ascendant and the
  /// midheaven at its longitude multiplied, in its equal house from the
  /// harmonic ascendant (C254), and every pair meeting within the orb, 12°
  /// by default (C252), closest first; null unless `harmonic` asked
  /// (`03-design/western-harmonics.md`).
  HarmonicChart? get harmonic {
    final all = _harmonicsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The chart's Western houses: the cusps of the asked division, else the
  /// profile's for the module, else Placidus (C249), and each planet's
  /// house and whether Leo reads it with the ascendant (C250); null unless
  /// `westernHouses` asked (`03-design/western-houses.md`).
  WesternHouses? get westernHouses {
    final all = _westernHousesOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The chart's equal distances, closest first: each planet within the
  /// orb of the axis through two others' midpoint, 0.5° by default (C245),
  /// on the shorter arc's midpoint or opposite it (C246); null unless
  /// `midpoints` asked (`03-design/western-midpoints.md`).
  List<MidpointRow>? get midpoints {
    final all = _midpointsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The parallels among the chart's planets, closest first: each pair the
  /// same distance from the equator within the orb (Leo's 1° by default),
  /// on either side of it (C243); null unless `parallels` asked.
  List<ParallelRow>? get parallels {
    final all = _declinationsOf(batch).$2;
    return index < all.length ? all[index] : null;
  }

  /// The Western aspects between this chart and the partner's, closest
  /// first; null unless `synastry` asked (`03-design/western-synastry.md`).
  List<SynastryRow>? get synastry {
    final all = _synastriesOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The parallels between this chart and the partner's, closest first;
  /// null unless `synastry` asked for `parallels`
  /// (`03-design/western-declinations.md`).
  List<SynastryParallelRow>? get synastryParallels {
    final all = _synastryParallelsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The antiscia between this chart and the partner's, closest first: a
  /// planet of this chart whose tropical longitude and one of the
  /// partner's sum to 180°, or 0° for the contrantiscion, within the orb
  /// read at the conjunction, Lilly's moieties by default (C244); null
  /// unless `synastry` asked for `antiscia`
  /// (`03-design/western-antiscia.md`).
  List<AntiscionRow>? get synastryAntiscia {
    final all = _synastryAntisciaOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The equal distances between this chart and the partner's, closest
  /// first: a planet of one chart within the orb of the axis through two of
  /// the other's, on the shorter arc's midpoint or opposite it, 0.5° by
  /// default (C245, C246); null unless `synastry` asked for `midpoints`
  /// (`03-design/western-midpoints.md`).
  List<SynastryMidpointRow>? get synastryMidpoints {
    final all = _synastryMidpointsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The composite of this chart and the partner's: each planet at the near
  /// midpoint of its two places, moving at the mean of its two speeds, the
  /// midheaven at the near midpoint of the two, and the lagna at theirs,
  /// turned by 180° when it stood before the midheaven (C247), in the
  /// synastry's zodiac; null unless `synastry` asked for `composite`
  /// (`03-design/western-composites.md`).
  Composite? get synastryComposite {
    final all = _synastryCompositesOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The Davison birth of this chart and the partner (C248): the mean of
  /// the two instants, of the two latitudes and altitudes, of the two
  /// longitudes the shorter way round, and of the two clocks, this chart's
  /// read on the request's; null unless `synastry` asked for `davison`
  /// (`03-design/western-composites.md`). It is a [Partner], so it founds
  /// a chart as a birth does, or stands as a synastry's partner.
  Partner? get synastryDavison {
    final all = _synastryDavisonsOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The Vaiseshikamsa, when `vaiseshikamsa: true` asked for it.
  VaiseshikamsaReading? get vaiseshikamsa {
    final all = _vaiseshikamsasOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The Vimshopaka, when `vimshopaka: true` asked for it.
  Vimshopaka? get vimshopaka {
    final all = _vimshopakasOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The dashas asked for, in the order asked; empty unless `dashas` named
  /// some (`03-design/dasha-kernels.md`).
  List<Dasha> get dashas {
    final all = _dashasOf(batch);
    return index < all.length ? all[index] : const <Dasha>[];
  }

  /// The charts drawn in the layouts asked for, in the order asked; empty
  /// unless `drawings` named some (`03-design/chart-geometry.md`).
  ///
  /// Each cell carries both the sign and the house it shows, and the bodies
  /// standing in it; `marks` places each body at its own degree on a wheel
  /// and is empty for a grid.
  List<Drawing> get drawings {
    final all = _drawingsOf(batch);
    return index < all.length ? all[index] : const <Drawing>[];
  }

  /// What this chart answers by rule, as the SDK writes it: `present`, each
  /// `{rule, result}` with the rule by key, and `houses` and `longevity` when
  /// asked; null unless the request named rules
  /// (`03-design/rules-at-the-boundary.md`).
  Map<String, Object?>? get rules {
    final all = _rulesOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// What this chart has to say, as the composers wrote it: a key per
  /// composer the request asked for, each a list of `{key, params}` holding
  /// no words at all — and only those the request named. An
  /// item's `params` are the very map [IntlArea.render] takes, so it says
  /// itself in the context's locale — and the same plan says it in any
  /// other. Null unless the request named a composer
  /// (`03-design/plans-at-the-boundary.md`).
  Map<String, Object?>? get plans {
    final all = _plansOf(batch);
    return index < all.length ? all[index] : null;
  }

  /// The divisional charts asked for, in the order they were asked.
  ///
  /// Empty unless `vargas` named some: a caller who wants a birth chart
  /// does not pay for twenty-one of them
  /// (`03-design/chart-reading.md` §4).
  List<VargaChart> get vargas {
    final count = batch.vargaCount;
    final grahaCount = batch.grahaCount;
    final v = batch.vargas;
    final g = batch.vargaGrahas;
    return List<VargaChart>.generate(count, (at) {
      final row = index * count + at;
      final from = row * grahaCount;
      return VargaChart(
        varga: Varga.byId(v.varga[row]),
        lagna: VargaPlacement(
          rashi: Rashi.byId(v.lagnaRashi[row]),
          part: v.lagnaPart[row],
          sign: Rashi.byId(v.lagnaSign[row]),
        ),
        grahas: List<PlacedInVarga>.generate(
          grahaCount,
          (j) => PlacedInVarga(
            graha: Graha.byId(batch.grahas.graha[index * grahaCount + j]),
            at: VargaPlacement(
              rashi: Rashi.byId(g.rashi[from + j]),
              part: g.part[from + j],
              sign: Rashi.byId(g.sign[from + j]),
            ),
          ),
        ),
      );
    });
  }

  /// The grahas, in the catalogue's order, one object each.
  ///
  /// The columns underneath are views over the blob's bytes, charts
  /// outermost; this reads this chart's stride out of them into the
  /// shape an application wants, which is a row.
  List<PlacedGraha> get grahas => _placed(batch.grahas, batch.grahaCount);

  /// Uranus, Neptune and Pluto, placed as the grahas are, or an empty
  /// list unless `outerPlanets: true` asked for them.
  ///
  /// ```dart
  /// final chart = ctx.chart.found(/* … */ outerPlanets: true);
  /// final uranus = chart.outer.firstWhere((at) => at.graha == Graha.uranus);
  /// ```
  ///
  /// The section holds the same number a chart, so the batch's rows
  /// divided by its charts is this chart's count.
  List<PlacedGraha> get outer {
    final o = batch.outer;
    final charts = batch.cast.instant.length;
    if (charts == 0) return const <PlacedGraha>[];
    if (o.length % charts != 0) {
      throw StateError('outer has ${o.length} rows for $charts charts');
    }
    // The generated section is its own class with the grahas' columns.
    final g = ChartsGrahas(
      graha: o.graha,
      longitudeDeg: o.longitudeDeg,
      tropicalDeg: o.tropicalDeg,
      latitudeDeg: o.latitudeDeg,
      distanceAu: o.distanceAu,
      speedDegPerDay: o.speedDegPerDay,
      houseBhava: o.houseBhava,
      houseMethod: o.houseMethod,
      houseThrough: o.houseThrough,
      houseFromMadhyaDeg: o.houseFromMadhyaDeg,
      placementBhava: o.placementBhava,
      placementMethod: o.placementMethod,
      placementThrough: o.placementThrough,
      placementFromMadhyaDeg: o.placementFromMadhyaDeg,
      length: o.length,
    );
    return _placed(g, o.length ~/ charts);
  }

  /// One chart's stride of a placed-bodies section, a row a body.
  List<PlacedGraha> _placed(ChartsGrahas g, int count) {
    final base = index * count;
    return List<PlacedGraha>.generate(count, (j) {
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

  /// What computed these charts, and under what; its `contentHash` is the
  /// whole batch's.
  Provenance get provenance => _provenanceOf(this, provenanceJson);

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
  const Span({
    required this.member,
    required this.whole,
    required this.inside,
    required this.sunrises,
    required this.ends,
  });

  /// Which member ran.
  final T member;

  /// When the member itself began and ended, inside the day or not.
  final Interval whole;

  /// The part inside the day: what an almanac row prints.
  final Interval inside;

  /// Which of the day's two sunrises the member was running at:
  /// [Sunrises.both] when it names two days (vriddhi), [Sunrises.neither]
  /// when it names none (kshaya).
  ///
  /// ```dart
  /// final kshaya = [
  ///   for (final span in day.tithi)
  ///     if (span.sunrises == Sunrises.neither) span.member,
  /// ];
  /// ```
  final Sunrises sunrises;

  /// When the member ended, in ghati-pala from the day's sunrise under
  /// `day.ghati_reckoning`; a member outlasting the day reads as the day's
  /// whole count.
  final GhatiPala ends;
}

/// A count from sunrise in ghatis of sixty palas of sixty vipalas.
final class GhatiPala {
  const GhatiPala({
    required this.ghati,
    required this.pala,
    required this.vipala,
  });

  /// Ghatis, 0 to 59 (60 when a civil day outlasts twenty-four hours).
  final int ghati;

  /// Palas, 0 to 59.
  final int pala;

  /// Vipalas, 0 to 59.
  final int vipala;
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

  /// What computed these days, and under what; its `contentHash` is the
  /// whole range's.
  late final Provenance provenance = Provenance.fromJson(
    jsonDecode(decoded.provenanceJson) as Map<String, Object?>,
  );

  /// The provenance envelope as the canonical JSON the library stamped:
  /// the bytes to store beside the result, byte-identical in every
  /// binding.
  String get provenanceJson => decoded.provenanceJson;

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

  /// The muhurta search the request asked for over these days, or `null`
  /// when it asked for none (`03-design/muhurta-at-the-boundary.md`): the
  /// windows judged clause by clause, the days the season closed, and what
  /// computed it. Parsed once.
  late final MuhurtaAnswer? muhurta =
      decoded.muhurta.isEmpty ? null : _muhurtaAnswer(decoded.muhurta);

  /// The days the rules the request asked for fall on over these days, or
  /// `null` when it asked for none (`03-design/festival-rules.md` §7): each
  /// observance with the case between its tithi's two days and the guard
  /// that decided. Parsed once.
  late final FestivalAnswer? festivals =
      decoded.festivals.isEmpty ? null : _festivalAnswer(decoded.festivals);

  /// The lunar years these days fall in, or `null` when the request did
  /// not ask with `years: true` (`03-design/calendar-indian-lunisolar.md`
  /// §10): each with the samvatsara it carries, its bounds from one
  /// Chaitra Shukla Pratipada's sunrise to the next, the Jovian years that
  /// ran in it and the one it expunged. Parsed once.
  late final LunarYears? years =
      decoded.years.isEmpty ? null : _lunarYears(decoded.years);

  /// The eclipses whose greatest moment falls in these days, or `null`
  /// when the request did not ask with `eclipses: true`
  /// (`03-design/eclipses.md`): each lunar and solar eclipse with its
  /// contacts and how the place sees it, the stretch above its horizon or
  /// `null`. Parsed once.
  late final Eclipses? eclipses =
      decoded.eclipses.isEmpty ? null : _eclipses(decoded.eclipses);

  /// Each day's Nepal Sambat date, or `null` when the request did not ask
  /// with `nepalSambat: true` (`03-design/calendar-indian-lunisolar.md`
  /// §11): one a day in the days' order, with its year, its month counted
  /// from Kachhala, the month's kind (adhika is Anala) and its half.
  /// Parsed once.
  late final NepalSambatDates? nepalSambat =
      decoded.nepalSambat.isEmpty ? null : _nepalSambat(decoded.nepalSambat);

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

  /// What computed this day, and under what: the batch's provenance
  /// stamped with this day's own `contentHash`.
  Provenance get provenance => _memberProvenance(
    batch.decoded.provenanceJson,
    batch.decoded.contentHashes,
    index,
  );

  /// The day itself: its date, weekday and sunrises.
  LocalDay get day => LocalDay._of(batch.decoded.day, index);

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

  /// Which season the day falls in, under `panchanga.ritu`: by default the
  /// season of the sidereal solar month the day belongs to, its first day
  /// placed by `panchanga.solar_month_start` (`03-design/ritu-measured.md`).
  Ritu get ritu => Ritu.byId(batch.decoded.days.ritu[index]);

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
      (c.sunrises, c.endsGhati, c.endsPala, c.endsVipala),
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
      (c.sunrises, c.endsGhati, c.endsPala, c.endsVipala),
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
      (c.sunrises, c.endsGhati, c.endsPala, c.endsVipala),
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
      (c.sunrises, c.endsGhati, c.endsPala, c.endsVipala),
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
      (c.sunrises, c.endsGhati, c.endsPala, c.endsVipala),
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
      (c.sunrises, c.endsGhati, c.endsPala, c.endsVipala),
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
      (c.sunrises, c.endsGhati, c.endsPala, c.endsVipala),
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
    (Uint8List, Uint8List, Uint8List, Uint8List) turns,
    T Function(int) byId,
  ) {
    final (sunrises, ghati, pala, vipala) = turns;
    return _rows(
      list,
      (i) => Span<T>(
        member: byId(member[i]),
        whole: Interval(from: wholeFrom[i], to: wholeTo[i]),
        inside: Interval(from: insideFrom[i], to: insideTo[i]),
        sunrises: Sunrises.byId(sunrises[i]),
        ends: GhatiPala(ghati: ghati[i], pala: pala[i], vipala: vipala[i]),
      ),
    );
  }
}

/// Each result's provenance, decoded once: an extension cannot hold a
/// field, so the generated result classes keep theirs here.
final Expando<Provenance> _provenances = Expando<Provenance>('provenance');

/// A result's provenance from its canonical JSON, decoded once.
Provenance _provenanceOf(Object result, String json) =>
    _provenances[result] ??= Provenance.fromJson(
      jsonDecode(json) as Map<String, Object?>,
    );

/// The provenance of one member of a batch handed out alone: the batch's,
/// with that member's own `content_hash` from the blob's `content_hashes`
/// section — so a chart founded alone carries the hash of its own value
/// and not of a list of one.
Provenance _memberProvenance(String json, String hashes, int index) =>
    Provenance.fromJson({
      ...jsonDecode(json) as Map<String, Object?>,
      'content_hash': hashes.substring(64 * index, 64 * index + 64),
    });
