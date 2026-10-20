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
  bool withoutBuiltin = false,
}) => Release(
  stem: 'teistro_ffi',
  version: '9.9.9',
  base: 'https://example.invalid/releases/download',
  digests: digests ?? {'android-arm64': sha256Hex(_library)},
  fetch: fetch,
  withoutBuiltin: withoutBuiltin,
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
    // Every name is one the release table can hold, and the names a
    // `library` map may be keyed by are these six.
    expect(mobilePlatforms, [
      'android-arm64',
      'android-arm',
      'android-x64',
      'ios-arm64',
      'ios-sim-arm64',
      'ios-sim-x64',
    ]);
    for (final name in mobilePlatforms) {
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

  group('the slice without the built-in', () {
    late Uri pubspec;
    setUp(() => pubspec = scratch.uri.resolve('app/pubspec.yaml'));

    /// The user defines an Android arm64 build reads, with [defines]
    /// declared in the pubspec at [pubspec].
    Future<HookInputUserDefines> read(Map<String, Object?> defines) async {
      late HookInputUserDefines read;
      await testCodeBuildHook(
        targetOS: OS.android,
        targetArchitecture: Architecture.arm64,
        userDefines: PackageUserDefines(
          workspacePubspec: PackageUserDefinesSource(
            defines: defines,
            basePath: pubspec,
          ),
        ),
        mainMethod: (arguments) => build(arguments, (input, _) async {
          read = input.userDefines;
        }),
        check: (_, _) {},
      );
      return read;
    }

    test('is asked for by `false` alone', () async {
      final sdk = _release(withoutBuiltin: true);
      expect(omitsBuiltin(await read({}), sdk), isFalse);
      expect(omitsBuiltin(await read({builtinDefine: true}), sdk), isFalse);
      expect(omitsBuiltin(await read({builtinDefine: false}), sdk), isTrue);
    });

    test('is refused as anything but a boolean', () async {
      final defines = await read({builtinDefine: 'false'});
      expect(
        () => omitsBuiltin(defines, _release(withoutBuiltin: true)),
        _refused('it takes `true` or `false`'),
      );
    });

    test('is refused on a release with no built-in to leave out', () async {
      final defines = await read({builtinDefine: false});
      expect(
        () => omitsBuiltin(defines, _release()),
        _refused('Write it under `hooks: user_defines: teistro_flutter:`'),
      );
      expect(Release.teistro.withoutBuiltin, isTrue);
    });

    test("is the release's slice by its own name, digest and cache", () async {
      final archives = Directory('${scratch.path}/archives')..createSync();
      File(
        '${archives.path}/libteistro_ffi-9.9.9-android-arm64-$withoutBuiltinSuffix.so.gz',
      ).writeAsBytesSync(gzip.encode(_library));
      final release = _release(
        digests: {'android-arm64-$withoutBuiltinSuffix': sha256Hex(_library)},
        withoutBuiltin: true,
      );
      await testCodeBuildHook(
        targetOS: OS.android,
        targetArchitecture: Architecture.arm64,
        userDefines: PackageUserDefines(
          workspacePubspec: PackageUserDefinesSource(
            defines: {builtinDefine: false, prebuiltDirDefine: '../archives'},
            basePath: pubspec,
          ),
        ),
        mainMethod: (arguments) => build(
          arguments,
          (input, output) => bundlePrebuilt(input, output, release: release),
        ),
        check: (_, output) {
          final file = output.assets.code.single.file!;
          expect(file.pathSegments.last, 'libteistro_ffi.so');
          expect(
            file.path,
            contains('/9.9.9/android-arm64-$withoutBuiltinSuffix/'),
          );
          expect(File.fromUri(file).readAsBytesSync(), _library);
        },
      );
    });
  });

  group('a library the app names', () {
    late Uri pubspec;
    setUp(() => pubspec = scratch.uri.resolve('app/pubspec.yaml'));

    /// The user defines a build of an iPhone simulator slice would read, with
    /// [library] declared in the pubspec at [pubspec].
    Future<HookInputUserDefines> defines(Object library) async {
      late HookInputUserDefines read;
      await testCodeBuildHook(
        targetOS: OS.iOS,
        targetArchitecture: Architecture.arm64,
        targetIOSSdk: IOSSdk.iPhoneSimulator,
        userDefines: PackageUserDefines(
          workspacePubspec: PackageUserDefinesSource(
            defines: {libraryDefine: library},
            basePath: pubspec,
          ),
        ),
        mainMethod: (arguments) => build(arguments, (input, _) async {
          read = input.userDefines;
        }),
        check: (_, _) {},
      );
      return read;
    }

    test('names nothing when the define is absent', () async {
      late HookInputUserDefines read;
      await testCodeBuildHook(
        targetOS: OS.iOS,
        targetArchitecture: Architecture.arm64,
        mainMethod: (arguments) => build(arguments, (input, _) async {
          read = input.userDefines;
        }),
        check: (_, _) {},
      );
      expect(namedLibrary(read, 'ios-arm64'), isNull);
    });

    test('as one path, names it for every platform, as before', () async {
      final relative = await defines('../lib/libteistro_ffi.dylib');
      final expected = scratch.uri.resolve('lib/libteistro_ffi.dylib');
      for (final platform in mobilePlatforms) {
        expect(namedLibrary(relative, platform), expected, reason: platform);
      }
      // The same path `HookInputUserDefines.path` has always given.
      expect(relative.path(libraryDefine), expected);
      final absolute = await defines('/opt/teistro/libteistro_ffi.dylib');
      expect(
        namedLibrary(absolute, 'ios-sim-x64'),
        Uri.file('/opt/teistro/libteistro_ffi.dylib'),
      );
    });

    test('as a map, names each platform its own path', () async {
      final read = await defines({
        'ios-sim-arm64': '../aarch64-apple-ios-sim/libteistro_ffi.dylib',
        'ios-sim-x64': '/opt/x86_64-apple-ios/libteistro_ffi.dylib',
      });
      // A relative path is read from the pubspec's directory, as one path is.
      expect(
        namedLibrary(read, 'ios-sim-arm64'),
        scratch.uri.resolve('aarch64-apple-ios-sim/libteistro_ffi.dylib'),
      );
      expect(
        namedLibrary(read, 'ios-sim-x64'),
        Uri.file('/opt/x86_64-apple-ios/libteistro_ffi.dylib'),
      );
    });

    test('as a map without the platform built, is refused with the line '
        'to add', () async {
      final read = await defines({
        'ios-sim-arm64': 'a.dylib',
        'android-arm64': 'b.so',
      });
      expect(
        () => namedLibrary(read, 'ios-sim-x64'),
        throwsA(
          isA<BuildError>().having(
            (e) => e.message,
            'message',
            allOf(
              contains('this build is for ios-sim-x64'),
              contains('android-arm64, ios-sim-arm64 only'),
              contains(
                'hooks: user_defines: teistro_flutter: library: '
                'ios-sim-x64: <path>',
              ),
            ),
          ),
        ),
      );
      final empty = await defines(<String, Object?>{});
      expect(
        () => namedLibrary(empty, 'ios-arm64'),
        _refused('names a library for no platform'),
      );
    });

    test('as a map with a key no platform has, is refused', () async {
      final read = await defines({'ios-sim-arm': 'a.dylib'});
      expect(
        () => namedLibrary(read, 'ios-sim-arm64'),
        _refused('the key(s) ios-sim-arm, and its keys are'),
      );
    });

    test('as anything else, is refused', () async {
      final number = await defines(3);
      expect(
        () => namedLibrary(number, 'ios-arm64'),
        _refused('it takes a path, or a map of release platform to path'),
      );
      final read = await defines({'ios-arm64': 3});
      expect(
        () => namedLibrary(read, 'ios-arm64'),
        _refused('names int `3` for ios-arm64'),
      );
    });

    test('that is not there is refused', () {
      final missing = scratch.uri.resolve('nothing.dylib');
      expect(
        () => refuseNamed(missing, 'ios-sim-x64'),
        _refused(
          'names ${missing.toFilePath()} for ios-sim-x64, and there is '
          'no file there',
        ),
      );
    });

    test('built for another processor is refused, naming the map', () {
      /// The first bytes of a library: an ELF or thin Mach-O header.
      Uri library(String name, List<int> head) {
        final file = File('${scratch.path}/$name')
          ..writeAsBytesSync([...head, ...List<int>.filled(64, 0)]);
        return file.uri;
      }

      final elf = [0x7f, 0x45, 0x4c, 0x46, 2, 1, ...List<int>.filled(12, 0)];
      final so = library('arm64.so', [...elf, 183, 0]);
      final arm = library('arm.so', [...elf, 40, 0]);
      final x64 = library('x64.so', [...elf, 62, 0]);
      final dylib = library('arm64.dylib', [
        0xcf, 0xfa, 0xed, 0xfe, 0x0c, 0, 0, 0x01, //
      ]);
      final simX64 = library('x64.dylib', [
        0xcf, 0xfa, 0xed, 0xfe, 0x07, 0, 0, 0x01, //
      ]);
      expect(processorOf(File.fromUri(so)), 'arm64');
      expect(processorOf(File.fromUri(arm)), 'arm');
      expect(processorOf(File.fromUri(x64)), 'x64');
      expect(processorOf(File.fromUri(dylib)), 'arm64');
      expect(processorOf(File.fromUri(simX64)), 'x64');
      for (final (file, platform) in [
        (so, 'android-arm64'),
        (arm, 'android-arm'),
        (x64, 'android-x64'),
        (dylib, 'ios-arm64'),
        (dylib, 'ios-sim-arm64'),
        (simX64, 'ios-sim-x64'),
      ]) {
        expect(() => refuseNamed(file, platform), returnsNormally);
      }
      expect(
        () => refuseNamed(dylib, 'ios-sim-x64'),
        _refused('an arm64 library, for ios-sim-x64. A build for several'),
      );
      expect(
        () => refuseNamed(so, 'android-x64'),
        _refused('library: {android-x64: <path>, ...}'),
      );
      // A header not read here is the loader's to judge.
      final other = File('${scratch.path}/other')..writeAsBytesSync(_library);
      expect(processorOf(other), isNull);
      expect(() => refuseNamed(other.uri, 'ios-sim-x64'), returnsNormally);
    });

    test('is bundled for the platform the map names it for', () async {
      final file = File('${scratch.path}/x64/libteistro_ffi.dylib')
        ..createSync(recursive: true)
        ..writeAsBytesSync(_library);
      await testCodeBuildHook(
        targetOS: OS.iOS,
        targetArchitecture: Architecture.x64,
        targetIOSSdk: IOSSdk.iPhoneSimulator,
        userDefines: PackageUserDefines(
          workspacePubspec: PackageUserDefinesSource(
            defines: {
              libraryDefine: {
                'ios-sim-arm64': '../arm64/libteistro_ffi.dylib',
                'ios-sim-x64': '../x64/libteistro_ffi.dylib',
              },
            },
            basePath: pubspec,
          ),
        ),
        mainMethod: (arguments) => build(
          arguments,
          (input, output) => bundlePrebuilt(input, output, release: _release()),
        ),
        check: (_, output) {
          expect(output.assets.code.single.file, file.uri);
          expect(output.dependencies, contains(file.uri));
        },
      );
    });
  });
}
