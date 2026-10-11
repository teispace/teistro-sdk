// A consumer of the published package, and nothing else.
//
// The binding's own tests are run from inside this repository against a
// library `TEISTRO_LIBRARY` names; they prove the code and not the
// package. This file is run from a throwaway project that depends on the
// staged package and installed the library with
// `dart run teistro:install`, so what it proves is the package: its name,
// its installer, the digest the installer checked, and the path
// `Teistro.open` finds without being told anything.
//
// It asserts the four facts the C smoke test prints (`facts.dart`, which
// the Flutter app's integration test asserts too), so that a package that
// loads but answers differently fails here rather than in the field.

import 'dart:io';

import 'package:teistro/teistro.dart';

import 'facts.dart';

void main() {
  stdout.writeln('the library came from ${installedLibrary()}');
  consumerFacts(Teistro.open(), (found, wanted, what) {
    if (found != wanted) {
      stderr.writeln('FAIL  $what: expected $wanted, found $found');
      exitCode = 1;
    }
  }, say: stdout.writeln);
  if (exitCode == 0) {
    stdout.writeln('the published Dart package answers as the library does');
  }
}
