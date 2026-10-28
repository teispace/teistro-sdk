// `@teistro/compat`'s declarations.
//
// HAND-WRITTEN, like `index.js`. The birth record's type is written from
// the rows an application on the baseline engine stores
// (`docs/03-design/compat-shim.md` §3.7), here and nowhere else; every
// answer is the SDK's own type.

import type { ContextInit } from '@teistro/sdk';

/**
 * A stored birth record, as the application keeps it. The instant is the
 * Julian day; the UTC date must agree with it to the millisecond.
 */
export interface BirthRecord {
  /** Read only by the calls that read a name (numerology, naam). */
  readonly name?: string;
  /** The instant as a UTC date, or its ISO-8601 text as a row stores it. */
  readonly dateTimeUtc: Date | string;
  /** The instant as a UT Julian day: what every call founds at. */
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

/** The shim over one set of context options, holding the contexts it opens. */
export interface Shim {
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
