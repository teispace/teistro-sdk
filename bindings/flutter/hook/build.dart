// Bundles the SDK's native library into the app being built: the slice
// for the ABI or SDK this invocation targets, fetched from the release (or
// a directory the app names) and checked against the digest this package
// carries before anything is written. See `lib/src/bundle.dart`.

import 'package:hooks/hooks.dart';
import 'package:teistro_flutter/hook.dart';

Future<void> main(List<String> arguments) => build(arguments, bundlePrebuilt);
