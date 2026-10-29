// The requests the shim forwards, each one SDK call asking for what the
// baseline's entry point answers (`docs/03-design/compat-shim.md` §3.2 to
// §3.5): the A/B's SDK side, moved here (§6.4, item 5).
//
// HAND-WRITTEN. Not exported by the package: `index.js` reads the stored
// birth record and snapshot and calls these on the contexts the snapshot
// means. The A/B (`bench/ab.mjs`) calls them on contexts of its own, so the
// requests it times are the shim's.

import { DashaSystem, Varga } from '@teistro/sdk';

/** The eighteen dasha systems the baseline's whole chart answers, each to depth 3. */
export const BASELINE_DASHAS = Object.freeze([
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
]);

/**
 * The whole chart: the performance page's parity chart (`app-speed`'s
 * `parity-chart`) with the baseline's eighteen dashas added to its
 * sixteen, the avakahada and the rules' longevity, at least what the
 * baseline's whole chart holds.
 */
export const FULL = Object.freeze({
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
});

/** The foundation: what the baseline's holds beside its positions. */
export const FOUNDATION = Object.freeze({
  vargas: Object.values(Varga),
  aspects: true,
  points: true,
  houses: true,
  state: true,
  jaimini: true,
});

/** The rule sets the baseline's yogas and doshas are read against. */
const YOGAS = Object.freeze({ shipped: ['NABHASAS', 'ARISHTAS', 'GANDANTAS', 'DOSHAS', 'YOGAS'] });

/**
 * The readings the baseline takes where a request names its own, which
 * the shim asks for: the Kuja dosha from the lagna, the Moon and Venus in
 * the six houses with the 2nd (cruxes C285, C286); Stree-deergha beyond
 * the 10th or on the bride's own star (C779); and the three spans with
 * Amsayu multiplied, compound enmity, each reduction taken in turn and
 * Nisargayu's listed years (C104, C303). The baseline lifts the Kuja
 * dosha by three of its nine cancellations, which no SDK lift set reads,
 * so no lift is asked.
 */
export const READINGS = Object.freeze({
  kuja: Object.freeze({ from: 'LAGNA_MOON_VENUS', houses: 'WITH_SECOND' }),
  porutham: Object.freeze({ deerghaBeyond: 'TENTH_OR_SAME_STAR' }),
  ayurdaya: Object.freeze({ amsayu_multiplied: true, enmity: 'compound', combine: 'each', nisarga: 'listed' }),
});

/**
 * The fourteen requests over a natal context and the KP context `kp`
 * opens, asking `readings` where a request names its own (the sources'
 * own when one is left out). Each takes `at`, a chart's `{ instant,
 * place, utcOffsetSeconds }`, and what the request needs beyond it, and
 * answers the SDK's own result.
 *
 * @param {import('@teistro/sdk').Context} natal
 * @param {() => import('@teistro/sdk').Context} kp
 * @param {{kuja?: object, porutham?: object, ayurdaya?: object}} [readings]
 */
export function requests(natal, kp, readings = {}) {
  const { kuja, porutham, ayurdaya } = readings;
  const full = ayurdaya ? { ...FULL, rules: { ...FULL.rules, ayurdaya } } : FULL;
  return {
    /** The foundation and the Vimshottari. */
    chart: (at) => natal.chart.found({ ...at, ...FOUNDATION, dashas: [DashaSystem.Vimshottari] }),
    /** The whole natal chart. */
    full: (at) => natal.chart.found({ ...at, ...full }),
    /** The foundation, which carries every divisional chart. */
    vargas: (at) => natal.chart.found({ ...at, ...FOUNDATION }),
    yogas: (at) => natal.chart.found({ ...at, rules: YOGAS }),
    strength: (at) => natal.chart.found({ ...at, shadbala: true, bhavaBala: true, ashtakavarga: true }),
    dashas: (at) => natal.chart.found({ ...at, dashas: BASELINE_DASHAS }),
    /** The almanac day of `date` at the chart's place, on its clock. */
    day: ({ place, utcOffsetSeconds }, date) => natal.almanac.day({ date, place, utcOffsetSeconds }),
    /** The almanac days `from` to `to`, both included. */
    month: ({ place, utcOffsetSeconds }, from, to) => natal.almanac.of({ from, to, place, utcOffsetSeconds }),
    /** The chart as the groom, matched with `partner`'s as the bride. */
    milan: (at, partner) =>
      natal.chart.found({
        ...at,
        matching: { partner, partnerRole: 'BRIDE', ...(kuja && { kuja }), ...(porutham && { porutham }) },
      }),
    /** The annual charts through `year`, with their matters, sahams and the Mudda dasha. */
    varsha: (at, year) =>
      natal.chart.found({
        ...at,
        varsha: { through: year, place: 'birth', matters: 'all', sahams: 'all', dashas: [DashaSystem.Mudda] },
      }),
    /**
     * KP's cusps, lords, significators and ruling planets, on the KP
     * context; with `anyAyanamsha`, on the natal context under its own
     * ayanamsha instead (C157).
     */
    kp: (at, anyAyanamsha = false) =>
      anyAyanamsha ? natal.chart.found({ ...at, kp: { anyAyanamsha } }) : kp().chart.found({ ...at, kp: {} }),
    /** The transits at `instants` over the chart, with the Ashtakavarga's say. */
    gochar: (at, instants) => natal.chart.found({ ...at, ashtakavarga: true, gochar: { instants, ashtakavarga: true } }),
    /** Sade Sati and the dhaiyyas over `window`, `{ from, to }`. */
    sadeSati: (at, window) => natal.chart.found({ ...at, sadeSati: window }),
    /** Whole charts at `instants`, at the chart's place and on its clock. */
    batch: ({ place, utcOffsetSeconds }, instants) => natal.chart.foundMany({ instants, place, utcOffsetSeconds, ...full }),
  };
}
