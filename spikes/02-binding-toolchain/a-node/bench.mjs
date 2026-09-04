/**
 * Spike 2, option A: the Node measurements. The callback cost is derived:
 * (chart with a JavaScript provider − chart with the native test provider)
 * / 9 calls. Results go to `../results/a-node.json`.
 */

import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { bench, derived, report, writeResults } from '../harness/bench.mjs';
import { Chart, chartNodeCount, createContext } from './lib/index.js';

const HERE = dirname(fileURLToPath(import.meta.url));
const JD = 2_460_000.5;

/** A JavaScript provider as cheap as the native one, so the boundary is what is measured. */
function jsProvider(jdUt, body) {
  return { longitudeDeg: (jdUt % 360) + body * 10, latitudeDeg: 0.5, speedDegPerDay: 1 };
}

const depth3 = createContext({ dashaDepth: 3 });
const depth3Host = createContext({ dashaDepth: 3 }, jsProvider);
const depth4 = createContext({ dashaDepth: 4 });
const depth5 = createContext({ dashaDepth: 5 });

const chart3 = depth3.computeChart(JD);
const chart4 = depth4.computeChart(JD);
const chart5 = depth5.computeChart(JD);
if (chart3.nodeCount !== chartNodeCount(3) || chart5.nodeCount !== 66_429) {
  throw new Error('node counts disagree with the library');
}
if (depth3Host.computeChart(JD).positions()[0].longitudeDeg === chart3.positions()[0].longitudeDeg) {
  throw new Error('the host provider was not used');
}

const native3 = bench('chart, depth 3 (819 nodes), native provider: compute + blob', () => depth3.computeChart(JD));
const host3 = bench('chart, depth 3, JavaScript provider: 9 callbacks + compute + blob', () => depth3Host.computeChart(JD));
const rows = [
  native3,
  host3,
  derived('one provider callback into JavaScript (derived)', host3, native3, 9),
  bench('decode columns, depth 3 (zero-copy views)', () => new Chart(chart3.bytes).decoded),
  bench('positions as 9 objects', () => chart3.positions()),
  bench('eager tree, depth 3 (819 objects)', () => chart3.dashaTree()),
  bench('one lazy row', () => chart3.dashaRow(400), { iterations: 20_000 }),
  bench('chart, depth 4 (7 380 nodes), native provider', () => depth4.computeChart(JD), { iterations: 500 }),
  bench('eager tree, depth 4 (7 380 objects)', () => chart4.dashaTree(), { iterations: 500 }),
  bench('chart, depth 5 (66 429 nodes), native provider', () => depth5.computeChart(JD), { iterations: 100 }),
  bench('decode columns, depth 5', () => new Chart(chart5.bytes).decoded, { iterations: 100 }),
  bench('eager tree, depth 5 (66 429 objects)', () => chart5.dashaTree(), { iterations: 50 }),
];
const blobBytes = { depth3: chart3.bytes.byteLength, depth4: chart4.bytes.byteLength, depth5: chart5.bytes.byteLength };

report(rows);
console.log(`\nblob bytes: depth 3 ${blobBytes.depth3}, depth 4 ${blobBytes.depth4}, depth 5 ${blobBytes.depth5}`);
writeResults(join(HERE, '..', 'results'), 'a-node.json', { option: 'A', binding: 'node', blob_bytes: blobBytes, rows });
