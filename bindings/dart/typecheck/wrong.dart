// The usages that must not compile, each with the error it must raise.
//
// `cargo xtask check-dart` analyses this file on its own and fails unless
// every expectation below is reported; the package excludes it, because
// a file whose whole purpose is to be wrong would fail the clean pass.

import 'package:teistro/teistro.dart';

void main() {
  // expect: The argument type 'Longitude' can't be assigned to the parameter type 'Latitude'
  Observer(
    latitudeDeg: Longitude(85.324),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );
  // expect: The argument type 'double' can't be assigned to the parameter type 'Latitude'
  Observer(
    latitudeDeg: 27.7172,
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );
  // expect: The argument type 'Altitude' can't be assigned to the parameter type 'Longitude'
  localMeanZone(Altitude(1400));
  // A layout is a key of its own kind: a registered one is made by that
  // kind, and no other kind's member stands in for it.
  // expect: The element type '(Graha, Varga)' can't be assigned to the list type '(KeyOf<ChartLayout>, Varga)'
  final List<(KeyOf<ChartLayout>, Varga)> drawings = [(Graha.sun, Varga.d1)];
  // expect: The element type '(Registered<Graha>, Varga)' can't be assigned to the list type '(KeyOf<ChartLayout>, Varga)'
  final List<(KeyOf<ChartLayout>, Varga)> registered = [
    (Graha.registered('ACME'), Varga.d1),
  ];
  print('$drawings $registered');
}
