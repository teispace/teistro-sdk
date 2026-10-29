// Each forward and compose row of the shim against the documented SDK call
// it stands for, over the charts of the baseline corpus, to the bit
// (`docs/03-design/compat-shim.md` §6.4): the shim reads the stored record
// and snapshot; this file reads them by the rules §3.7 and §4 write down,
// opens its own contexts, makes the call the shim's row names, and the two
// answers must be the same. A sealed answer is compared by its
// `provenance.contentHash`, taken over the canonical bytes, so equal hashes
// are the same answer; an answer the SDK does not seal (a date, a zone's
// resolution, a composed record) by its JSON. A call one side refuses, the
// other must refuse with the same record.
//
// `cargo xtask check-node` runs it with `@teistro/sdk` resolved to the
// binding beside this package (`sdk.mjs`).

import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { test } from 'node:test';

import {
  Body,
  Calendar,
  ChartLayout,
  Context,
  DashaSystem,
  Tithi,
  Varga,
  at,
  date,
  ianaZone,
  localMeanZone,
  returnsRequest,
} from '@teistro/sdk';

import { compat } from '@teistro/compat';
import { BASELINE_DASHAS, FOUNDATION, FULL, READINGS, nativeOf } from '../lib/forward.js';

const CHARTS = new URL('../../../../fixtures/baseline/charts/', import.meta.url);

/** A canonical snapshot's text, `field=value|…`, as the stored fields it lists. */
function snapshotOf(canonical) {
  const fields = {};
  for (const pair of canonical.split('|')) {
    const [field, value] = pair.split('=');
    fields[field] = /^-?\d+$/.test(value) ? Number(value) : value === 'true' ? true : value === 'false' ? false : value;
  }
  return fields;
}

/** Names the corpus's records are given, as the rows that read a name need: IAST, which naam reads too. */
const NAMES = ['Rama', 'Sita', 'Gita', 'Hari', 'Mira', 'Kavi', 'Rupa'];

/** Every chart of the corpus as an application stores it: its birth record and its snapshot. */
const CORPUS = readdirSync(CHARTS)
  .filter((name) => name.endsWith('.json'))
  .sort()
  .map((name, i) => {
    const { id, input, settings_hash: hash } = JSON.parse(readFileSync(new URL(name, CHARTS), 'utf8'));
    return {
      id,
      birth: {
        name: NAMES[i % NAMES.length],
        dateTimeUtc: input.resolved.utc,
        julianDay: input.resolved.jd_ut,
        latitude: input.place.latitude,
        longitude: input.place.longitude,
        altitude: input.place.altitude_m,
        timezone: input.resolved.iana,
        isLmt: input.local.is_lmt,
      },
      snapshot: snapshotOf(hash.canonical),
    };
  });

/**
 * How much the long rows ask for: enough to cross the mapping (more than
 * one chart, year and Saturn sign), few enough for a gate, since what is
 * held is the input mapped, not the computation, which the SDK's own
 * tests hold. Thirty annual charts cost a gate minutes.
 */
const BATCH = 3;
const YEARS = 2;
const SATURN_DAYS = 3 * 365.25;

/**
 * The documented calls, on contexts this file opens from
 * `recordedSettings` and keeps per snapshot, with a context of a patched
 * natal opening where a row names one.
 */
class Direct {
  #open = new Map();
  #translator = new Context({ ephemeris: 'NONE' });

  contexts(snapshot) {
    const key = JSON.stringify(snapshot);
    let kept = this.#open.get(key);
    if (kept === undefined) {
      const { natal, kp, muhurta } = this.#translator.recordedSettings(snapshot);
      const open = (opening) => new Context({ profile: opening.profile, settings: opening.settings, ephemeris: 'BUILTIN' });
      const variants = new Map();
      kept = {
        natal: open(natal),
        kp: open(kp),
        muhurta: open(muhurta),
        variants,
        /** The natal opening with `patch`'s groups laid over its own. */
        variant: (name, patch) => {
          if (!variants.has(name)) {
            const settings = { ...natal.settings };
            for (const [group, fields] of Object.entries(patch)) settings[group] = { ...settings[group], ...fields };
            variants.set(name, open({ profile: natal.profile, settings }));
          }
          return variants.get(name);
        },
      };
      this.#open.set(key, kept);
    }
    return kept;
  }

  dispose() {
    for (const { natal, kp, muhurta, variants } of this.#open.values()) {
      for (const context of variants.values()) context.dispose();
      natal.dispose();
      kp.dispose();
      muhurta.dispose();
    }
    this.#translator.dispose();
  }
}

/**
 * A stored birth as §3.7 reads it: the instant from the stored date, the
 * place, and the clock's offset there and then.
 */
function on(ctx, birth) {
  const instant = 2440587.5 + Date.parse(birth.dateTimeUtc) / 86_400_000;
  const zone = birth.isLmt ? localMeanZone(birth.longitude) : ianaZone(birth.timezone);
  const { civil, resolution } = ctx.time.civilOf(instant, zone, Calendar.Gregorian);
  const place = { latitude: birth.latitude, longitude: birth.longitude, altitude: birth.altitude };
  return { chart: { instant, place, utcOffsetSeconds: resolution.offsetSeconds }, civil, zone };
}

/** The almanac's clock on `day` at a birth's place: its zone's offset at noon. */
function dayClock(ctx, zone, day) {
  return ctx.time.resolve(at(day, { hour: 12 }), zone).offsetSeconds;
}

/** The almanac's place and clock on `day` at a birth's place. */
function dayAt(ctx, b, day) {
  return { place: b.chart.place, utcOffsetSeconds: dayClock(ctx, b.zone, day) };
}

/** The day `days` after `day`, Gregorian. */
function later(day, days) {
  const t = new Date(Date.UTC(day.year, day.month - 1, day.day + days));
  return date(Calendar.Gregorian, t.getUTCFullYear(), t.getUTCMonth() + 1, t.getUTCDate());
}

/** Every plan item said in `ctx`'s locale, by section. */
function said(ctx, plans) {
  const out = {};
  for (const [section, items] of Object.entries(plans ?? {})) {
    if (Array.isArray(items)) out[section] = items.flat().map((i) => ctx.intl.render(i.key, i.params).toString());
  }
  return out;
}

/** The milan's matching request, as §3.5's row and the A/B ask it. */
const matching = (natal, c) => ({
  partner: on(natal, c.partner.birth).chart,
  partnerRole: 'BRIDE',
  kuja: READINGS.kuja,
  porutham: READINGS.porutham,
});

/** A span's member and its whole bounds. */
const whole = ({ member, whole: { from, to } }) => ({ member, from, to });

/** The composers the narrative row is asked for here. */
const INTERPRET = { placements: true, houses: true };

/** A muhurta search a week long, the baseline's marriage rules. */
const SEARCH = { rules: 'BASELINE_MARRIAGE' };

/** Three births of the corpus as a study's, and its groups. */
const STUDY = { rules: { shipped: ['YOGAS'] }, design: { groups: [0, 1, 0] } };

/**
 * The rows the A/B times, asked of every chart, as they were when the A/B
 * moved here (item 5). The rest are asked of one chart in {@link SPARSE}
 * unless they name their own `every`: what is held is the input mapped,
 * which a sample of the corpus's snapshots and places crosses, and every
 * chart of every row would cost a gate two minutes.
 */
const TIMED = new Set(['chart', 'full', 'vargas', 'yogas', 'strength', 'dashas', 'day', 'month', 'milan', 'varsha', 'kp', 'kpAnyAyanamsha', 'gochar', 'sadeSati', 'batch']);
const SPARSE = 5;

/**
 * Each row: what the shim is asked, and the documented call that answers
 * it, given the record's next one as the partner where a row takes one.
 * `every` asks a costly row of one chart in that many; `refused` marks a
 * row the built-in ephemeris cannot answer, held to the same refusal.
 */
const ROWS = {
  // §3.1
  positions: {
    shim: (s, c, b) => s.positions(c.snapshot, { instants: [b.chart.instant], bodies: [Body.Sun, Body.Moon] }),
    sdk: ({ natal }, b) => natal.positions({ instants: [b.chart.instant], bodies: [Body.Sun, Body.Moon] }),
  },
  engine: {
    refused: true,
    shim: (s, c) => s.engine(c.snapshot, 'julday', { year: 2000, month: 1, day: 1, hour: 12 }),
    sdk: ({ natal }) => natal.engine.call('julday', { year: 2000, month: 1, day: 1, hour: 12 }),
  },
  ayanamsha: {
    shim: (s, c) => s.ayanamsha(c.birth, c.snapshot),
    sdk: ({ natal }, b) => {
      const chart = natal.chart.found(b.chart);
      return { ayanamsha: chart.ayanamsha, offsetDeg: chart.ayanamshaOffsetDeg, custom: chart.ayanamshaCustom };
    },
  },
  sunrise: {
    shim: (s, c) => s.sunrise(c.birth, c.snapshot),
    sdk: ({ natal }, b) => {
      const { day } = natal.almanac.day({ date: b.civil.date, ...dayAt(natal, b, b.civil.date) });
      return { date: day.date, sunrise: day.sunrise, sunset: day.sunset, nextSunrise: day.nextSunrise };
    },
  },
  resolve: {
    shim: (s, c, b) => s.resolve(c.snapshot, at(b.civil.date, { hour: 12 }), b.zone),
    sdk: ({ natal }, b) => natal.time.resolve(at(b.civil.date, { hour: 12 }), b.zone),
  },
  ghatiPala: {
    shim: (s, c) => s.ghatiPala(c.birth, c.snapshot),
    sdk: ({ natal, variant }, b) => {
      const own = natal.settings.day.ghati_reckoning;
      const other = own === 'CIVIL' ? 'PROPORTIONAL' : 'CIVIL';
      const second = variant(`ghati:${other}`, { day: { ghati_reckoning: other } });
      return { [own]: natal.chart.found(b.chart).timing, [other]: second.chart.found(b.chart).timing };
    },
  },
  bs: {
    shim: (s, c, b) => s.bs(c.snapshot).convert(b.civil.date, Calendar.BikramSambat),
    sdk: ({ natal }, b) => natal.calendar.convert(b.civil.date, Calendar.BikramSambat),
  },
  eras: {
    shim: (s, c) => s.eras(c.birth, c.snapshot),
    sdk: ({ natal }, b) => {
      const d = b.civil.date;
      const almanac = natal.almanac.of({ from: d, to: d, ...dayAt(natal, b, d), years: true, nepalSambat: true });
      return { years: almanac.years, nepalSambat: almanac.nepalSambat };
    },
  },
  entity: {
    shim: (s, c) => s.entity(c.snapshot, 'graha.SUN'),
    sdk: ({ natal }) => natal.intl.entity('graha.SUN'),
  },
  // §3.2
  chart: {
    shim: (s, c) => s.chart(c.birth, c.snapshot),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, ...FOUNDATION, dashas: [DashaSystem.Vimshottari] }),
  },
  full: {
    shim: (s, c) => s.full(c.birth, c.snapshot),
    sdk: ({ natal }, b) =>
      natal.chart.found({ ...b.chart, ...FULL, rules: { ...FULL.rules, ayurdaya: READINGS.ayurdaya } }),
  },
  vargas: { shim: (s, c) => s.vargas(c.birth, c.snapshot), sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, ...FOUNDATION }) },
  yogas: {
    shim: (s, c) => s.yogas(c.birth, c.snapshot),
    sdk: ({ natal }, b) =>
      natal.chart.found({ ...b.chart, rules: { shipped: ['NABHASAS', 'ARISHTAS', 'GANDANTAS', 'DOSHAS', 'YOGAS'] } }),
  },
  strength: {
    shim: (s, c) => s.strength(c.birth, c.snapshot),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, shadbala: true, bhavaBala: true, ashtakavarga: true }),
  },
  dashas: {
    shim: (s, c) => s.dashas(c.birth, c.snapshot),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, dashas: BASELINE_DASHAS }),
  },
  houses: { shim: (s, c) => s.houses(c.birth, c.snapshot), sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, houses: true }) },
  states: { shim: (s, c) => s.states(c.birth, c.snapshot), sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, state: true }) },
  aspects: { shim: (s, c) => s.aspects(c.birth, c.snapshot), sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, aspects: true }) },
  points: { shim: (s, c) => s.points(c.birth, c.snapshot), sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, points: true }) },
  jaimini: { shim: (s, c) => s.jaimini(c.birth, c.snapshot), sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, jaimini: true }) },
  longevity: {
    shim: (s, c) => s.longevity(c.birth, c.snapshot),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, rules: { longevity: true, ayurdaya: READINGS.ayurdaya } }),
  },
  timing: { shim: (s, c) => s.timing(c.birth, c.snapshot), sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, moment: true }) },
  avakahada: {
    shim: (s, c) => s.avakahada(c.birth, c.snapshot),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, avakahada: true }),
  },
  birthCalendar: {
    shim: (s, c) => s.birthCalendar(c.birth, c.snapshot),
    sdk: ({ natal }, b) => natal.calendar.convert(b.civil.date, Calendar.BikramSambat),
  },
  dossier: {
    shim: (s, c) => s.dossier(c.birth, c.snapshot, { sections: ['CHART', 'GRAHAS'] }),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, dossier: { sections: ['CHART', 'GRAHAS'] } }),
  },
  drawing: {
    shim: (s, c) => s.drawing(c.birth, c.snapshot, { drawings: [{ layout: ChartLayout.SouthIndian, varga: Varga.D9 }] }),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, drawings: [{ layout: ChartLayout.SouthIndian, varga: Varga.D9 }] }),
  },
  narrative: {
    shim: (s, c) => s.narrative(c.birth, c.snapshot, { interpret: INTERPRET }),
    sdk: ({ natal }, b) => {
      const chart = natal.chart.found({ ...b.chart, interpret: INTERPRET });
      return { chart, text: said(natal, chart.plans) };
    },
  },
  // §3.3
  day: {
    shim: (s, c) => s.day(c.birth, c.snapshot),
    sdk: ({ natal }, b) => natal.almanac.day({ date: b.civil.date, ...dayAt(natal, b, b.civil.date) }),
  },
  month: {
    shim: (s, c, b) => s.month(c.birth, c.snapshot, { from: b.civil.date, to: later(b.civil.date, 29) }),
    sdk: ({ natal }, b) => natal.almanac.of({ from: b.civil.date, to: later(b.civil.date, 29), ...dayAt(natal, b, b.civil.date) }),
  },
  changes: {
    shim: (s, c) => s.changes(c.birth, c.snapshot),
    sdk: ({ natal }, b) => {
      const day = natal.almanac.day({ date: b.civil.date, ...dayAt(natal, b, b.civil.date) });
      return { tithi: day.tithi.map(whole), nakshatra: day.nakshatra.map(whole), yoga: day.yoga.map(whole), karana: day.karana.map(whole) };
    },
  },
  muhurta: {
    shim: (s, c, b) => s.muhurta(c.birth, c.snapshot, { from: b.civil.date, to: later(b.civil.date, 6), search: SEARCH }),
    sdk: ({ natal, muhurta }, b) =>
      muhurta.almanac.of({
        from: b.civil.date,
        to: later(b.civil.date, 6),
        ...dayAt(natal, b, b.civil.date),
        muhurta: { ranking: 'BASELINE', ...SEARCH, native: nativeOf(natal.chart.found(b.chart)) },
      }),
  },
  sarvatobhadra: {
    shim: (s, c, b) => s.sarvatobhadra(c.birth, c.snapshot, { instants: [b.chart.instant + 365.25] }),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, sarvatobhadra: { instants: [b.chart.instant + 365.25] } }),
  },
  eclipses: {
    shim: (s, c, b) => s.eclipses(c.birth, c.snapshot, { from: b.civil.date, to: later(b.civil.date, 29) }),
    sdk: ({ natal }, b) =>
      natal.almanac.of({ from: b.civil.date, to: later(b.civil.date, 29), ...dayAt(natal, b, b.civil.date), eclipses: true }),
  },
  ingresses: {
    shim: (s, c, b) => s.ingresses(c.birth, c.snapshot, { from: b.civil.date, to: later(b.civil.date, 29) }),
    sdk: ({ natal }, b) =>
      [...natal.almanac.of({ from: b.civil.date, to: later(b.civil.date, 29), ...dayAt(natal, b, b.civil.date) })]
        .filter((day) => day.sankranti !== null)
        .map((day) => ({ date: day.day.date, instant: day.sankranti, into: day.sunSigns.at(-1)?.member })),
  },
  lunations: {
    shim: (s, c, b) => s.lunations(c.birth, c.snapshot, { from: b.civil.date, to: later(b.civil.date, 29) }),
    sdk: ({ natal }, b) => {
      const ends = new Set([Object.values(Tithi)[14], Object.values(Tithi)[29]]);
      const out = new Map();
      for (const day of natal.almanac.of({ from: b.civil.date, to: later(b.civil.date, 29), ...dayAt(natal, b, b.civil.date) })) {
        for (const span of day.tithi) {
          if (ends.has(span.member) && span.whole.to >= day.day.sunrise && span.whole.to < day.day.nextSunrise) {
            out.set(span.whole.to, span.member);
          }
        }
      }
      return [...out].map(([instant, tithi]) => ({ instant, tithi }));
    },
  },
  pakshi: {
    shim: (s, c, b) =>
      s.pakshi(c.birth, c.snapshot, {
        from: b.civil.date,
        to: later(b.civil.date, 1),
        native: { nakshatra: 'nakshatra.ASHWINI', paksha: 'paksha.SHUKLA' },
      }),
    sdk: ({ natal }, b) =>
      natal.almanac.pakshi({
        from: b.civil.date,
        to: later(b.civil.date, 1),
        ...dayAt(natal, b, b.civil.date),
        native: { nakshatra: 'nakshatra.ASHWINI', paksha: 'paksha.SHUKLA' },
      }),
  },
  // §3.4
  gochar: {
    shim: (s, c, b) => s.gochar(c.birth, c.snapshot, { instants: [b.chart.instant + 10_957.5] }),
    sdk: ({ natal }, b) =>
      natal.chart.found({
        ...b.chart,
        ashtakavarga: true,
        gochar: { instants: [b.chart.instant + 10_957.5], ashtakavarga: true },
      }),
  },
  sadeSati: {
    shim: (s, c, b) => s.sadeSati(c.birth, c.snapshot, { from: b.chart.instant, to: b.chart.instant + SATURN_DAYS }),
    sdk: ({ natal }, b) =>
      natal.chart.found({ ...b.chart, sadeSati: { from: b.chart.instant, to: b.chart.instant + SATURN_DAYS } }),
  },
  transitCalendar: {
    shim: (s, c, b) => s.transitCalendar(c.birth, c.snapshot, { hits: { from: b.chart.instant, to: b.chart.instant + 60 } }),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, hits: { from: b.chart.instant, to: b.chart.instant + 60 } }),
  },
  varsha: {
    shim: (s, c) => s.varsha(c.birth, c.snapshot, { through: YEARS }),
    sdk: ({ natal }, b) =>
      natal.chart.found({
        ...b.chart,
        varsha: { through: YEARS, place: 'birth', matters: 'all', sahams: 'all', dashas: [DashaSystem.Mudda] },
      }),
  },
  returns: {
    shim: (s, c, b) => s.returns(c.birth, c.snapshot, { from: b.chart.instant + 1, to: b.chart.instant + 60 }),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, hits: returnsRequest(b.chart.instant + 1, b.chart.instant + 60) }),
  },
  progressions: {
    shim: (s, c, b) => s.progressions(c.birth, c.snapshot, { at: b.chart.instant + 30 * 365.25 }),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, progressions: { at: b.chart.instant + 30 * 365.25 } }),
  },
  // §3.5
  kp: { shim: (s, c) => s.kp(c.birth, c.snapshot), sdk: ({ kp }, b) => kp.chart.found({ ...b.chart, kp: {} }) },
  kpAnyAyanamsha: {
    shim: (s, c) => s.kp(c.birth, c.snapshot, { anyAyanamsha: true }),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, kp: { anyAyanamsha: true } }),
  },
  milan: {
    shim: (s, c) => s.milan(c.birth, c.snapshot, { partner: c.partner.birth }),
    sdk: ({ natal }, b, c) => natal.chart.found({ ...b.chart, matching: matching(natal, c) }),
  },
  naam: {
    shim: (s, c) => s.naam(c.birth, c.snapshot, { partner: c.partner.birth, rules: { name: { latin: 'IAST' } } }),
    sdk: ({ natal }, b, c) => natal.matching.naam(c.partner.birth.name, c.birth.name, { name: { latin: 'IAST' } }),
  },
  rectify: {
    every: 11,
    shim: (s, c) => s.rectify(c.birth, c.snapshot, { uncertaintyMinutes: 10 }),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, rectification: { baseline: { uncertaintyMinutes: 10 } } }),
  },
  prashna: {
    shim: (s, c) => s.prashna(c.birth, c.snapshot),
    // Cast as the baseline casts a question's chart, on the muhurta
    // opening's frame: the Earth's centre and the true node (C800).
    sdk: ({ muhurta }, b) => muhurta.chart.found({ ...b.chart, prashna: { rules: 'BASELINE' } }),
  },
  lalkitab: { shim: (s, c) => s.lalkitab(c.birth, c.snapshot), sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, lalkitab: {} }) },
  numerology: {
    shim: (s, c) => s.numerology(c.birth, c.snapshot),
    sdk: ({ natal }, b, c) => {
      const { year, month, day } = b.civil.date;
      return natal.numerology.profile(c.birth.name, { year, month, day }, 'BASELINE');
    },
  },
  remedies: {
    shim: (s, c) => s.remedies(c.birth, c.snapshot),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, remedies: { rules: { functional: { scheme: 'BASELINE' } } } }),
  },
  nameCheck: {
    shim: (s, c) => s.nameCheck(c.birth, c.snapshot, { names: ['राम', 'चूडामणि'] }),
    sdk: ({ natal }, b) => {
      const { nakshatra, pada } = natal.chart.found({ ...b.chart, avakahada: true }).avakahada;
      return natal.naming.check(['राम', 'चूडामणि'], { nakshatra, pada });
    },
  },
  batch: {
    shim: (s, c, b) =>
      s.batch(c.birth, c.snapshot, { instants: Array.from({ length: BATCH }, (_, h) => b.chart.instant + h / 24) }),
    sdk: ({ natal }, b) =>
      natal.chart.foundMany({
        instants: Array.from({ length: BATCH }, (_, h) => b.chart.instant + h / 24),
        place: b.chart.place,
        utcOffsetSeconds: b.chart.utcOffsetSeconds,
        ...FULL,
        rules: { ...FULL.rules, ayurdaya: READINGS.ayurdaya },
      }),
  },
  synastry: {
    shim: (s, c) => s.synastry(c.birth, c.snapshot, { partner: c.partner.birth }),
    sdk: ({ natal }, b, c) => natal.chart.found({ ...b.chart, synastry: { partner: on(natal, c.partner.birth).chart } }),
  },
  research: {
    every: 11,
    shim: (s, c) => s.research(c.study.map((d) => d.birth), c.snapshot, STUDY),
    sdk: ({ natal }, b, c) => natal.research.counts({ ...STUDY, births: c.study.map((d) => on(natal, d.birth).chart) }),
  },
  // §3.6
  rashifal: {
    shim: (s, c, b) => s.rashifal(c.birth, c.snapshot, { first: b.civil.date, last: later(b.civil.date, 6) }),
    sdk: ({ natal }, b) =>
      natal.chart.rashifal({
        first: b.civil.date,
        last: later(b.civil.date, 6),
        ...dayAt(natal, b, b.civil.date),
        snapshot: { at: 'CLOCK', hour: 6, minute: 0 },
      }),
  },
  reading: {
    shim: (s, c) => s.reading(c.snapshot, 'sdk.reason.grahaInBhava', { graha: { $entity: 'graha.JUPITER' }, bhava: 7 }),
    sdk: ({ natal }) => natal.intl.render('sdk.reason.grahaInBhava', { graha: { $entity: 'graha.JUPITER' }, bhava: 7 }),
  },
  milanVerdict: {
    shim: (s, c) => s.milanVerdict(c.birth, c.snapshot, { partner: c.partner.birth }),
    sdk: ({ natal }, b, c) => {
      const chart = natal.chart.found({
        ...b.chart,
        matching: matching(natal, c),
        rules: { shipped: ['DOSHAS'] },
        interpret: { readings: true },
      });
      return { chart, text: said(natal, chart.plans) };
    },
  },
};

/** JSON of a value the SDK does not seal: typed arrays as arrays, bigints as text. */
const json = (value) =>
  JSON.stringify(value, (_, v) => (typeof v === 'bigint' ? `${v}n` : ArrayBuffer.isView(v) ? [...v] : v));

/** An answer's identity: its content hash when sealed, a composed record's parts, or its JSON. */
function digest(answer) {
  const hash = answer?.provenance?.contentHash;
  if (typeof hash === 'string') return hash;
  if (answer?.chart?.provenance !== undefined) return `${answer.chart.provenance.contentHash} ${json(answer.text)}`;
  if (typeof answer?.toString === 'function' && answer.constructor?.name === 'Rendered') return `${answer}`;
  return json(answer);
}

/** An answer's identity, or the refusal's record. */
function outcome(call) {
  try {
    return { answer: digest(call()) };
  } catch (error) {
    return { refused: error.code ?? error.name, field: error.field, message: error.message };
  }
}

test('the shim answers every row as its documented SDK call does, chart for chart, to the bit', () => {
  const shim = compat({ ephemeris: 'BUILTIN' });
  const direct = new Direct();
  try {
    const asked = Object.fromEntries(Object.keys(ROWS).map((row) => [row, 0]));
    const answered = { ...asked };
    CORPUS.forEach((chart, i) => {
      const c = {
        ...chart,
        partner: CORPUS[(i + 1) % CORPUS.length],
        study: [0, 1, 2].map((k) => CORPUS[(i + k) % CORPUS.length]),
      };
      const contexts = direct.contexts(c.snapshot);
      const b = on(contexts.natal, c.birth);
      for (const [row, { shim: ask, sdk, every = TIMED.has(row) ? 1 : SPARSE }] of Object.entries(ROWS)) {
        if (i % every !== 0) continue;
        asked[row] += 1;
        const got = outcome(() => ask(shim, c, b));
        const want = outcome(() => sdk(contexts, b, c));
        assert.deepEqual(got, want, `${c.id} ${row}`);
        if (got.answer !== undefined) answered[row] += 1;
      }
    });
    // Agreement on refusals alone proves nothing: every row answers most of
    // what it is asked, but the engine's own operations, which the
    // built-in ephemeris does not have.
    for (const [row, n] of Object.entries(answered)) {
      if (ROWS[row].refused) assert.equal(n, 0, `${row} answered on the built-in ephemeris`);
      else assert.ok(n >= asked[row] - Math.ceil(asked[row] / 10), `${row} answered ${n} of ${asked[row]}`);
    }
    // Each row called once a chart asked, KP twice: on its own context and on the natal one.
    const expected = (row) => (row === 'kp' ? asked.kp + asked.kpAnyAyanamsha : asked[row] ?? 0);
    assert.deepEqual(
      Object.entries(shim.calls()).filter(([row, n]) => n !== expected(row)),
      [],
    );
  } finally {
    shim.dispose();
    direct.dispose();
  }
});
