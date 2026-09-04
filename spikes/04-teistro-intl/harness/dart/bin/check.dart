// Runtime check of the generated accessors: every call reaches the
// renderer with the key and the parameters it was given, and nothing else.
import 'package:teistro_spike_intl_harness/sdk.dart';

final class Recorder implements Renderer {
  final calls = <String>[];

  @override
  String render(String key, [Map<String, Object?> params = const {}]) {
    calls.add('$key $params');
    return key;
  }

  @override
  EntityForms entity(String key) {
    calls.add('entity $key');
    return EntityForms(short: key, name: key, prose: key, iast: key);
  }
}

void expect(Object? actual, Object? expected) {
  if (actual != expected) {
    throw StateError('expected $expected, got $actual');
  }
}

void main() {
  final r = Recorder();
  final t = Messages(r);
  expect(t.sdk.reason.grahaInBhava(graha: GrahaKey.jupiter, bhava: 7),
      'sdk.reason.grahaInBhava');
  t.sdk.reason.greeting(gender: Gender.f, name: 'Sita');
  t.sdk.reason.occupants(
      grahas: [GrahaKey.sun.key, GrahaKey.moon.key], rashi: RashiKey.leo);
  t.sdk.reason.strength.rank(rank: 3);
  t.sdk.reason.appName();
  expect(t.sdk.entity.graha.sun.name, 'graha.SUN');
  expect(r.calls.join('\n'), [
    'sdk.reason.grahaInBhava {bhava: 7, graha: graha.JUPITER}',
    'sdk.reason.greeting {gender: f, name: Sita}',
    'sdk.reason.occupants {grahas: [graha.SUN, graha.MOON], rashi: rashi.LEO}',
    'sdk.reason.strength.rank {rank: 3}',
    'sdk.reason.appName {}',
    'entity graha.SUN',
  ].join('\n'));
  print(r.calls.length);
}
