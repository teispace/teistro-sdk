// A chart reading: one call for the whole document a reader interprets.
//
// `birth_chart.mjs` placed the grahas. A reading is what comes after:
// the divisional charts, the houses, what each graha **is** rather than
// where it is, which grahas look at which, and the derived points. Each is
// a section a request asks for by name, and each is computed from the same
// founded chart in the same crossing — so asking for all of them costs one
// call, and asking for none of them costs nothing
// (`docs/03-design/chart-reading.md`).
//
// What it teaches:
//
// 1. **Sections are asked for.** `vargas`, `aspects`, `points`, `houses`
//    and `state` are off by default, so a birth chart does not pay for
//    twenty-one divisional charts it will not show.
// 2. **Vargottama is a comparison, not a flag**: a graha whose navamsha
//    sign is the sign it stands in. The layer gives both signs.
// 3. **A dignity and a house are different questions**, answered by
//    different sections: `states` says how a graha fares in its sign,
//    `bhavas` who rules a house.
// 4. **The drishti are ragged**: how many there are depends on where the
//    grahas stand, not on how many grahas there are.
// 5. **A drawing is geometry, not pixels**: each cell's outline in a unit
//    square, the sign and house it shows and the grahas in it, so any
//    renderer draws the same chart.
//
// The record is `birth_chart.mjs`'s own, so the two can be read side by
// side.

import { Calendar, ChartLayout, Context, Graha, Point, RashiById, Varga, at, date, ianaZone } from '../lib/index.js';

const ctx = new Context({
  profile: 'nepali-default',
  locale: 'ne-Deva-NP',
  ephemeris: 'builtin',
});
const name = (key) => ctx.intl.entity(key).name;
/** The last part of a key, for a member no locale names: `kendra`, `half`. */
const plain = (key) => String(key).split('.').pop().toLowerCase();

const born = date(Calendar.BikramSambat, 2042, 9, 17);
const when = ctx.time.resolve(at(born, { hour: 0, minute: 20 }), ianaZone('Asia/Kathmandu'));

// ── One call for every section ─────────────────────────────────────────
const chart = ctx.chart.found({
  instant: when.instantJdUtc,
  place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
  utcOffsetSeconds: when.offsetSeconds,
  vargas: [Varga.D9, Varga.D10],
  aspects: true,
  points: true,
  houses: true,
  state: true,
  drawings: [{ layout: ChartLayout.NorthIndian, varga: Varga.D9 }],
});
const [navamsha, dasamsha] = chart.vargas;

console.log('reading  BS 2042-09-17  00:20  Kathmandu');
console.log(
  `lagna    ${name(RashiById.get(Math.floor(chart.lagnaDeg / 30)))} ${(chart.lagnaDeg % 30).toFixed(4)}°` +
    `   D9 ${name(navamsha.lagna.sign)}   D10 ${name(dasamsha.lagna.sign)}`,
);

// ── What each graha is ─────────────────────────────────────────────────
console.log('');
console.log('graha        house  dignity          age            D9 sign      vargottama  combust');
console.log('─'.repeat(88));
chart.states.forEach((state, j) => {
  const inNavamsha = navamsha.grahas[j].at;
  console.log(
    `${name(state.graha).padEnd(12)} ${String(state.house).padStart(5)}  ` +
      `${name(state.dignity).padEnd(16)} ${name(state.age).padEnd(14)} ` +
      `${name(inNavamsha.sign).padEnd(12)} ${(inNavamsha.sign === inNavamsha.rashi ? 'yes' : 'no').padEnd(11)} ` +
      `${plain(state.combustion.burning)}`,
  );
});

// ── The houses ─────────────────────────────────────────────────────────
// The tenth house, by the houses service: the sign its middle falls in,
// that sign's lord, and which kind of house it is.
const tenth = chart.bhavas[9];
console.log('');
console.log(
  `bhava 10 ${name(tenth.sign)}, ruled by ${name(tenth.lord)} (${plain(tenth.quadrant)})`,
);

// ── Which grahas look at the Moon ──────────────────────────────────────
// `houses` counts inclusively from the looking graha's sign, so the
// seventh is the house opposite it.
const onMoon = chart.aspects.filter((drishti) => drishti.to === Graha.Moon);
console.log(`drishti  ${chart.aspects.length} under ${chart.batch.drishtiTable}; on the Moon:`);
for (const drishti of onMoon) {
  console.log(`         ${name(drishti.from).padEnd(12)} house ${String(drishti.houses).padStart(2)} from it  ${plain(drishti.strength)}`);
}

// ── The derived points ─────────────────────────────────────────────────
// Gulika is Saturn's portion of the day's arc, which is why a chart with
// no day to divide has none — the section is ragged for that reason.
const gulika = chart.points.find((found) => found.point === Point.Gulika);
console.log(
  `gulika   ${gulika ? `${name(gulika.sign)} ${(gulika.longitudeDeg % 30).toFixed(4)}°` : 'none'}` +
    `   ${chart.points.length} points`,
);

// ── The navamsha, drawn ────────────────────────────────────────────────
// A North Indian chart keeps its houses still and moves the signs, so the
// lagna is always the top diamond; the cell says which sign landed there.
const [drawn] = chart.drawings;
const risen = drawn.cells.find((cell) => cell.lagna);
console.log(
  `drawing  ${plain(drawn.layout)} ${plain(drawn.varga)}: ${drawn.cells.length} cells, ` +
    `lagna in house ${risen.house} (${name(risen.sign)}), grahas there: ${risen.bodies.length}`,
);
console.log(`settings hash  ${ctx.settingsHash.slice(0, 16)}…`);
ctx.dispose();

