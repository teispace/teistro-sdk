/// What the build hook does: name the release platform a build targets,
/// hold it to the library's floor, find the slice (a library the app
/// names, a directory of archives, the cache, or the release), check it
/// against the digest this package carries, and hand it to Flutter to
/// bundle.
///
/// Every refusal is a sentence naming what was asked and what to do, so a
/// failing `flutter build` says why without a stack trace.
library;

import 'dart:io';
import 'dart:typed_data';

import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';
import 'package:teistro/prebuilt.dart';

/// The oldest Android API the library is linked for: Flutter's default
/// `minSdk`. `xtask/src/mobile.rs`'s `ANDROID_API` is the same number, and
/// a test there holds the two together.
const int androidFloor = 24;

/// The oldest iOS the library is linked for, below Flutter's own 15 so
/// that every Flutter app accepts it. `xtask/src/mobile.rs`'s `IOS_FLOOR`
/// is the same number, held together by the same test.
const int iosFloor = 13;

/// The user define naming a directory of release archives, read in place
/// of the release: an offline or air-gapped build, and CI's.
const String prebuiltDirDefine = 'prebuilt_dir';

/// The user define naming a library outright, bundled without a digest
/// check: a contributor's own build. The handshake still runs at load.
///
/// One path names one file for every platform the hook is invoked for. A
/// build for several (`flutter build ios --simulator`, an APK for several
/// ABIs) names one per platform instead, as a map keyed by the names
/// [platformFor] gives (`ios-sim-arm64`, `ios-sim-x64`, ...), each to its
/// own path. See [namedLibrary].
const String libraryDefine = 'library';

/// Every release platform [platformFor] can name, the keys a per-platform
/// [libraryDefine] map may hold.
const List<String> mobilePlatforms = [
  'android-arm64',
  'android-arm',
  'android-x64',
  'ios-arm64',
  'ios-sim-arm64',
  'ios-sim-x64',
];

/// The release a hook bundles from: the library's name, the version its
/// digests were cut from, where the archives are, and the digests.
///
/// [Release.teistro] is the SDK's own; an engine adapter's hook names its
/// own library the same way and calls [bundlePrebuilt] with it.
final class Release {
  /// Names a release.
  const Release({
    required this.stem,
    required this.version,
    required this.base,
    required this.digests,
    this.fetch,
  });

  /// The SDK's library, from the release this package was cut from.
  static const Release teistro = Release(
    stem: 'teistro_ffi',
    version: prebuiltVersion,
    base: prebuiltBase,
    digests: prebuiltDigests,
  );

  /// The library's name without its prefix and suffix.
  final String stem;

  /// The release's version.
  final String version;

  /// The URL the release's archives are under.
  final String base;

  /// SHA-256 of each platform's library, by release platform name.
  final Map<String, String> digests;

  /// How an archive is fetched; the release over HTTP when null, through
  /// the environment's proxy.
  final Future<List<int>> Function(Uri uri)? fetch;

  /// The file a platform's library is, the same for every ABI and SDK of
  /// one operating system, as Flutter requires.
  String file(String platform) => libraryFileNameOf(platform, stem: stem);

  /// Where the release publishes a platform's archive.
  Uri archive(String platform) => archiveUri(
    base: base,
    version: version,
    file: file(platform),
    platform: platform,
  );
}

/// The release platform a build targets: `android-arm64`, `ios-sim-x64`.
///
/// Null for an operating system the hook bundles nothing for (a desktop
/// app or a Dart program, which find the library on the search path as
/// they always have). A mobile target the release does not ship (32-bit
/// x86 or RISC-V Android) is refused by name.
String? platformFor(OS os, Architecture architecture, {IOSSdk? sdk}) {
  if (os == OS.android) {
    final cpu = switch (architecture) {
      Architecture.arm64 => 'arm64',
      Architecture.arm => 'arm',
      Architecture.x64 => 'x64',
      _ => null,
    };
    if (cpu == null) {
      throw BuildError(
        message:
            'the SDK ships no Android library for $architecture. Build for '
            'the ABIs it ships: `flutter build apk '
            '--target-platform android-arm64,android-arm,android-x64`.',
      );
    }
    return 'android-$cpu';
  }
  if (os == OS.iOS) {
    final simulator = sdk == IOSSdk.iPhoneSimulator;
    return switch ((architecture, simulator)) {
      (Architecture.arm64, false) => 'ios-arm64',
      (Architecture.arm64, true) => 'ios-sim-arm64',
      (Architecture.x64, true) => 'ios-sim-x64',
      _ => throw BuildError(
        message:
            'the SDK ships no iOS library for $architecture on '
            '${simulator ? 'the simulator' : 'a device'}: it ships arm64 '
            'devices and arm64 and x64 simulators.',
      ),
    };
  }
  return null;
}

/// Refuses a build whose minimum the library does not run on: an Android
/// `minSdk` below [androidFloor], or an iOS deployment target below
/// [iosFloor]. A higher minimum is always accepted.
void refuseFloor(CodeConfig code) {
  if (code.targetOS == OS.android) {
    final api = code.android.targetNdkApi;
    if (api < androidFloor) {
      throw BuildError(
        message:
            'the app\'s minSdk is $api, and the SDK\'s library needs Android '
            'API $androidFloor or later. Raise `minSdk` in '
            'android/app/build.gradle(.kts) to $androidFloor.',
      );
    }
  } else if (code.targetOS == OS.iOS) {
    final version = code.iOS.targetVersion;
    if (version < iosFloor) {
      throw BuildError(
        message:
            'the app\'s iOS deployment target is $version, and the SDK\'s '
            'library needs iOS $iosFloor or later. Raise '
            '`IPHONEOS_DEPLOYMENT_TARGET` (and the Podfile\'s `platform :ios`) '
            'to $iosFloor.0.',
      );
    }
  }
}

/// The build hook: bundles [release]'s library for the platform this
/// invocation targets, or nothing for a target that is not a phone.
Future<void> bundlePrebuilt(
  BuildInput input,
  BuildOutputBuilder output, {
  Release release = Release.teistro,
}) async {
  if (!input.config.buildCodeAssets) return;
  final code = input.config.code;
  final platform = platformFor(
    code.targetOS,
    code.targetArchitecture,
    sdk: code.targetOS == OS.iOS ? code.iOS.targetSdk : null,
  );
  if (platform == null) return;
  refuseFloor(code);
  final named = namedLibrary(input.userDefines, platform);
  final Uri file;
  if (named != null) {
    refuseNamed(named, platform);
    stderr.writeln(
      'teistro_flutter: bundling ${named.toFilePath()} for $platform '
      'unverified, as `$libraryDefine` asks',
    );
    output.dependencies.add(named);
    file = named;
  } else {
    final directory = input.userDefines.path(prebuiltDirDefine);
    file = await slice(
      release,
      platform,
      cache: input.outputDirectoryShared,
      from: directory,
    );
    if (directory != null) {
      output.dependencies.add(_archiveIn(directory, release, platform));
    }
  }
  output.assets.code.add(
    CodeAsset(
      package: input.packageName,
      name: '${release.stem}.dart',
      linkMode: DynamicLoadingBundled(),
      file: file,
    ),
  );
}

/// The library the app names for [platform] under [libraryDefine], or
/// null when it names none.
///
/// A string is one path for every platform. A map names one per platform,
/// keyed by the names [platformFor] gives; a map that does not name
/// [platform] is refused with the line to add, since a build for several
/// architectures invokes the hook once for each. A relative path is read
/// from the directory of the pubspec that declares it, in both forms, as
/// [HookInputUserDefines.path] reads one.
Uri? namedLibrary(HookInputUserDefines defines, String platform) {
  final value = defines[libraryDefine];
  if (value == null) return null;
  if (value is String) return defines.path(libraryDefine);
  if (value is! Map) {
    throw BuildError(
      message:
          'the `$libraryDefine` user define is ${value.runtimeType} '
          '`$value`; it takes a path, or a map of release platform to path: '
          '`$libraryDefine: {$platform: <path>}`.',
    );
  }
  final unknown = [
    for (final key in value.keys)
      if (!mobilePlatforms.contains(key)) '$key',
  ];
  if (unknown.isNotEmpty) {
    throw BuildError(
      message:
          'the `$libraryDefine` user define has the key(s) '
          '${unknown.join(', ')}, and its keys are the platforms the SDK '
          'ships: ${mobilePlatforms.join(', ')}.',
    );
  }
  final path = value[platform];
  if (path == null) {
    final names = [for (final key in value.keys) '$key']..sort();
    throw BuildError(
      message:
          'this build is for $platform, and the `$libraryDefine` user define '
          'names a library for '
          '${names.isEmpty ? 'no platform' : '${names.join(', ')} only'}. '
          'Add one for '
          '$platform to the map in the app\'s pubspec: `hooks: user_defines: '
          'teistro_flutter: $libraryDefine: $platform: <path>`.',
    );
  }
  if (path is! String) {
    throw BuildError(
      message:
          'the `$libraryDefine` user define names ${path.runtimeType} '
          '`$path` for $platform; it takes a path.',
    );
  }
  if (File(path).isAbsolute) return Uri.file(path);
  return defines.baseUri([libraryDefine, platform])!.resolve(path);
}

/// Refuses a library named for [platform] that is not there, or that is
/// built for another processor than [platform]'s.
///
/// The processor is read from the file's ELF or Mach-O header; a file whose
/// header says neither (a universal Mach-O, or a format not read here) is
/// let through, and the handshake still runs at load. An iOS device and an
/// arm64 simulator library share a processor, so that mix-up is the
/// loader's to report.
void refuseNamed(Uri named, String platform) {
  final file = File.fromUri(named);
  if (!file.existsSync()) {
    throw BuildError(
      message:
          'the `$libraryDefine` user define names ${named.toFilePath()} for '
          '$platform, and there is no file there.',
    );
  }
  final built = processorOf(file);
  final wanted = platform.substring(platform.lastIndexOf('-') + 1);
  if (built != null && built != wanted) {
    throw BuildError(
      message:
          'the `$libraryDefine` user define names ${named.toFilePath()}, an '
          '$built library, for $platform. A build for several architectures '
          'invokes the hook once for each, so name one library per platform: '
          '`$libraryDefine: {$platform: <path>, ...}`, keyed by '
          '${mobilePlatforms.join(', ')}.',
    );
  }
}

/// The processor a library is built for, in the release's spelling
/// (`arm64`, `arm`, `x64`), from its ELF or thin Mach-O header; null for
/// any other file.
String? processorOf(File file) {
  final RandomAccessFile handle;
  try {
    handle = file.openSync();
  } on FileSystemException {
    return null;
  }
  final Uint8List head;
  try {
    head = handle.readSync(20);
  } finally {
    handle.closeSync();
  }
  if (head.length < 20) return null;
  final bytes = ByteData.sublistView(head);
  if (bytes.getUint32(0) == 0x7f454c46) {
    final order = head[5] == 2 ? Endian.big : Endian.little;
    return switch (bytes.getUint16(18, order)) {
      183 => 'arm64',
      40 => 'arm',
      62 => 'x64',
      _ => null,
    };
  }
  if (bytes.getUint32(0, Endian.little) == 0xfeedfacf) {
    return switch (bytes.getUint32(4, Endian.little)) {
      0x0100000c => 'arm64',
      0x01000007 => 'x64',
      _ => null,
    };
  }
  return null;
}

/// The verified library for [platform], in [cache] under the release's
/// version, and the file's location.
///
/// A cached library that hashes right is used as it is. Otherwise the
/// archive is read from [from] when a directory is named, or fetched from
/// the release, then unpacked and checked before it is written, and
/// written under a temporary name and renamed, so that the hook's
/// invocations for other ABIs, running at once, never see half a file.
Future<Uri> slice(
  Release release,
  String platform, {
  required Uri cache,
  Uri? from,
}) async {
  final expected = release.digests[platform];
  if (expected == null) {
    final why = release.digests.isEmpty
        ? 'this build of the package carries no prebuilt libraries '
              '(version ${release.version} is not a release).'
        : 'release ${release.version} has no library for $platform; it '
              'ships ${(release.digests.keys.toList()..sort()).join(', ')}.';
    throw BuildError(
      message:
          '$why Name a library built for $platform in the app\'s pubspec: '
          '`hooks: user_defines: teistro_flutter: $libraryDefine: <path>`.',
    );
  }
  final directory = Directory.fromUri(
    cache.resolve('${release.version}/$platform/'),
  );
  final target = File.fromUri(directory.uri.resolve(release.file(platform)));
  if (target.existsSync() && sha256Hex(target.readAsBytesSync()) == expected) {
    return target.uri;
  }
  final List<int> archive;
  if (from != null) {
    final local = File.fromUri(_archiveIn(from, release, platform));
    if (!local.existsSync()) {
      throw BuildError(
        message:
            'the `$prebuiltDirDefine` user define names '
            '${from.toFilePath()}, and it holds no '
            '${_archiveName(release, platform)}.',
      );
    }
    archive = local.readAsBytesSync();
  } else {
    try {
      archive = await (release.fetch ?? _download)(release.archive(platform));
    } on InstallException catch (error) {
      throw InfraError(message: '$error');
    }
  }
  final Uint8List library;
  try {
    library = verified(archive, platform: platform, digests: release.digests);
  } on InstallException catch (error) {
    throw BuildError(message: '$error');
  }
  directory.createSync(recursive: true);
  final partial = File('${target.path}.$pid.partial');
  partial.writeAsBytesSync(library, flush: true);
  partial.renameSync(target.path);
  return target.uri;
}

Future<List<int>> _download(Uri uri) => download(
  uri,
  offline:
      'build offline from the release\'s archives in a directory: '
      '`hooks: user_defines: teistro_flutter: $prebuiltDirDefine: <dir>`',
);

String _archiveName(Release release, String platform) =>
    release.archive(platform).pathSegments.last;

Uri _archiveIn(Uri directory, Release release, String platform) {
  final base = directory.path.endsWith('/')
      ? directory
      : directory.replace(path: '${directory.path}/');
  return base.resolve(_archiveName(release, platform));
}
