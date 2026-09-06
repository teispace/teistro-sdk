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
// It asserts the four facts the C smoke test prints, so that a package
// that loads but answers differently fails here rather than in the field.

import 'dart:io';

import 'package:teistro/teistro.dart';

void expect(Object? found, Object? wanted, String what) {
  if (found != wanted) {
    stderr.writeln('FAIL  $what: expected $wanted, found $found');
    exitCode = 1;
  }
}

void main() {
  stdout.writeln('the library came from ${installedLibrary()}');
  final teistro = Teistro.open();
  stdout.writeln(
    'abi ${teistro.abi}, sdk ${teistro.version}, '
    '${teistro.build.target}, ${teistro.build.profile}',
  );
  expect(teistro.build.optimised, true, 'a published library is optimised');
  expect(teistro.build.sanitizer, '', 'a published library has no sanitizer');
  expect(teistro.version, prebuiltVersion, 'the library is this package\'s');

  final context = teistro.context(
    profile: 'nepali-default',
    locale: 'ne-Deva-NP',
    testProvider: true,
  );

  final bs = context.convert(
    CalendarDate(
      calendar: Calendar.gregorian,
      year: 2015,
      eraYear: 0,
      month: 4,
      day: 14,
      resolution: Resolution.defined,
      computedMonth: 0,
      computedDay: 0,
    ),
    Calendar.bikramSambat,
  );
  stdout.writeln('14 April 2015 is ${bs.year}-${bs.month}-${bs.day} BS');
  expect(
    '${bs.year}-${bs.month}-${bs.day}',
    '2072-1-1',
    'the Bikram Sambat date',
  );

  final rendered = context.render('sdk.reason.grahaInBhava', <String, Object?>{
    'graha': 'graha.JUPITER',
    'bhava': 7,
  });
  stdout.writeln('sdk.reason.grahaInBhava in ne-Deva-NP: ${rendered.text}');
  expect(rendered.text, 'गुरु ७औं भावमा', 'the rendered message');

  final positions = context.positions(
    instants: <double>[2451545.0],
    bodies: <Body>[Body.sun],
  );
  final sun = positions.at(0, 0);
  stdout.writeln(
    'the Sun at J2000 is at ${sun.longitude.toStringAsFixed(4)} degrees',
  );
  expect(sun.longitude.toStringAsFixed(4), '278.5768', 'the Sun');

  context.dispose();
  if (exitCode == 0) {
    stdout.writeln('the published Dart package answers as the library does');
  }
}
