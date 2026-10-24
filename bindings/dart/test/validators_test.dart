// The branded quantities' constructors against the shared validator
// corpus (`docs/03-design/validator-corpus.md`): every value
// `idl/validators.json` lists as valid makes a quantity, and every
// invalid one throws, as the Rust newtypes and every other binding agree.
// The corpus is found by walking up from the directory the suite runs in.

import 'dart:convert';
import 'dart:io';

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final Map<String, double Function(double)> constructors = {
  'latitude': (value) => Latitude(value),
  'longitude': (value) => Longitude(value),
  'altitude': (value) => Altitude(value),
};

Map<String, Object?> corpus() {
  for (var dir = Directory.current.absolute; ; dir = dir.parent) {
    final file = File('${dir.path}/idl/validators.json');
    if (file.existsSync()) {
      return jsonDecode(file.readAsStringSync()) as Map<String, Object?>;
    }
    if (dir.parent.path == dir.path) {
      fail('no idl/validators.json above ${Directory.current.path}');
    }
  }
}

void main() {
  test('every branded constructor agrees with the shared corpus', () {
    final found = corpus();
    expect(found['schema'], 'teistro-validators/1');
    final brands =
        (found['brands']! as List<Object?>).cast<Map<String, Object?>>();
    expect(brands, isNotEmpty, reason: 'the corpus names no brand');
    for (final entry in brands) {
      final brand = entry['brand']! as String;
      final make = constructors[brand];
      expect(
        make,
        isNotNull,
        reason:
            '$brand is branded in the description and has no constructor here',
      );
      for (final text in (entry['valid']! as List<Object?>).cast<String>()) {
        expect(
          make!(double.parse(text)),
          double.parse(text),
          reason: '$brand($text) should be accepted',
        );
      }
      for (final text in (entry['invalid']! as List<Object?>).cast<String>()) {
        expect(
          () => make!(double.parse(text)),
          throwsA(isA<RangeError>()),
          reason: '$brand($text) should be refused',
        );
      }
    }
  });
}
