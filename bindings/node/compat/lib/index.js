// `@teistro/compat`: the baseline engine's inputs, forwarded to the SDK
// (`docs/03-design/compat-shim.md`).
//
// HAND-WRITTEN. An application on the baseline engine keeps its stored
// birth records and settings snapshots and moves its calls to the SDK one
// at a time: a shim call takes a record and a snapshot as they are, opens
// the SDK under the settings the snapshot means, and answers the SDK's own
// result. Each function is a documented SDK call with its inputs mapped,
// and the package has an end (README, "When it ends").

import { createRequire } from 'node:module';

import { Calendar, Tithi, at as civilAt } from '@teistro/sdk';

import { CompatRefused, Contexts, at, calls, count, readBirth } from './core.js';
import { MUHURTA_FRAME, READINGS, nativeOf, requests } from './forward.js';

export { CompatRefused } from './core.js';
export { FAMILIES, facts } from './facts.js';

/**
 * The shim's call surface (§3), one row a function: `surface.json`, which
 * `test/surface.test.mjs` holds the exports and the declarations to.
 */
const SURFACE = createRequire(import.meta.url)('../surface.json').rows;

/** The rows the shim answers, each counted by `calls()` from zero. */
const ROWS = Object.freeze(SURFACE.map(({ row }) => row));

/** Each refused row: its guide anchor and what to do instead. */
const REFUSED = new Map(SURFACE.filter(({ kind }) => kind === 'refuse').map((row) => [row.row, row]));

/** A refused row's call: counted, then `CompatRefused` before any computation (§6.1). */
function refuse(row) {
  count(row);
  const { guide, instead } = REFUSED.get(row);
  throw new CompatRefused(row, guide, instead);
}

/** `options[field]`, or a `TypeError` naming it with an example. */
function required(options, field, example) {
  const value = options?.[field];
  if (value === undefined) throw new TypeError(`options.${field}: required, e.g. ${example}`);
  return value;
}

/** `options.from` and `options.to`, the days a range runs over, both included. */
function range(options) {
  return [
    required(options, 'from', 'date(Calendar.Gregorian, 2026, 4, 14)'),
    required(options, 'to', 'date(Calendar.Gregorian, 2027, 4, 13)'),
  ];
}

/** The tithis whose ends are the lunations: the 15th, the full Moon, and the 30th, the new one. */
const LUNATIONS = new Set([Object.values(Tithi)[14], Object.values(Tithi)[29]]);

/** A stored gender as rectification's `sex`, and a stored time accuracy as its `accuracy` (§3.7). */
const SEX = Object.freeze({ male: 'MALE', female: 'FEMALE' });
const ACCURACY = Object.freeze({ exact: 'EXACT', approximate: 'APPROXIMATE', rectified: 'RECTIFIED', unknown: 'UNKNOWN' });

/** A stored name, or a `TypeError` saying which call reads it. */
function named(read, row, whose = 'birth') {
  if (typeof read.name !== 'string' || read.name === '') {
    throw new TypeError(`${whose}.name: ${row} reads the stored name, e.g. 'राम'`);
  }
  return read.name;
}

/** Every plan item of a chart said in its context's locale, by section. */
function said(ctx, plans) {
  const out = {};
  for (const [section, items] of Object.entries(plans ?? {})) {
    if (!Array.isArray(items)) continue;
    out[section] = items.flat().map((item) => ctx.intl.render(item.key, item.params).toString());
  }
  return out;
}

/** A span's member and its whole bounds: where the limb began and where it ends. */
const whole = ({ member, whole: { from, to } }) => ({ member, from, to });

/**
 * The shim over one set of context options, holding the contexts it opens
 * (§2). Made by {@link compat}.
 */
class Shim {
  #contexts;

  /** @param {object} [options] */
  constructor(options) {
    this.#contexts = new Contexts(options);
  }

  /**
   * One forwarded call: the row counted, the birth record read, the
   * snapshot's contexts found, and `ask` given the requests over them
   * (`forward.js`), the birth's chart input on the natal context, that
   * context and the record as read.
   */
  #forward(row, birth, snapshot, ask) {
    count(row);
    const read = readBirth(birth);
    const natal = this.#contexts.natal(snapshot);
    const asked = requests(
      natal,
      () => this.#contexts.kp(snapshot),
      READINGS,
      () => this.#contexts.variant(snapshot, 'muhurta', MUHURTA_FRAME),
    );
    return ask(asked, at(natal, read), natal, read);
  }

  /** One forwarded call that reads no birth: the row counted and the snapshot's natal context given to `ask`. */
  #on(row, snapshot, ask) {
    count(row);
    return ask(this.#contexts.natal(snapshot));
  }

  /**
   * The almanac's place and clock for `date` (the birth's own civil date
   * when left out): the birth's place, and its zone's offset at noon that
   * day.
   */
  static #dayAt(natal, read, date) {
    const day = date ?? Shim.#civil(natal, read);
    const { offsetSeconds } = natal.time.resolve(civilAt(day, { hour: 12 }), read.zone);
    return { date: day, on: { place: read.place, utcOffsetSeconds: offsetSeconds } };
  }

  /** The birth's civil date, on the clock its zone kept then. */
  static #civil(natal, read) {
    return natal.time.civilOf(read.instant, read.zone, Calendar.Gregorian).civil.date;
  }

  // §3.1, the core package.

  /** Positions over a grid, on the snapshot's natal context. */
  positions(snapshot, request) {
    return this.#on('positions', snapshot, (natal) => natal.positions(request));
  }

  /** One of the engine's own operations, on the snapshot's natal context: a Teimeris chain's. */
  engine(snapshot, name, args) {
    return this.#on('engine', snapshot, (natal) => natal.engine.call(name, args));
  }

  /** The ayanamsha at the birth, or at `options.instant`: the member and its offset in degrees. */
  ayanamsha(birth, snapshot, options = {}) {
    return this.#forward('ayanamsha', birth, snapshot, (_, on, natal) => {
      const chart = natal.chart.found({ ...on, ...(options.instant !== undefined && { instant: options.instant }) });
      return { ayanamsha: chart.ayanamsha, offsetDeg: chart.ayanamshaOffsetDeg, custom: chart.ayanamshaCustom };
    });
  }

  /** The sunrise, sunset and next sunrise of `options.date`, the birth's own by default, at the birth's place. */
  sunrise(birth, snapshot, options = {}) {
    return this.#forward('sunrise', birth, snapshot, (r, _, natal, read) => {
      const { date, on } = Shim.#dayAt(natal, read, options.date);
      const { day } = r.day(on, date);
      return { date: day.date, sunrise: day.sunrise, sunset: day.sunset, nextSunrise: day.nextSunrise };
    });
  }

  /** A civil time in a zone, resolved on the snapshot's natal context. */
  resolve(snapshot, civil, zone) {
    return this.#on('resolve', snapshot, (natal) => natal.time.resolve(civil, zone));
  }

  /** Refused: the SDK has no geo port (guide §10). */
  zoneAt() {
    return refuse('zoneAt');
  }

  /**
   * The birth's ghati-pala under both reckonings, each a chart's timing by
   * the reckoning's name: the snapshot's own, and the other's, read on a
   * context with the other `day.ghati_reckoning`.
   */
  ghatiPala(birth, snapshot) {
    return this.#forward('ghatiPala', birth, snapshot, (_, on, natal) => {
      const own = natal.settings.day.ghati_reckoning;
      const other = own === 'CIVIL' ? 'PROPORTIONAL' : 'CIVIL';
      const second = this.#contexts.variant(snapshot, `ghati:${other}`, { day: { ghati_reckoning: other } });
      return { [own]: natal.chart.found(on).timing, [other]: second.chart.found(on).timing };
    });
  }

  /** The calendars on the snapshot's natal context: the Bikram Sambat's conversions, month lengths and leap years among them. */
  bs(snapshot) {
    return this.#on('bs', snapshot, (natal) => natal.calendar);
  }

  /** The lunar years and the Nepal Sambat date of `options.date`, the birth's own by default. */
  eras(birth, snapshot, options = {}) {
    return this.#forward('eras', birth, snapshot, (r, _, natal, read) => {
      const { date, on } = Shim.#dayAt(natal, read, options.date);
      const almanac = r.eras(on, date, date);
      return { years: almanac.years, nepalSambat: almanac.nepalSambat };
    });
  }

  /** A catalogue entity's names in every form, by its key (`graha.SUN`), in the snapshot's locale. */
  entity(snapshot, key) {
    return this.#on('entity', snapshot, (natal) => natal.intl.entity(key));
  }

  /** Refused: formatting is the application's (guide §2). */
  format() {
    return refuse('format');
  }

  // §3.2, the chart package.

  /** The foundation and the Vimshottari. */
  chart(birth, snapshot) {
    return this.#forward('chart', birth, snapshot, (r, on) => r.chart(on));
  }

  /** The whole natal chart. */
  full(birth, snapshot) {
    return this.#forward('full', birth, snapshot, (r, on) => r.full(on));
  }

  /** Every divisional chart, with the foundation. */
  vargas(birth, snapshot) {
    return this.#forward('vargas', birth, snapshot, (r, on) => r.vargas(on));
  }

  /** The yogas and doshas. */
  yogas(birth, snapshot) {
    return this.#forward('yogas', birth, snapshot, (r, on) => r.yogas(on));
  }

  /** The Shadbala, the Bhava bala and the Ashtakavarga. */
  strength(birth, snapshot) {
    return this.#forward('strength', birth, snapshot, (r, on) => r.strength(on));
  }

  /** The eighteen dasha systems. */
  dashas(birth, snapshot) {
    return this.#forward('dashas', birth, snapshot, (r, on) => r.dashas(on));
  }

  /** The houses and the Bhava-Chalit. */
  houses(birth, snapshot) {
    return this.#forward('houses', birth, snapshot, (r, on) => r.houses(on));
  }

  /** The avasthas, dignities and planetary war. */
  states(birth, snapshot) {
    return this.#forward('states', birth, snapshot, (r, on) => r.states(on));
  }

  /** The drishti. */
  aspects(birth, snapshot) {
    return this.#forward('aspects', birth, snapshot, (r, on) => r.aspects(on));
  }

  /** The upagrahas and special lagnas. */
  points(birth, snapshot) {
    return this.#forward('points', birth, snapshot, (r, on) => r.points(on));
  }

  /** The Jaimini slice. */
  jaimini(birth, snapshot) {
    return this.#forward('jaimini', birth, snapshot, (r, on) => r.jaimini(on));
  }

  /** The Ayurdaya under the baseline's readings, the marakas and the longevity windows. */
  longevity(birth, snapshot) {
    return this.#forward('longevity', birth, snapshot, (r, on) => r.longevity(on));
  }

  /** The birth's timing: ishtakaal, bhayat and bhabhoga. */
  timing(birth, snapshot) {
    return this.#forward('timing', birth, snapshot, (r, on) => r.timing(on));
  }

  /** The avakahada. */
  avakahada(birth, snapshot) {
    return this.#forward('avakahada', birth, snapshot, (r, on) => r.avakahada(on));
  }

  /** The birth's civil date in `options.into`, the Bikram Sambat by default. */
  birthCalendar(birth, snapshot, options = {}) {
    return this.#forward('birthCalendar', birth, snapshot, (_, __, natal, read) =>
      natal.calendar.convert(Shim.#civil(natal, read), options.into ?? Calendar.BikramSambat),
    );
  }

  /** The dossier: `options.preset`'s sections, or `options.sections`, read at `options.at` if given. */
  dossier(birth, snapshot, options = {}) {
    return this.#forward('dossier', birth, snapshot, (r, on) => r.dossier(on, options));
  }

  /** Refused: the gochar sidecar is Rust only (guide §10). */
  sidecar() {
    return refuse('sidecar');
  }

  /** The chart's drawings: each `{ layout, varga }` of `options.drawings`. */
  drawing(birth, snapshot, options) {
    const drawings = required(options, 'drawings', "[{ layout: 'chart_layout.NORTH_INDIAN', varga: 'varga.D1' }]");
    return this.#forward('drawing', birth, snapshot, (r, on) => r.drawing(on, drawings));
  }

  /**
   * What the composers `options.interpret` names say of the chart, each
   * plan said in the snapshot's locale, by section, beside the chart that
   * holds the plans. `options.rules` asks for the rule sets a reading
   * composer reads.
   */
  narrative(birth, snapshot, options) {
    const interpret = required(options, 'interpret', '{ placements: true, readings: true }');
    return this.#forward('narrative', birth, snapshot, (r, on, natal) => {
      const chart = r.narrative(on, interpret, options.rules);
      return { chart, text: said(natal, chart.plans) };
    });
  }

  // §3.3, the panchanga package.

  /** The almanac day of `options.date`, the birth's own by default, at the birth's place. */
  day(birth, snapshot, options = {}) {
    return this.#forward('day', birth, snapshot, (r, _, natal, read) => {
      const { date, on } = Shim.#dayAt(natal, read, options.date);
      return r.day(on, date);
    });
  }

  /** The almanac days `options.from` to `options.to`, both included, at the birth's place. */
  month(birth, snapshot, options) {
    const [from, to] = range(options);
    return this.#forward('month', birth, snapshot, (r, _, natal, read) => r.month(Shim.#dayAt(natal, read, from).on, from, to));
  }

  /**
   * The limbs of `options.date`, the birth's own by default: each tithi,
   * nakshatra, yoga and karana span of the day, with where it began and
   * where it ends, which are the previous change and the next.
   */
  changes(birth, snapshot, options = {}) {
    return this.#forward('changes', birth, snapshot, (r, _, natal, read) => {
      const { date, on } = Shim.#dayAt(natal, read, options.date);
      const day = r.day(on, date);
      return {
        tithi: day.tithi.map(whole),
        nakshatra: day.nakshatra.map(whole),
        yoga: day.yoga.map(whole),
        karana: day.karana.map(whole),
      };
    });
  }

  /**
   * A muhurta search over the days `options.from` to `options.to` under
   * the baseline's ranking and the rules `options.search.rules` names, on
   * the frame the baseline searches in, at the birth's place unless
   * `options.at` gives another place and clock. The natal tara and
   * chandra bala read the birth's Moon unless `options.search.native`
   * names another native, or is `null` for none.
   */
  muhurta(birth, snapshot, options) {
    const [from, to] = range(options);
    const search = required(options, 'search', "{ rules: 'BASELINE_MARRIAGE' }");
    return this.#forward('muhurta', birth, snapshot, (r, on, natal, read) => {
      const { native: asked, ...rest } = search;
      const native = asked === undefined ? nativeOf(natal.chart.found(on)) : asked;
      const where = options.at ?? Shim.#dayAt(natal, read, from).on;
      return r.muhurta(where, from, to, native === null ? rest : { ...rest, native });
    });
  }

  /** The Sarvatobhadra chakra at the birth and at `options.instants`, with what `options` asks beyond them. */
  sarvatobhadra(birth, snapshot, options) {
    required(options, 'instants', '[2461000.5]');
    return this.#forward('sarvatobhadra', birth, snapshot, (r, on) => r.sarvatobhadra(on, options));
  }

  /** The almanac days `options.from` to `options.to` with their eclipses, at the birth's place. */
  eclipses(birth, snapshot, options) {
    const [from, to] = range(options);
    return this.#forward('eclipses', birth, snapshot, (r, _, natal, read) =>
      r.eclipses(Shim.#dayAt(natal, read, from).on, from, to),
    );
  }

  /** The Sun's sign ingresses from `options.from` to `options.to`: each day's sankranti, its instant and the sign entered. */
  ingresses(birth, snapshot, options) {
    const [from, to] = range(options);
    return this.#forward('ingresses', birth, snapshot, (r, _, natal, read) =>
      [...r.month(Shim.#dayAt(natal, read, from).on, from, to)]
        .filter((day) => day.sankranti !== null)
        .map((day) => ({ date: day.day.date, instant: day.sankranti, into: day.sunSigns.at(-1)?.member })),
    );
  }

  /**
   * The full and new Moons from `options.from` to `options.to`: the end
   * of each 15th and 30th tithi that ends within one of the days.
   */
  lunations(birth, snapshot, options) {
    const [from, to] = range(options);
    return this.#forward('lunations', birth, snapshot, (r, _, natal, read) => {
      const seen = new Map();
      for (const day of r.month(Shim.#dayAt(natal, read, from).on, from, to)) {
        for (const span of day.tithi) {
          const end = span.whole.to;
          if (LUNATIONS.has(span.member) && end >= day.day.sunrise && end < day.day.nextSunrise) seen.set(end, span.member);
        }
      }
      return [...seen].map(([instant, tithi]) => ({ instant, tithi }));
    });
  }

  /** The Pancha Pakshi days `options.from` to `options.to` for `options.native`, at the birth's place. */
  pakshi(birth, snapshot, options) {
    const [from, to] = range(options);
    const native = required(options, 'native', "{ nakshatra: 'nakshatra.ASHWINI', paksha: 'paksha.SHUKLA' }");
    return this.#forward('pakshi', birth, snapshot, (r, _, natal, read) =>
      r.pakshi(Shim.#dayAt(natal, read, from).on, from, to, native, options.rules),
    );
  }

  // §3.4, the predictive package.

  /** The transits at `options.instants`, UT Julian days, over the birth chart. */
  gochar(birth, snapshot, options) {
    const instants = required(options, 'instants', '[2461000.5]');
    return this.#forward('gochar', birth, snapshot, (r, on) => r.gochar(on, instants));
  }

  /** Sade Sati and the dhaiyyas from `options.from` to `options.to`, UT Julian days. */
  sadeSati(birth, snapshot, options) {
    const from = required(options, 'from', '2447995.5');
    const to = required(options, 'to', '2458953.5');
    return this.#forward('sadeSati', birth, snapshot, (r, on) => r.sadeSati(on, { from, to }));
  }

  /** The transit calendar: the hits `options.hits` asks for. */
  transitCalendar(birth, snapshot, options) {
    const hits = required(options, 'hits', '{ from: 2460676.5, to: 2461041.5 }');
    return this.#forward('transitCalendar', birth, snapshot, (r, on) => r.transitCalendar(on, hits));
  }

  /** The annual charts through year `options.through` of life. */
  varsha(birth, snapshot, options) {
    const through = required(options, 'through', '30');
    return this.#forward('varsha', birth, snapshot, (r, on) => r.varsha(on, through));
  }

  /** The returns from `options.from` to `options.to`, UT Julian days: the Moon's unless `options.grahas` names others. */
  returns(birth, snapshot, options) {
    const from = required(options, 'from', '2461000.5');
    const to = required(options, 'to', '2461365.5');
    return this.#forward('returns', birth, snapshot, (r, on) => r.returns(on, from, to, options.grahas));
  }

  /** The progressions and directions `options` asks for. */
  progressions(birth, snapshot, options = {}) {
    return this.#forward('progressions', birth, snapshot, (r, on) => r.progressions(on, options));
  }

  // §3.5, the applied package.

  /**
   * KP's cusps, lords, significators and ruling planets, on the snapshot's
   * KP context; with `options.anyAyanamsha`, on its natal context under
   * the chart's own ayanamsha (§4.2).
   */
  kp(birth, snapshot, options = {}) {
    return this.#forward('kp', birth, snapshot, (r, on) => r.kp(on, options.anyAyanamsha === true));
  }

  /** The birth as the groom, matched with `options.partner`'s stored birth record as the bride. */
  milan(birth, snapshot, options) {
    const partner = required(options, 'partner', "the bride's stored birth record");
    return this.#forward('milan', birth, snapshot, (r, on, natal) => r.milan(on, at(natal, readBirth(partner))));
  }

  /** The two stored names matched star to star: `options.partner`'s as the bride's, the birth's as the groom's. */
  naam(birth, snapshot, options) {
    const partner = required(options, 'partner', "the bride's stored birth record");
    return this.#forward('naam', birth, snapshot, (_, __, natal, read) =>
      natal.matching.naam(named(readBirth(partner), 'naam', 'partner'), named(read, 'naam'), options.rules),
    );
  }

  /**
   * The rectification the baseline runs, over `options.uncertaintyMinutes`
   * either side of the birth and `options.events`, with the record's
   * gender and time accuracy carried in.
   */
  rectify(birth, snapshot, options) {
    const uncertaintyMinutes = required(options, 'uncertaintyMinutes', '30');
    return this.#forward('rectify', birth, snapshot, (r, on, _, read) => {
      const { events, coverage, dasha } = options;
      return r.rectify(on, {
        uncertaintyMinutes,
        ...(events !== undefined && { events }),
        ...(coverage !== undefined && { coverage }),
        ...(dasha !== undefined && { dasha }),
        ...(SEX[read.gender] !== undefined && { sex: SEX[read.gender] }),
        ...(ACCURACY[read.timeAccuracy] !== undefined && { accuracy: ACCURACY[read.timeAccuracy] }),
      });
    });
  }

  /** The prashna cast at the record's instant and place, under the baseline's timing, mook and score, asking `options.question`. */
  prashna(birth, snapshot, options = {}) {
    return this.#forward('prashna', birth, snapshot, (r, on) => r.prashna(on, options.question));
  }

  /** The Lal Kitab reading, with what `options` asks beyond it. */
  lalkitab(birth, snapshot, options = {}) {
    return this.#forward('lalkitab', birth, snapshot, (r, on) => r.lalkitab(on, options));
  }

  /** The stored name and the birth's civil date, under the baseline's readings unless `options.rules` names others. */
  numerology(birth, snapshot, options = {}) {
    return this.#forward('numerology', birth, snapshot, (_, __, natal, read) => {
      const { year, month, day } = Shim.#civil(natal, read);
      return natal.numerology.profile(named(read, 'numerology'), { year, month, day }, options.rules ?? 'BASELINE');
    });
  }

  /** The remedies under the baseline's functional nature, read at `options.at` if given. */
  remedies(birth, snapshot, options = {}) {
    return this.#forward('remedies', birth, snapshot, (r, on) => r.remedies(on, options.at));
  }

  /** Refused: gemstone safety is not built (the parity table). */
  gemstoneSafety() {
    return refuse('gemstoneSafety');
  }

  /** `options.names` checked against the birth's star and pada, read from its avakahada. */
  nameCheck(birth, snapshot, options) {
    const names = required(options, 'names', "['चूडामणि']");
    return this.#forward('nameCheck', birth, snapshot, (r, on, natal) => {
      const { nakshatra, pada } = r.avakahada(on).avakahada;
      return natal.naming.check(names, { nakshatra, pada }, options.rules);
    });
  }

  /** Refused: no name corpus ships; the application loads its own book (`namakarana.md` §3). */
  nameSuggestions() {
    return refuse('nameSuggestions');
  }

  /** Whole charts at `options.instants`, UT Julian days, at the birth's place and on its clock. */
  batch(birth, snapshot, options) {
    const instants = required(options, 'instants', '[2447995.5, 2447995.5417]');
    return this.#forward('batch', birth, snapshot, (r, on) => r.batch(on, instants));
  }

  /** The synastry with `options.partner`'s stored birth record, with what `options` asks beyond it. */
  synastry(birth, snapshot, options) {
    const partner = required(options, 'partner', "the partner's stored birth record");
    return this.#forward('synastry', birth, snapshot, (r, on, natal) =>
      r.synastry(on, { ...options, partner: at(natal, readBirth(partner)) }),
    );
  }

  /**
   * A study over stored birth records: how often each rule holds in each
   * group of `study.design`; whether the groups differ, with `study.test`;
   * or whether each rule is commoner than chance, with `study.control`.
   */
  research(births, snapshot, study) {
    if (!Array.isArray(births)) throw new TypeError('births: expected the stored birth records of the study');
    if (typeof study !== 'object' || study === null) {
      throw new TypeError("study: required, e.g. { rules: { shipped: ['YOGAS'] }, design: { groups } }");
    }
    return this.#on('research', snapshot, (natal) => {
      const request = { ...study, births: births.map((birth) => at(natal, readBirth(birth))) };
      if (study.control !== undefined) return natal.research.expected(request);
      if (study.test !== undefined) return natal.research.compare(request);
      return natal.research.counts(request);
    });
  }

  // §3.6, the rashifal and interpretation packages.

  /** The rashifal of every sign from `options.first` to `options.last`, at the birth's place, read at six on the clock. */
  rashifal(birth, snapshot, options) {
    const first = required(options, 'first', 'date(Calendar.Gregorian, 2026, 10, 4)');
    return this.#forward('rashifal', birth, snapshot, (r, _, natal, read) =>
      r.rashifal(Shim.#dayAt(natal, read, first).on, first, options.last, options.period),
    );
  }

  /** One message said in the snapshot's locale. */
  reading(snapshot, key, params) {
    return this.#on('reading', snapshot, (natal) => natal.intl.render(key, params));
  }

  /** The milan with `options.partner` as the bride and its doshas' readings said, beside the chart. */
  milanVerdict(birth, snapshot, options) {
    const partner = required(options, 'partner', "the bride's stored birth record");
    return this.#forward('milanVerdict', birth, snapshot, (r, on, natal) => {
      const chart = r.milanVerdict(on, at(natal, readBirth(partner)));
      return { chart, text: said(natal, chart.plans) };
    });
  }

  /**
   * How many times this thread called each row of the shim, zero
   * included. Logged once a day, it says when an application has finished
   * porting: every count is zero (§6.3).
   *
   * @returns {Readonly<Record<string, number>>}
   */
  calls() {
    return calls(ROWS);
  }

  /** Disposes every context the shim opened; a second call does nothing. */
  dispose() {
    this.#contexts.dispose();
  }

  /** The same, for `using`. */
  [Symbol.dispose]() {
    this.dispose();
  }
}

/**
 * The shim, over the options every context it opens shares: the ephemeris
 * chain, the locale when a snapshot stores none, and the rest a `Context`
 * takes but `profile` and `settings`, which each stored snapshot decides.
 *
 * ```js
 * import { compat } from '@teistro/compat';
 * import teimeris from '@teistro/ephemeris-teimeris';
 *
 * const shim = compat({ ephemeris: [teimeris({ dataDir: './ephe' })], locale: 'ne-Deva-NP' });
 * ```
 *
 * It keeps a natal context per distinct translated snapshot, and a KP
 * context opened on first use, at most sixteen snapshots' worth, and
 * disposes the one used longest ago to make room. The cache belongs to the
 * shim, so each worker thread has its own, as the SDK requires.
 *
 * @param {object} [options]
 */
export function compat(options) {
  return new Shim(options);
}
