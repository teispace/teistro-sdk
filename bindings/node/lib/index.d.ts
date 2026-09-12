// The ergonomic layer's declarations.
//
// HAND-WRITTEN, like `index.js` itself. Everything it names is generated:
// the enums and their tables (`catalogue.d.ts`), the boundary's value
// types (`types.d.ts`) and the decoded result blobs (`blob.d.ts`). What
// this file adds is the shape of the layer: the context, the results that
// decode on first use, and the error.

import type {
  Ayana,
  Body,
  Calendar,
  ChartKind,
  Choghadiya,
  DayPart,
  Direction,
  Graha,
  HouseSystem,
  Kaala,
  Karana,
  LunarMonth,
  Masa,
  MonthKind,
  MuhurtaYoga,
  Nakshatra,
  Paksha,
  Panchaka,
  Rashi,
  Scale,
  Status,
  Tithi,
  TimeScale,
  Vara,
  Yoga,
} from './catalogue.js';
import type {
  CalendarDate,
  CivilDateTime,
  ContextOptions,
  DeltaT,
  Frame,
  IntlLoaded,
  Observer,
  TimeConversion,
  ZoneResolution,
  ZoneSpec,
} from './types.js';
import type {
  Charts as DecodedCharts,
  IntlRender,
  Panchanga as DecodedAlmanac,
  Positions as DecodedPositions,
} from './blob.js';
// The typed accessors and an entity's forms, declared where `sdk.intl`
// answers with them: the flat surface had both members and declared
// neither, which the areas made visible.
import type { EntityForms, Messages } from './messages.js';

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

/** One graha of a chart, built on demand. */
export interface PlacedGraha {
  /** Which graha, by its catalogue key. */
  readonly graha: Graha | 'unknown';
  /** Its longitude in the chart's zodiac, degrees. */
  readonly longitudeDeg: number;
  /** Its tropical longitude, degrees. */
  readonly tropicalDeg: number;
  /** Its ecliptic latitude, degrees. */
  readonly latitudeDeg: number;
  /** Its distance, astronomical units. */
  readonly distanceAu: number;
  /** Its longitude speed, degrees per day. */
  readonly speedDegPerDay: number;
  /** Whether that speed is negative. */
  readonly retrograde: boolean;
  /** The bhava for "which house is it in". */
  readonly house: Placement;
  /** The bhava of the chart's chalit, which is a different question. */
  readonly placement: Placement;
}

/** Where a graha sits in a set of bhavas. */
export interface Placement {
  /** The bhava, 1 to 12. */
  readonly bhava: number;
  /** The house system that produced it. */
  readonly method: HouseSystem | 'unknown';
  /** How far through the bhava it is, 0 to 1. */
  readonly through: number;
  /** Its distance from the bhava's centre, degrees. */
  readonly fromMadhyaDeg: number;
}

/** One of the twelve bhavas. */
export interface Bhava {
  /** The bhava's centre, degrees. */
  readonly madhyaDeg: number;
  /** The bhava's opening cusp, degrees. */
  readonly sandhiDeg: number;
}

/** The place a chart was founded at. */
export interface ChartPlace {
  /** Degrees north. */
  readonly latitude: number;
  /** Degrees east. */
  readonly longitude: number;
  /** Metres above the ellipsoid. */
  readonly altitude: number;
}

/**
 * A batch of founded charts at one place, decoded on first use and only
 * once. The charts in it are views over those bytes rather than copies.
 */
export declare class Charts extends Decoded<DecodedCharts> {
  /** How many charts the batch holds. */
  readonly length: number;
  /** What kind of chart these are. */
  readonly kind: ChartKind | 'unknown';
  /** The place they were all founded at. */
  readonly place: ChartPlace;
  /**
   * The completion steps the SDK applied, in order, each
   * `name:Implementation`. A positions result spells the same steps as
   * objects; the asymmetry is a recorded open question.
   */
  readonly steps: readonly string[];
  /** The solar model that reckoned the days, as it describes itself. */
  readonly model: string;
  /** Everything that reproduces this result (ADR-0020). */
  readonly provenance: Record<string, unknown>;
  /** One chart of the batch, by index. */
  at(index: number): Chart;
  /** Every chart, in the order the instants were asked for. */
  [Symbol.iterator](): IterableIterator<Chart>;
}

/** One founded chart: a view over its batch, not a copy. */
export declare class Chart {
  /** The batch this chart belongs to. */
  readonly batch: Charts;
  /** Where in that batch it sits. */
  readonly index: number;
  /** The instant the chart is cast for, as a Julian day (UTC). */
  readonly instant: number;
  /** What kind of chart this is. */
  readonly kind: ChartKind | 'unknown';
  /** The place it was founded at. */
  readonly place: ChartPlace;
  /** The lagna at the instant, in the chart's zodiac, degrees. */
  readonly lagnaDeg: number;
  /** The lagna at the sunrise that opened the day, degrees. */
  readonly dayLagnaDeg: number;
  /** The ayanamsha applied at this instant, degrees; zero if tropical. */
  readonly ayanamshaOffsetDeg: number;
  /**
   * Which arc of its day the instant falls in. This and `dayElapsed`
   * belong to the instant, not to the day.
   */
  readonly dayPart: DayPart | 'unknown';
  /** How far through that arc the instant is, 0 to 1. */
  readonly dayElapsed: number;
  /**
   * The day the chart belongs to, which is not always its civil date:
   * the values the blob carries, with `vara` named.
   */
  readonly day: Omit<{ [K in keyof DecodedCharts['day']]: number }, 'length' | 'vara'> & {
    readonly vara: Vara | 'unknown';
  };
  /** Where in its day the moment falls, with `horaLord` named. */
  readonly timing: Omit<
    { [K in keyof DecodedCharts['timing']]: number },
    'length' | 'horaLord'
  > & { readonly horaLord: Graha | 'unknown' };
  /** The grahas, in the catalogue's order, one object each. */
  readonly grahas: readonly PlacedGraha[];
  /** The twelve bhavas for "which house is it in", first to twelfth. */
  readonly houses: readonly Bhava[];
  /** The twelve bhavas of the chart's chalit. */
  readonly chalit: readonly Bhava[];
  /** The completion steps the SDK applied, in order. */
  readonly steps: readonly string[];
  /** The provenance envelope of the batch this chart came from. */
  readonly provenance: Record<string, unknown>;
}

/** A span of time, as every almanac row carries one. */
export interface Interval {
  /** When it begins, as a Julian day (UTC). */
  readonly from: number;
  /** When it ends. */
  readonly to: number;
}

/** One member of a limb, with its own bounds and the clipped ones. */
export interface Span<T> {
  /** Which member ran. */
  readonly member: T | 'unknown';
  /** When the member itself began and ended, inside the day or not. */
  readonly whole: Interval;
  /** The part inside the day: what an almanac row prints. */
  readonly inside: Interval;
}

/** The lunar month a day falls in, under both conventions. */
export interface Month {
  /** The month under the profile's own convention. */
  readonly month: Masa | 'unknown';
  /** The amanta month: new moon to new moon. */
  readonly amanta: Masa | 'unknown';
  /** The purnimanta month: full moon to full moon. */
  readonly purnimanta: Masa | 'unknown';
  /** Which fortnight the day opens in. */
  readonly paksha: Paksha | 'unknown';
  /** Which convention `month` leads with. */
  readonly convention: LunarMonth | 'unknown';
  /**
   * Whether the month is ordinary, intercalary or omitted. The name
   * above needs no case for the intercalary one — an adhika month and
   * the nija month after it take the same name — so this is the mark
   * beside the name.
   */
  readonly kind: MonthKind | 'unknown';
}

/** One inauspicious eighth of the daylight. */
export interface KaalaPeriod extends Interval {
  /** Which one. */
  readonly kaala: Kaala | 'unknown';
}

/** One choghadiya, of the daylight or of the night. */
export interface ChoghadiyaPeriod extends Interval {
  /** Which choghadiya. */
  readonly choghadiya: Choghadiya | 'unknown';
  /** The graha that rules it. */
  readonly lord: Graha | 'unknown';
  /** Whether it is one of the eight of the daylight. */
  readonly daytime: boolean;
}

/** One hora, from sunrise. */
export interface Hora {
  /** Its number, 1 to 24. */
  readonly number: number;
  /** The graha that rules it. */
  readonly lord: Graha | 'unknown';
  /** When it begins, as a Julian day (UTC). */
  readonly start: number;
  /** When it ends. */
  readonly end: number;
}

/** One of the thirty muhurtas. */
export interface Muhurta extends Interval {
  /** Whether it is one of the fifteen of the daylight. */
  readonly daylight: boolean;
}

/** A moonrise or a moonset. */
export interface MoonEvent {
  /** Which it was. */
  readonly kind: 'rise' | 'set';
  /** When, as a Julian day (UTC). */
  readonly instant: number;
}

/** A muhurta yoga that held, and what made it hold. */
export interface HeldYoga extends Interval {
  /** Which yoga. */
  readonly yoga: MuhurtaYoga | 'unknown';
  /** What made it, so a reader can see why. */
  readonly because: {
    /** Which cause it is. */
    readonly kind: 'vara-nakshatra' | 'vara-tithi-nakshatra';
    /** The vara that makes it; every cause has one. */
    readonly vara: Vara | 'unknown';
    /** The tithi, when the cause has one; `null` otherwise. */
    readonly tithi: Tithi | 'unknown' | null;
    /** The nakshatra that makes it. */
    readonly nakshatra: Nakshatra | 'unknown';
  };
}

/** Abhijit, with whether it is effective. */
export interface Abhijit extends Interval {
  /** True on every day but a Wednesday. */
  readonly effective: boolean;
}

/**
 * A batch of daily panchangas at one place, decoded on first use and only
 * once. Each day is a view over those bytes rather than a copy.
 */
export declare class Almanac extends Decoded<DecodedAlmanac> {
  /** How many days the batch holds. */
  readonly length: number;
  /** The place they were all founded at. */
  readonly place: ChartPlace;
  /** The civil calendar the days' dates are read in. */
  readonly calendar: Calendar | 'unknown';
  /** The solar model that reckoned the days, as it describes itself. */
  readonly model: string;
  /** Everything that reproduces this result (ADR-0020). */
  readonly provenance: Record<string, unknown>;
  /** One day of the batch, by index. */
  at(index: number): AlmanacDay;
  /** Every day, in the order the range runs. */
  [Symbol.iterator](): IterableIterator<AlmanacDay>;
  /** Where day `index`'s rows of a per-day list begin and end. */
  range(list: string, index: number): [number, number];
}

/** One day of an almanac: a view over its batch, not a copy. */
export declare class AlmanacDay {
  /** The batch this day belongs to. */
  readonly batch: Almanac;
  /** Where in that batch it sits. */
  readonly index: number;
  /**
   * The day itself — the same eighteen fields a chart's day carries,
   * decoded into the same type, with `vara` named.
   */
  readonly day: Omit<{ [K in keyof DecodedAlmanac['day']]: number }, 'length' | 'vara'> & {
    readonly vara: Vara | 'unknown';
  };
  /** What the spans are clipped to. */
  readonly window: Interval;
  /** The lunar month, under both conventions. */
  readonly month: Month;
  /** Which half of the year the day falls in. */
  readonly ayana: Ayana | 'unknown';
  /** The direction not to travel in. */
  readonly dishaShool: Direction | 'unknown';
  /** When the Sun entered a new sign inside the day, or `null`. */
  readonly sankranti: number | null;
  /** Abhijit; `null` on a day with no daylight. */
  readonly abhijit: Abhijit | null;
  /** Brahma muhurta; `null` when the night before is not known. */
  readonly brahma: Interval | null;
  /** The tithis that touch the day. */
  readonly tithi: readonly Span<Tithi>[];
  /** The nakshatras the Moon was in. */
  readonly nakshatra: readonly Span<Nakshatra>[];
  /** The nitya yogas. */
  readonly yoga: readonly Span<Yoga>[];
  /** The karanas: half-tithis. */
  readonly karana: readonly Span<Karana>[];
  /** Panchaka, while the Moon is in the last five nakshatras. */
  readonly panchaka: readonly Span<Panchaka>[];
  /** The signs the Moon stood in. */
  readonly moonSigns: readonly Span<Rashi>[];
  /** The signs the Sun stood in. */
  readonly sunSigns: readonly Span<Rashi>[];
  /** The inauspicious eighths of the daylight. */
  readonly kaalas: readonly KaalaPeriod[];
  /** Eight choghadiya of the daylight and eight of the night. */
  readonly choghadiya: readonly ChoghadiyaPeriod[];
  /** The twenty-four horas, from sunrise. */
  readonly horas: readonly Hora[];
  /** The thirty muhurtas. */
  readonly muhurtas: readonly Muhurta[];
  /** Every moonrise and moonset inside the day's moon window. */
  readonly moonEvents: readonly MoonEvent[];
  /** The muhurta yogas that held. */
  readonly muhurtaYogas: readonly HeldYoga[];
  /** The provenance envelope of the batch this day came from. */
  readonly provenance: Record<string, unknown>;
}

/** What `Context.almanac` needs: a range of days at one place. */
export interface AlmanacRequest {
  /** The first day. */
  readonly from: CalendarDate;
  /** The last day, both ends included. */
  readonly to: CalendarDate;
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
}

/** What `Context.almanacDay` needs: one day at one place. */
export interface AlmanacDayRequest extends Omit<AlmanacRequest, 'from' | 'to'> {
  /** The day. */
  readonly date: CalendarDate;
}

/** What `Context.found` needs to found one chart. */
export interface ChartRequest {
  /** The instant, as a Julian day (UTC). */
  readonly instant: number;
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** A chart kind; `ChartKind.Natal` by default. */
  readonly kind?: ChartKind;
}

/** What `Context.foundMany` needs to found a batch at one place. */
export interface ChartBatchRequest extends Omit<ChartRequest, 'instant'> {
  /** The instants, as Julian days (UTC): one chart each. */
  readonly instants: ArrayLike<number>;
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
  readonly observer?: Observer;
}

/**
 * An ephemeris of your own. `positions` is asked once for a whole grid,
 * never in a loop, and answers with one value per cell, instants
 * outermost. Returning nothing means "not in that frame": the SDK then
 * asks again in the provider's own frame and completes the rest itself.
 */
export interface EphemerisProvider {
  /** What the provider is; every result's provenance is stamped with it. */
  readonly name: string;
  /** The bodies it answers, by their catalogue keys. */
  readonly bodies: readonly Body[];
  /** The one call: a grid in, the columns out. */
  positions(request: ProviderRequest): ProviderColumns | null | undefined;
  /** Its version; empty by default. */
  readonly version?: string;
  /** What identifies its data; empty by default. */
  readonly dataVersion?: string;
  /** The first Julian day it covers; year 0 by default. */
  readonly jdMin?: number;
  /** The last Julian day it covers; year 3000 by default. */
  readonly jdMax?: number;
  /** The frame it returns natively; the canonical frame by default. */
  readonly nativeFrame?: Frame;
  /** Whether it computes speeds; `true` by default. */
  readonly speeds?: boolean;
  /**
   * Whether identical requests give identical bits; `true` by default,
   * and a provider that is not deterministic must say so, because the
   * conformance contract rests on it (ADR-0022).
   */
  readonly deterministic?: boolean;
}

/** What a provider is asked for. */
export interface ProviderRequest {
  /** The instants, as Julian days on `scale`. */
  readonly jds: readonly number[];
  /** The bodies, by their catalogue keys. */
  readonly bodies: readonly Body[];
  /** The scale the instants are on. */
  readonly scale: TimeScale;
  /** The frame the positions are wanted in, packed; `unpackFrame` reads it. */
  readonly frameBits: number;
  /** Whether speeds are wanted. */
  readonly speeds: boolean;
  /** The place a topocentric frame needs. */
  readonly observer?: Observer;
}

/**
 * What a provider answers with: one value per cell, instants outermost,
 * so cell `i * bodies.length + j` is instant `i`, body `j`. A column left
 * out is zeroes, which is what a provider that computes no speeds means.
 */
export interface ProviderColumns {
  /** The frame the values are in; the request's by default. */
  readonly frameBits?: number;
  /** Longitudes in degrees. */
  readonly lon?: Float64Array | readonly number[];
  /** Latitudes in degrees. */
  readonly lat?: Float64Array | readonly number[];
  /** Distances. */
  readonly dist?: Float64Array | readonly number[];
  /** Longitude speeds in degrees per day. */
  readonly lonSpeed?: Float64Array | readonly number[];
  /** Latitude speeds in degrees per day. */
  readonly latSpeed?: Float64Array | readonly number[];
  /** Distance speeds per day. */
  readonly distSpeed?: Float64Array | readonly number[];
  /** A status per cell; zero, or absent, is a value. */
  readonly status?: Int32Array | readonly number[];
  /** What computed each cell. */
  readonly source?: Uint32Array | readonly number[];
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
  /** An ephemeris of your own, answered in this language. */
  readonly provider?: EphemerisProvider;
}

/**
 * A context: settings resolved from a profile and a patch, a locale, and
 * an ephemeris. One context serves one thread; a worker builds its own.
 */
/** One parameter of an engine's operation, as its manifest describes it. */
export interface EngineParam {
  /** The name, which is the key a caller uses. */
  name: string;
  /**
   * What it is for: `value`, `handle`, `struct_in`, `array_in`,
   * `array_len`, `scalar_out` and the rest — or a word this package has
   * never heard of, kept as the engine spelled it, because an engine
   * that gains a role must still describe itself.
   */
  role: string;
  /** The engine's own spelling of its type. */
  kind?: string;
  /** What it means, when the engine says. */
  doc?: string;
}

/** One operation an engine offers beyond the SDK's own. */
export interface EngineFunction {
  /** The name to call it by. */
  name: string;
  /** Its parameters, in the engine's own order. */
  params?: EngineParam[];
  /** What it answers with, in the engine's spelling. */
  returns?: string;
  /** What it does, when the engine says. */
  doc?: string;
}

/** What an engine says it offers. */
export interface EngineManifest {
  /** The engine's name. */
  engine?: string;
  /** Its version, which is what a caller caches against. */
  version?: string;
  /** The operations, in the engine's own order. */
  functions?: EngineFunction[];
}

/**
 * The engine's own operations, reached by the names it gives them.
 *
 * The SDK names eight operations; an engine names far more, and what it
 * names beyond them is reached through here. Nothing in this type is a
 * list of an engine's operations: a function the engine gains after this
 * package ships is callable through `call` without a new release of it,
 * so the names cannot be written down here.
 *
 * There is **no index signature**. ADR-0030 considered one and rejected
 * it under ADR-0023: it type-checks everything, the misspelling
 * included, and Dart and Rust cannot express it, so it would be a
 * surface that exists in two of five targets. What replaces it is the
 * typed façade an adapter generates from its own engine's description,
 * which augments this type from the adapter's own package.
 *
 * ```ts
 * const engine = context.engine;
 * const answer = engine.call('tp_echo', { value: 6 });
 * ```
 */
export declare class Engine {
  /** The manifest as the engine wrote it. */
  readonly manifestJson: string;
  /** The manifest, parsed and remembered. */
  readonly manifest: EngineManifest;
  /** Every operation the engine offers, in its own order. */
  readonly names: string[];
  /** What the manifest says about one operation, or `undefined`. */
  signature(name: string): EngineFunction | undefined;
  /** Calls an operation by name, with its parameters as an object. */
  call(name: string, argumentsObject?: Record<string, unknown>): unknown;
  /** Calls an operation with arguments as JSON, answering with JSON. */
  callJson(name: string, argumentsJson: string): string;
}

/**
 * `sdk.calendar` — the calendars, and the fixed day they share.
 *
 * An **area**: a value built once with the context, which a consumer may
 * destructure and keep (`const { calendar } = sdk`).
 */
export declare class CalendarArea {
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
}

/** `sdk.time` — the scales, the zones and what separates them. */
export declare class TimeArea {
  /** A civil date and time in a zone, resolved with its metadata. */
  resolve(civil: CivilDateTime, zone: ZoneSpec): ZoneResolution;
  /** The civil date and time of an instant in a zone. */
  civilOf(
    jdUtc: number,
    zone: ZoneSpec,
    calendar: Calendar,
  ): { readonly civil: CivilDateTime; readonly resolution: ZoneResolution };
  /** Converts an instant between the time scales. */
  convert(jd: number, from: Scale, to: Scale): TimeConversion;
  /** Delta T at a UT1 instant, with what produced it. */
  deltaT(jdUt1: number): DeltaT;
}

/** `sdk.intl` — the locale, its messages and the scripts they are in. */
export declare class IntlArea {
  /** The locale every render resolves from. */
  locale: string;
  /** Renders a message of the current locale with its parameters. */
  render(key: string, params?: Record<string, unknown>): Rendered;
  /** Whether the current locale or its fallbacks have a message. */
  has(key: string): boolean;
  /** Text from one script into another (`deva`, `iast`). */
  transliterate(text: string, from?: string, to?: string): string;
  /** An entity's forms in the current locale or its fallbacks. */
  entity(key: string): EntityForms;
  /** The typed accessors: every message and every catalogued entity. */
  readonly messages: Messages;
  /** Loads a `.tpack` or `.tbundle` file into the locale engine. */
  loadPack(bytes: Uint8Array): IntlLoaded;
}

/** `sdk.keys` — the catalogue's keys and their packed ids. */
export declare class KeysArea {
  /** The packed id of a catalogue key. */
  id(key: string): number;
  /** The catalogue key of a packed id. */
  name(id: number): string;
}

/** `sdk.frame` — the coordinate conventions a request is expressed in. */
export declare class FrameArea {
  /** The SDK's canonical frame. */
  canonical(): Frame;
}

/** `sdk.chart` — a chart founded at an instant and a place. */
export declare class ChartArea {
  /** Founds a chart at an instant and a place. */
  found(request: ChartRequest): Chart;
  /**
   * Founds a chart at each of many instants, at one place, in one
   * crossing: the founder shares the settings and the solar model across
   * the batch. A batch of none is an empty result rather than an error.
   */
  foundMany(request: ChartBatchRequest): Charts;
}

/**
 * `sdk.almanac` — a day, or a run of days, with its limbs.
 *
 * The boundary calls this `panchanga`; the area takes the consumer's
 * word, because an almanac is what the operation answers and a panchanga
 * is one tradition's name for five of its limbs.
 */
export declare class AlmanacArea {
  /**
   * The almanac of every day in a range, at one place: consecutive days
   * share a boundary, so a month costs much less than thirty days
   * computed separately.
   */
  of(request: AlmanacRequest): Almanac;
  /** The almanac of one day, which is the range of one unwrapped. */
  day(request: AlmanacDayRequest): AlmanacDay;
}

export declare class Context {
  constructor(options?: ContextInit);
  /**
   * The engine's own operations, beyond the eight the SDK names.
   *
   * **Not `ephemeris`**: `engine` says *this particular engine, not the
   * portable contract*, so a consumer reading their own code sees the
   * difference between a call that survives changing provider and one
   * that does not (ADR-0030).
   *
   * Throws when the context has no ephemeris, or when the one it has
   * describes nothing of its own.
   */
  readonly engine: Engine;
  /** The calendars, and the fixed day they share. */
  readonly calendar: CalendarArea;
  /** The scales, the zones and what separates them. */
  readonly time: TimeArea;
  /** The locale, its messages and the scripts they are in. */
  readonly intl: IntlArea;
  /** The catalogue's keys and their packed ids. */
  readonly keys: KeysArea;
  /** The coordinate conventions a request is expressed in. */
  readonly frame: FrameArea;
  /** A chart founded at an instant and a place. */
  readonly chart: ChartArea;
  /** A day, or a run of days, with its limbs. */
  readonly almanac: AlmanacArea;
  /** The id of the profile the settings came from. */
  readonly profile: string;
  /** The resolved settings, as their canonical document. */
  readonly settings: Record<string, unknown>;
  /**
   * The same document as the text the library wrote, which is what the
   * settings hash is taken over and what a stored chart keeps.
   */
  readonly settingsJson: string;
  /** The SHA-256 of the canonical settings, in hex. */
  readonly settingsHash: string;
  /**
   * Positions over a grid, completed into the frame asked for.
   *
   * On the context and not in an area, because an operation whose name is
   * its own area's name is a root operation: this is the SDK's one
   * primitive over the port, and every area is built on it.
   */
  positions(request: PositionsRequest): Positions;
  /**
   * Frees the context's native memory now, rather than when the
   * collector gets to it.
   *
   * The layer also implements `Symbol.dispose`, so a context works with
   * `using`. It is **not declared here**: the symbol exists only in
   * `lib: ["ESNext.Disposable"]` and above, and a declaration referring
   * to it would make this file refuse to compile for every consumer on
   * a lower target rather than for none.
   */
  dispose(): void;
}

/**
 * What the loaded addon says about its own build: the SDK version, the
 * ABI and catalogue versions, the commit it came from and whether that
 * tree was clean, the profile, the target, whether it is optimised, the
 * sanitizer if any, and the compiler. The two halves of the binding must
 * be one build, and the loader refuses one that is not.
 */
export interface BuildInfo {
  readonly sdk: string;
  readonly abi: number;
  readonly catalogue: number;
  readonly commit: string;
  readonly dirty: boolean;
  readonly profile: string;
  readonly target: string;
  readonly debug_assertions: boolean;
  readonly optimised: boolean;
  readonly sanitizer: string;
  readonly rustc: string;
}

/** What the loaded addon says about its own build. */
export declare const buildInfo: BuildInfo;

/**
 * The npm package that carries this host's prebuilt addon, as
 * `@teistro/sdk-<platform>-<arch>` in Node's own words for the host.
 *
 * A release publishes one such package per platform and this package
 * depends on all of them optionally, so npm installs the matching one.
 * It is exported because the name is what an install failure has to name,
 * and a deployment script may want to fetch it itself.
 */
export declare function platformPackage(): string;

/**
 * Why a build may not be loaded, or `null` when it may. The loader calls
 * it; a test or a packaging check may call it with a build of its own.
 *
 * @param info what the addon reported
 * @param named whether its path was given rather than searched for
 */
export declare function refuseBuild(info: BuildInfo, named: boolean): string | null;

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
