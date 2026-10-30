// The SDK failing inside itself, as this layer reports it: never expected,
// and never silent (`docs/03-design/wasm-binding.md` §4, "A panic traps").
//
// The C ABI turns a panic into the `INTERNAL` status before it can cross, so
// a call refuses with it like any refusal. A host's crash reporter hears of
// one through `onInternalError`, with the status, its code, which kind it
// was and the SDK's version: never the message, which may quote what the
// call was given.

/// The stable code of the `INTERNAL` status, the same in every binding.
const int internalCode = -10;

/// What a crash reporter hears of an `INTERNAL` refusal. These fields are
/// all it hears: there is no message, on purpose.
final class InternalReport {
  /// A report of the SDK at [sdkVersion].
  const InternalReport(this.sdkVersion);

  /// The SDK that failed.
  final String sdkVersion;

  /// The status, as its key.
  String get status => 'INTERNAL';

  /// [internalCode].
  int get code => internalCode;

  /// `boundary`: a panic the C ABI caught, the only kind outside a wasm
  /// module.
  String get kind => 'boundary';

  @override
  String toString() => 'InternalReport($status, $code, $kind, $sdkVersion)';
}

final Set<void Function(InternalReport)> _hooks = {};

/// Calls [hook] with every [InternalReport] from now on; the function it
/// returns stops that. Hooks belong to the isolate that added them, and a
/// refusal is heard in the isolate it happened in.
///
/// ```dart
/// final stop = onInternalError((report) => print(report.sdkVersion));
/// stop();
/// ```
void Function() onInternalError(void Function(InternalReport report) hook) {
  _hooks.add(hook);
  return () {
    _hooks.remove(hook);
  };
}

/// Tells every hook. A hook that throws is not the caller's failure, so what
/// it throws is dropped and the others still hear.
void reportInternal(String sdkVersion) {
  final report = InternalReport(sdkVersion);
  for (final hook in _hooks.toList()) {
    try {
      hook(report);
    } catch (_) {
      // A crash reporter's own failure is its own.
    }
  }
}
