// The hook's parts, without a network or a phone: which platform a build
// is, the floors, and where a slice comes from and when it is refused.
//
// What a real app bundles is proved by `cargo xtask check-flutter`, which
// builds an APK on the release's own slices and reads what is inside it.

import 'dart:io';

import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';
import 'package:teistro/prebuilt.dart';
import 'package:teistro_flutter/hook.dart';
import 'package:test/test.dart';

/// A stand-in library: the hook checks bytes, not what they mean.
final List<int> _library = List<int>.generate(4096, (i) => (i * 31) & 0xff);

Release _release({
  Map<String, String>? digests,
  Future<List<int>> Function(Uri uri)? fetch,
}) => Release(
  stem: 'teistro_ffi',
  version: '9.9.9',
  base: 'https://example.invalid/releases/download',
  digests: digests ?? {'android-arm64': sha256Hex(_library)},
  fetch: fetch,
);

Matcher _refused(String words) => throwsA(
  isA<HookError>().having((e) => e.message, 'message', contains(words)),
);

void main() {
  late Directory scratch;
  setUp(() => scratch = Directory.systemTemp.createTempSync('teistro-hook'));
  tearDown(() => scratch.deleteSync(recursive: true));

  test('a build is named as the release names its slice', () {
    expect(platformFor(OS.android, Architecture.arm64), 'android-arm64');
    expect(platformFor(OS.android, Architecture.arm), 'android-arm');
    expect(platformFor(OS.android, Architecture.x64), 'android-x64');
    expect(
      platformFor(OS.iOS, Architecture.arm64, sdk: IOSSdk.iPhoneOS),
      'ios-arm64',
    );
    expect(
      platformFor(OS.iOS, Architecture.arm64, sdk: IOSSdk.iPhoneSimulator),
      'ios-sim-arm64',
    );
    expect(
      platformFor(OS.iOS, Architecture.x64, sdk: IOSSdk.iPhoneSimulator),
      'ios-sim-x64',
    );
    // Every name is one the release table can hold.
    for (final name in [
      'android-arm64',
      'android-arm',
      'android-x64',
      'ios-arm64',
      'ios-sim-arm64',
      'ios-sim-x64',
    ]) {
      expect(
        archiveUri(base: 'b', version: '1', file: 'f.so', platform: name).path,
        endsWith('-$name.so.gz'),
      );
    }
  });

  test('a desktop or Dart build bundles nothing, as it always has', () {
    for (final os in [OS.macOS, OS.linux, OS.windows]) {
      expect(platformFor(os, Architecture.x64), isNull, reason: '$os');
    }
  });

  test('a mobile target the release does not ship is refused by name', () {
    expect(
      () => platformFor(OS.android, Architecture.riscv64),
      _refused('--target-platform android-arm64,android-arm,android-x64'),
    );
    expect(
      () => platformFor(OS.android, Architecture.ia32),
      _refused('no Android library'),
    );
    expect(
      () => platformFor(OS.iOS, Architecture.x64, sdk: IOSSdk.iPhoneOS),
      _refused('a device'),
    );
  });

  test('a slice is fetched, checked, written and then reused', () async {
    var fetched = 0;
    final release = _release(
      fetch: (uri) async {
        fetched++;
        expect(
          uri.toString(),
          'https://example.invalid/releases/download/v9.9.9/'
          'libteistro_ffi-9.9.9-android-arm64.so.gz',
        );
        return gzip.encode(_library);
      },
    );
    final first = await slice(release, 'android-arm64', cache: scratch.uri);
    expect(first.pathSegments.last, 'libteistro_ffi.so');
    expect(first.path, contains('/9.9.9/android-arm64/'));
    expect(File.fromUri(first).readAsBytesSync(), _library);
    final again = await slice(release, 'android-arm64', cache: scratch.uri);
    expect(again, first);
    expect(fetched, 1, reason: 'a cached slice that hashes right is reused');
    // No partial file is left beside it.
    expect(
      File.fromUri(first).parent.listSync().map((e) => e.uri.pathSegments.last),
      ['libteistro_ffi.so'],
    );
  });

  test('a cached slice that does not hash right is fetched again', () async {
    var fetched = 0;
    final release = _release(
      fetch: (_) async {
        fetched++;
        return gzip.encode(_library);
      },
    );
    final file = await slice(release, 'android-arm64', cache: scratch.uri);
    File.fromUri(file).writeAsBytesSync([1, 2, 3]);
    await slice(release, 'android-arm64', cache: scratch.uri);
    expect(fetched, 2);
    expect(File.fromUri(file).readAsBytesSync(), _library);
  });

  test('a download that is not the library built is never written', () async {
    final flipped = [..._library]..[100] ^= 1;
    final release = _release(fetch: (_) async => gzip.encode(flipped));
    await expectLater(
      slice(release, 'android-arm64', cache: scratch.uri),
      _refused('expected ${sha256Hex(_library)}'),
    );
    expect(
      Directory(scratch.path).listSync(recursive: true).whereType<File>(),
      isEmpty,
    );
  });

  test('an unreachable release is an infrastructure failure', () async {
    final release = _release(
      fetch: (_) async =>
          throw const InstallException('cannot reach the release: offline'),
    );
    await expectLater(
      slice(release, 'android-arm64', cache: scratch.uri),
      throwsA(isA<InfraError>()),
    );
  });

  test('a directory of archives is read in place of the release', () async {
    final archives = Directory('${scratch.path}/archives')..createSync();
    File(
      '${archives.path}/libteistro_ffi-9.9.9-android-arm64.so.gz',
    ).writeAsBytesSync(gzip.encode(_library));
    final release = _release(
      fetch: (_) async => fail('nothing is fetched when a directory is named'),
    );
    final file = await slice(
      release,
      'android-arm64',
      cache: Uri.directory('${scratch.path}/cache'),
      from: Uri.directory(archives.path),
    );
    expect(File.fromUri(file).readAsBytesSync(), _library);
    await expectLater(
      slice(
        _release(digests: {'android-x64': sha256Hex(_library)}),
        'android-x64',
        cache: Uri.directory('${scratch.path}/cache'),
        from: Uri.directory(archives.path),
      ),
      _refused('holds no libteistro_ffi-9.9.9-android-x64.so.gz'),
    );
  });

  test(
    'a platform the release has no digest for is refused with the cure',
    () async {
      await expectLater(
        slice(_release(), 'ios-arm64', cache: scratch.uri),
        _refused('it ships android-arm64'),
      );
      await expectLater(
        slice(_release(digests: const {}), 'ios-arm64', cache: scratch.uri),
        _refused('user_defines: teistro_flutter: library'),
      );
    },
  );

  test('the iOS file is one name for every SDK and architecture', () {
    final release = _release();
    expect(
      {
        for (final p in ['ios-arm64', 'ios-sim-arm64', 'ios-sim-x64'])
          release.file(p),
      },
      {'libteistro_ffi.dylib'},
    );
    expect(bundledLibraryName('ios'), 'teistro_ffi.framework/teistro_ffi');
  });

  test('the hook bundles nothing for a desktop build', () async {
    await testCodeBuildHook(
      targetOS: OS.macOS,
      targetArchitecture: Architecture.arm64,
      mainMethod: (arguments) => build(arguments, bundlePrebuilt),
      check: (input, output) => expect(output.assets.code, isEmpty),
    );
  });

  test('an app whose minimum is below the floor is refused', () async {
    // `build` exits the process on a refusal, so the config it was given
    // is held to the floor here, outside it.
    for (final (os, api, ios, words) in [
      (OS.android, androidFloor - 1, 17, 'Android API $androidFloor'),
      (OS.iOS, 30, iosFloor - 1, 'iOS $iosFloor or later'),
    ]) {
      late CodeConfig code;
      await testCodeBuildHook(
        targetOS: os,
        targetArchitecture: Architecture.arm64,
        targetAndroidNdkApi: api,
        targetIOSVersion: ios,
        mainMethod: (arguments) => build(arguments, (input, _) async {
          code = input.config.code;
        }),
        check: (_, _) {},
      );
      expect(() => refuseFloor(code), _refused(words), reason: '$os');
    }
    for (final (os, api, ios) in [
      (OS.android, androidFloor, 17),
      (OS.iOS, 30, iosFloor),
    ]) {
      late CodeConfig code;
      await testCodeBuildHook(
        targetOS: os,
        targetArchitecture: Architecture.arm64,
        targetAndroidNdkApi: api,
        targetIOSVersion: ios,
        mainMethod: (arguments) => build(arguments, (input, _) async {
          code = input.config.code;
        }),
        check: (_, _) {},
      );
      expect(() => refuseFloor(code), returnsNormally, reason: '$os');
    }
  });
}
