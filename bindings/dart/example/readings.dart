// A rule's own reading, in the reader's language — loaded, not embedded.
//
// `interpretation.dart` composed a plan and said it in two languages. One
// thing it could not say in Nepali was **what a rule's verse states**: that
// crosses as the words the rule cites, in the language the rule was written
// in, and a Nepali reading shows the seam.
//
// This is how the seam closes. The SDK carries a corpus of readings — one
// for each of 649 yogas and doshas, in Sanskrit, Nepali, English and Hindi
// — and it is **not compiled into the library**: it is several times the
// size of every message pack together, and a consumer computing a Julian
// day should not carry every Nepali yoga reading to do it. It is a pack
// that is loaded (`docs/03-design/interpretation-records.md`).
//
// What it teaches:
//
// 1. **A pack is bytes.** `intl.loadPack` takes them from wherever you got
//    them — a file beside your program, a download, an asset in your
//    application bundle. This example reads the files `teistro-intl build`
//    wrote from the SDK's own source root, as every binding's does.
// 2. **Loading changes what a composer says**, not how it is called.
//    `readings` asks the base locale whether it carries a reading of each
//    rule: with the pack, the item is the locale's own reading; without it,
//    the verse's cited words. The same code composes both.
// 3. **A plan item is a sentence.** The record also holds the full passage
//    and its named facets; those are read from the entity directly, which
//    is one call on an engine you already have.
//
//   cargo run -p teistro-intl -- --root packs/readings build --out target/packs/readings
//   dart run example/readings.dart

import 'dart:io';

import 'package:teistro/teistro.dart';

/// Where the built packs are: `TEISTRO_PACKS`, or the repository's
/// `target/packs`.
String packs() =>
    Platform.environment['TEISTRO_PACKS'] ??
    File.fromUri(
      Platform.script,
    ).parent.parent.parent.parent.uri.resolve('target/packs').toFilePath();

/// Every pack a corpus built, one a locale, in name order.
List<File> packsOf(String corpus) {
  final found =
      Directory('${packs()}/$corpus')
          .listSync()
          .whereType<File>()
          .where((file) => file.path.endsWith('.tpack'))
          .toList()
        ..sort((a, b) => a.path.compareTo(b.path));
  return found;
}

/// Enough of a passage to show it is there, without printing an essay; in
/// characters, not UTF-16 units, so a Devanagari passage is cut where every
/// binding cuts it.
String firstSentence(String text) =>
    text.runes.length > 96
        ? '${String.fromCharCodes(text.runes.take(96))}…'
        : text;

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );

  // ── The pack ─────────────────────────────────────────────────────────
  // One pack a locale, because a Nepali application wants Nepali and its
  // fallback and not four languages' worth of prose.
  for (final file in packsOf('readings')) {
    final bytes = file.readAsBytesSync();
    // This is the call a consumer makes, whatever the bytes came from.
    final loaded = ctx.intl.loadPack(bytes);
    print(
      'loaded ${loaded.locale.padRight(12)} '
      '${'${loaded.entries}'.padLeft(6)} readings, '
      '${'${bytes.length}'.padLeft(7)} bytes',
    );
  }

  // ── A chart, and the rules it holds ──────────────────────────────────
  // The computed yogas are the set whose rules the corpus wrote readings
  // for; the nabhasas are the kernel's own and have none, which is the gap
  // `interpret-measured.md` counts.
  final chart = ctx.chart.found(
    instant: 2447995.4895833335,
    place: Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    ),
    utcOffsetSeconds: 20700,
    rules: const RuleRequest(
      shipped: [ShippedRules.yogas, ShippedRules.doshas],
    ),
    interpret: const PlanRequest(readings: true),
  );
  final plan =
      (chart.plans!['readings']! as List<Object?>).cast<Map<String, Object?>>();

  // ── The plan, said twice ─────────────────────────────────────────────
  print('\n${plan.length} items');
  for (final locale in ['en-Latn', 'ne-Deva-NP']) {
    ctx.intl.locale = locale;
    print('\n$locale');
    for (final item in plan) {
      final said = ctx.intl.render(
        item['key']! as String,
        item['params']! as Map<String, Object?>,
      );
      // A fallback would mean this locale had no reading of its own, which
      // is exactly what an example must not hide.
      print('  ${said.text}${said.fallback ? '  (fallback)' : ''}');
    }
  }

  // ── The passage, which the plan does not carry ───────────────────────
  // A plan item is a sentence. The record holds the essay and its named
  // facets beside it, for a page that wants them.
  final says =
      plan.where((item) => item['key'] == 'sdk.reading.says').firstOrNull;
  final reading = (says?['params'] as Map<String, Object?>?)?['reading'];
  if (reading is Map<String, Object?> && reading[r'$entity'] is String) {
    final key = reading[r'$entity']! as String;
    final forms = ctx.intl.entity(key).forms;
    print('\n$key');
    for (final form in ['prose', 'career', 'mind', 'spirituality']) {
      final text = forms[form];
      if (text != null) print('  ${form.padRight(14)} ${firstSentence(text)}');
    }
  }

  ctx.dispose();
}
