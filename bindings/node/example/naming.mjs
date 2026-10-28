// Naming a child (nāmakaraṇa): the syllable a birth pada names a child
// by, names checked against it, a Latin spelling read every way it can
// be, a family's own book of names searched, and the clauses a name in
// daily use is held to, said in English and in Nepali.
// `cargo xtask check-node` runs this file, and every binding's `naming`
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

import { Context } from '../lib/index.js';
import { messages } from '../lib/messages.js';

const yes = (flag) => (flag ? 'yes' : 'no');

const ctx = new Context({ locale: 'en-Latn' });
const say = messages({ render: (key, params) => ctx.intl.render(key, params).text, entity: (key) => ctx.intl.entity(key) });

// ── The birth's syllable ────────────────────────────────────────────────
// The Moon in Ardra's third pada. Its syllable is ङ, which no name begins
// with, so the Nepali patro glosses it with ग: a substitute, listed after
// the cell's own letter.
const birth = { nakshatra: 'ARDRA', pada: 3 };
const names = ctx.naming.of(birth);
const syllable = names.syllable;
console.log(`${syllable.nakshatra} ${birth.pada}: ${syllable.devanagari} ${syllable.iast}, deity ${names.deity}`);
console.log(`initials ${syllable.initials.map((initial) => `${initial.devanagari} ${initial.by}`).join(', ')}`);

// ── Names checked against it ────────────────────────────────────────────
// Each name is answered on its own: ऐ is no cell of the cakra, and its
// refusal refuses no other name.
const asked = ['गणेश', 'चूडामणि', 'ऐश्वर्य'];
ctx.naming.check(asked, birth).forEach((one, i) => {
  const c = one.check;
  console.log(
    c === null
      ? `${asked[i]}: refused, ${one.refused.message}`
      : `${asked[i]}: same star ${yes(c.sameStar)}, same pada ${yes(c.samePada)}, by substitute ${yes(c.bySubstitute)}, ${c.clauses.syllables} syllables`,
  );
});

// ── A Latin spelling, read every way ────────────────────────────────────
const spellings = ['Krishna', 'Shiva'];
ctx.naming.read(spellings, { name: { latin: 'CASUAL' } }).forEach((one, i) => {
  const cells = one.reading.readings.map(
    (reading) => `${reading.spelled} ${reading.syllable.nakshatra ?? 'Abhijit'} ${reading.syllable.quarter}`,
  );
  console.log(`${spellings[i]}: ambiguous ${yes(one.reading.ambiguous)}, ${cells.join(' | ')}`);
});

// ── A family's own book ─────────────────────────────────────────────────
// The SDK ships no names. A book is loaded once into the context and named
// after by its id; औषधि begins with औ, no cell, and is listed.
const loaded = ctx.naming.load({
  names: [
    { devanagari: 'गणेश', sex: 'MALE' },
    { devanagari: 'गीता', sex: 'FEMALE' },
    { devanagari: 'गोपाल' },
    { devanagari: 'औषधि' },
  ],
});
console.log(`a book of ${loaded.names} names, refused: ${loaded.refused.map((one) => `${one.index} ${one.error.field}`).join(', ')}`);
// A boy's names: the book's boys and the names it gives no sex.
const found = ctx.naming.suggest(loaded.book, birth, { sex: 'MALE' });
console.log(`suggested for a boy: ${found.names.map((one) => `${one.reading.spelled} ${one.by}`).join(', ')}`);

// ── Said in each locale ─────────────────────────────────────────────────
// The answer holds keys; `sdk.naming`'s messages say them.
const [ganesh] = ctx.naming.check(['गणेश'], birth);
for (const locale of ['en-Latn', 'ne-Deva-NP']) {
  ctx.intl.locale = locale;
  console.log(locale);
  console.log(`  ${say.sdk.naming.pada({ nakshatra: 'nakshatra.ARDRA', pada: birth.pada, syllable: syllable.devanagari })}`);
  const clauses = ganesh.check.clauses;
  console.log(`  गणेश: ${say.sdk.naming.syllables({ count: clauses.syllables })}, ${say.sdk.naming.counted({ counted: clauses.counted })}`);
  for (const row of clauses.rows) {
    const clause = say.sdk.naming.clause({ clause: row.clause });
    console.log(`    ${clause}: ${say.sdk.naming.holds({ holds: yes(row.holds) })} (${say.sdk.naming.source({ source: row.source })})`);
  }
  for (const one of found.names) {
    console.log(`  ${one.reading.spelled}: ${say.sdk.naming.by({ by: one.by })}`);
  }
}

ctx.dispose();
