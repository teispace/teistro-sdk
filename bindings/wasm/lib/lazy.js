/**
 * The package's modules loaded when they are asked for, the calendar first
 * and the jyotish later (`03-design/wasm-lazy-loading.md`).
 *
 * Importing this file loads no module and none of the layer: each function
 * below is a dynamic `import()` of an entry the package already has, so a
 * bundler splits each into a chunk of its own, and a page that only ever
 * asks for the calendar never fetches the full module. Every host's loader
 * is the one its condition already picks.
 *
 * The full module holds the calendar too, so a page that needs a chart may
 * ask for it alone. Both may be loaded, but they are two modules: a context
 * or a value of one is nothing to the other, and each has its own
 * `TeistroError`, so read a refusal's `status`, not its class.
 */

export { INTERNAL_CODE, onInternalError } from './internal.js';

/**
 * One load, whoever asks and however often. A load that failed stays
 * failed: a module whose evaluation rejected is held so by the host, so an
 * `import()` of it again answers the same rejection without fetching
 * (measured in Chrome, `03-design/wasm-lazy-loading.md` C890), and only a
 * new page can load it.
 */
function once(what, load) {
  let loading;
  return () => {
    loading ??= load().catch((cause) => {
      throw new Error(`the ${what} module did not load (${cause?.message ?? cause}); load the page again`, {
        cause,
      });
    });
    return loading;
  };
}

/** The calendars, the almanac and the muhurta search: the `panchanga` module. */
export const loadCalendar = once('calendar', () => import('./panchanga.js'));

/** Everything, the charts and the jyotish among it: the full module. */
export const loadJyotish = once('jyotish', () => import('./index.js'));
