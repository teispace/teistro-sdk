// An interpretation: the same birth record, said in two languages.
//
// `chart_reading.mjs` read the chart in full. This is what comes after: a
// **composer** turns what was read into a narrative plan — an ordered list
// of message keys and their slots — and the locale engine says it. The plan
// holds no words at all, which is why one plan says the same chart in
// English and in Nepali without the composer knowing either language
// (`docs/03-design/plans-at-the-boundary.md`).
//
// What it teaches:
//
// 1. **The plan comes back in the same crossing as the chart.** `interpret`
//    is an option on `found`, like `rules` and `vargas`; nothing is founded
//    or evaluated twice, and a chart with no `interpret` pays nothing.
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
// The record is `birth_chart.mjs`'s own, so the examples can be read side
// by side.

import { Calendar, Context, TeistroError, at, date, ianaZone } from '../lib/index.js';

const ctx = new Context({
  profile: 'nepali-default',
  locale: 'en-Latn',
  ephemeris: 'builtin',
});

const born = date(Calendar.BikramSambat, 2042, 9, 17);
const when = ctx.time.resolve(at(born, { hour: 0, minute: 20 }), ianaZone('Asia/Kathmandu'));

// ── The chart, and what it has to say, in one call ─────────────────────
const chart = ctx.chart.found({
  instant: when.instantJdUtc,
  place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
  utcOffsetSeconds: when.offsetSeconds,
  // The rules whose answers the readings composer will say. The sections
  // they read are computed whether or not they are asked for here.
  rules: { shipped: ['nabhasas', 'arishtas'] },
  interpret: { placements: true, readings: true, strength: true, houses: true },
});

const { placements, readings, strength, houses } = chart.plans;
console.log('BS 2042-09-17  00:20  Kathmandu');
console.log(
  `plan     ${placements.length} placement items, ${readings.length} reading items, ` +
    `${strength.length} strengths, ${houses.length} lordships`,
);
console.log(`keys     ${[...new Set(placements.concat(readings).map((item) => item.key))].join(', ')}`);

// ── The same plan, said twice ──────────────────────────────────────────
// Nothing between an item and the renderer: `item.params` is what
// `render` takes, so this loop is the whole consumer story.
for (const locale of ['en-Latn', 'ne-Deva-NP']) {
  ctx.intl.locale = locale;
  console.log(`\n${locale}`);
  for (const item of [...placements, ...strength, ...houses]) {
    const said = ctx.intl.render(item.key, item.params);
    console.log(`  ${said.text}${said.isFallback ? '  (fallback)' : ''}`);
  }
  // A reading names its rule in a slot the message does not print, so a
  // consumer can group a plan by rule. Here it prefixes the line.
  for (const item of readings) {
    const said = ctx.intl.render(item.key, item.params);
    console.log(`  ${item.params.rule}: ${said.text}${said.isFallback ? '  (fallback)' : ''}`);
  }
}

// ── What it does not say, and what it refuses ──────────────────────────
const lagna = placements.some((item) => JSON.stringify(item.params).includes('LAGNA'));
// The Shadbala says whether a graha reaches its required rupas; no locale
// says it, so the plan does not either.
const strong = strength.some((item) => JSON.stringify(item.params).includes('strong'));
console.log(`a strength item claims "strong": ${strong}`);
// A bhava knows its sign, its cusps and its class; no locale says any of
// them, so the houses plan says the lord and stops there.
const classed = houses.some((item) => JSON.stringify(item.params).includes('rashi'));
console.log(`a houses item claims a sign: ${classed}`);
console.log(`\nthe lagna is in the placements: ${lagna}`);

try {
  ctx.chart.found({
    instant: when.instantJdUtc,
    place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
    utcOffsetSeconds: when.offsetSeconds,
    interpret: { readings: true },
  });
} catch (error) {
  if (!(error instanceof TeistroError)) throw error;
  console.log(`refused  ${error.field}: ${error.message}`);
}

ctx.dispose();
