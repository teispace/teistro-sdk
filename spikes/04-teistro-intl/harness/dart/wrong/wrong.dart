// Five mistakes a consumer could make; each must be an analysis error.
import 'package:teistro_spike_intl_harness/sdk.dart';

void wrong(Renderer r) {
  final t = Messages(r);
  t.sdk.reason.grahaInBhava(graha: 'graha.PLUTO', bhava: 7);
  t.sdk.reason.grahaInBhava(graha: GrahaKey.mars);
  t.sdk.reason.greeting(gender: 'x', name: 'S');
  t.sdk.reason.strength.rank(rank: 'third');
  t.sdk.reason.nothing();
}
