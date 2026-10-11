// The four facts every shipped library answers (`facts.dart`, the suite
// the published Dart package's consumer runs), asserted inside the app on
// a device or simulator, from the library the build hook bundled.
//
// `cargo xtask check-flutter ios-simulator` and `android-emulator` run it.

import 'package:flutter/foundation.dart' show debugPrint;
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:teistro_flutter/teistro_flutter.dart';

import '../../../dart/packaging/facts.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  test('the bundled library answers as the published one does', () {
    final failures = <String>[];
    consumerFacts(Teistro.open(), (found, wanted, what) {
      if (found != wanted) {
        failures.add('$what: expected $wanted, found $found');
      }
    }, say: debugPrint);
    expect(failures, isEmpty);
  });
}
