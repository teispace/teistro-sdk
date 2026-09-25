// What the chart *is*, read aloud: the state readings, loaded beside the
// rule readings.
//
// `readings.mjs` loaded a pack of readings for the yogas and doshas a chart
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
//   node example/phala.mjs

import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Context } from '../lib/index.js';

// The two corpora, each built from its source under `packs/` into packs of
// its own (`packs/README.md`).
const CORPORA = ['readings', 'states'];

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
function shortened(text) {
  const characters = Array.from(text);
  return characters.length > 88 ? `${characters.slice(0, 88).join('')}…` : text;
}

const ctx = new Context({ profile: 'nepali-default', ephemeris: 'BUILTIN' });

// ── The packs ────────────────────────────────────────────────────────────
for (const corpus of CORPORA) {
  for (const file of packsOf(corpus)) {
    const bytes = readFileSync(file);
    const loaded = ctx.intl.loadPack(bytes);
    console.log(
      `${`packs/${corpus}`.padEnd(15)} ${loaded.locale.padEnd(12)} ${String(loaded.entries).padStart(5)} records, ` +
        `${String(loaded.merged).padStart(4)} merged, ${String(bytes.length).padStart(7)} bytes`,
    );
  }
}

// ── The plan, said twice ─────────────────────────────────────────────────
// A composer is a member of `interpret` — `{ phala: true }` — off unless
// asked for, so a chart says nothing new until a consumer asks for it. The
// chart is founded with what the composer reads, its states and the
// panchanga's limbs among them, in the same call.
const plan = ctx.chart.found({
  instant: 2447995.4895833335,
  place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
  utcOffsetSeconds: 20700,
  interpret: { phala: true },
}).plans.phala;
console.log(`\n${plan.length} items`);
for (const locale of ['en-Latn', 'ne-Deva-NP']) {
  ctx.intl.locale = locale;
  console.log(`\n${locale}`);
  for (const item of plan.slice(0, 4)) {
    const said = ctx.intl.render(item.key, item.params);
    console.log(`  ${shortened(said.text)}${said.isFallback ? '  (fallback)' : ''}`);
  }
}

// ── The record that two corpora describe ─────────────────────────────────
// `nakshatra-phala` says what the nakshatra portends and
// `namakarana-nakshatra` what to name a child born under it. Both are forms
// on the record the SDK already names, beside its own `name` and `iast` —
// which is what the merge on load is for. A pack's forms are read through
// `forms`, since a record's forms are an open set.
ctx.intl.locale = 'en-Latn';
const ashwini = ctx.intl.entity('nakshatra.ASHWINI').forms;
console.log('\nnakshatra.ASHWINI');
for (const form of ['name', 'iast', 'phala', 'namakarana']) {
  if (ashwini[form] !== undefined) console.log(`  ${form.padEnd(12)} ${shortened(ashwini[form])}`);
}

// ── A reading no composer says ───────────────────────────────────────────
// Half the corpus is glossary rather than narrative: what it means for a
// graha to be exalted is true of every exalted graha, so no composer says it
// per chart. It is a record like any other, and any catalogue key you can
// name you can ask for — which is how a consumer builds a legend beside the
// plan.
console.log('\ndignity.EXALTED');
const exalted = ctx.intl.entity('dignity.EXALTED').forms;
for (const form of ['name', 'phala']) {
  if (exalted[form] !== undefined) console.log(`  ${form.padEnd(12)} ${shortened(exalted[form])}`);
}

ctx.dispose();
