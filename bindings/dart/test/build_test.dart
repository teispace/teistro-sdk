// The build handshake: the two halves of the binding must be one build,
// and the loader refuses one that is not.

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

BuildInfo info({
  String sdk = generatedSdkVersion,
  int abi = generatedAbiVersion,
  String sanitizer = '',
  bool optimised = true,
  String profile = 'release',
}) => BuildInfo(
  sdk: sdk,
  abi: abi,
  catalogue: 1,
  commit: 'a2a00beb59060011360f7c116d27d4d4fada69a1',
  dirty: false,
  profile: profile,
  target: 'aarch64-apple-darwin',
  debugAssertions: !optimised,
  optimised: optimised,
  sanitizer: sanitizer,
  rustc: 'rustc 1.98.0',
);

void main() {
  test('the open library describes the build it came from', () {
    final build = Teistro.open().build;
    expect(build.sdk, generatedSdkVersion);
    expect(build.abi, generatedAbiVersion);
    expect(build.catalogue, 1);
    expect(build.target, contains('-'));
    expect(build.rustc, startsWith('rustc'));
    expect(build.commit, matches(r'^([0-9a-f]{40}|unknown)$'));
    expect(build.toString(), contains('Teistro ${build.sdk}'));
  });

  test('a build that is the one these declarations came from is taken', () {
    expect(refuseBuild(info(), named: false), isNull);
    expect(refuseBuild(info(), named: true), isNull);
  });

  test('a build of another ABI or another version is refused', () {
    expect(
      refuseBuild(info(abi: 99), named: true),
      allOf(contains('ABI 99'), contains('generated for ABI 1')),
    );
    expect(
      refuseBuild(info(sdk: '9.9.9'), named: true),
      allOf(contains('Teistro 9.9.9'), contains(generatedSdkVersion)),
    );
  });

  test('a sanitizer build is refused however it was found', () {
    for (final named in [true, false]) {
      expect(
        refuseBuild(info(sanitizer: 'address'), named: named),
        contains('address sanitizer build'),
      );
    }
  });

  test('an unoptimised build is refused only when it was searched for', () {
    final debug = info(optimised: false, profile: 'debug');
    expect(
      refuseBuild(debug, named: false),
      allOf(contains('unoptimised'), contains(Teistro.pathVariable)),
      reason: 'a development build found by accident',
    );
    expect(
      refuseBuild(debug, named: true),
      isNull,
      reason: 'a path named is a deliberate act',
    );
  });

  test('a library that does not describe its build is refused', () {
    expect(() => BuildInfo.of('not json'), throwsStateError);
  });
}
