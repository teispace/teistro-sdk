// `@teistro/compat`: the baseline engine's inputs, forwarded to the SDK
// (`docs/03-design/compat-shim.md`).
//
// HAND-WRITTEN. An application on the baseline engine keeps its stored
// birth records and settings snapshots and moves its calls to the SDK one
// at a time: a shim call takes a record and a snapshot as they are, opens
// the SDK under the settings the snapshot means, and answers the SDK's own
// result. Each function is a documented SDK call with its inputs mapped,
// and the package has an end (README, "When it ends").

import { Calendar, at as civilAt } from '@teistro/sdk';

import { Contexts, at, calls, count, readBirth } from './core.js';
import { READINGS, requests } from './forward.js';

export { CompatRefused } from './core.js';

/**
 * The rows the shim answers, each counted by `calls()` from zero. Filled
 * as the forwarding functions land (§7, items 5 to 7).
 */
const ROWS = Object.freeze([
  'chart',
  'full',
  'vargas',
  'yogas',
  'strength',
  'dashas',
  'day',
  'month',
  'milan',
  'varsha',
  'kp',
  'gochar',
  'sadeSati',
  'batch',
]);

/** `options[field]`, or a `TypeError` naming it with an example. */
function required(options, field, example) {
  const value = options?.[field];
  if (value === undefined) throw new TypeError(`options.${field}: required, e.g. ${example}`);
  return value;
}

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
    const asked = requests(natal, () => this.#contexts.kp(snapshot), READINGS);
    return ask(asked, at(natal, read), natal, read);
  }

  /**
   * The almanac's place and clock for `date` (the birth's own civil date
   * when left out): the birth's place, and its zone's offset at noon that
   * day.
   */
  static #dayAt(natal, read, date) {
    const day = date ?? natal.time.civilOf(read.instant, read.zone, Calendar.Gregorian).civil.date;
    const { offsetSeconds } = natal.time.resolve(civilAt(day, { hour: 12 }), read.zone);
    return { date: day, on: { place: read.place, utcOffsetSeconds: offsetSeconds } };
  }

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

  /** The almanac day of `options.date`, the birth's own by default, at the birth's place. */
  day(birth, snapshot, options = {}) {
    return this.#forward('day', birth, snapshot, (r, _, natal, read) => {
      const { date, on } = Shim.#dayAt(natal, read, options.date);
      return r.day(on, date);
    });
  }

  /** The almanac days `options.from` to `options.to`, both included, at the birth's place. */
  month(birth, snapshot, options) {
    const from = required(options, 'from', 'date(Calendar.Gregorian, 2026, 4, 14)');
    const to = required(options, 'to', 'date(Calendar.Gregorian, 2026, 5, 13)');
    return this.#forward('month', birth, snapshot, (r, _, natal, read) => r.month(Shim.#dayAt(natal, read, from).on, from, to));
  }

  /** The birth as the groom, matched with `options.partner`'s stored birth record as the bride. */
  milan(birth, snapshot, options) {
    const partner = required(options, 'partner', "the bride's stored birth record");
    return this.#forward('milan', birth, snapshot, (r, on, natal) => r.milan(on, at(natal, readBirth(partner))));
  }

  /** The annual charts through year `options.through` of life. */
  varsha(birth, snapshot, options) {
    const through = required(options, 'through', '30');
    return this.#forward('varsha', birth, snapshot, (r, on) => r.varsha(on, through));
  }

  /**
   * KP's cusps, lords, significators and ruling planets, on the snapshot's
   * KP context; with `options.anyAyanamsha`, on its natal context under
   * the chart's own ayanamsha (§4.2).
   */
  kp(birth, snapshot, options = {}) {
    return this.#forward('kp', birth, snapshot, (r, on) => r.kp(on, options.anyAyanamsha === true));
  }

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

  /** Whole charts at `options.instants`, UT Julian days, at the birth's place and on its clock. */
  batch(birth, snapshot, options) {
    const instants = required(options, 'instants', '[2447995.5, 2447995.5417]');
    return this.#forward('batch', birth, snapshot, (r, on) => r.batch(on, instants));
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
