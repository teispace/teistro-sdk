// The requests the shim forwards, each one SDK call asking for what the
// baseline's entry point answers (`docs/03-design/compat-shim.md` §3.2 to
// §3.6): the A/B's SDK side, moved here (§6.4, item 5), and the rest of
// the chart, almanac and applied rows (item 6).
//
// HAND-WRITTEN. Not exported by the package: `index.js` reads the stored
// birth record and snapshot and calls these on the contexts the snapshot
// means. The A/B (`bench/ab.mjs`) calls them on contexts of its own, so the
// requests it times are the shim's.

import { DashaSystem, Nakshatra, Rashi, Varga, returnsRequest } from '@teistro/sdk';

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

/** The rule set a milan verdict reads beside the kootas: the doshas. */
export const VERDICT = Object.freeze({ shipped: Object.freeze(['DOSHAS']) });

/**
 * The readings the baseline takes where a request names its own, which
 * the shim asks for: the Kuja dosha from the lagna, the Moon and Venus in
 * the six houses with the 2nd (cruxes C285, C286); Stree-deergha beyond
 * the 10th or on the bride's own star (C779); and the three spans with
 * Amsayu multiplied, compound enmity, each reduction taken in turn and
 * Nisargayu's listed years (C104, C303); a rashifal read at six on the
 * clock with each period's events as the baseline scans them: none in a
 * day, the four slow grahas' in a week, every graha's but the Moon's in a
 * month or a year (C358, C360). The baseline lifts the Kuja dosha by
 * three of its nine cancellations, which no SDK lift set reads, so no
 * lift is asked.
 */
export const READINGS = Object.freeze({
  kuja: Object.freeze({ from: 'LAGNA_MOON_VENUS', houses: 'WITH_SECOND' }),
  porutham: Object.freeze({ deerghaBeyond: 'TENTH_OR_SAME_STAR' }),
  ayurdaya: Object.freeze({ amsayu_multiplied: true, enmity: 'compound', combine: 'each', nisarga: 'listed' }),
  rashifal: Object.freeze({
    snapshot: Object.freeze({ at: 'CLOCK', hour: 6, minute: 0 }),
    events: Object.freeze({
      DAILY: Object.freeze([]),
      WEEKLY: Object.freeze(['JUPITER', 'SATURN', 'RAHU', 'KETU']),
      MONTHLY: Object.freeze(['SUN', 'MARS', 'MERCURY', 'JUPITER', 'VENUS', 'SATURN', 'RAHU', 'KETU']),
      YEARLY: Object.freeze(['SUN', 'MARS', 'MERCURY', 'JUPITER', 'VENUS', 'SATURN', 'RAHU', 'KETU']),
    }),
  }),
});

/** The native a muhurta's natal tara and chandra bala are read against: a chart's Moon's star and sign. */
export function nativeOf(chart) {
  const moon = chart.grahas.find((placed) => placed.graha === 'graha.MOON').longitudeDeg;
  const lon = ((moon % 360) + 360) % 360;
  return {
    star: Object.values(Nakshatra)[Math.floor((lon * 27) / 360)],
    moonSign: Object.values(Rashi)[Math.floor(lon / 30)],
  };
}

/**
 * The readings the baseline's prashna and remedies take where a request
 * names its own: every reading of the baseline's prashna (C337 to C340),
 * and its functional nature.
 */
export const PRASHNA = Object.freeze({ rules: 'BASELINE' });
export const REMEDIES = Object.freeze({ functional: Object.freeze({ scheme: 'BASELINE' }) });

/**
 * The requests over a natal context and the KP context `kp`
 * opens, asking `readings` where a request names its own (the sources'
 * own when one is left out). Each takes `at`, a chart's `{ instant,
 * place, utcOffsetSeconds }`, and what the request needs beyond it, and
 * answers the SDK's own result.
 *
 * @param {import('@teistro/sdk').Context} natal
 * @param {() => import('@teistro/sdk').Context} kp
 * @param {{kuja?: object, porutham?: object, ayurdaya?: object, rashifal?: {snapshot: object, events: Record<string, readonly string[]>}}} [readings]
 * @param {() => import('@teistro/sdk').Context} [elect] the context a
 *   muhurta is searched on and a prashna cast on, opened on first use:
 *   the natal one unless a convention casts their sky in a frame of its
 *   own, as a stored snapshot's `muhurta` opening does, from the Earth's
 *   centre with the true node (`recordedSettings`, cruxes C781, C800)
 */
export function requests(natal, kp, readings = {}, elect = () => natal) {
  const { kuja, porutham, ayurdaya, rashifal } = readings;
  const full = ayurdaya ? { ...FULL, rules: { ...FULL.rules, ayurdaya } } : FULL;
  const matching = (partner) => ({ partner, partnerRole: 'BRIDE', ...(kuja && { kuja }), ...(porutham && { porutham }) });
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
    milan: (at, partner) => natal.chart.found({ ...at, matching: matching(partner) }),
    /** The milan, with the doshas' rules read and their readings planned for the verdict. */
    milanVerdict: (at, partner) =>
      natal.chart.found({ ...at, matching: matching(partner), rules: VERDICT, interpret: { readings: true } }),
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
    /**
     * A muhurta search over the almanac days `from` to `to` at `on`'s
     * place and clock, under the baseline's ranking unless `search` names
     * another, on the context `elect` opens.
     */
    muhurta: ({ place, utcOffsetSeconds }, from, to, search) =>
      elect().almanac.of({ from, to, place, utcOffsetSeconds, muhurta: { ranking: 'BASELINE', ...search } }),
    /** The houses and the Bhava-Chalit. */
    houses: (at) => natal.chart.found({ ...at, houses: true }),
    /** The avasthas, dignities and planetary war. */
    states: (at) => natal.chart.found({ ...at, state: true }),
    /** The drishti. */
    aspects: (at) => natal.chart.found({ ...at, aspects: true }),
    /** The upagrahas and special lagnas. */
    points: (at) => natal.chart.found({ ...at, points: true }),
    /** The Jaimini slice. */
    jaimini: (at) => natal.chart.found({ ...at, jaimini: true }),
    /** The Ayurdaya, the marakas and the longevity windows. */
    longevity: (at) =>
      natal.chart.found({ ...at, rules: { longevity: true, ...(ayurdaya && { ayurdaya }) } }),
    /** The birth's timing: ishtakaal, bhayat, bhabhoga. */
    timing: (at) => natal.chart.found({ ...at, moment: true }),
    avakahada: (at) => natal.chart.found({ ...at, avakahada: true }),
    /** The dossier, its sections or `preset`'s. */
    dossier: (at, dossier) => natal.chart.found({ ...at, dossier }),
    /** The chart's drawings. */
    drawing: (at, drawings) => natal.chart.found({ ...at, drawings }),
    /** What the composers `interpret` names say, as plans. */
    narrative: (at, interpret, rules) => natal.chart.found({ ...at, interpret, ...(rules && { rules }) }),
    /** The Sarvatobhadra chakra at the chart and at `sarvatobhadra.instants`. */
    sarvatobhadra: (at, sarvatobhadra) => natal.chart.found({ ...at, sarvatobhadra }),
    /** The almanac days `from` to `to` with their eclipses. */
    eclipses: ({ place, utcOffsetSeconds }, from, to) => natal.almanac.of({ from, to, place, utcOffsetSeconds, eclipses: true }),
    /** The almanac days `from` to `to` with their lunar years and Nepal Sambat dates. */
    eras: ({ place, utcOffsetSeconds }, from, to) =>
      natal.almanac.of({ from, to, place, utcOffsetSeconds, years: true, nepalSambat: true }),
    /** The Pancha Pakshi days `from` to `to` for `native`. */
    pakshi: ({ place, utcOffsetSeconds }, from, to, native, rules) =>
      natal.almanac.pakshi({ from, to, place, utcOffsetSeconds, native, ...(rules && { rules }) }),
    /** The transit calendar: the hits `hits` asks for. */
    transitCalendar: (at, hits) => natal.chart.found({ ...at, hits }),
    /** The returns from `from` to `to`, the Moon's unless `grahas` names others. */
    returns: (at, from, to, grahas) => natal.chart.found({ ...at, hits: returnsRequest(from, to, grahas) }),
    progressions: (at, progressions) => natal.chart.found({ ...at, progressions }),
    /** The rectification the baseline runs, over `baseline`'s window and events. */
    rectify: (at, baseline) => natal.chart.found({ ...at, rectification: { baseline } }),
    /** The prashna under every reading the baseline takes, on the context `elect` opens. */
    prashna: (at, question) => elect().chart.found({ ...at, prashna: { ...PRASHNA, ...(question && { question }) } }),
    lalkitab: (at, lalkitab = {}) => natal.chart.found({ ...at, lalkitab }),
    /** The remedies under the baseline's functional nature, read at `when` if given. */
    remedies: (at, when) => natal.chart.found({ ...at, remedies: { rules: REMEDIES, ...(when !== undefined && { at: when }) } }),
    /** The synastry with `partner`'s chart. */
    synastry: (at, synastry) => natal.chart.found({ ...at, synastry }),
    /**
     * The rashifal of each sign over `first` to `last`, with the
     * baseline's score for `period` if given; read at `rashifal`'s
     * clock and reporting its event grahas for `period` where the
     * readings name them (C358, C360), the sources' own otherwise.
     */
    rashifal: ({ place, utcOffsetSeconds }, first, last, period) =>
      natal.chart.rashifal(
        {
          first,
          ...(last && { last }),
          place,
          utcOffsetSeconds,
          ...(rashifal && { snapshot: rashifal.snapshot }),
          ...(rashifal && period !== undefined && { events: rashifal.events[period] }),
        },
        ...(period === undefined ? [] : [period]),
      ),
  };
}
