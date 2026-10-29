// The interleaved A/B of `docs/03-design/ab-baseline.md`: the SDK through
// Node against the baseline engine, on the golden corpus's inputs, printed
// as one JSON line a figure. `cargo xtask measure-ab` runs it.
//
//   TEISTRO_AB_BASELINE=/path/to/adapter.mjs \
//   TEISTRO_TEIMERIS_ADAPTER=/path/to/libteistro_ephemeris_teimeris.dylib \
//   node bench/ab.mjs
//
// The adapter is a module kept outside the SDK, beside the baseline
// engine's own exporter. Its contract is `Side` in `ab-inputs.mjs`: the
// SDK side here is written to the same contract, so the runner treats the
// two alike. After the interleaved requests (§2, §3) it drives the load,
// the tail, the cold start and the memory (§4 to §7) through
// `ab-load.mjs`, each side in processes of its own.

import { performance } from 'node:perf_hooks';
import { fileURLToPath } from 'node:url';

import { Calendar, Context, DashaSystem, Varga, date } from '../lib/index.js';
import { Agreement, figures, joinYogas, sdkFacts } from './ab-agree.mjs';
import { REQUESTS, SIDES, inputs, open, paths, sdkVersion } from './ab-inputs.mjs';
import {
  Kept,
  charakRung,
  committeeRung,
  kalaprakasikaRung,
  pyjhoraRung,
  ramanRung,
  swissEphemeris,
  swissRung,
  whoKey,
} from './ab-reference.mjs';

export { REQUESTS, SIDES, inputs } from './ab-inputs.mjs';

/** The fewest rounds a figure is taken from. */
export const ROUNDS = 15;

/**
 * How far apart the two sides' Suns may be and still be the same work:
 * an arcminute. A different instant or place moves the Sun by more; a
 * different ayanamsha model by less, and that is the golden vectors'
 * concern, not a timing's.
 */
export const SAME_SKY_DEG = 1 / 60;

/**
 * An answer that holds nothing the request asked for, such as a varsha
 * with no pravesha: an answer, not a refusal, and not the request's work
 * either, so {@link agreeing} keeps the input out of that request's timing
 * and reports it with its reason, apart from the refusals.
 */
export class Empty {
  /** @param {string} reason what the answer lacked */
  constructor(reason) {
    this.reason = reason;
  }
}

/** Takes values only so that reading them, which decodes them, is done. */
function decoded(...values) {
  return values.length;
}

/** Reads every period of every dasha, which decodes them. */
function periods(dashas) {
  let count = 0;
  for (const dasha of dashas) count += dasha.periods.length;
  return count;
}

/** The eighteen dasha systems the baseline's whole chart answers, each to depth 3 (§2). */
const BASELINE_DASHAS = [
  DashaSystem.Vimshottari,
  DashaSystem.Yogini,
  DashaSystem.Tribhagi,
  DashaSystem.Ashtottari,
  DashaSystem.Kalachakra,
  DashaSystem.DwisaptatiSama,
  DashaSystem.Shatabdika,
  DashaSystem.ChaturashitiSama,
  DashaSystem.Dwadashottari,
  DashaSystem.Panchottari,
  DashaSystem.Chara,
  DashaSystem.Narayana,
  DashaSystem.Shoola,
  DashaSystem.NiryanaShoola,
  DashaSystem.Padanadhamsa,
  DashaSystem.Mandooka,
  DashaSystem.Drig,
  DashaSystem.Trikona,
];

/**
 * The performance page's parity chart (`app-speed`'s `parity-chart`) with
 * the baseline's eighteen dashas added to its sixteen, the avakahada and
 * the rules' longevity: at least what the baseline's whole chart holds.
 */
const FULL = {
  vargas: Object.values(Varga),
  dashas: [
    ...new Set([
      DashaSystem.Vimshottari,
      DashaSystem.Ashtottari,
      DashaSystem.Yogini,
      DashaSystem.Kalachakra,
      DashaSystem.Shodashottari,
      DashaSystem.Dwadashottari,
      DashaSystem.Panchottari,
      DashaSystem.Shatabdika,
      DashaSystem.ChaturashitiSama,
      DashaSystem.DwisaptatiSama,
      DashaSystem.Shashtihayani,
      DashaSystem.ShattrimshaSama,
      DashaSystem.Chara,
      DashaSystem.Narayana,
      DashaSystem.Shoola,
      DashaSystem.SudarshanaChakra,
      ...BASELINE_DASHAS,
    ]),
  ],
  rules: { shipped: ['NABHASAS', 'READINGS', 'ARISHTAS', 'GANDANTAS', 'DOSHAS', 'YOGAS'], longevity: true },
  aspects: true,
  points: true,
  houses: true,
  state: true,
  ashtakavarga: true,
  vimshopaka: true,
  vaiseshikamsa: true,
  dashaPhala: true,
  jaimini: true,
  avakahada: true,
  moment: true,
  shadbala: true,
  bhavaBala: true,
};

/** Reads every section `FULL` asks a chart for. */
function readFull(chart) {
  return decoded(
    chart.grahas,
    chart.lagnaDeg,
    chart.day,
    chart.timing,
    chart.vargas,
    periods(chart.dashas),
    chart.rules,
    chart.aspects,
    chart.points,
    chart.bhavas,
    chart.states,
    chart.houses,
    chart.chalit,
    chart.ashtakavarga,
    chart.vimshopaka,
    chart.vaiseshikamsa,
    chart.dashaPhala,
    chart.jaimini,
    chart.avakahada,
    chart.moment,
    chart.shadbala,
    chart.bhavaBala,
  );
}

/** What the baseline's foundation holds beside its positions (§2, `vargas`). */
const FOUNDATION = { vargas: Object.values(Varga), aspects: true, points: true, houses: true, state: true, jaimini: true };

/** Reads every section `FOUNDATION` asks a chart for. */
function readFoundation(chart) {
  return decoded(
    chart.grahas,
    chart.lagnaDeg,
    chart.day,
    chart.vargas,
    chart.aspects,
    chart.points,
    chart.bhavas,
    chart.states,
    chart.houses,
    chart.jaimini,
  );
}

/** Reads every part of an almanac day that the baseline's day holds. */
function readDay(day) {
  return decoded(
    day.window,
    day.tithi,
    day.nakshatra,
    day.yoga,
    day.karana,
    day.kaalas,
    day.choghadiya,
    day.horas,
    day.moonEvents,
    day.muhurtaYogas,
  );
}

/** The civil date `offsetDays` after an input's. */
function civil(input, offsetDays = 0) {
  const { year, month, day } = input.civilDate;
  const at = new Date(Date.UTC(year, month - 1, day + offsetDays));
  return date(Calendar.Gregorian, at.getUTCFullYear(), at.getUTCMonth() + 1, at.getUTCDate());
}

/** The catalogues' joined yogas, read once (`ab-agree.mjs`). */
let joined;

/**
 * The sky a context casts in, from its resolved settings, in the terms of
 * `ab-swisseph.py`; `module` names a house module's override.
 */
export function skyOf(context, module) {
  const { frame, houses } = context.settings;
  const system = houses.module_overrides[module] ?? houses.placement_system;
  return {
    ayanamsha: frame.ayanamsha.id,
    basis: frame.ayanamsha_basis.toLowerCase(),
    centre: frame.centre.toLowerCase(),
    node: frame.node.toLowerCase(),
    houses: system === 'PLACIDUS' ? 'P' : 'W',
  };
}

/**
 * The sky a context casts a transit in: its chart's, seen from where
 * `gochar.transit_centre` puts it (crux C778).
 */
export function transitSkyOf(context) {
  const centre = context.settings.gochar.transit_centre;
  const sky = skyOf(context);
  return centre === 'AS_CHART' ? sky : { ...sky, centre: centre.toLowerCase() };
}

/**
 * The SDK's side over contexts the caller opened and disposes: `ctx` for
 * every request but `kp`, which answers on the context `kp` gives, opened
 * on its own profile when first asked for. `gochar` and `sadeSati` answer
 * on `ctx` too, their transits seen from where its `gochar.transit_centre`
 * puts them (§10).
 *
 * `asked` holds the readings a request asks under where a convention
 * names its own: `kuja` and `porutham`, the Kuja dosha's and the ten
 * considerations' for `milan`, and `ayurdaya`, the three spans' for
 * `full` and `batch`; the sources' own when left out.
 *
 * @param {Context} ctx
 * @param {() => Context} kp
 * @param {string} version
 * @param {{kuja?: object, porutham?: object, ayurdaya?: object}} [asked]
 * @returns {import('./ab-inputs.mjs').Side}
 */
export function sdkSide(ctx, kp, version, asked = {}) {
  const { kuja, porutham, ayurdaya } = asked;
  const full = ayurdaya ? { ...FULL, rules: { ...FULL.rules, ayurdaya } } : FULL;
  const at = (input) => ({ instant: input.jdUt, place: input.place, utcOffsetSeconds: input.utcOffsetSeconds });
  // Each request asks for what the baseline's answer holds
  // (`03-design/ab-baseline.md` §2). An answer decodes on read, and the
  // baseline's comes back built, so each request reads what it asked
  // for: the timing is of the same work, decoding included.
  return {
    version,
    chart: (input) => {
      const chart = ctx.chart.found({ ...at(input), ...FOUNDATION, dashas: [DashaSystem.Vimshottari] });
      decoded(readFoundation(chart), periods(chart.dashas));
      return chart;
    },
    day: (input) => {
      const day = ctx.almanac.day({ date: civil(input), place: input.place, utcOffsetSeconds: input.utcOffsetSeconds });
      readDay(day);
      return day;
    },
    full: (input) => {
      const chart = ctx.chart.found({ ...at(input), ...full });
      readFull(chart);
      return chart;
    },
    vargas: (input) => {
      const chart = ctx.chart.found({ ...at(input), ...FOUNDATION });
      readFoundation(chart);
      return chart;
    },
    yogas: (input) => {
      const chart = ctx.chart.found({
        ...at(input),
        rules: { shipped: ['NABHASAS', 'ARISHTAS', 'GANDANTAS', 'DOSHAS', 'YOGAS'] },
      });
      decoded(chart.grahas, chart.rules);
      return chart;
    },
    strength: (input) => {
      const chart = ctx.chart.found({ ...at(input), shadbala: true, bhavaBala: true, ashtakavarga: true });
      decoded(chart.grahas, chart.shadbala, chart.bhavaBala, chart.ashtakavarga);
      return chart;
    },
    dashas: (input) => {
      const chart = ctx.chart.found({ ...at(input), dashas: BASELINE_DASHAS });
      decoded(chart.grahas, periods(chart.dashas));
      return chart;
    },
    month: (input) => {
      const days = ctx.almanac.of({
        from: civil(input),
        to: civil(input, 29),
        place: input.place,
        utcOffsetSeconds: input.utcOffsetSeconds,
      });
      for (const day of days) readDay(day);
      return days;
    },
    milan: (input) => {
      const { partner } = input;
      const chart = ctx.chart.found({
        ...at(input),
        matching: {
          partner: { instant: partner.jdUt, place: partner.place, utcOffsetSeconds: partner.utcOffsetSeconds },
          partnerRole: 'BRIDE',
          ...(kuja && { kuja }),
          ...(porutham && { porutham }),
        },
      });
      decoded(chart.grahas, chart.matching, chart.porutham, chart.kuja, chart.marriageDoshas);
      return chart;
    },
    varsha: (input) => {
      const chart = ctx.chart.found({
        ...at(input),
        varsha: { through: input.year ?? 1, place: 'birth', matters: 'all', sahams: 'all', dashas: [DashaSystem.Mudda] },
      });
      const year = chart.praveshas.at(-1);
      if (year === undefined) return new Empty('answered no pravesha (an empty `praveshas`)');
      decoded(chart.grahas, year.annual);
      return chart;
    },
    kp: (input) => {
      const chart = kp().chart.found({ ...at(input), kp: {} });
      decoded(chart.grahas, chart.kp);
      return chart;
    },
    gochar: (input) => {
      const chart = ctx.chart.found({
        ...at(input),
        ashtakavarga: true,
        gochar: { instants: [input.transitAt], ashtakavarga: true },
      });
      decoded(chart.grahas, chart.ashtakavarga, chart.gochar);
      return chart;
    },
    sadeSati: (input) => {
      const chart = ctx.chart.found({ ...at(input), sadeSati: input.window });
      decoded(chart.grahas, chart.sadeSati);
      return chart;
    },
    batch: (input) => {
      const charts = ctx.chart.foundMany({
        instants: input.instants,
        place: input.place,
        utcOffsetSeconds: input.utcOffsetSeconds,
        ...full,
      });
      for (const chart of charts) readFull(chart);
      return charts;
    },
    sun: (answer) => answer.grahas.find((placed) => placed.graha === 'graha.SUN').longitudeDeg,
    facts: (request, answer, input) => sdkFacts(request, answer, input, (joined ??= figures())),
    get conventions() {
      return { chart: skyOf(ctx), kp: skyOf(kp(), 'kp'), gochar: transitSkyOf(ctx) };
    },
  };
}

/**
 * The two conventions the SDK answers under (§10): `timed`, the profiles
 * every figure is timed on, and `same`, the profiles that cast as the
 * baseline casts, so that what still differs is method. The SDK refuses a
 * polar day unless told how to reckon one, and the baseline answers
 * every day: the timed profiles reckon it from civil midnight, and the
 * same ones by the conformance profile's own policy, from the nearest
 * sunrise and sunset, given to KP too (§10).
 */
export const CONVENTIONS = {
  timed: {
    profile: 'parashari-classical',
    kp: 'kp-default',
    settings: { day: { polar_day_policy: 'CIVIL_MIDNIGHT' } },
    kpSettings: { day: { polar_day_policy: 'CIVIL_MIDNIGHT' } },
  },
  same: {
    profile: 'conformance-baseline',
    kp: 'kp-default',
    // KP on the baseline's frame: its mean node, topocentric and nutated;
    // and its ruling planets as the baseline reads them, as
    // `conformance-baseline` does: seven, the sub lords too, and the civil
    // weekday's lord (cruxes C150, C151).
    kpSettings: {
      frame: { node: 'MEAN', centre: 'TOPOCENTRIC', ayanamsha_basis: 'TRUE' },
      day: { polar_day_policy: 'NEAREST_EVENT' },
      kp: { day_lord_day: 'CIVIL', ruling_count: 'WITH_SUBS' },
    },
    // The baseline casts a birth from its place and a transit from the
    // Earth's centre, as `conformance-baseline` records (crux C778); said
    // here too, since the transit requests read it.
    settings: { gochar: { transit_centre: 'GEOCENTRIC' } },
    // The readings the baseline takes where a request names its own. The
    // Kuja dosha: Mars from the lagna, the Moon and Venus, in the six
    // houses with the 2nd (C285, C286); the baseline lifts it by three of
    // its nine cancellations, which no SDK lift set reads, so none is
    // asked. Stree-deergha beyond the 10th or on the bride's own star
    // (C779). The three spans: Amsayu multiplied, compound enmity, each
    // reduction taken in turn, and Nisargayu's listed years (C104, C303).
    asked: {
      kuja: { from: 'LAGNA_MOON_VENUS', houses: 'WITH_SECOND' },
      porutham: { deerghaBeyond: 'TENTH_OR_SAME_STAR' },
      ayurdaya: { amsayu_multiplied: true, enmity: 'compound', combine: 'each', nisarga: 'listed' },
    },
  },
};

/** Two settings patches, the second's groups laid over the first's. */
function merged(a, b) {
  const out = { ...a };
  for (const [group, fields] of Object.entries(b)) out[group] = { ...a[group], ...fields };
  return out;
}

/**
 * The SDK's side on its own contexts over `ephemeris`, under one of
 * {@link CONVENTIONS} with `patch` laid over both its profiles.
 *
 * @returns {{side: import('./ab-inputs.mjs').Side, dispose: () => void}}
 */
export function openSdk(ephemeris, version, convention = 'timed', patch = {}) {
  const c = CONVENTIONS[convention];
  const ctx = new Context({ profile: c.profile, ephemeris, settings: merged(c.settings, patch) });
  // Opened on first use, as the baseline builds its applied engines, so a
  // cold start pays for what its first chart reads and no more (§6).
  let kp;
  const kpContext = () => (kp ??= new Context({ profile: c.kp, ephemeris, settings: merged(c.kpSettings, patch) }));
  return {
    side: sdkSide(ctx, kpContext, version, c.asked),
    dispose: () => {
      kp?.dispose();
      ctx.dispose();
    },
  };
}

/**
 * Which inputs each request is timed over: the ones both sides answered
 * and, for every request, computed the same sky for. An input a side
 * refuses (the corpus's range edges, where an ephemeris's data ends, or a
 * polar day without a sunrise) is kept out of that request, with the side
 * and the reason; so is one a side answers {@link Empty}, reported apart
 * as `empty` since it was answered; an input whose
 * two Suns stand apart is kept out of every request, since its instant
 * or place was not the same on both sides. `requests` must hold `chart`,
 * whose Suns are compared.
 *
 * @returns {{kept: Record<string, import('./ab-inputs.mjs').Input[]>, differ: {id: string, a: number, b: number}[],
 *   refused: {id: string, side: number, request: string, reason: string}[],
 *   empty: {id: string, side: number, request: string, reason: string}[]}}
 */
export function agreeing(a, b, all, requests = REQUESTS) {
  const kept = Object.fromEntries(requests.map((request) => [request, []]));
  const differ = [];
  const refused = [];
  const empty = [];
  for (const input of all) {
    const answers = {};
    for (const request of requests) {
      answers[request] = [a, b].map((side, at) => {
        let answer;
        try {
          answer = side[request](input);
        } catch (error) {
          refused.push({ id: input.id, side: at, request, reason: error.message });
          return undefined;
        }
        if (answer instanceof Empty) {
          empty.push({ id: input.id, side: at, request, reason: answer.reason });
          return undefined;
        }
        return answer;
      });
    }
    const [ca, cb] = answers.chart;
    if (ca !== undefined && cb !== undefined) {
      const sa = a.sun(ca);
      const sb = b.sun(cb);
      if (Math.abs(((sa - sb + 540) % 360) - 180) > SAME_SKY_DEG) {
        differ.push({ id: input.id, a: sa, b: sb });
        continue;
      }
    }
    for (const request of requests) {
      if (answers[request].every((answer) => answer !== undefined)) kept[request].push(input);
    }
  }
  return { kept, differ, refused, empty };
}

/**
 * Both sides over every input, `rounds` times, alternating which goes
 * first, for each request. A round's figure is its time over the inputs,
 * per input, in microseconds. `inputs` is one list for every request, or
 * a list per request as {@link agreeing} keeps them.
 *
 * @returns {{side: string, request: string, rounds: number, inputs: number,
 *   median_us: number, min_us: number, max_us: number}[]}
 */
export function interleaved(sides, inputs, rounds = ROUNDS, requests = REQUESTS) {
  const out = [];
  for (const request of requests) {
    const all = Array.isArray(inputs) ? inputs : inputs[request];
    const runs = sides.map(() => []);
    // Warm each side's paths before anything is timed.
    for (const side of sides) for (const input of all) side[request](input);
    for (let round = 0; round < rounds; round += 1) {
      const order = round % 2 === 0 ? [0, 1] : [1, 0];
      for (const at of order) {
        const side = sides[at];
        const start = performance.now();
        for (const input of all) side[request](input);
        runs[at].push(((performance.now() - start) * 1000) / all.length);
      }
    }
    sides.forEach((side, at) => {
      const sorted = [...runs[at]].sort((x, y) => x - y);
      out.push({
        side: SIDES[at],
        version: side.version,
        request,
        rounds,
        inputs: all.length,
        median_us: sorted[sorted.length >> 1],
        min_us: sorted[0],
        max_us: sorted[sorted.length - 1],
      });
    });
  }
  return out;
}

/**
 * Every request's agreement (§10) under both of the SDK's conventions, over
 * the inputs its timing kept, and every rung's accuracy (§11), as lines of
 * the record. `reopen` opens the SDK under a convention with a patch, for
 * a worked example cast on its own ayanamsha; what it opens is disposed
 * here.
 *
 * @param {{timed: import('./ab-inputs.mjs').Side, same: import('./ab-inputs.mjs').Side,
 *   baseline: import('./ab-inputs.mjs').Side,
 *   reopen: (convention: string, patch: object) => {side: import('./ab-inputs.mjs').Side, dispose: () => void}}} sides
 */
export function accuracy({ timed, same, baseline, reopen }, kept, all, requests = REQUESTS) {
  const byId = new Map(all.map((input) => [input.id, input]));
  const agreements = { timed: new Agreement(), same: new Agreement() };
  const unanswered = [];
  const facts = new Kept();
  const opened = [];
  const whos = [
    { side: 'sdk', convention: 'timed', answers: timed },
    { side: 'sdk', convention: 'same', answers: same },
    { side: 'baseline', convention: 'timed', answers: baseline, reopened: () => baseline },
  ];
  for (const who of whos.filter((w) => w.side === 'sdk')) {
    who.reopened = (patch) => {
      const sdk = reopen(who.convention, patch);
      opened.push(sdk);
      return sdk.side;
    };
  }
  joined ??= figures();
  try {
    for (const request of requests) {
      for (const input of kept[request]) {
        const theirs = joinYogas(baseline.facts(request, baseline[request](input), input), joined);
        facts.add(whoKey(whos[2]), request, input.id, theirs);
        for (const who of whos.slice(0, 2)) {
          let ours;
          try {
            ours = who.answers.facts(request, who.answers[request](input), input);
          } catch (error) {
            unanswered.push({ kind: 'unanswered', convention: who.convention, request, id: input.id, reason: error.message });
            continue;
          }
          agreements[who.convention].add(request, input.id, ours, theirs);
          facts.add(whoKey(who), request, input.id, ours);
        }
      }
    }
    return [
      ...agreements.timed.lines({ kind: 'agree', convention: 'timed' }),
      ...agreements.same.lines({ kind: 'agree', convention: 'same' }),
      ...unanswered,
      ...swissRung(whos, facts, byId),
      ...pyjhoraRung(whos, facts, byId),
      ...committeeRung(whos),
      ...ramanRung(whos),
      ...charakRung(whos),
      ...kalaprakasikaRung(whos, facts, byId),
    ];
  } finally {
    for (const sdk of opened) sdk.dispose();
  }
}

async function main() {
  const { plugin } = paths();
  // Refused now rather than after the timings, when the rungs need it.
  swissEphemeris();
  const all = inputs();
  const sdk = openSdk({ plugin }, sdkVersion());
  const baseline = await open('baseline');
  let kept;
  try {
    const agreed = agreeing(sdk.side, baseline.side, all);
    kept = agreed.kept;
    for (const d of agreed.differ) {
      console.log(JSON.stringify({ differs: d.id, sdk: d.a, baseline: d.b }));
    }
    for (const r of agreed.refused) {
      console.log(JSON.stringify({ refused: r.id, side: SIDES[r.side], request: r.request, reason: r.reason }));
    }
    for (const e of agreed.empty) {
      console.log(JSON.stringify({ empty: e.id, side: SIDES[e.side], request: e.request, reason: e.reason }));
    }
    // A request at a time, so that each request's figures are printed as
    // they are taken.
    for (const request of REQUESTS) {
      for (const line of interleaved([sdk.side, baseline.side], kept, ROUNDS, [request])) {
        console.log(JSON.stringify({ ...line, corpus: all.length }));
      }
    }
    const same = openSdk({ plugin }, sdkVersion(), 'same');
    try {
      const reopen = (convention, patch) => openSdk({ plugin }, sdkVersion(), convention, patch);
      const sides = { timed: sdk.side, same: same.side, baseline: baseline.side, reopen };
      for (const line of accuracy(sides, kept, all)) console.log(JSON.stringify(line));
    } finally {
      same.dispose();
    }
  } finally {
    sdk.dispose();
  }
  // The load, the tail, the cold start and the memory, each side in
  // processes of its own over the inputs its requests kept. Imported
  // here, since those processes import this module to open the SDK.
  const { measureLoad } = await import('./ab-load.mjs');
  for await (const line of measureLoad(kept)) console.log(JSON.stringify(line));
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await main();
