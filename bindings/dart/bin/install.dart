/// Installs the shared library this package's version was built with.
///
/// ```sh
/// dart run teistro:install                 # from the release
/// dart run teistro:install --from lib.gz   # from a file, for a machine
///                                          # with no network
/// ```
///
/// The library is checked against a digest recorded when it was built
/// before it is written, and written under `.dart_tool/teistro/<version>/`,
/// which is where `Teistro.open` looks first. Nothing else is touched.
library;

import 'dart:io';

import 'package:teistro/src/install.dart';
import 'package:teistro/src/prebuilt.dart';

Future<void> main(List<String> arguments) async {
  if (arguments.contains('--help') || arguments.contains('-h')) {
    stdout.writeln(_usage);
    return;
  }
  String? from;
  for (var i = 0; i < arguments.length; i++) {
    final argument = arguments[i];
    if (argument == '--from') {
      if (i + 1 >= arguments.length) {
        stderr.writeln('--from needs a path\n\n$_usage');
        exitCode = 2;
        return;
      }
      from = arguments[++i];
    } else if (argument.startsWith('--from=')) {
      from = argument.substring('--from='.length);
    } else {
      stderr.writeln('unknown argument `$argument`\n\n$_usage');
      exitCode = 2;
      return;
    }
  }

  try {
    final installed = await install(from: from);
    final size = (installed.bytes / (1024 * 1024)).toStringAsFixed(1);
    stdout.writeln(
      installed.fetched
          ? 'Teistro $prebuiltVersion for $hostPlatform: '
              '${installed.path} ($size MB, checked)'
          : 'Teistro $prebuiltVersion for $hostPlatform is already at '
              '${installed.path}',
    );
  } on InstallException catch (error) {
    stderr.writeln(error);
    exitCode = 1;
  }
}

const String _usage = '''
Installs the Teistro shared library for this machine.

  dart run teistro:install              fetch it from the release this
                                        package's version was cut from
  dart run teistro:install --from PATH  install from an archive or a
                                        library you already have

The library is written to .dart_tool/teistro/<version>/ and checked
against the digest recorded when it was built. Set \$TEISTRO_LIBRARY to
load one from somewhere else instead.''';
