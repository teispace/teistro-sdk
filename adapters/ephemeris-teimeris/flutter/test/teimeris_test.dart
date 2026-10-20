import 'package:teistro_ephemeris_teimeris_flutter/src/release.dart';
import 'package:teistro_ephemeris_teimeris_flutter/teistro_ephemeris_teimeris_flutter.dart';
import 'package:teistro_flutter/hook.dart' show mobilePlatforms;
import 'package:test/test.dart';

void main() {
  test('teimeris() asks the adapter for the data built into it', () {
    final descriptor = teimeris(path: '/x/lib.so', profile: 'max');
    expect(descriptor.plugin, '/x/lib.so');
    expect(descriptor.config, {'data': 'embedded', 'profile': 'max'});
    expect(teimeris(path: '/x').config, {'data': 'embedded'});
  });

  test('teimeris() adds the ranges an app downloaded, by path', () {
    final descriptor = teimeris(
      path: '/x',
      files: ['/support/sepl_12.se1', '/support/semo_12.se1'],
      verify: false,
    );
    expect(descriptor.config, {
      'data': 'embedded',
      'files': ['/support/sepl_12.se1', '/support/semo_12.se1'],
      'verify': false,
    });
  });

  test('Android opens the adapter by its bare name', () {
    expect(
      bundledAdapter(operatingSystem: 'android'),
      'libteistro_ephemeris_teimeris.so',
    );
  });

  test('iOS opens the framework by its absolute path when it is there', () {
    const app = '/var/containers/Bundle/Application/X/Runner.app';
    const framework =
        'teistro_ephemeris_teimeris.framework/teistro_ephemeris_teimeris';
    expect(
      bundledAdapter(
        operatingSystem: 'ios',
        executable: '$app/Runner',
        exists: (path) => path == '$app/Frameworks/$framework',
      ),
      '$app/Frameworks/$framework',
    );
    expect(
      bundledAdapter(
        operatingSystem: 'ios',
        executable: '$app/Runner',
        exists: (_) => false,
      ),
      framework,
    );
  });

  test('the release is the adapter\'s, on the SDK\'s version and page', () {
    expect(adapterRelease.stem, 'teistro_ephemeris_teimeris');
    expect(
      adapterRelease.file('android-arm64'),
      'libteistro_ephemeris_teimeris.so',
    );
    expect(
      adapterRelease.file('ios-arm64'),
      'libteistro_ephemeris_teimeris.dylib',
    );
    for (final platform in adapterRelease.digests.keys) {
      expect(mobilePlatforms, contains(platform));
    }
  });
}
