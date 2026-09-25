// A rule's own reading, in the reader's language — loaded, not embedded.
//
// `interpretation.mjs` composed a plan and said it in two languages. One
// thing it could not say in Nepali was **what a rule's verse states**: that
// crosses as the words the rule cites, in the language the rule was written
// in, and a Nepali reading shows the seam.
//
// This is how the seam closes. The SDK carries a corpus of readings — one
// for each of 649 yogas and doshas, in Sanskrit, Nepali, English and Hindi
// — and it is **not compiled into the library**: it is several times the
// size of every message pack together, and a consumer computing a Julian
// day should not carry every Nepali yoga reading to do it. It is a pack that
// is loaded (`docs/03-design/interpretation-records.md`).
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
//    and its named facets; those are read from the entity directly, which is
//    one call on an engine you already have.
//
//   cargo run -p teistro-intl -- --root packs/readings build --out target/packs/readings
//   node example/readings.mjs

import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Context } from '../lib/index.js';

// Where the built packs are: `TEISTRO_PACKS`, or the repository's
// `target/packs`.
const PACKS = process.env.TEISTRO_PACKS ?? fileURLToPath(new URL('../../../target/packs', import.meta.url));

// Every pack a corpus built, one a locale, in name order.
function packsOf(corpus) {
  const directory = join(PACKS, corpus);
  return readdirSync(directory)
    .filter((name) => name.endsWith('.tpack'))
    .sort()
    .map((name) => join(directory, name));
}

// Enough of a passage to show it is there, without printing an essay; in
// characters, not UTF-16 units, so a Devanagari passage is cut where every
// binding cuts it.
function firstSentence(text) {
  const characters = Array.from(text);
  return characters.length > 96 ? `${characters.slice(0, 96).join('')}…` : text;
}

const ctx = new Context({ profile: 'nepali-default', ephemeris: 'BUILTIN' });

// ── The pack ─────────────────────────────────────────────────────────────
// One pack a locale, because a Nepali application wants Nepali and its
// fallback and not four languages' worth of prose.
for (const file of packsOf('readings')) {
  const bytes = readFileSync(file);
  // This is the call a consumer makes, whatever the bytes came from.
  const loaded = ctx.intl.loadPack(bytes);
  console.log(
    `loaded ${loaded.locale.padEnd(12)} ${String(loaded.entries).padStart(6)} readings, ` +
      `${String(bytes.length).padStart(7)} bytes`,
  );
}

// ── A chart, and the rules it holds ──────────────────────────────────────
// The computed yogas are the set whose rules the corpus wrote readings for;
// the nabhasas are the kernel's own and have none, which is the gap
// `interpret-measured.md` counts.
const plan = ctx.chart.found({
  instant: 2447995.4895833335,
  place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
  utcOffsetSeconds: 20700,
  rules: { shipped: ['YOGAS', 'DOSHAS'] },
  interpret: { readings: true },
}).plans.readings;

// ── The plan, said twice ─────────────────────────────────────────────────
console.log(`\n${plan.length} items`);
for (const locale of ['en-Latn', 'ne-Deva-NP']) {
  ctx.intl.locale = locale;
  console.log(`\n${locale}`);
  for (const item of plan) {
    const said = ctx.intl.render(item.key, item.params);
    // A fallback would mean this locale had no reading of its own, which is
    // exactly what an example must not hide.
    console.log(`  ${said.text}${said.isFallback ? '  (fallback)' : ''}`);
  }
}

// ── The passage, which the plan does not carry ───────────────────────────
// A plan item is a sentence. The record holds the essay and its named facets
// beside it, for a page that wants them.
const says = plan.find((item) => item.key === 'sdk.reading.says');
const key = says?.params.reading?.$entity;
if (key !== undefined) {
  const forms = ctx.intl.entity(key).forms;
  console.log(`\n${key}`);
  for (const form of ['prose', 'career', 'mind', 'spirituality']) {
    if (forms[form] !== undefined) console.log(`  ${form.padEnd(14)} ${firstSentence(forms[form])}`);
  }
}

ctx.dispose();
