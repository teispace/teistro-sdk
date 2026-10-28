// The load, the tail, the cold start and the memory of the A/B
// (`docs/03-design/ab-baseline.md` §4 to §7). `ab.mjs` drives them after
// its interleaved requests; this module is also the program each measured
// process runs, by its first argument:
//
//   node bench/ab-load.mjs agent SIDE REQUEST IDS    one process of the processes mode
//   node bench/ab-load.mjs pool SIDE REQUEST N IDS   N worker threads, the threads mode
//   node bench/ab-load.mjs tail SIDE REQUEST IDS     a sustained run's percentiles
//   node bench/ab-load.mjs cold SIDE ID              one cold start
//   node --expose-gc bench/ab-load.mjs memory SIDE IDS
//
// IDS is the corpus ids the request kept, as JSON. A measured process
// prints one JSON line carrying `ab` for each thing it reports, so a line
// a side's engine prints of its own is never read as one.

import { spawn } from 'node:child_process';
import { availableParallelism } from 'node:os';
import { performance } from 'node:perf_hooks';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';
import { Worker, isMainThread, parentPort, workerData } from 'node:worker_threads';

import { SIDES, inputs, open } from './ab-inputs.mjs';

/** The requests driven under load and for their tail (§4, §5). */
export const LOAD_REQUESTS = ['chart', 'day', 'full'];

/** The two ways a load runs: worker threads in one process, or processes. */
export const MODES = ['threads', 'processes'];

/** The trials a load count gets, and how long each lasts, in seconds. */
export const TRIALS = 3;
export const TRIAL_SECONDS = 3;

/** A sustained run's length in seconds, the windows it is cut into, and the percentiles taken. */
export const TAIL_SECONDS = 30;
export const TAIL_WINDOWS = 10;
export const PERCENTILES = [50, 95, 99];

/** The cold starts each side gets. */
export const STARTS = 9;

/** The processes each side's memory is read in, and the requests each answers. */
export const MEMORY_RUNS = 3;
export const MEMORY_WARM = 200;
export const MEMORY_REQUESTS = 10_000;

/**
 * How long a pool's process may run before it is taken to have hung and is
 * killed, in seconds: far past warming every input on every thread and
 * the trials. A side whose native engine is not safe across threads can
 * leave a worker wedged rather than dead, and the pool would wait on it
 * for ever.
 */
export const POOL_DEADLINE_SECONDS = 600;

/** How far ahead of now a trial is set to start, so every agent is waiting when it does. */
const LEAD_MS = 250;

const self = fileURLToPath(import.meta.url);

/** The wall clock in milliseconds, comparable across threads and processes. */
const now = () => performance.timeOrigin + performance.now();

/** 1, 2, 4, … up to `most`, and `most` itself. */
export function counts(most = availableParallelism()) {
  const out = [];
  for (let n = 1; n <= most; n *= 2) out.push(n);
  if (out.at(-1) !== most) out.push(most);
  return out;
}

/** The median, fastest and slowest of a list, under `unit`'s suffix. */
function spread(values, unit) {
  const sorted = [...values].sort((x, y) => x - y);
  return {
    [`median_${unit}`]: sorted[sorted.length >> 1],
    [`min_${unit}`]: sorted[0],
    [`max_${unit}`]: sorted[sorted.length - 1],
  };
}

/** The `p`th percentile of a sorted list, nearest rank. */
function percentile(sorted, p) {
  return sorted[Math.min(sorted.length - 1, Math.max(0, Math.ceil((p / 100) * sorted.length) - 1))];
}

/** The inputs with these ids, in corpus order. */
function kept(ids) {
  const wanted = new Set(ids);
  return inputs().filter((input) => wanted.has(input.id));
}

/**
 * A side opened and warmed on every input of `request`, with the Sun each
 * `chart` answered alone: under load a chart must answer the same sky.
 */
async function warmed(name, request, ids, ephemeris) {
  const opened = await open(name, ephemeris);
  const all = kept(ids);
  const suns = all.map((input) => {
    const answer = opened.side[request](input);
    return request === 'chart' ? opened.side.sun(answer) : undefined;
  });
  return { ...opened, all, suns };
}

/**
 * Waits for the wall-clock instant `startAt`, then asks `request` over the
 * inputs in turn for `seconds`: the answers finished inside the span. A
 * chart whose Sun is not the one the side answered alone refuses the
 * trial, since a side that shares state between threads answers another
 * sky, or none, when two ask at once.
 */
export function drive({ side, all, suns }, request, startAt, seconds) {
  while (now() < startAt);
  const end = startAt + seconds * 1000;
  let answered = 0;
  for (let at = 0; ; at = (at + 1) % all.length) {
    const answer = side[request](all[at]);
    if (suns[at] !== undefined && side.sun(answer) !== suns[at]) {
      throw new Error(`${all[at].id}: the Sun under load stands at ${side.sun(answer)}°, alone at ${suns[at]}°`);
    }
    if (now() > end) return answered;
    answered += 1;
  }
}

/**
 * The trials of one load count over agents already warm: each trial's
 * answers a second, all agents together.
 *
 * @param {{trial: (startAt: number, seconds: number) => Promise<number>}[]} agents
 */
export async function trials(agents, count = TRIALS, seconds = TRIAL_SECONDS) {
  const out = [];
  for (let trial = 0; trial < count; trial += 1) {
    const startAt = now() + LEAD_MS;
    const answered = await Promise.all(agents.map((agent) => agent.trial(startAt, seconds)));
    out.push(answered.reduce((sum, n) => sum + n, 0) / seconds);
  }
  return out;
}

/** Reads the `ab` lines a measured process prints, as they come. */
function lines(stream, onLine) {
  createInterface({ input: stream }).on('line', (line) => {
    if (!line.startsWith('{"ab"')) return;
    onLine(JSON.parse(line));
  });
}

/** Prints a report of this process's, marked as one. */
function report(kind, value) {
  console.log(JSON.stringify({ ab: kind, ...value }));
}

/**
 * Runs this module as a measured process and settles with every `ab`
 * line it printed, or refuses with why it failed: its own error, the
 * signal it died of and what it wrote to stderr, or, given a deadline in
 * seconds, that it had not ended by then, when it is killed.
 */
function run(args, node = [], deadline = undefined) {
  return new Promise((resolve, reject) => {
    const started = now();
    const child = spawn(process.execPath, [...node, self, ...args], { stdio: ['ignore', 'pipe', 'pipe'] });
    const seen = [];
    let stderr = '';
    let late = false;
    const timer = deadline === undefined ? undefined : setTimeout(() => {
      late = true;
      child.kill('SIGKILL');
    }, deadline * 1000);
    lines(child.stdout, (line) => seen.push(line));
    child.stderr.on('data', (chunk) => {
      stderr += chunk;
    });
    child.on('error', reject);
    child.on('close', (code, signal) => {
      clearTimeout(timer);
      const failed = seen.find((line) => line.ab === 'error');
      if (late) reject(new Error(`had not ended in ${deadline} s, and was killed`));
      else if (code === 0 && !failed) resolve({ lines: seen, ms: now() - started });
      else reject(new Error(failed?.reason ?? died(code, signal, stderr)));
    });
  });
}

/** Why a process ended without answering, in one sentence. */
function died(code, signal, stderr) {
  const last = stderr.trim().split('\n').filter(Boolean).at(-1) ?? 'nothing on stderr';
  return `${signal ? `died of ${signal}` : `exited with ${code}`}: ${last}`;
}

/** One process of the processes mode: opens, warms, then answers trials over stdin. */
function processAgent(name, request, ids) {
  const child = spawn(process.execPath, [self, 'agent', name, request, JSON.stringify(ids)], {
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  let stderr = '';
  child.stderr.on('data', (chunk) => {
    stderr += chunk;
  });
  const waiting = [];
  let ready;
  const readied = new Promise((resolve, reject) => {
    ready = { resolve, reject };
  });
  lines(child.stdout, (line) => {
    if (line.ab === 'ready') ready.resolve();
    else if (line.ab === 'answered') waiting.shift()?.resolve(line.answered);
    else if (line.ab === 'error') {
      const why = new Error(line.reason);
      ready.reject(why);
      for (const pending of waiting.splice(0)) pending.reject(why);
    }
  });
  child.on('close', (code, signal) => {
    const why = new Error(died(code, signal, stderr));
    ready.reject(why);
    for (const pending of waiting.splice(0)) pending.reject(why);
  });
  return {
    ready: readied,
    trial: (startAt, seconds) =>
      new Promise((resolve, reject) => {
        waiting.push({ resolve, reject });
        child.stdin.write(`${JSON.stringify({ startAt, seconds })}\n`);
      }),
    close: () =>
      new Promise((resolve) => {
        if (child.exitCode !== null || child.signalCode !== null) resolve();
        else child.once('close', resolve);
        child.stdin.end();
      }),
  };
}

/**
 * One worker thread of the threads mode, the same protocol over its port;
 * the SDK's over `ephemeris` when one is named.
 */
export function threadAgent(name, request, ids, ephemeris) {
  const worker = new Worker(self, { workerData: { agent: true, name, request, ids, ephemeris } });
  const waiting = [];
  const ready = new Promise((resolve, reject) => {
    worker.on('message', (message) => {
      if (message.ab === 'ready') resolve();
      else if (message.ab === 'error') reject(new Error(message.reason));
      else if (message.ab === 'answered') waiting.shift()?.resolve(message.answered);
    });
    worker.on('error', (error) => {
      reject(error);
      for (const pending of waiting.splice(0)) pending.reject(error);
    });
    // A worker that ends without an error, as one whose engine exits it
    // does, would otherwise leave its pool waiting on it.
    worker.on('exit', (code) => {
      const why = new Error(`the worker exited with ${code}`);
      reject(why);
      for (const pending of waiting.splice(0)) pending.reject(why);
    });
  });
  return {
    ready,
    trial: (startAt, seconds) =>
      new Promise((resolve, reject) => {
        waiting.push({ resolve, reject });
        worker.postMessage({ startAt, seconds });
      }),
    close: () => worker.terminate(),
  };
}

/**
 * A load count's trials over `agents`, closed afterwards: the figure, or
 * the reason the side could not be opened or asked there.
 */
export async function measured(agents, count = TRIALS, seconds = TRIAL_SECONDS) {
  try {
    await Promise.all(agents.map((agent) => agent.ready));
    return { rps: await trials(agents, count, seconds) };
  } catch (error) {
    return { reason: error.message };
  } finally {
    await Promise.all(agents.map((agent) => agent.close()));
  }
}

/** A load line, or the refusal of one. */
function loadLine(name, request, mode, threads, result) {
  return result.reason === undefined
    ? { kind: 'load', side: name, request, mode, threads, trials: TRIALS, seconds: TRIAL_SECONDS, ...spread(result.rps, 'rps') }
    : { kind: 'unloaded', side: name, request, mode, threads, reason: result.reason };
}

/**
 * Every figure of §4 to §7, as the lines `ab.mjs` prints: each side in
 * processes of its own, the two alternating so that a drift in the
 * machine falls on both.
 *
 * @param {Record<string, {id: string}[]>} keptBy the inputs each request kept
 */
export async function* measureLoad(keptBy) {
  const ids = (request) => keptBy[request].map((input) => input.id);
  yield { kind: 'machine', parallelism: availableParallelism() };
  let turn = 0;
  const alternating = () => (turn++ % 2 === 0 ? SIDES : [...SIDES].reverse());
  for (const request of LOAD_REQUESTS) {
    for (const threads of counts()) {
      for (const name of alternating()) {
        // A pool runs in a process of its own, so that a side whose engine
        // cannot live in a worker brings down that process and no other.
        const pool = await run(['pool', name, request, String(threads), JSON.stringify(ids(request))], [], POOL_DEADLINE_SECONDS).then(
          ({ lines: [line] }) => line,
          (error) => ({ kind: 'unloaded', side: name, request, mode: 'threads', threads, reason: error.message }),
        );
        delete pool.ab;
        yield pool;
        const agents = Array.from({ length: threads }, () => processAgent(name, request, ids(request)));
        yield loadLine(name, request, 'processes', threads, await measured(agents));
      }
    }
  }
  for (const request of LOAD_REQUESTS) {
    for (const name of alternating()) {
      const { lines: tail } = await run(['tail', name, request, JSON.stringify(ids(request))]);
      for (const { ab, ...line } of tail) yield { kind: 'tail', side: name, request, ...line };
    }
  }
  const cold = Object.fromEntries(SIDES.map((name) => [name, { open: [], first: [], process: [] }]));
  for (let start = 0; start < STARTS; start += 1) {
    for (const name of alternating()) {
      const { lines: [line], ms } = await run(['cold', name, ids('chart')[0]]);
      cold[name].open.push(line.open_ms);
      cold[name].first.push(line.first_ms);
      cold[name].process.push(ms);
    }
  }
  for (const name of SIDES) {
    for (const [phase, values] of Object.entries(cold[name])) {
      yield { kind: 'cold', side: name, phase, runs: values.length, ...spread(values, 'ms') };
    }
  }
  const memory = Object.fromEntries(SIDES.map((name) => [name, { 'peak-rss': [], heap: [], 'heap-growth': [] }]));
  for (let process_ = 0; process_ < MEMORY_RUNS; process_ += 1) {
    for (const name of alternating()) {
      const { lines: [line] } = await run(['memory', name, JSON.stringify(ids('chart'))], ['--expose-gc']);
      memory[name]['peak-rss'].push(line.peak_rss_mb);
      memory[name].heap.push(line.heap_mb);
      memory[name]['heap-growth'].push(line.growth_kb);
    }
  }
  const units = { 'peak-rss': 'MB', heap: 'MB', 'heap-growth': 'KB' };
  for (const name of SIDES) {
    for (const [measure, values] of Object.entries(memory[name])) {
      yield {
        kind: 'memory',
        side: name,
        request: 'chart',
        measure,
        unit: units[measure],
        runs: values.length,
        requests: MEMORY_REQUESTS,
        ...spread(values, 'value'),
      };
    }
  }
}

/** Collects until two reads of the heap and the external bytes agree to the kilobyte. */
async function settled() {
  let last;
  for (let tries = 1; ; tries += 1) {
    globalThis.gc();
    // A finaliser runs between turns of the event loop, not inside `gc()`.
    await new Promise((resolve) => setImmediate(resolve));
    const { heapUsed, external } = process.memoryUsage();
    const read = Math.round((heapUsed + external) / 1024);
    if (read === last || tries === 100) return read;
    last = read;
  }
}

/** The measured processes' programs, by their first argument. */
const programs = {
  async agent(name, request, ids) {
    const agent = await warmed(name, request, JSON.parse(ids));
    report('ready', {});
    for await (const line of createInterface({ input: process.stdin })) {
      const { startAt, seconds } = JSON.parse(line);
      report('answered', { answered: drive(agent, request, startAt, seconds) });
    }
  },
  async pool(name, request, threads, ids) {
    const agents = Array.from({ length: Number(threads) }, () => threadAgent(name, request, JSON.parse(ids)));
    const line = loadLine(name, request, 'threads', Number(threads), await measured(agents));
    report(line.kind, line);
  },
  async tail(name, request, ids) {
    const { side, all } = await warmed(name, request, JSON.parse(ids));
    const windowMs = (TAIL_SECONDS * 1000) / TAIL_WINDOWS;
    const windows = Array.from({ length: TAIL_WINDOWS }, () => []);
    const start = performance.now();
    for (let at = 0; ; at = (at + 1) % all.length) {
      const began = performance.now();
      side[request](all[at]);
      const ended = performance.now();
      const slot = Math.floor((began - start) / windowMs);
      if (slot >= TAIL_WINDOWS) break;
      windows[slot].push((ended - began) * 1000);
    }
    const sorted = windows.map((times) => times.sort((x, y) => x - y));
    const requests = sorted.reduce((sum, times) => sum + times.length, 0);
    for (const p of PERCENTILES) {
      report('tail', {
        percentile: p,
        seconds: TAIL_SECONDS,
        windows: TAIL_WINDOWS,
        requests,
        ...spread(
          sorted.map((times) => percentile(times, p)),
          'us',
        ),
      });
    }
  },
  async cold(name, id) {
    const { side } = await open(name);
    const ready = performance.now();
    side.chart(kept([id])[0]);
    report('cold', { open_ms: ready, first_ms: performance.now() - ready });
  },
  async memory(name, ids) {
    const { side } = await open(name);
    const all = kept(JSON.parse(ids));
    for (let at = 0; at < MEMORY_WARM; at += 1) side.chart(all[at % all.length]);
    const before = await settled();
    for (let at = 0; at < MEMORY_REQUESTS; at += 1) side.chart(all[at % all.length]);
    const after = await settled();
    report('memory', {
      // `maxRSS` is in kilobytes on every platform Node runs on.
      peak_rss_mb: process.resourceUsage().maxRSS / 1024,
      heap_mb: after / 1024,
      growth_kb: after - before,
    });
  },
};

if (!isMainThread && workerData?.agent) {
  // A worker thread of a pool: the agent's protocol over its port.
  const { name, request, ids, ephemeris } = workerData;
  warmed(name, request, ids, ephemeris).then(
    (agent) => {
      parentPort.on('message', ({ startAt, seconds }) => {
        parentPort.postMessage({ ab: 'answered', answered: drive(agent, request, startAt, seconds) });
      });
      parentPort.postMessage({ ab: 'ready' });
    },
    (error) => parentPort.postMessage({ ab: 'error', reason: error.message }),
  );
} else if (isMainThread && process.argv[1] === self) {
  // Not awaited at the top level: the SDK's side imports `ab.mjs`, and a
  // module still evaluating could not be imported back. The process ends
  // once its reports are written, whatever handles a side's engine holds.
  const [program, ...args] = process.argv.slice(2);
  programs[program](...args)
    .then(
      () => 0,
      (error) => {
        report('error', { reason: error.message });
        return 1;
      },
    )
    .then((code) => process.stdout.write('', () => process.exit(code)));
}
