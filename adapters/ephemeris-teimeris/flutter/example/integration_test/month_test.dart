// The month's hashes, computed inside the app on a device or simulator
// from the libraries the build hooks bundled, held to the desktop's.
//
// Run with the desktop's lines, `teistro-ephemeris-teimeris-day-hash`'s
// output, joined by `|`:
//
//   flutter test integration_test --dart-define=TEISTRO_EXPECTED="$(…| paste -sd'|' -)"

import 'package:flutter/foundation.dart' show debugPrint;
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:teistro_flutter/teistro_flutter.dart';
import 'package:teistro_teimeris_example/month.dart';

const String expected = String.fromEnvironment('TEISTRO_EXPECTED');

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  test('the phone gives the desktop answer', () {
    final teistro = Teistro.open();
    debugPrint('built-in ${hasBuiltin(teistro) ? 'bundled' : 'absent'}');
    final found = monthHashes(teistro);
    found.forEach(debugPrint);
    expect(
      expected,
      isNotEmpty,
      reason: 'TEISTRO_EXPECTED names the desktop lines',
    );
    expect(found, expected.split('|'));
  });
}
