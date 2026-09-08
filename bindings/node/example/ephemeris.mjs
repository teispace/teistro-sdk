// A year of the sky in one call, and what to do with the columns.
//
// The boundary takes a **grid** — instants by bodies — and answers with a
// result blob whose sections are columns. That shape is the whole reason
// a year of positions costs one crossing rather than 365, and it is what
// a service computing tables, transits or ingresses should be using.
//
// Three things this example is really about:
//
// 1. **One call, not a loop.** 366 instants by 3 bodies is 1098 cells in
//    a single request. Asking day by day would cross the boundary 366
//    times and recompute the provider's own setup each time.
// 2. **The columns are typed arrays, not objects.** `positions.cells.lon`
//    is a `Float64Array` over the blob's own bytes, so a table of a
//    million cells costs one allocation rather than a million objects.
// 3. **What the answer says about itself.** Every result carries the
//    steps applied and a provenance envelope with the settings hash —
//    the two things a cache key and an audit trail are made of.
//
// Honest about the provider: the SDK's analytic test provider is a smooth
// model, so nothing in it ever turns retrograde except the lunar node,
// which always is. The scan below therefore looks for **sign ingresses**,
// which do occur, and shows where a retrograde scan would go.

import {
  Ayanamsha,
  Body,
  Context,
  Graha,
  RashiById,
  buildInfo,
  canonicalFrame,
} from '../lib/index.js';

/** A year from the start of 2025, one sample a day at noon UTC. */
const START_JD = 2460676.5;
const DAYS = 366;

// A body is what an ephemeris answers; a graha is what a chart names.
// They are different catalogues and the lunar node is where they part —
// `MeanNode` is the body, `Rahu` the graha — so the two are paired
// explicitly rather than derived from each other's spelling.
const BODIES = [
  [Body.Sun, Graha.Sun],
  [Body.Mars, Graha.Mars],
  [Body.MeanNode, Graha.Rahu],
];

/**
 * Every day on which a body changed sign.
 *
 * Reads one body's column out of the grid. Cells run instants outermost,
 * so body `column` at day `i` is cell `i * stride + column`.
 */
function ingresses(longitudes, dayCount, stride, column) {
  const found = [];
  let previous = null;
  for (let day = 0; day < dayCount; day++) {
    const sign = RashiById.get(Math.floor(longitudes[day * stride + column] / 30));
    if (previous !== null && sign !== previous) found.push([day, sign]);
    previous = sign;
  }
  return found;
}

const ctx = new Context({
  profile: 'nepali-default',
  locale: 'ne-Deva-NP',
  testProvider: true,
});

// ── Which build am I talking to? ───────────────────────────────────────
// A service checks this once at start-up. The binding already refuses a
// library that is not the build it was generated from; this is how to log
// what it did load.
console.log(
  `library  Teistro ${buildInfo.sdk}  ABI ${buildInfo.abi}` +
    `  catalogue ${buildInfo.catalogue}  ${buildInfo.target}` +
    `  ${buildInfo.commit.slice(0, 8)}${buildInfo.dirty ? '-dirty' : ''}`,
);

// ── One call for the whole year ────────────────────────────────────────
const frame = { ...canonicalFrame(), sidereal: true, ayanamsha: Ayanamsha.Lahiri };
const sky = ctx.positions({
  instants: Array.from({ length: DAYS }, (_, day) => START_JD + day),
  bodies: BODIES.map(([body]) => body),
  frame,
});
console.log(
  `grid     ${sky.jdCount} instants x ${sky.bodyCount} bodies` +
    ` = ${sky.cells.length} cells in one call`,
);

// ── The columns ────────────────────────────────────────────────────────
const cells = sky.cells;
console.log(
  `columns  lon is a ${cells.lon.constructor.name}` +
    ` of ${cells.lon.length} values, ${cells.lon.byteLength} bytes`,
);

// ── What the columns are for ───────────────────────────────────────────
console.log('');
for (const [column, [body, graha]] of BODIES.entries()) {
  const name = ctx.entity(graha).name;
  const crossings = ingresses(cells.lon, DAYS, sky.bodyCount, column);
  const speed = cells.lonSpeed[column];
  const direction = speed < 0 ? 'retrograde' : 'direct';
  const key = body.split('.').at(-1);
  console.log(
    `  ${key.padEnd(10)} ${name.padEnd(8)} ${direction.padEnd(10)} at` +
      ` ${speed >= 0 ? '+' : ''}${speed.toFixed(4).padStart(7)}°/day,` +
      ` ${crossings.length} sign change(s)`,
  );
  for (const [day, sign] of crossings.slice(0, 3)) {
    console.log(
      `      day ${String(day).padStart(3)}  enters` +
        ` ${sign.split('.').at(-1).padEnd(12)} ${ctx.entity(sign).name}`,
    );
  }
  if (crossings.length > 3) console.log(`      … and ${crossings.length - 3} more`);
}

// The node is the only body here that ever moves backwards, and it always
// does. A real ephemeris would put Mars into retrograde for about ten
// weeks every two years, and the scan for it is the same shape as the one
// above, over `cells.lonSpeed` instead of `cells.lon`.

// ── What the answer says about itself ──────────────────────────────────
console.log('');
console.log(
  `steps    ${sky.steps.map((step) => `${step.name}:${step.implementation}`).join(', ')}`,
);
const provenance = sky.provenance;
console.log(`profile  ${provenance.profile}`);
console.log(`hash     ${provenance.settings_hash}`);
console.log(
  '         two contexts with the same settings hash compute the same' +
    ' numbers, so it is the cache key',
);
// The whole envelope is canonical JSON: byte-identical across every
// binding, which is what makes it safe to hash and store.
console.log(`envelope ${JSON.stringify(provenance).length} bytes of canonical JSON`);

ctx.dispose();
