// An interpretation: the same birth record, said in two languages.
//
// `chart_reading.dart` read the chart in full. This is what comes after: a
// **composer** turns what was read into a narrative plan — an ordered list
// of message keys and their slots — and the locale engine says it. The plan
// holds no words at all, which is why one plan says the same chart in
// English and in Nepali without the composer knowing either language
// (`docs/03-design/plans-at-the-boundary.md`).
//
// What it teaches:
//
// 1. **The plan comes back in the same crossing as the chart.** `interpret`
//    is an argument to `found`, like `rules` and `vargas`; nothing is
//    founded or evaluated twice, and a chart with no `interpret` pays
//    nothing.
// 2. **An item's `params` are `intl.render`'s own params.** There is no
//    conversion step in this file, and there is none in the binding either:
//    that is the whole design.
// 3. **One plan, every locale.** The same plan is said twice below, and the
//    rendering says which locale answered — so you can prove the locale had
//    the message rather than quietly falling back to English.
// 4. **A reading says what the rules found.** `readings` needs `rules`
//    beside it, because it composes their answers rather than re-deriving
//    them; asking for it alone is refused, by name.
// 5. **A composer says what it can say.** The lagna stands in the chart and
//    is in no placement item: those messages read a graha, and the lagna is
//    a point. A composer that guessed would be worse than one that is quiet.
//
// The record is `birth_chart.dart`'s own, so the examples can be read side
// by side.

import 'dart:convert';

import 'package:teistro/teistro.dart';

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    locale: 'en-Latn',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );

  final born = Calendar.bikramSambat.date(2042, 9, 17);
  final when = ctx.time.resolve(
    born.at(hour: 0, minute: 20),
    ianaZone('Asia/Kathmandu'),
  );
  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );

  // ── The chart, and what it has to say, in one call ───────────────────
  final chart = ctx.chart.found(
    instant: when.instantJdUtc,
    place: place,
    utcOffsetSeconds: when.offsetSeconds,
    // The rules whose answers the readings composer will say. The sections
    // they read are computed whether or not they are asked for here.
    rules: const RuleRequest(
      shipped: [ShippedRules.nabhasas, ShippedRules.arishtas],
    ),
    interpret: const PlanRequest(placements: true, readings: true),
  );

  final plans = chart.plans!;
  final placements =
      (plans['placements']! as List<Object?>).cast<Map<String, Object?>>();
  final readings =
      (plans['readings']! as List<Object?>).cast<Map<String, Object?>>();

  print('BS 2042-09-17  00:20  Kathmandu');
  print(
    'plan     ${placements.length} placement items, '
    '${readings.length} reading items',
  );
  final keys = <String>{
    for (final item in [...placements, ...readings]) item['key']! as String,
  };
  print('keys     ${keys.join(', ')}');

  // ── The same plan, said twice ────────────────────────────────────────
  // Nothing between an item and the renderer: `item['params']` is what
  // `render` takes, so this loop is the whole consumer story.
  for (final locale in ['en-Latn', 'ne-Deva-NP']) {
    ctx.intl.locale = locale;
    print('\n$locale');
    for (final item in placements) {
      final said = ctx.intl.render(
        item['key']! as String,
        item['params']! as Map<String, Object?>,
      );
      print('  ${said.text}${said.isFallback == 1 ? '  (fallback)' : ''}');
    }
    // A reading names its rule in a slot the message does not print, so a
    // consumer can group a plan by rule.
    for (final item in readings) {
      final params = item['params']! as Map<String, Object?>;
      final said = ctx.intl.render(item['key']! as String, params);
      print('  ${params['rule']}: ${said.text}');
    }
  }

  // ── What it does not say, and what it refuses ────────────────────────
  final lagna = placements.any(
    (item) => jsonEncode(item['params']).contains('LAGNA'),
  );
  print('\nthe lagna is in the placements: $lagna');

  try {
    ctx.chart.found(
      instant: when.instantJdUtc,
      place: place,
      utcOffsetSeconds: when.offsetSeconds,
      interpret: const PlanRequest(readings: true),
    );
  } on TeistroException catch (error) {
    print('refused  ${error.field}: ${error.message}');
  }

  ctx.dispose();
}
