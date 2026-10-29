// `@teistro/compat`'s declarations.
//
// HAND-WRITTEN, like `index.js`. The birth record's type is written from
// the rows an application on the baseline engine stores
// (`docs/03-design/compat-shim.md` §3.7), here and nowhere else; every
// answer is the SDK's own type.

import type { Almanac, AlmanacDay, CalendarDate, Chart, Charts, ContextInit } from '@teistro/sdk';

/**
 * A stored birth record, as the application keeps it. The instant is the
 * UTC date; the Julian day, derived from it, must agree with it to its
 * stored precision (six decimals, 43.2 ms).
 */
export interface BirthRecord {
  /** Read only by the calls that read a name (numerology, naam). */
  readonly name?: string;
  /** The instant as a UTC date, or its ISO-8601 text as a row stores it: what every call founds at. */
  readonly dateTimeUtc: Date | string;
  /** The instant as a UT Julian day, derived from the date: checked against it, never founded at. */
  readonly julianDay: number;
  readonly latitude: number;
  readonly longitude: number;
  /** Metres. */
  readonly altitude: number;
  /** The IANA zone the clock is read in, unless `isLmt`. */
  readonly timezone: string;
  /** The clock is local mean time at the longitude. */
  readonly isLmt: boolean;
  /** Read only by rectification. */
  readonly gender?: 'male' | 'female' | 'other';
  /** Read only by rectification; every other call founds at the instant. */
  readonly birthTimeUncertainty?: 'exact' | 'approximate' | 'rectified' | 'unknown';
  /** The application's own; no computation reads it. */
  readonly birthOrder?: number | null;
  /** The application's own; the instant already fixes the date. */
  readonly bsDate?: string | { readonly year: number; readonly month: number; readonly day: number } | null;
}

/** A stored settings snapshot: its fields, or their JSON. */
export type StoredSnapshot = Readonly<Record<string, unknown>> | string;

/**
 * What every context the shim opens shares. `profile` and `settings` are
 * each stored snapshot's to decide; `locale` is the one used when a
 * snapshot stores none.
 */
export type CompatOptions = Omit<ContextInit, 'profile' | 'settings'>;

/**
 * A call refused before any computation: an entry point, a stored field
 * or a stored value the SDK does not take, or a birth record that
 * contradicts itself. An SDK error is never one of these. When the SDK
 * refused a stored value, its `TeistroError` is the `cause`, record whole.
 */
export declare class CompatRefused extends Error {
  /** The surface or settings row: `birth.instant`, `snapshot.suryaBija`. */
  readonly row: string;
  /** The anchor in the migration guide that explains it. */
  readonly guide: string;
}

/**
 * The shim over one set of context options, holding the contexts it opens.
 * Every row takes a stored birth record and the stored snapshot it was
 * computed under, and answers the SDK's own result; the KP row answers on
 * the snapshot's KP context, every other on its natal one.
 */
export interface Shim {
  /** The foundation (every varga, aspects, points, houses, states, Jaimini) and the Vimshottari. */
  chart(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The whole natal chart: every section, the eighteen dashas, every shipped rule set and the longevity. */
  full(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** Every divisional chart, with the foundation. */
  vargas(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The yogas and doshas. */
  yogas(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The Shadbala, the Bhava bala and the Ashtakavarga. */
  strength(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The eighteen dasha systems. */
  dashas(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The almanac day of `date`, the birth's own civil date by default, at the birth's place on its zone's clock at noon. */
  day(birth: BirthRecord, snapshot: StoredSnapshot, options?: { readonly date?: CalendarDate }): AlmanacDay;
  /** The almanac days `from` to `to`, both included, at the birth's place on its zone's clock at noon on `from`. */
  month(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly from: CalendarDate; readonly to: CalendarDate }): Almanac;
  /** The birth as the groom, matched with `partner` as the bride. */
  milan(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly partner: BirthRecord }): Chart;
  /** The annual charts through year `through` of life, with their matters, sahams and the Mudda dasha. */
  varsha(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly through: number }): Chart;
  /**
   * KP's cusps, lords, significators and ruling planets, on the KP context;
   * with `anyAyanamsha`, on the natal context under the chart's own ayanamsha.
   */
  kp(birth: BirthRecord, snapshot: StoredSnapshot, options?: { readonly anyAyanamsha?: boolean }): Chart;
  /** The transits at `instants`, UT Julian days, over the chart, with the Ashtakavarga's say. */
  gochar(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly instants: ArrayLike<number> }): Chart;
  /** Sade Sati and the dhaiyyas from `from` to `to`, UT Julian days. */
  sadeSati(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly from: number; readonly to: number }): Chart;
  /** Whole charts at `instants`, UT Julian days, at the birth's place and on its clock. */
  batch(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly instants: ArrayLike<number> }): Charts;
  /**
   * How many times this thread called each row, zero included. When every
   * count is zero, the application has finished porting.
   */
  calls(): Readonly<Record<string, number>>;
  /**
   * Disposes every context the shim opened.
   *
   * The shim also implements `Symbol.dispose`, so it works with `using`.
   * It is not declared here, as the SDK's `Context` does not declare it:
   * the symbol exists only in `lib: ["ESNext.Disposable"]` and above.
   */
  dispose(): void;
}

/** The shim, over the options every context it opens shares. */
export declare function compat(options?: CompatOptions): Shim;
