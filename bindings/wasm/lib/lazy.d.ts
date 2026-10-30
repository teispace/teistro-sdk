export { INTERNAL_CODE, onInternalError } from './index.js';
export type { InternalReport } from './index.js';

/** The package's layer, as an entry gives it. */
export type Teistro = typeof import('./index.js');

/**
 * Loads the calendars, the almanac and the muhurta search once, however
 * often it is asked: the `panchanga` module, which refuses a chart as a
 * `CAPABILITY`. It rejects with an `Error` saying why, with the host's own
 * as its `cause`, and stays rejected until the page is loaded again.
 */
export declare function loadCalendar(): Promise<Teistro>;

/**
 * Loads everything once, however often it is asked: the full module. It is
 * a second module beside the calendar's, so a context or a value of one is
 * nothing to the other, and a refusal is told by its `status`. It rejects
 * as `loadCalendar` does.
 */
export declare function loadJyotish(): Promise<Teistro>;
