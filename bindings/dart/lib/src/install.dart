/// Fetching the shared library this package's version was built with.
///
/// A pub package cannot carry a binary for every platform without making
/// every consumer download all of them, so this package carries none and
/// fetches the one it needs: `dart run teistro:install` downloads the
/// library from the release its version was cut from, checks it against a
/// digest recorded when it was built, and writes it where `Teistro.open`
/// looks. A machine with no network installs from a file it already has
/// (`--from`), and a machine that builds from source needs none of this —
/// `cargo build --release -p teistro-ffi` and the search path finds it.
///
/// Everything here is a plain function over a byte list so that it can be
/// tested without a network: [verified] is the check, [writeLibrary] is
/// the write, and [install] is the two with a download in front.
library;

import 'dart:ffi' show Abi;
import 'dart:io';
import 'dart:typed_data';

import 'prebuilt.dart';
import 'sha256.dart';

/// The file name this platform gives the SDK's shared library.
String get libraryFileName {
  if (Platform.isMacOS) return 'libteistro_ffi.dylib';
  if (Platform.isWindows) return 'teistro_ffi.dll';
  return 'libteistro_ffi.so';
}

/// This host as the release names it: `<os>-<cpu>`, in the words Node's
/// `process.platform` and `process.arch` use, so that one release page
/// names one artefact for both bindings.
String get hostPlatform {
  final os =
      Platform.isMacOS
          ? 'darwin'
          : Platform.isWindows
          ? 'win32'
          : Platform.operatingSystem;
  final cpu = _architecture;
  return '$os-$cpu';
}

/// The architecture, in npm's words. Dart names the host's ABI, which
/// carries the operating system as well, so only the tail is read.
String get _architecture {
  final abi = Abi.current().toString();
  if (abi.endsWith('arm64')) return 'arm64';
  if (abi.endsWith('x64')) return 'x64';
  if (abi.endsWith('ia32')) return 'ia32';
  if (abi.endsWith('arm')) return 'arm';
  if (abi.endsWith('riscv64')) return 'riscv64';
  return abi;
}

/// Where an installed library is kept: under the project's own tool
/// directory, beside what `dart pub get` writes, in a directory named for
/// the version it belongs to.
///
/// Naming the version means a project that changes the SDK's version
/// fetches again rather than loading the library of the version before,
/// which would be refused at load time but only after the download had
/// been skipped.
Directory installDirectory([Directory? project]) => Directory(
  '${(project ?? Directory.current).path}'
  '${Platform.pathSeparator}.dart_tool'
  '${Platform.pathSeparator}teistro'
  '${Platform.pathSeparator}$prebuiltVersion',
);

/// The installed library's path, whether or not it is there yet.
String installedLibrary([Directory? project]) =>
    '${installDirectory(project).path}${Platform.pathSeparator}$libraryFileName';

/// The archive this platform's library is published as, on the release
/// this package's version was cut from.
Uri? prebuiltUri([String? platform]) {
  final name = platform ?? hostPlatform;
  if (!prebuiltDigests.containsKey(name)) return null;
  final file = libraryFileName;
  final dot = file.lastIndexOf('.');
  final stem = dot < 0 ? file : file.substring(0, dot);
  final extension = dot < 0 ? 'so' : file.substring(dot + 1);
  return Uri.parse(
    '$prebuiltBase/v$prebuiltVersion/$stem-$prebuiltVersion-$name.$extension.gz',
  );
}

/// What went wrong, in a sentence a person can act on.
class InstallException implements Exception {
  /// Builds the failure with what to do about it.
  const InstallException(this.message, {this.hint});

  /// What happened.
  final String message;

  /// What to do about it, when there is something.
  final String? hint;

  @override
  String toString() => hint == null ? message : '$message\n$hint';
}

/// The bytes of the library, unpacked if they arrived packed, checked
/// against what the release recorded for this platform.
///
/// `digests` is the table to check against, which is the release's own
/// unless a test supplies one: the check is the part worth testing, and
/// the repository's table is empty until a release fills it.
///
/// Throws an [InstallException] when the platform has no recorded digest
/// or the bytes are not the ones that were built.
Uint8List verified(
  List<int> bytes, {
  String? platform,
  bool packed = true,
  Map<String, String>? digests,
}) {
  final table = digests ?? prebuiltDigests;
  final name = platform ?? hostPlatform;
  final expected = table[name];
  if (expected == null) {
    throw InstallException(
      table.isEmpty
          ? 'this build of the package carries no prebuilt libraries '
              '(version $prebuiltVersion is not a release)'
          : 'no prebuilt library for $name in version $prebuiltVersion',
      hint:
          'build it instead: `cargo build --release -p teistro-ffi`, then '
          'point \$TEISTRO_LIBRARY at target/release/$libraryFileName',
    );
  }
  final Uint8List library;
  try {
    library =
        packed
            ? Uint8List.fromList(gzip.decode(bytes))
            : Uint8List.fromList(bytes);
  } on FormatException catch (error) {
    throw InstallException(
      'the download is not the archive it should be: ${error.message}',
      hint: 'try again, or fetch it by hand from ${prebuiltUri(name)}',
    );
  }
  final found = sha256Hex(library);
  if (found != expected) {
    throw InstallException(
      'the library that arrived is not the one this package was built '
      'against.\n  expected $expected\n  found    $found',
      hint:
          'do not load it. Fetch it again, and report it if it happens '
          'twice: https://github.com/teispace/teistro-sdk/issues',
    );
  }
  return library;
}

/// Writes the library where `Teistro.open` looks, and returns its path.
String writeLibrary(Uint8List library, {Directory? project}) {
  final directory = installDirectory(project)..createSync(recursive: true);
  final path = '${directory.path}${Platform.pathSeparator}$libraryFileName';
  File(path).writeAsBytesSync(library, flush: true);
  return path;
}

/// What an install did, for the command to report.
class Installed {
  /// Records the outcome.
  const Installed(this.path, {required this.fetched, required this.bytes});

  /// Where the library is now.
  final String path;

  /// Whether it was fetched, rather than already being there.
  final bool fetched;

  /// How large it is.
  final int bytes;
}

/// Installs the library for this platform: from [from] when a file is
/// named, from the release otherwise.
///
/// A library that is already installed and hashes correctly is left where
/// it is; anything else is fetched and checked before it is written, so a
/// failed install never leaves a half-written library behind.
Future<Installed> install({String? from, Directory? project}) async {
  final path = installedLibrary(project);
  final existing = File(path);
  if (existing.existsSync() &&
      prebuiltDigests[hostPlatform] == sha256Hex(existing.readAsBytesSync())) {
    return Installed(path, fetched: false, bytes: existing.lengthSync());
  }
  final List<int> bytes;
  final bool packed;
  if (from != null) {
    final file = File(from);
    if (!file.existsSync()) {
      throw InstallException('no file at $from');
    }
    bytes = file.readAsBytesSync();
    packed = from.endsWith('.gz');
  } else {
    final uri = prebuiltUri();
    if (uri == null) {
      // `verified` says why, in the same words as every other refusal.
      verified(const <int>[], packed: false);
      throw StateError('unreachable: no library and no refusal');
    }
    bytes = await _download(uri);
    packed = true;
  }
  final library = verified(bytes, packed: packed);
  return Installed(
    writeLibrary(library, project: project),
    fetched: true,
    bytes: library.length,
  );
}

/// Downloads a file, following the redirect a release asset always has.
Future<List<int>> _download(Uri uri) async {
  final client = HttpClient();
  try {
    final request = await client.getUrl(uri);
    final response = await request.close();
    if (response.statusCode != HttpStatus.ok) {
      throw InstallException(
        'the release did not serve $uri (HTTP ${response.statusCode})',
        hint:
            response.statusCode == HttpStatus.notFound
                ? 'this version may not be released yet; build from source with '
                    '`cargo build --release -p teistro-ffi`'
                : 'try again in a moment',
      );
    }
    final bytes = <int>[];
    await for (final chunk in response) {
      bytes.addAll(chunk);
    }
    return bytes;
  } on SocketException catch (error) {
    throw InstallException(
      'cannot reach the release: ${error.message}',
      hint:
          'install from a file you already have with '
          '`dart run teistro:install --from <archive>`',
    );
  } finally {
    client.close();
  }
}
