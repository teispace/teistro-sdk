// The SDK failing inside itself (SR-52): an `INTERNAL` refusal at the
// boundary reaches a host's crash reporter, without the message.
//
// No build of the library can be made to panic on request, so the real
// library's own sentences are classified here at the layer that classifies
// them, given the `INTERNAL` status a panic answers with.

import 'dart:ffi' as ffi;

import 'package:ffi/ffi.dart' as pkg_ffi;
import 'package:teistro/src/ffi.dart' show ErrorStruct, plainRefusal, refusalOf;
import 'package:teistro/src/internal.dart' show reportInternal;
import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

void main() {
  final library = Teistro.open().library;

  /// `refusalOf` over an empty record of the library's own layout.
  TeistroException record(int status) => pkg_ffi.using((arena) {
    final raw = arena<ErrorStruct>();
    raw.ref.structSize = ffi.sizeOf<ErrorStruct>();
    return refusalOf(status, raw);
  });

  test(
    'a hook hears the status, code, kind and version, never the message',
    () {
      final heard = <InternalReport>[];
      final stop = onInternalError(heard.add);
      final thrower = onInternalError(
        (_) => throw StateError('a broken reporter'),
      );
      reportInternal('1.2.3');
      stop();
      stop();
      thrower();
      reportInternal('1.2.3');
      expect(heard, hasLength(1));
      final report = heard.single;
      expect(report.status, 'INTERNAL');
      expect(report.code, -10);
      expect(report.kind, 'boundary');
      expect(report.sdkVersion, '1.2.3');
      expect(
        report.toString(),
        'InternalReport(INTERNAL, -10, boundary, 1.2.3)',
      );
    },
  );

  test('the code is the INTERNAL status of every binding', () {
    expect(internalCode, -10);
    expect(Status.internal.id, internalCode);
  });

  test('an INTERNAL refusal tells the hook, by either way it is read', () {
    final heard = <InternalReport>[];
    addTearDown(onInternalError(heard.add));
    expect(record(internalCode).status, Status.internal);
    final plain = plainRefusal(library, internalCode);
    expect(plain.status, Status.internal);
    expect(plain.message, 'internal error');
    expect(heard.map((r) => r.sdkVersion), [
      generatedSdkVersion,
      generatedSdkVersion,
    ]);
  });

  test('a refusal of any other status tells no one', () {
    final heard = <InternalReport>[];
    addTearDown(onInternalError(heard.add));
    expect(record(Status.invalidArg.id).status, Status.invalidArg);
    expect(
      plainRefusal(library, Status.invalidArg.id).status,
      Status.invalidArg,
    );
    expect(
      () => Teistro.open().context(
        profile: 'no-such-profile',
        testProvider: true,
      ),
      throwsA(isA<TeistroException>()),
    );
    expect(heard, isEmpty);
  });
}
