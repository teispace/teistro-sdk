// Bundles the Teimeris adapter into the app being built: the slice for the
// ABI or SDK this invocation targets, held to the digest this package
// carries, through the SDK's own bundler. See `lib/src/release.dart`.

import 'package:hooks/hooks.dart';
import 'package:teistro_ephemeris_teimeris_flutter/src/release.dart';
import 'package:teistro_flutter/hook.dart';

Future<void> main(List<String> arguments) => build(
  arguments,
  (input, output) => bundlePrebuilt(input, output, release: adapterRelease),
);
