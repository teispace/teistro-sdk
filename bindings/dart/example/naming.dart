// Naming a child (nāmakaraṇa): the syllable a birth pada names a child
// by, names checked against it, a Latin spelling read every way it can
// be, a family's own book of names searched, and the clauses a name in
// daily use is held to, said in English and in Nepali.
// `cargo xtask check-dart` runs this file, and every binding's `naming`
// prints these lines (`03-design/namakarana.md`).
//
// What it is really teaching:
//
// 1. **Facts, not a score.** A check says whether a name begins in the
//    birth star's row, in the pada's own cell, or by the patro's
//    substitute letter, and what each clause of the gṛhyasūtras makes of
//    it. Nothing adds them up: which of them matters is the family's.
// 2. **A Latin spelling is read every way it can be.** `CASUAL` answers
//    each cell a spelling reaches and says it is ambiguous, rather than
//    choosing one.
// 3. **The SDK ships no names.** A book is the consumer's, loaded once
//    and named by its id; a name the cakra cannot read is listed, not
//    dropped.
// 4. **A clause is a key; the locale says it.** `sdk.naming`'s messages
//    turn each key into words, so one answer reads in every locale.
//
// It reads no sky, so it names no ephemeris.

import 'package:teistro/messages.dart' as intl;
import 'package:teistro/teistro.dart';

/// A flag as every binding prints it.
String yes(bool flag) => flag ? 'yes' : 'no';

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(locale: 'en-Latn');

  // ── The birth's syllable ───────────────────────────────────────────────
  // The Moon in Ardra's third pada. Its syllable is ङ, which no name
  // begins with, so the Nepali patro glosses it with ग: a substitute,
  // listed after the cell's own letter.
  const birth = Birth(nakshatra: Nakshatra.ardra, pada: 3);
  final names = ctx.naming.of(birth);
  final syllable = names.syllable;
  print(
    '${syllable.nakshatra?.fullKey ?? 'Abhijit'} ${birth.pada}: '
    '${syllable.devanagari} ${syllable.iast}, deity ${names.deity.fullKey}',
  );
  final initials = [
    for (final initial in syllable.initials)
      '${initial.devanagari} ${initial.by.key}',
  ];
  print('initials ${initials.join(', ')}');

  // ── Names checked against it ───────────────────────────────────────────
  // Each name is answered on its own: ऐ is no cell of the cakra, and its
  // refusal refuses no other name.
  const asked = ['गणेश', 'चूडामणि', 'ऐश्वर्य'];
  for (final (i, one) in ctx.naming.check(asked, birth).indexed) {
    final c = one.check;
    print(
      c == null
          ? '${asked[i]}: refused, ${one.refused?.message}'
          : '${asked[i]}: same star ${yes(c.sameStar)}, '
              'same pada ${yes(c.samePada)}, '
              'by substitute ${yes(c.bySubstitute)}, '
              '${c.clauses.syllables} syllables',
    );
  }

  // ── A Latin spelling, read every way ───────────────────────────────────
  const spellings = ['Krishna', 'Shiva'];
  final casual = ctx.naming.read(
    spellings,
    const NamingRules(name: NameRules(latin: LatinName.casual)),
  );
  for (final (i, one) in casual.indexed) {
    final reading = one.reading!;
    final cells = [
      for (final spelled in reading.readings)
        '${spelled.spelled} '
            '${spelled.syllable.nakshatra?.fullKey ?? 'Abhijit'} '
            '${spelled.syllable.quarter}',
    ];
    print(
      '${spellings[i]}: ambiguous ${yes(reading.ambiguous)}, '
      '${cells.join(' | ')}',
    );
  }

  // ── A family's own book ────────────────────────────────────────────────
  // The SDK ships no names. A book is loaded once into the context and
  // named after by its id; औषधि begins with औ, no cell, and is listed.
  final loaded = ctx.naming.load(
    const Book(
      names: [
        BookName(devanagari: 'गणेश', sex: Sex.male),
        BookName(devanagari: 'गीता', sex: Sex.female),
        BookName(devanagari: 'गोपाल'),
        BookName(devanagari: 'औषधि'),
      ],
    ),
  );
  final refused = [
    for (final one in loaded.refused) '${one.index} ${one.error.field}',
  ];
  print('a book of ${loaded.names} names, refused: ${refused.join(', ')}');
  // A boy's names: the book's boys and the names it gives no sex.
  final found = ctx.naming.suggest(
    loaded.book,
    birth,
    const SuggestFilter(sex: Sex.male),
  );
  final suggested = [
    for (final one in found.names) '${one.reading.spelled} ${one.by.key}',
  ];
  print('suggested for a boy: ${suggested.join(', ')}');

  // ── Said in each locale ────────────────────────────────────────────────
  // The answer holds keys; `sdk.naming`'s messages say them.
  final ganesh = ctx.naming.check(['गणेश'], birth).first.check!;
  final say = ctx.intl.messages.sdk.naming;
  for (final locale in ['en-Latn', 'ne-Deva-NP']) {
    ctx.intl.locale = locale;
    print(locale);
    print(
      '  ${say.pada(nakshatra: intl.NakshatraKey.ardra, pada: birth.pada, syllable: syllable.devanagari)}',
    );
    final clauses = ganesh.clauses;
    print(
      '  गणेश: ${say.syllables(count: clauses.syllables)}, '
      '${say.counted(counted: clauses.counted.key)}',
    );
    for (final row in clauses.rows) {
      print(
        '    ${say.clause(clause: row.clause.key)}: '
        '${say.holds(holds: yes(row.holds))} '
        '(${say.source(source: row.source.key)})',
      );
    }
    for (final one in found.names) {
      print('  ${one.reading.spelled}: ${say.by(by: one.by.key)}');
    }
  }

  ctx.dispose();
}
