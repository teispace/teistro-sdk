// The ergonomic layer's declarations.
//
// HAND-WRITTEN, like `index.js` itself. Everything it names is generated:
// the enums and their tables (`catalogue.d.ts`), the boundary's value
// types (`types.d.ts`) and the decoded result blobs (`blob.d.ts`). What
// this file adds is the shape of the layer: the context, the results that
// decode on first use, and the error.

import type {
  Body,
  Calendar,
  Graha,
  Scale,
  Status,
  TimeScale,
} from './catalogue.js';
import type {
  CalendarDate,
  CivilDateTime,
  ContextOptions,
  DeltaT,
  Frame,
  IntlLoaded,
  TimeConversion,
  ZoneResolution,
  ZoneSpec,
} from './types.js';
import type { IntlRender, Positions as DecodedPositions } from './blob.js';

export * from './catalogue.js';
export type * from './types.js';
export type * from './blob.js';
export { decodeIntlRender, decodePositions } from './blob.js';

/** A failed call, with everything the library said about it. */
export declare class TeistroError extends Error {
  /** The status's name, the same in every binding. */
  readonly status: Status;
  /** Its stable numeric code. */
  readonly code: number;
  /** What, more precisely, went wrong. */
  readonly detail: string | null;
  /** The field involved. */
  readonly field: string | null;
  /** A suggestion (`did you mean ...`). */
  readonly hint: string | null;
  /** The localisable message key. */
  readonly messageKey: string | null;
  /** The provider's own code when the status is `provider`. */
  readonly providerCode: number;
}

/** A result the library returned, decoded on first use and only once. */
declare class Decoded<T> {
  /** The bytes the library returned; the columns are views over them. */
  readonly bytes: Uint8Array;
  /** The decoded sections. */
  readonly decoded: T;
}

/** One cell of a positions grid, built on demand. */
export interface Cell {
  /** Longitude in degrees, 0 to 360. */
  readonly longitude: number;
  /** Latitude in degrees. */
  readonly latitude: number;
  /** Distance in the provider's unit. */
  readonly distance: number;
  /** Longitude speed in degrees per day. */
  readonly longitudeSpeed: number;
  /** Latitude speed in degrees per day. */
  readonly latitudeSpeed: number;
  /** Distance speed per day. */
  readonly distanceSpeed: number;
  /** The cell's status code; zero is a value. */
  readonly status: number;
  /** What computed the cell, packed as the port packs it. */
  readonly source: number;
}

/** One step of the frame completion, and who did it. */
export interface Step {
  /** The step's name. */
  readonly name: string;
  /** `NATIVE`, `SDK` or `PASS_THROUGH`. */
  readonly implementation: string;
}

/** Positions over a grid, with the cells readable one at a time. */
export declare class Positions extends Decoded<DecodedPositions> {
  /** The instants of the request, in order. */
  readonly instants: Float64Array;
  /** The bodies of the request, in order, by their catalogue key. */
  readonly bodies: readonly Body[];
  /** The bodies as the ids the blob carries, without a copy. */
  readonly bodyIds: Uint16Array;
  /** The time scale the instants are on. */
  readonly scale: TimeScale | 'unknown';
  /** The cells, instants outermost, as typed arrays over the blob. */
  readonly cells: DecodedPositions['cells'];
  /** The completion steps the SDK applied, in order. */
  readonly steps: readonly Step[];
  /** Everything that reproduces this result (ADR-0020). */
  readonly provenance: Record<string, unknown>;
  /** One cell as a plain object; the columns stay where they are. */
  at(instant: number, body: number): Cell;
}

/** A rendered message. */
export declare class Rendered extends Decoded<IntlRender> {
  /** The plain text, markup stripped. */
  readonly text: string;
  /** The locale whose message answered, `null` when none had it. */
  readonly resolvedFrom: string | null;
  /** Whether a fallback locale answered. */
  readonly isFallback: boolean;
  /** Whether a runtime override answered. */
  readonly isOverride: boolean;
  /** Every problem met; rendering continues past each. */
  readonly warnings: readonly string[];
  /** The text, so a rendered message reads where a string is expected. */
  toString(): string;
}

/** What a positions request names. */
export interface PositionsRequest {
  /** The instants, as Julian days on `scale`. */
  readonly instants: readonly number[] | Float64Array;
  /** The bodies, by their catalogue key. */
  readonly bodies: readonly Body[];
  /** The scale the instants are on; `ut1` by default. */
  readonly scale?: TimeScale;
  /** The frame the positions are wanted in; the canonical one by default. */
  readonly frame?: Frame;
  /** Whether speeds are wanted; `true` by default. */
  readonly speeds?: boolean;
  /** The place a topocentric frame needs. */
  readonly observer?: { readonly longitudeDeg: number; readonly latitudeDeg: number; readonly altitudeM: number };
}

/** How a context is built. */
export interface ContextInit {
  /** A shipped profile's id; `defaultProfile()` names the default. */
  readonly profile?: string;
  /** A settings patch over the profile, as the settings document shapes it. */
  readonly settings?: Record<string, unknown>;
  /** The locale every render resolves from. */
  readonly locale?: string;
  /** Use the SDK's analytic test provider; for examples and tests only. */
  readonly testProvider?: boolean;
}

/**
 * A context: settings resolved from a profile and a patch, a locale, and
 * an ephemeris. One context serves one thread; a worker builds its own.
 */
export declare class Context {
  constructor(options?: ContextInit);
  /** The id of the profile the settings came from. */
  readonly profile: string;
  /** The resolved settings, as their canonical document. */
  readonly settings: Record<string, unknown>;
  /** The SHA-256 of the canonical settings, in hex. */
  readonly settingsHash: string;
  /** The locale every render resolves from. */
  locale: string;
  /** The SDK's canonical frame. */
  canonicalFrame(): Frame;
  /** Positions over a grid, completed into the frame asked for. */
  positions(request: PositionsRequest): Positions;
  /** Renders a message of the current locale with its parameters. */
  render(key: string, params?: Record<string, unknown>): Rendered;
  /** Whether the current locale or its fallbacks have a message. */
  has(key: string): boolean;
  /** Loads a `.tpack` or `.tbundle` file into the locale engine. */
  loadPack(bytes: Uint8Array): IntlLoaded;
  /** The date a fixed day falls on in a calendar. */
  dateOf(calendar: Calendar, fixed: number): CalendarDate;
  /** The fixed day of a date. */
  fixedOf(date: CalendarDate): number;
  /** The same date in another calendar. */
  convert(date: CalendarDate, into: Calendar): CalendarDate;
  /** The weekday of a date, Monday `1` to Sunday `7`. */
  weekdayOf(date: CalendarDate): number;
  /** The length of a month. */
  monthLength(calendar: Calendar, year: number, month: number): number;
  /** Whether a year is a leap year. */
  isLeap(calendar: Calendar, year: number): boolean;
  /** A civil date and time in a zone, resolved with its metadata. */
  resolve(civil: CivilDateTime, zone: ZoneSpec): ZoneResolution;
  /** The civil date and time of an instant in a zone. */
  civilOf(
    jdUtc: number,
    zone: ZoneSpec,
    calendar: Calendar,
  ): { readonly civil: CivilDateTime; readonly resolution: ZoneResolution };
  /** Converts an instant between the time scales. */
  convertTime(jd: number, from: Scale, to: Scale): TimeConversion;
  /** Delta T at a UT1 instant, with what produced it. */
  deltaT(jdUt1: number): DeltaT;
  /** The packed id of a catalogue key. */
  keyId(key: string): number;
  /** The catalogue key of a packed id. */
  keyName(id: number): string;
}

/** The ABI the addon implements. */
export declare function abiVersion(): number;
/** The SDK's version. */
export declare function sdkVersion(): string;
/** The catalogue's schema version, stamped in every result's provenance. */
export declare function catalogueVersion(): number;
/** The profile a context uses when none is named. */
export declare function defaultProfile(): string;
/** The SDK's canonical frame. */
export declare function canonicalFrame(): Frame;
/** The Julian day at the UTC midnight that begins a fixed day. */
export declare function julianDayOfFixed(fixed: number): number;
/** The fixed day a Julian day falls in, and the fraction elapsed. */
export declare function fixedOfJulianDay(jd: number): { readonly value: number; readonly fraction: number };
/** Packs a frame's fields into the bits a position request carries. */
export declare function packFrame(frame: Frame): number;
/** Reads packed frame bits back into their fields. */
export declare function unpackFrame(bits: number): Frame;
