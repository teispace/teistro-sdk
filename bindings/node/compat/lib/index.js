// `@teistro/compat`: the baseline engine's inputs, forwarded to the SDK
// (`docs/03-design/compat-shim.md`).
//
// HAND-WRITTEN. An application on the baseline engine keeps its stored
// birth records and settings snapshots and moves its calls to the SDK one
// at a time: a shim call takes a record and a snapshot as they are, opens
// the SDK under the settings the snapshot means, and answers the SDK's own
// result. Each function is a documented SDK call with its inputs mapped,
// and the package has an end (README, "When it ends").

import { Contexts, calls } from './core.js';

export { CompatRefused } from './core.js';

/**
 * The rows the shim answers, each counted by `calls()` from zero. Filled
 * as the forwarding functions land (§7, items 5 to 7).
 */
const ROWS = Object.freeze([]);

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
