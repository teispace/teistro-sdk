// A Flutter app on the SDK over Teimeris: `teistro_flutter` bundles the
// SDK's library and `teistro_ephemeris_teimeris_flutter` the adapter, its
// data built in. No file is copied out and no path is named.

import 'package:flutter/material.dart';
import 'package:teistro_flutter/teistro_flutter.dart';

import 'month.dart';

void main() => runApp(const TeimerisExample());

/// November 2025's almanac hashes, and whether the built-in is bundled.
class TeimerisExample extends StatelessWidget {
  /// Builds the app.
  const TeimerisExample({super.key});

  @override
  Widget build(BuildContext context) {
    final teistro = Teistro.open();
    final lines = [
      'Teistro ${teistro.version}',
      'built-in ${hasBuiltin(teistro) ? 'bundled' : 'absent'}',
      ...monthHashes(teistro),
    ];
    lines.forEach(debugPrint);
    return MaterialApp(
      home: Scaffold(
        body: Center(
          child: Text(lines.join('\n'), textAlign: TextAlign.center),
        ),
      ),
    );
  }
}
