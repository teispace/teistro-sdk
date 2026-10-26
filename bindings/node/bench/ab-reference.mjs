// The A/B's accuracy rungs (`docs/03-design/ab-baseline.md` §11): each
// side's facts held to an answer neither side computed. A rung reads the
// facts a run kept (`Kept`), or asks the sides for an input of its own (a
// worked example's birth, a printed almanac's day), and compares them
// with the reference's in the vocabulary of `ab-agree.mjs`.

import { spawnSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Agreement } from './ab-agree.mjs';

const here = fileURLToPath(new URL('.', import.meta.url));
const root = join(here, '../../..');

/** The grahas a sky places, and the lagna. */
const PLACED = ['SUN', 'MOON', 'MARS', 'MERCURY', 'JUPITER', 'VENUS', 'SATURN', 'RAHU', 'KETU', 'LAGNA'];

/**
 * The rungs, each with its evidence rank (the conformance corpus's: 1 a
 * text or an institution's own print, 2 an established implementation or
 * a source's worked example, 3 another implementation) and the requests it
 * judges. A request no rung judges is compared between the sides alone.
 */
export const RUNGS = {
  'swiss-ephemeris': { rank: 2, requests: ['chart', 'full', 'batch', 'kp', 'gochar'] },
  pyjhora: { rank: 3, requests: ['chart', 'full', 'dashas'] },
  committee: { rank: 1, requests: ['day'] },
  raman: { rank: 2, requests: ['strength'] },
  charak: { rank: 2, requests: ['varsha'] },
  kalaprakasika: { rank: 1, requests: ['milan'] },
};

/**
 * What a run keeps of each side's facts for the rungs that read the
 * corpus: the longitudes, the Vimshottari, the Moon's nakshatra and the
 * poruthams.
 */
export function wanted(request, key) {
  if (key.startsWith('longitude/')) return RUNGS['swiss-ephemeris'].requests.includes(request);
  if (key.startsWith('dashaStart/VIMSHOTTARI/') || key.startsWith('dashaLord/VIMSHOTTARI/')) {
    return RUNGS.pyjhora.requests.includes(request);
  }
  return (request === 'chart' && key === 'nakshatra/MOON') || (request === 'milan' && key.startsWith('porutham/'));
}

/** The facts a run kept, by who answered, request and input. */
export class Kept {
  constructor() {
    /** @type {Map<string, Map<string, Map<string, Map<string, unknown>>>>} */
    this.by = new Map();
  }

  add(who, request, id, facts) {
    const kept = facts.filter(([key]) => wanted(request, key));
    if (kept.length === 0) return;
    if (!this.by.has(who)) this.by.set(who, new Map());
    const requests = this.by.get(who);
    if (!requests.has(request)) requests.set(request, new Map());
    requests.get(request).set(id, new Map(kept));
  }

  get(who, request) {
    return this.by.get(who)?.get(request) ?? new Map();
  }
}

/**
 * Who answers a rung: a side under one convention, with how to ask it.
 *
 * @typedef {{side: string, convention: string, answers: import('./ab-inputs.mjs').Side,
 *   reopened?: (patch: object) => import('./ab-inputs.mjs').Side}} Who
 */

/** The key a who's facts are kept under. */
export const whoKey = (who) => `${who.side} ${who.convention}`;

/** Compares each who's facts with a rung's, as lines of the record. */
function compare(rung, rows) {
  const out = [];
  const byWho = new Map();
  for (const row of rows) {
    const key = whoKey(row.who);
    if (!byWho.has(key)) byWho.set(key, { who: row.who, agreement: new Agreement(['got', 'reference']) });
    const facts = [...row.facts].filter(([k]) => row.reference.has(k));
    byWho.get(key).agreement.add(row.request, row.id, facts, [...row.reference]);
  }
  for (const { who, agreement } of byWho.values()) {
    out.push(...agreement.lines({ kind: 'reference', rung, rank: RUNGS[rung].rank, side: who.side, convention: who.convention }));
  }
  return out;
}

/** The sky a longitude fact was cast in, or none. */
function skyOf(request, key, input, conventions) {
  const [family, ...parts] = key.split('/');
  if (family !== 'longitude') return null;
  const at = (what) => what.split('@');
  if (request === 'chart' || request === 'full' || request === 'batch') {
    const [name, i] = at(parts.join('/'));
    if (!PLACED.includes(name)) return null;
    const jd = i === undefined ? input.jdUt : input.instants[Number(i)];
    return { sky: { jd, ...conventions.chart }, read: (r) => (name === 'LAGNA' ? r.lagna : r.grahas[name]) };
  }
  if (request === 'kp') {
    const [what, name] = parts;
    if (what === 'cusp') return { sky: { jd: input.jdUt, ...conventions.kp }, read: (r) => r.cusps?.[Number(name) - 1] };
    if (what === 'kp' && PLACED.includes(name)) return { sky: { jd: input.jdUt, ...conventions.kp }, read: (r) => r.grahas[name] };
    return null;
  }
  if (request === 'gochar' && parts[0] === 'transit') {
    return { sky: { jd: input.transitAt, ...conventions.gochar }, read: (r) => r.grahas[parts[1]] };
  }
  return null;
}

/** The interpreter and files the Swiss Ephemeris rung needs, refused with a sentence naming them. */
export function swissEphemeris() {
  const python = process.env.TEISTRO_AB_PYTHON;
  if (!python || !process.env.SE_EPHE_PATH) {
    throw new Error('set TEISTRO_AB_PYTHON to a Python with pyswisseph 2.10.03 and SE_EPHE_PATH to the Swiss Ephemeris files');
  }
  return python;
}

/** Places skies through pyswisseph, `ab-swisseph.py`, in one call. */
function placeAll(skies) {
  const run = spawnSync(swissEphemeris(), [join(here, 'ab-swisseph.py')], {
    input: JSON.stringify(skies),
    maxBuffer: 1 << 28,
    encoding: 'utf8',
  });
  if (run.status !== 0) throw new Error(`ab-swisseph.py: ${run.stderr || run.error}`);
  return JSON.parse(run.stdout);
}

/**
 * Each side's longitudes against Swiss Ephemeris at the side's own declared
 * convention (`conventions`), so a difference is the side's arithmetic and
 * not a convention the two chose apart.
 */
export function swissRung(whos, kept, byId) {
  const skies = [];
  const index = new Map();
  const pending = [];
  for (const who of whos) {
    for (const request of RUNGS['swiss-ephemeris'].requests) {
      for (const [id, facts] of kept.get(whoKey(who), request)) {
        const input = byId.get(id);
        for (const key of facts.keys()) {
          const cast = skyOf(request, key, input, who.answers.conventions);
          if (!cast) continue;
          const sky = { ...cast.sky, lat: input.place.latitude, lon: input.place.longitude, alt: input.place.altitude };
          const at = JSON.stringify(sky);
          if (!index.has(at)) {
            index.set(at, skies.length);
            skies.push(sky);
          }
          pending.push({ who, request, id, key, at: index.get(at), read: cast.read, facts });
        }
      }
    }
  }
  const placed = placeAll(skies);
  const rows = new Map();
  for (const p of pending) {
    const value = p.read(placed[p.at]);
    if (value === undefined) continue;
    const at = `${whoKey(p.who)} ${p.request} ${p.id}`;
    if (!rows.has(at)) rows.set(at, { who: p.who, request: p.request, id: p.id, facts: p.facts, reference: new Map() });
    rows.get(at).reference.set(p.key, value);
  }
  return compare('swiss-ephemeris', [...rows.values()]);
}

/** The Gregorian year PyJHora's recorded periods are counted in, and the year both sides count. */
const PYJHORA_YEAR = 'GREGORIAN_YEAR';
const JULIAN_YEAR = 365.25;

/**
 * How PyJHora casts: the Swiss Ephemeris's sidereal longitude, the true
 * ayanamsha taken from the nutated one, where the SDK's root takes the
 * mean. The two Moons stand apart by the nutation in longitude, up to
 * 17″, which moves a twenty-year mahadasha's end by up to three days.
 */
export const PYJHORA = { reading: { frame: { ayanamsha_basis: 'TRUE' } } };

/**
 * Each side's Vimshottari against PyJHora's (rank 3), its Gregorian-year
 * periods rescaled about the birth to the 365.25-day year both sides
 * count: every mahadasha's lord, and every start after the birth's
 * mahadasha, whose antardashas the tool lists from before the birth and
 * the sides from it; and the SDK once more on its default profile told
 * PyJHora's reading, so that what then differs is the sky.
 */
export function pyjhoraRung(whos, kept, byId) {
  const dir = join(root, 'fixtures/pyjhora/vimshottari/charts');
  const recorded = new Map();
  for (const name of readdirSync(dir).filter((n) => n.endsWith('.json'))) {
    const file = JSON.parse(readFileSync(join(dir, name), 'utf8'));
    const birth = file.inputs.jd_ut;
    const { year_days: days, periods } = file.durations[PYJHORA_YEAR];
    const reference = new Map();
    periods.forEach(([maha, antar, start], k) => {
      const i = Math.floor(k / 9);
      const j = k % 9;
      const at = birth + ((start - birth) * JULIAN_YEAR) / days;
      if (j === 0) reference.set(`dashaLord/VIMSHOTTARI/${i}`, maha);
      if (i === 0) return;
      if (j === 0) reference.set(`dashaStart/VIMSHOTTARI/${i}`, at);
      reference.set(`dashaLord/VIMSHOTTARI/${i}.${j}`, antar);
      reference.set(`dashaStart/VIMSHOTTARI/${i}.${j}`, at);
    });
    recorded.set(name.slice(0, name.indexOf('-')), reference);
  }
  const rows = [];
  for (const who of whos) {
    for (const request of RUNGS.pyjhora.requests) {
      for (const [id, facts] of kept.get(whoKey(who), request)) {
        if (recorded.has(id)) rows.push({ who, request, id, facts, reference: recorded.get(id) });
      }
    }
  }
  const timed = whos.find((who) => who.side === 'sdk' && who.convention === 'timed');
  const source = { side: 'sdk', convention: 'source' };
  const side = timed.reopened(PYJHORA.reading);
  for (const [id, reference] of recorded) {
    const input = byId.get(id);
    if (input === undefined) continue;
    const facts = side.facts('dashas', side.dashas(input), input).filter(([key]) => wanted('dashas', key));
    rows.push({ who: source, request: 'dashas', id, facts: new Map(facts), reference });
  }
  return compare('pyjhora', rows);
}

/** Kathmandu as the committee prints for it, on its own clock (+05:45), at sea level. */
const COMMITTEE_OFFSET = 5.75 * 3600;

/** A printed local time on a civil date, hours past 24 running into the next day, as a UT Julian day. */
function printedAt(gregorian, hhmm, offsetSeconds) {
  const [year, month, day] = gregorian.split('-').map(Number);
  const [h, m] = hhmm.split(':').map(Number);
  const midnight = Date.UTC(year, month - 1, day) / 86_400_000 + 2_440_587.5;
  return midnight + (h * 3600 + m * 60 - offsetSeconds) / 86_400;
}

/** The tithi a printed name is, 0 the first of the bright half. */
function tithiIndex(printed) {
  if (printed === 'purnima') return 14;
  if (printed === 'amavasya') return 29;
  const [paksha, n] = printed.split(' ');
  return (paksha === 'shukla' ? 0 : 15) + Number(n) - 1;
}

/** One printed day as an input both sides' `day` reads. */
function committeeDay(place, gregorian) {
  const [year, month, day] = gregorian.split('-').map(Number);
  return {
    id: `npns ${gregorian}`,
    jdUt: printedAt(gregorian, '12:00', COMMITTEE_OFFSET),
    utc: `${gregorian}T06:15:00Z`,
    iana: 'Asia/Kathmandu',
    place: { latitude: place.latitude_deg, longitude: place.longitude_deg, altitude: 0 },
    utcOffsetSeconds: COMMITTEE_OFFSET,
    civilDate: { year, month, day },
  };
}

/**
 * Each side's day against what Nepal's panchanga committee printed (rank
 * 1): the sunrise and sunset of 22 days and the end of 8 tithis, to the
 * printed minute.
 */
export function committeeRung(whos) {
  const book = JSON.parse(readFileSync(join(root, 'fixtures/official/npns-2082-2083.json'), 'utf8'));
  const days = new Map();
  const want = (gregorian) => {
    if (!days.has(gregorian)) days.set(gregorian, { input: committeeDay(book.place, gregorian), arc: null, tithis: [] });
    return days.get(gregorian);
  };
  for (const arc of book.day_arcs) want(arc.gregorian).arc = arc;
  for (const end of book.tithi_ends) want(end.gregorian).tithis.push(end);
  const rows = [];
  for (const who of whos) {
    for (const { input, arc, tithis } of days.values()) {
      const facts = new Map(who.answers.facts('day', who.answers.day(input), input));
      const reference = new Map();
      if (arc) {
        reference.set('event/sunrise', printedAt(arc.gregorian, arc.sunrise, COMMITTEE_OFFSET));
        reference.set('event/sunset', printedAt(arc.gregorian, arc.sunset, COMMITTEE_OFFSET));
      }
      for (const end of tithis) {
        // The span of that tithi in the side's day: its end is the printed one.
        const span = [...facts].find(([key, value]) => key.startsWith('limb/tithi/') && value === tithiIndex(end.tithi));
        const key = span ? `event/tithi/${span[0].split('/')[2]}` : `event/tithi/${end.tithi}`;
        reference.set(key, printedAt(end.gregorian, end.time, COMMITTEE_OFFSET));
      }
      rows.push({ who, request: 'day', id: input.id, facts, reference });
    }
  }
  return compare('committee', rows);
}

/**
 * B.V. Raman's Standard Horoscope (*Graha and Bhava Balas*, Example 56):
 * 16 October 1918, 2h 6m 16s p.m. LMT at 13° N, 77° 35′ E, cast on his
 * ayanamsha, and the Shadbala of each graha in virupas as he prints it.
 */
export const RAMAN = {
  input: {
    id: 'raman',
    jdUt: 2_421_882.5 + (8 + 55 / 60 + 56 / 3600) / 24,
    utc: '1918-10-16T08:55:56Z',
    iana: 'Asia/Kolkata',
    place: { latitude: 13, longitude: 77 + 35 / 60, altitude: 0 },
    utcOffsetSeconds: Math.round(((77 + 35 / 60) / 15) * 3600),
    civilDate: { year: 1918, month: 10, day: 16 },
    ayanamsha: 'RAMAN',
  },
  printed: { SUN: 424.24, MOON: 389.8, MARS: 298.14, MERCURY: 537.02, JUPITER: 433.17, VENUS: 376.15, SATURN: 389.01 },
  // Sripati's reading as Raman works it, where the SDK's default profile
  // reads BPHS instead (`ShadbalaRules::SRIPATI` in the strength crate).
  reading: {
    strength: {
      drekkana: 'MALE_NEUTER_FEMALE',
      kranti: 'HINDU_TABLE',
      luminary_cheshta: 'NONE',
      drik: 'QUARTER',
      required_rupas: 'SRIPATI',
      ishta_kashta: 'SQUARE_ROOTS',
    },
  },
};

/**
 * Each side's Shadbala for Raman's Standard Horoscope against his printed
 * totals; and the SDK once more cast as the source casts, on its default
 * profile told Sripati's reading, so that what then differs is the sky.
 */
export function ramanRung(whos) {
  const ayanamsha = { frame: { ayanamsha: { id: RAMAN.input.ayanamsha, kind: 'CATALOGUED' } } };
  const timed = whos.find((who) => who.side === 'sdk' && who.convention === 'timed');
  const source = { side: 'sdk', convention: 'source', reopened: (patch) => timed.reopened({ ...patch, ...RAMAN.reading }) };
  const rows = [...whos, source].map((who) => {
    const side = who.reopened(ayanamsha);
    const facts = new Map(side.facts('strength', side.strength(RAMAN.input), RAMAN.input));
    const reference = new Map(Object.entries(RAMAN.printed).map(([g, v]) => [`shadbala/${g}/total`, v]));
    return { who, request: 'strength', id: RAMAN.input.id, facts, reference };
  });
  return compare('raman', rows);
}

/**
 * K.S. Charak's worked year (*A Textbook of Varshaphala*, Chart III-1): a
 * birth at Bombay on 20 August 1944 at 07:11 IST, its forty-first year
 * returning at 13:17:29 IST, its five office-bearers and the lord of the
 * year as the source prints them.
 */
export const CHARAK = {
  input: {
    id: 'charak',
    jdUt: 2_431_322.570_138_889,
    utc: '1944-08-20T01:41:00Z',
    iana: 'Asia/Kolkata',
    place: { latitude: 18 + 58 / 60, longitude: 72 + 50 / 60, altitude: 11 },
    utcOffsetSeconds: 19_800,
    civilDate: { year: 1944, month: 8, day: 20 },
    year: 40,
  },
  returnsAt: '13:17:29',
  printed: {
    'office/munthaSign': 8,
    'office/munthaLord': 'JUPITER',
    'office/janmaLagna': 'SUN',
    'office/varshaLagna': 'MARS',
    'office/triRashi': 'MARS',
    'office/dinaRatri': 'SUN',
    'office/yearLord': 'SUN',
  },
};

/** Each side's worked year against Charak's: the return on the side's own date, at the printed time. */
export function charakRung(whos) {
  const rows = whos.map((who) => {
    const { input } = CHARAK;
    const facts = new Map(who.answers.facts('varsha', who.answers.varsha(input), input));
    const reference = new Map(Object.entries(CHARAK.printed));
    const at = facts.get('event/return');
    if (at !== undefined) {
      const [h, m, s] = CHARAK.returnsAt.split(':').map(Number);
      const local = at + input.utcOffsetSeconds / 86_400 + 0.5;
      reference.set('event/return', Math.floor(local) - 0.5 + (h * 3600 + m * 60 + s - input.utcOffsetSeconds) / 86_400);
    }
    return { who, request: 'varsha', id: input.id, facts, reference };
  });
  return compare('charak', rows);
}

/** *Kalaprakasika* XIII p. 75's five rajjus, each its nakshatras, 0 Ashwini. */
const RAJJUS = [
  [0, 8, 9, 17, 18, 26],
  [1, 7, 10, 16, 19, 25],
  [2, 6, 11, 15, 20, 24],
  [3, 5, 12, 14, 21, 23],
  [4, 13, 22],
];

/** The counts from the bride's star to the groom's that Mahendra favours. */
const MAHENDRA = [4, 7, 10, 13, 16, 19, 22, 25];

/**
 * Each side's Rajju, Mahendra and Stree-deergha against *Kalaprakasika*
 * XIII's printed lists, read from the side's own Moons: the rule judged,
 * not the sky.
 */
export function kalaprakasikaRung(whos, kept, byId) {
  const rajju = (star) => RAJJUS.findIndex((stars) => stars.includes(star));
  const rows = [];
  for (const who of whos) {
    const moons = kept.get(whoKey(who), 'chart');
    for (const [id, facts] of kept.get(whoKey(who), 'milan')) {
      const groom = moons.get(id)?.get('nakshatra/MOON');
      const bride = moons.get(byId.get(id).partner.id)?.get('nakshatra/MOON');
      if (groom === undefined || bride === undefined) continue;
      const count = ((groom - bride + 27) % 27) + 1;
      const reference = new Map([
        ['porutham/RAJJU', rajju(bride) !== rajju(groom)],
        ['porutham/MAHENDRA', MAHENDRA.includes(count)],
        ['porutham/STREE_DEERGHA', count > 13],
      ]);
      rows.push({ who, request: 'milan', id, facts, reference });
    }
  }
  return compare('kalaprakasika', rows);
}
