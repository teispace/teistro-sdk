// An ephemeris provider written in Dart, bound into the port's vtable
// (`docs/02-architecture/07-binding-architecture.md`, "Ports across the
// boundary").
//
// HAND-WRITTEN, and one of only two files in this package that is. The
// architecture puts the port adapter in the ergonomic layer because every
// binding wraps its own callback mechanism: napi in Node, and here
// `NativeCallable.isolateLocal`, whose function pointer is callable only
// from the isolate that made it. That is exactly the boundary's contract
// (one context, one thread at a time), so the SDK calling back into Dart
// inside a call this isolate made is the only way it is ever reached.
//
// What the adapter does is small, because the port carries the machinery:
// it describes the provider once into the capabilities struct, and turns
// one call per grid into the columns the SDK allocated.

import 'dart:ffi' as ffi;
import 'dart:typed_data';

import 'package:ffi/ffi.dart' as pkg_ffi;

import 'catalogue.dart';
import 'ffi.dart';

/// The code a callback returns when Dart threw before it could answer.
/// `NativeCallable` takes it as a compile-time constant, so this is the
/// one number this file writes; [HostProvider] checks it against the
/// catalogue's own value when it binds, so the two cannot drift.
const int _refusedOnThrow = -4;

/// The grid a provider is asked to fill: one call for the whole of it,
/// never a loop. Cell `i * bodies.length + b` is body `b` at instant `i`,
/// which is the order the answer's columns are read in.
final class PositionQuery {
  const PositionQuery({
    required this.scale,
    required this.frameBits,
    required this.speeds,
    required this.observer,
    required this.jds,
    required this.bodies,
  });

  /// The scale the instants are on.
  final TimeScale scale;

  /// The frame the positions are wanted in, packed; `Teistro.unpackFrame`
  /// reads it back into its fields.
  final int frameBits;

  /// Whether speeds are wanted. A provider that computes none leaves
  /// those columns out.
  final bool speeds;

  /// The place a topocentric frame is seen from, null otherwise.
  final Observer? observer;

  /// The instants, on [scale].
  final Float64List jds;

  /// The bodies, in the order the cells run.
  final List<Body> bodies;

  /// How many cells the answer must hold.
  int get cellCount => jds.length * bodies.length;
}

/// The columns a provider answers with. A column left out is zeroes,
/// which is what a provider that computes no speeds means; `lon`, `lat`,
/// `dist` and `status` must be as long as [PositionQuery.cellCount].
final class PositionAnswer {
  const PositionAnswer({
    required this.lon,
    required this.lat,
    required this.dist,
    this.lonSpeed,
    this.latSpeed,
    this.distSpeed,
    this.status,
    this.source,
    this.frameBits,
  });

  /// Longitudes, degrees.
  final Float64List lon;

  /// Latitudes, degrees.
  final Float64List lat;

  /// Distances, in the provider's distance unit.
  final Float64List dist;

  /// Longitude speeds, degrees a day.
  final Float64List? lonSpeed;

  /// Latitude speeds, degrees a day.
  final Float64List? latSpeed;

  /// Distance speeds.
  final Float64List? distSpeed;

  /// Per-cell status codes; all zero (a value) when left out.
  final Int32List? status;

  /// Per-cell sources; unknown when left out.
  final Uint32List? source;

  /// The frame these values are in; the frame asked for when left out.
  final int? frameBits;
}

/// An ephemeris the SDK can drive, written in Dart.
///
/// Everything but [name], [bodies] and [positions] has a default, because
/// a provider that answers the canonical frame with apparent geocentric
/// positions is the common case and should not have to say so.
///
/// ```dart
/// final class StraightLine extends EphemerisProvider {
///   @override
///   String get name => 'straight-line';
///   @override
///   List<Body> get bodies => const [Body.sun];
///   @override
///   PositionAnswer positions(PositionQuery query) => PositionAnswer(
///     lon: Float64List(query.cellCount),
///     lat: Float64List(query.cellCount),
///     dist: Float64List(query.cellCount),
///   );
/// }
/// ```
abstract base class EphemerisProvider {
  /// A provider with the defaults.
  const EphemerisProvider();

  /// What the provider is, stamped in every result's provenance.
  String get name;

  /// The bodies it answers. A request for another is refused by name
  /// before the call reaches [positions].
  List<Body> get bodies;

  /// Its version; empty by default.
  String get version => '';

  /// What identifies its data, an ephemeris file's edition; empty by
  /// default.
  String get dataVersion => '';

  /// The first Julian day it covers; year 0 by default.
  double get jdMin => 1721057.5;

  /// The last Julian day it covers; year 3000 by default.
  double get jdMax => 2816787.5;

  /// The frame it returns natively; the SDK's canonical frame by
  /// default. A request in another frame reaches [positions] first, and
  /// answering null there has the SDK ask again in this one.
  Frame? get nativeFrame => null;

  /// Whether it computes speeds; true by default.
  bool get speeds => true;

  /// Whether identical requests give identical bits; true by default,
  /// and a provider that is not deterministic must say so, because the
  /// conformance contract rests on it (ADR-0022).
  bool get deterministic => true;

  /// What its distances are measured in.
  DistanceUnit get distanceUnit => DistanceUnit.astronomicalUnits;

  /// What its speeds are.
  SpeedModel get speedModel => SpeedModel.derivative;

  /// Whether it is modern astronomy or a classical text's model.
  Astronomy get astronomy => Astronomy.modern;

  /// The positions for a whole grid, or null for "not in that frame", in
  /// which case the SDK asks again in [nativeFrame] and completes the
  /// rest itself, stamping every step it applied.
  PositionAnswer? positions(PositionQuery query);
}

/// A [EphemerisProvider] bound into the port's vtable.
///
/// The vtable, the capability strings and the callbacks live as long as
/// this object, which the context that uses them owns; [dispose] closes
/// the callbacks and frees the memory, and the context is freed first.
final class HostProvider {
  HostProvider._(this.provider, this._lib);

  /// Binds a provider. The vtable is ready as soon as this returns.
  factory HostProvider(TeistroLibrary lib, EphemerisProvider provider) {
    if (provider.name.isEmpty) {
      throw ArgumentError.value(
        provider.name,
        'name',
        'a provider must have a name, which every result is stamped with',
      );
    }
    if (provider.bodies.isEmpty) {
      throw ArgumentError.value(
        provider.bodies,
        'bodies',
        'a provider must answer at least one body',
      );
    }
    final host = HostProvider._(provider, lib);
    host._build();
    return host;
  }

  /// The provider this drives.
  final EphemerisProvider provider;

  final TeistroLibrary _lib;

  /// What the provider threw, kept for the layer above to rethrow: only a
  /// code crosses the C boundary, and a provider written in Dart has more
  /// to say than a code.
  Object? thrown;

  late final ffi.Pointer<ProviderVtableStruct> _vtable;
  late final ffi.Pointer<CapabilitiesStruct> _capabilities;
  late final ffi.NativeCallable<CapabilitiesFnNative> _capabilitiesFn;
  late final ffi.NativeCallable<PositionsFnNative> _positionsFn;
  final List<ffi.Pointer<ffi.NativeType>> _owned = [];
  var _disposed = false;

  /// The vtable the SDK drives the provider through.
  ffi.Pointer<ProviderVtableStruct> get vtable => _vtable;

  /// The pointer the SDK hands back to every callback. The callbacks
  /// close over this object, so nothing has to be passed back; it is null
  /// rather than a fabricated address.
  ffi.Pointer<ffi.Void> get userData => ffi.nullptr;

  void _build() {
    if (_refusedOnThrow != ProviderCode.refused.id) {
      throw StateError(
        'the refusal code moved: the boundary says ${ProviderCode.refused.id}, '
        'this binding was written for $_refusedOnThrow',
      );
    }
    _capabilitiesFn = ffi.NativeCallable<CapabilitiesFnNative>.isolateLocal(
      _fillCapabilities,
      exceptionalReturn: _refusedOnThrow,
    );
    _positionsFn = ffi.NativeCallable<PositionsFnNative>.isolateLocal(
      _fillPositions,
      exceptionalReturn: _refusedOnThrow,
    );
    _capabilities = _describe();
    _vtable = pkg_ffi.calloc<ProviderVtableStruct>();
    _owned.add(_vtable);
    _vtable.ref
      ..structSize = ffi.sizeOf<ProviderVtableStruct>()
      ..abiVersion = vtableAbiVersion
      ..capabilities = _capabilitiesFn.nativeFunction
      ..positions = _positionsFn.nativeFunction;
  }

  /// The capabilities, described once: the SDK reads them whenever it
  /// asks, and nothing about a provider changes while it is bound.
  ffi.Pointer<CapabilitiesStruct> _describe() {
    final out = pkg_ffi.calloc<CapabilitiesStruct>();
    _owned.add(out);
    final bodies = pkg_ffi.calloc<ffi.Uint16>(provider.bodies.length);
    _owned.add(bodies);
    for (var i = 0; i < provider.bodies.length; i++) {
      bodies[i] = provider.bodies[i].id;
    }
    final frame = provider.nativeFrame;
    out.ref
      ..structSize = ffi.sizeOf<CapabilitiesStruct>()
      ..speeds = provider.speeds ? 1 : 0
      ..deterministic = provider.deterministic ? 1 : 0
      ..tier = 0
      ..distanceUnit = provider.distanceUnit.id
      ..speedModel = provider.speedModel.id
      ..astronomy = provider.astronomy.id
      ..name = _cString(provider.name)
      ..version = _cString(provider.version)
      ..dataVersion = _cString(provider.dataVersion)
      ..jdMin = provider.jdMin
      ..jdMax = provider.jdMax
      ..bodies = bodies
      ..bodyCount = provider.bodies.length
      ..nativeFrameBits =
          frame == null
              ? framePack(_lib, frameCanonical(_lib))
              : framePack(_lib, frame)
      ..overrides = 0
      ..ayanamshaCount = 0
      ..hashCount = 0;
    return out;
  }

  ffi.Pointer<ffi.Char> _cString(String text) {
    final pointer = text.toNativeUtf8(allocator: pkg_ffi.calloc);
    _owned.add(pointer);
    return pointer.cast<ffi.Char>();
  }

  int _fillCapabilities(
    ffi.Pointer<ffi.Void> userData,
    ffi.Pointer<CapabilitiesStruct> out,
  ) {
    if (out == ffi.nullptr) return ProviderCode.invalid.id;
    // The SDK reads what it asked for out of the struct we filled once,
    // field by field, so a struct of another size is still answered.
    final size = out.ref.structSize;
    out.ref = _capabilities.ref;
    out.ref.structSize = size;
    return ProviderCode.ok.id;
  }

  int _fillPositions(
    ffi.Pointer<ffi.Void> userData,
    ffi.Pointer<PositionRequestStruct> request,
    ffi.Pointer<PositionColumnsStruct> out,
  ) {
    if (request == ffi.nullptr || out == ffi.nullptr) {
      return ProviderCode.invalid.id;
    }
    try {
      return _answer(request.ref, out.ref);
    } catch (error) {
      // Only a code crosses; the sentence is kept for the layer above.
      thrown = error;
      return ProviderCode.refused.id;
    }
  }

  int _answer(PositionRequestStruct request, PositionColumnsStruct out) {
    final query = _read(request);
    // The same checks the port runs before a native provider is asked, so
    // a body, an observer or an instant a provider never declared is
    // refused here rather than left for it to discover.
    final refusal = _validate(query);
    if (refusal != null) return refusal;
    final cells = query.cellCount;
    if (out.capacity < cells) return ProviderCode.invalid.id;
    final answer = provider.positions(query);
    // Nothing means "not in that frame"; the SDK asks again in ours.
    if (answer == null) return ProviderCode.unsupported.id;
    for (final (name, column) in [
      ('lon', answer.lon),
      ('lat', answer.lat),
      ('dist', answer.dist),
    ]) {
      if (column.length != cells) {
        thrown = StateError(
          'the provider returned ${column.length} values in `$name` for '
          '$cells cells',
        );
        return ProviderCode.refused.id;
      }
    }
    out.frameBits = answer.frameBits ?? query.frameBits;
    _write(out.lon, answer.lon, cells);
    _write(out.lat, answer.lat, cells);
    _write(out.dist, answer.dist, cells);
    _write(out.lonSpeed, answer.lonSpeed, cells);
    _write(out.latSpeed, answer.latSpeed, cells);
    _write(out.distSpeed, answer.distSpeed, cells);
    final status = answer.status;
    final source = answer.source;
    for (var i = 0; i < cells; i++) {
      out.status[i] = status == null || i >= status.length ? 0 : status[i];
      out.source[i] = source == null || i >= source.length ? 0 : source[i];
    }
    return ProviderCode.ok.id;
  }

  /// The request as a Dart value. The instants and the ids are the SDK's
  /// memory, valid for the length of the call, so they are copied.
  PositionQuery _read(PositionRequestStruct request) => PositionQuery(
    scale: TimeScale.byId(request.scale),
    frameBits: request.frameBits,
    speeds: request.speeds != 0,
    observer:
        request.hasObserver == 0 ? null : Observer.readFrom(request.observer),
    jds: Float64List.fromList(request.jds.asTypedList(request.jdCount)),
    bodies: [
      for (var i = 0; i < request.bodyCount; i++) Body.byId(request.bodies[i]),
    ],
  );

  /// What the port checks before any provider is asked.
  int? _validate(PositionQuery query) {
    final frame = frameUnpack(_lib, query.frameBits);
    if (frame.centre == Centre.topocentric && query.observer == null) {
      thrown = StateError('a topocentric frame needs an observer');
      return ProviderCode.invalid.id;
    }
    for (final body in query.bodies) {
      if (!provider.bodies.contains(body)) {
        thrown = StateError(
          'the provider does not answer ${body.key}; it answers '
          '${provider.bodies.map((b) => b.key).join(', ')}',
        );
        return ProviderCode.unsupported.id;
      }
    }
    for (final jd in query.jds) {
      if (!jd.isFinite) {
        thrown = StateError('the instant $jd is not a number');
        return ProviderCode.invalid.id;
      }
      if (jd < provider.jdMin || jd > provider.jdMax) {
        thrown = StateError(
          'the instant $jd is outside the provider\'s coverage '
          '(${provider.jdMin} to ${provider.jdMax})',
        );
        return ProviderCode.outOfRange.id;
      }
    }
    return null;
  }

  static void _write(
    ffi.Pointer<ffi.Double> column,
    Float64List? values,
    int cells,
  ) {
    if (column == ffi.nullptr) return;
    final into = column.asTypedList(cells);
    if (values == null) {
      into.fillRange(0, cells, 0);
      return;
    }
    for (var i = 0; i < cells; i++) {
      into[i] = i < values.length ? values[i] : 0;
    }
  }

  /// Closes the callbacks and frees the vtable. The context that used
  /// them must be freed first, which [Context.dispose] sees to.
  void dispose() {
    if (_disposed) return;
    _disposed = true;
    _capabilitiesFn.close();
    _positionsFn.close();
    for (final pointer in _owned) {
      pkg_ffi.calloc.free(pointer);
    }
    _owned.clear();
  }
}
