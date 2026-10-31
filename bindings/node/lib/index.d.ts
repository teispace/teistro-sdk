// The ergonomic layer's declarations.
//
// HAND-WRITTEN, like `index.js` itself. Everything it names is generated:
// the enums and their tables (`catalogue.d.ts`), the boundary's value
// types (`types.d.ts`) and the decoded result blobs (`blob.d.ts`). What
// this file adds is the shape of the layer: the context, the results that
// decode on first use, and the error.

import type {
  Ayanamsha,
  Balance,
  Ephemeris,
  Modality,
  AvasthaBaladi,
  AgniVasa,
  AshubhaYoga,
  BhadraVasa,
  ShivaVasa,
  AvasthaDeeptadi,
  AvasthaSayanadi,
  AvasthaCheshta,
  AvasthaJagradadi,
  AvasthaLajjitadi,
  Ayana,
  Body,
  Burning,
  Calendar,
  Era,
  Leap,
  ChartKind,
  ChartLayout,
  Choghadiya,
  DashaSystem,
  DayPart,
  Dignity,
  Direction,
  Graha,
  HouseSystem,
  Kaala,
  Karana,
  LunarMonth,
  Masa,
  MonthKind,
  MuhurtaYoga,
  Sunrises,
  Deity,
  Nakshatra,
  Paksha,
  Panchaka,
  Point,
  Quadrant,
  Rashi,
  Ritu,
  Samvatsara,
  Gender,
  TibetanAnimal,
  TibetanElement,
  Shodhana,
  Ekadhipatya,
  Vaiseshikamsa,
  VimshopakaScoring,
  DashaPhase,
  Nature,
  BrahmaRule,
  BrahmaOutcome,
  ChainEnd,
  ChosenBy,
  Signification,
  AshtakavargaGoodFrom,
  Fruition,
  GocharFrom,
  KakshyaLord,
  SarvaStanding,
  HitKind,
  Reckoning,
  Sect,
  SectRule,
  Terms,
  Triplicities,
  Accident,
  Partile,
  Siege,
  PlaceReading,
  FortuneRule,
  Lot,
  PtolemaicAspect,
  RadicalGround,
  ApplicationKind,
  ImpedimentKind,
  BirthEclipse,
  Way,
  Motion,
  WesternAspect,
  AspectPhase,
  MoonPhase,
  Lilith,
  GocharVerdict,
  NodeObstruction,
  NodeVedha,
  Relationship,
  Scale,
  Status,
  Strength,
  Tithi,
  TithiClass,
  TimeScale,
  Vara,
  Varga,
  Yoga,
  YearYoga,
  Saham,
  SahamStrong,
  SahamWeak,
  HarshaGrade,
  TajikaRelation,
  TajikaDrishti,
  TajikaYoga,
  VarsheshaChosen,
  GhatiReckoning,
  Affliction,
  PolarDayPolicy,
  PolarKind,
  Sunrise,
  Longitude,
  BlackoutKind,
  LunarEclipseKind,
  SolarEclipseKind,
  Varna,
  Yoni,
  Gana,
  Nadi,
  VashyaRelation,
  DhinamRule,
  Rajju,
  Koota,
  YoniRelation,
  MaitriRelation,
  NameVarga,
  Overflow,
  VargaRelation,
} from './catalogue.js';
import type {
  CalendarDate,
  CivilDateTime,
  ContextOptions,
  DateSpan,
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
import type { FeedAlarm, FeedHint, FeedSelector, SuppliedEvent } from './requests.js';
import type { EntityForms, Messages } from './messages.js';
import type { Provenance, Step } from './records.js';
// The request records, generated from the façade's own schemas of them
// (`03-design/generated-requests.md`).
import type {
  AccidentalRules,
  AccidentalScores,
  ActivityRules,
  AlmutenRules,
  Anniversary,
  AnnualDashaRules,
  AntisciaRequest,
  AshauchKind,
  AshauchLength,
  AshauchRelation,
  AshauchRequest,
  AshauchRunning,
  AshauchSex,
  AshauchTable,
  AshauchTradition,
  BaselinePeriod,
  BaselineRectificationRequest,
  BirthDate,
  BirthFacts,
  ChakraVowel,
  ConsiderationRules,
  Criterion,
  DayFifth,
  DignityRequest,
  DignityScores,
  DossierRequest,
  EkadashiRule,
  EkadashiVedha,
  FestivalCase,
  FestivalChoice,
  FestivalWhich,
  FirstYearRequest,
  FollowingRule,
  FortitudeRequest,
  GeoPlace,
  HarmonicRequest,
  HarshaRules,
  HitRequest,
  DateTextRequest,
  InWordsRequest,
  TimeTextRequest,
  Interval,
  KootaRules,
  KpRequest,
  KujaLifts,
  KujaRules,
  LalKitabRequest,
  LifeEventKind,
  LotRequest,
  MahadvadashiKind,
  MasaDashaRules,
  MasaPhalaRules,
  MaseshaRules,
  MatchingRequest,
  MidpointRequest,
  MuhurtaBaselineEvent,
  MuhurtaClauseKind,
  MuhurtaGrade,
  MuhurtaNative,
  MuhurtaRanking,
  MuhurtaRequest,
  MuhurtaUnjudged,
  MuhurtaUnwanted,
  MunthaDegree,
  NaamRules,
  NameSounds,
  Birth,
  Book,
  NameClause,
  FinalA,
  NamingRules,
  SuggestFilter,
  SyllablesOf,
  NatalPoint,
  NightRule,
  NumerologyRequestRules,
  OfficeRules,
  PakshiBird,
  ParallelRequest,
  Partner,
  PerfectionRequest,
  PlanRequest,
  PoruthamRules,
  PrashnaRequest,
  PrashnaRules,
  ProgressionsRequest,
  PublishedSet,
  PurifierRules,
  RectificationRequest,
  RemedyRequest,
  RemedyRules,
  ResearchDesign,
  ResearchReferents,
  ReturnLongitude,
  RuleRequest,
  SadeSatiRequest,
  SahamRules,
  SahamStrengthReadings,
  SaitRule,
  SarvatobhadraRequest,
  SarvatobhadraRules,
  Tara,
  TithiPraveshaRequest,
  TithiRule,
  VarsheshaRules,
  VedhaMotion,
  VrataParana,
  WesternAspectRequest,
  YogaRules,
  ChangeKind,
  AnniversariesRequest,
  ChangesRequest,
} from './requests.js';

export * from './catalogue.js';
export * from './records.js';
export type * from './types.js';
export type * from './blob.js';
export type * from './requests.js';
// A request record named like a catalogue enum, an answer record or a
// boundary type is exported under that name as the other: the request's
// own is reached through the field that takes it.
export { Partile, Siege, Terms } from './catalogue.js';
export type {
  CalendarResolution,
  CalendarResolutionComputed,
  CalendarResolutionDefined,
  CalendarResolutionDivergent,
  CalendarResolutionTabular,
  Confidence,
  MonthDay,
} from './records.js';
export type { CalendarDate, CivilTime, ZoneSpec } from './types.js';
/** One subject of an event study: a birth and the event of its life. */
export interface ResearchSubject {
  readonly birth: ResearchBirth;
  /** When the event happened, a Julian day in UTC. */
  readonly event: number;
}

/**
 * A date in a calendar, without naming the fields a call fills in: the era
 * and the era year are what the call resolves them to, and the resolution
 * is `DEFINED`, which is what a date a caller states means. `leap` marks
 * the first of a repeated month or day, in a calendar that repeats them
 * (the Tibetan); every other calendar refuses it.
 *
 * @example date(Calendar.Gregorian, 2015, 4, 14)
 * @example date(Calendar.TibetanPhugpa, 2000, 1, 1, Leap.Month)
 */
export declare function date(calendar: Calendar, year: number, month: number, day: number, leap?: Leap): CalendarDate;

/**
 * A date at a time of day.
 *
 * @example at(date(Calendar.Gregorian, 1986, 1, 1), { hour: 0, minute: 20 })
 */
export declare function at(
  day: CalendarDate,
  time?: {
    readonly hour?: number;
    readonly minute?: number;
    readonly second?: number;
    readonly nanos?: number;
  },
): CivilDateTime;

/**
 * A date whose time of day is unknown. Nothing guesses one: unless the
 * profile sets `time.unknown_time`, a resolution refuses it by name.
 *
 * @example whenUnknown(date(Calendar.Gregorian, 1986, 1, 1))
 */
export declare function whenUnknown(day: CalendarDate): CivilDateTime;

/**
 * The hit list that is each graha's returns: its conjunction, at 0°, with
 * its own natal place (`03-design/western-returns.md`): the Moon's by
 * default, the lunar return; the Sun's is the solar. Asked for several
 * grahas, the list also holds each one's crossing of another's natal
 * place, its `to` naming the place.
 *
 * @example
 * const lunar = ctx.chart.found({ ...birth, hits: returnsRequest(from, to) }).hits;
 * // Each return's chart, erected where the native is (C258).
 * const figure = ctx.chart.found({ instant: lunar[0].instant, place, utcOffsetSeconds });
 */
export declare function returnsRequest(
  from: number,
  to: number,
  grahas?: readonly GrahaName[],
): HitRequest;

/**
 * A period already running, as an ashauch request's `during` reads it:
 * the parts of an earlier `almanac.ashauch` answer's value the overlap
 * rules need, and nothing else.
 *
 * @example
 * const first = ctx.almanac.ashauch({ place, utcOffsetSeconds, request: grandfather }).value;
 * ctx.almanac.ashauch({ place, utcOffsetSeconds, request: { ...uncle, during: [ashauchRunning(first)] } });
 */
export declare function ashauchRunning(answer: Ashauch): AshauchRunning;

/** A zone of the embedded database, by its IANA name. */
export declare function ianaZone(name: string): ZoneSpec;

/** A fixed offset from UTC, in seconds east. */
export declare function fixedZone(offsetSeconds: number): ZoneSpec;

/**
 * Local mean time at a longitude east of Greenwich, which is what a chart
 * from before the zone existed is cast in.
 */
export declare function localMeanZone(longitudeDeg: Longitude): ZoneSpec;

export { BlobFormatError, decodeCharts, decodeDigest, decodeIntlRender, decodeNaam, decodePanchanga, decodePositions } from './blob.js';
export { entityForms, messages } from './messages.js';

/** The stable code of the `INTERNAL` status, the same in every binding. */
export declare const INTERNAL_CODE: -10;

/** What a host's crash reporter hears of the SDK failing inside itself. */
export interface InternalReport {
  readonly status: 'INTERNAL';
  /** `INTERNAL_CODE`. */
  readonly code: number;
  /** A panic the C ABI caught, or a wasm module that trapped. */
  readonly kind: 'boundary' | 'trap';
  /** The SDK that failed. */
  readonly sdkVersion: string;
}

/**
 * Calls `hook` with every `InternalReport` from now on: the status, its
 * code, its kind and the version, never the message, which may quote what
 * the call was given. The function returned stops that.
 */
export declare function onInternalError(hook: (report: InternalReport) => void): () => void;

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
  /** The provider's own code when the status is `PROVIDER`. */
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

/** Positions over a grid, with the cells readable one at a time. */
export declare class Positions extends Decoded<DecodedPositions> {
  /** The instants of the request, in order. */
  readonly instants: Float64Array;
  /** The bodies of the request, in order, by their catalogue key. */
  readonly bodies: readonly Body[];
  /** The bodies as the ids the blob carries, without a copy. */
  readonly bodyIds: Uint16Array;
  /**
   * How many instants the grid covers: with `bodyCount`, the stride a
   * caller needs to read a column — cell `i * bodyCount + j` is instant
   * `i`, body `j`.
   */
  readonly jdCount: number;
  /** How many bodies the grid covers. */
  readonly bodyCount: number;
  /** The time scale the instants are on. */
  readonly scale: TimeScale | 'unknown';
  /** The cells, instants outermost, as typed arrays over the blob. */
  readonly cells: DecodedPositions['cells'];
  /** The completion steps the SDK applied, in order. */
  readonly steps: readonly Step[];
  /** Everything that reproduces this result (ADR-0020). */
  readonly provenance: Provenance;
  /** The provenance envelope as the canonical JSON the library stamped: the bytes to store beside the result, byte-identical in every binding. */
  readonly provenanceJson: string;
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

/**
 * The Black Moon Lilith, placed as a graha is (`03-design/lilith.md`): the
 * lunar apogee the request's `lilith` named, in the chart's zodiac and from
 * its centre. The mean apogee is a direction on the ecliptic, so its
 * `latitudeDeg` and `distanceAu` are zero; only the osculating one is ever
 * retrograde.
 */
export interface LilithPosition extends Omit<PlacedGraha, 'graha'> {
  /** Which lunar apogee. */
  readonly apogee: Exclude<Lilith, 'NONE'> | 'unknown';
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

/**
 * How near a longitude stands to the boundaries that would change how it
 * reads, degrees: the edge of its sign, its nakshatra and its pada. What an
 * ayanamsha that moved would change first.
 */
export interface Boundaries {
  /** Degrees to the nearer edge of its sign. */
  readonly signDeg: number;
  /** Degrees to the nearer edge of its nakshatra. */
  readonly nakshatraDeg: number;
  /** Degrees to the nearer edge of its pada. */
  readonly padaDeg: number;
}

/** Where a divisional chart puts one longitude. */
export interface DivisionalPlacement {
  /** The sign the longitude stands in. */
  readonly rashi: Rashi | 'unknown';
  /** Which part of that sign, counted from zero. */
  readonly part: number;
  /** The sign the divisional chart puts it in; the same as `rashi` is vargottama in the navamsha. */
  readonly sign: Rashi | 'unknown';
}

/** A point in a drawing's unit square, y downwards. */
export interface UnitPoint {
  readonly x: number;
  readonly y: number;
}

/** One step of an outline, from wherever the previous step ended. */
export type Segment =
  | { readonly kind: 'LINE'; readonly to: UnitPoint }
  | { readonly kind: 'QUAD'; readonly control: UnitPoint; readonly to: UnitPoint }
  | {
      readonly kind: 'ARC';
      /** The circle's centre. */
      readonly centre: UnitPoint;
      /** Which way the arc runs, as a reader sees it. */
      readonly clockwise: boolean;
      readonly to: UnitPoint;
    };

/** A closed outline: a start and the steps back to it. */
export interface Outline {
  readonly start: UnitPoint;
  readonly segments: readonly Segment[];
}

/** One region of a drawn chart. */
export interface DrawnCell {
  /** The region's outline in the unit square. */
  readonly outline: Outline;
  /** The sign the cell shows; for a house between cusps, its cusp's sign. */
  readonly sign: Rashi;
  /** The house the cell shows, 1 to 12. */
  readonly house: number;
  /** Whether the lagna stands in this cell. */
  readonly lagna: boolean;
  /** The ring, innermost 0; a grid's cells are all 0. */
  readonly ring: number;
  /** Where the sign or house number is drawn. */
  readonly label: UnitPoint;
  /** Where the cell's bodies are stacked about. */
  readonly anchor: UnitPoint;
  /** The bodies in the cell, as catalogue keys (`graha.SUN`). */
  readonly bodies: readonly string[];
}

/** A chart drawn in a layout (`03-design/chart-geometry.md`). */
/** One period of a dasha. */
export interface DashaPeriod {
  /** Its place at each level from the mahadasha down, joined by `/`: `2/5/3`. */
  readonly path: string;
  /** How deep: 1 for a mahadasha. */
  readonly level: number;
  /** The sign it is the period of, in a sign-based dasha; `null` otherwise. */
  readonly sign: Rashi | null;
  /** Its lord. */
  readonly lord: Graha;
  /** When it begins, a Julian day (UTC). */
  readonly from: number;
  /** When it ends, a Julian day (UTC). */
  readonly to: number;
}

/**
 * A dasha of a founded chart: its periods, and for a nakshatra-seeded one its
 * seed and balance at birth. A sign-based dasha has neither, and its periods
 * name their signs.
 */
export interface Dasha {
  /** Which system: a catalogued one, or a registered one by its full key. */
  readonly system: DashaSystem | DashaKey;
  /** The nakshatra the Moon stood in, which seeds it; `null` for a sign-based dasha. */
  readonly seed: Nakshatra | null;
  /** The lord it starts with. */
  readonly firstLord: Graha;
  /** Whether the seed lay outside a conditional system's nakshatras. */
  readonly overflow: boolean;
  /** What remained of the first period at birth; `null` for a sign-based dasha, whose first period runs whole from birth. */
  readonly balance: {
    /** How it was measured. */
    readonly method: Balance | 'unknown';
    /** The fraction still to run, 0 to 1. */
    readonly remaining: number;
    /** That fraction of the first lord's years, in days. */
    readonly days: number;
    /** The same written as years, months, days, hours and minutes. */
    readonly written: {
      readonly years: number;
      readonly months: number;
      readonly days: number;
      readonly hours: number;
      readonly minutes: number;
    };
  } | null;
  /** The Moon's stay in its nakshatra, when the balance read one. */
  readonly moonSpan: { readonly from: number; readonly to: number } | null;
  /** How many levels the periods go down. */
  readonly depth: number;
  /** Every period of the birth cycle to `depth`, depth first in time order. */
  readonly periods: readonly DashaPeriod[];
  /**
   * The periods running at a Julian day (UTC), from the mahadasha down to
   * `depth`; empty before birth and past the end of the cycle.
   */
  at(jd: number): readonly DashaPeriod[];
}

/** One graha's Ashtakavarga. */
export interface GrahaAshtakavarga {
  /** Which graha, Sun to Saturn. */
  readonly graha: Graha;
  /** Its bindus by sign, Aries to Pisces, 0 to 8. */
  readonly bindus: readonly number[];
  /** The same after both reductions, when they were made in each graha's own; `null` otherwise. */
  readonly reduced: readonly number[] | null;
  /** Its rashi pinda. */
  readonly rashiPinda: number;
  /** Its graha pinda. */
  readonly grahaPinda: number;
  /** Its yoga pinda, the two together. */
  readonly yogaPinda: number;
}

/** A chart's Ashtakavarga: each graha's, the sarvashtakavarga, and their reductions. */
export interface Ashtakavarga {
  /** Where the reductions and pindas were made. */
  readonly shodhana: Shodhana;
  /** How a co-ruled sign beside an occupied one was reduced. */
  readonly ekadhipatya: Ekadhipatya;
  /** Each graha's, Sun to Saturn. */
  readonly grahas: readonly GrahaAshtakavarga[];
  /** The seven grahas' bindus by sign, 337 in all. */
  readonly sarva: readonly number[];
  /** The sum after the trine reduction. */
  readonly trikona: readonly number[];
  /** The sum after both reductions. */
  readonly reduced: readonly number[];
}

/** One graha's Vimshopaka, each score out of 20. */
export interface GrahaVimshopaka {
  /** Which graha, Sun to Saturn. */
  readonly graha: Graha;
  /** Over the six vargas. */
  readonly shadvarga: number;
  /** Over the seven. */
  readonly saptavarga: number;
  /** Over the ten. */
  readonly dashavarga: number;
  /** Over the sixteen. */
  readonly shodashavarga: number;
}

/** A graha's Sthana bala by component, virupas. */
export interface SthanaBala {
  /** From its distance to its debilitation point, 0 to 60. */
  readonly uchcha: number;
  /** From its dignity in the seven vargas. */
  readonly saptavargaja: number;
  /** From its rasi's and navamsha's parity, 0, 15 or 30. */
  readonly ojayugma: number;
  /** From its house: 60, 30 or 15. */
  readonly kendradi: number;
  /** From its decanate: 0 or 15. */
  readonly drekkana: number;
}

/** A graha's Kaala bala by component, virupas. */
export interface KaalaBala {
  /** From the hour, 0 to 60. */
  readonly nathonnatha: number;
  /** From the Moon's elongation, the Moon's doubled. */
  readonly paksha: number;
  /** 60 to the lord of the third of the day or night, and to Jupiter. */
  readonly tribhaga: number;
  /** 15 to the year's lord. */
  readonly abda: number;
  /** 30 to the month's lord. */
  readonly masa: number;
  /** 45 to the weekday's lord. */
  readonly vara: number;
  /** 60 to the hour's lord. */
  readonly hora: number;
  /** From its declination. */
  readonly ayana: number;
  /** Gained by the victor and lost by the vanquished of a planetary war. */
  readonly yuddha: number;
}

/** One graha's Shadbala, in virupas. */
export interface GrahaShadbala {
  /** Which graha, Sun to Saturn. */
  readonly graha: Graha;
  /** Positional strength by component. */
  readonly sthana: SthanaBala;
  /** Directional strength, 0 to 60. */
  readonly dig: number;
  /** Temporal strength by component. */
  readonly kaala: KaalaBala;
  /** Motional strength. */
  readonly cheshta: number;
  /** Natural strength. */
  readonly naisargika: number;
  /** Aspectual strength, which may be negative. */
  readonly drik: number;
  /** The six together. */
  readonly virupas: number;
  /** The six together, in rupas. */
  readonly rupas: number;
  /** The rupas it must reach to be strong. */
  readonly requiredRupas: number;
  /** Whether it reaches them. */
  readonly strong: boolean;
  /** How far it tends to good, 0 to 60 (BPHS ch. 28). */
  readonly ishta: number;
  /** How far it tends to harm, 0 to 60. */
  readonly kashta: number;
  /** Its auspicious rays, 1 to 7: the mean of its Uchcha and Cheshta rays (BPHS ch. 28 v. 5). */
  readonly subhaRashmi: number;
  /** Its inauspicious rays, 8 less the auspicious. */
  readonly ashubhaRashmi: number;
}

/** A chart's Shadbala, read under the context's `strength.*` settings. */
export interface Shadbala {
  /** Each graha's, Sun to Saturn. */
  readonly grahas: readonly GrahaShadbala[];
}

/** One bhava's Bhava bala, in virupas. */
export interface BhavaStrength {
  /** Which bhava, 1 to 12. */
  readonly bhava: number;
  /** The lord of the sign its madhya falls in. */
  readonly lord: Graha;
  /** The lord's Shadbala. */
  readonly adhipati: number;
  /** From its direction, 0 to 60. */
  readonly dig: number;
  /** From the drishtis it receives, which may be negative. */
  readonly drishti: number;
  /** From its occupants and its sign's rising, under BPHS's special rules. */
  readonly special: number;
  /** The four together. */
  readonly virupas: number;
}

/** A chart's Bhava bala, read under the context's `strength.bhava_*` settings. */
export interface BhavaBala {
  /** Each bhava's, the first to the twelfth. */
  readonly bhavas: readonly BhavaStrength[];
}

/** A graha's standing in one scheme of vargas. */
export interface VaiseshikamsaStanding {
  /** How many of the scheme's vargas are good for it. */
  readonly goodVargas: number;
  /** The name that count earns, from two good vargas; `null` below. */
  readonly name: Vaiseshikamsa | null;
}

/** One graha's Vaiseshikamsa (BPHS ch. 6 vv. 42 to 53). */
export interface GrahaVaiseshikamsa {
  /** Which graha, Sun to Saturn. */
  readonly graha: Graha;
  /** Over the six vargas. */
  readonly shadvarga: VaiseshikamsaStanding;
  /** Over the seven. */
  readonly saptavarga: VaiseshikamsaStanding;
  /** Over the ten. */
  readonly dashavarga: VaiseshikamsaStanding;
  /** Over the sixteen. */
  readonly shodashavarga: VaiseshikamsaStanding;
  /** Whether it is combust, defeated in war or in Shayana, its names then not auspicious. */
  readonly impaired: boolean;
}

/** One graha's dasha phala (BPHS ch. 28 vv. 7 to 10, ch. 47 vv. 3 to 6). */
export interface GrahaDashaPhala {
  /** Which graha, Sun to Ketu. */
  readonly graha: Graha | 'unknown';
  /** Its Subhanka in the D1, D2, D3, D7, D9, D12 and D30: out of 60 in the first and 30 in the rest. */
  readonly subhankas: readonly number[];
  /** The seven together, out of 240. */
  readonly subhanka: number;
  /** Their complements together, out of 240. */
  readonly asubhanka: number;
  /** Whether its rasi place is auspicious (benefic), neutral or inauspicious (malefic). */
  readonly nature: Nature;
  /** Where in its dasha its effects come. */
  readonly phase: DashaPhase | 'unknown';
  /** Whether its placement makes its dasha favourable. */
  readonly favourable: boolean;
  /** Whether its placement makes its dasha unfavourable; both can hold. */
  readonly unfavourable: boolean;
}

/** A chart's karakamsha: the Atmakaraka's navamsha sign (BPHS ch. 33 v. 1). */
export interface Karakamsha {
  /** The Atmakaraka, under `jaimini.chara_karakas`. */
  readonly atmakaraka: Graha | 'unknown';
  /** The karakamsha, the Atmakaraka's navamsha sign. */
  readonly sign: Rashi | 'unknown';
  /** Each graha's house from it in the rasi chart, 1 to 12, the Sun to Ketu. */
  readonly inRasi: readonly number[];
  /** Each graha's house from it in the navamsha, 1 to 12, the Sun to Ketu (C130). */
  readonly inNavamsha: readonly number[];
}

/** A chart's Brahma graha, and how it was found (BPHS ch. 46 vv. 170 to 173). */
export interface Brahma {
  /** The rule it was sought under, `jaimini.brahma`. */
  readonly rule: BrahmaRule | 'unknown';
  /** The stronger of the lagna and the 7th, which the rule counts from. */
  readonly countedFrom: Rashi | 'unknown';
  /** The planets that met the rule's marks, in id order. */
  readonly qualified: readonly Graha[];
  /** The Brahma graha; `null` where the rule finds none. */
  readonly graha: Graha | 'unknown' | null;
  /** Saturn or the node that passed Brahma-hood to the planet in the 6th from it (C127). */
  readonly passedFrom: Graha | 'unknown' | null;
  /** Why there is none; `null` when there is one. */
  readonly none: Exclude<BrahmaOutcome, 'FOUND'> | 'unknown' | null;
}

/**
 * A chart's Jaimini significators, read under the settings' `jaimini` group.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, jaimini: true });
 * const brahma = chart.jaimini?.brahma;
 * if (brahma?.graha === null) console.log(`no Brahma: ${brahma.none}`);
 */
export interface JaiminiReading {
  /** The karakamsha, with every graha's house from it in both charts. */
  readonly karakamsha: Karakamsha;
  /** The Brahma graha, or why there is none. */
  readonly brahma: Brahma;
  /**
   * Each graha's arudha, the Sun to Ketu (BPHS ch. 29 vv. 6 and 7), under
   * `jaimini.graha_arudha_exception`; `null` for a node that owns no sign
   * under `jaimini.node_co_lordship`.
   */
  readonly grahaArudhas: readonly (Rashi | 'unknown' | null)[];
  /**
   * The sthira karakas of BPHS ch. 32 vv. 18 to 20, ten in the table's
   * order: the father's and the mother's the stronger of their pair (C408,
   * C409), and `chosenBy` says how.
   *
   * @example
   * const father = chart.jaimini!.sthiraKarakas.find((k) => k.signification === 'FATHER')!;
   * // father.graha is 'graha.SUN' or 'graha.VENUS', father.chosenBy why
   */
  readonly sthiraKarakas: readonly SthiraKaraka[];
  /**
   * The karaka bhavas of BPHS ch. 32 vv. 22 to 24, seven: the house counted
   * from a graha's sign, whole signs in the rasi chart, and the sign it
   * falls in.
   */
  readonly karakaBhavas: readonly KarakaBhava[];
}

/** One sthira karaka of a chart (`03-design/jaimini-significators.md` §7). */
export interface SthiraKaraka {
  /** What it signifies. */
  readonly signification: Signification | 'unknown';
  /** The grahas the verse names for it, in id order: one, or the pair the stronger is taken of. */
  readonly candidates: readonly Graha[];
  /** The karaka. */
  readonly graha: Graha | 'unknown';
  /** How it was chosen. */
  readonly chosenBy: ChosenBy | 'unknown';
}

/** One karaka bhava of a chart: a house counted from a graha. */
export interface KarakaBhava {
  /** The graha counted from. */
  readonly graha: Graha | 'unknown';
  /** The house counted from its sign, 1 to 12. */
  readonly house: number;
  /** What that house shows. */
  readonly signification: Signification | 'unknown';
  /** The sign it falls in. */
  readonly sign: Rashi | 'unknown';
}

/** The syllable a child is named by: the birth pada's own cell (C297 to C299). */
export interface BirthSyllable {
  /** Its place among the śatapada cakra's 112 cells, 0 for a, Krittika's first. */
  readonly cell: number;
  /** As *Muhurta Chintamani* p. 173 prints it, e.g. `चू`. */
  readonly devanagari: string;
  /** Its IAST, e.g. `cū`. */
  readonly iast: string;
  /** The letter group it begins in (VI.35). */
  readonly varga: NameVarga | 'unknown';
}

/**
 * What a janma-patrika prints of the Moon: its star and pada, the syllable
 * the child is named by, and the readings the Ashta Koota takes of the same
 * Moon (C301). Vashya, paya, disha and tatwa are not here (C300).
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, avakahada: true });
 * console.log(chart.avakahada?.syllable.devanagari);
 */
export interface Avakahada {
  readonly nakshatra: Nakshatra | 'unknown';
  /** The pada, 1 to 4. */
  readonly pada: number;
  readonly rashi: Rashi | 'unknown';
  /** The nakshatra's lord, the Vimshottari dasha's. */
  readonly nakshatraLord: Graha | 'unknown';
  /** The sign's lord, the one Graha Maitri reads. */
  readonly rashiLord: Graha | 'unknown';
  /** The sign's varna, as Varna koota reads it (VI.22). */
  readonly varna: Varna;
  readonly yoni: Yoni;
  readonly gana: Gana;
  readonly nadi: Nadi;
  readonly syllable: BirthSyllable;
  /** The ghata chakra of the Moon's sign: what strikes the native (Muhurta Chintamani XI vv. 27–30, C758). */
  readonly ghata: Ghata;
}

/** What strikes a native of one Moon sign: Muhurta Chintamani's four rows of the ghata chakra (C758). */
export interface Ghata {
  /** The sign the Moon strikes from (v. 27). */
  readonly chandra: Rashi | 'unknown';
  /** The class of tithi that strikes (v. 28). */
  readonly tithi: TithiClass;
  /** The weekday that strikes (v. 29). */
  readonly vara: Vara | 'unknown';
  /** The nakshatra that strikes (v. 30). */
  readonly nakshatra: Nakshatra | 'unknown';
}

/** The sign rising at a chart's instant, and its span (C378). */
export interface RisingSign {
  readonly sign: Rashi | 'unknown';
  /**
   * When it began to rise, a UTC Julian day found to a second; `null` when
   * it was already rising a day before, which only a place inside a polar
   * circle sees.
   */
  readonly rose: number | null;
  /** When the next sign begins to rise, a UTC Julian day; `null` likewise. */
  readonly sets: number | null;
}

/**
 * The eclipse a chart's instant falls in, seen where the chart is cast: a
 * lunar eclipse's umbral phase with the Moon up, or a solar eclipse with
 * the Sun up. A classical astronomy's chart is `UNJUDGED` (C188).
 */
export interface MomentEclipse {
  readonly kind: BirthEclipse | 'unknown';
  /** Its greatest moment, a UT1 Julian day; `null` unless `LUNAR` or `SOLAR`. */
  readonly greatest: number | null;
}

/**
 * The Moon's stay in the nakshatra it holds at a chart's instant, and the
 * two counts a janma patrika prints of it in ghatis of twenty-four minutes
 * (C756): the bhayat, how much had passed, and the bhabhoga, the whole stay,
 * which can run past sixty ghatis.
 */
export interface MoonStay {
  /** When the Moon entered the nakshatra, a UTC Julian day. */
  readonly entered: number;
  /** When it leaves it, a UTC Julian day. */
  readonly leaves: number;
  /** The stay passed at the instant. */
  readonly bhayat: GhatiPala;
  /** The whole stay. */
  readonly bhabhoga: GhatiPala;
}

/**
 * What a chart's instant falls in: the sign rising and its span, which BPHS
 * ch. 92 v. 4's lagna gandanta reads, the eclipse seen where the chart is
 * cast, which Brihat Jataka ch. 6 v. 9 reads (C378), and the Moon's stay in
 * its nakshatra.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, moment: true });
 * console.log(chart.moment?.rising.sign, chart.moment?.eclipse.kind);
 * console.log(chart.moment?.nakshatra?.bhayat, chart.moment?.nakshatra?.bhabhoga);
 */
export interface Moment {
  readonly rising: RisingSign;
  readonly eclipse: MomentEclipse;
  /** The Moon's stay in its nakshatra; `null` on a tropical chart, where a nakshatra means nothing. */
  readonly nakshatra: MoonStay | null;
}

/** What a gochar reading counted its houses from, and that point's sign. */
export interface GocharReference {
  /** The natal Moon (Phaladeepika ch. 26 v. 1) or, asked, the lagna (C139). */
  readonly from: GocharFrom | 'unknown';
  /** Its sign. */
  readonly sign: Rashi | 'unknown';
}

/** One graha's transit, read from the reference sign. */
export interface GrahaGochar {
  readonly graha: Graha | 'unknown';
  /** The sign it transits and its degrees within it, 0 to 30. */
  readonly transit: { readonly sign: Rashi | 'unknown'; readonly degrees: number };
  /** Its house from the reference sign, 1 to 12. */
  readonly house: number;
  /** Whether v. 2 makes a transit of this house good. */
  readonly goodHouse: boolean;
  /** The house whose occupant obstructs it (vv. 3 to 8); `null` where nothing can. */
  readonly vedhaHouse: number | null;
  /** The grahas standing in the vedha house that obstruct it, the verses' exemptions left out. */
  readonly obstructedBy: readonly Graha[];
  /** What the transit comes to. */
  readonly verdict: GocharVerdict | 'unknown';
  /** The decanate in which its transit bears fruit (v. 25). */
  readonly fruition: Fruition | 'unknown';
  /** Whether it stands in that decanate now. */
  readonly fruitfulNow: boolean;
}

/**
 * Every graha's transit at one instant, read from the natal chart
 * (`03-design/gochar.md`).
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, gochar: { instants: [2460676.5] } });
 * const good = chart.gochar[0].grahas.filter((g) => g.verdict === 'GOOD').map((g) => g.graha);
 */
export interface GocharReading {
  /** The instant, a UTC Julian day. */
  readonly instant: number;
  /** What the houses are counted from. */
  readonly reference: GocharReference;
  /** The readings of the nodes the transits were judged under, the settings' `gochar` group. */
  readonly rules: {
    readonly nodeVedha: NodeVedha | 'unknown';
    readonly nodeObstruction: NodeObstruction | 'unknown';
    readonly ashtakavargaGoodFrom: AshtakavargaGoodFrom | 'unknown';
  };
  /** Each graha's, the Sun to Ketu. */
  readonly grahas: readonly GrahaGochar[];
  /** The seven judged by the natal Ashtakavarga, Sun to Saturn; `null` unless `ashtakavarga: true` asked. */
  readonly ashtakavarga: readonly AshtakavargaTransit[] | null;
}

/**
 * One graha's transit judged by the natal Ashtakavarga (Phaladeepika ch. 23;
 * `03-design/gochar-ashtakavarga.md`).
 */
export interface AshtakavargaTransit {
  readonly graha: Graha | 'unknown';
  /** The bindus its own Ashtakavarga put in the sign it transits, 0 to 8 (v. 11). */
  readonly bindus: number;
  /** Whether they reach `gochar.ashtakavarga_good_from`. */
  readonly good: boolean;
  /** The eighth of the sign it stands in, 1 to 8, and its lord (vv. 16 to 19). */
  readonly kakshya: { readonly index: number; readonly lord: KakshyaLord | 'unknown' };
  /** Whether that lord gave a bindu there, so that a bindu bears its fruit now. */
  readonly kakshyaBindu: boolean;
  /** The sign's sarvashtakavarga. */
  readonly sarva: number;
  /** Where it stands against 28 (v. 20). */
  readonly sarvaStanding: SarvaStanding | 'unknown';
}

/** A graha named bare (`'SUN'`), as the Rust key spells it, or full (`'graha.SUN'`). */
export type GrahaName = Exclude<Graha, 'unknown'> | BareKey<Graha>;

/** A catalogue key without its kind: `'graha.SUN'` is `'SUN'`. */
type BareKey<Full> = Full extends `${string}.${infer Bare}` ? Bare : never;

/** What a hit was, tagged by `kind`; each kind carries its own fields. */
export type HitEvent =
  | {
      readonly kind: 'SIGN_INGRESS';
      /** The sign entered; a retrograde ingress enters the one before its line. */
      readonly into: Rashi | 'unknown';
      readonly motion: Motion | 'unknown';
    }
  | {
      readonly kind: 'NAKSHATRA_INGRESS';
      readonly into: Nakshatra | 'unknown';
      readonly motion: Motion | 'unknown';
    }
  | {
      readonly kind: 'STATION';
      /** The motion the graha turned to. */
      readonly turns: Motion | 'unknown';
    }
  | {
      readonly kind: 'ASPECT';
      readonly to: NatalPoint;
      /** The angle, 0 to 180 degrees, either side of the natal point (C145). */
      readonly angle: number;
      /** Where in the orb's window (C146); always `'EXACT'` without an orb. */
      readonly phase: AspectPhase | 'unknown';
      readonly motion: Motion | 'unknown';
    }
  | {
      readonly kind: 'VARGA_INGRESS';
      /** The divisional chart whose sign changed (C418). */
      readonly varga: Varga | 'unknown';
      /** The sign entered in that chart. */
      readonly into: Rashi | 'unknown';
      readonly motion: Motion | 'unknown';
    }
  | {
      readonly kind: 'LUNAR_PHASE';
      /** Which of the four the Moon reached, in the chart's frame (C419). */
      readonly phase: MoonPhase | 'unknown';
    }
  | {
      /** The Moon's elongation from the Sun came back to the natal chart's (C420). */
      readonly kind: 'PHASE_RETURN';
    }
  | {
      readonly kind: 'SOLAR_ECLIPSE';
      /** The natal point the eclipsed Sun fell on, within `eclipseOrbDeg` (C421). */
      readonly to: NatalPoint;
      readonly eclipse: SolarEclipseKind | 'unknown';
    }
  | {
      readonly kind: 'LUNAR_ECLIPSE';
      /** The natal point the eclipsed Moon fell on, within `eclipseOrbDeg` (C421). */
      readonly to: NatalPoint;
      readonly eclipse: LunarEclipseKind | 'unknown';
    };

/**
 * One event of the transit hit list (`03-design/transit-hit-list.md`).
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, hits: { from: 2460676.5, to: 2461041.5 } });
 * const saturn = chart.hits.filter((h) => h.graha === 'graha.SATURN' && h.event.kind === 'SIGN_INGRESS');
 */
export interface Hit {
  /** When, as a UTC Julian day. */
  readonly instant: number;
  /** The transiting graha. */
  readonly graha: Graha | 'unknown';
  readonly event: HitEvent;
}

/** The instant a rashifal period's sky is read at (C358). */
export type RashifalSnapshot =
  | { readonly at: 'SUNRISE' }
  /** A clock time on the reference day at the request's offset; the baseline engine's is 06:00. */
  | { readonly at: 'CLOCK'; readonly hour: number; readonly minute: number };

/**
 * A period of civil days at a place, to read for each of the twelve signs;
 * everything but the days, the place and the offset is optional.
 */
export interface RashifalRequest {
  /** The first day; its calendar is the period's, Gregorian when left out. */
  readonly first: { readonly calendar?: Calendar; readonly year: number; readonly month: number; readonly day: number };
  /** The last day, both ends included; the first when left out. */
  readonly last?: { readonly year: number; readonly month: number; readonly day: number };
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The civil days' offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** Sunrise on the middle day by default (C358, C359). */
  readonly snapshot?: RashifalSnapshot;
  /** The grahas whose ingresses and stations are reported; every one but the Moon by default (C360). */
  readonly events?: readonly GrahaName[];
  /** The houses counted as Saturn's smaller spells; the 4th and 8th by default (C149). */
  readonly spells?: readonly number[];
}

/** Where Saturn stands in Sade Sati: the 12th, the 1st or the 2nd. */
export type SadeSatiPhase = 'RISING' | 'PEAK' | 'SETTING';

/** One event of a rashifal period, counted from a sign. */
export interface RashifalEvent {
  /** The event, and the sign it happened in: the one entered, or the one a station stood in. */
  readonly event: { readonly hit: Hit; readonly sign: Rashi | 'unknown' };
  /** That sign's house from the reading's sign, 1 to 12. */
  readonly house: number;
  /** Whether v. 2 makes the graha's transit of that house good. */
  readonly goodHouse: boolean;
}

/** One sign's reading of a period. */
export interface RashiReading {
  /** The sign, taken as a reader's janma rashi. */
  readonly rashi: Rashi | 'unknown';
  /** Phaladeepika ch. 26's gochar from it at the period's instant. */
  readonly gochar: GocharReading;
  /** Saturn's house from it, its Sade Sati phase, and whether the house is one of the smaller spells. */
  readonly saturn: { readonly house: number; readonly sadeSati: SadeSatiPhase | null; readonly spell: boolean };
  /** Every event of the period, in time order, counted from it. */
  readonly events: readonly RashifalEvent[];
}

/** One period read for each of the twelve signs. */
export interface RashifalPeriod {
  readonly first: CalendarDate;
  readonly last: CalendarDate;
  /** The day it is read at, the middle one (C359). */
  readonly reference: CalendarDate;
  /** The instant it is read at, a UTC Julian day (C358). */
  readonly instant: number;
  /** Each graha's sign and degrees then, the Sun to Ketu. */
  readonly transits: readonly { readonly sign: Rashi | 'unknown'; readonly degrees: number }[];
  /** Whether each was moving backwards then, the Sun to Ketu. */
  readonly retrograde: readonly boolean[];
  /** What the baseline's score reads of the reference day's panchanga at sunrise. */
  readonly panchanga: { readonly tithi: Tithi | 'unknown'; readonly yoga: Yoga | 'unknown'; readonly muhurtaYogas: number };
  /** Each sign's reading, Aries to Pisces. */
  readonly readings: readonly RashiReading[];
}

/** The baseline engine's score of one sign's reading, `BASELINE` and unsourced (C361). */
export interface BaselineScore {
  /** 0 to 100. */
  readonly overall: number;
  /** The eight life areas, each 0 to 100, in the baseline's order. */
  readonly areas: readonly {
    readonly area: 'OVERALL' | 'CAREER' | 'FINANCE' | 'HEALTH' | 'RELATIONSHIPS' | 'FAMILY' | 'EDUCATION' | 'SPIRITUALITY';
    readonly score: number;
  }[];
  /** The five grahas the baseline names, in its order. */
  readonly keyInfluences: readonly { readonly graha: Graha | 'unknown'; readonly house: number; readonly verdict: GocharVerdict }[];
  /** The sign lord's lucky elements. */
  readonly lucky: { readonly colour: string; readonly number: number; readonly day: Vara | 'unknown'; readonly direction: Direction | 'unknown' };
}

/** One period's answer: the reading, and the baseline's twelve scores when asked. */
export interface RashifalAnswer {
  readonly period: RashifalPeriod;
  /** Aries to Pisces; `null` unless a baseline period was asked. */
  readonly baseline: readonly BaselineScore[] | null;
}

/** One period's answer as `rashifal` hands it out, with what sealed it. */
export interface RashifalSealed extends RashifalAnswer {
  /** What computed it, and under what; `inputHash` seals the request. */
  readonly provenance: Provenance;
}

/** Many periods' answers, in the requests' order, under one provenance. */
export interface RashifalAnswers {
  readonly value: readonly RashifalAnswer[];
  /** What computed them, and under what; `inputHash` seals the request. */
  readonly provenance: Provenance;
}

/** One stay of Saturn's in a house, half-open. */
export interface SadeSatiVisit {
  /** When Saturn entered, a UTC Julian day; `null` before the ephemeris's coverage. */
  readonly from: number | null;
  /** When it left; `null` after the ephemeris's coverage. */
  readonly to: number | null;
}

/** Every stay of Saturn's in one house of one period, a retrograde re-entry a visit of its own (C148). */
export interface SadeSatiSpell {
  /** The house from the reference, 1 to 12. */
  readonly house: number;
  readonly visits: readonly SadeSatiVisit[];
}

/** One Sade Sati: the rising (12th), peak (1st) and setting (2nd) spells, in order. */
export interface SadeSati {
  readonly phases: readonly SadeSatiSpell[];
}

/**
 * A chart's Sade Satis and smaller spells, each period whole however far
 * its bounds fall outside the window asked about.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, sadeSati: { from: 2460676.5, to: 2464329 } });
 * const peak = chart.sadeSati?.sadeSati[0]?.phases[1]?.visits[0];
 */
export interface SadeSatiReport {
  /** What the houses were counted from, and that point's sign. */
  readonly reference: GocharReference;
  readonly reckoning: Reckoning | 'unknown';
  /** Every Sade Sati reaching into the window, in time order. */
  readonly sadeSati: readonly SadeSati[];
  /** The smaller spells asked for, in time order. */
  readonly spells: readonly SadeSatiSpell[];
}

/** The dignities and debilities a planet holds where it stands. */
export interface EssentialDignity {
  readonly house: boolean;
  readonly exaltation: boolean;
  readonly triplicity: boolean;
  readonly term: boolean;
  readonly face: boolean;
  readonly detriment: boolean;
  readonly fall: boolean;
}

/** One planet's dignities and its score. */
export interface PlanetDignity {
  readonly planet: Graha;
  /** Degrees of the chart's zodiac. */
  readonly longitudeDeg: number;
  readonly dignity: EssentialDignity;
  /** Whether it is in none of its five dignities, whatever its debilities. */
  readonly peregrine: boolean;
  /** Its score from its own dignities alone. */
  readonly score: number;
  /**
   * What Lilly's table adds for mutual reception (p. 115): the house's
   * score when received by house, the exaltation's when by exaltation,
   * nothing for a mixed reception or one by a lesser dignity (C210). A
   * total is `score + reception`.
   */
  readonly reception: number;
}

/** One of the five essential dignities, by the name its flag has. */
export type DignityKind = 'house' | 'exaltation' | 'triplicity' | 'term' | 'face';

/**
 * Two planets each standing in at least one of the other's five
 * dignities (Lilly, p. 112), each side reported whole.
 *
 * @example
 * const byHouse = chart.dignities?.receptions.filter((one) => one.mutual.includes('house'));
 */
export interface Reception {
  /** The two, in the Chaldean order. */
  readonly planets: readonly [Graha, Graha];
  /** The second's dignities where the first stands. */
  readonly firstIn: EssentialDignity;
  /** The first's dignities where the second stands. */
  readonly secondIn: EssentialDignity;
  /** The kinds each stands in of the other's, strongest first; empty for a mixed reception. */
  readonly mutual: readonly DignityKind[];
}

/**
 * A chart's essential dignities, with everything that made them: the sect,
 * the rule that chose it, the terms (`'TABLE'` for the request's own
 * table), the triplicities and the scores.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, dignities: {} });
 * const mars = chart.dignities?.planets.find((at) => at.planet === 'graha.MARS');
 */
export interface Dignities {
  readonly sect: Sect | 'unknown';
  readonly sectRule: SectRule | 'unknown';
  readonly rules: {
    readonly terms: Terms | 'unknown';
    readonly triplicities: Triplicities | 'unknown';
  };
  readonly scores: DignityScores;
  /** The seven in the Chaldean order, Saturn first. */
  readonly planets: readonly PlanetDignity[];
  /** Every pair in reception, in the Chaldean order of the first and then the second. */
  readonly receptions: readonly Reception[];
  /**
   * The seven's chains of dispositors under the traditional rulers, where
   * they stop, and the final dispositor when every chain stops at one
   * (`03-design/essential-dignities.md` §Dispositors).
   *
   * @example
   * const { finalDispositor, chains } = chart.dignities!.dispositions;
   * const sun = chains.find((c) => c.planet === 'graha.SUN')!;
   * // sun.dispositors: the Sun's dispositor, its dispositor, and so on
   */
  readonly dispositions: Dispositions;
}

/** A chart's dispositors (`03-design/essential-dignities.md` §Dispositors). */
export interface Dispositions {
  /** The rulers the chains were followed under: the dignities' own, always traditional. */
  readonly rulers: 'TRADITIONAL';
  /** One chain a planet, in the order `planets` gives them. */
  readonly chains: readonly DispositorChain[];
  /** Where the chains stop, each once, in the order the chains first reach it. */
  readonly ends: readonly DispositorEnd[];
  /** The one planet in its own sign every chain stops at; `null` for more than one end or a cycle (C415). */
  readonly finalDispositor: Graha | 'unknown' | null;
}

/** One planet's dispositors, followed until one repeats. */
export interface DispositorChain {
  /** The planet the chain starts from. */
  readonly planet: Graha | 'unknown';
  /** Its dispositor, that planet's, and so on; empty for a planet in its own sign. */
  readonly dispositors: readonly (Graha | 'unknown')[];
  /** Where it stops: one of `ends`, the same object. */
  readonly end: DispositorEnd;
}

/** Where chains of dispositors stop. */
export interface DispositorEnd {
  /** A planet in its own sign, or a cycle. */
  readonly kind: ChainEnd | 'unknown';
  /** The planet in its own sign, or the cycle's from its slowest member. */
  readonly planets: readonly (Graha | 'unknown')[];
}

/** What a chart's accidental fortitudes were read from, in the chart's zodiac. */
export interface AccidentalSky {
  /** Regiomontanus, Lilly's, unless a profile names another division for the `hellenistic` module. */
  readonly houses: HouseSystem;
  /** The twelve cusps, the first to the twelfth, in degrees. */
  readonly cuspsDeg: readonly number[];
  /** The ascendant, from the chart's angles: whole-sign and equal houses do not put it on a cusp. */
  readonly ascendantDeg: number;
  /** The midheaven, from the chart's angles. */
  readonly midheavenDeg: number;
  /** The seven's daily motions in the Chaldean order, negative when retrograde. */
  readonly speedsDegPerDay: readonly number[];
  readonly northNodeDeg: number;
  /** The star's apparent place of date, as are Spica's and Algol's. */
  readonly regulusDeg: number;
  readonly spicaDeg: number;
  readonly algolDeg: number;
}

/** One accidental line a planet meets, and what it scores for that planet. */
export interface AccidentLine {
  readonly accident: Accident | 'unknown';
  /** Orientality scores Saturn, Jupiter and Mars one way and Venus and Mercury the other. */
  readonly points: number;
}

/** One planet's accidental fortitudes and debilities. */
export interface PlanetAccidents {
  readonly planet: Graha;
  /** Its house, 1 to 12, under the five-degree rule. */
  readonly house: number;
  /** Every line beyond its house, in Lilly's order. */
  readonly accidents: readonly AccidentLine[];
  /** The sum of its fortitudes, its house's included. */
  readonly fortitude: number;
  /** The sum of its debilities, its house's included, as a positive number. */
  readonly debility: number;
  /** Lilly's net over the whole table: its essential `score + reception` and `fortitude - debility`. */
  readonly net: number;
}

/**
 * Both halves of Lilly's table in one chart, with everything that made
 * them.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, fortitudes: {} });
 * const strongest = [...(chart.fortitudes?.planets ?? [])].sort((a, b) => b.net - a.net)[0];
 */
export interface Fortitudes {
  /** The essential half, which the chart's `dignities` also reads. */
  readonly dignities: Dignities;
  readonly sky: AccidentalSky;
  readonly rules: AccidentalRules;
  readonly scores: AccidentalScores;
  /** The seven in the Chaldean order, Saturn first. */
  readonly planets: readonly PlanetAccidents[];
  readonly almutens: Almutens;
}

/** One planet's total in an almuten's ranking. */
export interface AlmutenTotal {
  readonly planet: Graha;
  readonly total: number;
}

/**
 * An almuten as a ranking. Lilly breaks no tie, so every planet holding the
 * greatest total is an almuten (C219).
 */
export interface Almuten {
  /** The seven's totals, in the Chaldean order. */
  readonly totals: readonly AlmutenTotal[];
  /** Every planet holding the greatest total: one unless they tie. */
  readonly almutens: readonly Graha[];
  /** Every planet holding the next total down, Chapter CV's partakers; empty when all seven tie. */
  readonly partakers: readonly Graha[];
}

/**
 * A chart's almutens three ways, with the rules that made them.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, fortitudes: { almuten: { place: 'SIGN' } } });
 * const lord = chart.fortitudes?.almutens.figure.almutens; // Lilly's lord of the geniture
 */
export interface Almutens {
  readonly rules: AlmutenRules;
  /** The Part of Fortune, one of the five places. */
  readonly fortuneDeg: number;
  /** Lilly's almuten of the figure: each planet's `net`. */
  readonly figure: Almuten;
  /** Chapter CV's: essential dignities over the ascendant, midheaven, Sun, Moon and Fortune. */
  readonly places: Almuten;
  /** Each house's, of its cusp, the first to the twelfth. */
  readonly houses: readonly Almuten[];
}

/** Where a lot fell, in the chart's zodiac. */
export interface LotPlace {
  /** Degrees in [0, 360). */
  readonly longitudeDeg: number;
  readonly sign: Rashi;
  /** The sign's lord, the lot's ruler. */
  readonly lord: Graha;
  /** 1 to 12, counted in whole signs from the ascendant's sign. */
  readonly house: number;
}

/** One lot and where it fell. */
export interface PlacedLot {
  readonly lot: Lot;
  readonly place: LotPlace;
}

/**
 * A chart's lots, with its sect and the rules they were read under.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, lots: {} });
 * const fortune = chart.lots?.lots.find((at) => at.lot === 'FORTUNE')?.place.sign;
 */
export interface Lots {
  readonly sect: Sect;
  /** The request as it stood, every field filled. */
  readonly request: Required<LotRequest>;
  /** Whether Fortune was counted from the Moon to the Sun, and Daimon the other way. */
  readonly fortuneReversed: boolean;
  /** All fourteen, in the catalogue's order. */
  readonly lots: readonly PlacedLot[];
}

/** An aspect the Moon perfects with a planet. */
export interface Perfection {
  readonly planet: Graha;
  readonly aspect: PtolemaicAspect;
  /** Days until it is exact, at the motions of the moment. */
  readonly days: number;
  /** How far it is from exact now, degrees. */
  readonly gapDeg: number;
}

/** Where the Moon is going before she leaves her sign (p. 112). */
export interface MoonCourse {
  /** Her first perfection before she leaves her sign; `null` when void by that reading. */
  readonly next: Perfection | null;
  /** The first already within the two planets' moieties; `null` when void by Lilly's moieties (C230). */
  readonly withinOrb: Perfection | null;
  /** Days until she leaves her sign. */
  readonly daysInSign: number;
  /** Taurus, Cancer, Sagittarius or Pisces, where void "somewhat she performes". */
  readonly eased: boolean;
}

/**
 * A chart's considerations before judgement, each clause with the facts it
 * rests on and none folded into a verdict.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, considerations: {} });
 * const radical = (chart.considerations?.radicality.grounds.length ?? 0) > 0;
 */
export interface Considerations {
  readonly radicality: {
    /** The lord of the chart's planetary hour. */
    readonly hourLord: Graha;
    readonly ascendantLord: Graha;
    /** Every ground that holds; empty when the figure is not radical. */
    readonly grounds: readonly RadicalGround[];
  };
  readonly ascendant: {
    readonly sign: Rashi;
    /** Degrees within the sign, [0, 30). */
    readonly degree: number;
    /** Fewer than 3 degrees rise. */
    readonly early: boolean;
    /** 27 degrees or more rise. */
    readonly late: boolean;
    /** A sign of short ascension, Capricorn to Gemini. */
    readonly shortAscension: boolean;
  };
  readonly moon: {
    readonly sign: Rashi;
    readonly degree: number;
    readonly late: boolean;
    /** Gemini, Scorpio or Capricorn. */
    readonly lateSign: boolean;
    /** Libra 15° to Scorpio 15°. */
    readonly viaCombusta: boolean;
    readonly course: MoonCourse;
  };
  readonly seventh: {
    readonly cuspDeg: number;
    readonly lord: Graha;
    /** Saturn and Mars, when counted in the seventh house (C231). */
    readonly infortunesInHouse: readonly Graha[];
    readonly lordRetrograde: boolean;
    readonly lordCombust: boolean;
    readonly lordInFall: boolean;
    readonly lordInInfortuneTerm: boolean;
    /** Essential and accidental fortitudes less debilities. */
    readonly lordNet: number;
  };
  /** 1 to 12. */
  readonly saturnHouse: number;
  readonly saturnRetrograde: boolean;
  readonly ascendantLordCombust: boolean;
  /** The request as it stood, every field filled. */
  readonly rules: Required<ConsiderationRules>;
}

/** A span of time a progression's rate is stated in. */
export type ProgressionSpan = 'DAY' | 'SYNODIC_MONTH' | 'SIDEREAL_MONTH' | 'YEAR' | { readonly DAYS: number };

/**
 * The aspect configurations to find in every chart
 * (`03-design/western-configurations.md`): the aspect table the shapes
 * are made of, as a {@link WesternAspectRequest}, and how many bodies in
 * one tropical sign make a stellium. Every field is optional.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, outerPlanets: true,
 *   westernConfigurations: { stelliumLeast: 4 } });
 * for (const shape of chart.westernConfigurations ?? []) {
 *   if (shape.inside.length === 0) console.log(shape.kind, shape.bodies);
 * }
 */
export interface WesternConfigurationRequest extends WesternAspectRequest {
  /** The bodies in one sign that make a stellium, three or more; three by default. */
  readonly stelliumLeast?: number;
}

/** Which aspect configuration a shape is. */
export type WesternConfigurationKind =
  | 'GRAND_TRINE'
  | 'T_SQUARE'
  | 'GRAND_CROSS'
  | 'YOD'
  | 'KITE'
  | 'MYSTIC_RECTANGLE'
  | 'STELLIUM';

/** One shape a chart's planets make, of aspects the chart holds. */
export interface WesternConfiguration {
  readonly kind: WesternConfigurationKind;
  /** Its bodies, in the order the chart's planets are read. */
  readonly bodies: readonly Graha[];
  /** The graha a T-square, yod or kite points at; `null` for the others. */
  readonly apex: Graha | null;
  /** The tropical rashi a stellium stands in; `null` for the others. */
  readonly sign: Rashi | null;
  /** How far from exact its loosest aspect is, degrees; `null` for a stellium. */
  readonly widestFromExactDeg: number | null;
  /** The positions in the chart's list of the larger shapes this one is wholly inside. */
  readonly inside: readonly number[];
}

/**
 * How wide a Western aspect may be (C240): Leo's by aspect, Lilly's
 * moieties by planet, or the caller's own by aspect.
 */
export type WesternOrbModel =
  | { readonly model: 'LEO' }
  | {
      readonly model: 'MOIETIES';
      /** Each planet's whole orb; a pair is within half the sum of theirs. */
      readonly orbs: readonly { readonly graha: GrahaName; readonly orbDeg: number }[];
    }
  | {
      readonly model: 'BY_ASPECT';
      /** Each aspect's orb, whatever the pair. */
      readonly orbs: readonly { readonly aspect: WesternAspect; readonly orbDeg: number }[];
    };

/**
 * What a chart's Western houses are asked (`03-design/western-houses.md`):
 * the division, else the profile's `houses.module_overrides.western`, else
 * Placidus, the division Leo's figures are cast in (C249).
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, westernHouses: {} });
 * for (const planet of chart.westernHouses?.planets ?? []) console.log(planet.graha, planet.house);
 */
export interface WesternHouseRequest {
  readonly system?: HouseSystem;
}

/** Where a planet is counted in a chart's Western houses. */
export interface WesternHousePlacement {
  readonly graha: Graha | 'unknown';
  /** The house whose cusp it has passed and whose next cusp it has not, 1 to 12. */
  readonly house: number;
  /**
   * Whether Leo reads it with the ascendant (C250): in the first house, or
   * above the ascendant no further than `reachDeg`. The house is never moved.
   */
  readonly withAscendant: boolean;
}

/** A point of a harmonic chart: a planet, the ascendant or the midheaven. */
export type HarmonicPoint =
  | { readonly point: 'GRAHA'; readonly graha: Graha | 'unknown' }
  | { readonly point: 'ASCENDANT' }
  | { readonly point: 'MIDHEAVEN' };

/** A point's place in a harmonic chart. */
export interface HarmonicPlaced {
  readonly point: HarmonicPoint;
  /** Its longitude multiplied by the harmonic, degrees in `[0, 360)`. */
  readonly longitudeDeg: number;
  /** Its equal house from the harmonic ascendant, 1 to 12 (C254). */
  readonly house: number;
}

/** Two points meeting in a harmonic chart, within the orb of each other there. */
export interface HarmonicRow {
  /** The earlier point: the planets in the catalogue's order, then the ascendant, then the midheaven. */
  readonly first: HarmonicPoint;
  readonly second: HarmonicPoint;
  /** How far apart they stand in the harmonic chart, degrees. */
  readonly apartDeg: number;
  /** Which multiple k of the harmonic's aspect, k × 360° / n, they stand at in the chart itself. */
  readonly multiple: number;
  /** The orb the request allowed, degrees. */
  readonly orbDeg: number;
}

/** A chart's harmonic chart, in its own zodiac (C253). */
export interface HarmonicChart {
  /** Which harmonic. */
  readonly harmonic: number;
  /** The planets in the catalogue's order, then the ascendant and the midheaven. */
  readonly points: readonly HarmonicPlaced[];
  /** The pairs meeting within the orb, closest first. */
  readonly rows: readonly HarmonicRow[];
}

/** The five exceptions of VI.32–33 that lift a bad Bhakoot, each a clause that holds or not. */
export interface BhakootExceptions {
  /** One lord rules both signs. */
  readonly oneLord: boolean;
  /** The sign lords are each other's friends. */
  readonly lordsFriends: boolean;
  /** The navamsha lords are one or each other's friends. */
  readonly navamshaLordsFriends: boolean;
  /** The tara is pure both ways. */
  readonly taraPure: boolean;
  /** One sign is vashya to the other. */
  readonly vashya: boolean;
}

/** What a koota read, tagged by the koota. */
export type KootaReading =
  | { readonly koota: 'koota.VARNA'; readonly bride: Varna; readonly groom: Varna }
  | { readonly koota: 'koota.VASHYA'; readonly relation: VashyaRelation | 'unknown' }
  | {
      readonly koota: 'koota.TARA';
      /** Counted from the bride's nakshatra to the groom's, 1 to 9; the 3rd, 5th and 7th are bad. */
      readonly brideToGroom: number;
      readonly groomToBride: number;
    }
  | {
      readonly koota: 'koota.YONI';
      readonly bride: Yoni;
      readonly groom: Yoni;
      readonly relation: YoniRelation | 'unknown';
    }
  | {
      readonly koota: 'koota.GRAHA_MAITRI';
      /** The lord of the bride's Moon sign. */
      readonly bride: Graha | 'unknown';
      readonly groom: Graha | 'unknown';
      readonly relation: MaitriRelation | 'unknown';
      /** Whether a good Bhakoot lifts the lords' enmity (VI.33); false with no enmity. */
      readonly lifted: boolean;
    }
  | {
      readonly koota: 'koota.GANA';
      readonly bride: Gana;
      readonly groom: Gana;
      /** Whether a Rakshasa stands beside another gana. */
      readonly dosha: boolean;
      /**
       * Whether the dosha is lifted: the sign lords or the navamsha lords
       * befriended (VI.33), or one sign or one star between the two (VI.36);
       * false with no dosha.
       */
      readonly lifted: boolean;
    }
  | {
      readonly koota: 'koota.BHAKOOT';
      /** The groom's Moon sign counted from the bride's, 1 to 12. */
      readonly apart: number;
      /** The bad Bhakoot the signs stand at, or `null`. */
      readonly dosha: 'SIX_EIGHT' | 'FIVE_NINE' | 'TWO_TWELVE' | 'unknown' | null;
      readonly exceptions: BhakootExceptions;
      /** Whether the exceptions lift the dosha under the rules; false with no dosha. */
      readonly lifted: boolean;
    }
  | {
      readonly koota: 'koota.NADI';
      readonly bride: Nadi;
      readonly groom: Nadi;
      /** Whether the shared nadi is a dosha under the rules. */
      readonly dosha: boolean;
      /**
       * Whether the dosha is lifted by one sign with two stars, one star across
       * two signs or one star in two padas (VI.36); false with no dosha.
       */
      readonly lifted: boolean;
    }
  | { readonly koota: 'unknown' };

/** One koota's points and what it read. */
export interface KootaRow {
  /** Its points, a multiple of a half. */
  readonly points: number;
  /** The most it gives, 1 for Varna to 8 for Nadi. */
  readonly maxPoints: number;
  readonly reading: KootaReading;
}

/** The Ashta Koota of a bride and a groom. */
export interface AshtaKoota {
  /** The eight, in the verse's order. */
  readonly kootas: readonly KootaRow[];
  /** Their points, out of 36. */
  readonly total: number;
}

/** What one of the ten considerations read, tagged by its catalogue koota (C282). */
export type PoruthamReading =
  | {
      readonly koota: 'koota.TARA';
      /** Dhinam: the groom's nakshatra counted from the bride's, 1 to 27. */
      readonly count: number;
      /** The chapter's rule that decided it. */
      readonly rule: DhinamRule | 'unknown';
    }
  | {
      readonly koota: 'koota.GANA';
      readonly bride: Gana;
      readonly groom: Gana;
      /** A Rakshasa's evil "diminishes" by the bride's star beyond the 14th from the groom's (C279). */
      readonly diminished: boolean;
    }
  | { readonly koota: 'koota.MAHENDRA'; readonly count: number }
  | { readonly koota: 'koota.STREE_DEERGHA'; readonly count: number }
  | {
      readonly koota: 'koota.YONI';
      /** On the chapter's own table: Uttarashadha the cow (C278). */
      readonly bride: Yoni;
      readonly groom: Yoni;
      readonly hostile: boolean;
    }
  | {
      readonly koota: 'koota.BHAKOOT';
      /** Rasi: the groom's Moon sign counted from the bride's, 1 to 12. */
      readonly apart: number;
    }
  | {
      readonly koota: 'koota.GRAHA_MAITRI';
      /** Rasyadhipathi: the lord of the bride's Moon sign. */
      readonly bride: Graha | 'unknown';
      readonly groom: Graha | 'unknown';
      /** On the chapter's own friendships; a lord is its own friend. */
      readonly brideCallsFriend: boolean;
      readonly groomCallsFriend: boolean;
    }
  | {
      readonly koota: 'koota.VASHYA';
      /** Vasyam, on p. 75's table: the bride's sign concordant to the groom's. */
      readonly brideToGroom: boolean;
      readonly groomToBride: boolean;
    }
  | { readonly koota: 'koota.RAJJU'; readonly bride: Rajju | 'unknown'; readonly groom: Rajju | 'unknown' }
  | { readonly koota: 'koota.VEDHA'; readonly pierced: boolean }
  | { readonly koota: 'unknown' };

/** One consideration: whether it agrees, and what it read. */
export interface PoruthamRow {
  /** Whether it agrees, a lift included. */
  readonly agrees: boolean;
  /** Whether it agrees only by the p. 76 exception. */
  readonly lifted: boolean;
  readonly reading: PoruthamReading;
}

/** The p. 76 exception's clauses, any one of which lifts Ganam, Rasi, Rajju and Vedhai (C277). */
export interface PoruthamException {
  readonly oneLord: boolean;
  readonly lordsFriendly: boolean;
  readonly opposite: boolean;
}

/** The ten considerations of a bride and a groom (*Kalaprakasika* XIII). */
export interface Porutham {
  /** The ten, in the chapter's order. */
  readonly considerations: readonly PoruthamRow[];
  /** How many agree; the chapter asks "at least five". */
  readonly agreeing: number;
  /** How many of the chief five agree: Dhinam, Ganam, Yoni, Rasi and Rajju. */
  readonly chiefAgreeing: number;
  readonly exception: PoruthamException;
}

/** A name's first syllable in the śatapada cakra (*Svarodaya* vv. 3–8). */
export interface NameSyllable {
  /** Its place among the cakra's 112 cells, 0 for a, Krittika's first. */
  readonly cell: number;
  /** Its star, or null for Abhijit, which is none of the 27; `rules.name.abhijit` decides the star it is matched as. */
  readonly nakshatra: Nakshatra | null;
  /** Which of the star's four syllables it is, 1 to 4: the pada, for one of the 27. */
  readonly quarter: number;
  /** The letter group the name begins in (VI.35). */
  readonly varga: NameVarga | 'unknown';
}

/** How an initial reaches a pada's cell: its own letter, a paired letter (C294), or the patro's substitute for a nasal no name begins with (C770). */
export type InitialBy = 'CELL' | 'PAIRED' | 'SUBSTITUTE';

/** Whose text a clause of a name in daily use is reported from (C774). */
export type ClauseSource = 'PARASKARA' | 'ASHVALAYANA' | 'APASTAMBA_HIRANYAKESHIN';

/** A letter a name may begin with to take a pada's syllable. */
export interface NameInitial {
  /** The letter, as Devanagari. */
  readonly devanagari: string;
  /** Its IAST. */
  readonly iast: string;
  /** How it reaches the cell. */
  readonly by: InitialBy;
}

/** One cell of the cakra as a name may begin with it (C297, C770, C771). */
export interface PadaSyllable {
  /** Its place among the cakra's 112 cells, 0 for a, Krittika's first. */
  readonly cell: number;
  /** Its star, or null for Abhijit's row. */
  readonly nakshatra: Nakshatra | null;
  /** Its pada, 1 to 4, or null for Abhijit's, which is no pada (C298). */
  readonly pada: number | null;
  /** The sign it stands in; Makara for Abhijit's (C771). */
  readonly rashi: Rashi;
  /** The print's spelling (C299). */
  readonly devanagari: string;
  /** Its IAST. */
  readonly iast: string;
  /** The varga of its letter (C295). */
  readonly varga: NameVarga;
  /** Every letter that reaches the cell: its own first. */
  readonly initials: readonly NameInitial[];
}

/** One clause of a name in daily use, as one sutra gives it (C774). */
export interface NameClauseRow {
  readonly clause: NameClause;
  /** Whether the name meets it. */
  readonly holds: boolean;
  /** Whose text gives it. */
  readonly source: ClauseSource;
}

/** A name's syllables and the clauses of a name in daily use (C774). */
export interface NameClauses {
  /** Its syllables, counted by their vowels. */
  readonly syllables: number;
  /** How the final inherent a was counted. */
  readonly counted: FinalA;
  /** The rows of the rules' sutra. */
  readonly rows: readonly NameClauseRow[];
}

/** One cell a name's spelling reaches. */
export interface SpelledName {
  /** The Devanagari it was read as. */
  readonly spelled: string;
  /** Its first syllable in the cakra. */
  readonly syllable: NameSyllable;
  /** The sign that syllable's pada stands in. */
  readonly rashi: Rashi;
  /** Its syllables and clauses. */
  readonly clauses: NameClauses;
}

/** A name read: one entry per first syllable its spelling reaches. */
export interface NameReading {
  readonly readings: readonly SpelledName[];
  /** Whether there is more than one. */
  readonly ambiguous: boolean;
}

/** Why a name of a batch, or of a book, was refused: the SDK's error as a record. */
export interface NameRefusal {
  readonly status: string;
  readonly detail: string | null;
  readonly message: string;
  /** The field it names: `naming.names[2]`, `book.names[4].devanagari`. */
  readonly field: string | null;
  readonly hint: string | null;
  readonly key: unknown;
}

/** One name of a batch read: its reading, or its refusal. */
export interface NameRead {
  readonly reading: NameReading | null;
  readonly refused: NameRefusal | null;
}

/** A name read against a birth's pada: facts, not a score (C334, C770). */
export interface NameCheck {
  /** The name's first syllable. */
  readonly read: NameSyllable;
  /** The birth pada's syllable, with every initial that reaches it. */
  readonly prescribed: PadaSyllable;
  /** Whether the name begins in the birth star's row. */
  readonly sameStar: boolean;
  /** Whether it begins in the birth pada's own cell. */
  readonly samePada: boolean;
  /** Whether it begins with the pada's substitute (C770). */
  readonly bySubstitute: boolean;
  readonly clauses: NameClauses;
}

/** One name of a batch checked: its check, or its refusal. */
export interface NameChecked {
  readonly check: NameCheck | null;
  readonly refused: NameRefusal | null;
}

/** The names a birth gives, of Dharmasindhu's four the SDK computes (C773). */
export interface Names {
  /** The pada's syllable: the default, the avakahada reading. */
  readonly syllable: PadaSyllable;
  /** The star's deity, whose name the Śāṅkhāyanas give. */
  readonly deity: Deity;
}

/** A name a book holds that the cakra refused. */
export interface RefusedName {
  /** Its place in the book. */
  readonly index: number;
  readonly error: NameRefusal;
}

/** A book loaded into a context. */
export interface LoadedBook {
  /** Its id, the SHA-256 of its canonical JSON, which a suggestion names it by. */
  readonly book: string;
  /** How many names it holds. */
  readonly names: number;
  readonly refused: readonly RefusedName[];
}

/** A name a book suggests for a birth. */
export interface Suggested {
  /** Its place in the book. */
  readonly index: number;
  readonly reading: SpelledName;
  /** How it reaches the birth pada. */
  readonly by: InitialBy;
  /** Whether it begins in the pada's own cell. */
  readonly samePada: boolean;
}

/** The names a book suggests for a birth. */
export interface Suggestions {
  /** The book asked. */
  readonly book: string;
  /** The birth pada's syllable, with its initials. */
  readonly prescribed: PadaSyllable;
  /** In cakra order, then the book's. */
  readonly names: readonly Suggested[];
  /** Every name the book holds that the cakra refused, listed, never dropped. */
  readonly refused: readonly RefusedName[];
}

/** Two names' vargas and how they stand (*Muhurta Chintamani* VI.35, C295). */
export interface VargaKoota {
  readonly bride: NameVarga | 'unknown';
  readonly groom: NameVarga | 'unknown';
  /** One varga, enemies (each the 5th from the other), or neither. */
  readonly relation: VargaRelation | 'unknown';
}

/** Two names matched star to star (naam milan). */
export interface NaamMilan {
  readonly bride: NameSyllable;
  readonly groom: NameSyllable;
  readonly varga: VargaKoota;
  /** The Ashta Koota of the two name stars, as a chart's `matching` reads two Moons. */
  readonly ashta: AshtaKoota;
  /** The ten considerations of the two name stars, as a chart's `porutham` reads two Moons. */
  readonly porutham: Porutham;
}

/** A number reduced: every sum, so "33, so 6" and "38, so 11" read back. */
export interface Reduction {
  /** The sums in order: the number reduced, then each digit sum. */
  readonly steps: readonly number[];
  /** Where it stopped: one digit, or a master the rules keep. */
  readonly number: number;
}

/** One letter of a word and what the system's table makes it worth. */
export interface NumerologyLetter {
  /** The letter, upper case. */
  readonly letter: string;
  readonly value: number;
}

/** One word of a name. */
export interface WordNumber {
  /** The word as written. */
  readonly text: string;
  readonly letters: readonly NumerologyLetter[];
  /** The letters added. */
  readonly total: number;
  readonly reduction: Reduction;
}

/** A name read under one system. */
export interface NameNumber {
  readonly system: 'PYTHAGOREAN' | 'CHALDEAN';
  /** Every word, in order, so either reduction can be read back. */
  readonly words: readonly WordNumber[];
  /** What the final reduction started from. */
  readonly total: number;
  /** Cheiro's compound number, for the Chaldean system only. */
  readonly compound: number | null;
  readonly reduction: Reduction;
}

/** Balliett's birth number. */
export interface BirthNumber {
  readonly month: Reduction;
  readonly day: Reduction;
  readonly year: Reduction;
  /** The parts that are not masters, added and reduced; null when every part is a master. */
  readonly sum: Reduction | null;
  /** The parts that are masters, which stand apart from the sum (Balliett p. 90): month, day, year. */
  readonly apart: readonly number[];
}

/** Cheiro's numbers of a date, which are "not added together" (p. 93). */
export interface ChaldeanDate {
  /** The day of the month, reduced. */
  readonly birth: Reduction;
  /** The year's digits reduced. */
  readonly year: Reduction;
}

/** The numbers only the baseline engine computes, with no public-domain source. */
export interface BaselineNumerology {
  /** The vowels A, E, I, O and U under Balliett's cycle. */
  readonly soul: Reduction;
  /** The other letters. */
  readonly personality: Reduction;
  /** The date's digit sum under Cheiro, which he forbids (p. 93). */
  readonly chaldeanDestiny: Reduction;
}

/** A context to open: a profile and the settings patch over it. */
export interface RecordedOpening {
  readonly profile: string;
  readonly settings: Readonly<Record<string, unknown>>;
}

/** A stored field read and set aside: nothing computes from it. */
export interface RecordedNote {
  readonly field: string;
  readonly value: unknown;
  readonly why: string;
}

/** A stored baseline settings snapshot, translated. */
export interface RecordedSettings {
  /** The chart's context. */
  readonly natal: RecordedOpening;
  /** The KP context, on the natal frame. */
  readonly kp: RecordedOpening;
  /** The context a muhurta is searched on: the natal one from the Earth's centre with the true node (crux C781). */
  readonly muhurta: RecordedOpening;
  /** The ephemeris the natal chain needs at its front, when it needs one. */
  readonly ephemeris: 'SURYA_SIDDHANTA' | null;
  /** The locale the stored language is, when one is stored. */
  readonly locale: string | null;
  /** Every field read and set aside, in table order. */
  readonly notes: readonly RecordedNote[];
}

/** A number or an amount in words and in figures, as `intl.inWords` gives it. */
export interface InWords {
  /** The whole, in words. */
  readonly text: string;
  /** The figures, grouped, in the digits asked for. */
  readonly figure: string;
  /** Where the words came from: `cldr-<release>/<file>/<rule set>`. */
  readonly rules: string;
  /** The locale the words and units are in. */
  readonly locale: string;
  /** An amount's parts; `null` for a number. */
  readonly amount: AmountInWords | null;
}

/** A date or a time as text and parts, as `intl.time` and `intl.date` give it. */
export interface CivilText {
  /** The whole. */
  readonly text: string;
  /** The parts, in the locale's order; joined, they are `text`. */
  readonly parts: readonly CivilPart[];
  /** The locale asked for. */
  readonly locale: string;
  /** The locale whose pattern answered. */
  readonly resolvedFrom: string;
  /** Whether a fallback locale answered. */
  readonly isFallback: boolean;
  /** Whether any of it is text no reviewer has signed off. */
  readonly isDraft: boolean;
  /** Every problem met on the way. */
  readonly warnings: readonly string[];
}

/** One part of a date or a time. */
export interface CivilPart {
  /** What it is. */
  readonly kind: 'YEAR' | 'MONTH' | 'DAY' | 'WEEKDAY' | 'ERA' | 'HOUR' | 'MINUTE' | 'SECOND' | 'DAY_PERIOD' | 'LITERAL';
  /** Its text. */
  readonly text: string;
}

/** An amount's parts, for an application that composes them its own way. */
export interface AmountInWords {
  /** The ISO 4217 code. */
  readonly currency: string;
  /** The major units. */
  readonly major: number;
  /** The minor units after them. */
  readonly minor: number;
  /** The major units in words. */
  readonly majorWords: string;
  /** The major unit's word for that count. */
  readonly majorUnit: string;
  /** The minor units in words; `null` for a currency with no minor unit. */
  readonly minorWords: string | null;
  /** The minor unit's word for that count; `null` with `minorWords`. */
  readonly minorUnit: string | null;
  /** Whether a decimal lost digits to the rounding. */
  readonly rounded: boolean;
}

/** Everything numerology says of a name and a birth date. */
export interface NumerologyProfile {
  readonly pythagoreanName: NameNumber;
  readonly pythagoreanBirth: BirthNumber;
  readonly chaldeanName: NameNumber;
  readonly chaldeanBirth: ChaldeanDate;
  /** The baseline engine's own numbers, present only under every baseline reading. */
  readonly baseline: BaselineNumerology | null;
}

/** Mars's house from one reference. */
export interface KujaReading {
  /** The place the house is counted from. */
  readonly from: 'LAGNA' | 'MOON' | 'VENUS';
  /** Mars's house from it by sign, 1 to 12 (C287). */
  readonly house: number;
  /** Whether the house is one of the rules' houses; it makes the dosha only from a reference the rules count. */
  readonly inHouses: boolean;
}

/** One native's Kuja dosha. */
export interface KujaSide {
  /** Mars's house from the lagna, the Moon and Venus, whatever the rules count. */
  readonly readings: readonly KujaReading[];
  /** Whether Mars stands in one of the rules' houses from a reference the rules count. */
  readonly dosha: boolean;
  /** Every lift that holds, whatever the rules count; `null` where the chart gave no sky to read them in. */
  readonly held: KujaLifts | null;
  /** Whether the side carries the dosha and a lift the rules count holds. */
  readonly lifted: boolean;
}

/** The Kuja dosha of a bride and a groom (*Manasagari*, jāyābhāva v. 4), as clauses, with each side's lifts (C288, C755). */
export interface Kuja {
  readonly bride: KujaSide;
  readonly groom: KujaSide;
  /** Whether both carry it, the fact the popular cancellation reads. */
  readonly both: boolean;
}

/**
 * One marriage dosha a match carries (C289): its reading, its koota or
 * consideration (`null` for the Kuja dosha, which is no koota), the side
 * carrying it for the Kuja dosha (`null` for the rest), and whether an
 * exception the source names lifts it. No severity (C290).
 */
export interface MarriageDosha {
  readonly system: 'ASHTA_KOOTA' | 'PORUTHAM' | 'KUJA' | 'unknown';
  readonly koota: Koota | 'unknown' | null;
  readonly side: 'BRIDE' | 'GROOM' | 'unknown' | null;
  readonly lifted: boolean;
}

/** A chart's Western houses, in its own zodiac. */
export interface WesternHouses {
  /** The division the cusps are of: the one asked, or the one a polar policy fell back to. */
  readonly system: HouseSystem | 'unknown';
  /** The twelve cusps, first to twelfth, degrees. */
  readonly cuspsDeg: readonly number[];
  /** The ascendant, degrees. */
  readonly ascendantDeg: number;
  /** The degree that rose one sidereal hour before the birth, degrees (Leo, p. 90). */
  readonly reachDeg: number;
  /** Each planet's house, in the catalogue's order. */
  readonly planets: readonly WesternHousePlacement[];
}

/** A planet's reflection upon a cusp's very degree (Lilly, p. 165; C251). */
export interface CuspAntiscion {
  readonly graha: Graha | 'unknown';
  /** The house whose cusp its reflection falls on, 1 to 12. */
  readonly house: number;
  /** Whether it is the contrantiscion; false for the antiscion. */
  readonly contrary: boolean;
}

/** A planet's two reflections, tropical degrees. */
export interface Antiscion {
  readonly graha: Graha | 'unknown';
  /** Its reflection about the solstices: 180° less its longitude. */
  readonly antiscionDeg: number;
  /** Its reflection about the equinoxes: 360° less its longitude. */
  readonly contrantiscionDeg: number;
}

/** Two planets in antiscion within the orb. */
export interface AntiscionRow {
  /**
   * The first planet: in a chart's own pair the earlier in catalogue
   * order, across a synastry the chart's.
   */
  readonly first: Graha | 'unknown';
  /** The second: across a synastry, the partner's. */
  readonly second: Graha | 'unknown';
  /** Whether it is the contrantiscion, the reflection about the equinoxes. */
  readonly contrary: boolean;
  /** How far the one's reflection stands from the other, degrees. */
  readonly apartDeg: number;
  /** The orb the request allowed the pair, degrees. */
  readonly orbDeg: number;
}

/** A planet equally distant from two others, within the orb of their axis. */
export interface MidpointRow {
  /** The earlier planet of the pair, in catalogue order. */
  readonly first: Graha | 'unknown';
  readonly second: Graha | 'unknown';
  /** The planet equally distant from the two. */
  readonly middle: Graha | 'unknown';
  /** Whether it stands opposite the midpoint of the pair's shorter arc (C246). */
  readonly far: boolean;
  /** How far it stands from each of the two, the mean of the two arcs, degrees. */
  readonly distanceDeg: number;
  /** How far it stands from the nearer point of the axis, degrees. */
  readonly fromAxisDeg: number;
  /** The orb the request allowed, degrees. */
  readonly orbDeg: number;
}

/** An equal distance across a synastry: a planet of one chart on the axis of two of the other's. */
export interface SynastryMidpointRow extends MidpointRow {
  /** True when the pair is the partner's and `middle` the chart's; false when the pair is the chart's. */
  readonly partnersPair: boolean;
}

/** A chart's antiscia. */
export interface Antiscia {
  /** Each planet's reflections, in the catalogue's order. */
  readonly points: readonly Antiscion[];
  /** The pairs within the orb, closest first. */
  readonly pairs: readonly AntiscionRow[];
  /** The planets the orbs give none, which stand in no pair. */
  readonly unpaired: readonly (Graha | 'unknown')[];
  /** The reflections upon a cusp's very degree; empty unless `cusps` asked. */
  readonly onCusps: readonly CuspAntiscion[];
  /** The division the cusps were read in; `null` unless `cusps` asked. */
  readonly cuspSystem: HouseSystem | 'unknown' | null;
}

/** One pair of planets within an aspect's orb. */
export interface WesternAspectRow {
  /** The earlier planet of the pair, in catalogue order. */
  readonly first: Graha | 'unknown';
  readonly second: Graha | 'unknown';
  readonly aspect: WesternAspect | 'unknown';
  /** The shorter arc between them, degrees 0 to 180. */
  readonly apartDeg: number;
  /** How far that arc is from the aspect's exact angle, degrees. */
  readonly fromExactDeg: number;
  /** The orb the model allowed the pair at this aspect, degrees. */
  readonly orbDeg: number;
  /** Whether the faster planet is closing on the exact angle. */
  readonly applying: boolean;
}

/**
 * A synastry: every chart of the batch read against one partner's birth
 * (`03-design/western-synastry.md`). The aspect table's fields ask the
 * same as `westernAspects`; Leo's nine under his orbs, the lagna read, in
 * the tropical zodiac when every one is absent (C240–C242).
 *
 * @example
 * const chart = ctx.chart.found({
 *   instant, place, utcOffsetSeconds,
 *   synastry: { partner: { instant: 2403113.4993, place: { latitude: 51.5058, longitude: -0.1878, altitude: 0 } } },
 * });
 * for (const row of chart.synastry ?? []) console.log(row.first, row.aspect, row.second);
 */
export interface SynastryRequest extends WesternAspectRequest {
  /** The partner's birth, founded once for the whole batch. */
  readonly partner: {
    readonly instant: number;
    readonly place: GeoPlace;
    /** The partner's clock, seconds east of UTC; 0 when absent. */
    readonly utcOffsetSeconds?: number;
  };
  /**
   * Whether each chart's lagna joins its planets; true by default. It
   * stands as a planet in Leo's orbs (C242); Lilly's moieties give it none,
   * so they need `lagna: false`.
   */
  readonly lagna?: boolean;
  /**
   * `'TROPICAL'` (the default, Leo's frame) or `'CHARTS'`, each chart's
   * own zodiac, which refuses two charts founded in different ones (C241).
   */
  readonly zodiac?: 'TROPICAL' | 'CHARTS';
  /**
   * The parallels across the two charts too (`chart.synastryParallels`):
   * each point of the chart the same distance from the equator as one of
   * the partner's, within the orb (Leo's 1° when `{}`), on either side of
   * it (C243). The lagna joins as `lagna` says.
   */
  readonly parallels?: ParallelRequest;
  /**
   * The antiscia across the two charts too (`chart.synastryAntiscia`):
   * each planet of the chart whose reflection about the solstices or the
   * equinoxes falls on one of the partner's, within the orb read at the
   * conjunction (Lilly's moieties when `{}`, C244).
   */
  readonly antiscia?: AntisciaRequest;
  /**
   * The equal distances across the two charts too
   * (`chart.synastryMidpoints`): each planet of one chart on the axis
   * through two of the other's, within the orb (0.5° when `{}`, C245).
   */
  readonly midpoints?: MidpointRequest;
  /**
   * The composite of the two charts too (`chart.synastryComposite`): each
   * planet and both angles at the near midpoint of the two charts', in
   * this request's zodiac (C247). False when absent.
   */
  readonly composite?: boolean;
  /**
   * Each chart's Davison birth with the partner too
   * (`chart.synastryDavison`): the midpoint of the two births in time and
   * place (C248). False when absent.
   */
  readonly davison?: boolean;
}

/** A planet of a composite chart. */
export interface CompositePlanet {
  readonly graha: Graha | 'unknown';
  /** The near midpoint of its two places, degrees, in the synastry's zodiac. */
  readonly longitudeDeg: number;
  /** The mean of its two speeds, degrees a day; negative when retrograde. */
  readonly speedDegPerDay: number;
}

/**
 * The composite of a chart and a synastry's partner
 * (`03-design/western-composites.md`, C247).
 */
export interface Composite {
  /** Its planets, in the chart's order. */
  readonly planets: readonly CompositePlanet[];
  /** The near midpoint of the two lagnas, turned when `lagnaTurned` says, degrees. */
  readonly lagnaDeg: number;
  /** The near midpoint of the two midheavens, degrees. */
  readonly midheavenDeg: number;
  /** Whether the lagnas' near midpoint stood before the midheaven and was turned by 180°. */
  readonly lagnaTurned: boolean;
  /**
   * Its twelve cusps, first to twelfth: each the near midpoint of the two
   * charts', turned as the lagna is; `null` where the profile refuses the
   * `western` module's division at either birthplace.
   */
  readonly cuspsDeg: readonly number[] | null;
}

/** A chart's distances from the equator, degrees north. */
export interface Declinations {
  /** The true obliquity at the chart's instant, which turned every one. */
  readonly obliquityDeg: number;
  /** The planets, in the catalogue's order. */
  readonly grahas: readonly { readonly graha: Graha | 'unknown'; readonly declinationDeg: number }[];
  /** The lagna's: the Sun's at that degree (Leo, p. 141). */
  readonly lagnaDeg: number;
  /** The midheaven's, read the same way. */
  readonly midheavenDeg: number;
}

/** One pair of planets the same distance from the equator, within the orb. */
export interface ParallelRow {
  /** The earlier planet of the pair, in catalogue order. */
  readonly first: Graha | 'unknown';
  readonly second: Graha | 'unknown';
  /** Whether the two stand on opposite sides of the equator (C243). */
  readonly contrary: boolean;
  /** How far apart their distances from the equator are, degrees. */
  readonly apartDeg: number;
  /** The orb the request allowed, degrees. */
  readonly orbDeg: number;
}

/** One point of a chart and one of the partner's within an aspect's orb. */
export interface SynastryRow {
  /** The chart's point. */
  readonly first: NatalPoint;
  /** The partner's point. */
  readonly second: NatalPoint;
  readonly aspect: WesternAspect | 'unknown';
  /** The shorter arc between them, degrees 0 to 180. */
  readonly apartDeg: number;
  /** How far that arc is from the aspect's exact angle, degrees. */
  readonly fromExactDeg: number;
  /** The orb the model allowed the pair at this aspect, degrees. */
  readonly orbDeg: number;
}

/** One point of a chart and one of the partner's the same distance from the equator, within the orb. */
export interface SynastryParallelRow {
  /** The chart's point. */
  readonly first: NatalPoint;
  /** The partner's point. */
  readonly second: NatalPoint;
  /** Whether the two stand on opposite sides of the equator (C243). */
  readonly contrary: boolean;
  /** How far apart their distances from the equator are, degrees. */
  readonly apartDeg: number;
  /** The orb the request allowed, degrees. */
  readonly orbDeg: number;
}

/** A progressed or directed planet. */
export interface ProgressedPlanet {
  readonly graha: Graha | 'unknown';
  /** Its longitude in the chart's zodiac, degrees. */
  readonly longitudeDeg: number;
}

/** One exact aspect a progressed planet makes to a radical point. */
export interface ProgressedContact {
  /** The instant of life it falls due, a UTC Julian day. */
  readonly life: number;
  /** The instant of sky it is exact at, a UTC Julian day. */
  readonly sky: number;
  /** The progressed planet. */
  readonly graha: Graha | 'unknown';
  /** The radical point, spelled as a hit's `to`. */
  readonly to: NatalPoint;
  /** The aspect's angle, a whole degree 0 to 180. */
  readonly angle: number;
  readonly motion: Motion | 'unknown';
}

/** Two progressed planets exactly in aspect with each other: a line of Leo's "P. to P." list (p. 41). */
export interface MutualContact {
  /** The instant of life it falls due, a UTC Julian day. */
  readonly life: number;
  /** The instant of sky it is exact at, a UTC Julian day. */
  readonly sky: number;
  /** The first planet, in the request's order. */
  readonly first: Graha | 'unknown';
  readonly second: Graha | 'unknown';
  /** The aspect's angle, a whole degree 0 to 180. */
  readonly angle: number;
}

/**
 * A stretch of life a progressed planet stands within the orb of a radical
 * point's distance from the equator, on either side of it (Leo, *The
 * Progressed Horoscope*, p. 34).
 */
export interface ProgressedParallel {
  /** The progressed planet. */
  readonly graha: Graha | 'unknown';
  /** The radical point, spelled as a hit's `to`. */
  readonly to: NatalPoint;
  /** The instant of life it came within the orb, a UTC Julian day; `null` when it already stood within it at the window's start. */
  readonly enters: number | null;
  /** The instant of life it left the orb; `null` when it still stood within it at the window's end. */
  readonly leaves: number | null;
  /** The first instant of life the two distances stood closest: the exact parallel, when the span reaches it. */
  readonly closest: number;
  /** How far apart the two distances stood then, degrees. */
  readonly closestDeg: number;
  /** Whether the two stood on opposite sides of the equator then. */
  readonly contrary: boolean;
}

/**
 * A birth read through its progressions (`03-design/western-progressions.md`).
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, progressions: { at: 2460676.5 } });
 * const moon = chart.progressions?.progressed?.grahas.find((g) => g.graha === 'graha.MOON');
 */
export interface Progressions {
  /** The chart at the instant of sky that measures `at`; `null` without `at`. */
  readonly progressed: {
    readonly life: number;
    readonly sky: number;
    /** The progressed meridian's right ascension, degrees. */
    readonly armcDeg: number;
    /** The progressed angles, by the request's `angles`. */
    readonly angles: { readonly ascendantDeg: number; readonly midheavenDeg: number };
    readonly grahas: readonly (ProgressedPlanet & {
      readonly tropicalDeg: number;
      /** Degrees a day at the instant of sky; below zero when retrograde. */
      readonly speedDegPerDay: number;
    })[];
  } | null;
  /** The birth's points moved by one arc; `null` without `at`. */
  readonly directed: {
    readonly life: number;
    /** The arc, degrees; a solar arc is signed. */
    readonly arcDeg: number;
    readonly ascendantDeg: number;
    readonly midheavenDeg: number;
    readonly planets: readonly ProgressedPlanet[];
  } | null;
  /** The contacts in the window, as they fall due; `null` without `contacts`. */
  readonly contacts: readonly ProgressedContact[] | null;
  /** The parallels of declination in the window, by their closest instant; `null` unless `contacts` asked `parallels`. */
  readonly parallels: readonly ProgressedParallel[] | null;
  /** The progressed planets' contacts among themselves, as they fall due; `null` unless `contacts` asked `mutual`. */
  readonly mutual: readonly MutualContact[] | null;
}

/** The significators coming to an aspect (p. 107). */
export interface Application {
  readonly aspect: PtolemaicAspect;
  /** Days until it is exact. */
  readonly days: number;
  /** The significator whose motion closes it. */
  readonly applying: Graha;
  readonly kind: ApplicationKind;
  /** How far it is from exact now, degrees. */
  readonly gapDeg: number;
  /** Whether the gap is already within the two planets' moieties of orb. */
  readonly withinMoieties: boolean;
}

/** Two planets past an aspect and still within their moieties (p. 110). */
export interface Separation {
  readonly aspect: PtolemaicAspect;
  /** How far past exact, degrees. */
  readonly pastDeg: number;
}

/** What stops or hinders the application (pp. 110–113). */
export interface Impediment {
  readonly kind: ImpedimentKind;
  /** The significator it falls on. */
  readonly significator: Graha;
  /** The third planet; `null` for a refranation. */
  readonly third: Graha | null;
  /** The aspect the third perfects, or the one refrained from. */
  readonly aspect: PtolemaicAspect;
  /** Days until the contact, or the station. */
  readonly days: number;
}

/** A lighter planet carrying one significator's light to the other (p. 111). */
export interface Translation {
  readonly translator: Graha;
  readonly from: Graha;
  readonly to: Graha;
  /** Its separation from `from`. */
  readonly separating: Separation;
  /** The aspect it applies to `to` by. */
  readonly aspect: PtolemaicAspect;
  /** Days until that is exact. */
  readonly days: number;
  /** The dignities of `from` the translator stands in: how it is received (p. 126). */
  readonly received: EssentialDignity;
}

/** A significator's application to a collector. */
export interface ContactAhead {
  readonly aspect: PtolemaicAspect;
  /** Days until it is exact. */
  readonly days: number;
}

/** A heavier planet both significators apply to (p. 112); who must receive whom is C233. */
export interface Collection {
  readonly collector: Graha;
  readonly fromQuerent: ContactAhead;
  readonly fromQuesited: ContactAhead;
  readonly collectorInQuerent: EssentialDignity;
  readonly collectorInQuesited: EssentialDignity;
  readonly querentInCollector: EssentialDignity;
  readonly quesitedInCollector: EssentialDignity;
}

/** Where a significator stands. */
export interface SignificatorPlace {
  readonly planet: Graha;
  /** 1 to 12. */
  readonly house: number;
  /** Its own dignities at its degree. */
  readonly dignity: EssentialDignity;
}

/** The ways of perfection (pp. 125–127): what they weigh, and which hold. */
export interface Ways {
  readonly querent: SignificatorPlace;
  readonly quesited: SignificatorPlace;
  /** Each stands in the other's house. */
  readonly mutualByHouse: boolean;
  /** Saturn and Mars among the thirds that come between the significators before they perfect. */
  readonly infortunesBetween: readonly Graha[];
  /** The Moon, neither significator, separating from the quesited's and coming next to the querent's. */
  readonly moonRelays: boolean;
  /** The quesited's significator in the first house. */
  readonly quesitedInAscendant: boolean;
  /** The ways the figure holds, in `Way`'s order. */
  readonly held: readonly Way[];
}

/**
 * Whether a horary matter is brought to pass: the relations between two
 * significators with the facts each rests on, never a verdict.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, perfection: { house: 7 } });
 * const perfects = (chart.perfection?.ways.held.length ?? 0) > 0;
 */
export interface Matter {
  readonly querent: Graha;
  readonly quesited: Graha;
  /** Their application within the horizon; `null` when none. */
  readonly application: Application | null;
  /** Their separation at the figure; `null` when none. */
  readonly separation: Separation | null;
  readonly impediments: readonly Impediment[];
  readonly translations: readonly Translation[];
  readonly collections: readonly Collection[];
  readonly ways: Ways;
  /** How many days ahead it was read. */
  readonly horizonDays: number;
  /** The rules as they stood: `horizonDays` is `null` when unset. */
  readonly rules: {
    readonly orbsDeg: readonly number[];
    readonly horizonDays: number | null;
    readonly withinSign: boolean;
  };
}

/** A chart read as a prashna (`03-design/prashna.md`). */
export interface Prashna {
  /** The readings it was given under, every member filled. */
  readonly rules: Required<PrashnaRules>;
  /** Whether the matter succeeds: the clauses and *Shatpanchashika* I.4's outcome over them. */
  readonly verdict: {
    readonly clauses: readonly PrashnaClause[];
    readonly outcome: 'SUCCEEDS' | 'WITH_DIFFICULTY' | 'FAILS';
  };
  /** Whether the matter stays as it is (II.1–2). */
  readonly change: 'STAYS' | 'CHANGES';
  /** When the matter comes to pass. */
  readonly timing: PrashnaTiming;
  /** What an unspoken question is about. */
  readonly mook: PrashnaMook;
  /** The Tajika yogas between the lagna lord and the asked house's lord; `null` when no house was asked. */
  readonly links: PrashnaLinks | null;
  /** What those yogas say of the matter (Prashna Tantra vv. 14–21); `null` when no house was asked. */
  readonly link: PrashnaLink | null;
  /** The lagnesha's own yogas read as past, present and future (n179, C375). */
  readonly triKala: TriKala;
  /** Each of the seven's weaknesses of the Samjna Tantra v. 72, the Sun first. */
  readonly weak: readonly {
    readonly graha: Graha;
    readonly clauses: readonly ('UNSEEN_LAGNA' | 'SETTING_PART' | 'OFFICELESS' | 'MALEFIC_ISHRAFA')[];
  }[];
  /** The Bhuvanapradipa's graded yogas (n178, C373); `null` unless `rules.graded` asks. */
  readonly graded: PrashnaGraded | null;
  /** The Moon's weaknesses that hold, in the verses' order (Samjna Tantra vv. 73–74, C352). */
  readonly moon: {
    readonly rules: { readonly kshina: 'DARK_EIGHTH_TO_BRIGHT_EIGHTH' | 'DARK_ELEVENTH_TO_NEW_MOON' };
    readonly clauses: readonly MoonWeaknessClause[];
  };
  /** The baseline engine's points; `null` unless `rules.score` is `BASELINE`. Unsourced. */
  readonly score: PrashnaScore | null;
  /** The sign of the querent's number by the baseline's rule; `null` unless a number was given. */
  readonly numberSign: Rashi | null;
}

/** One clause of the verdict, naming its verse by its kind. */
export interface PrashnaClause {
  readonly kind: 'LAGNA_RISING' | 'IN_LAGNA' | 'RISING_NAVAMSHA' | 'RISING_SHADVARGA' | 'ASPECTS_LAGNA' | 'ASPECTS_MOON' | 'KARYA_HOUSE';
  /** The graha it is about; `null` for how the lagna rises. */
  readonly graha: Graha | null;
  readonly favour: 'FOR' | 'AGAINST' | 'BOTH';
}

/** When the matter comes to pass. */
export interface PrashnaTiming {
  readonly rule: 'STRONGEST_GRAHA' | 'FIRST_OCCUPIED' | 'MOON_DAYS' | 'BASELINE';
  /** The graha the count was taken from. */
  readonly graha: Graha;
  /** Whether that graha won a tie in strength (C338). */
  readonly tie: boolean;
  /** The count, 1 to 12. */
  readonly count: number;
  /** What the count is multiplied by. */
  readonly multiplier: number;
  /** The time; `null` where the rule gives none. */
  readonly amount: number | null;
  readonly unit: 'DAYS' | 'MONTHS' | 'YEARS';
  /** The grahas standing between the lagna and the Moon, under `MOON_DAYS`. */
  readonly between: readonly Graha[];
}

/** What an unspoken question is about. */
export interface PrashnaMook {
  readonly rule: 'SHATPANCHASHIKA' | 'MOON_HOUSE' | 'BASELINE';
  /** The graha whose house was read. */
  readonly graha: Graha;
  /** Whether that graha won a tie in strength. */
  readonly tie: boolean;
  /** The house read, 1 to 12. */
  readonly house: number;
  /** The person, under `SHATPANCHASHIKA` only. */
  readonly person:
    | 'QUERENT'
    | 'FRIEND'
    | 'ENEMY'
    | 'BROTHER'
    | 'MOTHER_OR_SISTER'
    | 'SON'
    | 'WIFE'
    | 'RELIGIOUS'
    | 'GURU'
    | 'UNSPECIFIED'
    | null;
  /** The class of the thing thought of (I.7). */
  readonly thought: 'MINERAL' | 'ROOT' | 'LIVING';
}

/** A prashna's Tajika links: a year's matter, with the states it was judged on. */
export interface PrashnaLinks extends TajikaMatter {
  /** Which of the seven are retrograde and which combust; `null` when not given. */
  readonly states: { readonly retrograde: readonly Graha[]; readonly combust: readonly Graha[] } | null;
}

/** One of the four offices, best first. */
export type Adhikara = 'UTTAMA' | 'MADHYAMA' | 'SAMA' | 'ADHAMA';

/** The offices one graha holds. */
export interface TajikaOffices {
  readonly graha: Graha;
  /** Every clause of the gloss's definition it meets, in the definition's order. */
  readonly held: readonly ('OWN_SIGN' | 'EXALTED' | 'OWN_HUDDA' | 'OWN_DREKKANA' | 'OWN_NAVAMSHA' | 'DEBILITATED' | 'ENEMY_SIGN')[];
  /** The office that counts, by the rules' `order`. */
  readonly adhikara: Adhikara;
}

/** A Kamboola's grade: the Moon's offices and the pair's (Samjna Tantra vv. 36–46). */
export interface KamboolaGrade {
  readonly rules: Required<OfficeRules>;
  readonly moon: TajikaOffices;
  readonly lagnesha: TajikaOffices;
  readonly karyesha: TajikaOffices;
  /** The lords' office when they hold the same one; `null` when they differ (C372). */
  readonly pair: Adhikara | null;
}

/** What the Tajika links say of the matter. */
export interface PrashnaLink {
  readonly kind: 'DIRECT' | 'THROUGH' | 'DESTROYED' | 'SEPARATED' | 'SAME_LORD' | 'UNLINKED';
  /** What it says under the question's tense. */
  readonly meaning: 'PROMISED' | 'FAILS' | 'PAST' | 'FAILS_OR_PAST' | 'UNREAD';
  /** The yoga that made it: Ithasala, Nakta, Yamaya, Manau or Ishrafa. */
  readonly yoga: YearYoga | null;
  /** The third graha carrying or gathering the light, or the malefic destroying it. */
  readonly through: Graha | null;
  /** The Kamboola's grade, when the Moon joins the Ithasala. */
  readonly kamboola: KamboolaGrade | null;
  /** v. 96: whether the Moon looks at neither lord; `null` where `moonScope` does not reach the question. */
  readonly moonless: boolean | null;
  /** Samarasimha's conditions that hold, in his order. */
  readonly samarasimha: readonly (
    | 'LORDS_SEE_THEIR_HOUSES'
    | 'KARYESHA_IN_LAGNA_SEES_LAGNESHA'
    | 'KARYESHA_ELSEWHERE_SEES_LAGNESHA'
    | 'MOON_SEES_LAGNESHA'
  )[];
}

/**
 * The Bhuvanapradipa's graded yogas: every clause the grades are built from, and the grades whose
 * clauses hold, by name, never ranked (C373).
 */
export interface PrashnaGraded {
  /** The aspect the clauses were read by. */
  readonly rule: 'PARASHARI' | 'TAJIKA';
  /** The lords' configurations that hold; empty when no house was asked. */
  readonly lords: readonly ('OWN_HOUSES' | 'EACH_OTHERS_HOUSES' | 'EACH_OTHER')[];
  /** Whether the Moon looks at the lagna. */
  readonly moonSeesLagna: boolean;
  /** Whether the lagnesha looks at the lagna. */
  readonly lagneshaSeesLagna: boolean;
  /** The benefics, the lagnesha left out, looking at the lagna. */
  readonly onLagna: readonly Graha[];
  /** The benefics looking at the lagnesha. */
  readonly onLagnesha: readonly Graha[];
  /** Whether no malefic looks at the Moon. */
  readonly moonFree: boolean;
  /** The grades whose clauses hold, in the passage's order. */
  readonly grades: readonly ('PARIPURNA' | 'PADA' | 'ARDHA' | 'PADONA' | 'TRIBHAGONA' | 'PURNA')[];
  /** The grahas that give ill: sharing a sign with a malefic, looked at by one, or burnt by the Sun. */
  readonly ill: readonly Graha[];
}

/** The lagnesha's own yogas read as time: those it separates from, is joined with, and will join. */
export interface TriKala {
  readonly lagnesha: Graha;
  readonly past: readonly Graha[];
  readonly present: readonly Graha[];
  readonly future: readonly Graha[];
}

/** A clause of the Samjna Tantra vv. 73–74 that weakens the Moon. */
export type MoonWeaknessClause =
  | 'TWELFTH_FROM_SUN'
  | 'SCORPIO_FIRST_HALF'
  | 'LIBRA_LAST_HALF'
  | 'UNSEEN_BY_LORD'
  | 'UNSEEN_BY_ALL'
  | 'SHUNYA_MARGA'
  | 'KSHINA'
  | 'BHANTE'
  | 'HUNGRY_ASPECT';

/** The baseline engine's points for a query chart. Unsourced (C337). */
export interface PrashnaScore {
  readonly points: number;
  readonly answer: 'YES' | 'NO' | 'UNCERTAIN';
  /** Each factor that gave points, in the baseline's order. */
  readonly factors: readonly {
    readonly kind:
      | 'LAGNA_LORD_KENDRA_OR_TRIKONA'
      | 'LAGNA_LORD_DUSTHANA'
      | 'MOON_WAXING'
      | 'MOON_VOID'
      | 'BENEFICS_IN_KENDRAS'
      | 'MALEFICS_IN_KENDRAS'
      | 'MOON_APPLYING_TO_BENEFIC'
      | 'MOON_APPLYING_TO_MALEFIC';
    readonly points: number;
  }[];
  /** The baseline's void Moon, over grahas held still. */
  readonly void: boolean;
  /** The graha nearest ahead of the Moon by conjunction distance; `null` when none is within 180°. */
  readonly applyingTo: Graha | null;
}

/** One of a graha's three lines of sight. */
export type VedhaLine = 'LEFT' | 'FRONT' | 'RIGHT';

/** A native's point a vedha can strike. */
export type VedhaPoint = 'NAKSHATRA' | 'LETTER' | 'VOWEL' | 'TITHI' | 'RASHI' | 'VARA';

/** A star of the chakra's 28: one of the 27, or Abhijit. */
export type ChakraStar = { readonly star: 'NAKSHATRA'; readonly nakshatra: Nakshatra } | { readonly star: 'ABHIJIT' };

/** One cell of the chakra. */
export type ChakraCell =
  | { readonly cell: 'STAR'; readonly star: ChakraStar }
  | { readonly cell: 'VOWEL'; readonly vowel: ChakraVowel }
  | { readonly cell: 'LETTER'; readonly letter: string }
  | { readonly cell: 'RASHI'; readonly rashi: Rashi }
  | { readonly cell: 'TITHI'; readonly class: TithiClass };

/** How a graha reaches what it strikes: along a line, from a corner's juncture, or by standing in a letter's star (v. 24). */
export type VedhaReach =
  | { readonly by: 'LINE'; readonly line: VedhaLine; readonly cell: ChakraCell }
  | { readonly by: 'CORNER'; readonly cell: ChakraCell }
  | { readonly by: 'STANDS' };

/** What one transiting graha strikes. */
export interface GrahaVedha {
  readonly graha: Graha;
  /** Where it stands: the chakra's star, and the nakshatra and pada of the 27. */
  readonly place: { readonly star: ChakraStar; readonly nakshatra: Nakshatra; readonly pada: number };
  readonly motion: VedhaMotion;
  /** The lines it looks along. */
  readonly lines: readonly VedhaLine[];
  readonly nature: 'BENEFIC' | 'MALEFIC';
  /** For the Moon, whether it wanes, which makes it malefic (v. 12). */
  readonly waning: boolean;
  /** For a benefic, the malefics in its sign, which make it malefic. */
  readonly joinedBy: readonly Graha[];
  /** Every cell it strikes, and how. */
  readonly cells: readonly VedhaReach[];
  /** The native's points among them, with every way it strikes each. */
  readonly points: readonly { readonly point: VedhaPoint; readonly by: readonly VedhaReach[] }[];
}

/**
 * The Sarvatobhadra chakra at one instant against the native
 * (`03-design/gochar.md` §8): clauses, scored by nothing.
 */
export interface Sarvatobhadra {
  readonly rules: Required<SarvatobhadraRules>;
  /** The native's points: the natal Moon's star and sign, the birth's tithi and weekday, and the name when given. */
  readonly native: {
    readonly nakshatra: Nakshatra;
    readonly rashi: Rashi;
    readonly tithi: Tithi;
    readonly vara: Vara;
    readonly name: NameSounds | null;
  };
  /** Each graha's, the Sun to Ketu. */
  readonly grahas: readonly GrahaVedha[];
  /** Each of the native's points with the malefics and benefics that strike it. */
  readonly points: readonly { readonly point: VedhaPoint; readonly malefics: readonly Graha[]; readonly benefics: readonly Graha[] }[];
  /** How many of v. 28's five (star, letter, vowel, tithi, sign) a malefic strikes. */
  readonly maleficPoints: number;
}

/**
 * A section of a chart's dossier, in the dossier's own order
 * (`03-design/dossier.md`).
 */
export type DossierSection =
  | 'CHART'
  | 'NOW'
  | 'GRAHAS'
  | 'HOUSES'
  | 'VARGAS'
  | 'DASHAS'
  | 'PANCHANGA'
  | 'ASHTAKAVARGA'
  | 'SHADBALA'
  | 'BHAVA_BALA'
  | 'VIMSHOPAKA'
  | 'JAIMINI'
  | 'POINTS'
  | 'ASPECTS'
  | 'AVAKAHADA'
  /** What held under the request's `rules` record, a row a rule. */
  | 'RULES'
  /** Each graha's transit at each of the request's `gochar` instants. */
  | 'GOCHAR';

/** One year's tithi pravesha: the instant, the solar return it was chosen around and which rule chose it. */
export interface TithiPravesha {
  /** The year, `1` the first return, a year after birth, as `Pravesha.year` counts it. */
  readonly year: number;
  /** The instant the elongation returns, a Julian day (UTC): found the year's chart here. */
  readonly at: number;
  /** The year's sidereal solar return, a Julian day (UTC): what "nearest" is measured from. */
  readonly solarReturn: number;
  /** The birth's elongation returned to, the Moon less the Sun, degrees in `[0, 360)`. */
  readonly elongationDeg: number;
  /** The rule asked for. */
  readonly rule: TithiRule;
  /** The rule that chose this instant: `NEAREST_RETURN` where `NATAL_SIGN` found none in the sign (C446). */
  readonly picked: TithiRule;
  /** How many phase returns fell while the Sun was in its natal sign: 0, 1 or 2. */
  readonly inSign: number;
  /** Whether the year fell back: `rule` and `picked` differ. */
  readonly fellBack: boolean;
}

/**
 * One year's month charts, or one month's day charts, to find for every
 * chart (`03-design/annual-chart.md`, "The month's and the day's chart").
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, masaPravesha: { year: 30, month: 4 } });
 * for (const day of chart.masaPravesha ?? []) {
 *   // The day's chart, founded where the native is.
 *   const daily = ctx.chart.found({ instant: day.at, place: residence, utcOffsetSeconds });
 * }
 */
export interface MasaPraveshaRequest {
  /** The year divided, 1 to 200, counted as `Pravesha.year` counts it. */
  readonly year: number;
  /** The month whose thirty days are wanted, 1 to 12; the year's twelve months when left out. */
  readonly month?: number;
  /** The zodiac the year divides in; `SIDEREAL` by default. The mean year places no Sun. */
  readonly reading?: Exclude<ReturnLongitude, 'MEAN'>;
  /**
   * Where each month's chart is founded, as `varsha.place` is written; given,
   * each month carries its `masesha`, its `results` and its `dasha`, and
   * beside a `month` each day carries its `dinesha` and its `dayResults`.
   */
  readonly place?: 'birth' | AnnualPlace;
  /** The readings the month's lord, and the day's, part on; the verse's own by default. */
  readonly masesha?: MaseshaRules;
  /** The floors the month's and the day's results judge strong and weak by; the year yogas' own by default. */
  readonly results?: MasaPhalaRules;
  /** The readings each month's dasha is computed under; the verse's own by default. */
  readonly dasha?: MasaDashaRules;
}

/** One Hudda lord's run of a month. */
export interface MasaDashaPeriod {
  readonly lord: Graha;
  /** The Hudda's width, degrees: the solar days it runs. */
  readonly degrees: number;
  /** When it begins, a Julian day (UTC). */
  readonly starts: number;
  /** When it ends, a Julian day (UTC): where the next begins. */
  readonly ends: number;
}

/**
 * A month's dasha (*Tajika Nilakanthi*, Varsha Tantra, v. 14): the five
 * Huddas of the stronger lord's sign, each running its degrees in days.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, masaPravesha: { year: 30, place: 'birth' } });
 * for (const month of chart.masaPravesha ?? []) {
 *   const first = month.dasha?.periods[0].lord;
 * }
 */
export interface MasaDasha {
  /** Whose lord was the stronger: the Muntha's or the month lagna's. */
  readonly office: 'MUNTHA' | 'MASA_LAGNA';
  readonly graha: Graha;
  /** The sign whose Huddas run the month. */
  readonly sign: Rashi;
  readonly rules: Required<MasaDashaRules>;
  /** The sign's five Huddas, in order from the month's opening. */
  readonly periods: readonly MasaDashaPeriod[];
}

/** The six office-bearers of one month. */
export interface MonthBearers {
  readonly masaLagna: Graha;
  /** The Muntha's sign's lord, the Muntha moved on 2°30′ a month. */
  readonly muntha: Graha;
  readonly janmaLagna: Graha;
  /** The month lagna's triplicity lord, by day or by night. */
  readonly triRashi: Graha;
  /** The lord of the Sun's sign by day, of the Moon's by night. */
  readonly dinaRatri: Graha;
  readonly varshaLagna: Graha;
  /** Whether the month opened by day, which chose two of them. */
  readonly byDay: boolean;
}

/** The lord of one month, and the chain it was read beside. */
export interface Masesha {
  /** The lord under `reading`. */
  readonly graha: Graha;
  readonly reading: 'CHAIN' | 'LAGNA_LORD';
  /** The six it was chosen among. */
  readonly bearers: MonthBearers;
  /** The chain over the six, carried under either reading so a reader sees where the two part. */
  readonly chain: YearLord;
}

/** One month's or one day's chart instant: the Sun a sign further each month, a degree further each day. */
export interface MasaPravesha {
  /** The year, as `Pravesha.year` counts it. */
  readonly year: number;
  /** The month, 1 (the annual chart itself) to 12. */
  readonly month: number;
  /** The day, 1 (the month's chart itself) to 30; 1 on every row of a year's months. */
  readonly day: number;
  /** The instant, a Julian day (UTC): found the chart here. */
  readonly at: number;
  /** Where the Sun stands then, degrees in the year's zodiac. */
  readonly sunDeg: number;
  /** The month's lord, where the request named a `place`; absent otherwise. */
  readonly masesha?: Masesha;
  /** The month's results, where the request named a `place`; absent otherwise. */
  readonly results?: MasaPhala;
  /** The month's dasha, where the request named a `place`; absent otherwise. */
  readonly dasha?: MasaDasha;
  /** The day's lord, where the request named a `month` and a `place`; absent otherwise. */
  readonly dinesha?: Dinesha;
  /** The day's results, where the request named a `month` and a `place`; absent otherwise. */
  readonly dayResults?: DinaPhala;
}

/** The seven office-bearers of one day (*Tajika Nilakanthi*, Varsha Tantra, v. 17). */
export interface DayBearers {
  /** The Muntha's sign's lord, the Muntha moved on 5′ a day. */
  readonly muntha: Graha;
  readonly janmaLagna: Graha;
  /** The day lagna's triplicity lord, by day or by night. */
  readonly triRashi: Graha;
  /** The lord of the Sun's sign by day, of the Moon's by night. */
  readonly dinaRatri: Graha;
  readonly dinaLagna: Graha;
  readonly masaLagna: Graha;
  readonly varshaLagna: Graha;
  /** Whether the day opened by day, which chose two of them. */
  readonly byDay: boolean;
}

/**
 * The lord of one day, and the chain it was read beside
 * (`03-design/varshesha.md`, "The day's lord").
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, masaPravesha: { year: 30, month: 4, place: 'birth' } });
 * const lords = (chart.masaPravesha ?? []).map((day) => day.dinesha?.graha);
 */
export interface Dinesha {
  /** The lord under `reading`. */
  readonly graha: Graha;
  readonly reading: 'CHAIN' | 'LAGNA_LORD';
  /** The seven it was chosen among. */
  readonly bearers: DayBearers;
  /** The chain over the seven, carried under either reading. */
  readonly chain: YearLord;
}

/**
 * A clause of vv. 7–11 about one house (*Tajika Nilakanthi*, Varsha Tantra,
 * the month's results): the house's navamsha lord and the lord's navamsha
 * lord in friendly or inimical aspect, the Moon's aspect on both, their
 * debility and combustion, one planet in both roles, and the first's
 * strength.
 */
export type MonthHouseClause =
  | 'FRIENDLY'
  | 'MOON_FRIENDLY'
  | 'INIMICAL'
  | 'MOON_INIMICAL'
  | 'LOW_AND_SET'
  | 'BOTH_LOW'
  | 'BOTH_SET'
  | 'ONE_PLANET'
  | 'STRONG'
  | 'WEAK';

/** The month's lords the results read: the year lagna's, the Masesha, the Varshesha, the month lagna's, the Muntha's. */
export type MonthLord = 'VARSHA_LAGNA' | 'MASESHA' | 'VARSHESHA' | 'MASA_LAGNA' | 'MUNTHA';

/** A clause of vv. 12–13 about one of the month's lords. */
export type MonthLordClause = 'TRIKA' | 'WITH_MALEFIC' | 'MALEFIC_INIMICAL' | 'STRONG' | 'WELL_PLACED';

/** One house of the month, read through its two planets. */
export interface MonthHouse {
  /** The house, 1 to 12 by whole signs from the month lagna. */
  readonly house: number;
  /** The lord of the navamsha the house's point falls in. */
  readonly amshaLord: Graha;
  /** The lord of the navamsha the house's sign lord stands in. */
  readonly lordAmshaLord: Graha;
  /** The clauses that hold, in the verses' order. */
  readonly clauses: readonly MonthHouseClause[];
  /** The lords of v. 10 the house's navamsha lord sees: a friendly aspect or a shared sign. */
  readonly seenByAmshaLord: readonly MonthLord[];
  /** The lords of v. 10 the Moon sees. */
  readonly seenByMoon: readonly MonthLord[];
}

/** Where one of the month's lords stands. */
export interface MonthLordPlace {
  readonly office: MonthLord;
  readonly graha: Graha;
  /** The house it stands in, 1 to 12 from the month lagna. */
  readonly house: number;
  /** The clauses that hold, in the verses' order. */
  readonly clauses: readonly MonthLordClause[];
}

/**
 * The month's results (`03-design/varshesha.md`, "The month's results"):
 * the clauses that hold, no verdict.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, masaPravesha: { year: 30, place: 'birth' } });
 * for (const month of chart.masaPravesha ?? []) {
 *   const body = month.results?.houses[0].clauses.includes('FRIENDLY');
 * }
 */
export interface MasaPhala {
  /** The twelve houses, first to twelfth. */
  readonly houses: readonly MonthHouse[];
  /** The five lords, in `MonthLord`'s order. */
  readonly lords: readonly MonthLordPlace[];
}

/** The Moon's twelve states within a sign, 2°30′ each, in the gloss's order (vv. 30–33). */
export type MoonState =
  | 'PRAVASA'
  | 'NASHTA'
  | 'MRITA'
  | 'JAYA'
  | 'HASYA'
  | 'RATI'
  | 'KRIDITA'
  | 'PRASUPTA'
  | 'BHUKTA'
  | 'JVARA'
  | 'KAMPITA'
  | 'SUSTHITA';

/** A clause of vv. 20–21 about one house's navamsha lord. */
export type DayHouseClause = 'WITH_BENEFIC' | 'BENEFIC_FRIENDLY' | 'MOON_FRIENDLY' | 'WITH_MALEFIC' | 'MALEFIC_INIMICAL';

/** The day's lords v. 19 judges: the Dinesha, the Varshesha, the Masesha, the Muntha's. */
export type DayLord = 'DINESHA' | 'VARSHESHA' | 'MASESHA' | 'MUNTHA';

/** A clause of vv. 18 and 26–29 about the day's chart as a whole. */
export type DayClause =
  | 'BENEFICS_WELL_PLACED_FROM_LAGNA'
  | 'BENEFICS_WELL_PLACED_FROM_MOON'
  | 'MALEFICS_WELL_PLACED_FROM_LAGNA'
  | 'MALEFICS_WELL_PLACED_FROM_MOON'
  | 'MALEFICS_FLANK'
  | 'BENEFIC_TWELFTH'
  | 'PAPA_KARTARI'
  | 'SHUBHA_KARTARI'
  | 'WANING_MOON_AFFLICTED'
  | 'MOON_MARS_DUSTHANA'
  | 'MALEFIC_FOURTH'
  | 'BENEFIC_SEVENTH'
  | 'BENEFIC_SECOND'
  | 'BENEFIC_NINTH';

/** One house of the day, read through its navamsha lord. */
export interface DayHouse {
  /** The house, 1 to 12 by whole signs from the day lagna. */
  readonly house: number;
  /** The lord of the navamsha the house's point falls in. */
  readonly amshaLord: Graha;
  /** The clauses that hold, in the verses' order. */
  readonly clauses: readonly DayHouseClause[];
}

/** Where one of the day's lords stands. */
export interface DayLordPlace {
  readonly office: DayLord;
  readonly graha: Graha;
  /** The house it stands in, 1 to 12 from the day lagna. */
  readonly house: number;
  /** The clauses that hold, the month's lords' own. */
  readonly clauses: readonly MonthLordClause[];
}

/**
 * The day's results (`03-design/varshesha.md`, "The day's results"): the
 * clauses that hold, no verdict.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, masaPravesha: { year: 30, month: 4, place: 'birth' } });
 * const states = (chart.masaPravesha ?? []).map((day) => day.dayResults?.moonState);
 */
export interface DinaPhala {
  /** The Moon's state within her sign. */
  readonly moonState: MoonState;
  /** The twelve houses, first to twelfth. */
  readonly houses: readonly DayHouse[];
  /** The four lords, in `DayLord`'s order. */
  readonly lords: readonly DayLordPlace[];
  /** The clauses about the whole chart that hold, in the verses' order. */
  readonly clauses: readonly DayClause[];
}

/** How strongly one house looks at another. */
export type LalKitabStrength = 'QUARTER' | 'HALF' | 'FULL';

/** What a teva says (`03-design/lalkitab.md` §3). */
export interface LalKitabReading {
  readonly planets: readonly {
    readonly graha: Graha;
    /** Its house, 1 to 12, the whole-sign house from the lagna. */
    readonly house: number;
    readonly dignities: readonly ('PAKKA' | 'EXALTED' | 'DEBILITATED' | 'OWN')[];
    /** The house's owners and how this planet regards each (1952 p. 31). */
    readonly owners: readonly { readonly owner: Graha; readonly regard: 'FRIEND' | 'EQUAL' | 'ENEMY' }[];
    readonly awake: boolean;
    /** In a dignity, alone in its house and looked at from no occupied house. */
    readonly kayam: boolean;
    /** The aspects it casts, forward only. */
    readonly casts: readonly { readonly to: number; readonly strength: LalKitabStrength; readonly onto: readonly Graha[] }[];
  }[];
  readonly houses: readonly {
    readonly house: number;
    readonly occupants: readonly Graha[];
    /** The occupied houses that look at it. */
    readonly lookedAtBy: readonly { readonly from: number; readonly strength: LalKitabStrength }[];
    /** Occupied, or looked at from an occupied house. */
    readonly awake: boolean;
    /** The planet whose presence wakes it (1952 p. 98). */
    readonly waker: Graha;
  }[];
  /** The artificial planets pairs in one house make (1952 p. 27). */
  readonly masnui: readonly {
    readonly pair: readonly Graha[];
    readonly house: number;
    readonly countsAs:
      | 'JUPITER'
      | 'SUN'
      | 'MOON'
      | 'VENUS'
      | 'MARS_BENEFIC'
      | 'MARS_MALEFIC'
      | 'MERCURY'
      | 'SATURN_LIKE_KETU'
      | 'SATURN_LIKE_RAHU'
      | 'RAHU_EXALTED'
      | 'RAHU_DEBILITATED'
      | 'KETU_EXALTED'
      | 'KETU_DEBILITATED';
  }[];
  /** The debts the teva carries (1952 p. 125). */
  readonly rinas: readonly {
    readonly rin: 'PITRI' | 'SWA' | 'MATRI' | 'STRI' | 'RISHTEDARI' | 'BHAGINI' | 'ZALIMANA' | 'AJANMA' | 'DAIVI';
    readonly of: Graha;
    readonly seated: readonly { readonly enemy: Graha; readonly house: number }[];
  }[];
  /** The ancestors' debt's first state: a planet in 9 with Mercury in its root (1952 p. 128). */
  readonly pitri: readonly { readonly ninth: Graha; readonly mercury: number }[];
  readonly flags: {
    readonly ratandha: boolean;
    readonly nabalig: boolean;
    readonly dharmi: readonly Graha[];
    readonly sathi: readonly (readonly Graha[])[];
  };
}

/** A chart read as Lal Kitab reads it (`03-design/lalkitab.md`). */
export interface LalKitab {
  readonly reading: LalKitabReading;
  /** Where the cycle was started. */
  readonly cycle: { readonly planet: Graha; readonly year: number };
  /** The 35-year cycle's periods over years 1 to 120 of life, both ends included. */
  readonly periods: readonly { readonly planet: Graha; readonly from: number; readonly to: number }[];
  /** The year asked for; `null` unless the request named one. */
  readonly year: {
    readonly year: number;
    readonly ruler: Graha;
    /** The planets of months 1–4, 5–8 and 9–12 (1952 p. 34). */
    readonly thirds: readonly Graha[];
    /** The annual teva's reading; `null` without a `varshphal` list. */
    readonly annual: LalKitabReading | null;
  } | null;
}

/** A chart's remedies: whom to propitiate, why, how, and the chosen deity (`03-design/remedies.md`). */
export interface Remedies {
  /** The readings it was given under, every member filled. */
  readonly rules: {
    readonly functional: Required<NonNullable<RemedyRules['functional']>>;
    readonly shanti: Required<NonNullable<RemedyRules['shanti']>>;
    readonly devata: Required<NonNullable<RemedyRules['devata']>>;
  };
  /** What the lagna's lordships make of the seven grahas (Laghu Parashari). */
  readonly functional: {
    readonly lagna: Rashi;
    readonly scheme: 'LAGHU_PARASHARI' | 'BASELINE';
    readonly rows: readonly {
      readonly graha: Graha;
      /** The houses it lords, counted from the lagna. */
      readonly houses: readonly number[];
      readonly clauses: readonly { readonly kind: FunctionalClauseKind; readonly house: number }[];
      readonly nature: 'YOGAKARAKA' | 'BENEFIC' | 'NEUTRAL' | 'MALEFIC';
    }[];
    readonly yogakarakas: readonly Graha[];
    /** The lords of the 2nd and the 7th (LP v. 23). */
    readonly marakas: readonly Graha[];
    /** The 11th from a movable lagna, the 9th from a fixed, the 7th from a dual, and its lord. */
    readonly badhaka: { readonly house: number; readonly lord: Graha };
  };
  /** The grahas to propitiate, each with its reasons. */
  readonly subjects: {
    /** Each graha with at least one reason, in catalogue order, unranked. */
    readonly subjects: readonly { readonly graha: Graha; readonly reasons: readonly RemedyReason[] }[];
    /** The running antardaśā's śānti and which of its conditions hold; `null` without `at`. */
    readonly antardasha: {
      readonly shanti: {
        readonly mahadasha: Graha;
        readonly antardasha: Graha;
        readonly chapter: number;
        readonly verses: string;
        readonly page: number;
        /** Any of which brings the evil; empty when the verses name none. */
        readonly conditions: readonly DashaShantiCondition[];
        readonly remedies: readonly DashaRemedy[];
      };
      /** Whether each condition holds, in the printed order; `null` where the verse leaves it open. */
      readonly holds: readonly (boolean | null)[];
    } | null;
  };
  /** The graha-śānti of each subject, in their order (BPHS ch. 84). */
  readonly shantis: readonly RemedyShanti[];
  /** The ishṭa-devatā, the 12th from the kārakāṁśa, in both charts. */
  readonly ishtaDevata: {
    readonly atmakaraka: Graha;
    readonly karakamsha: Rashi;
    /** Read from the kārakāṁśa in the rāśi chart. */
    readonly inRasi: IshtaDevata;
    /** Read from the kārakāṁśa in the navāṁśa. */
    readonly inNavamsha: IshtaDevata;
    /** The same read from the amātyakāraka (vv. 76–79). */
    readonly amatya: {
      /** The amātyakāraka, under the chart's chara kāraka scheme. */
      readonly graha: Graha;
      /** Its navāṁśa sign, which the 12th is counted from (C357). */
      readonly amsha: Rashi;
      /** Read with the grahas and the lagna placed by the rāśi chart. */
      readonly inRasi: AmatyaDevata;
      /** Read with the grahas and the lagna placed by the navāṁśa. */
      readonly inNavamsha: AmatyaDevata;
    };
  };
}

/** One stretch of a day under one nadi and one tattva (Shiva Svarodaya, 1899 vv. 62–63, 71–72, 154, 197). */
export interface SvarodayaRun extends Interval {
  /** The Moon's (left, female) or the Sun's (right, male), v. 60. */
  readonly nadi: 'MOON' | 'SUN';
  /** Its turn of two and a half ghatis in the day, 0 the one rising at sunrise, to 23. */
  readonly turn: number;
  /** In each turn air, fire, earth and water, then ether at the junction. */
  readonly tattva: 'PRITHVI' | 'JALA' | 'AGNI' | 'VAYU' | 'AKASHA';
  /** The sex v. 60 gives the nadi. */
  readonly sex: 'MALE' | 'FEMALE';
}

/** One test of the lagna against one point of one purifier. */
export interface PurifierClause {
  readonly purifier: 'PRANAPADA' | 'GULIKA' | 'MOON';
  /** Which of its points. */
  readonly reference: 'ITSELF' | 'SEVENTH' | 'NAVAMSHA' | 'NAVAMSHA_SEVENTH';
  /** The sign that point stands in. */
  readonly sign: Rashi;
  /** The lagna's sign. */
  readonly lagna: Rashi;
  /** The lagna's house counted from that sign, one to twelve. */
  readonly house: number;
  /** Whether the house is one that purifies this native. */
  readonly held: boolean;
  /** Whether the clause counts toward the verdict: false only for v. 76's extension while the pranapada or the Moon holds. */
  readonly counted: boolean;
}

/** What the purifier finds at an instant: every clause, and whether any counted one held (v. 75). */
export interface PurifierVerdict {
  /** Every clause judged, in the order pranapada, Gulika, Moon. */
  readonly clauses: readonly PurifierClause[];
  readonly pure: boolean;
}

/** One run of the window between two edges, with the verdict every instant of it shares. */
export interface PurifiedRun extends Interval {
  readonly verdict: PurifierVerdict;
}

/** What the baseline's stages say they did, by `kind`. */
export type BaselineNote =
  | {
      readonly kind: 'TATTVA_SEX';
      readonly sex: 'MALE' | 'FEMALE';
      /** The minutes of every 90 that admit it. */
      readonly admittedMinutes: number;
      /** The candidates penalised, not excluded. */
      readonly penalised: number;
      /** The candidates. */
      readonly of: number;
    }
  | {
      readonly kind: 'REPORTED_TIME';
      readonly accuracy: 'EXACT' | 'APPROXIMATE' | 'RECTIFIED' | 'UNKNOWN';
      /** Its uncertainty, minutes. */
      readonly uncertaintyMinutes: number;
    }
  | {
      readonly kind: 'EVENT_FIT';
      /** Its index in the request. */
      readonly event: number;
      /** The caller's name for it; `null` when it had none. */
      readonly id: string | null;
      readonly eventKind: LifeEventKind;
      /** The period lords that fit it, mahadasha first, where any did. */
      readonly lords: readonly Graha[];
      /** Its best score over every candidate, weighted. */
      readonly contribution: number;
    };

/** A chart read as a birth time to rectify: each member `null` unless asked (`03-design/rectification.md`). */
export interface Rectification {
  /** What the purifier leaves standing of the window. */
  readonly purified: {
    /** The maximal runs no bar removed, in order; every run under `purifyAs: 'WEIGHT'`. */
    readonly intervals: readonly PurifiedRun[];
    /** The runs a bar removed, in order, each verdict naming every clause that failed. */
    readonly removed: readonly PurifiedRun[];
    /** Every instant inside the window where a clause changes. */
    readonly edges: readonly number[];
    /** The seed grid, so a run reproduces. */
    readonly grid: { readonly stepDays: number; readonly cells: number };
  } | null;
  /** The three reports the birth gives beside the purifier. */
  readonly conception: {
    /** The birth read, a Julian day (UTC). */
    readonly birth: number;
    /** The pranapada's house from the lagna, and the birth it judges (Jha's ch. 3 vv. 73–74). */
    readonly pranapadaHouse: {
      readonly pranapadaDeg: number;
      readonly lagnaDeg: number;
      readonly house: number;
      readonly auspicious: boolean;
    };
    /** The conception BPHS counts back to, and its lagna judged. */
    readonly nisheka: {
      readonly count: {
        /** The points v. 27 reads at the birth, degrees. */
        readonly points: {
          readonly mandiDeg: number;
          readonly saturnDeg: number;
          readonly lagnaDeg: number;
          readonly ninthDeg: number;
          readonly lagnaLordDeg: number;
          readonly moonDeg: number;
        };
        /** v. 27's two arcs, their sum and the span before birth they give. */
        readonly span: {
          readonly saturnToMandiDeg: number;
          readonly lagnaToNinthDeg: number;
          /** The Moon's degrees in her sign, added when the lagna's lord is in the invisible half (v. 28). */
          readonly moonAddedDeg: number | null;
          readonly arcDeg: number;
          /** The arc as v. 28 writes it. */
          readonly written: { readonly months: number; readonly days: number; readonly ghatis: number; readonly palas: number };
          readonly daysBefore: number;
        };
        /** The conception, a Julian day (UTC): the birth less the span. */
        readonly instant: number;
        /** How many days the conception moves when the birth moves a minute later. */
        readonly daysPerBirthMinute: number;
      };
      /** The conception's lagna, degrees, in the conception chart's own zodiac. */
      readonly lagnaDeg: number;
      /** That lagna under the purifier, at the birth's place (v. 29). */
      readonly verdict: PurifierVerdict;
    };
    /** BJ IV.21 read at the conception against the birth. */
    readonly moon: {
      readonly predicted: {
        /** The dvadashamsha the Moon occupies in her sign, one to twelve. */
        readonly dvadashamsha: number;
        /** The sign the Moon holds at birth. */
        readonly sign: Rashi;
        /** Bhattotpala's nakshatra, under `count: 'NEXT_AFTER_DVADASHAMSHA'` only. */
        readonly nakshatra: Nakshatra | null;
      };
      readonly moonSign: Rashi;
      readonly moonNakshatra: Nakshatra | null;
      readonly signAgrees: boolean;
      /** `null` where no nakshatra is predicted. */
      readonly nakshatraAgrees: boolean | null;
      /** The conception's rising sign or navamsha. */
      readonly rising: Rashi;
      readonly predictedPart: 'DAY' | 'NIGHT' | 'EITHER';
      readonly bornByDay: boolean;
      readonly partAgrees: boolean;
      readonly risenFraction: number;
      readonly elapsedFraction: number;
    };
  } | null;
  /** *Brihat Jataka* ch. V's circumstances. */
  readonly circumstance: {
    /** The sky the clauses read: the lagna, the seven grahas Sun to Saturn, degrees. */
    readonly sky: { readonly lagnaDeg: number; readonly grahasDeg: readonly number[]; readonly lordRetrograde: boolean };
    /** V.1–2: the father. */
    readonly father: {
      /** The Moon's aspect on the lagna's sign (BJ II.13). */
      readonly moonAspect: Strength;
      readonly unseen: boolean;
      readonly saturnRising: boolean;
      readonly marsSetting: boolean;
      readonly moonHemmed: boolean;
      /** Whether any of them holds: the father away. */
      readonly away: boolean;
      /** Where he is, where V.1 holds and the Sun has fallen from the 10th. */
      readonly whereabouts: 'ABROAD' | 'OWN_COUNTRY' | 'RETURNING' | null;
      readonly sunHouse: number;
    };
    /** V.17: the presentation. */
    readonly presentation: {
      readonly by: 'RISING_SIGN' | 'LAGNA_LORD_MOTION';
      /** How the rising sign rises. */
      readonly rising: 'SIRSHODAYA' | 'PRISHTODAYA' | 'UBHAYODAYA';
      /** The lagna's lord. */
      readonly lord: Graha;
      readonly lordRetrograde: boolean;
      readonly foretold: 'HEAD' | 'FEET' | 'HANDS';
    };
    /** V.18: the lamp, one at the start of the sign and none at its end. */
    readonly lamp: {
      readonly oil: number;
      readonly oilLevel: 'FULL' | 'HALF' | 'SPENT';
      readonly wick: number;
      readonly wickLevel: 'FULL' | 'HALF' | 'SPENT';
    };
    /** V.22: the attendants. */
    readonly attending: {
      readonly between: readonly Graha[];
      /** Those of them in the visible half. */
      readonly visible: readonly Graha[];
      readonly inside: number;
      readonly outside: number;
    };
    /** One per fact given, set against the clause that reads it. */
    readonly weights: readonly {
      readonly indication:
        | 'FATHER'
        | 'PRESENTATION'
        | 'OIL'
        | 'WICK'
        | 'ATTENDANTS_TOTAL'
        | 'ATTENDANTS_INSIDE'
        | 'ATTENDANTS_OUTSIDE';
      readonly agrees: boolean;
    }[];
  } | null;
  /** The baseline engine's cascade around the chart's instant. */
  readonly baseline: {
    readonly window: Interval;
    /** The sunrise the tattva cycle counted from, a Julian day (UTC). */
    readonly sunrise: number;
    /** The intervals holding the asked coverage of the posterior. */
    readonly intervals: readonly Interval[];
    readonly intervalWidthMinutes: number;
    /** The finest the stages that told candidates apart can tell, minutes. */
    readonly resolutionMinutes: number;
    /** The posterior's mode, a Julian day (UTC). */
    readonly suggested: number;
    /** How concentrated the posterior is, 0 (flat) to 1. */
    readonly concentration: number;
    /** The most probable candidates, best first. */
    readonly candidates: readonly {
      readonly at: number;
      readonly probability: number;
      readonly logPosterior: number;
      readonly lagna: Rashi;
      readonly lagnaNakshatra: Nakshatra;
    }[];
    readonly stages: readonly {
      readonly stage: 'PRIOR' | 'DASHA_BOUNDARY';
      readonly applied: boolean;
      /** Whether it told no candidate from another. */
      readonly flat: boolean;
      readonly resolutionMinutes: number;
      readonly notes: readonly BaselineNote[];
    }[];
    readonly eventsUsed: number;
    readonly eventsHeldOut: number;
    /** The held-out events, tested against the fit. */
    readonly holdOut: readonly {
      readonly event: number;
      readonly kind: LifeEventKind;
      readonly scoreAtFit: number;
      /** Its mean score over candidates spread across the grid. */
      readonly baseline: number;
      readonly supported: boolean;
    }[];
  } | null;
  /** The Shiva Svarodaya around the chart's instant: a report, never a bar. */
  readonly svarodaya: {
    /** The reading at the chart's instant. */
    readonly at: {
      /** The sunrise the turns are counted from, a Julian day (UTC). */
      readonly sunrise: number;
      readonly nextSunrise: number;
      /** The tithi at that sunrise, which gives its nadi (v. 62). */
      readonly tithi: Tithi;
      readonly sunriseNadi: 'MOON' | 'SUN';
      readonly run: SvarodayaRun;
      /** The turn's start and end, where the sushumna flows for a moment. */
      readonly junctions: readonly [number, number];
    };
    /** Every run inside the window, in order and clipped to it. */
    readonly runs: readonly SvarodayaRun[];
  } | null;
}

/** A graha's śānti: its image, ṛk, japa, fuel, food, fee, gem and place (BPHS ch. 84). */
export interface RemedyShanti {
  readonly graha: Graha;
  readonly image: 'COPPER' | 'CRYSTAL' | 'RED_SANDALWOOD' | 'GOLD' | 'SILVER' | 'IRON' | 'LEAD' | 'BRONZE';
  /** Its ṛk's opening words in IAST, as BPHS 84.17–18 prints them, or Yājñavalkya I.301 for Rahu under `YAJNAVALKYA`. */
  readonly rik: string;
  /** Its japa count in thousands (BPHS 84.19–20). */
  readonly japaThousands: number;
  readonly samidh: 'ARKA' | 'PALASHA' | 'KHADIRA' | 'APAMARGA' | 'PIPPALA' | 'UDUMBARA' | 'SHAMI' | 'DURVA' | 'KUSHA';
  readonly food: 'GUDAUDANA' | 'PAYASA' | 'HAVISHYA' | 'KSHIRA_SHASHTIKA' | 'DADHYODANA' | 'HAVIS' | 'CHURNA' | 'MAMSA' | 'CHITRANNA';
  readonly dakshina: 'MILCH_COW' | 'CONCH' | 'BULL' | 'GOLD' | 'CLOTH' | 'HORSE' | 'BLACK_COW' | 'IRON' | 'GOAT';
  readonly gem: 'RUBY' | 'PEARL' | 'CORAL' | 'EMERALD' | 'TOPAZ' | 'DIAMOND' | 'SAPPHIRE' | 'HESSONITE' | 'CATS_EYE';
  /** The substance it rules (Brihat Jataka II.12); `null` for the nodes. */
  readonly substance: 'COPPER' | 'GEMS' | 'GOLD' | 'ALLOY' | 'SILVER' | 'PEARLS' | 'IRON' | null;
  /** Its direction (Brihat Jataka II.5); `null` for Ketu, which the verse leaves out (C345). */
  readonly direction: Direction | null;
  /** Its place in the Matsya maṇḍala. */
  readonly mandala: 'CENTRE' | 'EAST' | 'SOUTHEAST' | 'SOUTH' | 'SOUTHWEST' | 'WEST' | 'NORTHWEST' | 'NORTH' | 'NORTHEAST';
}

/** The 12th from the kārakāṁśa in one chart, read as BPHS vv. 70–76 read it. */
export interface IshtaDevata {
  readonly rules: Required<NonNullable<RemedyRules['devata']>>;
  /** The twelfth sign from the kārakāṁśa. */
  readonly sign: Rashi;
  /** Each graha in it with its devotion, in catalogue order; empty when the sign is empty. */
  readonly devotions: readonly Devotion[];
  /**
   * The grahas there that make a devotee of minor deities in a sign a natural malefic rules:
   * Saturn or Venus from the kārakāṁśa (vv. 75–76), every natural malefic too from the amātya (v. 78).
   */
  readonly minor: readonly Graha[];
}

/** A graha and the devotion its verse names (BPHS, the 1923 print's ch. 9 vv. 70–74). */
export interface Devotion {
  readonly graha: Graha;
  readonly deities: readonly IshtaDeity[];
  /** The verse, in the 1923 print's ch. 9. */
  readonly verse: number;
  /** Whether Ketu stands in the same sign (C355); always false of the nodes. */
  readonly withKetu: boolean;
}

/** The amātya's devotions in one chart, read as BPHS vv. 77–79 read them. */
export interface AmatyaDevata {
  /** The 12th from the amātya's navāṁśa, read as the kārakāṁśa's (vv. 77–78). */
  readonly twelfth: IshtaDevata;
  /** The amātya's own sign in this chart. */
  readonly sign: Rashi;
  /** Its house by sign from this chart's lagna, 1 to 12 (v. 79). */
  readonly house: number;
  /** Each graha joined to it in that sign, with its devotion (v. 79). */
  readonly joined: readonly Devotion[];
}

/** A deity the ishṭa-devatā verses name (BPHS, the 1923 print's ch. 9 vv. 70–76). */
export type IshtaDeity =
  | 'SHIVA'
  | 'GAURI'
  | 'PARVATI'
  | 'LAKSHMI'
  | 'SKANDA'
  | 'VISHNU'
  | 'DURGA'
  | 'HERAMBA'
  | 'SURYA'
  | 'MINOR'
  | 'CHANDRA'
  | 'SARASWATI'
  | 'NARASIMHA'
  | 'HANUMAN'
  | 'DATTATREYA'
  | 'SHANI'
  | 'KRISHNA'
  | 'BRIHASPATI';

/** Why a graha is to be propitiated. */
export type RemedyReason =
  | 'MAHADASHA'
  | 'ANTARDASHA'
  | 'DEBILITATED'
  | 'COMBUST'
  | 'DUSTHANA'
  | 'FUNCTIONAL_MALEFIC'
  | 'MARAKA'
  | 'BADHAKESHA';

/** A condition the antardasha's verses name for their remedy. */
export type DashaShantiCondition =
  | 'LORD_OF_SECOND_OR_SEVENTH'
  | 'LORD_OF_SEVENTH'
  | 'IN_SECOND_OR_SEVENTH'
  | 'IN_SECOND_SEVENTH_OR_EIGHTH'
  | 'WITH_EIGHTH_OR_TWELFTH'
  | 'EIGHTH_LORD_IN_EIGHTH'
  | 'SIXTH_OR_EIGHTH_FROM_DASHA_LORD_OR_WEAK'
  | 'DUSTHANA_FROM_DASHA_LORD_WITH_MALEFIC'
  | 'IN_SECOND_OR_SEVENTH_WITH_ITS_LORD'
  | 'RELATED_TO_SECOND_OR_SEVENTH_LORD'
  | 'EIGHTH_LORD_WITH_SECOND_OR_SEVENTH_LORD'
  | 'WITH_SECOND_OR_SEVENTH_LORDS'
  | 'LORD_OF_EIGHTH_OR_SEVENTH_OR_IN_SECOND'
  | 'LORD_OF_SECOND_AND_SIXTH'
  | 'SECOND_OR_SEVENTH_LORD_IN_SEVENTH'
  | 'JOINED_OR_ASPECTED';

/** A remedy the antardasha verses prescribe. */
export type DashaRemedy =
  | 'MRITYUNJAYA_JAPA'
  | 'MAHA_MRITYUNJAYA_JAPA'
  | 'RUDRA_JAPA'
  | 'DURGA_JAPA'
  | 'DURGA_DEVI_JAPA'
  | 'DURGA_LAKSHMI_JAPA'
  | 'DURGA_PATHA'
  | 'VISHNU_SAHASRANAMA'
  | 'SHIVA_SAHASRANAMA'
  | 'ADITYA_HRIDAYA_JAPA'
  | 'ISHTA_JAPA'
  | 'SUBRAHMA_JAPA_DANA'
  | 'SHANTI'
  | 'GRIHYA_SHANTI'
  | 'SHANTI_HOMA'
  | 'AYUTA_HOMA'
  | 'TILA_HOMA'
  | 'DARSHA_SHANTI'
  | 'SURYA_PRITI'
  | 'SURYA_PUJA'
  | 'CHANDRA_PRITI'
  | 'BUDHA_PRITI_DANA'
  | 'SHIVA_PUJA'
  | 'BRAHMANA_ARCANA'
  | 'DEVA_BRAHMANA_BHOJANA'
  | 'CHAGA_DANA'
  | 'SHVETA_GO_MAHISHI'
  | 'KRISHNA_GO_MAHISHI'
  | 'SHVETA_GO_RAJATA'
  | 'GO_DANA'
  | 'DHENU_DANA'
  | 'KAPILA_GO_DANA'
  | 'SVARNA_DHENU'
  | 'GO_BHU_HIRANYA_DANA'
  | 'ANADVAN_DANA'
  | 'ASHVA_DANA'
  | 'NAGA_DANA'
  | 'SVARNA_DANA'
  | 'SVARNA_PRATIMA_DANA'
  | 'RAJATA_PRATIMA_DANA'
  | 'ANNA_DANA'
  | 'VASTRA_DANA'
  | 'GUDA_GHRITA_DADHI_TANDULA';

/** A Laghu Parashari clause that made a graha's nature. */
export type FunctionalClauseKind =
  | 'LAGNESHA'
  | 'TRIKONA_LORD'
  | 'TRISHADAYA_LORD'
  | 'RANDHRESHA'
  | 'RANDHRESHA_VOID_LAGNESHA'
  | 'RANDHRESHA_VOID_LUMINARY'
  | 'KENDRA_BENEFIC_LOSES'
  | 'KENDRA_MALEFIC_LOSES'
  | 'BY_ASSOCIATION'
  | 'YOGAKARAKA'
  | 'MARAKA'
  | 'BADHAKESHA';

/** An arc of the zodiac, half-open, in nanoarcseconds (divide by `3.6e12` for degrees). */
export interface KpSpan {
  readonly start: number;
  readonly end: number;
}

/** One level of a point's lords below the sign: its lord, and the arc it rules. */
export interface KpLevel {
  readonly lord: Graha;
  readonly span: KpSpan;
}

/** A point's lords: of its sign, its star, its sub and its sub-sub. */
export interface KpLords {
  readonly sign: Graha;
  readonly star: KpLevel;
  readonly sub: KpLevel;
  readonly subSub: KpLevel;
}

/** A cusp, its longitude in nanoarcseconds of the sidereal zodiac. */
export interface KpCusp {
  /** 1 to 12. */
  readonly house: number;
  readonly longitude: number;
  readonly lords: KpLords;
}

/** A planet, its longitude in nanoarcseconds of the sidereal zodiac. */
export interface KpPlanet {
  readonly graha: Graha;
  readonly longitude: number;
  readonly retrograde: boolean;
  /** The house whose cusp arc holds it, 1 to 12. */
  readonly house: number;
  readonly lords: KpLords;
}

/** A chart as KP reads it: its cusps, the horary number's when one was named, and its planets. */
export interface KpChart {
  readonly system: HouseSystem;
  readonly cusps: readonly KpCusp[];
  readonly planets: readonly KpPlanet[];
}

/** A house's significators in KP Reader VI's order, strongest first (C154). */
export interface KpHouseSignificators {
  readonly house: number;
  /** (a) Planets in the stars of the house's occupants. */
  readonly inOccupantsStars: readonly Graha[];
  /** (b) The occupants. */
  readonly occupants: readonly Graha[];
  /** (c) Planets in the star of the house's lord. */
  readonly inLordsStar: readonly Graha[];
  /** (d) The house's lord. */
  readonly lord: Graha;
  /** (e) Planets joined to a significator above. */
  readonly conjoined: readonly Graha[];
  /** (f) Planets aspecting the house under the settings' node aspects. */
  readonly aspected: readonly Graha[];
  /** Signs wholly inside the house. */
  readonly intercepted: readonly Rashi[];
}

/** What a node stands for, in Reader VI's order (C155). */
export interface KpNodeAgency {
  readonly node: Graha;
  readonly conjoined: readonly Graha[];
  readonly starLord: Graha;
  readonly aspecting: readonly Graha[];
  readonly signLord: Graha;
}

/** A chart's significators: the twelve houses, and the nodes' agency. */
export interface KpSignificators {
  readonly houses: readonly KpHouseSignificators[];
  readonly nodes: readonly KpNodeAgency[];
}

/** Why a planet is a ruling planet; an agent is a node standing for a ruler (C152). */
export type KpReason =
  | {
      readonly kind:
        | 'LAGNA_STAR'
        | 'LAGNA_SIGN'
        | 'LAGNA_SUB'
        | 'MOON_STAR'
        | 'MOON_SIGN'
        | 'MOON_SUB'
        | 'DAY_LORD';
    }
  | { readonly kind: 'AGENT'; readonly of: Graha; readonly by: 'IN_ITS_SIGN' | 'CONJOINED' };

/** A retrograde planet rejecting a ruler through its star, or its sub (C153). */
export interface KpRejection {
  readonly retrograde: Graha;
  readonly byStar: boolean;
}

/** One ruling planet, every reason it rules, and what rejects it. */
export interface KpRuler {
  readonly graha: Graha;
  /** Every reason it rules, the first the strongest. */
  readonly reasons: readonly KpReason[];
  /** Itself retrograde, which the Reader reads as delay and not rejection. */
  readonly retrograde: boolean;
  /** What rejects it under the settings; `null` when it stands. */
  readonly rejectedBy: KpRejection | null;
  /** What would reject it under the other reading of C153. */
  readonly rejectedBySub: KpRejection | null;
}

/** The ruling planets of a moment, and the settings they were read under. */
export interface KpRuling {
  readonly rulers: readonly KpRuler[];
  readonly rules: {
    readonly count: 'FIVE' | 'WITH_SUBS';
    readonly nodeRulers: 'SIGN_OR_CONJOINED' | 'SIGN';
    readonly retrogradeRejection: 'STAR' | 'STAR_OR_SUB';
  };
}

/**
 * A chart read as KP: the chart, its significators and the ruling planets
 * of its moment.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, kp: { number: 74 } });
 * const standing = chart.kp?.ruling.rulers.filter((ruler) => ruler.rejectedBy === null);
 */
export interface KpReading {
  readonly chart: KpChart;
  readonly significators: KpSignificators;
  readonly ruling: KpRuling;
}

/** The transits to read against every chart of a request. */
export interface GocharRequest {
  /** The instants, UTC Julian days: at least one. */
  readonly instants: ArrayLike<number>;
  /** What to count the houses from: the natal Moon by default (v. 1). */
  readonly from?: GocharFrom;
  /** Whether to judge the seven by the natal Ashtakavarga too; false by default. */
  readonly ashtakavarga?: boolean;
}

/**
 * A chart's dasha phala, read under `dasha.shanta_sign`.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, dashaPhala: true });
 * const saturn = chart.dashaPhala?.grahas.find((g) => g.graha === 'graha.SATURN');
 */
export interface DashaPhalaReading {
  /** Each graha's, Sun to Ketu. */
  readonly grahas: readonly GrahaDashaPhala[];
}

/** A chart's Vaiseshikamsa. */
export interface VaiseshikamsaReading {
  /** Each graha's, Sun to Saturn. */
  readonly grahas: readonly GrahaVaiseshikamsa[];
}

/** A chart's Vimshopaka: each graha's strength across the divisional charts. */
export interface Vimshopaka {
  /** How each varga was scored. */
  readonly scoring: VimshopakaScoring;
  /** Each graha's, Sun to Saturn. */
  readonly grahas: readonly GrahaVimshopaka[];
}

/** A registered layout's full key, as the context that registered it resolves it. */
export type LayoutKey = `chart_layout.${string}`;

/** The annual charts a request asks for. */
export interface VarshaRequest {
  /** Which longitude the Sun returns to; `SIDEREAL` by default. */
  readonly reading?: ReturnLongitude;
  /** The last year of life wanted, 1 to 200. */
  readonly through: number;
  /** Where the Muntha stands inside its sign; `SIGN_START` by default. */
  readonly muntha?: MunthaDegree;
  /**
   * Where each year's own chart is cast, when you want the charts and not
   * only their instants: `'birth'`, or a residence in the shape `found`
   * takes. **Absent, none is founded** — the SDK does not choose between
   * the birthplace and a residence for you, because the schools differ.
   */
  readonly place?: 'birth' | AnnualPlace;
  /** The readings the year lord's chain parts on, where authorities differ. */
  readonly varshesha?: VarsheshaRules;
  /**
   * The matters each year's sixteen Tajika yogas are judged for: `'all'`,
   * or house numbers 1 to 12 in the order you want them answered. Fourteen
   * of the sixteen are judgements about the lagnesha and the lord of the
   * house asked about, so the yogas answer a matter and not a chart.
   * **Needs `place`**; absent, none is judged.
   *
   * @example { through: 40, place: 'birth', matters: [7, 10] }
   */
  readonly matters?: 'all' | readonly number[];
  /** The readings the sixteen part on, where the source leaves a choice. */
  readonly yogas?: YogaRules;
  /**
   * The sahams each year's chart is read for: `'all'`, the forty-one in
   * the source's order, or their keys in the order you want them answered.
   * **Needs `place`**; absent, none is read.
   *
   * @example { through: 40, place: 'birth', sahams: ['PUNYA', 'VIVAHA'] }
   */
  readonly sahams?: 'all' | readonly Saham[];
  /** The readings the sahams part on, where the sources differ. */
  readonly sahamRules?: SahamRules;
  /** The readings a saham's strength parts on. */
  readonly sahamStrength?: SahamStrengthReadings;
  /** The Harsha bala's reading of Venus's house of joy. */
  readonly harshaRules?: HarshaRules;
  /**
   * The annual dashas each year is divided by: `'all'`, the three in the
   * catalogue's order, or their keys in the order you want them answered.
   * The Sun is read over a year once however many are asked for.
   * **Needs `place`**; absent, none is read.
   *
   * @example { through: 40, place: 'birth', dashas: [DashaSystem.Mudda] }
   */
  readonly dashas?: 'all' | readonly AnnualDashaSystem[];
  /** The readings the annual dashas part on, where the sources differ. */
  readonly dashaRules?: AnnualDashaRules;
}

/**
 * The annual dashas: the Patyayini, read from the year's own chart, and the
 * two nakshatra years — the `DashaSystem` keys a year's dasha is read back
 * as, so `DashaSystem.Mudda` asks for one.
 */
export type AnnualDashaSystem = 'dasha_system.PATYAYINI' | 'dasha_system.MUDDA' | 'dasha_system.VARSHA_YOGINI';

/** One lord of the ring a year's dasha runs round. */
export interface AnnualDashaShare {
  /** Its lord: the graha, or the sign's lord when the share is a sign's. */
  readonly lord: Graha | 'unknown';
  /** The sign, when the share is one's: the Patyayini's lagna; `null` for a planet's. */
  readonly sign: Rashi | 'unknown' | null;
  /**
   * Its weight, of which a lord's share of the year is its weight over the
   * ring's: a nakshatra year's lord's natal years, or a Patyayini share's
   * patyamsha in nanoarcseconds. 0 for a lord that runs for no time.
   */
  readonly weight: number;
}

/** One annual dasha of a year: its ring, the year it divides, and its periods. */
export interface AnnualDasha {
  /** Which of the three. */
  readonly system: AnnualDashaSystem | 'unknown';
  /** The birth Moon's nakshatra, which seeds a nakshatra year; `null` for the Patyayini. */
  readonly seed: Nakshatra | 'unknown' | null;
  /** The lord the year opens with: `ring.shares[ring.first].lord`. */
  readonly firstLord: Graha | 'unknown';
  /** The lords the year runs round. */
  readonly ring: {
    /** In the order the ring runs. */
    readonly shares: readonly AnnualDashaShare[];
    /** The place in `shares` the year opens with, from 0. */
    readonly first: number;
    /**
     * How much of the first lord's share was still to run at the return, 0
     * to 1, the rest closing the year; `null` when it runs whole from the
     * return: the Patyayini, and a `'WHOLE'` balance.
     */
    readonly remaining: number | null;
  };
  /** The year, Julian days (UTC): from its return to where the clock closes it, under the default clock the next return. */
  readonly year: { readonly from: number; readonly to: number };
  /** Every period to the rules' depth, depth first in time order; a period that runs for no time is not listed. */
  readonly periods: readonly DashaPeriod[];
  /** The periods running at a Julian day (UTC), from the mahadasha down; empty outside the year. */
  at(jd: number): readonly DashaPeriod[];
}

/** A residence to cast each year's chart for, in `found`'s own place shape. */
export interface AnnualPlace {
  /** Degrees north. */
  readonly latitude: number;
  /** Degrees east. */
  readonly longitude: number;
  /** Metres; 0 when absent. */
  readonly altitude?: number;
  /** The clock kept there, seconds east of UTC. */
  readonly utcOffsetSeconds: number;
}

/** The annual chart's five office-bearers, one of whom becomes the year's lord. */
export interface OfficeBearers {
  /** The lord of the Muntha's sign. */
  readonly muntha: Graha | 'unknown';
  /** The lord of the birth lagna. */
  readonly janmaLagna: Graha | 'unknown';
  /** The lord of the annual lagna. */
  readonly varshaLagna: Graha | 'unknown';
  /** The annual lagna's triplicity lord for the part of the day. */
  readonly triRashi: Graha | 'unknown';
  /** The lord of the Sun's sign by day, of the Moon's by night. */
  readonly dinaRatri: Graha | 'unknown';
}

/**
 * A Tajika strength, exact. The boundary carries it as an integer count of
 * **sub-sub units**, 3600 to a unit, because two office-bearers a sub-sub
 * unit apart decide a year between them.
 */
export interface Bala {
  /** Whole units, at most twenty: the figure a reader compares. */
  readonly units: number;
  /** The sub-units after those, 0 to 59. */
  readonly subUnits: number;
  /** The sub-sub units after those, 0 to 59. */
  readonly subSub: number;
  /** The whole of it in sub-sub units: what to compare and sum. */
  readonly total: number;
  /** `14:20:15`, as the sources write one. */
  toString(): string;
}

/** One office-bearer's claim on the year's lordship. */
export interface YearClaim {
  /** Whose claim it is. */
  readonly graha: Graha | 'unknown';
  /** Its five-fold strength. */
  readonly vishwa: Bala;
  /** How many of the five offices it holds, 1 to 5: the tie-break. */
  readonly portfolios: number;
  /** Whether it gives the Tajika aspect to the annual lagna, which it must to hold the year. */
  readonly aspectsLagna: boolean;
}

/** The lord of the year, and the reckoning it came out of. */
export interface YearLord {
  /** The lord of the year. */
  readonly graha: Graha | 'unknown';
  /** Which step of the chain decided it. */
  readonly chosen: VarsheshaChosen | 'unknown';
  /** Its five-fold strength. */
  readonly vishwa: Bala;
  /** Whether the Moon led on strength and stepped aside, being "unable to govern". */
  readonly moonPassedOver: boolean;
  /** Every claimant, strongest first, so the decision can be read rather than trusted. */
  readonly claims: readonly YearClaim[];
}

/** A return's own chart, read down to what Tajika reads from it. */
export interface AnnualChart {
  /** The annual chart's lagna, sidereal degrees, at the place it was cast for. */
  readonly lagnaDeg: number;
  /** Whether the return fell between sunrise and sunset there. */
  readonly byDay: boolean;
  /** The five office-bearers. */
  readonly officeBearers: OfficeBearers;
  /** The lord of the year, chosen among them. */
  readonly yearLord: YearLord;
  /**
   * The pairs of the seven that make a Tajika yoga in this chart. The
   * pairs that make none do not cross; `sdk.chart().drishtis` in Rust has
   * all twenty-one.
   */
  readonly yogas: readonly TajikaPair[];
  /** The seven retrograde in this chart: what the matters were judged on. */
  readonly retrograde: readonly (Graha | 'unknown')[];
  /** The seven combust in this chart, under the context's combustion table. */
  readonly combust: readonly (Graha | 'unknown')[];
  /** The sixteen yogas for each matter `varsha.matters` asked about, in its order; empty otherwise. */
  readonly matters: readonly TajikaMatter[];
  /** Each saham `varsha.sahams` asked for, in its order, with its strength under the year's lord; empty otherwise. */
  readonly sahams: readonly TajikaSaham[];
  /** The seven's Harsha bala in this year's chart, in the catalogue's order. */
  readonly harsha: readonly HarshaBala[];
  /** Each annual dasha `varsha.dashas` asked for, in its order, under `varsha.dashaRules`; empty otherwise. */
  readonly dashas: readonly AnnualDasha[];
}

/** One planet's Harsha bala: four places it is happy in, five units each. */
export interface HarshaBala {
  /** Whose. */
  readonly graha: Graha | 'unknown';
  /** The house it stands in, whole signs from the annual lagna. */
  readonly house: number;
  /** In its house of joy. */
  readonly sthana: boolean;
  /** In its exaltation or own sign. */
  readonly uchchaSwakshetra: boolean;
  /** In a house of its own gender, Tajika's genders. */
  readonly striPurusha: boolean;
  /** In a year opening at its own part of the day. */
  readonly dinaRatri: boolean;
  /** The parts held, five units each: 0 to 20. */
  readonly total: number;
  /** What the source calls that total. */
  readonly grade: HarshaGrade | 'unknown';
}

/** How one of the seven stands to a saham. */
export interface SahamSeven {
  /** Which planet. */
  readonly graha: Graha | 'unknown';
  /** The Tajika aspect its sign casts on the saham's. */
  readonly drishti: TajikaDrishti | 'unknown';
  /** How it stands to the saham's lord, under the friendship read. */
  readonly relation: TajikaRelation | 'unknown';
  /** Whether it keeps the saham company, in the saham's sign. */
  readonly company: boolean;
}

/** Where a saham fell in a year's chart, and what it fell in. */
export interface TajikaSaham {
  /** Which of the forty-one. */
  readonly saham: Saham | 'unknown';
  /** Where it fell, sidereal degrees in [0, 360). */
  readonly longitudeDeg: number;
  /** The sign it fell in. */
  readonly sign: Rashi | 'unknown';
  /** That sign's lord: the saham's lord, by whose strength the source judges it. */
  readonly lord: Graha | 'unknown';
  /** The house it fell in, 1 to 12, by whole signs from the annual lagna. */
  readonly house: number;
  /** Whether it was carried a sign further because c did not fall between b and a. */
  readonly addedSign: boolean;
  /**
   * The clauses of the source's strong list that hold. Reported and never
   * weighed: the source gives no score, and three sahams in five meet
   * clauses on both lists.
   */
  readonly strong: readonly (SahamStrong | 'unknown')[];
  /** The clauses of the source's weak list that hold. */
  readonly weak: readonly (SahamWeak | 'unknown')[];
  /** The saham lord's Panchavargiya Vishwa bala. */
  readonly lordVishwa: Bala;
  /** The saham lord's Harsha bala grade. */
  readonly lordHarsha: HarshaGrade | 'unknown';
  /** Whether it stands in the Rahu-Ketu axis; `null` when the chart placed no nodes. */
  readonly inNodeAxis: boolean | null;
  /** In the 6th, 8th or 12th, where the source calls a saham handicapped. */
  readonly handicapped: boolean;
  /** How each of the seven stands to it, in the catalogue's order. */
  readonly seven: readonly SahamSeven[];
}

/**
 * The sixteen Tajika yogas for one matter of a year: the question it asked
 * as well as the answer, because a list of yogas whose pair a reader
 * cannot see is not checkable.
 */
export interface TajikaMatter {
  /** The house asked about, 1 to 12, counted from the annual lagna. */
  readonly house: number;
  /** The sign that house falls in. */
  readonly sign: Rashi | 'unknown';
  /** The lord of the annual lagna. */
  readonly lagnesha: Graha | 'unknown';
  /** The lord of the house asked about. */
  readonly karyesha: Graha | 'unknown';
  /** One planet is both — always so of the first house — so there is no pair to judge. */
  readonly sameLord: boolean;
  /** How the two lords stand to each other; `null` when they are one planet. */
  readonly between: TajikaBetween | null;
  /** Every yoga that holds, once for each third planet that makes it. */
  readonly held: readonly HeldYearYoga[];
  /**
   * The yogas this call could not answer for. A yoga absent from `held`
   * did not hold **only** if it is not listed here.
   */
  readonly unanswered: readonly (YearYoga | 'unknown')[];
  /** Whether a yoga holds: `null` where this call could not say, which is not `false`. */
  holds(yoga: YearYoga): boolean | null;
}

/** One of the sixteen holding, with what made it hold. */
export interface HeldYearYoga {
  /** Which of the sixteen. */
  readonly yoga: YearYoga | 'unknown';
  /** The lords' own relation, where that is what made it; else `null`. */
  readonly between: TajikaBetween | null;
  /** The third planet it turns on, where one does. */
  readonly through: Graha | 'unknown' | null;
  /** The planet judged on entering the next sign: Gairi-Kamboola's Moon, Tambira's lord. */
  readonly entering: Graha | 'unknown' | null;
  /** How the third planet stands to each of the pair, read from the next sign for `entering`. */
  readonly legs: readonly [TajikaBetween, TajikaBetween] | null;
  /** The lords' afflictions, where those made it: Rudda and Durapha. */
  readonly afflictions: {
    readonly lagnesha: readonly (Affliction | 'unknown')[];
    readonly karyesha: readonly (Affliction | 'unknown')[];
  } | null;
}

/** Two planets of an annual chart, and what they make — which may be nothing. */
export interface TajikaBetween extends Omit<TajikaPair, 'yoga'> {
  /** What they are doing; `null` when they make neither an Ithasala nor an Ishrafa. */
  readonly yoga: TajikaYoga | 'unknown' | null;
}

/** Two planets of an annual chart, and what they make. */
export interface TajikaPair {
  /** The faster of the two by the tradition's ranking. */
  readonly faster: Graha | 'unknown';
  /** The slower. */
  readonly slower: Graha | 'unknown';
  /** The aspect between the signs they stand in. */
  readonly drishti: TajikaDrishti | 'unknown';
  /** What they are doing. */
  readonly yoga: TajikaYoga | 'unknown';
  /** The orb governing them, degrees: the mean of their deeptamshas. */
  readonly orbDeg: number;
  /** How far apart within their signs, degrees; positive when the faster is behind. */
  readonly apartDeg: number;
}

/**
 * The Muntha at one return: the birth lagna's sign advanced one sign for
 * each completed year, and that sign's lord — the Munthesha, first of the
 * annual chart's five office-bearers.
 */
export interface Muntha {
  /** The sign it has reached; the same under either `MunthaDegree`. */
  readonly sign: Rashi | 'unknown';
  /** The lord of that sign. */
  readonly lord: Graha | 'unknown';
  /** Its longitude at the return, degrees, under the reading asked for. */
  readonly longitudeDeg: number;
}

/** One annual chart's instant. */
export interface Pravesha {
  /**
   * How many years the native has completed at this instant: 1 is the first
   * return, a year after birth. Counted in returns and not in years of life,
   * because the two namings differ by one and both are in use.
   */
  readonly year: number;
  /** The instant, a Julian day (UTC), to pass to `found`. */
  readonly instant: number;
  /** The Muntha standing at it, progressed by this year's own count. */
  readonly muntha: Muntha;
  /** The year's own chart, or `null` unless `varsha.place` asked for it. */
  readonly annual: AnnualChart | null;
}

/** A registered dasha system's full key, as the context that registered it resolves it. */
export type DashaKey = `dasha_system.${string}`;

/**
 * A dasha system of your own, of either kernel, as `dashaSystems` takes it
 * (`03-design/dasha-kernels.md`, "A consumer's own system").
 *
 * The `kernel` is **stated** and never guessed from the fields present, so
 * a typo is refused by the field you wrote rather than by one you did not.
 *
 * @example
 * const ctx = new Context({
 *   dashaSystems: [{
 *     kernel: 'UDU',
 *     key: 'ACME_SAPTAKA',
 *     lords: ['SUN', 'MOON', 'MARS', 'MERCURY', 'JUPITER', 'VENUS', 'SATURN']
 *       .map((graha) => ({ graha, years: 10 })),
 *     reference: 'KRITTIKA',
 *   }, {
 *     kernel: 'RASHI',
 *     key: 'ACME_STHIRA',
 *     length: { BY_MODALITY: { movable: 7, fixed: 8, dual: 9 } },
 *   }],
 * });
 * ctx.chart.found({ instant, place, utcOffsetSeconds, dashas: ['dasha_system.ACME_SAPTAKA'] });
 */
export type DashaDefinition = UduDashaDefinition | RashiDashaDefinition;

/**
 * A nakshatra-seeded dasha system of your own. Keys are bare (`SUN`,
 * `KRITTIKA`), as the document spells them; every optional field defaults
 * to Vimshottari's shape.
 */
export interface UduDashaDefinition {
  /** The kernel that runs it: lords for years, seeded by the Moon's nakshatra. */
  readonly kernel: 'UDU';
  /** Its key: `[A-Z][A-Z0-9_]`, at most 48 characters, and not one the catalogue has. */
  readonly key: string;
  /** Where the table comes from. */
  readonly sources?: readonly string[];
  /** The lords, in the order they run, each with its whole years. */
  readonly lords: readonly { readonly graha: string; readonly years: number }[];
  /** The nakshatra that maps to the first lord, bare (`ASHWINI`). */
  readonly reference: string;
  /** Which way the seed is counted; forwards by default. */
  readonly count?: 'FROM_REFERENCE' | 'TO_REFERENCE';
  /** How many nakshatras each lord covers; one by default. */
  readonly span?: number;
  /** How many nakshatras each lord covers, lord by lord, when they differ: BPHS's Ashtottari is `[4, 3, 4, 3, 4, 3, 4, 3]`. None by default. */
  readonly groups?: readonly number[];
  /** The circle the seed is counted round: the 27 nakshatras by default, or the 28 with Abhijit. */
  readonly wheel?: 'NAKSHATRAS' | 'WITH_ABHIJIT';
  /** What is added after the division, before the modulo; none by default. */
  readonly offset?: number;
  /** Whether the lords run round the nakshatras again; true by default. */
  readonly repeats?: boolean;
  /** The factor on the mahadashas' years and the rounds in a cycle. */
  readonly scale?: { readonly numerator: number; readonly denominator: number; readonly rounds: number };
  /** The length of its year; `JULIAN_365_25` by default. */
  readonly year_length?: 'JULIAN_365_25' | 'SAVANA_360' | 'SIDEREAL' | 'TROPICAL' | 'LUNAR' | 'NAKSHATRA_324';
  /** How many levels of periods a reading carries, 1 to 6; three by default. */
  readonly depth?: number;
}

/** How long a sign's period runs, in a sign-based system. */
export type RashiLength =
  | 'COUNT_TO_LORD'
  | 'COUNT_TO_LORD_BY_DIGNITY'
  | { readonly FIXED: number }
  | { readonly BY_MODALITY: { readonly movable: number; readonly fixed: number; readonly dual: number } };

/**
 * A sign-based (Jaimini) dasha system of your own: its key, and optionally
 * where it starts, the order it visits the signs in, how long a sign runs,
 * which lord a mahadasha names, and the houses to start from the strongest
 * of. Everything unsaid is Chara's.
 */
export interface RashiDashaDefinition {
  /** The kernel that runs it: the twelve signs in an order, each for a number of years. */
  readonly kernel: 'RASHI';
  /** Its key: `[A-Z][A-Z0-9_]`, at most 48 characters, and not one the catalogue has. */
  readonly key: string;
  /** Where the row comes from. */
  readonly sources?: readonly string[];
  /** Where it starts; the lagna by default. */
  readonly start?: 'LAGNA' | 'ARUDHA_LAGNA' | 'NAVAMSA_LAGNA' | 'BRAHMA';
  /** The order it visits the signs in; every sign in turn by default. `FORWARD` is every sign in turn forward whatever the start's parity. */
  readonly order?: 'CONSECUTIVE' | 'TRINE_GROUPS' | 'DRISHTI_CHAIN' | 'LEAP' | 'FORWARD';
  /** How long a sign's period runs; the count to its stronger lord by default. */
  readonly length?: RashiLength;
  /** Which lord a mahadasha names; the stronger of a dual-lorded sign's two by default. */
  readonly named_lord?: 'STRONGER' | 'FIRST';
  /** The houses from the lagna to start from the strongest of; none by default. */
  readonly stronger_of?: readonly number[];
  /** The length of its year; `JULIAN_365_25` by default. */
  readonly year_length?: 'JULIAN_365_25' | 'SAVANA_360' | 'SIDEREAL' | 'TROPICAL' | 'LUNAR' | 'NAKSHATRA_324';
  /** How many levels of periods a reading carries, 1 to 6; three by default. */
  readonly depth?: number;
  /** Whether the cycle runs again for life whatever the settings' `after_cycle` says, as the Sudarshana chakra's does; false by default. */
  readonly recurs?: boolean;
}

/** What a grid cell always carries: a sign, or a house 1 to 12. */
export type LayoutHolds =
  | { readonly kind: 'SIGN'; readonly value: RashiName }
  | { readonly kind: 'HOUSE'; readonly value: number };

/** A sign as a layout row spells it: the bare key, `ARIES`. */
export type RashiName =
  | 'ARIES'
  | 'TAURUS'
  | 'GEMINI'
  | 'CANCER'
  | 'LEO'
  | 'VIRGO'
  | 'LIBRA'
  | 'SCORPIO'
  | 'SAGITTARIUS'
  | 'CAPRICORN'
  | 'AQUARIUS'
  | 'PISCES';

/** One region of a grid layout. */
export interface LayoutCell {
  /** The region's outline, in the unit square. */
  readonly outline: Outline;
  /** The sign or house the cell always carries. */
  readonly holds: LayoutHolds;
  /** Where the sign or house number is drawn. */
  readonly label: UnitPoint;
  /** Where the cell's bodies are stacked about. */
  readonly bodies: UnitPoint;
}

/** One ring of a radial layout. */
export interface LayoutRing {
  /** The inner radius, a fraction of the square's side; 0 makes wedges. */
  readonly inner: number;
  /** The outer radius, at most a half. */
  readonly outer: number;
  /** What the ring counts its first house from. */
  readonly counts_from: 'LAGNA' | 'MOON' | 'SUN' | 'CUSPS' | 'ZODIAC';
}

/** Twelve cells fixed in the row, or rings computed per chart. */
export type LayoutShape =
  | {
      readonly kind: 'GRID';
      readonly cells: readonly LayoutCell[];
      readonly frame: readonly Outline[];
      readonly direction: 'clockwise' | 'anticlockwise';
    }
  | {
      readonly kind: 'RADIAL';
      readonly rings: readonly LayoutRing[];
      /** The clock hour house 1 starts at, 1 to 12. */
      readonly starts_at: number;
      readonly direction: 'clockwise' | 'anticlockwise';
    };

/**
 * A chart layout as a row: its key, what cites it, and its shape. Crosses
 * as JSON with the SDK's own field names, as a theme does.
 */
export interface LayoutRow {
  /** The key, in the key grammar: `[A-Z][A-Z0-9_]`, at most 48 characters. */
  readonly key: string;
  /** The sources the row comes from; at least one. */
  readonly sources: readonly string[];
  /** Its cells or its rings. */
  readonly shape: LayoutShape;
}

/** How a drawing looks: every field optional, over the theme it extends. */
export interface ThemeStyle {
  /** The drawing's width and height, in SVG user units. */
  readonly size?: number;
  /** The page behind the chart, as `#rrggbb`. */
  readonly background?: string;
  /** Lines and text, as `#rrggbb`. */
  readonly ink?: string;
  /** A cell's fill, as `#rrggbb`. */
  readonly cell?: string;
  /** The fill of the cell the lagna stands in, as `#rrggbb`. */
  readonly lagna_cell?: string;
  /** The lagna's own label and mark, as `#rrggbb`. */
  readonly accent?: string;
  /** Line width, as a fraction of the size. */
  readonly stroke?: number;
  /** The font family every text asks for. */
  readonly font_family?: string;
  /** The largest a body's label is drawn, as a fraction of the size. */
  readonly body_size?: number;
  /** A cell's label, as a fraction of the size. */
  readonly label_size?: number;
  /** A body at its degree on a wheel, as a fraction of the size. */
  readonly mark_size?: number;
  /** The width one character is estimated at, in ems. */
  readonly advance?: number;
  /** The distance between two lines of a stack, in ems. */
  readonly line_height?: number;
  /** How far below a line's centre its baseline sits, in ems. */
  readonly baseline_shift?: number;
}

/** What a drawing says: every field optional, over the theme it extends. */
export interface ThemeContent {
  /** The locale form a body is written in. */
  readonly body_form?: 'SHORT' | 'GLYPH';
  /** What a cell's label shows; `AUTO` is the sign's number, or on a wheel the house and the sign's glyph. */
  readonly cell_label?: 'AUTO' | 'SIGN_NUMBER' | 'SIGN_SHORT' | 'SIGN_GLYPH' | 'HOUSE' | 'NOTHING';
  /** Whether the lagna is written first in the cell it stands in. */
  readonly lagna_mark?: boolean;
  /** What is written after a retrograde graha's name, or null for nothing. */
  readonly retrograde_mark?: string | null;
  /** Whether a graha's degree follows its name, on the founded chart. */
  readonly degrees?: boolean;
}

/** What a chart answers by rule, as the SDK writes it. */
export interface RulesReading {
  /** Every rule that held, in the set's order, each by key with what it answered. */
  readonly present: readonly { readonly rule: string; readonly result: Readonly<Record<string, unknown>> }[];
  /** The twelve house readings, when asked for. */
  readonly houses?: readonly Readonly<Record<string, unknown>>[];
  /** The longevity readings, when asked for; a maraka result is a vulnerability, never a date. */
  readonly longevity?: Readonly<Record<string, unknown>>;
  /** An input a rule named that the chart could not have, such as `'points'`. */
  readonly unreadable?: readonly string[];
}

/**
 * One thing to say: a message key and its slots. The slots are the very
 * record `intl.render` takes, so `sdk.intl.render(item.key, item.params)`
 * says it, in whatever locale the context is in.
 */
export interface PlanItem {
  /** The message to say it with. */
  readonly key: string;
  /** Its slots, ready for `intl.render`. */
  readonly params: Readonly<Record<string, unknown>>;
}

/** What a chart has to say, holding no words: the composers asked for. */
export interface Plans {
  /** Where the grahas stand; absent unless `placements` asked for it. */
  readonly placements?: readonly PlanItem[];
  /** What the rules answered; absent unless `readings` asked for it. */
  readonly readings?: readonly PlanItem[];
  /** What each graha weighs; absent unless `strength` asked for it. */
  readonly strength?: readonly PlanItem[];
  /** Who rules each bhava; absent unless `houses` asked for it. */
  readonly houses?: readonly PlanItem[];
  /** Where each graha stands to the degree; absent unless `positions` asked. */
  readonly positions?: readonly PlanItem[];
  /** Which graha looks at which; absent unless `aspects` asked for it. */
  readonly aspects?: readonly PlanItem[];
  /** What each graha is where it stands; absent unless `conditions` asked. */
  readonly conditions?: readonly PlanItem[];
  /** Each graha's chara karakas; absent unless `karakas` asked for them. */
  readonly karakas?: readonly PlanItem[];
  /** Where the two house readings disagree; absent unless `chalit` asked. */
  readonly chalit?: readonly PlanItem[];
  /** What a loaded corpus says of the chart's subjects; absent unless `phala` asked. */
  readonly phala?: readonly PlanItem[];
  /** Each bhava's strength; absent unless `bhavaBala` asked for it. */
  readonly bhavaBala?: readonly PlanItem[];
  /** Each graha's Vimshopaka; absent unless `vimshopaka` asked for it. */
  readonly vimshopaka?: readonly PlanItem[];
  /** The almanac of the day; absent unless `panchanga` asked for it. */
  readonly panchanga?: readonly PlanItem[];
  /** A graha's friendships and avasthas; absent unless `states` asked. */
  readonly states?: readonly PlanItem[];
  /** What a placement says of its dasha; absent unless `dashaPhala` asked. */
  readonly dashaPhala?: readonly PlanItem[];
  /** What the Ashtakavarga says; absent unless `ashtakavarga` asked for it. */
  readonly ashtakavarga?: readonly PlanItem[];
  /** What a corpus says of Saturn's periods; absent unless `sadeSati` asked. */
  readonly sadeSati?: readonly PlanItem[];
  /** What the transits come to, one plan an instant; absent unless `gochar` asked. */
  readonly gochar?: readonly (readonly PlanItem[])[];
}

/** A theme the SDK ships, by its key: dark ink on white, or light on dark. */
export type ShippedTheme = 'LIGHT' | 'DARK';

/**
 * The theme a request writes its drawings as SVG in: a shipped theme's key,
 * or a record naming only what it changes (`03-design/render-svg.md`).
 */
export type Theme =
  | ShippedTheme
  | {
      readonly extends?: ShippedTheme;
      readonly style?: ThemeStyle;
      readonly content?: ThemeContent;
    };

export interface Drawing {
  /**
   * The drawing as SVG, in the request's theme and the context's locale;
   * absent when the request gave no theme.
   */
  readonly svg?: string;
  /** The layout it is drawn in. */
  readonly layout: ChartLayout | LayoutKey;
  /** Which chart: `varga.D1` for the founded chart, or a divisional one. */
  readonly varga: Varga;
  /** The cells, in the layout's order. */
  readonly cells: readonly DrawnCell[];
  /** The lines drawn that hold nothing. */
  readonly frame: readonly Outline[];
  /** Each body at its own degree, on a wheel; empty for a grid. */
  readonly marks: readonly {
    readonly body: string;
    readonly ring: number;
    readonly at: UnitPoint;
    readonly longitudeDeg: number;
  }[];
}

/** One divisional chart of a founded chart. */
export interface DivisionalChart {
  /** Which divisional chart. */
  readonly varga: Varga | 'unknown';
  /** Where it puts the lagna. */
  readonly lagna: DivisionalPlacement;
  /** Where it puts each graha, in the chart's graha order. */
  readonly grahas: readonly {
    /** Which graha. */
    readonly graha: Graha | 'unknown';
    /** Where the divisional chart puts it. */
    readonly at: DivisionalPlacement;
  }[];
}

/** One drishti a graha casts. */
export interface Drishti {
  /** The graha casting it. */
  readonly from: Graha | 'unknown';
  /** The graha it reaches. */
  readonly to: Graha | 'unknown';
  /** Which house from the casting graha's sign the other stands in, counting inclusively from one. */
  readonly houses: number;
  /** How strongly, under the chart's drishti table. */
  readonly strength: Strength | 'unknown';
  /** How near the casting graha stands to a boundary. */
  readonly fromEdge: Boundaries;
  /** How near the graha reached stands to a boundary. */
  readonly toEdge: Boundaries;
}

/** One graha's sphuta drishti on another body, by degree (crux C45). */
export interface SphutaDrishti {
  /** The graha looking: one of the seven. */
  readonly from: Graha | 'unknown';
  /** The body looked at, a node included. */
  readonly to: Graha | 'unknown';
  /** How strongly, in virupas: 0 to 60, Mars's, Jupiter's and Saturn's special aspects adding 15, 30 and 45 over their spans. */
  readonly virupas: number;
}

/** One body's Jaimini rashi drishti. */
export interface RashiDrishti {
  /** The body whose sign looks. */
  readonly graha: Graha | 'unknown';
  /** The three signs its sign aspects, in the zodiac's order from Aries. */
  readonly signs: readonly (Rashi | 'unknown')[];
}

/** One derived point: an upagraha or a special lagna. */
export interface DerivedPoint {
  /** Which point. */
  readonly point: Point | 'unknown';
  /** Its longitude in the chart's zodiac, degrees. */
  readonly longitudeDeg: number;
  /** The sign it stands in. */
  readonly sign: Rashi | 'unknown';
  /** How near it stands to a boundary. */
  readonly boundaries: Boundaries;
}

/** One of the twelve bhavas as the houses service reads it. */
export interface HouseReading {
  /** The bhava, 1 to 12. */
  readonly number: number;
  /** The sign its middle falls in, which under an unequal division is not always the sign it opens in. */
  readonly sign: Rashi | 'unknown';
  /** The graha that rules that sign. */
  readonly lord: Graha | 'unknown';
  /** Which kind of house it is. */
  readonly quadrant: Quadrant | 'unknown';
}

/** What one graha **is**, as opposed to where it is. */
export interface GrahaState {
  /** Which graha. */
  readonly graha: Graha | 'unknown';
  /** The sign it stands in. */
  readonly sign: Rashi | 'unknown';
  /** The whole-sign house it stands in, 1 to 12. */
  readonly house: number;
  /** Its dignity in that sign. */
  readonly dignity: Dignity | 'unknown';
  /** Its relationships to the lord of the sign it stands in. */
  readonly friendship: {
    readonly natural: Relationship | 'unknown';
    readonly temporary: Relationship | 'unknown';
    readonly compound: Relationship | 'unknown';
    /** The lord of its sign, or `null` for a graha that has none. */
    readonly dispositor: Graha | 'unknown' | null;
  };
  /** Whether the Sun burns it, and by how much; a value it may not have is `null`. */
  readonly combustion: {
    readonly burning: Burning | 'unknown';
    readonly fromSunDeg: number | null;
    readonly orbDeg: number | null;
    readonly deepOrbDeg: number | null;
  };
  /** Its age: the baladi avastha. */
  readonly age: AvasthaBaladi | 'unknown';
  /** Its wakefulness: the jagradadi avastha. */
  readonly wakefulness: AvasthaJagradadi | 'unknown';
  /** Its deeptadi avastha, or `null` where the scheme gives none. */
  readonly deeptadi: AvasthaDeeptadi | 'unknown' | null;
  /** The lajjitadi avasthas: those it holds, those ruled out, and those undecided. */
  readonly lajjitadi: {
    readonly holding: readonly AvasthaLajjitadi[];
    readonly ruledOut: readonly AvasthaLajjitadi[];
    readonly undecided: readonly AvasthaLajjitadi[];
  };
  /** The planetary war it is in, or `null`. */
  readonly war: {
    readonly opponent: Graha | 'unknown';
    readonly isWinner: boolean;
    readonly apartDeg: number;
  } | null;
  /**
   * The Sayanadi state and its sub-states (BPHS ch. 45 vv. 30 to 37), or
   * `null` for a body the verses give no number.
   */
  readonly sayanadi: Sayanadi | null;
  /**
   * Whether it stands in one of its sign's two pushkara navamshas
   * (Vidyamadhaviyam ch. 1 v. 18), or `null` for an outer planet.
   */
  readonly pushkaraNavamsha: boolean | null;
  /**
   * Whether it stands on its sign's pushkara bhaga in the table
   * `state.pushkara_bhaga` names, or `null` for a body that table gives no
   * row: Jataka Parijata's gives the Moon's alone.
   */
  readonly pushkaraBhaga: boolean | null;
  /**
   * Whether it stands in its marana karaka sthana (Jataka Parijata ch. 17
   * vv. 34 and 35), or `null` for Ketu and the outer planets, which the
   * verses do not name.
   */
  readonly maranaKarakaSthana: boolean | null;
  /** How near it stands to a boundary. */
  readonly boundaries: Boundaries;
}

/**
 * A graha's Sayanadi state, with its sub-state under a name of each anka.
 *
 * @example
 * // The sub-state under a name whose first syllable's anka is 3.
 * const cheshta = state.sayanadi?.cheshtas[3 - 1];
 */
export interface Sayanadi {
  /** The state, Shayana to Nidra. */
  readonly avastha: AvasthaSayanadi | 'unknown';
  /** The sub-state under a name whose first syllable's anka is 1 to 5, in that order. */
  readonly cheshtas: readonly (AvasthaCheshta | 'unknown')[];
}

/**
 * A batch of founded charts at one place, decoded on first use and only
 * once. The charts in it are views over those bytes rather than copies.
 */
export declare class Charts extends Decoded<DecodedCharts> {
  /**
   * A batch over the bytes the library returned, naming the dasha systems a
   * context registered by the ids it gave them.
   */
  constructor(bytes: Uint8Array, dashaNames?: ReadonlyMap<number, string>);
  /** The full key of a registered dasha system's id, when this batch knows it. */
  dashaName(id: number): string | undefined;
  /** How many charts the batch holds. */
  readonly length: number;
  /** What kind of chart these are. */
  readonly kind: ChartKind | 'unknown';
  /** The place they were all founded at. */
  readonly place: GeoPlace;
  /** How many divisional charts each chart of the batch holds. */
  readonly vargaCount: number;
  /** The drishti table every aspect was read under; empty when none were asked for. */
  readonly drishtiTable: string;
  /**
   * The completion steps the SDK applied, in order, each
   * `name:Implementation`. A positions result spells the same steps as
   * objects; the asymmetry is a recorded open question.
   */
  readonly steps: readonly string[];
  /** The solar model that reckoned the days, as it describes itself. */
  readonly model: string;
  /** Everything that reproduces this result (ADR-0020). */
  readonly provenance: Provenance;
  /** The provenance envelope as the canonical JSON the library stamped: the bytes to store beside the result, byte-identical in every binding. */
  readonly provenanceJson: string;
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
  readonly place: GeoPlace;
  /** The lagna at the instant, in the chart's zodiac, degrees. */
  readonly lagnaDeg: number;
  /** The lagna at the sunrise that opened the day, degrees. */
  readonly dayLagnaDeg: number;
  /** The ayanamsha applied at this instant, degrees; zero if tropical. */
  readonly ayanamshaOffsetDeg: number;
  /**
   * The catalogued ayanamsha this chart was read under, or `null` when
   * none was applied -- a tropical chart -- or the settings defined their
   * own, which `ayanamshaCustom` says. One for the batch, since the frame
   * is the request's.
   */
  readonly ayanamsha: Ayanamsha | 'unknown' | null;
  /** Whether the ayanamsha is one the settings define rather than a catalogued one. */
  readonly ayanamshaCustom: boolean;
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
  readonly day: LocalDay;
  /** Where in its day the moment falls, in the reckonings the settings named. */
  readonly timing: ChartTiming;
  /** The grahas, in the catalogue's order, one object each. */
  readonly grahas: readonly PlacedGraha[];
  /**
   * Uranus, Neptune and Pluto, placed as the grahas are; empty unless
   * `outerPlanets` asked for them.
   *
   * @example
   * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, outerPlanets: true });
   * const uranus = chart.outer.find((at) => at.graha === 'graha.URANUS');
   */
  readonly outer: readonly PlacedGraha[];
  /**
   * The Black Moon Lilith the request's `lilith` named, placed as the
   * grahas are; `null` unless it named one.
   *
   * @example
   * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, lilith: Lilith.Mean });
   * const sign = Math.floor(chart.lilith!.longitudeDeg / 30);
   */
  readonly lilith: LilithPosition | null;
  /** The divisional charts asked for, in the order asked; empty unless `vargas` named some. */
  readonly vargas: readonly DivisionalChart[];
  /** The charts drawn in the layouts asked for, in the order asked; empty unless `drawings` named some. */
  readonly drawings: readonly Drawing[];
  /** What the chart answers by rule; `null` unless `rules` named some. */
  readonly rules: RulesReading | null;
  /** What the chart has to say; `null` unless `interpret` named a composer. */
  readonly plans: Plans | null;
  /** The dashas asked for, in the order asked; empty unless `dashas` named some. */
  readonly dashas: readonly Dasha[];
  /** The Ashtakavarga; `null` unless `ashtakavarga` asked for it. */
  readonly ashtakavarga: Ashtakavarga | null;
  /** The Vimshopaka; `null` unless `vimshopaka` asked for it. */
  readonly vimshopaka: Vimshopaka | null;
  /** The Vaiseshikamsa; `null` unless `vaiseshikamsa` asked for it. */
  readonly vaiseshikamsa: VaiseshikamsaReading | null;
  /** The dasha phala; `null` unless `dashaPhala` asked for it. */
  readonly dashaPhala: DashaPhalaReading | null;
  /** Jaimini's significators; `null` unless `jaimini` asked for them. */
  readonly jaimini: JaiminiReading | null;
  /** The Moon's avakahada; `null` unless `avakahada` asked for it. */
  readonly avakahada: Avakahada | null;
  /** What the instant falls in; `null` unless `moment` asked for it. */
  readonly moment: Moment | null;
  /** The Shadbala; `null` unless `shadbala` asked for it. */
  readonly shadbala: Shadbala | null;
  /** The Bhava bala; `null` unless `bhavaBala` asked for it. */
  readonly bhavaBala: BhavaBala | null;
  /**
   * The drishti the chart's grahas cast; empty unless `aspects` asked. The
   * count differs from chart to chart, because relations depend on where
   * the grahas stand.
   */
  readonly aspects: readonly Drishti[];
  /**
   * Each of the seven grahas' sphuta drishti on every other body, by degree
   * (crux C45), a zero included, 56 a chart; empty unless `aspects` asked.
   */
  readonly sphutaDrishti: readonly SphutaDrishti[];
  /**
   * Each body's Jaimini rashi drishti: the three signs its sign aspects;
   * empty unless `aspects` asked.
   */
  readonly rashiDrishti: readonly RashiDrishti[];
  /**
   * The annual charts' instants, in year order; empty unless `varsha`
   * asked. Fewer than asked for is the answer when the ephemeris ends
   * first, so read the length rather than the number you requested.
   *
   * The place is yours: a return is an instant, and whether the annual
   * chart is cast for the birthplace or for a residence is a choice the
   * schools differ on, so pass the instant to `found` yourself.
   */
  readonly praveshas: readonly Pravesha[];
  /**
   * The transits read against this chart, one reading an instant in the
   * order `gochar.instants` asked; empty unless asked for.
   */
  readonly gochar: readonly GocharReading[];
  /**
   * The transit hit list, sorted by instant, then graha, then kind; empty
   * unless `hits` asked. The sky is searched once for the whole batch.
   */
  readonly hits: readonly Hit[];
  /**
   * Sade Sati and the smaller spells, every period reaching into the
   * window whole; `null` unless `sadeSati` asked. Saturn is searched once
   * for the whole batch.
   */
  readonly sadeSati: SadeSatiReport | null;
  /**
   * The chart read as KP — its cusps and planets to the sub-sub lord, its
   * significators and the ruling planets of its moment; `null` unless
   * `kp` asked (`03-design/kp.md`).
   */
  readonly kp: KpReading | null;
  /**
   * The chart read as a prashna, the chart of the moment a question was
   * asked: the verdict's clauses, whether the matter stays, when, what an
   * unspoken question is about, the Tajika links when a house is asked
   * and the Moon's weaknesses; `null` unless `prashna` asked
   * (`03-design/prashna.md`).
   */
  readonly prashna: Prashna | null;
  /**
   * The chart's remedies: each graha's functional nature, whom to
   * propitiate and why, each one's shānti, the running antardasha's
   * verses under `at`, and the chosen deity; `null` unless `remedies`
   * asked (`03-design/remedies.md`).
   */
  readonly remedies: Remedies | null;
  /**
   * The chart read as Lal Kitab reads it: the teva's reading, the 35-year
   * cycle and the year asked for; `null` unless `lalkitab` asked
   * (`03-design/lalkitab.md`).
   */
  readonly lalkitab: LalKitab | null;
  /**
   * The Sarvatobhadra chakra at each of the record's instants, in its
   * order; `null` unless `sarvatobhadra` asked (`03-design/gochar.md` §8).
   */
  readonly sarvatobhadra: readonly Sarvatobhadra[] | null;
  /**
   * The tithi praveshas of the chart's years, `1` up; fewer than asked for
   * where the ephemeris ends first, `null` unless `tithiPravesha` asked
   * (`03-design/tithi-pravesha.md`).
   */
  readonly tithiPravesha: readonly TithiPravesha[] | null;
  /**
   * The year's twelve month charts, each as its first day, or the month's
   * thirty day charts; empty where the ephemeris ends first, `null` unless
   * `masaPravesha` asked (`03-design/annual-chart.md`).
   */
  readonly masaPravesha: readonly MasaPravesha[] | null;
  /** The chart as a dossier for a language model; `null` unless `dossier` asked (`03-design/dossier.md`). */
  readonly dossier: string | null;
  /**
   * The chart read as a birth time to rectify: the purifier's runs, the
   * conception, the circumstances and the baseline's cascade, each `null`
   * unless asked; the whole `null` unless `rectification` asked
   * (`03-design/rectification.md`).
   */
  readonly rectification: Rectification | null;
  /**
   * The seven planets' essential dignities and the chart's sect; `null`
   * unless `dignities` asked (`03-design/essential-dignities.md`).
   */
  readonly dignities: Dignities | null;
  /**
   * Both halves of Lilly's table, the essential dignities and the
   * accidental fortitudes; `null` unless `fortitudes` asked
   * (`03-design/essential-dignities.md` §Accidental fortitudes).
   */
  readonly fortitudes: Fortitudes | null;
  /**
   * Valens's fourteen lots; `null` unless `lots` asked
   * (`03-design/hellenistic-lots.md`).
   */
  readonly lots: Lots | null;
  /**
   * Lilly's considerations before judgement; `null` unless
   * `considerations` asked (`03-design/hellenistic-considerations.md`).
   */
  readonly considerations: Considerations | null;
  /**
   * Whether a horary matter is brought to pass; `null` unless `perfection`
   * asked (`03-design/hellenistic-perfection.md`).
   */
  readonly perfection: Matter | null;
  /**
   * The birth read through its progressions; `null` unless `progressions`
   * asked (`03-design/western-progressions.md`).
   */
  readonly progressions: Progressions | null;
  /**
   * The Western aspect table; `null` unless `westernAspects` asked
   * (`03-design/western-aspects.md`).
   */
  readonly westernAspects: readonly WesternAspectRow[] | null;
  /**
   * The aspect configurations the chart's planets make; `null` unless
   * `westernConfigurations` asked (`03-design/western-configurations.md`).
   */
  readonly westernConfigurations: readonly WesternConfiguration[] | null;
  /**
   * The Western aspects between this chart and the partner's, closest
   * first; `null` unless `synastry` asked (`03-design/western-synastry.md`).
   */
  readonly synastry: readonly SynastryRow[] | null;
  /**
   * The parallels between this chart and the partner's, closest first;
   * `null` unless `synastry` asked for `parallels`
   * (`03-design/western-declinations.md`).
   */
  readonly synastryParallels: readonly SynastryParallelRow[] | null;
  /**
   * The antiscia between this chart and the partner's, closest first;
   * `null` unless `synastry` asked for `antiscia`
   * (`03-design/western-antiscia.md`).
   */
  readonly synastryAntiscia: readonly AntiscionRow[] | null;
  /**
   * The equal distances between this chart and the partner's, closest
   * first; `null` unless `synastry` asked for `midpoints`
   * (`03-design/western-midpoints.md`).
   */
  readonly synastryMidpoints: readonly SynastryMidpointRow[] | null;
  /**
   * The composite of this chart and the partner's; `null` unless
   * `synastry` asked for `composite` (`03-design/western-composites.md`).
   */
  readonly synastryComposite: Composite | null;
  /**
   * The Davison birth of this chart and the partner; `null` unless
   * `synastry` asked for `davison` (`03-design/western-composites.md`).
   */
  readonly synastryDavison: Required<Partner> | null;
  /**
   * The chart's distances from the equator; `null` unless `parallels`
   * asked (`03-design/western-declinations.md`).
   */
  readonly declinations: Declinations | null;
  /**
   * The parallels among the chart's planets, closest first; `null` unless
   * `parallels` asked.
   */
  readonly parallels: readonly ParallelRow[] | null;
  /**
   * The chart's antiscia: each planet's reflections and the pairs standing
   * in one; `null` unless `antiscia` asked (`03-design/western-antiscia.md`).
   */
  readonly antiscia: Antiscia | null;
  /**
   * The chart's equal distances, closest first; `null` unless `midpoints`
   * asked (`03-design/western-midpoints.md`).
   */
  readonly midpoints: readonly MidpointRow[] | null;
  /**
   * The chart's Western houses: the division's cusps and each planet's
   * house; `null` unless `westernHouses` asked (`03-design/western-houses.md`).
   */
  readonly westernHouses: WesternHouses | null;
  /**
   * The chart's harmonic chart: each point multiplied and the pairs
   * meeting in it; `null` unless `harmonic` asked (`03-design/western-harmonics.md`).
   */
  readonly harmonic: HarmonicChart | null;
  /**
   * The chart matched with the record's partner by the Ashta Koota; `null`
   * unless `matching` asked (`03-design/matching.md`).
   */
  readonly matching: AshtaKoota | null;
  /**
   * The chart matched with the same partner by the ten considerations of
   * *Kalaprakasika* XIII; `null` unless `matching` asked (`03-design/matching.md`).
   */
  readonly porutham: Porutham | null;
  /**
   * The chart's Kuja dosha beside the same partner's (*Manasagari*,
   * jāyābhāva v. 4); `null` unless `matching` asked (`03-design/matching.md`).
   */
  readonly kuja: Kuja | null;
  /**
   * Every marriage dosha the match carries, in the answers' own order,
   * each with whether it is lifted; `null` unless `matching` asked
   * (`03-design/matching.md`).
   */
  readonly marriageDoshas: readonly MarriageDosha[] | null;
  /**
   * The birth chart's own sahams with their strength, in the order
   * `varsha.sahams` named them; empty unless it asked. Needs no place.
   */
  readonly sahams: readonly TajikaSaham[];
  /** The upagrahas and special lagnas; empty unless `points` asked. */
  readonly points: readonly DerivedPoint[];
  /** The twelve bhavas as the houses service reads them; empty unless `houses` asked. */
  readonly bhavas: readonly HouseReading[];
  /** What each graha is, in `grahas` order; empty unless `state` asked. */
  readonly states: readonly GrahaState[];
  /** The twelve bhavas for "which house is it in", first to twelfth. */
  readonly houses: readonly Bhava[];
  /** The twelve bhavas of the chart's chalit. */
  readonly chalit: readonly Bhava[];
  /** The completion steps the SDK applied, in order. */
  readonly steps: readonly string[];
  /** What computed this chart, and under what: the batch's provenance stamped with this chart's own `contentHash`. */
  readonly provenance: Provenance;
}

/** One member of a limb, with its own bounds and the clipped ones. */
export interface Span<T> {
  /** Which member ran. */
  readonly member: T | 'unknown';
  /** When the member itself began and ended, inside the day or not. */
  readonly whole: Interval;
  /** The part inside the day: what an almanac row prints. */
  readonly inside: Interval;
  /**
   * Which of the day's two sunrises the member was running at: `'BOTH'`
   * when it names two days (vriddhi), `'NEITHER'` when it names none
   * (kshaya).
   *
   * ```ts
   * const kshaya = day.tithi.filter((span) => span.sunrises === Sunrises.Neither);
   * ```
   */
  readonly sunrises: Sunrises | 'unknown';
  /**
   * When the member ended, in ghati-pala from the day's sunrise under
   * `day.ghati_reckoning`; a member outlasting the day reads as the day's
   * whole count.
   */
  readonly ends: GhatiPala;
}

/** A count from sunrise in ghatis of sixty palas of sixty vipalas. */
export interface GhatiPala {
  /** Ghatis, 0 to 59 (60 when a civil day outlasts twenty-four hours). */
  readonly ghati: number;
  /** Palas, 0 to 59. */
  readonly pala: number;
  /** Vipalas, 0 to 59. */
  readonly vipala: number;
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

/**
 * A local day, as a chart and an almanac both read it: the civil date,
 * its weekday, the sunrise that opened it, its sunset and the sunrise that
 * closes it, whether it had a sunrise at all, and by which convention.
 */
export interface LocalDay {
  /** The civil date, spelled as `calendar.convert` spells one, so it can be handed back to it. */
  readonly date: CalendarDate;
  /** The weekday, which the sunrise-anchored reckoning keeps from sunrise to sunrise. */
  readonly vara: Vara | 'unknown';
  /** The sunrise that opened the day, or what the polar policy put in its place, as a Julian day (UTC). */
  readonly sunrise: number;
  /** The sunset that closed its daylight, as a Julian day (UTC). */
  readonly sunset: number;
  /** The sunrise that closes it, as a Julian day (UTC). */
  readonly nextSunrise: number;
  /** `null` for a day the Sun rose and set on; what happened instead, for one it did not. */
  readonly polar: PolarDay | null;
  /** The named sunrise convention the day was reckoned by; `null` for a custom altitude. */
  readonly convention: Sunrise | 'unknown' | null;
  /** The custom altitude of the Sun's centre, degrees, when `convention` is `null`; `null` otherwise. */
  readonly customAltitudeDeg: number | null;
  /**
   * The air the horizon was refracted through, resolved at the place, when the settings named one
   * (`{ kind: 'ATMOSPHERIC', which, air }`); `null` for the almanac's fixed 34′ or no refraction.
   */
  readonly air: Air | null;
}

/**
 * An air as it was applied: a part the settings left out is the engines' standard at the place,
 * the ICAO atmosphere's pressure at its height and 15 °C.
 */
export interface Air {
  /** The pressure at the observer, hectopascals. */
  readonly pressureHpa: number;
  /** The temperature at the observer, degrees Celsius. */
  readonly temperatureC: number;
}

/** A day with no sunrise or no sunset: which, and what the policy did about it. */
export interface PolarDay {
  /** Whether the Sun stayed up or stayed down. */
  readonly kind: PolarKind | 'unknown';
  /** The policy that put bounds on the day. */
  readonly policy: PolarDayPolicy | 'unknown';
}

/**
 * Where in its day a chart's moment falls: the ishtakaal and the hora. The
 * same record in every binding.
 */
export interface ChartTiming {
  /** The ishtakaal's ghatis since sunrise, 0 to 59. */
  readonly ghati: number;
  /** Its palas, 0 to 59. */
  readonly pala: number;
  /** Its vipalas, 0 to 59. */
  readonly vipala: number;
  /** How the ghatis were measured. */
  readonly ghatiReckoning: GhatiReckoning | 'unknown';
  /** Which hora of the day holds the instant, 1 to 24. */
  readonly horaNumber: number;
  /** The graha that rules it. */
  readonly horaLord: Graha | 'unknown';
  /** When that hora began, as a Julian day (UTC). */
  readonly horaStart: number;
  /** When it ends, as a Julian day (UTC). */
  readonly horaEnd: number;
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
  /** Whether the day's vara censures it under `panchanga.durmuhurta` (crux C398). */
  readonly durmuhurta: boolean;
}

/**
 * The catalogue's `MoonEvent` members, which this module exports at run
 * time beside the record of the same name: the record's interface would
 * otherwise hide the catalogue's value from a TypeScript consumer, since
 * an explicit export beats `export *`. A value and an interface of one
 * name merge, so both are reachable.
 */
export declare const MoonEvent: typeof import('./catalogue.js').MoonEvent;

/** A moonrise or a moonset. */
export interface MoonEvent {
  /** Which it was. */
  readonly kind: import('./catalogue.js').MoonEvent | 'unknown';
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
    readonly kind: 'VARA_NAKSHATRA' | 'VARA_TITHI_NAKSHATRA';
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
  readonly place: GeoPlace;
  /** The civil calendar the days' dates are read in. */
  readonly calendar: Calendar | 'unknown';
  /** The solar model that reckoned the days, as it describes itself. */
  readonly model: string;
  /** The muhurta search the request asked for over these days, or `null` when it asked for none. */
  readonly muhurta: MuhurtaAnswer | null;
  /** The days the festival rules the request asked for fall on over these days, or `null` when it asked for none. */
  readonly festivals: FestivalAnswer | null;
  /** The lunar years the days fall in, when the request asked for them with `years: true`, or `null`. */
  readonly years: LunarYears | null;
  /** The eclipses of the days with the place's view of each, when the request asked for them with `eclipses: true`, or `null`. */
  readonly eclipses: Eclipses | null;
  /** Each day's Nepal Sambat date, when the request asked for them with `nepalSambat: true`, or `null`. */
  readonly nepalSambat: NepalSambatDates | null;
  /** Each day's sky, when the request asked for it with `sky: true`, or `null` (`03-design/day-sky.md`). */
  readonly sky: DaySkies | null;
  /** Everything that reproduces this result (ADR-0020). */
  readonly provenance: Provenance;
  /** The provenance envelope as the canonical JSON the library stamped: the bytes to store beside the result, byte-identical in every binding. */
  readonly provenanceJson: string;
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
  readonly day: LocalDay;
  /** What the spans are clipped to. */
  readonly window: Interval;
  /** The lunar month, under both conventions. */
  readonly month: Month;
  /** Which half of the year the day falls in. */
  readonly ayana: Ayana | 'unknown';
  /**
   * Which season the day falls in, under `panchanga.ritu`: by default the
   * season of the sidereal solar month the day belongs to, its first day
   * placed by `panchanga.solar_month_start`.
   */
  readonly ritu: Ritu | 'unknown';
  /** The clock the day was read on, in seconds east of UTC: a zone's offset at the day's opening, or the offset stated. */
  readonly utcOffsetSeconds: number;
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
  /**
   * The varjyam: each nakshatra's four ghatis of poison, placed by `panchanga.varjyam` (crux C165). A span's `whole` is the window itself and `inside` the part of it
   * in the day; empty under `NONE`.
   */
  readonly varjyam: readonly Span<Nakshatra>[];
  /**
   * The amrita kalam: each nakshatra's four auspicious ghatis, placed by `panchanga.amrit_kalam` (crux C677). A span's `whole` is the window itself and `inside` the part of it
   * in the day; empty under `NONE`.
   */
  readonly amritKalam: readonly Span<Nakshatra>[];
  /**
   * The lagna table, when the request asked with `lagnas: true`: each sign rising from sunrise to the next sunrise, from the ascendant
   * `panchanga.lagna_table` names (crux C680). A span's `whole` is the sign's rising and `inside` the part of it in the day (C681); empty when not asked for.
   */
  readonly lagnas: readonly Span<Rashi>[];
  /**
   * The nakshatra shool: while the Moon is in one of the four nakshatras that block a direction, that direction
   * (Muhurta Chintamani, Yatra Prakarana v. 10, crux C402); empty on a day the Moon crosses none of them. The
   * vara's own direction is `dishaShool`.
   */
  readonly nakshatraShool: readonly Span<Direction>[];
  /** The chandra vasa: the direction the Moon dwells in, by the sign it stands in (crux C403). */
  readonly chandraVasa: readonly Span<Direction>[];
  /**
   * Whose chandrashtama the day holds (Kalaprakasika, crux C404): each member is a **natal** Moon sign, and its
   * intervals the Moon's own run through the eighth sign from it.
   */
  readonly chandrashtama: readonly Span<Rashi>[];
  /**
   * Where fire dwells for a kamya homa, one span for each tithi the day held, read with the day's vara (Muhurta
   * Chintamani, Nakshatra-prakarana v. 36, crux C748); `PRITHVI`, the earth, is the favourable one.
   */
  readonly agniVasa: readonly Span<AgniVasa>[];
  /**
   * The inauspicious yogas the day's tithis and nakshatras make with its vara (Muhurta Chintamani,
   * Shubhashubha-prakarana vv. 8-9, crux C760): dagdha, visha and hutashana by the tithi, yamaghanta by the
   * nakshatra, each span the limb's own.
   */
  readonly ashubhaYogas: readonly Span<AshubhaYoga>[];
  /**
   * Where Bhadra dwells while the vishti karana holds, by the sign the Moon stands in (Muhurta Chintamani,
   * Shubhashubha-prakarana v. 45, crux C820); only `MARTYA`, the earth, is harmful, and a sign change in Bhadra
   * splits it.
   */
  readonly bhadraVasa: readonly Span<BhadraVasa>[];
  /**
   * Where Shiva dwells, one span for each tithi the day held: a printed convention with no text behind it, chosen by
   * `panchanga.shiva_vasa` and named in the provenance's `applied_conventions` (crux C750).
   */
  readonly shivaVasa: readonly Span<ShivaVasa>[];
  /**
   * The panchaka the day holds, named by the vara it began on (Sunday Roga, Monday Raja, Tuesday Agni, Friday Chora,
   * Saturday Mrityu): a printed convention, chosen by `panchanga.panchaka_name` and named in the provenance's
   * `applied_conventions` (crux C821). At most one; `whole` runs from the panchaka's first instant to the Moon's
   * leaving Revati.
   */
  readonly namedPanchaka: readonly Span<Panchaka>[];
  /**
   * Rahu's direction over the daylight, an eighth of sunrise to sunset each from the east (Jyotirnibandha p. 337,
   * crux C749); none after sunset, and none on a day with no daylight.
   */
  readonly rahuVasa: readonly Span<Direction>[];
  /**
   * The punya kala of a sankranti, when the day's daylight holds one (Dharmasindhu pp. 2-3, crux C752): the sign
   * the Sun entered, `whole` the ghatis `panchanga.punya_kala` gives or the half day a night's entry makes
   * meritorious, `inside` the window; empty on all but some twelve days a year.
   */
  readonly punyaKala: readonly Span<Rashi>[];
  /**
   * The homa ahuti, when the request asked with `homaAhuti: true`: the graha whose mouth an oblation falls in, by
   * the Moon's nakshatra counted from the Sun's in threes (Muhurta Chintamani, Nakshatra-prakarana v. 35, crux
   * C751), one span for each stretch the two stars both held; empty when not asked for.
   */
  readonly homaAhuti: readonly Span<Graha>[];
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
  /** What computed this day, and under what: the batch's provenance stamped with this day's own `contentHash`. */
  readonly provenance: Provenance;
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
  /** A muhurta search over the same days, answered as `Almanac.muhurta`; none by default, which costs nothing. */
  readonly muhurta?: MuhurtaRequest;
  /** Festival rules to fall over the same days, answered as `Almanac.festivals`; none by default, which costs nothing. */
  readonly festivals?: FestivalRequest;
  /** Whether to answer the lunar years the days fall in, as `Almanac.years`; `false` by default, which costs nothing. */
  readonly years?: boolean;
  /** Whether to answer the eclipses of the days and the place's view of each, as `Almanac.eclipses`; `false` by default, which costs nothing. */
  readonly eclipses?: boolean;
  /** Whether to answer each day's Nepal Sambat date, as `Almanac.nepalSambat`; `false` by default, which costs nothing. */
  readonly nepalSambat?: boolean;
  /** Whether to give each day its lagna table, as `AlmanacDay.lagnas`; `false` by default, which costs nothing. */
  readonly lagnas?: boolean;
  /** Whether to give each day its homa ahuti, as `AlmanacDay.homaAhuti`; `false` by default, which costs nothing. */
  readonly homaAhuti?: boolean;
  /** Whether to answer each day's sky, as `Almanac.sky`; `false` by default, which costs nothing. */
  readonly sky?: boolean;
  /** The step of the sky's paths in minutes, 1 to 240, ten when left out. */
  readonly skyStepMinutes?: number;
  /** `false` to leave the days out and answer only what is asked beside them; festivals alone are then founded as only what their rules read, which is what `AlmanacArea.festivals` asks. `true` by default. */
  readonly days?: boolean;
}

/**
 * Festival rules to reckon over an almanac's days
 * (`03-design/festival-rules.md` §7). A catalogue member may be written
 * bare (`'ASHWINA'`) or in full (`'masa.ASHWINA'`).
 */
export interface FestivalRequest {
  /**
   * A pack the SDK ships, by name; or a list whose items each name a pack
   * or spell a rule out, in order: an Ekadashi rule when it has a `vedha`,
   * a sait rule when it has `windows` (`03-design/festival-sait.md`),
   * a following rule when it has an `after`, a dated rule when it has a
   * `civilDate`, a `civilWeekday`, a `sunEnters` or an `easter`, an anniversary when it
   * has an `origin`, else a festival rule. A later rule
   * replaces an earlier one with its key.
   */
  readonly rules:
    | FestivalPack
    | readonly (FestivalPack | FestivalRule | EkadashiRule | FollowingRule | DatedRule | Anniversary | SaitRule)[];
  /**
   * Published sets laid over the rules' answer in order, a later set's
   * entry replacing an earlier one's for the same occurrence
   * (`03-design/published-overrides.md`).
   */
  readonly published?: readonly PublishedSet[];
  /**
   * What the answer keeps, by the rules' `meta`; every rule when left out
   * (`03-design/festival-metadata.md`).
   */
  readonly only?: import('./requests.js').MetaFilter | null;
  /**
   * The days the tabular Hijri calendar is moved toward a sighting, −2 to
   * 2, for the rules kept on its dates: under 1 a day is read as the table
   * reads the next, so a month begins a day earlier. A published set's day
   * still lays over the moved one; none when left out
   * (`03-design/published-overrides.md` §7).
   */
  readonly hijriOffset?: -2 | -1 | 0 | 1 | 2 | null;
}

/** One change a published day was tried under, or the reason it differs. */
export type PublishedCause =
  | { readonly by: 'PLACE'; readonly name: string }
  | { readonly by: 'SKY'; readonly profile: string; readonly settingsHash: string }
  | { readonly by: 'RULE'; readonly pack: string }
  /** No change tried gives the day: the publisher's own choice. */
  | { readonly by: 'PUBLISHER' };

/** Why each published day an answer moved differs from the rule's. */
export interface PublishedReasons {
  readonly reasons: readonly {
    readonly rule: string;
    /** The set that published it. */
    readonly set: string;
    /** The published day. */
    readonly day: CalendarDate;
    /** The rule's own day here. */
    readonly computed: CalendarDate;
    readonly reason: PublishedCause;
    /** Every change tried, in order: the rule's day under it, absent when it gives none within seven days. */
    readonly tried: readonly { readonly change: PublishedCause; readonly day?: CalendarDate; readonly reproduces: boolean }[];
  }[];
  readonly provenance: Provenance;
}

/** What a published entry did to a day; absent on a day no entry touched, which is the rule's. */
export interface Published {
  readonly status: 'PUBLISHED' | 'WITHDRAWN';
  readonly publisher: string;
  /** The set's id. */
  readonly set: string;
  readonly version: number;
  /** Where the publisher says it. */
  readonly source: string;
  /** The rule's own day, when the publisher's differs. */
  readonly computed?: CalendarDate;
  /** A published time. */
  readonly window?: Interval;
}

/** An anniversary's day, as the festival evaluator judged it. */
export interface AnniversaryDay {
  readonly key: string;
  /** Which anniversary: 1 for the first recurrence after the origin, a janma tithi's the age turned; absent for a lunar origin given no moment. */
  readonly count?: number;
  readonly kind: 'EKODDISHTA' | 'PARVANA' | 'JANMA_TITHI' | 'EVENT';
  /** The origin as found: the tithi, the amanta month it names, that lunation's kind and the tithi's half. */
  readonly origin: {
    readonly tithi: Tithi;
    readonly month: Masa;
    readonly kind: 'NIJA' | 'ADHIKA' | 'KSHAYA';
    readonly half: 'FIRST' | 'SECOND';
    readonly nakshatra?: Nakshatra;
    /** The moment the years are counted from, a UTC Julian day. */
    readonly at?: number;
    /** Whether the tithi is a date's sunrise tithi, for want of an hour. */
    readonly assumedSunrise: boolean;
    /** Every tithi the date held when more than one; empty otherwise. */
    readonly candidates: readonly Tithi[];
  };
  /** The clause that chose the month (C666, C667, C669). */
  readonly monthBy: 'NIJA' | 'FIRST_YEAR_ADHIKA' | 'ADHIKA_ORIGIN' | 'KSHAYA_JOINED' | 'SAMSARPA';
  /** The rule that chose the day (`03-design/anniversary-span.md`). */
  readonly dayRule: AnniversaryDayRule;
  readonly observance: FestivalObservance;
}

/** The rule applied to an anniversary's day. */
export interface AnniversaryDayRule {
  /** How the tithi stood: one sunrise, none or two (C910). */
  readonly tithi: 'ORDINARY' | 'KSHAYA' | 'VRIDDHI';
  /** The rite's time it was read at, as a rule spells a window. */
  readonly at: { readonly window: string; readonly [field: string]: unknown };
  /** The clause that chose the day. */
  readonly clause: 'ONE_DAY' | 'BOTH_WHOLE' | 'NEITHER' | 'EQUAL_PARTS' | 'GREATER_SHARE' | 'BIRTH_NAKSHATRA' | 'TWO_MUHURTA_FLOOR';
  /** Where the clause is stated, or the convention's name. */
  readonly source: string;
}

/**
 * A pack of festival rules the SDK ships: *Dharmasindhu*'s; those as
 * Nepal's national panchanga keeps them (a rite of the daylight on the day
 * whose sunrise holds its tithi) with the days it counts from them
 * (`03-design/festival-rules.md` §9.4–9.5); or those with the further days
 * the panchanga prints, each rule fitted to the committee's editions and
 * citing its page (§10).
 */
export type FestivalPack = 'DHARMASINDHU' | 'NEPAL' | 'NEPAL_PANCHANGA' | 'PITRI_PAKSHA' | 'GAUDIYA' | 'INDIA_CENTRAL';

/**
 * An observance kept on a date of a calendar
 * (`03-design/festival-rule-language.md` §3.6): a sankranti on the first
 * of a Bikram Sambat month, a national day; or on the civil day the Sun
 * enters a nakshatra, Bhumiraj on its entry into Ardra; or by a count
 * from Easter Sunday, Good Friday at -2. Exactly one of `civilDate`,
 * `civilWeekday`, `sunEnters` and `easter`.
 */
export type DatedRule = {
  /** Its key in its pack, `'MAKARA_SANKRANTI'`; an observance is named by it. */
  readonly key: string;
  /** Where the rule is stated. */
  readonly source: string;
} & (
  | {
      /** A day of a month, 1 for the first; a negative day counts back from its end, -1 for the last. */
      readonly civilDate: {
        readonly calendar: Calendar;
        readonly month: number;
        readonly day: number;
        /** Which neighbour keeps a skipped Tibetan date; the day before when left out. Tibetan only. */
        readonly skipped?: 'BEFORE' | 'AFTER';
      };
      readonly civilWeekday?: never;
      readonly sunEnters?: never;
      readonly easter?: never;
    }
  | {
      /** The `nth` weekday of a month; a negative `nth` counts back from its end, -1 for the last. */
      readonly civilWeekday: {
        readonly calendar: Calendar;
        readonly month: number;
        readonly weekday: Vara;
        readonly nth: number;
      };
      readonly civilDate?: never;
      readonly sunEnters?: never;
      readonly easter?: never;
    }
  | {
      /**
       * The civil day, local midnight to the next, in which the sidereal Sun
       * enters `nakshatra` in the almanac's own sky, or `days` civil days
       * after it (0 when left out, at most 15).
       */
      readonly sunEnters: { readonly nakshatra: Nakshatra; readonly days?: number };
      readonly civilDate?: never;
      readonly civilWeekday?: never;
      readonly easter?: never;
    }
  | {
      /**
       * The day `days` days from Easter Sunday by the Western (`'GREGORIAN'`)
       * or the Eastern (`'JULIAN'`) computus, before it when negative (0 when
       * left out, at most 70 either way); none in a year before the
       * computus's first, 1583 for the Gregorian.
       */
      readonly easter: { readonly computus: 'GREGORIAN' | 'JULIAN'; readonly days?: number };
      readonly civilDate?: never;
      readonly civilWeekday?: never;
      readonly sunEnters?: never;
    }
);

/** A day a dated rule gave. */
export interface DatedObservance {
  /** The rule's key. */
  readonly rule: string;
  /** The day. */
  readonly day: CalendarDate;
  /** The same day in the rule's own calendar. */
  readonly on: CalendarDate;
  /** What a published entry did to the day. */
  readonly published?: Published;
}

/** The time of a rite. */
export type FestivalWindow =
  | { readonly window: 'SUNRISE' }
  /** The instant of sunset, which begins the evening; Nepal's monthly full-moon fast is judged there (C200). */
  | { readonly window: 'SUNSET' }
  | { readonly window: 'PART'; readonly part: DayFifth }
  | { readonly window: 'PRADOSHA' }
  | { readonly window: 'NISHITHA' }
  /** A fifteenth of the night counted from sunset, 1 to 15; Shivaratri's niśītha is the 8th. */
  | { readonly window: 'NIGHT_MUHURTA'; readonly muhurta: number }
  /** Fifteenths of the daylight counted from sunrise, `from` to `to` inclusive, 1 to 15; purvahna is 1 to 6 (C602). */
  | { readonly window: 'DAY_MUHURTA'; readonly from: number; readonly to: number }
  /** The first moonrise after the day's sunrise; the Sankashta chaturthi's (*Dharmasindhu* p. 9). A day the Moon does not rise on leaves the occurrence unjudged. */
  | { readonly window: 'MOONRISE' };

/** Something a guard asks of the two days. */
export type FestivalPredicate =
  | { readonly is: 'CASE'; readonly case: FestivalCase }
  | { readonly is: 'JOINED'; readonly day: FestivalWhich; readonly nakshatra: Nakshatra; readonly at?: FestivalWindow }
  | { readonly is: 'STANDS'; readonly day: FestivalWhich; readonly nakshatra: Nakshatra; readonly at: FestivalWindow }
  | { readonly is: 'LASTS'; readonly day: FestivalWhich; readonly from: 'SUNRISE' | 'SUNSET'; readonly ghatis: number }
  /** The tithi holds the whole of that day's window, or its instant. */
  | { readonly is: 'WHOLLY'; readonly day: FestivalWhich }
  /** That day's window holds more of the tithi, in time, than the other's; equal times hold for neither (C601). */
  | { readonly is: 'GREATER'; readonly day: FestivalWhich }
  /** That day is this vara, sunrise to sunrise (C618). */
  | { readonly is: 'ON_VARA'; readonly day: FestivalWhich; readonly vara: Vara };

/** What a rule is kept on: a tithi, or a nakshatra in a paksha of the month (the Samavedis' upakarma is Hasta in Bhadrapada's bright half). */
export type FestivalOccurs =
  | {
      /** The tithi; its paksha is the tithi's. */
      readonly tithi: Tithi;
      readonly nakshatra?: never;
      readonly paksha?: never;
      readonly dayOfPaksha?: never;
    }
  | {
      readonly tithi?: never;
      readonly nakshatra?: never;
      readonly paksha?: never;
      /** The same day of both pakshas, 1 to 14: the pradosha fast is the 13th of each. */
      readonly dayOfPaksha: number;
    }
  | {
      readonly tithi?: never;
      /** The nakshatra; judged between its two days as a tithi is, in the month and paksha of the tithi running at its middle. */
      readonly nakshatra: Nakshatra;
      /** The half of the month it must fall in. */
      readonly paksha: Paksha;
      readonly dayOfPaksha?: never;
    };

/** A rule: when an observance falls, and how its day is decided (`03-design/festival-rules.md` §4.1). */
export type FestivalRule = FestivalRuleFields & FestivalOccurs;

/** A festival rule's fields beside what it is kept on. */
export interface FestivalRuleFields {
  /** Its key in its pack, `'JANMASHTAMI'`; an observance is named by it. */
  readonly key: string;
  /** Where the rule is stated. */
  readonly source: string;
  /** The month, under `convention`; every month when left out, for a rite kept on the same tithi of each. */
  readonly month?: Masa;
  /** The convention the month is named in; `'AMANTA'` by default. */
  readonly convention?: LunarMonth;
  /**
   * Which of a doubled month's two holds it (C171): `'NIJA'` (or `false`, the default)
   * the nija month alone, `'ALSO'` (or `true`) both, `'ONLY'` the adhika alone, and
   * `'FIRST'` the adhika when the named month is doubled, the nija otherwise. A rule
   * read back writes `NIJA` and `ALSO` as `false` and `true`.
   */
  readonly inAdhika?: boolean | 'NIJA' | 'ALSO' | 'ONLY' | 'FIRST';
  /** The time of the rite. */
  readonly at: FestivalWindow;
  /** The guards, in order; the first whose predicates all hold decides. */
  readonly decide: readonly { readonly when: readonly FestivalPredicate[]; readonly choose: FestivalChoice }[];
  /** What decides when no guard holds. */
  readonly otherwise: FestivalChoice;
  /** When the fast kept on the day is broken, each reading answered in order (§8.7); none when left out. */
  readonly parana?: readonly VrataParana[];
}

/** Which of the 11th and the 12th hold the sunrise after their own day. */
export type EkadashiExcess = 'ELEVENTH' | 'TWELFTH' | 'BOTH' | 'NEITHER';

/** A mahadvadashi that held, and the fast it set. */
export interface Mahadvadashi {
  /** The kind that set the fast: the first of the rule's kinds that held. */
  readonly kind: MahadvadashiKind;
  /** Every kind that held, in the rule's order. */
  readonly held: readonly MahadvadashiKind[];
  /** The verse that defines `kind`. */
  readonly clause: string;
  /** The joined nakshatra and its whole stay, for Jaya, Vijaya, Jayanti and Papanashini. */
  readonly joined?: { readonly nakshatra: Nakshatra; readonly span: Interval };
}

/** A daylight window a parana is kept in: a fifth of the daylight, or a run of its muhurtas. */
export type ParanaWindow = Extract<FestivalWindow, { readonly window: 'PART' | 'DAY_MUHURTA' }>;

/**
 * A nakshatra a parana is not kept in (C700): after the fast of the 11th `tithi` in its nija
 * amanta `month`, the `which`th of `parts` equal shares of `nakshatra`'s arc, the whole when
 * `portion` is left out. *Dharmasindhu*'s (pp. 41–42, 56–57, 80–81): the first quarter of Anuradha, the
 * middle third of Shravana and the last quarter of Revati, after the bright 11ths of Ashadha,
 * Bhadrapada and Kartika.
 */
export interface NakshatraBar {
  readonly tithi: 'tithi.SHUKLA_EKADASHI' | 'tithi.KRISHNA_EKADASHI';
  readonly month: Masa;
  readonly nakshatra: Nakshatra;
  /** The share: the `which`th of `parts` (1 to 12) equal parts; the whole when left out. */
  readonly portion?: { readonly parts: number; readonly which: number };
}

/** A rule's nakshatra bar met on the parana day, and the Moon's stay in its share. */
export interface ParanaBarred {
  readonly bar: NakshatraBar;
  readonly span: Interval;
  /** Whether the bar was set aside, as the share outlasts the tithi the meal is kept in. */
  readonly setAside: boolean;
}

/** What set an edge of the window a parana was kept in. */
export type ParanaEdge = 'WINDOW' | 'HARI_VASARA' | 'TITHI_START' | 'TITHI_END' | 'NAKSHATRA' | 'JOINED_NAKSHATRA';

/** The window a parana was kept in, and the clause that set each edge. */
export interface ParanaKept {
  /** When the meal may be taken. */
  readonly span: Interval;
  /** The rule's window it lies in. */
  readonly at: ParanaWindow;
  readonly fromBy: ParanaEdge;
  readonly toBy: ParanaEdge;
}

/** When an Ekadashi fast is broken, under its rule's parana. */
export interface Parana {
  /** The day after the fast. */
  readonly day: CalendarDate;
  /** The tithi the meal is kept in: the 12th when it stands past that day's sunrise, and otherwise the tithi at the sunrise. */
  readonly tithi: Tithi;
  /** When the hari vasara ends, a UTC Julian day, if after that day's sunrise; otherwise `null`. */
  readonly hariVasaraEnds: number | null;
  /** The window kept; `null` when none of the rule's leaves time. */
  readonly kept: ParanaKept | null;
  /** The rule's nakshatra bar, when one met the parana day. */
  readonly barred?: ParanaBarred;
}

/** An Ekadashi's fast under one rule: the day, and the facts that gave it. */
export interface EkadashiFast {
  /** The rule's key. */
  readonly rule: string;
  /** The bright or the dark 11th. */
  readonly tithi: Tithi;
  /** Its amanta month. */
  readonly month: Masa;
  /** Whether that month is adhika. */
  readonly adhika: boolean;
  /** The 10th, the 11th and the 12th, whole. */
  readonly tithis: readonly [Interval, Interval, Interval];
  /** The 11th's own day and the day after (C181). */
  readonly days: readonly [CalendarDate, CalendarDate];
  /** Where the 10th pierces the 11th's day, whatever the rule reckons, or `null`. */
  readonly piercedAt: EkadashiVedha | null;
  /** Whether that pierces by the rule's vedha. */
  readonly pierced: boolean;
  readonly excess: EkadashiExcess;
  /** The table's day, which `day` resolves unless a mahadvadashi moved it. */
  readonly choice: FestivalWhich;
  /** The fast: `choice`'s day, or the one a mahadvadashi set, which may be the day after both. */
  readonly day: CalendarDate;
  /** How far the place can move: the seconds of time every instant the fast was decided from (the sunsets that give the 11th its own day, that day's sunrise and, under a Vaishnava vedha, its arunodaya, and the two sunrises after) may move, as another place moves them, before the day can change. */
  readonly marginSeconds: number;
  /** When it is broken, under the rule's parana; absent for a rule without one. */
  readonly parana?: Parana;
  /** The mahadvadashi that set `day`, when the rule keeps them and one held. */
  readonly mahadvadashi?: Mahadvadashi;
  /** What a published entry did to the fast; a moved fast has no `parana` unless the entry gave a window. */
  readonly published?: Published;
}

/** A day's window for the rite, and the fraction of it the tithi held. */
export interface FestivalExtent {
  readonly day: CalendarDate;
  /** The window; an instant's has no length. */
  readonly window: Interval;
  /** 0 to 1; an instant's is 0 or 1. */
  readonly held: number;
}

/** An observance: the day a rule falls on, and why. */
export interface FestivalObservance {
  /** The rule's key. */
  readonly rule: string;
  /** The day. */
  readonly day: CalendarDate;
  /** The tithi's occurrence judged. */
  readonly tithi: Interval;
  /** Its amanta month, as an Ekadashi fast's is: which month's occurrence a rule kept every month decided. */
  readonly month: Masa;
  /** Whether that month is adhika. */
  readonly adhika: boolean;
  readonly case: FestivalCase;
  /** The earlier day's extent and the later's. */
  readonly extents: readonly [FestivalExtent, FestivalExtent];
  /**
   * The guard, by its index in the rule's list, or the rule's `otherwise`;
   * or, for a following rule, the rule it counts from and how many days.
   */
  readonly decidedBy:
    | { readonly by: 'GUARD'; readonly index: number }
    | { readonly by: 'OTHERWISE' }
    | { readonly by: 'AFTER'; readonly rule: string; readonly days: number };
  /** The choice that decided, which `day` resolves; a following rule's is its leader's. */
  readonly choice: FestivalChoice;
  /** How far the place can move: the seconds of time every instant the day was decided from (its sunrises and the windows the rule draws) may move, as another place moves them, before the day can change; 0 for two equal parts. */
  readonly marginSeconds: number;
  /** When the fast kept on the day is broken, under each of the rule's readings in order (`03-design/festival-rules.md` §8.7); absent when the rule has none. */
  readonly parana?: readonly VrataParanaTime[];
  /** What a published entry did to the day; a moved day has no `parana` unless the entry gave a window. */
  readonly published?: Published;
  /** The rite's time on the day, elected by the rule's sait (`03-design/festival-sait.md`); absent for a rule with none. */
  readonly sait?: FestivalSait;
}

/** A festival day's sait: the window it was found in, every candidate there, and the one chosen. */
export interface FestivalSait {
  /** Which of the rule's windows gave it, from 0; the last when none did. */
  readonly window: number;
  /** That window on the day. */
  readonly within: Interval;
  /** Every candidate in it, in order. */
  readonly candidates: readonly FestivalSaitCandidate[];
  /** The chosen candidate's index; absent when the window holds none. */
  readonly chosen?: number;
  /** The chosen candidate's middle, a UTC Julian day: the instant an almanac prints. */
  readonly at?: number;
}

/** A stretch a sait may be in. */
export interface FestivalSaitCandidate {
  /** When. */
  readonly span: Interval;
  /** Whether its navamsa lies wholly inside the free window; always for a rule reading no rising sign. */
  readonly whole: boolean;
  /** The lagna's sign, when the rule reads a rising sign. */
  readonly lagna?: Rashi;
  /** The navamsa's sign, when the rule reads a rising sign. */
  readonly navamsa?: Rashi;
}

/** What set an edge of a vrata's parana. */
export type VrataEdge = 'SUNRISE' | 'TITHI_END' | 'NAKSHATRA_END' | 'YAMAS' | 'WINDOW_END' | 'RITE';

/** When a vrata's fast is broken under one reading. */
export interface VrataParanaTime {
  readonly reading: VrataParana;
  /** The day after the observance. */
  readonly day: CalendarDate;
  /** When the meal may begin, a UTC Julian day. */
  readonly from: number;
  readonly fromBy: VrataEdge;
  /** When it must be taken by, for a reading with a window. */
  readonly to?: number;
  readonly toBy?: VrataEdge;
}

/**
 * The eclipses whose greatest moment falls in an almanac's days
 * (`03-design/eclipses.md`), each with how the almanac's place sees it,
 * frozen to its leaves. Every instant is a UT1 Julian day, UTC to within
 * a second.
 */
export interface Eclipses {
  /** The lunar and solar eclipses, each list in order. */
  readonly value: { readonly lunar: readonly LunarEclipseHere[]; readonly solar: readonly SolarEclipseHere[] };
  /** What computed them, and the hash of `value`. */
  readonly provenance: Provenance;
}

/** A lunar eclipse and the place's view of it. */
export interface LunarEclipseHere {
  /** The eclipse, the same everywhere. */
  readonly eclipse: LunarEclipse;
  /** Its contacts with the Moon's altitude at the place. */
  readonly here: LunarEclipseView;
  /** The vedha, the sutak, under `panchanga.eclipse_vedha`: from when food is given up to the eclipse's end as seen; `null` where the place does not see it. */
  readonly vedha: EclipseSeen | null;
}

/** A solar eclipse and the place's view of it. */
export interface SolarEclipseHere {
  /** The eclipse, the same everywhere. */
  readonly eclipse: SolarEclipse;
  /** The place's own contacts and magnitude, or `null` where the Moon's disc never touches the Sun's from it. */
  readonly here: SolarEclipseView | null;
  /** The vedha, the sutak, as a lunar eclipse's; `null` where the place does not see it. */
  readonly vedha: EclipseSeen | null;
}

/** A lunar eclipse: its kind, gamma, magnitudes and contacts under a rule for the Earth's shadow. */
export interface LunarEclipse {
  /** The greatest eclipse. */
  readonly greatest: number;
  /** Penumbral, partial or total. */
  readonly kind: LunarEclipseKind;
  /** The Moon's centre from the shadow's axis at greatest, Earth radii, signed by north. */
  readonly gamma: number;
  /** The umbral magnitude: negative for a penumbral eclipse, 1 or more for a total one. */
  readonly umbralMagnitude: number;
  /** The penumbral magnitude. */
  readonly penumbralMagnitude: number;
  /** The penumbra's and the umbra's contacts; the umbral ones `null` where the eclipse lacks them. */
  readonly contacts: {
    readonly p1: number;
    readonly u1: number | null;
    readonly u2: number | null;
    readonly u3: number | null;
    readonly u4: number | null;
    readonly p4: number;
  };
  /** The rule the shadow was enlarged by (`panchanga.eclipse_shadow`). */
  readonly shadow: 'DANJON' | 'CHAUVENET';
}

/** A solar eclipse: its kind at greatest, gamma, magnitude and where on the Earth it is greatest. */
export interface SolarEclipse {
  /** The greatest eclipse. */
  readonly greatest: number;
  /** Partial, annular, total, or hybrid (read at greatest). */
  readonly kind: SolarEclipseKind;
  /** The shadow's axis from the Earth's centre at greatest, Earth radii, signed by north. */
  readonly gamma: number;
  /** The magnitude at greatest. */
  readonly magnitude: number;
  /** Where on the Earth it is greatest, degrees, east and north positive. */
  readonly point: { readonly latitude: number; readonly longitude: number };
}

/** One moment of an eclipse at the place: when, and the body's topocentric geometric altitude there in degrees. */
export interface EclipseMoment {
  readonly at: number;
  readonly altitudeDeg: number;
}

/** The stretch of an eclipse its body stands above `day.sunrise`'s horizon. */
export interface EclipseSeen {
  readonly from: number;
  readonly to: number;
}

/** A lunar eclipse at the place: each contact with the Moon's altitude. */
export interface LunarEclipseView {
  readonly p1: EclipseMoment;
  readonly u1: EclipseMoment | null;
  readonly u2: EclipseMoment | null;
  readonly greatest: EclipseMoment;
  readonly u3: EclipseMoment | null;
  readonly u4: EclipseMoment | null;
  readonly p4: EclipseMoment;
  /** When the Moon is up during the eclipse, or `null` where the place does not see it. */
  readonly seen: EclipseSeen | null;
  /** When the Moon is up during the umbral phase, the part the eye sees, or `null` (always for a penumbral eclipse). */
  readonly umbralSeen: EclipseSeen | null;
}

/** A solar eclipse at the place: its own contacts, maximum and magnitude. */
export interface SolarEclipseView {
  /** What the place sees at its maximum: never hybrid. */
  readonly kind: Exclude<SolarEclipseKind, 'solar_eclipse_kind.HYBRID'>;
  /** The fraction of the Sun's diameter covered; the ratio of the diameters in a central phase. */
  readonly magnitude: number;
  /** The fraction of the Sun's disc covered. */
  readonly obscuration: number;
  readonly first: EclipseMoment;
  /** Totality's or the ring's beginning, `null` outside the central path. */
  readonly second: EclipseMoment | null;
  readonly third: EclipseMoment | null;
  readonly fourth: EclipseMoment;
  /** The greatest magnitude. */
  readonly maximum: EclipseMoment;
  /** When the Sun is up during the eclipse, or `null` where the place does not see it. */
  readonly seen: EclipseSeen | null;
}

/**
 * The lunar years an almanac's days fall in
 * (`03-design/calendar-indian-lunisolar.md` §10), in order and abutting,
 * frozen to its leaves.
 */
export interface LunarYears {
  /** The years, each from one Chaitra Shukla Pratipada's sunrise to the next. */
  readonly value: readonly LunarYear[];
  /** What computed them, and the hash of `value`. */
  readonly provenance: Provenance;
}

/** One lunar year: the name it carries, its numbers and bounds, and the Jovian years that ran in it. */
export interface LunarYear {
  /** The name the year carries under `calendars.samvatsara`. */
  readonly samvatsara: Samvatsara;
  /** Which count named it. */
  readonly count: 'BARHASPATYA' | 'BARHASPATYA_RUNNING' | 'CHANDRAMANA';
  /** The Vikrama year. */
  readonly vikrama: number;
  /** The Shaka year, whose number the southern count reads. */
  readonly shaka: number;
  /** The Kali year elapsed: the Shaka year plus 3179. */
  readonly kali: number;
  /** The new moon that opened the year's first Chaitra, a UTC Julian day. */
  readonly opened: number;
  /** The sunrise of Chaitra Shukla Pratipada, where the name is read, a UTC Julian day. */
  readonly began: number;
  /** The next year's first sunrise, which ends this one, a UTC Julian day. */
  readonly ended: number;
  /** The Jovian years running between `began` and `ended`, in order. */
  readonly jovian: readonly JovianYear[];
  /** The Jovian year that began and ended inside this one and so names no year, or `null`. */
  readonly lupta: Samvatsara | null;
}

/**
 * Each day of an almanac's Nepal Sambat date
 * (`03-design/calendar-indian-lunisolar.md` §11), in the days' order,
 * frozen to its leaves. `sdk.calendar.nepalSambatDate` says one.
 */
export interface NepalSambatDates {
  /** One date a day, in the days' order. */
  readonly value: readonly NepalSambatDate[];
  /** The days' own provenance, and the hash of `value`. */
  readonly provenance: Provenance;
}

/**
 * Each day's sky (`03-design/day-sky.md`), in the days' order, frozen to
 * its leaves.
 */
export interface DaySkies {
  /** One record a day. */
  readonly value: readonly DaySky[];
  /** The days' own provenance with the window's reading named, and the hash of `value`. */
  readonly provenance: Provenance;
}

/** The sky of one day at one place. */
export interface DaySky {
  /** The window's start, Julian day, UT1 read as UTC. */
  readonly from: number;
  /** The window's end; an event here is the next window's. */
  readonly to: number;
  /** The step asked, minutes. */
  readonly stepMinutes: number;
  /** The Sun. */
  readonly sun: BodySky;
  /** The Moon. */
  readonly moon: BodySky;
}

/** One body's day. */
export interface BodySky {
  /** How the window stood to the horizon: a polar day holds no crossing. */
  readonly state: 'CROSSES' | 'ALWAYS_ABOVE' | 'ALWAYS_BELOW';
  /** Every step's reading and every exact event, in time order. */
  readonly track: readonly SkyPoint[];
}

/** A point on a track. */
export interface SkyPoint {
  /** The instant, Julian day, UT1 read as UTC. */
  readonly at: number;
  /** A step's reading or an exact event. */
  readonly kind: 'SAMPLE' | 'RISE' | 'TRANSIT' | 'SET';
  /** The airless topocentric altitude of the centre, degrees. */
  readonly altitudeDeg: number;
  /** The altitude with the standard atmosphere's refraction, degrees; the airless altitude where the body stays below the horizon. */
  readonly refractedAltitudeDeg: number;
  /** The topocentric azimuth, degrees in `[0, 360)`, from north through east. */
  readonly azimuthDeg: number;
  /** The Moon's shape at the instant, on the Moon's track only. */
  readonly moon?: MoonShape;
}

/** The Moon's lit shape at an instant, seen from the place. Turn the bright limb from the zenith by `brightLimbDeg - parallacticAngleDeg` to draw it. */
export interface MoonShape {
  /** The fraction of the disc lit, 0 to 1, from the place. */
  readonly illuminatedFraction: number;
  /** The angle at the Moon between the place and the Sun, degrees: 0 at full, 180 at new. */
  readonly phaseAngleDeg: number;
  /** Whether the Moon is growing, the Earth's-centre sign of her elongation, which agrees with the paksha. */
  readonly waxing: boolean;
  /** The position angle of the bright limb's midpoint, degrees in `[0, 360)`, from the disc's north point through the east. */
  readonly brightLimbDeg: number;
  /** The position angle of the zenith, degrees in `(-180, 180]`, from the disc's north point through the east. */
  readonly parallacticAngleDeg: number;
}

/** A day's Nepal Sambat date: the committee's "ने.सं. ११४६ (कछलाथ्व)". */
export interface NepalSambatDate {
  /** The year, which opens at Kachhala's first day: 1146 from 2025-10-22. */
  readonly year: number;
  /** The month, 1 for Kachhala (amanta Kartika) to 12 for Kaula (amanta Ashwina); an adhika month keeps the number of the month it repeats. */
  readonly month: number;
  /** Whether the month is ordinary, intercalary (Anala) or omitted. */
  readonly kind: MonthKind;
  /** The half: `'paksha.SHUKLA'` is thwa and `'paksha.KRISHNA'` ga. */
  readonly paksha: Paksha;
}

/** One Jovian year of the Surya Siddhanta's count (I.55). */
export interface JovianYear {
  /** The year's name. */
  readonly member: Samvatsara;
  /** The signs mean Jupiter had crossed since the Kali age began, from 0. */
  readonly count: number;
  /** When mean Jupiter entered the sign, a UTC Julian day. */
  readonly from: number;
  /** When it entered the next, a UTC Julian day. */
  readonly to: number;
}

/** What a set of festival rules gives over an almanac's days (`03-design/festival-rules.md` §7.3), frozen to its leaves. */
export interface FestivalAnswer {
  /** Each rule's days, in the order of the tithis. */
  readonly observances: readonly FestivalObservance[];
  /** Each Ekadashi rule's fasts, in the order of the tithis. */
  readonly ekadashis: readonly EkadashiFast[];
  /** The occurrences no day could be given to, and why. */
  readonly unjudged: readonly { readonly rule: string; readonly tithi: Interval; readonly why: string }[];
  /** Each dated rule's days, in the order of the days; empty when none was asked. */
  readonly dated: readonly DatedObservance[];
  /** Each anniversary's days, in the order of the tithis; empty when none was asked. */
  readonly anniversaries: readonly AnniversaryDay[];
  /** The days published sets added; empty when none. */
  readonly added: readonly {
    readonly key: string;
    readonly day: CalendarDate;
    readonly names: Readonly<Record<string, string>>;
    readonly published: Published;
  }[];
  /** The published entries in range no occurrence took. */
  readonly unplaced: readonly { readonly set: string; readonly entry: number; readonly rule: string; readonly day: CalendarDate }[];
  /** The published sets laid over the answer, in order, each with the SHA-256 of its canonical JSON. */
  readonly applied: readonly { readonly id: string; readonly version: number; readonly publisher: string; readonly hash: string }[];
  /** What computed it: the widened days among the applied conventions as `festival.days`, and the hash of this value. */
  readonly provenance: Provenance;
}

/** A set of rules the SDK ships. */
export type MuhurtaActivity =
  | 'RAMAN_MARRIAGE'
  | 'BASELINE_MARRIAGE'
  | 'RAMAN_NAMAKARANA'
  | 'RAMAN_ANNAPRASANA'
  | 'RAMAN_UPANAYANA'
  | 'RAMAN_GRIHA_PRAVESHA'
  | 'RAMAN_KARNAVEDHA'
  | 'RAMAN_CHUDAKARANA'
  | 'RAMAN_VIDYARAMBHA'
  | 'RAMAN_YATRA'
  | 'RAMAN_TRADE_PURCHASE'
  | 'RAMAN_SELLING'
  | 'RAMAN_NEW_CLOTHES'
  | 'RAMAN_VEHICLE';

/** A visibility criterion the SDK names. */
export type AstaName = 'SURYA_SIDDHANTA' | 'COMBUSTION_ORB' | 'PTOLEMY';

/** A list's members by grade, and the grade of any it does not name. */
export interface Graded<T> {
  readonly best: readonly T[];
  readonly middling: readonly T[];
  readonly rejected: readonly T[];
  readonly otherwise: MuhurtaGrade;
}

/** What bars a time: every clause of a kind, by its key, or one clause exactly. */
export type MuhurtaBar = MuhurtaClauseKey | MuhurtaClauseKind;

/** A kind of clause: the `clause` tag of each. */
export type MuhurtaClauseKey = MuhurtaClauseKind['clause'];

/** A clause and the interval it held over. */
export type MuhurtaClause = MuhurtaClauseKind & { readonly at: Interval };

/** What one of the baseline engine's weights measured. */
export type BaselineDimension =
  | 'TITHI_QUALITY'
  | 'NAKSHATRA_SUITABILITY'
  | 'WEEKDAY_SUITABILITY'
  | 'RAHU_KAAL'
  | 'YOGA_SHUDDHI'
  | 'KARANA_SHUDDHI'
  | 'PANCHAKA'
  | 'MUHURTA_YOGA'
  | 'PAKSHA_BALA'
  | 'VARA_EVENT'
  | 'TITHI_EVENT'
  | 'TARA_BALA'
  | 'CHANDRA_BALA'
  | 'CHOGHADIYA'
  | 'ABHIJIT'
  | 'LAGNA_LORD'
  | 'LAGNA_PLACEMENT'
  | 'EIGHTH_HOUSE'
  | 'UDAYASTA_SHUDDHI'
  | 'KARTARI'
  | 'DOSHA_BHANGA'
  | 'KARAKA_STRENGTH'
  | 'KARAKA_COMBUST'
  | 'KARAKA_RETROGRADE';

/** A window judged: when, by which clauses, what barred it, and the baseline's score under that ranking. */
export interface MuhurtaWindow {
  readonly at: Interval;
  readonly clauses: readonly MuhurtaClause[];
  /** The bars that struck it; empty when the rite may be held in it. */
  readonly barredBy: readonly MuhurtaBar[];
  /** The baseline engine's score, under the `'BASELINE'` ranking; `null` under the texts'. */
  readonly score: {
    readonly value: number;
    readonly factors: readonly {
      readonly dimension: BaselineDimension;
      readonly weight: number;
      readonly graha: Graha | null;
    }[];
    readonly cappedAt: number | null;
  } | null;
}

/** A muhurta search's answer (`03-design/muhurta-at-the-boundary.md` §4), frozen to its leaves. */
export interface MuhurtaAnswer {
  /** The windows judged, best first under the ranking, at most `most`. */
  readonly windows: readonly MuhurtaWindow[];
  /** The days the season closed, with the blackouts that closed each. */
  readonly closed: readonly { readonly date: CalendarDate; readonly by: readonly BlackoutKind[] }[];
  /** How many days were judged whole. */
  readonly daysJudged: number;
  /** How many of those were cut into windows. */
  readonly daysCut: number;
  /** How many windows fell inside a blackout that did not cover their whole day, and were left out. */
  readonly windowsBlackedOut: number;
  /** The ranking the windows are in. */
  readonly ranking: MuhurtaRanking;
  /** What the rules ask that was not judged. */
  readonly unjudged: readonly MuhurtaUnjudged[];
  /** What computed it and under what: the asta criterion and the zodiac's instant among the applied conventions, and the hash of this value. */
  readonly provenance: Provenance;
}

/** What `Context.almanacDay` needs: one day at one place. */
/** What a bird does in a yama or a sub-period. */
export type PakshiActivity = 'EATING' | 'WALKING' | 'RULING' | 'SLEEPING' | 'DYING';

/** How a native regards a sub-period's owner; `OWN` for its own sub-period (P12). */
export type PakshiRelation = 'FRIEND' | 'ENEMY' | 'NEUTRAL' | 'OWN';

/** A span of time, Julian days (UTC). */
export interface PakshiSpan {
  readonly from: number;
  readonly to: number;
}

/** Days of a native's bird at a place (`03-design/pakshi.md`). */
export interface PakshiRequest {
  /** The first day. */
  readonly from: CalendarDate;
  /** The last day, both ends included; `from` when left out. */
  readonly to?: CalendarDate;
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** The bird itself, or the birth star and paksha that give it under `rule` (P1). */
  readonly native:
    | { readonly bird: PakshiBird }
    | {
        readonly nakshatra: Nakshatra;
        readonly paksha: Paksha;
        /** `BY_PAKSHA` (the dark half reverses the birds, the default) or `SINGLE` (one table). */
        readonly rule?: 'BY_PAKSHA' | 'SINGLE';
      };
  /** What the days are read under, each the texts' default when left out. */
  readonly rules?: {
    /** How a half is cut into yamas (P3). */
    readonly clock?: 'STRETCHED' | 'NAZHIGAI';
    /** How long the sub-periods run (P4). */
    readonly subs?: 'AGASTYA' | 'PULIPPANI';
    /** Whose friends and enemies (P6). */
    readonly relations?: 'AGASTYA' | 'PULIPPANI';
  };
}

/** A native's bird over a range of days. */
export interface PakshiDays {
  /** The days, in order. */
  readonly value: readonly PakshiDay[];
  /** What computed them, and under what; `inputHash` seals the request. */
  readonly provenance: Provenance;
}

/** One day of a native's bird. */
export interface PakshiDay {
  /** The civil day. */
  readonly date: CalendarDate;
  /** Its reading; `null` on a day the Sun does not both rise and set. */
  readonly reading: {
    readonly day: {
      readonly sunrise: number;
      readonly sunset: number;
      readonly nextSunrise: number;
      /** The weekday of its sunrise (P9). */
      readonly vara: Vara;
      /** The paksha at its sunrise (P2). */
      readonly paksha: Paksha;
    };
    readonly bird: PakshiBird;
    /** The bird dead the whole day and night, reported beside the yamas (P10). */
    readonly deathBird: PakshiBird;
    readonly deadToday: boolean;
    /** The birds eating in the first yama of the day and of the night. */
    readonly eaters: readonly PakshiBird[];
    /** The ten yamas, the day's five and then the night's. */
    readonly yamas: readonly {
      readonly half: 'DAY' | 'NIGHT';
      /** Which yama of the half, 1 to 5. */
      readonly yama: number;
      readonly span: PakshiSpan;
      readonly activity: PakshiActivity;
      readonly quality: 'GOOD' | 'MIDDLING' | 'BAD';
      readonly subs: readonly {
        readonly activity: PakshiActivity;
        /** The bird whose main activity it is. */
        readonly owner: PakshiBird;
        /** Its share of the yama, in 144ths. */
        readonly share: number;
        readonly ownerIs: PakshiRelation;
        readonly span: PakshiSpan;
      }[];
    }[];
  } | null;
}

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
  /**
   * The divisional charts to compute, in the order wanted; none by default,
   * so a caller who wants a birth chart does not pay for twenty-one.
   */
  readonly vargas?: readonly Varga[];
  /**
   * The dashas to compute, a system each, in the order wanted; none by
   * default. A system the catalogue names and this build does not compute
   * is refused by its place in the request.
   */
  readonly dashas?: readonly (DashaSystem | DashaKey)[];
  /**
   * The charts to draw, each a layout and which chart to place in it
   * (`Varga.D1` for the founded chart), in the order wanted; none by default.
   */
  readonly drawings?: readonly { readonly layout: ChartLayout | LayoutKey; readonly varga: Varga }[];
  /**
   * The theme to write every drawing as SVG in, read back as each drawing's
   * `svg`; no SVG by default.
   */
  readonly theme?: Theme;
  /**
   * Rules to answer over every chart, read back as each chart's `rules`; the
   * sections they read are computed whether or not they are asked for here.
   * None by default.
   */
  readonly rules?: RuleRequest;
  /**
   * Narrative plans to compose over every chart, read back as each chart's
   * `plans`: keys and slots holding no words, which `intl.render` says in
   * any locale. None by default.
   */
  readonly interpret?: PlanRequest;
  /**
   * The annual charts to answer for every chart, read back as each
   * chart's `praveshas`: the instants the Sun returns to where it stood
   * at birth. None by default (`03-design/annual-chart.md`).
   */
  readonly varsha?: VarshaRequest;
  /**
   * The transits to read against every chart, read back as each chart's
   * `gochar`: the grahas at each instant counted from the natal Moon, their
   * vedha and verdict (`03-design/gochar.md`). None by default.
   */
  readonly gochar?: GocharRequest;
  /**
   * The transit hit list to search against every chart, read back as each
   * chart's `hits`: ingresses, stations and aspects to natal points over a
   * window (`03-design/transit-hit-list.md`). None by default.
   */
  readonly hits?: HitRequest;
  /**
   * Sade Sati and Saturn's smaller spells to find for every chart, read
   * back as each chart's `sadeSati` (`03-design/sade-sati.md`). None by
   * default.
   */
  readonly sadeSati?: SadeSatiRequest;
  /**
   * A KP reading to make of every chart, read back as each chart's `kp`
   * (`03-design/kp.md`). None by default.
   */
  readonly kp?: KpRequest;
  /**
   * The question to read every chart as the chart of its moment, read
   * back as each chart's `prashna` (`03-design/prashna.md`). A prashna
   * weighs the seven by their Shadbala, so it asks for that too. None by
   * default; `{}` reads the texts' own rules with no house asked.
   */
  readonly prashna?: PrashnaRequest;
  /**
   * The remedies to read in every chart, read back as each chart's
   * `remedies` (`03-design/remedies.md`). An `at` asks for the
   * Vimśottarī daśā too. None by default; `{}` reads the texts' own
   * rules with no periods running.
   */
  readonly remedies?: RemedyRequest;
  /**
   * Lal Kitab to read in every chart, read back as each chart's
   * `lalkitab` (`03-design/lalkitab.md`). None by default; `{}` reads the
   * teva and the general cycle with no year.
   */
  readonly lalkitab?: LalKitabRequest;
  /**
   * The Sarvatobhadra chakra to read against every chart at each of its
   * instants, read back as each chart's `sarvatobhadra`
   * (`03-design/gochar.md` §8). None by default.
   */
  readonly sarvatobhadra?: SarvatobhadraRequest;
  /**
   * The tithi praveshas to find for every chart's years, read back as each
   * chart's `tithiPravesha` (`03-design/tithi-pravesha.md`). None by
   * default.
   */
  readonly tithiPravesha?: TithiPraveshaRequest;
  /**
   * One year's month charts, or one month's day charts, to find for every
   * chart, read back as each chart's `masaPravesha`
   * (`03-design/annual-chart.md`). None by default.
   */
  readonly masaPravesha?: MasaPraveshaRequest;
  /**
   * Every chart written as a dossier for a language model, read back as
   * each chart's `dossier` (`03-design/dossier.md`). None by default.
   */
  readonly dossier?: DossierRequest;
  /**
   * A birth time to rectify, read around every chart's instant and read
   * back as each chart's `rectification` (`03-design/rectification.md`).
   * None by default; each member asks for one reading.
   */
  readonly rectification?: RectificationRequest;
  /**
   * The seven planets' essential dignities to read in every chart, read
   * back as each chart's `dignities` (`03-design/essential-dignities.md`).
   * None by default; `{}` is Valens's horizon and Lilly's tables.
   */
  readonly dignities?: DignityRequest;
  /**
   * Both halves of Lilly's table to read in every chart, read back as each
   * chart's `fortitudes`, and its essential half as `dignities` too
   * (`03-design/essential-dignities.md` §Accidental fortitudes). None by
   * default; `{}` is Lilly's throughout. Asking for `dignities` as well is
   * refused: put that record under `fortitudes.dignities`.
   */
  readonly fortitudes?: FortitudeRequest;
  /**
   * Valens's lots to read in every chart, read back as each chart's `lots`
   * (`03-design/hellenistic-lots.md`). None by default; `{}` is Valens's
   * horizon and his II.22 Fortune.
   */
  readonly lots?: LotRequest;
  /**
   * Lilly's considerations before judgement to read in every chart, read
   * back as each chart's `considerations`, from the fortitudes `fortitudes`
   * asks for or Lilly's (`03-design/hellenistic-considerations.md`). None
   * by default; `{}` is Lilly's.
   */
  readonly considerations?: ConsiderationRules;
  /**
   * The horary matter to weigh in every chart, read back as each chart's
   * `perfection`, on the fortitudes `fortitudes` asks for or Lilly's
   * (`03-design/hellenistic-perfection.md`). None by default.
   */
  readonly perfection?: PerfectionRequest;
  /**
   * The progressions to read every chart's birth through, read back as each
   * chart's `progressions`: the progressed chart and the direction at an
   * instant of life, the contacts over a window, or both
   * (`03-design/western-progressions.md`). None by default.
   */
  readonly progressions?: ProgressionsRequest;
  /**
   * The Western aspects to read in every chart, read back as each chart's
   * `westernAspects`: Leo's nine under his orbs when `{}` (C240). None by
   * default.
   */
  readonly westernAspects?: WesternAspectRequest;
  /**
   * The aspect configurations to find in every chart, read back as each
   * chart's `westernConfigurations`. None by default.
   */
  readonly westernConfigurations?: WesternConfigurationRequest;
  /**
   * The partner every chart is read against, read back as each chart's
   * `synastry`. None by default.
   */
  readonly synastry?: SynastryRequest;
  /**
   * The parallels to read in every chart, with its declinations, read back
   * as each chart's `parallels` and `declinations`: Leo's 1° when `{}`.
   * None by default.
   */
  readonly parallels?: ParallelRequest;
  /**
   * Lilly's antiscia (`03-design/western-antiscia.md`), read as each
   * chart's `antiscia`: his moieties when `{}`. None by default.
   */
  readonly antiscia?: AntisciaRequest;
  /**
   * Leo's equal distances (`03-design/western-midpoints.md`), read as each
   * chart's `midpoints`: 0.5° from the axis when `{}`. None by default.
   */
  readonly midpoints?: MidpointRequest;
  /**
   * A chart's Western houses (`03-design/western-houses.md`), read as each
   * chart's `westernHouses`: Placidus when `{}`. None by default.
   */
  readonly westernHouses?: WesternHouseRequest;
  /**
   * A chart's harmonic (`03-design/western-harmonics.md`), read as each
   * chart's `harmonic`: `{ number: 9 }` for the 9th. None by default.
   */
  readonly harmonic?: HarmonicRequest;
  /**
   * A match with a partner's birth (`03-design/matching.md`), read as each
   * chart's `matching`. None by default.
   */
  readonly matching?: MatchingRequest;
  /** Whether to compute the drishti; false by default. */
  readonly aspects?: boolean;
  /** Whether to compute the upagrahas and special lagnas; false by default. */
  readonly points?: boolean;
  /** Whether to read the bhavas through the houses service; false by default. */
  readonly houses?: boolean;
  /** Whether to compute the Ashtakavarga; false by default. */
  readonly ashtakavarga?: boolean;
  /** Whether to compute the Vimshopaka; false by default. */
  readonly vimshopaka?: boolean;
  /** Whether to compute the Vaiseshikamsa; false by default. */
  readonly vaiseshikamsa?: boolean;
  /** Whether to compute the dasha phala; false by default. */
  readonly dashaPhala?: boolean;
  /** Whether to compute Jaimini's significators; false by default. */
  readonly jaimini?: boolean;
  /**
   * Whether to read the Moon's avakahada; false by default. A tropical
   * chart refuses it, named `avakahada`.
   */
  readonly avakahada?: boolean;
  /**
   * Whether to read what the instant falls in, the rising sign's span and
   * the eclipse seen from the place; false by default.
   */
  readonly moment?: boolean;
  /**
   * Whether to place Uranus, Neptune and Pluto beside the nine, in
   * `chart.outer`; false by default. A progression's later charts and a
   * hit list's natal points then reach them too.
   */
  readonly outerPlanets?: boolean;
  /**
   * Which lunar apogee to place as the Black Moon Lilith, in
   * `chart.lilith`: `Lilith.Mean` (ELP2000-82B's mean perigee plus 180°)
   * or `Lilith.Osculating` (the apogee of the Moon's instantaneous
   * ellipse, the "true" Lilith); none by default, which leaves every
   * other number as it was, to the bit.
   */
  readonly lilith?: Exclude<Lilith, 'NONE'>;
  /** Whether to compute the Shadbala; false by default. */
  readonly shadbala?: boolean;
  /** Whether to compute the Bhava bala; false by default. */
  readonly bhavaBala?: boolean;
  /** Whether to compute what each graha is — its dignity, avasthas, combustion and war; false by default. */
  readonly state?: boolean;
}

/** What `Context.foundMany` needs to found a batch at one place. */
export interface ChartBatchRequest extends Omit<ChartRequest, 'instant'> {
  /** The instants, as Julian days (UTC): one chart each. */
  readonly instants: ArrayLike<number>;
}

/**
 * One part of a rendered message: its text, or a markup tag standing in
 * the text.
 *
 * MF2 markup (`{#b}…{/b}`) is how a message says that part of it is a
 * link, a name or emphasis, **without saying what that looks like** —
 * the message stays free of HTML and the renderer decides. A renderer
 * walks the parts, writes the text ones and opens or closes whatever a
 * tag means in its own world.
 */
export type MessagePart =
  | {
      readonly type: 'text';
      /** The text, already formatted and localised. */
      readonly value: string;
    }
  | {
      readonly type: 'markup';
      /** Whether the tag opens, closes, or stands alone. */
      readonly kind: 'open' | 'close' | 'standalone';
      /** The tag's name, as the message wrote it: `b`, `link`, … */
      readonly name: string;
      /** Its options, each already resolved to a string. */
      readonly options: Readonly<Record<string, string>>;
    };

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
  /** Whether any of the text is unreviewed, from a namespace its locale marks draft. */
  readonly isDraft: boolean;
  /**
   * The message in parts, its markup kept: what a rich renderer walks.
   *
   * Joining the `text` parts gives exactly `text`, so a renderer that
   * does not know a tag can ignore it and lose nothing. A message with
   * no markup is one text part holding the whole of `text`.
   */
  readonly parts: readonly MessagePart[];
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
  /** The scale the instants are on; `UT1` by default. */
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
  /**
   * The first Julian day it covers; year 0 by default. An instant outside
   * the span is never asked for: its cells come back `OUT_OF_RANGE` and the
   * rest of the batch is answered.
   */
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
  /**
   * Chart layouts of your own, to draw in beside the shipped ones: each a
   * row as `sdk.chart.layout(key)` answers it, with a key of its own. A row
   * is checked by the rules a shipped one passes, and a key the SDK ships is
   * refused (`03-design/chart-geometry.md` §7f).
   */
  readonly layouts?: readonly LayoutRow[];
  /**
   * Dasha systems of your own, of either kernel, asked for by
   * `dasha_system.<KEY>` in a request's `dashas`. Each names its `kernel` and
   * is checked by the rules a shipped row passes, and a key the catalogue has
   * is refused whichever kernel asks for it.
   */
  readonly dashaSystems?: readonly DashaDefinition[];
  /** Use the SDK's analytic test provider; for examples and tests only. */
  readonly testProvider?: boolean;
  /** An ephemeris of your own, answered in this language. */
  readonly provider?: EphemerisProvider;
  /**
   * Which ephemeris to compute with, or an **ordered chain** of them,
   * tried in order (ADR-0029).
   *
   * A chain is a caller **saying** they will accept the fallback. One
   * entry is one entry: a context asked for an engine and given the
   * built-in without being told is the silence this refuses.
   *
   * `provider` and `ephemeris` each answer the same question, so both
   * together is a `TypeError` rather than one silently winning.
   */
  readonly ephemeris?: EphemerisChoice | readonly EphemerisChoice[];
}

/**
 * An adapter's descriptor: the platform binary its package ships, and
 * that adapter's own configuration.
 *
 * What the configuration means is the adapter's to say and its
 * package's to type; the SDK hands it over as JSON and reads none of it.
 */
export interface PluginEphemeris {
  /** The adapter's platform binary. */
  readonly plugin: string;
  /** That adapter's own options. */
  readonly config?: Record<string, unknown>;
}

/**
 * One entry of an ephemeris chain: one of the SDK's own by name, an
 * adapter's descriptor, or an ephemeris written in JavaScript — the way a
 * browser, which cannot open an adapter's binary, puts an engine behind
 * `SURYA_SIDDHANTA`.
 *
 * `BUILTIN` is the analytic ephemeris the SDK carries, which needs no
 * files, no network and no licence beyond the SDK's own; `TEST` is the
 * test provider, whose positions are **not astronomy**. Those are the
 * **fallback** — in most cases a consumer plugs a real engine.
 */
export type EphemerisChoice = Ephemeris | PluginEphemeris | EphemerisProvider;

/**
 * **Nepal's official days**: the options that open a context on the
 * Nepali committee's reckoning. The tithi, nakshatra and yoga come from
 * the Surya Siddhanta with the committee's bija; the day's sunrise and the
 * outer planets come from `modern`, the engine named after the text.
 * Spread it into a context's options:
 *
 * ```ts
 * const official = new Context({ ...nepalOfficial(teimeris()), locale: 'ne-Deva-NP' });
 * ```
 *
 * `modern` is an adapter's descriptor or an ephemeris written in
 * JavaScript, or a list of them tried in order. With no `modern`, the
 * sunrise is the built-in ephemeris's.
 */
export function nepalOfficial(
  modern?:
    | PluginEphemeris
    | EphemerisProvider
    | readonly (PluginEphemeris | EphemerisProvider)[],
): { readonly profile: string; readonly ephemeris: readonly EphemerisChoice[] };

/** The profile `nepalOfficial` opens: the Nepali committee's. */
export const NEPAL_OFFICIAL_PROFILE: 'nepali-committee';

/**
 * A context: settings resolved from a profile and a patch, a locale, and
 * an ephemeris. One context serves one thread; a worker builds its own.
 */
/** One parameter of an engine's operation, as its manifest describes it. */
export interface EngineParam {
  /** The name, which is the key a caller uses. */
  name: string;
  /**
   * What it is for: `in` and `out`, or finer, `value`, `handle`,
   * `struct_in`, `array_in`, `array_len`, `scalar_out` and the rest — or a word this package has
   * never heard of, kept as the engine spelled it, because an engine
   * that gains a role must still describe itself.
   */
  role: string;
  /** The engine's own spelling of its type. */
  kind?: string;
  /** Whether a caller may leave it out, which crosses as null. */
  optional?: boolean;
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
  /** Whether the call changes the engine's own state, when the engine says. */
  mutatesEngineState?: boolean;
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
  /**
   * The date `span` after `date`, in its own calendar: the years and months
   * first, a day number the month lacks resolved by `overflow`
   * (`CONSTRAIN` when left out), then the days.
   *
   * @example
   * const on = sdk.calendar.add(date(Calendar.BikramSambat, 2081, 2, 32), { months: 1 }); // Ashadh 31
   */
  add(date: CalendarDate, span: DateSpanAsked, overflow?: Overflow): CalendarDate;
  /**
   * The years, months and days from `from` to `to`, counted as an age is:
   * the most whole months whose anniversary of `from` is not past `to`,
   * then the days from it.
   *
   * @example
   * const age = sdk.calendar.between(date(Calendar.BikramSambat, 2050, 2, 32), date(Calendar.BikramSambat, 2082, 2, 31)); // 32 years
   */
  between(from: CalendarDate, to: CalendarDate, overflow?: Overflow): DateSpan;
  /**
   * A Tibetan year's names (`calendars/tibetan.md`): its rabjung cycle and
   * place, the Indian cycle's name, and its element, gender and animal.
   *
   * @example
   * const { animal } = sdk.calendar.tibetanYear(2026); // 'tibetan_animal.HORSE'
   */
  tibetanYear(year: number): TibetanYear;
}

/** What `calendar.add` adds: each part signed, zero when left out. */
export interface DateSpanAsked {
  readonly years?: number;
  readonly months?: number;
  readonly days?: number;
}

/** A Tibetan year's names, as `calendar.tibetanYear` gives them, frozen. */
export interface TibetanYear {
  /** The year, the Gregorian year the Tibetan year begins in. */
  readonly year: number;
  /** The rabjung cycle, the first beginning in 1027: 2026 is in the 17th. */
  readonly rabjung: number;
  /** The year's place in its cycle, 1 to 60. */
  readonly yearInRabjung: number;
  /** The Indian cycle's name for that place, read in order with none expunged. */
  readonly samvatsara: Samvatsara;
  /** The element, held for two years. */
  readonly element: TibetanElement;
  /** Male or female, alternating; the animal decides it too. */
  readonly gender: Gender;
  /** The animal. */
  readonly animal: TibetanAnimal;
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
  /** The zone database this context reads (`03-design/zone-packs.md` §4). */
  zoneDatabase(): ZoneDatabase;
  /**
   * Replaces this context's zone database with a signed pack, for every
   * answer after this one. A release under `atLeast` is refused.
   */
  loadZones(pack: Uint8Array, signature: Uint8Array, key: Uint8Array, atLeast?: string): ZoneDatabase;
}

/** The zone database a context reads. */
export interface ZoneDatabase {
  /** Where it comes from. */
  readonly source: 'EMBEDDED' | 'PACK';
  /** Its release (`2026f`), as every answer's provenance names it. */
  readonly version: string;
  /** How many names it holds, links included. */
  readonly zones: number;
  /** The release compiled into this build: the fallback. */
  readonly bundled: string;
}

/** `sdk.intl` — the locale, its messages and the scripts they are in. */
export declare class IntlArea {
  /** The locale every render resolves from. Set a short tag and the loaded locale it means is chosen, by RFC 4647 lookup: read this to see which. */
  locale: string;
  /** The loaded locale a tag means, by RFC 4647 lookup, changing nothing: `ne` is `ne-Deva-NP`. */
  lookupLocale(tag: string): string;
  /** Renders a message of the current locale with its parameters. */
  render(key: string, params?: Record<string, unknown>): Rendered;
  /** Whether the current locale or its fallbacks have a message. */
  has(key: string): boolean;
  /** Text from one script into another (`deva`, `iast`). */
  transliterate(text: string, from?: string, to?: string): string;
  /** An entity's forms in the current locale or its fallbacks. */
  entity(key: string): EntityForms;
  /** A number or an amount of money in words and in figures, by CLDR's rules. */
  inWords(request: InWordsRequest): InWords;
  /** A time of day as text and parts, in the locale's pattern. */
  time(request: TimeTextRequest): CivilText;
  /** A date as text and parts, in its own calendar and the locale's pattern. */
  date(request: DateTextRequest): CivilText;
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
  /** Packs a frame's fields into the bits a position request carries. */
  pack(frame: Frame): number;
  /** The frame a packed set of bits describes. */
  unpack(bits: number): Frame;
}

/** `sdk.chart` — a chart founded at an instant and a place. */
export declare class ChartArea {
  /**
   * A layout this context can draw in, shipped or registered, as its row: a
   * fresh object to copy, rename and register.
   */
  layout(key: ChartLayout | LayoutKey | string): LayoutRow;
  /** Founds a chart at an instant and a place. */
  found(request: ChartRequest): Chart;
  /**
   * Founds a chart at each of many instants, at one place, in one
   * crossing: the founder shares the settings and the solar model across
   * the batch. A batch of none is an empty result rather than an error.
   */
  foundMany(request: ChartBatchRequest): Charts;
  /**
   * One period of civil days at a place read for each of the twelve signs
   * (`03-design/rashifal.md`); with `baseline`, the baseline engine's score
   * of each sign for that period.
   *
   * @example
   * const week = ctx.chart.rashifal({ first, last, place, utcOffsetSeconds: 20700 }, 'WEEKLY');
   * const leo = week.period.readings.find((r) => r.rashi === 'rashi.LEO');
   * const sealed = week.provenance.inputHash;
   */
  rashifal(request: RashifalRequest, baseline?: BaselinePeriod): RashifalSealed;
  /** Many periods, each read as `rashifal` reads it alone, under one founder. */
  rashifalMany(requests: readonly RashifalRequest[], baseline?: BaselinePeriod): RashifalAnswers;
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
  /**
   * `of` on a zone's clock: each day on the offset of its own noon
   * (`03-design/time-and-timezone.md` §3.4).
   */
  inZone(request: Omit<AlmanacRequest, 'utcOffsetSeconds'> & { readonly zone: ZoneSpec }): Almanac;
  /** The almanac of one day, which is the range of one unwrapped. */
  day(request: AlmanacDayRequest): AlmanacDay;
  /**
   * The days a set of festival rules falls on in a range, at one place
   * (`03-design/festival-rules.md` §7.5), **without the days**: each day
   * founded as only what the rules read, so a year costs a fraction of the
   * year's almanac and answers exactly what `of({ festivals })` does. Each
   * observance's `marginSeconds` says how far the place can move before
   * its day can change.
   */
  festivals(
    request: Omit<AlmanacRequest, 'festivals' | 'muhurta' | 'nepalSambat' | 'lagnas' | 'homaAhuti' | 'sky' | 'skyStepMinutes' | 'days'> & {
      readonly festivals: FestivalRequest;
    },
  ): FestivalAnswer;
  /**
   * A published set read from a signed pack
   * (`03-design/published-overrides.md` §5), ready for a festival
   * request's `published`: `pack` the set's exact bytes, `signature` the
   * publisher's detached Ed25519 signature over them (64 bytes), `key` the
   * publisher's public key (32 bytes), which the app fixes at its own
   * build. A version under `atLeast` is refused, so a replayed pack cannot
   * roll a correction back.
   *
   * @throws {TeistroError} `INVALID_ARG` naming `key`, `signature`, `version` or the set's own field
   */
  publishedFromSigned(
    pack: Uint8Array | ArrayBuffer | readonly number[],
    signature: Uint8Array | ArrayBuffer | readonly number[],
    key: Uint8Array | ArrayBuffer | readonly number[],
    atLeast?: number,
  ): PublishedSet;
  /**
   * A native's bird read over every day of a range under Pancha Pakshi
   * (`03-design/pakshi.md`). A day the Sun does not both rise and set has
   * a `null` reading. The answer is the envelope: the days, and the
   * provenance that sealed the request.
   *
   * @example
   * const days = ctx.almanac.pakshi({
   *   from: date('calendar.GREGORIAN', 1984, 10, 31),
   *   place: { latitude: 13.0827, longitude: 80.2707, altitude: 6 },
   *   utcOffsetSeconds: 19800,
   *   native: { nakshatra: 'nakshatra.UTTARA_ASHADHA', paksha: 'paksha.SHUKLA' },
   * });
   */
  pakshi(request: PakshiRequest): PakshiDays;
  /**
   * **The time a sankalpa recites** at an instant and a place
   * (`03-design/sankalpa.md`): the year, ayana, ritu, month and vara at
   * the sunrise of the day holding the instant, and the limbs and three
   * signs at the instant itself. A rite before dawn is the day before's.
   *
   * @example
   * const { value } = sdk.almanac.sankalpa({
   *   at: 2461055.59375,
   *   place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
   *   utcOffsetSeconds: 20700,
   * });
   */
  sankalpa(request: SankalpaRequest): SankalpaAnswer;
  /**
   * **The period of impurity after a birth or a death** at a place
   * (`03-design/ashauch.md`): from when, until when, and by which clause,
   * counted in sunrise days under the table (the *Dharmasindhu*'s when
   * left out). A period already running goes in `request.during`, each
   * one `ashauchRunning(earlier.value)`.
   *
   * @example
   * const { value } = sdk.almanac.ashauch({
   *   place: { latitude: 27.7172, longitude: 85.324, altitude: 1337 },
   *   utcOffsetSeconds: 20700,
   *   request: { key: 'grandfather', relation: 'SAPINDA', event: { event: 'DEATH', at: 2461051.677083 } },
   * });
   * value.clause.clause; // 'RELATION'
   * value.days; // 10
   */
  ashauch(query: AshauchQuery): AshauchAnswer;
  /**
   * **A death's first year** at a place (`03-design/shraddha.md` §3.3):
   * the *Dharmasindhu*'s sixteen shraddhas, the sapindana and the first
   * abdika, each on its day. Every reading is the *Dharmasindhu*'s when
   * left out; Nepal's, as Dhakal records it, is
   * `{ firstAbdika: 'ELEVENTH_DAY', firstYearAdhika: 'NIJA' }`.
   *
   * @example
   * const { value } = sdk.almanac.firstYear({
   *   place: { latitude: 27.7172, longitude: 85.324, altitude: 1337 },
   *   utcOffsetSeconds: 20700,
   *   request: { key: 'GRANDFATHER', at: 2461333.677083 },
   * });
   * value.rites[0].rite; // 'ADYA_MASIKA'
   */
  firstYear(query: FirstYearQuery): FirstYearAnswer;
  /**
   * **Where this context's reckoning and another's disagree**, and why
   * (`03-design/setup-disagreements.md`): the days on which some limb's
   * member at sunrise differs, with the instant the two members meet
   * under each sky, and the festivals each puts on another day. Each is
   * `null` when not asked.
   *
   * @example
   * const official = new Context(nepalOfficial(teimeris()));
   * const modern = new Context({ profile: 'nepali-drik', ephemeris: teimeris() });
   * const { days } = official.almanac.disagreements(modern, {
   *   from: date('calendar.GREGORIAN', 2025, 11, 1),
   *   to: date('calendar.GREGORIAN', 2025, 11, 30),
   *   place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
   *   utcOffsetSeconds: 20700,
   * });
   */
  disagreements(other: Context, request: DisagreementRequest): Disagreements;
  /**
   * **Why each published day differs from the rule's**
   * (`03-design/published-overrides.md` §4): the request asked again under
   * each of `places`, then `sky`, then each shipped pack's rule of the
   * same key; the first change that gives the published day is the
   * reason, else `PUBLISHER`.
   */
  publishedReasons(request: {
    readonly from: CalendarDate;
    readonly to?: CalendarDate;
    readonly place: AlmanacRequest['place'];
    readonly utcOffsetSeconds: number;
    readonly festivals: FestivalRequest;
    readonly places?: readonly {
      readonly name: string;
      readonly place: AlmanacRequest['place'];
      readonly utcOffsetSeconds: number;
    }[];
    readonly sky?: Context;
  }): PublishedReasons;

  /**
   * A range's observances and Ekadashi fasts as an iCalendar feed (RFC
   * 5545): one whole-day event each, named in this context's locale, its
   * `UID` stable for the rule, the day and the place. CRLF lines.
   *
   * @example
   * const ics = sdk.almanac.festivalCalendar({
   *   from: date(2026, 1, 1),
   *   to: date(2026, 12, 31),
   *   place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
   *   utcOffsetSeconds: 20700,
   *   festivals: { rules: 'NEPAL_PANCHANGA' },
   * });
   */
  festivalCalendar(request: FestivalCalendarRequest): string;
  /**
   * **The N-day digest** (`03-design/digest-format.md`): the days from
   * `from`, 60 unless `days` says, as bytes a process with no engine reads
   * with `bindings/c/digest`, and `decodeDigest` reads here.
   *
   * @example
   * const bytes = sdk.almanac.digest({
   *   from: { year: 2026, month: 1, day: 1 },
   *   place: { latitude: 27.7172, longitude: 85.324 },
   *   utcOffsetSeconds: 20700,
   * });
   */
  digest(request: DigestOptions): Uint8Array;

  /**
   * The next day a rule or a kind of day falls on after an instant at a
   * place, with no year asked; the instant's own civil day counts. Looks
   * `limitDays` ahead, 400 when left out.
   *
   * @example
   * const next = sdk.almanac.next({
   *   after: 2461345.5,
   *   place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
   *   utcOffsetSeconds: 20700,
   *   kind: 'EKADASHI',
   * });
   */
  next(request: NextRequest): NextOccurrence;

  /**
   * Anniversaries kept by the tithi over a span of years, for many people
   * in one call (`03-design/anniversary-span.md`); each day names the rule
   * that chose it in `dayRule`.
   *
   * @example
   * const span = sdk.almanac.anniversaries({
   *   fromYear: 2026, years: 10, latitudeDeg: 27.7172, longitudeDeg: 85.324, utcOffsetSeconds: 20700,
   *   anniversaries: [{ key: 'GRANDFATHER', origin: { by: 'LUNAR', month: 'KARTIKA', tithi: 'SHUKLA_DASHAMI' } }],
   * });
   */
  anniversaries(request: AnniversariesRequest): FestivalAnswer;
  /**
   * **A day in sentences** (`03-design/day-in-sentences.md`): one date's facts
   * as short sentences from templates, every value from the day's answer. A
   * locale without the templates is refused on `locale`.
   *
   * @example
   * const day = sdk.almanac.sentences({
   *   date: { year: 2026, month: 3, day: 18 },
   *   place: { latitude: 27.7172, longitude: 85.324 },
   *   utcOffsetSeconds: 20700,
   * });
   */
  sentences(request: DaySentencesAsked): DaySentences;
  /**
   * Every instant in a range at which something changes, sorted, as one
   * list (`03-design/change-points.md`).
   */
  changes(request: ChangesRequest): ChangePoints;

  /**
   * Every rule of a festival pack with what it is, and the fields no source
   * gives.
   *
   * @example
   * const { rules } = sdk.almanac.festivalPack('INDIA_CENTRAL');
   */
  festivalPack(pack: string): FestivalPackListing;
}

/** A festival pack's rows (`almanac.festivalPack`). */
export interface FestivalPackListing {
  /** The pack's name. */
  readonly pack: string;
  /** Its rows, by key. */
  readonly rules: readonly FestivalPackRow[];
}

/** One rule of a pack. */
export interface FestivalPackRow {
  /** Its key, as its days are named. */
  readonly key: string;
  /** How its day is found: `LUNAR`, `EKADASHI`, `FOLLOWING`, `DATED` or `PUBLISHED`. */
  readonly form: string;
  /** Where its day comes from. */
  readonly source: string;
  /** What it is: `kind`, `tradition`, `communities`, `regions`, `optional`, each left out when unset. */
  readonly meta?: {
    readonly kind?: string;
    readonly tradition?: string;
    readonly communities?: readonly string[];
    readonly regions?: readonly string[];
    readonly optional?: boolean;
  };
  /** The fields of `meta` no source gives. */
  readonly unsourced: readonly string[];
  /** For a published row, what may move its days. */
  readonly subjectTo?: string;
  /** For a published row, the last year with a day. */
  readonly lastYear?: number;
}

/** What `almanac.sentences` says, where and in which locale. */
export interface DaySentencesAsked {
  /** The day. */
  readonly date: { readonly year: number; readonly month: number; readonly day: number };
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** The date's calendar; Gregorian when left out. */
  readonly calendar?: Calendar;
  /** The locale; the context's when left out. */
  readonly locale?: string;
}

/** A day said in a locale, frozen. */
export interface DaySentences {
  /** The locale the sentences are in. */
  readonly locale: string;
  /** Whether its templates still wait for a native reviewer (Q42). */
  readonly draft: boolean;
  /** The sentences, in the order a printed day runs. */
  readonly sentences: readonly DaySentence[];
  /** The facts a polar day holds elsewhere and so does not say here. */
  readonly withheld: readonly string[];
}

/** One sentence of `almanac.sentences`. */
export interface DaySentence {
  /** The template, as `sdk.day.tithi`. */
  readonly key: string;
  /** Its slots, as `intl.render` takes them. */
  readonly params: Readonly<Record<string, unknown>>;
  /** The sentence. */
  readonly text: string;
}

/** What `almanac.next` looks for, and where and how far. */
export interface NextRequest {
  /** The instant, a UTC Julian day. */
  readonly after: number;
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** The calendar the answer's days are written in; Gregorian when left out. */
  readonly calendar?: Calendar;
  /** A kind of day; or `rule`, not both. */
  readonly kind?: 'EKADASHI' | 'PURNIMA' | 'AMAVASYA' | 'SANKRANTI';
  /** A key of `festivals`; or `kind`, not both. */
  readonly rule?: string;
  /** The festivals a `rule` is looked for in. */
  readonly festivals?: FestivalRequest;
  /** How many days ahead to look, 1 to 3660; 400 when left out. */
  readonly limitDays?: number;
}

/** The next day `almanac.next` found, frozen. */
export interface ChangePoint {
  /** The instant, as a UTC Julian day. */
  readonly at: number;
  /** What changes. */
  readonly kind: ChangeKind;
  /** The catalogue key of the member that ended. */
  readonly from: string;
  /** The catalogue key of the member that began. */
  readonly to: string;
}

/** A range's change points, sorted, with the window they were read over. */
export interface ChangePoints {
  /** Every point at or after the window's start and before its end. */
  readonly points: readonly ChangePoint[];
  /** The instants the range covers. */
  readonly window: { readonly from: number; readonly to: number };
  /** What computed it, and the clock it was asked on. */
  readonly provenance: Provenance;
}

/** The next day `AlmanacArea.next` found. */
export interface NextOccurrence {
  /** The day. */
  readonly day: CalendarDate;
  /** The keys of the rules that fall on it. */
  readonly keys: readonly string[];
  /** The festival reckoning's entries for those keys on that day. */
  readonly festivals: FestivalAnswer;
  /** The first day searched: the civil day holding the instant. */
  readonly from: CalendarDate;
  /** The look-ahead limit, in days. */
  readonly limitDays: number;
  /** The days reckoned until it was found. */
  readonly searchedDays: number;
  /** What computed it. */
  readonly provenance: Provenance;
}

/** A feed of a range's observances, as `almanac.festivalCalendar` writes it. */
export interface FestivalCalendarRequest {
  /** The first day. */
  readonly from: CalendarDate;
  /** The last day, both ends included; `from` when left out. */
  readonly to?: CalendarDate;
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** The festivals. */
  readonly festivals: FestivalRequest;
  /** Every event's `DTSTAMP`, in seconds since 1970; now when left out. */
  readonly stampUnixSeconds?: number;
  /** The calendar's name; "Festivals" when left out. */
  readonly name?: string;
  /** The `PRODID`; the SDK's own when left out. */
  readonly product?: string;
  /** How often a subscriber fetches it again; a day when left out, none when zero. */
  readonly refreshMinutes?: number;
  /** The calendar's own categories. */
  readonly categories?: readonly string[];
  /** The calendar's colour, a CSS3 name. */
  readonly color?: string;
  /** The alarm on every event no hint or event of its own gives another. */
  readonly alarm?: FeedAlarm;
  /** Which events the feed keeps: those any selector selects. */
  readonly only?: readonly FeedSelector[];
  /** Categories, a colour and an alarm by what an event is. */
  readonly hints?: readonly FeedHint[];
  /** The caller's own events, merged in. */
  readonly events?: readonly SuppliedEvent[];
}

/** A tithi's number in the lunar month, 1 for Shukla Pratipada to 30 for Amavasya. */
export function tithiMonthNumber(tithi: Tithi): number;

/** A range of days, as `almanac.digest` writes them. */
export interface DigestOptions {
  /** The first day, and the calendar the digest's dates are in. */
  readonly from: CalendarDate;
  /** How many days, 1 to 366; 60 when left out. */
  readonly days?: number;
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive; or `zone`, not both. */
  readonly utcOffsetSeconds?: number;
  /** The zone, each day on its own offset; or `utcOffsetSeconds`, not both. */
  readonly zone?: ZoneSpec;
  /** The festivals to list; none when left out. */
  readonly festivals?: FestivalRequest;
}

/** An instant and a place, as `almanac.sankalpa` reads them. */
export interface SankalpaRequest {
  /** The instant of the rite, a UTC Julian day. */
  readonly at: number;
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** How the ayana turns: at Makara and Karka sankranti, or at the solstices; `'SIDEREAL'` when left out. */
  readonly ayana?: 'SIDEREAL' | 'TROPICAL';
}

/** What `almanac.sankalpa` answers, frozen to its leaves. */
export interface SankalpaAnswer {
  /** The time recited. */
  readonly value: Sankalpa;
  /** What computed it, and the hash of `value`. */
  readonly provenance: Provenance;
}

/** The time a sankalpa recites (`03-design/sankalpa.md`). */
export interface Sankalpa {
  /** The instant of the rite, a UTC Julian day. */
  readonly at: number;
  /** The civil day whose sunrise the instant follows. */
  readonly date: CalendarDate;
  /** That day's weekday, which runs from its sunrise. */
  readonly vara: Vara;
  /** That day's sunrise, a UTC Julian day. */
  readonly sunrise: number;
  /** The next day's sunrise, where the day ends, a UTC Julian day. */
  readonly nextSunrise: number;
  /** The lunar year the day's sunrise falls in. */
  readonly year: LunarYear;
  /** The half of the year at the day's sunrise. */
  readonly ayana: Ayana;
  /** How `ayana` was reckoned: the request's. */
  readonly ayanaReckoning: 'SIDEREAL' | 'TROPICAL';
  /** The season at the day's sunrise. */
  readonly ritu: Ritu;
  /** How `ritu` was reckoned: `panchanga.ritu`. */
  readonly rituReckoning: 'SOLAR' | 'LUNAR' | 'TROPICAL';
  /** The lunar month at the day's sunrise, under both conventions. */
  readonly month: Month;
  /** The fortnight of the tithi at the rite. */
  readonly paksha: Paksha;
  /** The tithi at the rite. */
  readonly tithi: Tithi;
  /** The nakshatra at the rite. */
  readonly nakshatra: Nakshatra;
  /** The yoga at the rite. */
  readonly yoga: Yoga;
  /** The karana at the rite. */
  readonly karana: Karana;
  /** The Sun's sidereal sign at the rite. */
  readonly sun: Rashi;
  /** The Moon's sidereal sign at the rite. */
  readonly moon: Rashi;
  /** Jupiter's sidereal sign at the rite. */
  readonly jupiter: Rashi;
}

/** A place, a clock, a table and a request, as `almanac.ashauch` reads them. */
export interface AshauchQuery {
  /** Where, in degrees and metres: the place whose sunrises count the days. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** The tradition's table, every row data; the *Dharmasindhu*'s when left out. */
  readonly table?: AshauchTable;
  /** A table the SDK ships, by its name; never beside `table`. */
  readonly tradition?: AshauchTradition;
  /** The event, the observer and the periods already running. */
  readonly request: AshauchRequest;
}

/** What `almanac.ashauch` answers, frozen to its leaves. */
export interface AshauchAnswer {
  /** The span of impurity. */
  readonly value: Ashauch;
  /** What computed it, and the hash of `value`. */
  readonly provenance: Provenance;
}

/** The span of impurity, and the clause that set it (`03-design/ashauch.md`). */
export interface Ashauch {
  /** The request's key. */
  readonly key: string;
  /** A birth's or a death's. */
  readonly kind: AshauchKind;
  /** The request's relation. */
  readonly relation: AshauchRelation;
  /** The deceased's sex, or the child's, as the request gave it. */
  readonly sex?: AshauchSex;
  /** When it began: the event, the cremation, or the hearing, a UTC Julian day. */
  readonly start: number;
  /** The day it counts from. */
  readonly dayOne: CalendarDate;
  /** Why that day. */
  readonly dayOneBy: AshauchDayOneBy;
  /** When it ends: a sunrise, a nightfall, or `start` itself for a bath, a UTC Julian day. */
  readonly end: number;
  /** The last day it touches, for a calendar to shade; none for a bath. */
  readonly lastDay?: CalendarDate;
  /** The length its clause named. */
  readonly length: AshauchLength;
  /** The days, when the length is a count. */
  readonly days?: number;
  /** The clause. */
  readonly clause: AshauchClause;
  /** What an overlap did, when it did. */
  readonly overlap?: AshauchOverlapped;
  /** When the mother of a birth may take part in rites again, a UTC Julian day, under a table that reports it. */
  readonly ritesFrom?: number;
  /** The table's name. */
  readonly table: string;
}

/** Why the first day is the day it is, by its `by`. */
export type AshauchDayOneBy =
  /** The instant fell in the day's daylight. */
  | { readonly by: 'DAYLIGHT' }
  /** It fell in the night, and the rule gave the day. */
  | { readonly by: 'NIGHT'; readonly rule: NightRule };

/** The clause that set the span, by its `clause`; `row` indexes the table's rows. */
export type AshauchClause =
  /** A relation's row of the table. */
  | { readonly clause: 'RELATION'; readonly row: number; readonly said: string }
  /** An age band's row. */
  | { readonly clause: 'BAND'; readonly row: number; readonly said: string }
  /** A loss's row. */
  | { readonly clause: 'LOSS'; readonly row: number; readonly said: string }
  /** A child dying inside the birth period: the birth period runs to its own end. */
  | { readonly clause: 'BIRTH_PERIOD_RUNS' }
  /** Heard inside the period: the days left. */
  | { readonly clause: 'REMAINDER' }
  /** Heard past the full period: the late table's row, within `within_days` of the event. */
  | { readonly clause: 'LATE'; readonly within_days: number; readonly abroad: boolean }
  /** Heard past the late table: a bath. */
  | { readonly clause: 'LATE_BATH' }
  /** Heard past the period, by the relation row's own late length. */
  | { readonly clause: 'ROW_HEARD_LATE'; readonly row: number; readonly said: string }
  /** A shorter period heard past its end: a bath. */
  | { readonly clause: 'SHORT_HEARD_LATE' }
  /** A birth heard past its period: none, the father a bath. */
  | { readonly clause: 'BIRTH_HEARD_LATE' };

/** What an overlap did, by its `overlap`; `earlier` the earlier answer's key. */
export type AshauchOverlapped =
  /** It ends with the earlier period, which is no shorter. */
  | { readonly overlap: 'ENDS_WITH_EARLIER'; readonly earlier: string }
  /** Known in the earlier equal period's last night: its end and `days` more. */
  | { readonly overlap: 'LAST_NIGHT'; readonly earlier: string; readonly days: number }
  /** A mother's death in the father's period: the father's end and a pakṣiṇī. */
  | { readonly overlap: 'MOTHER_AFTER_FATHER'; readonly earlier: string };

/** A place, a clock and a request, as `almanac.firstYear` reads them. */
export interface FirstYearQuery {
  /** Where, in degrees and metres: the place whose sunrises count the days. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** The death and the readings. */
  readonly request: FirstYearRequest;
}

/** Which rite of the first year. */
export type Rite =
  | 'ADYA_MASIKA'
  | 'UNA_MASIKA'
  | 'MASIKA'
  | 'TRAIPAKSHIKA'
  | 'UNA_SHANMASIKA'
  | 'UNA_ABDIKA'
  | 'ABDIKA'
  | 'SAPINDANA';

/** How a rite is performed: for the one dead before the sapindana, for three generations after it. */
export type RiteForm = 'EKODDISHTA' | 'PARVANA';

/** What DS bars an *ūna* rite's day for, found at its sunrise. */
export type UnaBar = 'NANDA' | 'FRIDAY' | 'CHATURDASHI' | 'KRITTIKA';

/** What `almanac.firstYear` answers, frozen to its leaves. */
export interface FirstYearAnswer {
  /** The schedule. */
  readonly value: FirstYearSchedule;
  /** What computed it, and the hash of `value`. */
  readonly provenance: Provenance;
}

/** A death's first year (`03-design/shraddha.md` §3.3). */
export interface FirstYearSchedule {
  /** The request's key. */
  readonly key: string;
  /** The death's lunar date, as an anniversary's origin is found. */
  readonly origin: AnniversaryDay['origin'];
  /** The first day of the count, as the ashauch's. */
  readonly dayOne: CalendarDate;
  /** Why it is that day. */
  readonly dayOneBy: AshauchDayOneBy;
  /** The eleventh day. */
  readonly eleventhDay: CalendarDate;
  /** The sapindana's day. */
  readonly sapindana: CalendarDate;
  /** Every rite, in the order of their own days, then of DS's sixteen. */
  readonly rites: readonly FirstYearRite[];
}

/** One rite of the first year. */
export interface FirstYearRite {
  /** Which rite. */
  readonly rite: Rite;
  /** Its place in DS's sixteen, 1 to 16; a doubled month's two monthlies share it. None for the abdika and the sapindana. */
  readonly place?: number;
  /** The month a monthly closes, 1 to 12; none for every other rite. */
  readonly month?: number;
  /** Which yearly shraddha, 1 for the first; none for every other rite. */
  readonly year?: number;
  /** Whether it is kept in an adhika lunation: the first of a doubled month's two monthlies. */
  readonly adhika: boolean;
  /** Its own day. */
  readonly day: CalendarDate;
  /** How it is performed on that day; none for the sapindana. */
  readonly form?: RiteForm;
  /** The sapindana's day, when the rite is first done there by the ekoddishta and again on its own day by the parvana. */
  readonly drawnTo?: CalendarDate;
  /** What DS bars an *ūna* rite's day for; absent for every other rite. */
  readonly bars?: readonly UnaBar[];
  /** The festival evaluator's judgement of a day kept on a tithi; none for one counted in days. */
  readonly observance?: FestivalObservance;
  /** Where the rule is stated. */
  readonly clause: string;
}

/** Two reckonings of one range at one place, as `almanac.disagreements` compares them. */
export interface DisagreementRequest {
  /** The first day. */
  readonly from: CalendarDate;
  /** The last day, both ends included; `from` when left out. */
  readonly to?: CalendarDate;
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /** Whether the days are compared; true when left out. */
  readonly days?: boolean;
  /** The festivals compared, when asked. */
  readonly festivals?: FestivalRequest;
}

/** The instant one reckoning's member gives way to the other's, under one sky. */
export interface LimbChange {
  /** The instant, a Julian day (UT). */
  readonly at: number;
  /** Minutes from that sky's sunrise to it, negative before sunrise. */
  readonly minutesFromSunrise: number;
}

/** One limb of one day as one reckoning names it at its sunrise. */
export interface ReckonedLimb<T> {
  /** The member standing at this reckoning's sunrise. */
  readonly member: T;
  /** That member's whole span under this reckoning's sky. */
  readonly span: Interval;
  /** This reckoning's sunrise, a Julian day (UT). */
  readonly sunrise: number;
  /** Where the two members meet under this sky, or `null` when they are not neighbours. */
  readonly change: LimbChange | null;
}

/** One limb both reckonings name differently at sunrise. */
export interface LimbDisagreement<T> {
  /** As this context reckons it: the day the app shows. */
  readonly official: ReckonedLimb<T>;
  /** As the context compared with reckons it. */
  readonly other: ReckonedLimb<T>;
}

/** A day on which some limb's member at sunrise differs; a limb both name alike is `null`. */
export interface DayDisagreement {
  readonly date: CalendarDate;
  readonly tithi: LimbDisagreement<Tithi> | null;
  readonly nakshatra: LimbDisagreement<Nakshatra> | null;
  readonly yoga: LimbDisagreement<Yoga> | null;
  readonly karana: LimbDisagreement<Karana> | null;
}

/** One rule's occurrence the two reckonings give differently, or one alone gives. */
export interface FestivalDisagreement {
  /** The rule's key. */
  readonly rule: string;
  /** Its amanta month as this context names it, or as the other does when only it gives one. */
  readonly month: Masa;
  /** Whether that month is adhika, the same way. */
  readonly adhika: boolean;
  /** This context's observance, with each day's extent of the rite's window, which is the reason. */
  readonly official: FestivalObservance | null;
  /** The other's. */
  readonly other: FestivalObservance | null;
}

/** One Ekadashi the two reckonings fast on differently, or one alone fasts. */
export interface EkadashiDisagreement {
  readonly rule: string;
  readonly tithi: Tithi;
  readonly month: Masa;
  readonly adhika: boolean;
  /** This context's fast, with the vedha and excess that decided. */
  readonly official: EkadashiFast | null;
  readonly other: EkadashiFast | null;
}

/** What `almanac.disagreements` answers, frozen to its leaves. */
export interface Disagreements {
  /** The days, when compared. */
  readonly days: {
    /** The profile the other context resolved. */
    readonly otherProfile: string;
    /** The other context's settings hash, so a stored comparison is known stale when either side changes. */
    readonly otherSettingsHash: string;
    /** How many days were compared. */
    readonly compared: number;
    /** The days that differ, in order. */
    readonly days: readonly DayDisagreement[];
    /** This context's, which sealed the record. */
    readonly provenance: Provenance;
  } | null;
  /** The festivals, when asked. */
  readonly festivals: {
    readonly otherProfile: string;
    readonly otherSettingsHash: string;
    /** How many observances and fasts this context gave. */
    readonly compared: number;
    readonly observances: readonly FestivalDisagreement[];
    readonly ekadashis: readonly EkadashiDisagreement[];
    readonly provenance: Provenance;
  } | null;
}

/**
 * `sdk.matching` — what matches without a chart: two names, star to star.
 * A match of two births is asked of the charts, through `matching`
 * beside a chart request.
 */
export declare class MatchingArea {
  /**
   * Two names matched star to star (naam milan). A name is read in
   * Devanagari, or in IAST when `rules.name.latin` is `'IAST'`; a refusal
   * is named, `naam.groom.abhijit` and the like.
   *
   * @example
   * const read = ctx.matching.naam('सीता', 'राम');
   * console.log(read.bride.nakshatra, read.varga.relation, read.ashta.total);
   */
  naam(bride: string, groom: string, rules?: NaamRules): NaamMilan;
}

/**
 * `sdk.numerology` — a name and a birth date under Balliett's letter cycle
 * and Cheiro's Chaldean table. It reads no sky.
 */
export declare class NumerologyArea {
  /**
   * Everything numerology says of a name and a birth date. A character
   * outside A to Z is refused as `numerology.name` unless
   * `rules.nonLatin` is `'SKIP'`; a date the calendar does not have, as
   * `numerology.date`. `rules` is the record, or `'BASELINE'` for every
   * reading the baseline engine takes, under which the answer also
   * carries the baseline's own numbers.
   *
   * @example
   * const read = ctx.numerology.profile('Henry Elder', { year: 1872, month: 1, day: 17 });
   * console.log(read.pythagoreanName.reduction.number, read.chaldeanBirth.birth.number);
   * const baseline = ctx.numerology.profile('Henry Elder', { year: 1872, month: 1, day: 17 }, 'BASELINE');
   * console.log(baseline.baseline);
   */
  profile(name: string, date: BirthDate, rules?: NumerologyRequestRules): NumerologyProfile;
}

/**
 * `sdk.naming` — the naming of a child (nāmakaraṇa): the syllables a
 * pada, a star or a sign names a child by, names read and checked, the
 * names a birth gives, and a consumer's own name book. It reads no sky,
 * and the SDK ships no names. Every refusal is named under `naming`.
 */
export declare class NamingArea {
  /**
   * The syllables a pada, a star or a sign names a child by, in cakra order.
   *
   * @example
   * const makara = ctx.naming.syllables({ of: 'RASHI', rashi: 'CAPRICORN' });
   * console.log(makara.length); // 13: nine padas and Abhijit's four
   */
  syllables(of: SyllablesOf, rules?: NamingRules): readonly PadaSyllable[];
  /**
   * Names read into the cakra, each on its own; a name refused is its row's
   * `refused`, named `naming.names[i]`.
   *
   * @example
   * const [ram] = ctx.naming.read(['राम']);
   * console.log(ram.reading?.readings[0]?.clauses.syllables); // 2
   */
  read(names: readonly string[], rules?: NamingRules): readonly NameRead[];
  /**
   * Names checked against a birth's pada, each on its own.
   *
   * @example
   * const [checked] = ctx.naming.check(['चूडामणि'], { nakshatra: 'ASHWINI', pada: 1 });
   * console.log(checked.check?.samePada); // true
   */
  check(names: readonly string[], birth: Birth, rules?: NamingRules): readonly NameChecked[];
  /**
   * The names a birth gives: its pada's syllable and its star's deity.
   *
   * @example
   * ctx.naming.of({ nakshatra: 'KRITTIKA', pada: 1 }).deity; // 'deity.AGNI'
   */
  of(birth: Birth, rules?: NamingRules): Names;
  /**
   * Loads a name book into this context and answers its id.
   *
   * @example
   * const { book } = ctx.naming.load({ names: [{ devanagari: 'चूडामणि', sex: 'MALE' }] });
   */
  load(book: Book): LoadedBook;
  /**
   * The names a book suggests for a birth: a loaded book by its id, or a
   * book given whole for this one call.
   *
   * @example
   * const found = ctx.naming.suggest(book, { nakshatra: 'ASHWINI', pada: 1 }, { sex: 'MALE' });
   */
  suggest(book: string | Book, birth: Birth, filter?: SuggestFilter, rules?: NamingRules): Suggestions;
}

/** A birth of a study (`03-design/research.md`). */
export interface ResearchBirth {
  /** The instant, a Julian day in UTC. */
  readonly instant: number;
  /** Where, in degrees and metres. */
  readonly place: { readonly latitude: number; readonly longitude: number; readonly altitude?: number };
  /** The local clock's offset from UTC in seconds, east positive. */
  readonly utcOffsetSeconds: number;
  /**
   * How far either side the recorded time may be wrong, 0 to 720 minutes; a
   * rule whose answer differs anywhere in that window, read every step of
   * the study, is counted unstable on the chart.
   */
  readonly uncertaintyMinutes?: number;
}

/** A study's seed: a number, or a `bigint` past `Number.MAX_SAFE_INTEGER`. */
export type ResearchSeed = number | bigint;

/** The direction a test looks in, declared before the data. */
export type ResearchAlternative = 'GREATER' | 'LESS' | 'TWO_SIDED';

/** How many threads count the permutations; the answer is the same bits for each. */
export type ResearchParallelism = 'ONE' | { readonly THREADS: number };

/** What a study's charts are read for, beside its births. */
export interface ResearchStudy {
  /** The predicates, one per rule, as a chart request's `rules` record. */
  readonly rules: RuleRequest;
  /** When a rule counts as holding: standing (the default) or merely formed. */
  readonly holds?: 'STANDING' | 'FORMED';
}

/** The births a study reads, and how finely an uncertain one's window is read. */
export interface ResearchBirths {
  /** In the order the design labels them. */
  readonly births: readonly ResearchBirth[];
  /**
   * How far apart, in minutes, an uncertain birth's window is read, out
   * to both edges; a rule that changes anywhere in it is unstable there.
   * 2 when left out.
   */
  readonly uncertaintyStepMinutes?: number;
}

/** A permutation test of a design's groups. */
export interface ResearchGroupTest {
  readonly seed: ResearchSeed;
  /** 1 to 10 000 000, fixed in the request. */
  readonly permutations: number;
  readonly contrast:
    | { readonly kind: 'CASE_VS_REST'; readonly case: number }
    | { readonly kind: 'ANY_DIFFERENCE' };
  readonly alternative?: ResearchAlternative;
  /** Every interval's confidence, in (0, 1); 0.95 when left out. */
  readonly level?: number;
  /** When given, each row says which methods put it at or under this. */
  readonly alpha?: number;
  readonly parallelism?: ResearchParallelism;
}

/** A test of events against the shuffled-event null. */
export interface ResearchEventTest {
  readonly seed: ResearchSeed;
  readonly permutations: number;
  readonly alternative?: ResearchAlternative;
  /** Refuse a pairing that puts an event before a birth, or permute only among those that do not. */
  readonly afterBirth?: 'REFUSE' | 'RESTRICT_PAIRINGS';
  readonly level?: number;
  readonly alpha?: number;
  readonly parallelism?: ResearchParallelism;
  /** The most instants a transit rule may have the sky read at; a study above it is refused. The SDK's default when absent. */
  readonly maxPairs?: number;
}

/** A predicate's charts in one group. */
export interface ResearchGroupCount {
  readonly present: number;
  readonly absent: number;
  /** Not read, and left out of the denominator. */
  readonly unreadable: number;
  /** Different answers inside the time uncertainty, and left out of the denominator. */
  readonly unstable: number;
}

/** An estimate with its interval at the test's level. */
export interface ResearchInterval {
  readonly estimate: number;
  readonly low: number;
  readonly high: number;
}

/** One predicate's row of a study's counts. */
export interface ResearchCountRow {
  readonly predicate: string;
  /** Its charts in each group, groups in index order. */
  readonly counts: readonly ResearchGroupCount[];
}

/** One predicate's row of a test. */
export interface ResearchRow extends ResearchCountRow {
  /**
   * The statistic under the observed labels; `null` when it is unbounded, a
   * recombined sample beyond replicates that all agree.
   */
  readonly observed: number | null;
  /** `(exceed + 1)/(m + 1)`, never zero, with its Clopper–Pearson interval. */
  readonly p: { readonly exceed: number; readonly value: number; readonly low: number; readonly high: number };
  /** The hypergeometric p of the same statistic, where it applies. */
  readonly exact?: number;
  readonly adjusted: {
    readonly maxT: number;
    readonly holm: number;
    readonly bonferroni: number;
    readonly bh: number;
    readonly by: number;
  };
  /** The cases against the rest, for a case-against-rest test. */
  readonly effect?: {
    readonly riskCase: ResearchInterval;
    readonly riskRest: ResearchInterval;
    readonly riskDifference: ResearchInterval;
    readonly riskRatio: ResearchInterval | null;
    readonly oddsRatio: number | null;
    readonly cohenH: number;
    /**
     * The ratios pooled over the design's strata, for a design that has
     * them; beside the crude ones, the difference is the confounding the
     * strata removed.
     */
    readonly pooled?: {
      /** Mantel–Haenszel's risk ratio, with Greenland and Robins's interval. */
      readonly riskRatio: ResearchInterval | null;
      /** Mantel–Haenszel's odds ratio, with Robins, Breslow and Greenland's interval. */
      readonly oddsRatio: ResearchInterval | null;
      /** Breslow–Day's p (Tarone's correction) that the strata share one odds ratio. */
      readonly homogeneityP: number | null;
    };
  };
  /** The share observed against the share the null expects, for a one-group study. */
  readonly expected?: { readonly observed: number; readonly expected: number; readonly ratio?: number };
  readonly underAlpha?: {
    readonly raw: boolean;
    readonly maxT: boolean;
    readonly holm: boolean;
    readonly bonferroni: boolean;
    readonly bh: boolean;
    readonly by: boolean;
  };
}

/** A study's counts, with the provenance whose `inputHash` is its pre-registration. */
export interface ResearchCounts {
  readonly rows: readonly ResearchCountRow[];
  readonly provenance: Provenance;
}

/** A test's rows, per predicate and never a single verdict. */
export interface ResearchTested {
  readonly rows: readonly ResearchRow[];
  /** Permutations drawn, or, when `enumerated`, the arrangements the reference set holds. */
  readonly permutations: number;
  /** The smallest p-value they can give: `1/(m + 1)` drawn, `1/N` enumerated. */
  readonly resolution: number;
  /** Whether every arrangement was read once, because there were no more than the permutations asked: each p-value is then exact. */
  readonly enumerated: boolean;
  /** The generator and shuffle, so a reader can rerun the study. */
  readonly shuffle: string;
  readonly provenance: Provenance;
}

/** `sdk.research` — counts and permutation tests over a batch of births. */
export declare class ResearchArea {
  /**
   * How often each rule holds in each group.
   *
   * @example
   * const table = ctx.research.counts({ births, rules: { shipped: ['YOGAS'] }, design: { groups } });
   */
  counts(request: ResearchStudy & ResearchBirths & { readonly design: ResearchDesign }): ResearchCounts;
  /** Whether the design's groups differ on each rule, labels permuted. */
  compare(
    request: ResearchStudy &
      ResearchBirths & {
      readonly design: ResearchDesign;
      readonly test: ResearchGroupTest;
    },
  ): ResearchTested;
  /** Whether each rule is commoner than in the sample's recombined population. */
  expected(
    request: ResearchStudy &
      ResearchBirths & {
      readonly control: { readonly seed: ResearchSeed; readonly replicates: number; readonly strata?: readonly number[] };
      readonly test?: { readonly alternative?: ResearchAlternative; readonly level?: number; readonly alpha?: number };
    },
  ): ResearchTested;
  /** Whether each rule is delivered by a dasha at the subjects' own events more often than chance. */
  timed(
    request: ResearchStudy & {
      readonly subjects: readonly ResearchSubject[];
      readonly dasha: DashaSystem | DashaKey;
      /** 1 (the mahadasha) to 6; 2 when left out. */
      readonly depth?: number;
      readonly test: ResearchEventTest;
    } & (
        | {
          /** What the null keeps: the calendar of events, or each person's age at the event. No default. */
          readonly shuffle: 'EVENT_DATES' | 'AGES_AT_EVENT';
          readonly strata?: readonly number[];
          readonly referents?: never;
        }
        | {
          /** Each person their own control: the event drawn among the person's referent instants. */
          readonly shuffle: 'SELF_MATCHED';
          readonly referents: ResearchReferents;
          readonly strata?: never;
        }
      ),
  ): ResearchTested;
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
  /** What matches without a chart: two names, star to star. */
  readonly matching: MatchingArea;
  /** What a name and a birth date say under numerology's two systems. */
  readonly numerology: NumerologyArea;
  /** The naming of a child: syllables, names read and checked, and a name book. */
  readonly naming: NamingArea;
  /** Counts and permutation tests over a batch of births. */
  readonly research: ResearchArea;
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
   * A stored baseline settings snapshot translated into what a context
   * opens with; reads none of this context's settings. A field the
   * translation does not know is `INVALID_ARG`, one it decided not to
   * carry `UNSUPPORTED`, each on `snapshot.<field>`.
   */
  recordedSettings(snapshot: Readonly<Record<string, unknown>> | string): RecordedSettings;
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
 * @param sanitizer the sanitizer the caller says it is loading, which
 *   admits a sanitizer build of that kind whose path was named
 */
export declare function refuseBuild(
  info: BuildInfo,
  named: boolean,
  sanitizer?: string,
): string | null;

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
