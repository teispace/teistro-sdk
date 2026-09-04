/**
 * Spike 2, option B: the JavaScript measurements over Diplomat's wasm
 * binding, the same rows as option A's Node benchmark where the backend
 * can express them. There is no provider row: Diplomat 0.16's JavaScript
 * backend refuses traits and callbacks ("Traits are not supported by this
 * backend"), so a host provider cannot be passed at all.
 *
 * Results go to `../results/b-js.json`.
 */

import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { bench, report, unavailable, writeResults } from '../harness/bench.mjs';
import { Ayanamsha, Context, Info, NodeKind, Settings } from './api/index.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const JD = 2_460_000.5;
const NOT_EXPRESSIBLE = 'not expressible: the backend refuses traits and callbacks';

function settings(depth) {
  return Settings.fromFields({ ayanamsha: Ayanamsha.Lahiri, node: NodeKind.Mean, dashaDepth: depth });
}

/** The nine positions as objects, one accessor call each. */
function positions(chart) {
  const n = chart.positionCount;
  const out = new Array(n);
  for (let i = 0; i < n; i += 1) out[i] = chart.position(i);
  return out;
}

/** The tree as nested objects, one accessor call per row. */
function tree(chart) {
  const n = chart.dashaRowCount;
  const nodes = new Array(n);
  const roots = [];
  for (let i = 0; i < n; i += 1) {
    const r = chart.dashaRow(i);
    const node = { lord: r.lord, level: r.level, startJd: r.startJd, endJd: r.endJd, children: [] };
    nodes[i] = node;
    if (r.parent < 0) roots.push(node);
    else nodes[r.parent].children.push(node);
  }
  return roots;
}

const depth3 = new Context(settings(3));
const depth4 = new Context(settings(4));
const depth5 = new Context(settings(5));
const chart3 = depth3.computeChart(JD);
const chart4 = depth4.computeChart(JD);
const chart5 = depth5.computeChart(JD);
if (chart3.dashaRowCount !== Info.nodeCountForDepth(3) || chart5.dashaRowCount !== 66_429) {
  throw new Error('node counts disagree with the library');
}
if (tree(chart3).length !== 9 || positions(chart3)[1].sign !== 0) {
  throw new Error('the accessors do not reproduce the chart');
}

const rows = [
  bench('chart, depth 3 (819 nodes), built-in provider: compute (opaque handle)', () => depth3.computeChart(JD)),
  unavailable('chart, depth 3, JavaScript provider', NOT_EXPRESSIBLE),
  unavailable('one provider callback into JavaScript', NOT_EXPRESSIBLE),
  bench('positions as 9 objects (9 accessor calls)', () => positions(chart3)),
  bench('eager tree, depth 3 (819 accessor calls)', () => tree(chart3)),
  bench('one lazy row (one accessor call)', () => chart3.dashaRow(400), { iterations: 20_000 }),
  bench('chart, depth 4 (7 380 nodes), built-in provider', () => depth4.computeChart(JD), { iterations: 500 }),
  bench('eager tree, depth 4 (7 380 accessor calls)', () => tree(chart4), { iterations: 500 }),
  // Fewer calls than option A: every chart is an opaque handle freed only when
  // the FinalizationRegistry runs, and 300 depth-5 charts exhausted the wasm
  // memory (`rust_oom`) before that happened.
  bench('chart, depth 5 (66 429 nodes), built-in provider', () => depth5.computeChart(JD), { iterations: 20, rounds: 1 }),
  bench('eager tree, depth 5 (66 429 accessor calls)', () => tree(chart5), { iterations: 10, rounds: 1 }),
];

report(rows);
writeResults(join(HERE, '..', 'results'), 'b-js.json', { option: 'B', binding: 'js-wasm', rows });
