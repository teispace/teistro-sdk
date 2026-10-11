// The smallest Flutter app on the SDK: one dependency, `teistro_flutter`,
// whose build hook bundled the library this opens. Nothing is installed
// and no path is named; `Teistro.open()` finds the bundled library.

import 'package:flutter/material.dart';
import 'package:teistro_flutter/teistro_flutter.dart';

void main() => runApp(const TeistroExample());

/// Today's Bikram Sambat date, from the bundled library.
class TeistroExample extends StatelessWidget {
  /// Builds the app.
  const TeistroExample({super.key});

  @override
  Widget build(BuildContext context) {
    final teistro = Teistro.open();
    final calendar = teistro.context(profile: 'nepali-default').calendar;
    final now = DateTime.now();
    final bs = calendar.convert(
      CalendarDate(
        calendar: Calendar.gregorian,
        year: now.year,
        eraYear: 0,
        month: now.month,
        day: now.day,
        resolution: Resolution.defined,
        computedMonth: 0,
        computedDay: 0,
      ),
      Calendar.bikramSambat,
    );
    return MaterialApp(
      home: Scaffold(
        body: Center(
          child: Text(
            'Teistro ${teistro.version}\n'
            'Today is ${bs.year}-${bs.month}-${bs.day} BS',
            textAlign: TextAlign.center,
          ),
        ),
      ),
    );
  }
}
