// Each forward of the shim against the documented SDK call it stands for,
// over every chart of the baseline corpus, to the bit
// (`docs/03-design/compat-shim.md` §6.4): the shim reads the stored record
// and snapshot; this file reads them by the rules §3.7 and §4 write down,
// opens its own contexts, makes the call the shim's row names, and the two
// answers' `provenance.contentHash` must be equal. The hash is taken over
// the canonical bytes, so equal hashes are the same answer. A call one
// side refuses, the other must refuse with the same record.
//
// `cargo xtask check-node` runs it with `@teistro/sdk` resolved to the
// binding beside this package (`sdk.mjs`).

import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { test } from 'node:test';

import { Calendar, Context, DashaSystem, at, date, ianaZone, localMeanZone } from '@teistro/sdk';

import { compat } from '@teistro/compat';
import { BASELINE_DASHAS, FOUNDATION, FULL, READINGS } from '../lib/forward.js';

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

/** Every chart of the corpus as an application stores it: its birth record and its snapshot. */
const CORPUS = readdirSync(CHARTS)
  .filter((name) => name.endsWith('.json'))
  .sort()
  .map((name) => {
    const { id, input, settings_hash: hash } = JSON.parse(readFileSync(new URL(name, CHARTS), 'utf8'));
    return {
      id,
      birth: {
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
 * `recordedSettings` and keeps per snapshot.
 */
class Direct {
  #open = new Map();
  #translator = new Context({ ephemeris: 'NONE' });

  contexts(snapshot) {
    const key = JSON.stringify(snapshot);
    let kept = this.#open.get(key);
    if (kept === undefined) {
      const { natal, kp } = this.#translator.recordedSettings(snapshot);
      const open = (opening) => new Context({ profile: opening.profile, settings: opening.settings, ephemeris: 'BUILTIN' });
      kept = { natal: open(natal), kp: open(kp) };
      this.#open.set(key, kept);
    }
    return kept;
  }

  dispose() {
    for (const { natal, kp } of this.#open.values()) {
      natal.dispose();
      kp.dispose();
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

/** The day thirty days after `day`, Gregorian. */
function later(day, days) {
  const t = new Date(Date.UTC(day.year, day.month - 1, day.day + days));
  return date(Calendar.Gregorian, t.getUTCFullYear(), t.getUTCMonth() + 1, t.getUTCDate());
}

/**
 * Each row: what the shim is asked, and the documented call that answers
 * it, given the record's next one as the partner where a row takes one.
 */
const ROWS = {
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
  day: {
    shim: (s, c) => s.day(c.birth, c.snapshot),
    sdk: ({ natal }, b) =>
      natal.almanac.day({
        date: b.civil.date,
        place: b.chart.place,
        utcOffsetSeconds: dayClock(natal, b.zone, b.civil.date),
      }),
  },
  month: {
    shim: (s, c, b) => s.month(c.birth, c.snapshot, { from: b.civil.date, to: later(b.civil.date, 29) }),
    sdk: ({ natal }, b) =>
      natal.almanac.of({
        from: b.civil.date,
        to: later(b.civil.date, 29),
        place: b.chart.place,
        utcOffsetSeconds: dayClock(natal, b.zone, b.civil.date),
      }),
  },
  milan: {
    shim: (s, c) => s.milan(c.birth, c.snapshot, { partner: c.partner.birth }),
    sdk: ({ natal }, b, c) =>
      natal.chart.found({
        ...b.chart,
        matching: {
          partner: on(natal, c.partner.birth).chart,
          partnerRole: 'BRIDE',
          kuja: READINGS.kuja,
          porutham: READINGS.porutham,
        },
      }),
  },
  varsha: {
    shim: (s, c) => s.varsha(c.birth, c.snapshot, { through: YEARS }),
    sdk: ({ natal }, b) =>
      natal.chart.found({
        ...b.chart,
        varsha: { through: YEARS, place: 'birth', matters: 'all', sahams: 'all', dashas: [DashaSystem.Mudda] },
      }),
  },
  kp: { shim: (s, c) => s.kp(c.birth, c.snapshot), sdk: ({ kp }, b) => kp.chart.found({ ...b.chart, kp: {} }) },
  kpAnyAyanamsha: {
    shim: (s, c) => s.kp(c.birth, c.snapshot, { anyAyanamsha: true }),
    sdk: ({ natal }, b) => natal.chart.found({ ...b.chart, kp: { anyAyanamsha: true } }),
  },
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
};

/** An answer's content hash, or the refusal's record. */
function outcome(call) {
  try {
    return { hash: call().provenance.contentHash };
  } catch (error) {
    return { refused: error.code ?? error.name, field: error.field, message: error.message };
  }
}

test('the shim answers every row as its documented SDK call does, chart for chart, to the bit', () => {
  const shim = compat({ ephemeris: 'BUILTIN' });
  const direct = new Direct();
  try {
    const answered = Object.fromEntries(Object.keys(ROWS).map((row) => [row, 0]));
    CORPUS.forEach((chart, i) => {
      const c = { ...chart, partner: CORPUS[(i + 1) % CORPUS.length] };
      const contexts = direct.contexts(c.snapshot);
      const b = on(contexts.natal, c.birth);
      for (const [row, { shim: ask, sdk }] of Object.entries(ROWS)) {
        const got = outcome(() => ask(shim, c, b));
        const want = outcome(() => sdk(contexts, b, c));
        assert.deepEqual(got, want, `${c.id} ${row}`);
        if (got.hash !== undefined) answered[row] += 1;
      }
    });
    // Agreement on refusals alone proves nothing: every row answers most of the corpus.
    for (const [row, n] of Object.entries(answered)) {
      assert.ok(n >= CORPUS.length - 5, `${row} answered ${n} of ${CORPUS.length}`);
    }
    // Each row called once a chart, KP twice: on its own context and on the natal one.
    const expected = (row) => (row === 'kp' ? 2 : 1) * CORPUS.length;
    assert.deepEqual(
      Object.entries(shim.calls()).filter(([row, n]) => n !== expected(row)),
      [],
    );
  } finally {
    shim.dispose();
    direct.dispose();
  }
});
