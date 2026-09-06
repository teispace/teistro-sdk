// The generated decoders against blobs the library really produced.
//
// `cargo xtask check-dart` writes the fixtures with
// `cargo run -p teistro-ffi --example blob_fixtures` and runs this file;
// `TEISTRO_FIXTURES` names the directory they are in.

import 'dart:io';
import 'dart:typed_data';

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final _dir = Platform.environment['TEISTRO_FIXTURES'] ?? '../../target/tsrb';

Uint8List _read(String name) => File('$_dir/$name').readAsBytesSync();

void main() {
  test('a positions blob decodes into views over its own bytes', () {
    final bytes = _read('positions.tsrb');
    final positions = decodePositions(bytes);

    // The summary says what grid the cells cover.
    expect(positions.jdCount, 2);
    expect(positions.bodyCount, 3);
    expect(positions.scale, 0, reason: 'UT1, the scale the request named');

    // The instants and bodies come back in the order they were asked for.
    expect(positions.instants.jd, [2451545.0, 2451546.0]);
    expect(positions.bodies.body, [0, 1, 4], reason: 'the Sun, the Moon, Mars');
    expect(positions.instants.length, 2);
    expect(positions.bodyKeys, [Body.sun, Body.moon, Body.mars]);
    expect(positions.timeScale, TimeScale.ut1);

    // One row per cell, instants outermost.
    final cells = positions.cells;
    expect(cells.length, 6);
    expect(cells.lon, isA<Float64List>());
    expect(cells.status, isA<Int32List>());
    expect(cells.source, isA<Uint32List>());
    for (var i = 0; i < cells.length; i++) {
      expect(cells.status[i], 0, reason: 'cell $i has a value');
      expect(cells.lon[i], inInclusiveRange(0, 360), reason: 'cell $i');
    }
    // The Moon moves faster than the Sun, and everything moves.
    expect(cells.lonSpeed[1].abs(), greaterThan(cells.lonSpeed[0].abs()));

    // The columns are views: writing through one writes into the blob.
    final before = cells.lon[0];
    cells.lon[0] = -1;
    expect(
      decodePositions(bytes).cells.lon[0],
      -1,
      reason: 'the column shares the buffer',
    );
    cells.lon[0] = before;

    // The steps and the provenance are the JSON the library wrote.
    final steps = positions.stepsApplied;
    expect(steps, isNotEmpty);
    for (final step in steps.cast<Map<String, Object?>>()) {
      expect(step['name'], isA<String>());
      expect(step['implementation'], isA<String>());
    }
    final provenance = positions.provenanceOf;
    expect(provenance['profile'], 'nepali-default');
    expect(provenance['calculation_version'], 1);
    expect((provenance['settings_hash']! as String).length, 64);
    expect(
      (provenance['provider']! as Map<String, Object?>)['frame'],
      'GEOCENTRIC/OF_DATE/ECLIPTIC/TROPICAL/APPARENT',
    );
    expect(
      (provenance['time']! as Map<String, Object?>)['delta_t_model'],
      'TABLE_THEN_MODEL',
    );
  });

  test('a render blob decodes its text, its locale and its warnings', () {
    final rendered = decodeIntlRender(_read('intl_render.tsrb'));
    expect(rendered.resolvedFrom, 'ne-Deva-NP');
    expect(rendered.from, 'ne-Deva-NP');
    expect(rendered.fallback, isFalse);
    expect(rendered.override, isFalse);
    expect(rendered.warningCount, 0);
    expect(rendered.warningList, isEmpty);
    expect(rendered.text, contains('७'), reason: 'the Nepali numeral seven');
    expect(
      rendered.text,
      contains('गुरु'),
      reason: 'Jupiter by its Nepali name',
    );
  });

  test('a blob of the wrong shape is refused, never misread', () {
    final bytes = _read('positions.tsrb');
    expect(
      () => decodePositions(Uint8List(4)),
      throwsA(
        isA<FormatException>().having(
          (e) => e.message,
          'message',
          contains('not a Teistro result blob'),
        ),
      ),
    );
    final version = Uint8List.fromList(bytes)..fillRange(4, 8, 0);
    expect(
      () => decodePositions(version),
      throwsA(
        isA<FormatException>().having(
          (e) => e.message,
          'message',
          contains('layout version 0'),
        ),
      ),
    );
    expect(
      () => decodeIntlRender(bytes),
      throwsA(
        isA<FormatException>().having(
          (e) => e.message,
          'message',
          contains('blob schema 1, expected 2'),
        ),
      ),
    );
    expect(
      () => decodePositions(bytes.sublist(0, bytes.length - 8)),
      throwsA(
        isA<FormatException>().having(
          (e) => e.message,
          'message',
          contains('the header says'),
        ),
      ),
    );
  });

  test('a decoder reads a blob that does not start on an eight-byte '
      'boundary', () {
    final bytes = _read('positions.tsrb');
    final shifted = Uint8List(bytes.length + 1)
      ..setRange(1, bytes.length + 1, bytes);
    final view = Uint8List.sublistView(shifted, 1);
    expect(view.offsetInBytes % 8, 1);
    expect(
      decodePositions(view).jdCount,
      2,
      reason: 'the odd offset is copied once, not misread',
    );
  });

  test('the catalogue tables are the keys every pack and fixture carries', () {
    expect(Graha.sun.fullKey, 'graha.SUN');
    expect(Graha.ketu.fullKey, 'graha.KETU');
    expect(
      Kind.nakshatra.key,
      'nakshatra',
      reason: "a kind names itself as a key's first segment does",
    );
    expect(Kind.avasthaBaladi.key, 'avastha_baladi');
    expect(Status.invalidArg.key, 'invalid-arg');
    expect(Body.meanNode.key, 'mean-node');
    expect(TimeScale.ut1.key, 'ut1');
    expect(Graha.byKey('graha.SUN'), Graha.sun);
    expect(
      Graha.values.map((g) => g.key).toSet().length,
      Graha.values.length,
      reason: 'no two members share a key',
    );
  });

  test('the constants come from the boundary, not from a literal', () {
    expect(generatedAbiVersion, 1);
    expect(contextTestProvider, 1);
    expect(vtableAbiVersion, 2);
  });
}
