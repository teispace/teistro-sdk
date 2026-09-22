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

/// `TS_CHART_ASPECTS`, the one section bit this layer offers so far.
///
/// The bits are the C ABI's vocabulary; a consumer of this binding
/// writes `aspects: true` (`03-design/chart-reading.md` §5).
const int _sectionAspects = 4;

/// `TS_CHART_POINTS`, the derived points.
const int _sectionPoints = 8;

/// `TS_CHART_HOUSES`, the houses service.
const int _sectionHouses = 16;

/// `TS_CHART_ASHTAKAVARGA`, the Ashtakavarga.
const int _sectionAshtakavarga = 32;

/// `TS_CHART_VIMSHOPAKA`, the Vimshopaka.
const int _sectionVimshopaka = 64;

/// `TS_CHART_VAISESHIKAMSA`, the Vaiseshikamsa.
const int _sectionVaiseshikamsa = 512;

/// `TS_CHART_DASHA_PHALA`, the dasha phala.
const int _sectionDashaPhala = 1024;

/// `TS_CHART_SHADBALA`, the Shadbala.
const int _sectionShadbala = 128;

/// `TS_CHART_BHAVA_BALA`, the Bhava bala.
const int _sectionBhavaBala = 256;

/// `TS_CHART_STATE`, the planetary states.
const int _sectionState = 2;

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
    bool aspects = false,
    bool points = false,
    bool houses = false,
    bool ashtakavarga = false,
    bool vimshopaka = false,
    bool vaiseshikamsa = false,
    bool dashaPhala = false,
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
    aspects: aspects,
    points: points,
    houses: houses,
    ashtakavarga: ashtakavarga,
    vimshopaka: vimshopaka,
    vaiseshikamsa: vaiseshikamsa,
    dashaPhala: dashaPhala,
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
    bool aspects = false,
    bool points = false,
    bool houses = false,
    bool ashtakavarga = false,
    bool vimshopaka = false,
    bool vaiseshikamsa = false,
    bool dashaPhala = false,
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
                (aspects ? _sectionAspects : 0) |
                (points ? _sectionPoints : 0) |
                (houses ? _sectionHouses : 0) |
                (ashtakavarga ? _sectionAshtakavarga : 0) |
                (vimshopaka ? _sectionVimshopaka : 0) |
                (vaiseshikamsa ? _sectionVaiseshikamsa : 0) |
                (dashaPhala ? _sectionDashaPhala : 0) |
                (shadbala ? _sectionShadbala : 0) |
                (bhavaBala ? _sectionBhavaBala : 0) |
                (state ? _sectionState : 0),
            vargas: vargas,
            dashas: _dashaIds(dashas, _context._registeredDashas),
            drawings: _drawingBits(drawings, _context._registeredLayouts),
            themeJson: theme?._json,
            rulesJson: rules?._json,
            interpretJson: interpret?._json,
            varshaJson: varsha?._json,
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
  ///
  /// Depth first order means a period's children follow it, so one walk
  /// that takes the next level's running period finds the chain.
  List<DashaPeriod> at(double jd) {
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
      'quad' => QuadSegment(
        UnitPoint._of(raw['control']! as Map<String, Object?>),
        to,
      ),
      'arc' => ArcSegment(
        UnitPoint._of(raw['centre']! as Map<String, Object?>),
        raw['clockwise']! as bool,
        to,
      ),
      _ => LineSegment(to),
    };
  }

  Map<String, Object?> _json() => switch (this) {
    LineSegment() => {'kind': 'line', 'to': to._json()},
    QuadSegment(:final control) => {
      'kind': 'quad',
      'control': control._json(),
      'to': to._json(),
    },
    ArcSegment(:final centre, :final clockwise) => {
      'kind': 'arc',
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
  clockwise,

  /// Against them.
  anticlockwise,
}

/// What a grid cell always carries: a sign, or a house.
sealed class CellHolds {
  const CellHolds();

  factory CellHolds._of(Map<String, Object?> raw) => switch (raw['kind']) {
    'sign' => HoldsSign(Rashi.byKey(raw['value']! as String) ?? Rashi.unknown),
    _ => HoldsHouse(raw['value']! as int),
  };

  Map<String, Object?> _json() => switch (this) {
    HoldsSign(:final sign) => {'kind': 'sign', 'value': sign.key},
    HoldsHouse(:final house) => {'kind': 'house', 'value': house},
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
  lagna,

  /// The Moon's sign.
  moon,

  /// The Sun's sign.
  sun,

  /// The chart's cusps, each house as wide as it is.
  cusps,

  /// Twelve signs of 30°, turned so the lagna's degree sits at the start.
  zodiac,
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
    'counts_from': countsFrom.name,
  };
}

/// A layout's shape: twelve cells fixed in the row, or rings computed per
/// chart.
sealed class LayoutShape {
  const LayoutShape(this.direction);

  /// Which way the signs or houses run.
  final LayoutDirection direction;

  factory LayoutShape._of(Map<String, Object?> raw) {
    final direction = LayoutDirection.values.byName(
      raw['direction']! as String,
    );
    List<Map<String, Object?>> objects(String name) => [
      for (final item in raw[name]! as List<Object?>)
        item! as Map<String, Object?>,
    ];
    return switch (raw['kind']) {
      'radial' => RadialShape(
        rings: [
          for (final ring in objects('rings'))
            LayoutRing(
              inner: (ring['inner']! as num).toDouble(),
              outer: (ring['outer']! as num).toDouble(),
              countsFrom: RingReference.values.byName(
                ring['counts_from']! as String,
              ),
            ),
        ],
        startsAt: raw['starts_at']! as int,
        direction: direction,
      ),
      _ => GridShape(
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
    };
  }

  Map<String, Object?> _json() => switch (this) {
    GridShape(:final cells, :final frame) => {
      'kind': 'grid',
      'cells': [for (final cell in cells) cell._json()],
      'frame': [for (final path in frame) path._json()],
      'direction': direction.name,
    },
    RadialShape(:final rings, :final startsAt) => {
      'kind': 'radial',
      'rings': [for (final ring in rings) ring._json()],
      'starts_at': startsAt,
      'direction': direction.name,
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
///   RashiDashaDefinition(key: 'ACME_STHIRA', length: {'by_modality': {'movable': 7, 'fixed': 8, 'dual': 9}}),
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
    'kernel': 'udu',
    'key': key,
    'lords': [for (final lord in lords) lord.toJson()],
    'reference': reference.key,
    if (sources.isNotEmpty) 'sources': sources,
    if (count != null) 'count': count,
    if (span != null) 'span': span,
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

  /// `lagna` (the default), `arudha_lagna` or `navamsa_lagna`.
  final String? start;

  /// `consecutive` (the default), `trine_groups`, `drishti_chain` or `leap`.
  final String? order;

  /// `count_to_lord` (the default), `count_to_lord_by_dignity`,
  /// `{'fixed': 9}` or `{'by_modality': {'movable': 7, 'fixed': 8, 'dual': 9}}`.
  final Object? length;

  /// `stronger` (the default) or `first`.
  final String? namedLord;

  /// The houses from the lagna to start from the strongest of; none by
  /// default, which starts from `start` itself.
  final List<int> strongerOf;

  /// The length of its year (`JULIAN_365_25` by default, `SAVANA_360`, …).
  final String? yearLength;

  /// How many levels of periods a reading carries, 1 to 6; three by default.
  final int? depth;

  @override
  Map<String, Object?> toJson() => {
    'kernel': 'rashi',
    'key': key,
    if (sources.isNotEmpty) 'sources': sources,
    if (start != null) 'start': start,
    if (order != null) 'order': order,
    if (length != null) 'length': length,
    if (namedLord != null) 'namedLord': namedLord,
    if (strongerOf.isNotEmpty) 'strongerOf': strongerOf,
    if (yearLength != null) 'year_length': yearLength,
    if (depth != null) 'depth': depth,
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
  short('short'),

  /// The symbol: `☉`.
  glyph('glyph');

  const BodyForm(this.key);

  /// The key the theme record spells it with.
  final String key;
}

/// What a drawn cell's label shows.
enum CellLabel {
  /// The sign's number, or on a wheel the house and the sign's glyph.
  auto('auto'),

  /// The sign's number, 1 for Aries.
  signNumber('sign_number'),

  /// The sign's abbreviation.
  signShort('sign_short'),

  /// The sign's symbol.
  signGlyph('sign_glyph'),

  /// The house's number.
  house('house'),

  /// Nothing.
  nothing('nothing');

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
  static const light = ChartTheme._('light', ThemeStyle(), ThemeContent());

  /// Light ink on a dark page.
  static const dark = ChartTheme._('dark', ThemeStyle(), ThemeContent());

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
  final path = <int>[];
  final periods = List<DashaPeriod>.generate(count, (k) {
    final i = start + k;
    final level = p.level[i];
    path
      ..length = level - 1
      ..add(p.index[i]);
    return DashaPeriod(
      path: path.join('/'),
      level: level,
      sign: signed ? Rashi.byId(p.sign[i]) : null,
      lord: Graha.byId(p.lord[i]),
      from: p.fromJd[i],
      to: p.toJd[i],
    );
  }, growable: false);
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
    periods: List<DashaPeriod>.unmodifiable(periods),
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
  doshas,

  /// The recording engine's yogas the SDK computes.
  yogas,

  /// The gandantas of BPHS ch. 92.
  gandantas,

  /// The evils at birth and their cancellations.
  arishtas,

  /// The generated readings of a graha, a pair and a rising part.
  readings,

  /// The yogas, doshas and classes of life written from the texts.
  nabhasas,
}

/// The readings a request evaluates rules under.
enum RuleReadings {
  /// The texts' readings wherever a text settles one.
  texts('texts'),

  /// The recording engine's.
  recordingEngine('recording-engine');

  const RuleReadings(this._key);
  final String _key;
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
    'shipped': [for (final set in shipped) set.name],
    'rules': rules,
    'readings': readings._key,
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
  sidereal('sidereal'),

  /// The natal tropical longitude: the Western solar return. Forty years
  /// on it is most of a circle of lagna from the sidereal one, so it is a
  /// choice and never a fallback.
  tropical('tropical'),

  /// A whole sidereal year for each year of life, from birth: the older
  /// arithmetic, and the only reading that needs no ephemeris.
  mean('mean');

  const VarshaReading(this.key);

  /// The key the boundary reads.
  final String key;
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
  });

  /// The last year of life wanted, 1 to 200.
  final int through;

  /// Which longitude the Sun returns to.
  final VarshaReading reading;

  /// Where the Muntha stands inside the sign it has reached.
  final MunthaDegree muntha;

  String get _json => jsonEncode(<String, Object?>{
    'reading': reading.key,
    'through': through,
    'muntha': muntha.key,
  });
}

/// Where the Muntha stands inside the sign it has reached (crux C107).
///
/// Both readings give the same sign at the return and part over the
/// course of the year, so they differ for a Tajika aspect taken to the
/// Muntha and for nothing else.
enum MunthaDegree {
  /// It enters each year at its sign's first degree and crosses the whole
  /// sign during the year: the source's own reading.
  signStart('sign_start'),

  /// It carries the natal lagna's degree into each new sign.
  natalDegree('natal_degree');

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
  });

  /// The Muntha standing at it, progressed by this year's own count.
  final Muntha muntha;

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
  List<Pravesha> get praveshas {
    final counts = batch.cast.praveshaCount;
    var from = 0;
    for (var i = 0; i < index; i += 1) {
      from += counts[i];
    }
    final p = batch.praveshas;
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
