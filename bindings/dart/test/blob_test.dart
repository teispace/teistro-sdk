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
    for (final step in steps) {
      expect(step.name, isNotEmpty);
      expect(Implementation.values, contains(step.implementation));
    }
    final provenance = positions.provenance;
    expect(provenance.profile, 'nepali-default');
    expect(provenance.calculationVersion, 1);
    expect(provenance.settingsHash.length, 64);
    expect(
      provenance.provider.frame,
      'GEOCENTRIC/OF_DATE/ECLIPTIC/TROPICAL/APPARENT',
    );
    expect(provenance.time.deltaTModel, 'TABLE_THEN_MODEL');
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
    expect(Status.invalidArg.key, 'INVALID_ARG');
    expect(Body.meanNode.key, 'MEAN_NODE');
    expect(TimeScale.ut1.key, 'UT1');
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
    expect(vtableAbiVersion, 4);
  });

  test(
    'a corrupted blob of every schema is decoded or refused, never anything else',
    () {
      for (final (name, decode) in _schemas) {
        var decoded = 0;
        var refused = 0;
        for (final (i, kind, bytes) in _mutants(_read(name))) {
          try {
            decode(bytes);
            decoded++;
          } on FormatException {
            refused++;
          } on Object catch (error) {
            fail('$name, mutant $i (mutation $kind): $error');
          }
        }
        // Both ways out were taken, so the mutants reach past the header.
        expect(
          decoded > 0 && refused > 0,
          isTrue,
          reason: '$name: $decoded, $refused',
        );
      }
    },
  );
}

// Every blob of every schema, corrupted the same way in every binding's
// test: a seeded xorshift32 stream picks, two thousand times a blob, one of
// four mutations: a few bytes flipped past the header, one aligned word set
// to an edge value (0, 1, 3, 4, 8, 2^31 - 1, 2^31, 2^32 - 1 or the blob's
// length), the blob cut short, or the blob grown with zeros, the last two
// with the header's length mended so the table of contents is read. Each
// mutant is decoded or refused as a [FormatException]; anything else thrown
// is a fault in the decoder.
const _mutantCount = 2000;
const _edges = [0, 1, 3, 4, 8, 0x7fffffff, 0x80000000, 0xffffffff, -1];
final _schemas = <(String, Object Function(Uint8List))>[
  ('positions.tsrb', decodePositions),
  ('intl_render.tsrb', decodeIntlRender),
  ('charts.tsrb', decodeCharts),
  ('panchanga.tsrb', decodePanchanga),
  ('digest.tsrb', decodeDigest),
  ('naam.tsrb', decodeNaam),
];

Iterable<(int, int, Uint8List)> _mutants(Uint8List good) sync* {
  var x = 0x9e3779b9;
  int next() {
    x ^= (x << 13) & 0xffffffff;
    x ^= x >> 17;
    x ^= (x << 5) & 0xffffffff;
    return x;
  }

  for (var i = 0; i < _mutantCount; i++) {
    var bytes = Uint8List.fromList(good);
    final kind = next() % 4;
    if (kind == 0) {
      for (var n = 1 + next() % 4; n > 0; n--) {
        bytes[20 + next() % (bytes.length - 20)] ^= 1 + next() % 255;
      }
    } else if (kind == 1) {
      final at = (next() % (bytes.length >> 2)) * 4;
      final edge = _edges[next() % _edges.length];
      // The magic, the version and the schema stay, or nothing past them
      // is read.
      if (at < 8 || (at >= 12 && at < 20)) continue;
      ByteData.sublistView(
        bytes,
      ).setUint32(at, edge == -1 ? bytes.length : edge, Endian.little);
    } else {
      bytes =
          kind == 2
              ? Uint8List.sublistView(
                bytes,
                0,
                32 + next() % (bytes.length - 32),
              )
              : (Uint8List(bytes.length + 8 * (1 + next() % 8))
                ..setAll(0, bytes));
      bytes = Uint8List.fromList(bytes);
      ByteData.sublistView(bytes).setUint32(12, bytes.length, Endian.little);
    }
    yield (i, kind, bytes);
  }
}
