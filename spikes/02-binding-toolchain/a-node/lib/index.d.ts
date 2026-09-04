/**
 * Spike 2, option A: the typed surface of the ergonomic layer.
 *
 * HAND-WRITTEN over `generated.d.ts`. The generated file is the contract;
 * this file adds what a generator cannot know yet: branded scalars whose
 * constructors validate (ADR-0023), option objects with defaults, and the
 * lazily decoded `Chart`.
 */

import type { Ayanamsha, NodeKind, PositionProvider, Settings, Status } from './generated.js';

export type { Position, PositionProvider, Settings } from './generated.js';
export { Ayanamsha, Body, NodeKind, Status } from './generated.js';

/** A Julian Day in UT that passed `julianDay`; a plain `number` does not type-check where one is required. */
export type JulianDay = number & { readonly __brand: 'JulianDay' };

/** A dasha depth from 1 to 5; the type carries the range. */
export type DashaDepth = 1 | 2 | 3 | 4 | 5;

/** Validates a Julian Day: finite, or a `TypeError`. */
export function julianDay(value: number): JulianDay;

/** Validates a dasha depth: an integer from 1 to 5, or a `RangeError`. */
export function dashaDepth(value: number): DashaDepth;

/** The ABI version of the loaded addon. */
export function abiVersion(): number;

/** The number of tree nodes a chart of `depth` levels holds. */
export function chartNodeCount(depth: DashaDepth): number;

/** The default settings: Lahiri, the mean node, three levels. */
export function settingsDefault(): Settings;

/** A static English message for a status. */
export function statusMessage(status: Status): string;

/** What `createContext` accepts: every field optional, defaults from `settingsDefault`. */
export interface ContextOptions {
  /** The ayanamsha; default `'lahiri'`. */
  readonly ayanamsha?: Ayanamsha;
  /** The lunar node; default `'mean'`. */
  readonly node?: NodeKind;
  /** Dasha levels to build; default `3`. */
  readonly dashaDepth?: DashaDepth;
}

/** The positions section as typed-array columns: one entry per body, in body order. */
export interface PositionColumns {
  readonly body: Uint8Array;
  readonly longitudeNas: BigInt64Array;
  readonly longitudeDeg: Float64Array;
  readonly latitudeDeg: Float64Array;
  readonly speedDegPerDay: Float64Array;
  readonly sign: Uint8Array;
  readonly nakshatra: Uint8Array;
  readonly pada: Uint8Array;
  readonly retrograde: Uint8Array;
  readonly length: number;
}

/** The dasha section as typed-array columns, rows in pre-order with parent links. */
export interface DashaColumns {
  readonly level: Uint8Array;
  readonly lord: Uint8Array;
  readonly parent: Int32Array;
  readonly startJd: Float64Array;
  readonly endJd: Float64Array;
  readonly length: number;
}

/** A decoded blob: the chart header and the two column sections, views over the bytes. */
export interface DecodedChart {
  readonly jdUt: number;
  readonly ayanamshaDeg: number;
  readonly depth: number;
  readonly positionCount: number;
  readonly nodeCount: number;
  readonly positions: PositionColumns;
  readonly dasha: DashaColumns;
}

/** One classified position as a plain object. */
export interface BodyPosition {
  readonly body: number;
  readonly longitudeDeg: number;
  readonly longitudeNas: bigint;
  readonly latitudeDeg: number;
  readonly speedDegPerDay: number;
  readonly sign: number;
  readonly nakshatra: number;
  readonly pada: number;
  readonly retrograde: boolean;
}

/** One period with its sub-periods. */
export interface DashaNode {
  readonly lord: number;
  readonly level: number;
  readonly startJd: number;
  readonly endJd: number;
  readonly children: DashaNode[];
}

/** One row of the tree. */
export interface DashaRow {
  readonly lord: number;
  readonly level: number;
  readonly parent: number;
  readonly startJd: number;
  readonly endJd: number;
}

/** A computed chart: the blob, decoded on first use. */
export class Chart {
  constructor(bytes: Uint8Array);
  /** The bytes the native side returned. */
  readonly bytes: Uint8Array;
  /** The columns, decoded once without copying. */
  readonly decoded: DecodedChart;
  /** The instant the chart was computed for. */
  readonly jdUt: number;
  /** The number of tree nodes. */
  readonly nodeCount: number;
  /** The nine positions as plain objects, built on demand. */
  positions(): readonly BodyPosition[];
  /** The tree as nested objects, all of it. */
  dashaTree(): readonly DashaNode[];
  /** One row of the tree, without touching the rest. */
  dashaRow(index: number): DashaRow;
}

/** A context: settings plus a provider, no global state. */
export interface Context {
  /** The one batch call. A plain finite number is accepted and validated; a `JulianDay` skips nothing but says what it is. */
  computeChart(jdUt: JulianDay | number): Chart;
  /** The settings the context was built with. */
  settings(): Settings;
  /** The provider's code from the last failure, `0` when there was none. */
  lastProviderCode(): number;
}

/**
 * Creates a context. `provider` returns a tropical `Position` for a body at
 * an instant; omit it for the built-in analytic test provider.
 */
export function createContext(options?: ContextOptions, provider?: PositionProvider | null): Context;
