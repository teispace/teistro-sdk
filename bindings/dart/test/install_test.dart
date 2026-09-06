// The installer's parts, without a network: the digest it checks with,
// the names it builds, and the two ways a download is refused.
//
// The install itself is proved against a real release in
// `cargo xtask check-package`, which stages the packages, installs the
// library from the archive it just built and runs a consumer on it. What
// is here is everything that can be decided from bytes alone.

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:teistro/src/install.dart';
import 'package:teistro/src/prebuilt.dart';
import 'package:teistro/src/sha256.dart';
import 'package:test/test.dart';

/// The published vectors (FIPS 180-4), which any implementation must give.
const Map<String, String> _vectors = <String, String>{
  '': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
  'abc': 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
  'abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq':
      '248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1',
};

void main() {
  test('the digest is SHA-256', () {
    _vectors.forEach((message, digest) {
      expect(sha256Hex(utf8.encode(message)), digest, reason: message);
    });
    // A million a's, which is the vector that catches a wrong block loop.
    expect(
      sha256Hex(List<int>.filled(1000000, 0x61)),
      'cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0',
    );
    // Every length across a block boundary, against the digest of the
    // same bytes taken by the platform's own tool would need a process;
    // what is checked here is that no length throws or repeats.
    final seen = <String>{};
    for (var length = 0; length <= 130; length++) {
      final digest = sha256Hex(List<int>.generate(length, (i) => i & 0xff));
      expect(digest.length, 64);
      expect(seen.add(digest), isTrue, reason: 'length $length repeats');
    }
  });

  test('this host is named as the release names it', () {
    expect(hostPlatform, matches(RegExp(r'^[a-z0-9]+-[a-z0-9]+$')));
    expect(
      libraryFileName,
      anyOf('libteistro_ffi.so', 'libteistro_ffi.dylib', 'teistro_ffi.dll'),
    );
    expect(installedLibrary(), endsWith(libraryFileName));
    expect(installedLibrary(), contains('.dart_tool'));
    expect(installedLibrary(), contains(prebuiltVersion));
  });

  test('a library is accepted only when it is the one that was built', () {
    final library = Uint8List.fromList(utf8.encode('not really a library'));
    final digests = <String, String>{'test-cpu': sha256Hex(library)};

    expect(
      verified(gzip.encode(library), platform: 'test-cpu', digests: digests),
      library,
      reason: 'the packed bytes unpack to the library',
    );
    expect(
      verified(library, platform: 'test-cpu', packed: false, digests: digests),
      library,
      reason: 'an unpacked library is taken as it is',
    );

    final tampered = Uint8List.fromList(<int>[...library, 0x21]);
    expect(
      () => verified(
        gzip.encode(tampered),
        platform: 'test-cpu',
        digests: digests,
      ),
      throwsA(
        isA<InstallException>().having(
          (e) => e.toString(),
          'says what it expected and what came',
          allOf(
            contains('not the one this package was built'),
            contains('do not load it'),
          ),
        ),
      ),
    );
    expect(
      () => verified(library, platform: 'test-cpu', digests: digests),
      throwsA(isA<InstallException>()),
      reason: 'bytes that are not an archive are refused, not loaded',
    );
  });

  test('a platform with no prebuilt library is told how to build one', () {
    expect(
      () => verified(const <int>[], platform: 'sunos-sparc', digests: const {}),
      throwsA(
        isA<InstallException>().having(
          (e) => e.toString(),
          'names the build command',
          contains('cargo build --release -p teistro-ffi'),
        ),
      ),
    );
  });

  test('the repository itself carries no prebuilt libraries', () {
    // The table is written when a release is staged; a checkout has the
    // unreleased version and an empty table, and the installer says so
    // rather than fetching a release that does not exist.
    if (prebuiltVersion == '0.0.0') {
      expect(prebuiltDigests, isEmpty);
      expect(prebuiltUri(), isNull);
    } else {
      expect(prebuiltDigests, isNotEmpty);
      expect(
        prebuiltUri('linux-x64').toString(),
        'https://github.com/teispace/teistro-sdk/releases/download/'
        'v$prebuiltVersion/libteistro_ffi-$prebuiltVersion-linux-x64.so.gz',
        skip:
            Platform.isMacOS || Platform.isWindows
                ? 'the name is built from this host\'s library suffix'
                : null,
      );
    }
  });
}
