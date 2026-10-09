// A two-group study, as `03-design/research.md` asks one to be run: the
// predicates named, the labels and the test fixed, and the input hash
// published before any data are read. `cargo xtask check-node` runs this
// file, and every binding's `research` prints these lines.

import { Context } from '../lib/index.js';

const ctx = new Context({ profile: 'nepali-default', ephemeris: 'BUILTIN' });

// Forty-eight births at Kathmandu, two months and an hour apart, and the
// first sixteen called the cases. The labels mean nothing, so a study that
// reads them honestly finds nothing: that is what the corrections are for.
const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
const births = Array.from({ length: 48 }, (_, i) => ({
  instant: 2444240.5 + 61.37 * i + (i % 24) / 24,
  place,
  utcOffsetSeconds: 20700,
}));
const rules = { shipped: ['YOGAS'] };
const design = { groups: births.map((_, i) => (i < 16 ? 1 : 0)) };

const table = ctx.research.counts({ births, rules, design });
console.log(`${table.rows.length} rules over ${births.length} births`);

const tested = ctx.research.compare({
  births,
  rules,
  design,
  test: { seed: 2026, permutations: 999, contrast: { kind: 'CASE_VS_REST', case: 1 }, alpha: 0.05 },
});
// The registration: the births, the rules, the labels and the test.
console.log(`registered as ${tested.provenance.inputHash}`);
console.log(`${tested.permutations} permutations, none can say less than p = ${tested.resolution.toFixed(4)}`);

const under = (method) => tested.rows.filter((row) => row.underAlpha[method]).length;
console.log(`under 0.05: ${under('raw')} raw, ${under('maxT')} after max-T, ${under('holm')} after Holm`);

// The smallest raw p, the first such row on a tie, and what the family
// makes of it.
const best = tested.rows.reduce((kept, row) => (row.p.value < kept.p.value ? row : kept));
const [rest, cases] = best.counts;
console.log(
  `${best.predicate}: ${cases.present} of ${cases.present + cases.absent} cases, ` +
    `${rest.present} of ${rest.present + rest.absent} others, ` +
    `p ${best.p.value.toFixed(3)}, max-T ${best.adjusted.maxT.toFixed(3)}`,
);
if (best.effect !== undefined) {
  const d = best.effect.riskDifference;
  console.log(`difference ${d.estimate.toFixed(3)} (${d.low.toFixed(3)} to ${d.high.toFixed(3)})`);
}

ctx.dispose();
