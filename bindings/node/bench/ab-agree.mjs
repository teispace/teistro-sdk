// What the two sides of the A/B answer, compared field by field
// (`docs/03-design/ab-baseline.md` §10). Each side reads its own answers
// into facts in one vocabulary, `[key, value]` with the key
// `family/what`: the SDK's reader is `sdkFacts` here, the baseline's is
// the adapter's `facts`. The families say how two values compare:
// continuous ones by their difference, in a stated unit and against a
// stated tolerance, categorical ones by equality.

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Karana, Nakshatra, Rashi, Tithi, Yoga } from '../lib/index.js';

const root = join(fileURLToPath(new URL('.', import.meta.url)), '../../..');

/**
 * How each family of facts compares. A continuous family has a unit, the
 * factor from the facts' own unit to it, and a tolerance in that unit: an
 * arcsecond for a longitude, a minute for an instant of the day (what an
 * almanac prints), a day for a dasha's start, a virupa for a strength.
 *
 * @type {Record<string, {kind: 'angle' | 'number' | 'category', unit?: string, scale?: number, tolerance?: number, what: string}>}
 */
export const FAMILIES = {
  longitude: { kind: 'angle', unit: '″', scale: 3600, tolerance: 1, what: 'sidereal longitudes: grahas, lagna, cusps, sahams, transits' },
  sign: { kind: 'category', what: 'the sign each graha and the lagna stand in' },
  nakshatra: { kind: 'category', what: 'the nakshatra each graha and the lagna stand in' },
  pada: { kind: 'category', what: 'the nakshatra quarter' },
  retrograde: { kind: 'category', what: 'whether a graha is retrograde' },
  varga: { kind: 'category', what: 'the sign each graha and the lagna take in each divisional chart' },
  dashaLord: { kind: 'category', what: 'the lord (graha or sign) of each mahadasha and antardasha' },
  dashaStart: { kind: 'number', unit: 'd', scale: 1, tolerance: 1, what: 'the start of each mahadasha and antardasha' },
  event: { kind: 'number', unit: 's', scale: 86_400, tolerance: 60, what: 'instants: sunrise, sunset, a limb\'s end, a solar return, a Sade Sati phase\'s edge' },
  limb: { kind: 'category', what: 'the tithi, nakshatra, yoga and karana spans of a day' },
  shadbala: { kind: 'number', unit: 'virupas', scale: 1, tolerance: 1, what: 'Shadbala, each of its six parts and the total' },
  bhavaBala: { kind: 'number', unit: 'virupas', scale: 1, tolerance: 1, what: 'Bhava bala, each house\'s total' },
  bindus: { kind: 'category', what: 'Ashtakavarga bindus, each graha\'s and the sarva, and a transit sign\'s in its graha\'s own and in the sarva' },
  yoga: { kind: 'category', what: 'whether each yoga or dosha both catalogues name is present' },
  koota: { kind: 'category', what: 'the points of each of the eight kootas and their total' },
  porutham: { kind: 'category', what: 'whether Rajju, Vedha, Mahendra and Stree-deergha agree' },
  office: { kind: 'category', what: 'the varshaphala\'s muntha sign, its five office-bearers and its year lord' },
  kpLord: { kind: 'category', what: 'the sign, star, sub and sub-sub lords of each cusp and graha' },
  kpSignificator: { kind: 'category', what: 'each house\'s four significator levels, each a set of the nine grahas' },
  kpRuler: { kind: 'category', what: 'the ruling planets: the lagna\'s and the Moon\'s sign and star lords, and the day\'s lord' },
  house: { kind: 'category', what: 'the house a transit stands in, from the Moon' },
  drishti: { kind: 'category', what: 'the grahas each graha aspects in full, counted by sign, as a set of the nine' },
  jaiminiPair: { kind: 'category', what: 'the class of life each of the three pairs gives, and the class they decide' },
  ayurdaya: { kind: 'number', unit: 'y', scale: 1, tolerance: 0.01, what: 'Pindayu, Amsayu and Nisargayu: each graha\'s years before and after the reductions, the lagna\'s, and the total' },
  maraka: { kind: 'category', what: 'the marakas by lordship: the lords of the second and the seventh' },
  balarishta: { kind: 'category', what: 'whether the Moon in the sixth, eighth or twelfth is afflicted, and whether that stands uncancelled' },
  gochar: { kind: 'category', what: 'whether a transit is in a good house from the Moon, whether a good one is obstructed (vedha), and whether its own Ashtakavarga makes it strong' },
  kuja: { kind: 'category', what: 'whether the bride and the groom each carry the Kuja dosha' },
  marriageDosha: { kind: 'category', what: 'whether a match carries the Ashta Koota\'s nadi, bhakoot and gana doshas' },
  muhurta: { kind: 'category', what: 'a marriage muhurta search\'s ranking whole, each ranked window\'s score, and the score of each day one falls on' },
};

/** The grahas both sides place, in the SDK's order, and the lagna. */
const GRAHAS = ['SUN', 'MOON', 'MARS', 'MERCURY', 'JUPITER', 'VENUS', 'SATURN', 'RAHU', 'KETU'];

/** A KP house's four significator levels, the strongest first. */
export const SIGNIFICATOR_LEVELS = ['inOccupantsStars', 'occupants', 'inLordsStar', 'lord'];

/** The ruling planets both sides name, each by why it rules. */
export const RULER_SLOTS = ['LAGNA_SIGN', 'LAGNA_STAR', 'MOON_SIGN', 'MOON_STAR', 'DAY_LORD'];

/** A level's grahas as one comparable value: the nine's, in their order. */
export const grahaSet = (grahas) => GRAHAS.filter((g) => grahas.includes(g)).join(',');

/** A key's last part: `graha.SUN` is `SUN`. */
const bare = (key) => key.slice(key.indexOf('.') + 1);

const SIGNS = Object.values(Rashi);
const NAKSHATRAS = Object.values(Nakshatra);
const TITHIS = Object.values(Tithi);
const YOGAS = Object.values(Yoga);
const KARANAS = Object.values(Karana);

/** The categorical facts a sidereal longitude decides, the same on both sides. */
export function placed(name, deg) {
  const lon = ((deg % 360) + 360) % 360;
  return [
    [`longitude/${name}`, lon],
    [`sign/${name}`, Math.floor(lon / 30)],
    [`nakshatra/${name}`, Math.floor(lon / (40 / 3))],
    [`pada/${name}`, Math.floor(lon / (10 / 3)) % 4],
  ];
}

/**
 * The yogas and doshas both catalogues name, each with the baseline's key
 * or keys for it: the SDK's rule for the baseline's figure where the rules
 * crate's own comparison names one (`crates/rules/tests/same-figures.json`),
 * and the baseline's own rules the SDK ships under their keys.
 *
 * @returns {[string, string[]][]}
 */
export function figures() {
  const read = (path) => JSON.parse(readFileSync(join(root, path), 'utf8'));
  const own = ['computed-doshas.json', 'computed-yogas.json'].flatMap((file) =>
    read(`crates/rules/rules/${file}`).rules.map((rule) => [rule.key, [rule.key]]),
  );
  return [...read('crates/rules/tests/same-figures.json'), ...own];
}

/** Every fact of one chart the positions decide: each graha's and the lagna's. */
function sdkPositions(chart) {
  const out = [];
  for (const placedGraha of chart.grahas) {
    const name = bare(placedGraha.graha);
    if (!GRAHAS.includes(name)) continue;
    out.push(...placed(name, placedGraha.longitudeDeg), [`retrograde/${name}`, placedGraha.retrograde]);
  }
  out.push(...placed('LAGNA', chart.lagnaDeg));
  return out;
}

function sdkVargas(chart) {
  const out = [];
  for (const varga of chart.vargas) {
    const d = bare(varga.varga);
    out.push([`varga/${d}/LAGNA`, SIGNS.indexOf(varga.lagna.sign)]);
    for (const g of varga.grahas) out.push([`varga/${d}/${bare(g.graha)}`, SIGNS.indexOf(g.at.sign)]);
  }
  return out;
}

/** A dasha's mahadashas and antardashas: each lord, and each start but the first. */
function sdkDashas(dashas) {
  const out = [];
  for (const dasha of dashas) {
    const system = bare(dasha.system);
    for (const period of dasha.periods) {
      if (period.level > 2) continue;
      const path = period.path.replace('/', '.');
      const lord = period.sign === null ? bare(period.lord) : SIGNS.indexOf(period.sign);
      out.push([`dashaLord/${system}/${path}`, lord]);
      if (period.path !== '0' && period.path !== '0/0') out.push([`dashaStart/${system}/${path}`, period.from]);
    }
  }
  return out;
}

function sdkStrength(chart) {
  const out = [];
  const sum = (parts) => Object.values(parts).reduce((a, b) => a + b, 0);
  for (const g of chart.shadbala?.grahas ?? []) {
    const name = bare(g.graha);
    const parts = { sthana: sum(g.sthana), dig: g.dig, kaala: sum(g.kaala), cheshta: g.cheshta, naisargika: g.naisargika, drik: g.drik, total: g.virupas };
    for (const [part, value] of Object.entries(parts)) out.push([`shadbala/${name}/${part}`, value]);
  }
  for (const b of chart.bhavaBala?.bhavas ?? []) out.push([`bhavaBala/${b.bhava}`, b.virupas]);
  if (chart.ashtakavarga) {
    for (const g of chart.ashtakavarga.grahas) g.bindus.forEach((n, sign) => out.push([`bindus/${bare(g.graha)}/${sign}`, n]));
    chart.ashtakavarga.sarva.forEach((n, sign) => out.push([`bindus/SARVA/${sign}`, n]));
  }
  return out;
}

/** Each joined figure's presence, from the SDK's present rules. */
function sdkYogas(chart, joined) {
  const present = new Set((chart.rules?.present ?? []).map((p) => p.rule));
  return joined.map(([key]) => [`yoga/${key}`, present.has(key)]);
}

/** The limbs of one almanac day, keyed by `at` (a day of a month, or none). */
function sdkDay(day, at = '') {
  const out = [
    [`event/sunrise${at}`, day.day.sunrise],
    [`event/sunset${at}`, day.day.sunset],
  ];
  const limbs = { tithi: TITHIS, nakshatra: NAKSHATRAS, yoga: YOGAS, karana: KARANAS };
  for (const [limb, members] of Object.entries(limbs)) {
    day[limb].forEach((span, i, spans) => {
      out.push([`limb/${limb}${at}/${i}`, members.indexOf(span.member)]);
      if (i < spans.length - 1) out.push([`event/${limb}${at}/${i}`, span.whole.to]);
    });
  }
  return out;
}

/** Each graha's full aspects, as the set of the grahas they reach. */
function sdkDrishti(chart) {
  const full = new Map(GRAHAS.map((g) => [g, []]));
  for (const d of chart.aspects) if (d.strength === 'FULL') full.get(bare(d.from))?.push(bare(d.to));
  return GRAHAS.map((g) => [`drishti/${g}`, grahaSet(full.get(g))]);
}

/** The three Ayurdaya methods both sides compute, each with its graha givers and the lagna's. */
export const AYURDAYAS = ['pindayu', 'amsayu', 'nisargayu'];

/** Why a graha is a maraka by lordship, as both sides name the house. */
export const MARAKA_LORDS = { 'lord-of-second': 'second', 'lord-of-seventh': 'seventh' };

/** The rule the SDK reads Balarishta's afflicted Moon by (BPHS ch. 9 vv. 3 to 6). */
export const BALARISHTA = 'ARISHTA_MOON_IN_DUSTHANA_MALEFIC_ASPECT';

/** The longevity a full chart's rules hold: the three pairs, the Ayurdayas, the marakas and Balarishta. */
function sdkLongevity(chart) {
  const longevity = chart.rules?.longevity;
  if (!longevity) return [];
  const out = [];
  const pairs = longevity.threePairs;
  if (pairs) {
    for (const p of pairs.pairs) out.push([`jaiminiPair/${p.pair}`, p.class]);
    // The class the pairs decide, before Saturn's or Jupiter's shift.
    out.push(['jaiminiPair/decided', pairs.decided_class]);
  }
  for (const method of AYURDAYAS) {
    const span = longevity.ayurdaya[method];
    for (const c of span.contributions) {
      const giver = typeof c.giver === 'string' ? 'LAGNA' : c.giver.graha;
      if (giver !== 'LAGNA') out.push([`ayurdaya/${method}/${giver}/basic`, c.basic]);
      out.push([`ayurdaya/${method}/${giver}/net`, c.net]);
    }
    out.push([`ayurdaya/${method}/total`, span.years]);
  }
  const lords = Object.fromEntries(Object.values(MARAKA_LORDS).map((house) => [house, []]));
  longevity.marakas.by_graha.forEach((reasons, i) => {
    for (const reason of reasons) if (reason in MARAKA_LORDS) lords[MARAKA_LORDS[reason]].push(GRAHAS[i]);
  });
  for (const [house, grahas] of Object.entries(lords)) out.push([`maraka/${house}`, grahaSet(grahas)]);
  const moon = (chart.rules.present ?? []).find((p) => p.rule === BALARISHTA);
  out.push(['balarishta/present', moon !== undefined], ['balarishta/standing', moon?.result.status === 'active']);
  return out;
}

/** Everything a full chart holds that the baseline's whole chart holds too. */
function sdkFull(chart, joined) {
  return [
    ...sdkPositions(chart),
    ...sdkVargas(chart),
    ...sdkDashas(chart.dashas),
    ...sdkStrength(chart),
    ...sdkYogas(chart, joined),
    ...sdkDrishti(chart),
    ...sdkLongevity(chart),
  ];
}

/** The Ashta Koota's marriage doshas the baseline lists too. */
export const MARRIAGE_DOSHAS = ['NADI', 'BHAKOOT', 'GANA'];

/** The four of the ten South Indian considerations the baseline answers too. */
export const PORUTHAMS = ['RAJJU', 'VEDHA', 'MAHENDRA', 'STREE_DEERGHA'];

/** The varshaphala's five office-bearers: each fact's name, and the SDK's field. */
export const OFFICES = { munthaLord: 'muntha', janmaLagna: 'janmaLagna', varshaLagna: 'varshaLagna', triRashi: 'triRashi', dinaRatri: 'dinaRatri' };

/** Nano-arcseconds, a KP longitude's unit, in degrees. */
const KP_UNIT = 3_600e9;

/**
 * The SDK's facts for one request's answer, in the vocabulary of
 * {@link FAMILIES}.
 *
 * @param {string} request
 * @param {any} answer
 * @param {import('./ab-inputs.mjs').Input} input the input it answered
 * @param {[string, string[]][]} joined {@link figures}
 * @returns {[string, unknown][]}
 */
export function sdkFacts(request, answer, input, joined) {
  switch (request) {
    case 'chart':
      return [...sdkPositions(answer), ...sdkDashas(answer.dashas)];
    case 'day':
      return sdkDay(answer);
    case 'full':
      return sdkFull(answer, joined);
    case 'vargas':
      return sdkVargas(answer);
    case 'yogas':
      return sdkYogas(answer, joined);
    case 'strength':
      return sdkStrength(answer);
    case 'dashas':
      return sdkDashas(answer.dashas);
    case 'month':
      return [...answer].flatMap((day, i) => sdkDay(day, `@${i}`));
    case 'milan': {
      const out = answer.matching.kootas.map((k) => [`koota/${bare(k.reading.koota)}`, k.points]);
      // The rule's own verdict, before p. 76's exception lifts a failed
      // one: the baseline lifts none, and the printed lists are the rule's
      // (§10).
      for (const c of answer.porutham.considerations) {
        const koota = bare(c.reading.koota);
        if (PORUTHAMS.includes(koota)) out.push([`porutham/${koota}`, c.agrees && !c.lifted]);
      }
      // Each side's Kuja dosha as the rules asked count it, after any lift
      // they count; and the Ashta Koota's doshas the match carries, lifted
      // or not, as the baseline lists them.
      out.push(['kuja/BRIDE', answer.kuja.bride.dosha && !answer.kuja.bride.lifted]);
      out.push(['kuja/GROOM', answer.kuja.groom.dosha && !answer.kuja.groom.lifted]);
      const doshas = new Set(answer.marriageDoshas.filter((d) => d.system === 'ASHTA_KOOTA').map((d) => bare(d.koota)));
      for (const koota of MARRIAGE_DOSHAS) out.push([`marriageDosha/${koota}`, doshas.has(koota)]);
      return [...out, ['koota/TOTAL', answer.matching.total]];
    }
    case 'varsha': {
      // The last year asked for: the first, or a worked example's (§11).
      const year = answer.praveshas.at(-1);
      const bearers = year.annual.officeBearers;
      const out = [
        ['event/return', year.instant],
        ['office/munthaSign', SIGNS.indexOf(year.muntha.sign)],
        ['office/yearLord', bare(year.annual.yearLord.graha)],
        ...Object.entries(OFFICES).map(([office, field]) => [`office/${office}`, bare(bearers[field])]),
      ];
      for (const s of year.annual.sahams) out.push([`longitude/saham/${bare(s.saham)}`, s.longitudeDeg]);
      return [...out, ...sdkDashas(year.annual.dashas)];
    }
    case 'kp': {
      const out = [];
      const lords = (what, l) => [
        [`kpLord/${what}/sign`, bare(l.sign)],
        [`kpLord/${what}/star`, bare(l.star.lord)],
        [`kpLord/${what}/sub`, bare(l.sub.lord)],
        [`kpLord/${what}/subSub`, bare(l.subSub.lord)],
      ];
      for (const c of answer.kp.chart.cusps) out.push([`longitude/cusp/${c.house}`, c.longitude / KP_UNIT], ...lords(`cusp/${c.house}`, c.lords));
      for (const p of answer.kp.chart.planets) out.push([`longitude/kp/${bare(p.graha)}`, p.longitude / KP_UNIT], ...lords(bare(p.graha), p.lords));
      for (const h of answer.kp.significators.houses) {
        const levels = [h.inOccupantsStars, h.occupants, h.inLordsStar, [h.lord]];
        levels.forEach((level, n) => out.push([`kpSignificator/${h.house}/${SIGNIFICATOR_LEVELS[n]}`, grahaSet(level.map(bare))]));
      }
      for (const r of answer.kp.ruling.rulers) {
        for (const { kind } of r.reasons) if (RULER_SLOTS.includes(kind)) out.push([`kpRuler/${kind}`, bare(r.graha)]);
      }
      return out;
    }
    case 'gochar': {
      const [at] = answer.gochar;
      const out = [];
      for (const g of at.grahas) {
        const name = bare(g.graha);
        out.push([`longitude/transit/${name}`, SIGNS.indexOf(g.transit.sign) * 30 + g.transit.degrees], [`house/${name}`, g.house]);
        // Phaladeepika ch. 26: v. 2's good houses, and vv. 3 to 8's vedha,
        // which obstructs a good transit only.
        out.push([`gochar/${name}/good`, g.goodHouse]);
        if (g.goodHouse) out.push([`gochar/${name}/vedha`, g.obstructedBy.length > 0]);
      }
      for (const a of at.ashtakavarga) {
        const name = bare(a.graha);
        out.push([`bindus/transit/${name}`, a.bindus], [`bindus/transitSarva/${name}`, a.sarva], [`gochar/${name}/strong`, a.good]);
      }
      return out;
    }
    case 'sadeSati':
      return sdkSadeSati(answer, input.window);
    case 'batch':
      return [...answer].flatMap((chart, i) => sdkFull(chart, joined).map(([key, value]) => [`${key}@${i}`, value]));
    case 'muhurta':
      return [...sdkMuhurta('general', answer.general), ...sdkMuhurta('native', answer.native)];
    default:
      throw new Error(`no facts for ${request}`);
  }
}

/**
 * The baseline's day score, from a window's factors: the day's four
 * clamped on their own, then the shuddhi's, the event's and the native's
 * added and clamped (`muhurta.md` §4.5, steps 1 and 2).
 */
const DAY_PART = new Set(['TITHI_QUALITY', 'NAKSHATRA_SUITABILITY', 'WEEKDAY_SUITABILITY', 'RAHU_KAAL']);
const DAY_REST = new Set([
  'YOGA_SHUDDHI',
  'KARANA_SHUDDHI',
  'PANCHAKA',
  'MUHURTA_YOGA',
  'PAKSHA_BALA',
  'VARA_EVENT',
  'TITHI_EVENT',
  'TARA_BALA',
  'CHANDRA_BALA',
]);
const clamped = (value) => Math.max(0, Math.min(100, value));
const summed = (factors, dimensions) =>
  factors.filter((f) => dimensions.has(f.dimension)).reduce((total, f) => total + f.weight, 0);
const dayScore = (factors) => clamped(clamped(summed(factors, DAY_PART)) + summed(factors, DAY_REST));

/** The baseline's caps on what it ranks: windows a day, and rows. */
const WINDOWS_A_DAY = 2;
const MOST_ROWS = 30;

/** The shortest window the baseline offers: six minutes, in days. */
const SHORTEST = 6 / 1440;

/** A second, in days: a window starting later than its choghadiya by more starts after an avoided kaala. */
const SECOND = 1 / 86_400;

/** A civil date as `YYYY-MM-DD`. */
const iso = ({ year, month, day }) =>
  `${String(year).padStart(4, '0')}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;

/**
 * One search's facts in the baseline's vocabulary (the adapter's
 * `muhurtaFacts`), its windows presented as the baseline presents them.
 * The SDK cuts a day wherever a clause changes and scores each piece; the
 * baseline offers each good choghadiya less the kaala it avoids, scored
 * at its start. So the open, scored windows are joined into the runs a
 * choghadiya holds, by the almanac day and choghadiya each starts in, a
 * run under six minutes dropped, and each run takes its first window's
 * score. Then the baseline's order: the days by their score, highest
 * first and the earlier first among equals; each day's two best runs by
 * the day's score and the choghadiya's weight, the earlier first among
 * equals; and every row by its score, highest first, its place kept
 * among equals, thirty at most.
 */
function sdkMuhurta(kind, almanac) {
  const days = [...almanac].map((d) => ({
    date: iso(d.day.date),
    sunrise: d.day.sunrise,
    next: d.day.nextSunrise,
    choghadiya: d.choghadiya,
  }));
  const runs = [];
  let last = null;
  const open = almanac.muhurta.windows
    .filter((w) => w.barredBy.length === 0 && w.score !== null)
    .sort((a, b) => a.at.from - b.at.from);
  for (const w of open) {
    const at = days.findIndex((d) => d.sunrise <= w.at.from && w.at.from < d.next);
    if (at < 0) continue;
    const ordinal = days[at].choghadiya.findIndex((c) => c.from <= w.at.from && w.at.from < c.to);
    if (
      last !== null &&
      last.day === at &&
      last.ordinal === ordinal &&
      Math.abs(last.to - w.at.from) <= SECOND
    ) {
      last.to = w.at.to;
      continue;
    }
    last = { day: at, ordinal, from: w.at.from, to: w.at.to, window: w };
    runs.push(last);
  }
  const byDay = new Map();
  for (const run of runs) {
    if (run.to - run.from < SHORTEST) continue;
    const factors = run.window.score.factors;
    const day = dayScore(factors);
    const choghadiya = factors.find((f) => f.dimension === 'CHOGHADIYA')?.weight ?? 0;
    const start = days[run.day].choghadiya[run.ordinal]?.from ?? run.from;
    const row = {
      key: `${days[run.day].date}#${run.ordinal}${run.from > start + SECOND ? 'b' : ''}`,
      date: days[run.day].date,
      day,
      first: clamped(Math.round(day + choghadiya)),
      score: run.window.score.value,
    };
    if (!byDay.has(run.day)) byDay.set(run.day, []);
    byDay.get(run.day).push(row);
  }
  const order = [...byDay.keys()].sort((a, b) => byDay.get(b)[0].day - byDay.get(a)[0].day || a - b);
  const rows = order
    .flatMap((at) => [...byDay.get(at)].sort((a, b) => b.first - a.first).slice(0, WINDOWS_A_DAY))
    .sort((a, b) => b.score - a.score)
    .slice(0, MOST_ROWS);
  const out = [
    [`muhurta/${kind}/ranking`, rows.map((r) => `${r.key}=${r.score}`).join(' ')],
    [`muhurta/${kind}/count`, rows.length],
  ];
  for (const r of rows) out.push([`muhurta/${kind}/window/${r.key}`, r.score]);
  for (const date of new Set(rows.map((r) => r.date))) {
    out.push([`muhurta/${kind}/day/${date}`, rows.find((r) => r.date === date).day]);
  }
  return out;
}

/** The house from the Moon each Sade Sati phase is Saturn's transit of. */
const PHASES = { 12: 'rising', 1: 'peak', 2: 'setting' };

/**
 * Each Sade Sati phase that falls in the window, from Saturn's first entry
 * to its last exit, numbered in order: its start and its end, each where
 * the window does not cut it, as the baseline reports a phase.
 */
function sdkSadeSati(answer, { from, to }) {
  const out = [];
  let n = 0;
  for (const cycle of answer.sadeSati.sadeSati) {
    for (const phase of cycle.phases) {
      const start = phase.visits[0].from;
      const end = phase.visits.at(-1).to;
      if (end <= from || start >= to) continue;
      const key = `${PHASES[phase.house]}/${n}`;
      n += 1;
      if (start > from) out.push([`event/sadeSati/${key}/start`, start]);
      if (end < to) out.push([`event/sadeSati/${key}/end`, end]);
    }
  }
  return out;
}

/**
 * The baseline's yoga facts in the joined vocabulary: a figure is present
 * where any of the baseline's keys for it is.
 *
 * @param {[string, unknown][]} facts the adapter's, `yoga/<its key>` among them
 */
export function joinYogas(facts, joined) {
  if (!facts.some(([key]) => key.startsWith('yoga/'))) return facts;
  const present = new Set(facts.filter(([key, value]) => key.startsWith('yoga/') && value).map(([key]) => key.slice(5)));
  const suffix = (key) => key.slice(key.indexOf('@'));
  const rest = facts.filter(([key]) => !key.startsWith('yoga/'));
  // A batch's yogas carry their chart's `@i`; each chart is joined alone.
  const charts = [...new Set(facts.filter(([key]) => key.startsWith('yoga/') && key.includes('@')).map(([key]) => suffix(key)))];
  if (charts.length === 0) {
    return [...rest, ...joined.map(([key, theirs]) => [`yoga/${key}`, theirs.some((k) => present.has(k))])];
  }
  return [
    ...rest,
    ...charts.flatMap((at) => joined.map(([key, theirs]) => [`yoga/${key}${at}`, theirs.some((k) => present.has(`${k}${at}`))])),
  ];
}

/** The family a key belongs to. */
const familyOf = (key) => key.slice(0, key.indexOf('/'));

/** How far apart two values of a continuous family are, in its unit. */
function apart(family, a, b) {
  const { kind, scale } = FAMILIES[family];
  const d = kind === 'angle' ? Math.abs(((a - b + 540) % 360) - 180) : Math.abs(a - b);
  return d * scale;
}

/** The mismatches a family's tally keeps to show. */
const SHOWN = 3;

/** What a mismatch between the two sides calls their values. */
const SIDE_NAMES = ['sdk', 'baseline'];

/**
 * A running comparison of two sets of facts, over every input and request,
 * read out as one line a request and family: a side's against the other
 * side's, or against a reference's. `names` are what a mismatch calls the
 * two values.
 */
export class Agreement {
  constructor(names = SIDE_NAMES) {
    this.names = names;
    /** @type {Map<string, {request: string, family: string, diffs: number[], worst: {id: string, key: string, apart: number} | null, matches: number, mismatches: Record<string, unknown>[], mismatched: number, unpaired: number, inputs: Set<string>}>} */
    this.tallies = new Map();
  }

  tally(request, family) {
    const at = `${request} ${family}`;
    let t = this.tallies.get(at);
    if (!t) {
      t = { request, family, diffs: [], worst: null, matches: 0, mismatches: [], mismatched: 0, unpaired: 0, inputs: new Set() };
      this.tallies.set(at, t);
    }
    return t;
  }

  /** One input's two sets of facts, for one request. */
  add(request, id, facts, others) {
    const theirs = new Map(others);
    const ours = new Map(facts);
    const [a, b] = this.names;
    for (const [key, value] of ours) {
      const family = familyOf(key);
      if (!(family in FAMILIES)) throw new Error(`${request}: ${key} is in no family`);
      const t = this.tally(request, family);
      t.inputs.add(id);
      if (!theirs.has(key)) {
        t.unpaired += 1;
        continue;
      }
      const other = theirs.get(key);
      if (FAMILIES[family].kind === 'category') {
        if (value === other) t.matches += 1;
        else {
          t.mismatched += 1;
          if (t.mismatches.length < SHOWN) t.mismatches.push({ id, key, [a]: value, [b]: other });
        }
      } else {
        const d = apart(family, value, other);
        t.diffs.push(d);
        if (t.worst === null || d > t.worst.apart) t.worst = { id, key, apart: d };
      }
    }
    for (const key of theirs.keys()) {
      if (!ours.has(key) && familyOf(key) in FAMILIES) this.tally(request, familyOf(key)).unpaired += 1;
    }
  }

  /**
   * One line a request and family: a continuous family's median and
   * largest difference and the share within its tolerance, a categorical
   * one's share of exact matches and its first mismatches.
   */
  lines(head) {
    return [...this.tallies.values()].map((t) => {
      const family = FAMILIES[t.family];
      const base = { ...head, request: t.request, family: t.family, inputs: t.inputs.size, unpaired: t.unpaired };
      if (family.kind === 'category') {
        const compared = t.matches + t.mismatched;
        return { ...base, compared, matches: t.matches, share: compared ? t.matches / compared : 0, mismatches: t.mismatches };
      }
      const sorted = [...t.diffs].sort((a, b) => a - b);
      const within = sorted.filter((d) => d <= family.tolerance).length;
      return {
        ...base,
        compared: sorted.length,
        matches: within,
        share: sorted.length ? within / sorted.length : 0,
        unit: family.unit,
        tolerance: family.tolerance,
        median: sorted[sorted.length >> 1] ?? 0,
        max: sorted.at(-1) ?? 0,
        worst: t.worst,
      };
    });
  }
}
