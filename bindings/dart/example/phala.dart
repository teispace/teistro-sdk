// What the chart *is*, read aloud: the state readings, loaded beside the
// rule readings.
//
// `readings.dart` loaded a pack of readings for the yogas and doshas a chart
// **holds**. This one loads the other half of that corpus: a reading for
// what the chart *is* without holding anything — Jupiter in the first
// house, the lagna's sign, the tithi, the nakshatra the Moon stands in
// (`docs/03-design/state-readings.md`).
//
// What it teaches:
//
// 1. **Two packs, one engine.** Each root builds one pack a locale, and a
//    consumer loads the ones it wants. They are loaded in either order.
// 2. **A record gains forms; it does not lose them.** Both corpora
//    describe some of the same subjects, and so may yours: a pack carrying
//    one form adds that form and leaves the rest of the record standing.
//    `loaded.merged` counts the records that kept something.
// 3. **A composer says nothing it has no words for.** `phala` asks the
//    base locale for each subject and is silent where the answer is no, so
//    a chart composes exactly as it did before until a pack is loaded.
//
// The packs are the files `teistro-intl build` writes, one a locale, which
// `cargo xtask check-parity` builds before it runs any example:
//
//   cargo run -p teistro-intl -- --root packs/readings build --out target/packs/readings
//   cargo run -p teistro-intl -- --root packs/states build --out target/packs/states
//   dart run example/phala.dart

import 'dart:io';

import 'package:teistro/teistro.dart';

/// The two corpora, each built from its source under `packs/` into packs of
/// its own (`packs/README.md`).
const corpora = ['readings', 'states'];

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
String shortened(String text) =>
    text.runes.length > 88
        ? '${String.fromCharCodes(text.runes.take(88))}…'
        : text;

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );

  // ── The packs ────────────────────────────────────────────────────────
  for (final corpus in corpora) {
    for (final file in packsOf(corpus)) {
      final bytes = file.readAsBytesSync();
      final loaded = ctx.intl.loadPack(bytes);
      print(
        '${'packs/$corpus'.padRight(15)} ${loaded.locale.padRight(12)} '
        '${'${loaded.entries}'.padLeft(5)} records, '
        '${'${loaded.merged}'.padLeft(4)} merged, '
        '${'${bytes.length}'.padLeft(7)} bytes',
      );
    }
  }

  // ── The plan, said twice ─────────────────────────────────────────────
  // A composer is a member of `PlanRequest` — `phala: true` — off unless
  // asked for, so a chart says nothing new until a consumer asks for it.
  // The chart is founded with what the composer reads, its states and the
  // panchanga's limbs among them, in the same call.
  final chart = ctx.chart.found(
    instant: 2447995.4895833335,
    place: Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    ),
    utcOffsetSeconds: 20700,
    interpret: const PlanRequest(phala: true),
  );
  final plan =
      (chart.plans!['phala']! as List<Object?>).cast<Map<String, Object?>>();
  print('\n${plan.length} items');
  for (final locale in ['en-Latn', 'ne-Deva-NP']) {
    ctx.intl.locale = locale;
    print('\n$locale');
    for (final item in plan.take(4)) {
      final said = ctx.intl.render(
        item['key']! as String,
        item['params']! as Map<String, Object?>,
      );
      print('  ${shortened(said.text)}${said.fallback ? '  (fallback)' : ''}');
    }
  }

  // ── The record that two corpora describe ─────────────────────────────
  // `nakshatra-phala` says what the nakshatra portends and
  // `namakarana-nakshatra` what to name a child born under it. Both are
  // forms on the record the SDK already names, beside its own `name` and
  // `iast` — which is what the merge on load is for. A pack's forms are
  // read through `forms`, since a record's forms are an open set.
  ctx.intl.locale = 'en-Latn';
  final ashwini = ctx.intl.entity('nakshatra.ASHWINI').forms;
  print('\nnakshatra.ASHWINI');
  for (final form in ['name', 'iast', 'phala', 'namakarana']) {
    final text = ashwini[form];
    if (text != null) print('  ${form.padRight(12)} ${shortened(text)}');
  }

  // ── A reading no composer says ───────────────────────────────────────
  // Half the corpus is glossary rather than narrative: what it means for a
  // graha to be exalted is true of every exalted graha, so no composer says
  // it per chart. It is a record like any other, and any catalogue key you
  // can name you can ask for — which is how a consumer builds a legend
  // beside the plan.
  print('\ndignity.EXALTED');
  final exalted = ctx.intl.entity('dignity.EXALTED').forms;
  for (final form in ['name', 'phala']) {
    final text = exalted[form];
    if (text != null) print('  ${form.padRight(12)} ${shortened(text)}');
  }

  ctx.dispose();
}
