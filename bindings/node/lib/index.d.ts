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
  AvasthaBaladi,
  AvasthaDeeptadi,
  AvasthaSayanadi,
  AvasthaCheshta,
  AvasthaJagradadi,
  AvasthaLajjitadi,
  Ayana,
  Body,
  Burning,
  Calendar,
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
  Nakshatra,
  Paksha,
  Panchaka,
  Point,
  Quadrant,
  Rashi,
  Ritu,
  Samvatsara,
  Shodhana,
  Ekadhipatya,
  Vaiseshikamsa,
  VimshopakaScoring,
  DashaPhase,
  Nature,
  BrahmaRule,
  BrahmaOutcome,
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
  Way,
  Motion,
  WesternAspect,
  AspectPhase,
  GocharVerdict,
  NodeObstruction,
  NodeVedha,
  Relationship,
  Scale,
  Status,
  Strength,
  Tithi,
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
  VargaRelation,
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
import type { Provenance, Step } from './records.js';

export * from './catalogue.js';
export * from './records.js';
export type * from './types.js';
export type * from './blob.js';
/**
 * A date in a calendar, without naming the fields a call fills in: the era
 * and the era year are what the call resolves them to, and the resolution
 * is `DEFINED`, which is what a date a caller states means.
 *
 * @example date(Calendar.Gregorian, 2015, 4, 14)
 */
export declare function date(calendar: Calendar, year: number, month: number, day: number): CalendarDate;

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

/** A zone of the embedded database, by its IANA name. */
export declare function ianaZone(name: string): ZoneSpec;

/** A fixed offset from UTC, in seconds east. */
export declare function fixedZone(offsetSeconds: number): ZoneSpec;

/**
 * Local mean time at a longitude east of Greenwich, which is what a chart
 * from before the zone existed is cast in.
 */
export declare function localMeanZone(longitudeDeg: Longitude): ZoneSpec;

export { decodeCharts, decodeIntlRender, decodeNaam, decodePanchanga, decodePositions } from './blob.js';
export { entityForms, messages } from './messages.js';

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

/**
 * A natal point a transit aspects: a natal graha or the lagna. The same
 * spelling goes into `HitRequest.points` and comes back as an aspect's `to`.
 */
export type NatalPoint =
  | { readonly point: 'GRAHA'; readonly graha: Graha | 'unknown' }
  | { readonly point: 'LAGNA' };

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

/**
 * The transit hit list to search against every chart of a request; every
 * field but the window is optional, and an absent one is the default.
 */
export interface HitRequest {
  /** The window's start, a UTC Julian day. */
  readonly from: number;
  /** The window's end, after the start. */
  readonly to: number;
  /** The grahas to search, bare (`'SUN'`) or full (`'graha.SUN'`); the nine by default. */
  readonly grahas?: readonly GrahaName[];
  /** The kinds of event to report; all four by default. */
  readonly kinds?: readonly HitKind[];
  /** The natal points aspected: a graha's key, `'LAGNA'`, or an aspect's `to`; the nine and the lagna by default. */
  readonly points?: readonly (
    | NatalPoint
    | { readonly point: 'GRAHA'; readonly graha: GrahaName }
    | 'LAGNA'
    | GrahaName
  )[];
  /** The aspects' angles, whole degrees from 0 to 180; the conjunction and opposition by default (C145). */
  readonly aspects?: readonly number[];
  /** An orb in degrees, more than 0, under 15 and under half the step between the aspects' lines, for each window's opening and closing; exact only by default (C146). */
  readonly orbDeg?: number;
}

/**
 * Sade Sati and Saturn's smaller spells to find for every chart of a request
 * (`03-design/sade-sati.md`); every field but `from` is optional, and an
 * absent one is the default.
 */
export interface SadeSatiRequest {
  /** The window's start, a UTC Julian day. */
  readonly from: number;
  /** The window's end, not before the start; `from` by default, one instant. */
  readonly to?: number;
  /** The natal point the houses are counted from: the Moon by default, or the lagna (C139). */
  readonly countedFrom?: GocharFrom;
  /** Whole signs from the reference's sign by default, or 30° houses centred on its degree (C147). */
  readonly reckoning?: Reckoning;
  /** The smaller spells, houses 3 to 11 each named once; the 4th and the 8th by default (C149). */
  readonly spells?: readonly number[];
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

/** One term: its lord, and the degree within the sign it ends at, exclusive. */
export interface Term {
  readonly lord: GrahaName;
  readonly end: number;
}

/**
 * The terms and triplicities a reading of the dignities uses: a system of
 * terms by name or a table of the caller's own, twelve signs of five terms
 * from Aries (`03-design/essential-dignities.md`, C208).
 */
export interface DignityRules {
  /** Lilly's printing of Ptolemy's terms by default. */
  readonly terms?: Exclude<Terms, 'TABLE' | 'unknown'> | { readonly TABLE: readonly (readonly Term[])[] };
  /** Lilly's by default, Mars ruling water alone. */
  readonly triplicities?: Exclude<Triplicities, 'unknown'>;
}

/** What each dignity and debility is worth; Lilly's (p. 115) for any left out. */
export interface DignityScores {
  readonly house: number;
  readonly exaltation: number;
  readonly triplicity: number;
  readonly term: number;
  readonly face: number;
  readonly detriment: number;
  readonly fall: number;
  readonly peregrine: number;
}

/**
 * How to read every chart's essential dignities
 * (`03-design/essential-dignities.md`); every field is optional, and an
 * absent one is the default.
 */
export interface DignityRequest {
  /** The Sun's centre above the true horizon by default, Valens's hemisphere (C209). */
  readonly sectRule?: Exclude<SectRule, 'unknown'>;
  readonly rules?: DignityRules;
  readonly scores?: Partial<DignityScores>;
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
}

/**
 * The orbs and limits Lilly's accidental fortitudes are judged by
 * (`03-design/essential-dignities.md` §Accidental fortitudes); Lilly's for
 * any left out.
 */
export interface AccidentalRules {
  /** Combust within this many degrees of the Sun; 8°30′ by default. */
  readonly combustionDeg: number;
  /** Whether combustion also asks for the Sun's sign (C211); true by default. */
  readonly combustionInSign: boolean;
  /** Under the beams within this many degrees (C212); 17 by default. */
  readonly beamsDeg: number;
  /** Cazimi within this many degrees; 17′ by default. */
  readonly cazimiDeg: number;
  /** A planet this near the next cusp is in its house (p. 33, C214); 5 by default. */
  readonly cuspOrbDeg: number;
  /** With a star within this many degrees; 5 by default. */
  readonly starOrbDeg: number;
  /** Partile by the same degree, Lilly's, or within an orb of the exact aspect (C216). */
  readonly partile: Exclude<Partile, 'WITHIN'> | { readonly WITHIN: { readonly orbDeg: number } };
  /** Besieged within one sign, Lilly's example, or on an arc no wider than a span (C215). */
  readonly siege: Exclude<Siege, 'WITHIN'> | { readonly WITHIN: { readonly spanDeg: number } };
  /** The mean daily motions swift and slow are judged against, the seven in the Chaldean order. */
  readonly meanMotionDeg: readonly number[];
}

/** What each of Lilly's accidental lines is worth (p. 115); his for any left out. */
export interface AccidentalScores {
  /** The first house to the twelfth. */
  readonly houses: readonly number[];
  readonly direct: number;
  readonly retrograde: number;
  readonly swift: number;
  readonly slow: number;
  /** Saturn, Jupiter or Mars oriental. */
  readonly superiorOriental: number;
  /** Saturn, Jupiter or Mars occidental. */
  readonly superiorOccidental: number;
  /** Venus or Mercury oriental. */
  readonly inferiorOriental: number;
  /** Venus or Mercury occidental. */
  readonly inferiorOccidental: number;
  readonly increasing: number;
  readonly decreasing: number;
  readonly freeFromCombustion: number;
  readonly cazimi: number;
  readonly combust: number;
  readonly underBeams: number;
  readonly conjunctBenefic: number;
  readonly conjunctNorthNode: number;
  readonly trineBenefic: number;
  readonly sextileBenefic: number;
  readonly conjunctMalefic: number;
  readonly conjunctSouthNode: number;
  readonly opposedMalefic: number;
  readonly squareMalefic: number;
  readonly besieged: number;
  readonly regulus: number;
  readonly spica: number;
  readonly algol: number;
}

/**
 * How to read every chart's fortitudes, both halves of Lilly's table
 * (`03-design/essential-dignities.md` §Accidental fortitudes); every field
 * is optional, and an absent one is Lilly's. An answer's `rules` and
 * `scores` are requests as they stand.
 *
 * @example
 * const asked: FortitudeRequest = { rules: { partile: { WITHIN: { orbDeg: 1 } } }, scores: { regulus: 5 } };
 */
export interface FortitudeRequest {
  readonly dignities?: DignityRequest;
  readonly rules?: Partial<AccidentalRules>;
  readonly scores?: Partial<AccidentalScores>;
  readonly almuten?: Partial<AlmutenRules>;
}

/** How the almutens are read; Lilly's by default (`03-design/essential-dignities.md` §The almuten). */
export interface AlmutenRules {
  /** What of a place its dignities are counted from: `DEGREE` (all five) or `SIGN` (house, exaltation, triplicity), C218. */
  readonly place: PlaceReading;
  /** How Fortune is taken by night: `DAY_AND_NIGHT`, Lilly's, `REVERSED_BY_NIGHT` or `REVERSED_WHILE_MOON_UP`, C220 and C221. */
  readonly fortune: FortuneRule;
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

/**
 * How to read every chart's lots (`03-design/hellenistic-lots.md`); every
 * field is optional, and an absent one is Valens's.
 *
 * @example
 * const asked: LotRequest = { fortune: 'REVERSED_WHILE_MOON_UP' };
 */
export interface LotRequest {
  /** The Sun's centre above the true horizon by default, Valens's hemisphere (C209). */
  readonly sectRule?: Exclude<SectRule, 'unknown'>;
  /** How Fortune is taken by night; `REVERSED_BY_NIGHT`, Valens's II.22, by default (C221). */
  readonly fortune?: Exclude<FortuneRule, 'unknown'>;
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

/**
 * How to read every chart's considerations before judgement
 * (`03-design/hellenistic-considerations.md`); every field is optional,
 * and an absent one is Lilly's.
 *
 * @example
 * const asked: ConsiderationRequest = { moonLateFromDeg: 25 };
 */
export interface ConsiderationRequest {
  /** From what degree of her sign the Moon is late; 27 by default, his late Ascendant (C229). */
  readonly moonLateFromDeg?: number;
  /**
   * The seven whole orbs in the Chaldean order (Saturn, Jupiter, Mars, the
   * Sun, Venus, Mercury, the Moon), half of each counting toward an
   * application; Lilly's p. 107 by default (C230).
   */
  readonly orbsDeg?: readonly [number, number, number, number, number, number, number];
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
  readonly rules: Required<ConsiderationRequest>;
}

/** A span of time a progression's rate is stated in. */
export type ProgressionSpan = 'DAY' | 'SYNODIC_MONTH' | 'SIDEREAL_MONTH' | 'YEAR' | { readonly DAYS: number };

/** How the progressed midheaven moves (C237). */
export type AngleMethod =
  | 'NAIBOD_RIGHT_ASCENSION'
  | 'NAIBOD_LONGITUDE'
  | 'SOLAR_ARC_LONGITUDE'
  | 'SOLAR_ARC_RIGHT_ASCENSION'
  | 'QUOTIDIAN';

/**
 * What to read every chart's birth through: the progressed chart and the
 * direction at an instant of life (`at`), the contacts over a window
 * (`contacts`), or both (`03-design/western-progressions.md`). Every other
 * field is Leo's default when absent.
 *
 * @example
 * const leo: ProgressionsRequest = { at: 2460676.5, year: 'NOON_SIDEREAL_TIME' };
 * const moon: ProgressionsRequest = { contacts: { from: 2460676.5, to: 2461041.5, grahas: ['MOON'] } };
 */
export interface ProgressionsRequest {
  /** The instant of life, a UTC Julian day. */
  readonly at?: number;
  /** How much sky measures how much life; a day for a year by default. */
  readonly rate?: { readonly sky: ProgressionSpan; readonly life: ProgressionSpan };
  /** How long a year of life is (C236); the tropical year by default. */
  readonly year?: 'TROPICAL' | 'JULIAN' | 'NOON_SIDEREAL_TIME';
  /** How the progressed midheaven moves; Leo's mean Sun in right ascension by default. */
  readonly angles?: AngleMethod;
  /** The direction's arc: the Sun's under the rate by default, or a measure's. */
  readonly direction?: 'SOLAR' | 'NAIBOD' | 'PTOLEMY' | { readonly PER_YEAR: number };
  /** A window of life to find the contacts in, spelled as a hit list spells them. */
  readonly contacts?: {
    readonly from: number;
    readonly to: number;
    /** The progressed planets; the seven by default. */
    readonly grahas?: HitRequest['grahas'];
    /** The radical points; the seven and the lagna by default. */
    readonly points?: HitRequest['points'];
    /** The aspects' angles, whole degrees to 180; Leo's table (p. 48) by default. */
    readonly aspects?: HitRequest['aspects'];
  };
}

/**
 * Which Western aspects to look for, and under which orbs
 * (`03-design/western-aspects.md`). Leo's nine under his orbs (*How to
 * Judge a Nativity*, pp. 43–47) when every field is absent (C240).
 *
 * @example
 * const leo: WesternAspectRequest = {};
 * const lilly: WesternAspectRequest = {
 *   aspects: ['CONJUNCTION', 'SEXTILE', 'SQUARE', 'TRINE', 'OPPOSITION'],
 *   orbs: { model: 'MOIETIES', orbs: [{ graha: 'SUN', orbDeg: 17 }, { graha: 'MOON', orbDeg: 12.5 }] },
 * };
 */
export interface WesternAspectRequest {
  /** The aspects looked for, each once; Leo's nine by default. */
  readonly aspects?: readonly WesternAspect[];
  /** The orb model; Leo's by default. */
  readonly orbs?: WesternOrbModel;
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
 * What the antiscia are asked (`03-design/western-antiscia.md`): the orbs
 * a pair is read under, at the conjunction; Lilly's moieties when absent
 * (C244), which give the outer three none.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, antiscia: {} });
 * for (const row of chart.antiscia?.pairs ?? []) console.log(row.first, row.contrary ? 'contrantiscion' : 'antiscion', row.second);
 */
export interface AntisciaRequest {
  readonly orbs?: WesternOrbModel;
  /**
   * Reads each reflection on the cusps too, in the division named, Lilly's
   * Regiomontanus when `{}` (`03-design/western-houses.md`, C251). None by
   * default.
   */
  readonly cusps?: WesternHouseRequest;
}

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

/**
 * What a chart's harmonic is asked (`03-design/western-harmonics.md`):
 * `number`, a whole number from 1 to 360 every longitude is multiplied by
 * (Addey), and `orbDeg`, how close two points meet in the harmonic chart,
 * 12° when absent (C252), at most 30°.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, harmonic: { number: 9 } });
 * for (const row of chart.harmonic?.rows ?? []) console.log(row.first, row.second, row.apartDeg);
 */
export interface HarmonicRequest {
  readonly number: number;
  readonly orbDeg?: number;
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

/**
 * A match with a partner's birth (`03-design/matching.md`): the partner,
 * founded once for the whole batch under the context's sidereal profile;
 * the side the partner stands on, every chart standing on the other; and
 * the rules, each the source's own when absent.
 *
 * @example
 * const chart = ctx.chart.found({
 *   instant, place, utcOffsetSeconds,
 *   matching: { partner: { instant: 2447892.5, place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 } }, partnerRole: 'BRIDE' },
 * });
 * for (const row of chart.matching?.kootas ?? []) console.log(row.reading.koota, row.points, row.maxPoints);
 */
export interface MatchingRequest {
  /** The partner's birth. */
  readonly partner: {
    readonly instant: number;
    readonly place: ChartPlace;
    /** The partner's clock, seconds east of UTC; 0 when absent. */
    readonly utcOffsetSeconds?: number;
  };
  /** The side the partner stands on: Varna and Gana read differently when the two swap. */
  readonly partnerRole: 'BRIDE' | 'GROOM';
  readonly rules?: KootaRules;
  /** The readings the ten considerations are computed under. */
  readonly porutham?: PoruthamRules;
  /** The readings the Kuja dosha is computed under. */
  readonly kuja?: KujaRules;
}

/** The readings the Kuja dosha is computed under; each default is the verse's own. */
export interface KujaRules {
  /** Which houses of Mars make the dosha: *Manasagari*'s five, `'MANASAGARI'` (the default), or `'WITH_SECOND'` (C285). */
  readonly houses?: 'MANASAGARI' | 'WITH_SECOND';
  /** From where they are counted: `'LAGNA'` (the default) or `'LAGNA_MOON_VENUS'` (C286). */
  readonly from?: 'LAGNA' | 'LAGNA_MOON_VENUS';
}

/** The readings the ten considerations are computed under; each default is the chapter's own. */
export interface PoruthamRules {
  /** Whose quarter comes first in one star across two signs: `'GROOM_EARLIER'` (the default) or `'BRIDE_FIRST_SIGN'` (C270). */
  readonly twoSignStar?: 'GROOM_EARLIER' | 'BRIDE_FIRST_SIGN';
  /** How far Sthree-Dheergham asks: beyond the `'THIRTEENTH'` (the default) or the `'SEVENTH'` (C272). */
  readonly deerghaBeyond?: 'THIRTEENTH' | 'SEVENTH';
  /** Which friendship of the lords agrees: `'MUTUAL'` (the default) or `'ONE_WAY'` (C273). */
  readonly lordsFriendship?: 'MUTUAL' | 'ONE_WAY';
}

/** The readings an Ashta Koota is computed under; each default is the source's own. */
export interface KootaRules {
  /** The point of an equal varna: `'WHOLE'` (the default) or `'HALF'` (C259). */
  readonly equalVarna?: 'WHOLE' | 'HALF';
  /** The points of a Deva bride and a Manushya groom: `'FOUR'` (the default) or `'THREE'` (C262). */
  readonly devaBride?: 'FOUR' | 'THREE';
  /** How a bad Bhakoot is lifted: `'ANY_ONE'` exception (the default) or `'GARGA'`'s count (C263). */
  readonly bhakootLift?: 'ANY_ONE' | 'GARGA';
  /** Which shared nadi is a dosha: `'ANY'` (the default) or `'MIDDLE_ONLY'` (C264). */
  readonly nadiDosha?: 'ANY' | 'MIDDLE_ONLY';
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

/** How a name is read for naam milan; each default is the source's own (C291, C293). */
export interface NameRules {
  /** A name in Latin letters: `'REFUSE'`d (the default), or read as `'IAST'`; English "ch" is IAST "c", so it is never guessed. */
  readonly latin?: 'REFUSE' | 'IAST';
  /** Where a syllable in Abhijit's row is placed: `'REFUSE'` (the default), `'UTTARA_ASHADHA'` (its 4th quarter) or `'SHRAVANA'` (its 1st) (C294). */
  readonly abhijit?: 'REFUSE' | 'UTTARA_ASHADHA' | 'SHRAVANA';
}

/** The readings naam milan is computed under; each optional, each the source's own when absent. */
export interface NaamRules {
  readonly name?: NameRules;
  /** The readings the Ashta Koota of the name stars is computed under. */
  readonly koota?: KootaRules;
  /** The readings the ten considerations of the name stars are computed under. */
  readonly porutham?: PoruthamRules;
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
}

/** The Kuja dosha of a bride and a groom (*Manasagari*, jāyābhāva v. 4), as clauses: nothing is lifted (C288). */
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

/**
 * What the equal distances are asked (`03-design/western-midpoints.md`):
 * how far from the axis through two planets' midpoint a third may stand,
 * 0.5° when absent (C245), at most 10°.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, midpoints: {} });
 * for (const row of chart.midpoints ?? []) console.log(row.middle, 'between', row.first, 'and', row.second);
 */
export interface MidpointRequest {
  readonly orbDeg?: number;
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
    readonly place: ChartPlace;
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

/**
 * The Davison birth of a chart and a synastry's partner (C248), in the
 * shape a chart request takes, so it founds a chart as a birth does.
 *
 * @example
 * const davison = chart.synastryDavison;
 * if (davison !== null) ctx.chart.found({ ...davison });
 */
export interface DavisonBirth {
  /** The mean instant, a Julian day on the UTC scale. */
  readonly instant: number;
  /** The mean place, the longitude taken the shorter way round. */
  readonly place: ChartPlace;
  /** The mean of the two clocks, seconds east of UTC: it names only the civil day. */
  readonly utcOffsetSeconds: number;
}

/**
 * How close two distances from the equator must stand to be a parallel
 * (`03-design/western-declinations.md`): Leo's 1° (*How to Judge a
 * Nativity*, p. 47) when absent, at most 10°.
 *
 * @example
 * const chart = ctx.chart.found({ instant, place, utcOffsetSeconds, parallels: {} });
 * for (const row of chart.parallels ?? []) console.log(row.first, row.contrary ? 'contra' : 'parallel', row.second);
 */
export interface ParallelRequest {
  readonly orbDeg?: number;
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
}

/**
 * How to read every chart's perfection (`03-design/hellenistic-perfection.md`);
 * name the quesited's significator or the house of the matter, not both.
 *
 * @example
 * const marriage: PerfectionRequest = { house: 7 };
 * const named: PerfectionRequest = { querent: 'VENUS', quesited: 'MARS', rules: { horizonDays: 30 } };
 */
export interface PerfectionRequest {
  /** The querent's significator; the Ascendant's lord by default. One of the seven. */
  readonly querent?: GrahaName;
  /** The quesited's significator. One of the seven, never the querent's. */
  readonly quesited?: GrahaName;
  /** The house of the matter, 1 to 12: its cusp's lord signifies the quesited. */
  readonly house?: number;
  readonly rules?: {
    /** The seven whole orbs in the Chaldean order; Lilly's p. 107 by default. */
    readonly orbsDeg?: readonly [number, number, number, number, number, number, number];
    /** How many days ahead to look; by default until the swifter significator leaves its sign (C232). */
    readonly horizonDays?: number;
    /**
     * Whether a third planet's contact counts only before the planet applying
     * leaves its sign (C234); `true` by default, `false` counts every contact
     * inside the horizon.
     */
    readonly withinSign?: boolean;
  };
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

/**
 * A KP reading to make of every chart of a request (`03-design/kp.md`);
 * every field is optional, and an absent one is the default.
 */
export interface KpRequest {
  /** The querent's horary number, 1 to 249: the cusps are cast from it, the ruling planets stay the moment's (C156). */
  readonly number?: number;
  /** Seconds east of UT that the civil day lord is the weekday on; the request's own `utcOffsetSeconds` by default (C151). */
  readonly clock?: number;
  /** Whether to read a chart whose zodiac is not Krishnamurti's; false by default, which refuses it (C157). */
  readonly anyAyanamsha?: boolean;
}

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

/** Which longitude an annual chart's Sun returns to (`03-design/annual-chart.md`). */
export type VarshaReading =
  /** The natal sidereal longitude, read on the chart's own ayanamsha basis: the tradition's. */
  | 'SIDEREAL'
  /** The natal tropical longitude: the Western solar return. Forty years on it is most of a circle of lagna from the sidereal one, so it is a choice and never a fallback. */
  | 'TROPICAL'
  /** A whole sidereal year for each year of life, from birth: the older arithmetic, and the only reading that needs no ephemeris. */
  | 'MEAN';

/**
 * Where the Muntha stands inside the sign it has reached (crux C107).
 *
 * Both readings give the same sign at the return and part over the year,
 * so they differ for a Tajika aspect taken to the Muntha and nothing else.
 */
export type MunthaDegree =
  /** It enters each year at its sign's first degree and crosses the sign during the year: the source's own reading. */
  | 'SIGN_START'
  /** It carries the natal lagna's degree into each new sign. */
  | 'NATAL_DEGREE';

/** The annual charts a request asks for. */
export interface VarshaRequest {
  /** Which longitude the Sun returns to; `SIDEREAL` by default. */
  readonly reading?: VarshaReading;
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

/** Where the sources differ on an annual dasha, each a named reading (`03-design/annual-dashas.md`). */
export interface AnnualDashaRules {
  /**
   * What a unit of the year is (crux C122): the Sun's motion through one
   * degree, `'SUN_DEGREES'`, the source's own, so the year closes on the
   * next return; an `'EVEN'` share of the time between the returns; or
   * `{ DAYS: n }`, the whole year as so many civil days from the return,
   * the printed durations (360, and 365 for the Patyayini).
   */
  readonly clock?: 'SUN_DEGREES' | 'EVEN' | { readonly DAYS: number };
  /**
   * Where a nakshatra year's balance comes from (crux C123): what remained
   * of the birth Moon's nakshatra, `'NATAL_MOON'`, the source's own; the
   * Moon's at the return, `'ENTRY_MOON'`; or `'WHOLE'`, none.
   */
  readonly balance?: 'NATAL_MOON' | 'ENTRY_MOON' | 'WHOLE';
  /**
   * How the balance is measured, `'SPATIAL'` by arc or `'TEMPORAL'` by
   * time; absent, each balance's source's own: by arc for the birth Moon,
   * by time for the Moon at the return.
   */
  readonly measure?: 'SPATIAL' | 'TEMPORAL';
  /** How the first lord's two pieces are divided among sub-lords, as the natal birth period's; `'COMPRESSED'` by default. */
  readonly birthPeriod?: 'COMPRESSED' | 'ELAPSED';
  /** How many levels the periods go down: `2`, mahadashas and antardashas, by default. */
  readonly depth?: number;
}

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

/** Where the sources differ on a saham's strength (`03-design/tajika-saham-strength.md`). */
export interface SahamStrengthReadings {
  /**
   * Which planets are benefic and malefic: the `'CHAPTER'`'s own, the Sun
   * a malefic among them, or the catalogue's `'PARASHARI'` natures.
   */
  readonly natures?: 'CHAPTER' | 'PARASHARI';
  /** Tajika's `'POSITIONAL'` friendship, the source's, or the catalogue's `'NATURAL'` one. */
  readonly friendship?: 'POSITIONAL' | 'NATURAL';
  /** The Vishwa bala below which a saham's lord is weak, in **sub-sub units**: `5 * 3600` by default. */
  readonly weakBelow?: number;
}

/** Where the sources differ on the Harsha bala (`03-design/tajika-harsha.md`). */
export interface HarshaRules {
  /** Venus's house of joy: the verse's `'FIFTH'`, or the `'TWELFTH'` a widely used program reads. */
  readonly venus?: 'FIFTH' | 'TWELFTH';
}

/** Where the sources differ on a saham, each a named reading (`03-design/tajika-sahams.md`). */
export interface SahamRules {
  /**
   * When a saham is carried a sign further: when c does not fall between b
   * and a by `'DEGREES'`, the source's own; by whole `'SIGNS'`, as a widely
   * used program reads it; or `'NEVER'`.
   */
  readonly addSign?: 'DEGREES' | 'SIGNS' | 'NEVER';
  /**
   * Where a house's point stands: `'SRIPATI'`'s mid-point built from the
   * angles, the source's own; the chart's `'CHALIT'` under its profile; or
   * `'EQUAL'` houses from the lagna's degree.
   */
  readonly houses?: 'SRIPATI' | 'CHALIT' | 'EQUAL';
  /** Roga's formula: lagna − Moon + lagna, `'LAGNA'`, or the other authority's `'SATURN'`. */
  readonly roga?: 'LAGNA' | 'SATURN';
}

/** Where the source leaves the sixteen Tajika yogas a choice, each a named reading. */
export interface YogaRules {
  /** How a pair less than a degree past reads; `'POORNA'` by default (crux C112). */
  readonly drishti?: { readonly subDegree?: 'POORNA' | 'ISHRAFA' };
  /**
   * The strength below which a planet with no dignity is weak, in **sub-sub
   * units**, 3600 to a unit: `5 * 3600` by default (crux C116).
   */
  readonly weakBelow?: number;
  /** The strength from which a planet is strong, sub-sub units; `10 * 3600` by default. */
  readonly strongFrom?: number;
  /**
   * Which lord a Tambira lets reach the next sign: the definition's
   * `'KARYESHA'` by default, or `'EITHER_LORD'`, the source's "some
   * authorities".
   */
  readonly tambira?: 'KARYESHA' | 'EITHER_LORD';
  /**
   * When the Moon counts among Kuttha's benefics: `'ALWAYS'` by default,
   * Charak's list, or `'WAXING'`, the commentary's "full Moon" (crux C117).
   */
  readonly moonBenefic?: 'ALWAYS' | 'WAXING';
}

/** Where the sources differ on the lord of the year, each a named reading. */
export interface VarsheshaRules {
  /**
   * Who takes the year when nobody aspects the lagna: the Muntha's lord by
   * default, the annual lagna's lord, or the Nilakanthi's strongest of the five.
   */
  readonly noneAspects?: 'MUNTHA_LORD' | 'ANNUAL_LAGNA_LORD' | 'STRONGEST';
  /** Who takes it on an outright tie; the Muntha's lord by default. */
  readonly tied?: 'MUNTHA_LORD' | 'DINA_RATRI_PATI';
  /**
   * Whether the Moon may hold it: `'PASSED_OVER'` by default, stepping down
   * to the next claimant and else to its Ithasala successor; `'ITHASALA'`,
   * the Nilakanthi's successor at once; or `'LIKE_ANY_OTHER'`.
   */
  readonly moon?: 'PASSED_OVER' | 'ITHASALA' | 'LIKE_ANY_OTHER';
  /** Who may succeed the Moon: any planet by default, or only an office-bearer. */
  readonly moonPartner?: 'ANY_PLANET' | 'OFFICE_BEARER';
  /** How the Ithasala the Moon's successor needs is read, as for the yogas. */
  readonly drishti?: { readonly subDegree?: 'POORNA' | 'ISHRAFA' };
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
  readonly start?: 'LAGNA' | 'ARUDHA_LAGNA' | 'NAVAMSA_LAGNA';
  /** The order it visits the signs in; every sign in turn by default. */
  readonly order?: 'CONSECUTIVE' | 'TRINE_GROUPS' | 'DRISHTI_CHAIN' | 'LEAP';
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

/** A set of rules the SDK ships. */
export type ShippedRules = 'DOSHAS' | 'YOGAS' | 'GANDANTAS' | 'ARISHTAS' | 'READINGS' | 'NABHASAS';

/**
 * The rules a request asks a chart to answer (`03-design/rules-at-the-boundary.md`):
 * shipped sets by name and a consumer's own rules in the SDK's rule format.
 */
export interface RuleRequest {
  /** The shipped sets to evaluate. */
  readonly shipped?: readonly ShippedRules[];
  /** A consumer's own rules, which may name shipped rules by key. */
  readonly rules?: readonly Readonly<Record<string, unknown>>[];
  /** The readings to evaluate under; `'TEXTS'` by default. */
  readonly readings?: 'TEXTS' | 'RECORDING_ENGINE';
  /** Whether to add the twelve house readings. */
  readonly houses?: boolean;
  /**
   * Whether to add the three pairs, the three spans, the rays, the span the
   * strongest names and the marakas; asks the chart for the Shadbala and
   * Bhava bala they weigh.
   */
  readonly longevity?: boolean;
  /** How the three spans are read, when `longevity` asks; BPHS's by default. */
  readonly ayurdaya?: AyurdayaRules;
  /** How the three pairs are read, when `longevity` asks; the verses' by default. */
  readonly threePairs?: ThreePairsRules;
  /** How the rays are read, when `longevity` asks; the translator's note's by default. */
  readonly rasmi?: RasmiRules;
}

/**
 * How Pindayu, Nisargayu and Amsayu are read (cruxes C104, C302 to C304);
 * every field optional, BPHS's readings by default.
 *
 * @example
 * // As *Jataka Parijata* ch. 5 reads Varahamihira's reductions.
 * const rules: RuleRequest = {
 *   longevity: true,
 *   ayurdaya: { enemy_exempt: 'mars', enmity: 'compound', rising: 'every' },
 * };
 */
export interface AyurdayaRules {
  /** Several reductions on one graha: the largest only, or each in turn. */
  readonly combine?: 'largest' | 'each';
  /** Nisargayu's years given as Pindayu gives them, or listed alone (always 120). */
  readonly nisarga?: 'like-pindayu' | 'listed';
  /** Whether Amsayu triples and doubles a graha in dignity. */
  readonly amsayu_multiplied?: boolean;
  /** Who keeps its years in an enemy's sign: a retrograde graha, or Mars. */
  readonly enemy_exempt?: 'retrograde' | 'mars';
  /** Whose enmity takes a third: natural, or compound. */
  readonly enmity?: 'natural' | 'compound';
  /** Whose years a rising malefic takes: its own, or every giver's. */
  readonly rising?: 'malefic' | 'every';
}

/** How the three pairs are read (crux C103); every field optional, the verses' by default. */
export interface ThreePairsRules {
  /** Which degrees rectify the class: those still to run, or those gone. */
  readonly rectification?: 'remaining' | 'elapsed';
  /** What the degrees multiply: the class's years once, or once per pair. */
  readonly basis?: 'class-years' | 'per-pair';
  /** Whether Saturn among the contributors lowers the class or raises it. */
  readonly saturn?: 'lowers' | 'raises';
}

/**
 * How the seven grahas' rays and Rasmija years are read (*Jataka Parijata* ch. 5 vv. 22 to 25,
 * cruxes C305 to C307); every field optional, the translator's note's
 * readings by default, whose figure the SDK reproduces.
 *
 * @example
 * // As v. 24 reads them: the place by the sign.
 * const rules: RuleRequest = { longevity: true, rasmi: { place: 'sign' } };
 */
export interface RasmiRules {
  /** Which place doubles them or takes a share: the dwadasamsa, or the sign. */
  readonly place?: 'dwadasamsa' | 'sign';
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
 * The narrative plans a request asks a chart for
 * (`03-design/plans-at-the-boundary.md`). Each composer is off by default,
 * and `readings` needs `rules` beside it, since it says what the rules a
 * chart held answered; the sections the others read are computed for them.
 */
export interface PlanRequest {
  /** Where each of the nine grahas stands and who shares a sign. */
  readonly placements?: boolean;
  /** What each rule the chart held says. */
  readonly readings?: boolean;
  /** Each graha's Shadbala in rupas, the strongest first. */
  readonly strength?: boolean;
  /** The lord of each of the twelve bhavas, first house first. */
  readonly houses?: boolean;
  /** Where each graha stands to the degree, which `placements` rounds away. */
  readonly positions?: boolean;
  /** Which graha looks at which, and how strongly. */
  readonly aspects?: boolean;
  /** What each graha is where it stands: dignity, navamsha, motion, combustion. */
  readonly conditions?: boolean;
  /** Which chara karaka each graha holds, under both schemes. */
  readonly karakas?: boolean;
  /**
   * Where the placement system and the chalit put a graha in different
   * bhavas. Says nothing of a chart whose readings agree.
   */
  readonly chalit?: boolean;
  /**
   * What a loaded corpus of state readings says of this chart's subjects: a
   * graha in a bhava, the lagna's sign, each limb of the panchanga, and
   * what the birth nakshatra is. Says nothing until a pack carrying those
   * readings is loaded.
   */
  readonly phala?: boolean;
  /**
   * The other half of a graha's state: how it stands to its dispositor
   * under all three friendships, and the four avasthas — the fifth of its
   * sign, its wakefulness, its brightness where the chart decides one,
   * and the lajjitadi that hold beside the ones nothing decides.
   */
  /**
   * The almanac of the chart's day: the tithi with its paksha, the vara,
   * the nakshatra and the Moon's pada in it, the yoga and the karana, and
   * whether the birth fell by day where the chart says.
   */
  /**
   * Each bhava's strength in virupas, the first house first. It says the
   * weight and never a verdict: a bhava carries no requirement.
   */
  readonly bhavaBala?: boolean;
  /** Each graha's Vimshopaka under all four schemes, each naming its own. */
  readonly vimshopaka?: boolean;
  readonly panchanga?: boolean;
  readonly states?: boolean;
  /**
   * What each graha's placement says of its dasha: when in the dasha its
   * effects come, whether its place is auspicious, the points its dignity
   * earns and whether the placement makes the dasha favourable — with the
   * reading a loaded corpus carries of that graha as a dasha lord. Reads
   * the dasha phala section, so it is computed for you when asked.
   */
  readonly dashaPhala?: boolean;
  /**
   * What the Ashtakavarga says: each graha's bindus in the sign it stands
   * in, and each sign's sarvashtakavarga. The two numbers a text quotes
   * and no verdict, because the reading carries no threshold. Reads the
   * Ashtakavarga section and the chart's own placements, so both are
   * computed for you when asked.
   */
  readonly ashtakavarga?: boolean;
  /**
   * What a loaded corpus says of Saturn's periods from the natal Moon: the
   * reading of each Sade Sati phase and smaller spell the chart's report
   * holds, each house once, in the order Saturn first reaches it. It says
   * the report `sadeSati` on the chart request finds, so it needs that
   * window beside it, and like `phala` it says nothing until a pack of
   * state readings is loaded.
   */
  readonly sadeSati?: boolean;
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
  readonly place: ChartPlace;
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
  readonly place: ChartPlace;
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
  readonly synastryDavison: DavisonBirth | null;
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
  readonly place: ChartPlace;
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
   * a following rule when it has an `after`, else a festival rule. A later
   * rule replaces an earlier one with its key.
   */
  readonly rules:
    | FestivalPack
    | readonly (FestivalPack | FestivalRule | EkadashiRule | FollowingRule)[];
}

/**
 * A pack of festival rules the SDK ships: *Dharmasindhu*'s, or those as
 * Nepal's national panchanga keeps them (a rite of the daylight on the day
 * whose sunrise holds its tithi) with the days it counts from them
 * (`03-design/festival-rules.md` §9.4–9.5).
 */
export type FestivalPack = 'DHARMASINDHU' | 'NEPAL';

/**
 * An observance on the day a number of civil days after another rule's
 * (`03-design/festival-rules.md` §9.4): the Terai's Holi, the day after
 * the Holika fire.
 */
export interface FollowingRule {
  /** Its key in its pack, `'HOLI_TERAI'`; an observance is named by it. */
  readonly key: string;
  /** Where the rule is stated. */
  readonly source: string;
  /** The key of the festival rule whose day it counts from; never another following rule. */
  readonly after: string;
  /** How many civil days after that day, 0 for the same day; at most 15. */
  readonly days: number;
}

/** A fifth of the daylight (*Dharmasindhu*, p. 6). */
export type FestivalDayPart = 'PRATAH' | 'SANGAVA' | 'MADHYAHNA' | 'APARAHNA' | 'SAYAHNA';

/** The time of a rite. */
export type FestivalWindow =
  | { readonly window: 'SUNRISE' }
  /** The instant of sunset, which begins the evening; Nepal's monthly full-moon fast is judged there (C200). */
  | { readonly window: 'SUNSET' }
  | { readonly window: 'PART'; readonly part: FestivalDayPart }
  | { readonly window: 'PRADOSHA' }
  | { readonly window: 'NISHITHA' }
  /** A fifteenth of the night counted from sunset, 1 to 15; Shivaratri's niśītha is the 8th. */
  | { readonly window: 'NIGHT_MUHURTA'; readonly muhurta: number };

/** One of the tithi's two days. */
export type FestivalWhich = 'EARLIER' | 'LATER';

/** How a tithi held the rite's time on its two days. */
export type FestivalCase =
  | 'EARLIER_ONLY'
  | 'LATER_ONLY'
  | 'BOTH'
  | 'NEITHER'
  | 'EQUAL_PARTS'
  | 'UNEQUAL_PARTS';

/** Something a guard asks of the two days. */
export type FestivalPredicate =
  | { readonly is: 'CASE'; readonly case: FestivalCase }
  | { readonly is: 'JOINED'; readonly day: FestivalWhich; readonly nakshatra: Nakshatra; readonly at?: FestivalWindow }
  | { readonly is: 'STANDS'; readonly day: FestivalWhich; readonly nakshatra: Nakshatra; readonly at: FestivalWindow }
  | { readonly is: 'LASTS'; readonly day: FestivalWhich; readonly from: 'SUNRISE' | 'SUNSET'; readonly ghatis: number }
  /** The tithi holds the whole of that day's window, or its instant. */
  | { readonly is: 'WHOLLY'; readonly day: FestivalWhich };

/** What a guard, or a rule's `otherwise`, decides. */
export type FestivalChoice = 'EARLIER' | 'LATER' | 'BY_YUGMA';

/** What a rule is kept on: a tithi, or a nakshatra in a paksha of the month (the Samavedis' upakarma is Hasta in Bhadrapada's bright half). */
export type FestivalOccurs =
  | {
      /** The tithi; its paksha is the tithi's. */
      readonly tithi: Tithi;
      readonly nakshatra?: never;
      readonly paksha?: never;
    }
  | {
      readonly tithi?: never;
      /** The nakshatra; judged between its two days as a tithi is, in the month and paksha of the tithi running at its middle. */
      readonly nakshatra: Nakshatra;
      /** The half of the month it must fall in. */
      readonly paksha: Paksha;
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
  /** Whether an adhika month holds it too; only the nija month by default (C171). */
  readonly inAdhika?: boolean;
  /** The time of the rite. */
  readonly at: FestivalWindow;
  /** The guards, in order; the first whose predicates all hold decides. */
  readonly decide: readonly { readonly when: readonly FestivalPredicate[]; readonly choose: FestivalChoice }[];
  /** What decides when no guard holds. */
  readonly otherwise: FestivalChoice;
}

/** Where the 10th pierces the 11th's day: four ghatis before sunrise (the Vaishnavas'), or at sunrise (the Smartas'). */
export type EkadashiVedha = 'ARUNODAYA' | 'SUNRISE';

/** Which of the 11th and the 12th hold the sunrise after their own day. */
export type EkadashiExcess = 'ELEVENTH' | 'TWELFTH' | 'BOTH' | 'NEITHER';

/** The day each of the four kinds takes: the 11th's own, or the day after. */
export interface EkadashiKinds {
  readonly eleventh: FestivalWhich;
  readonly twelfth: FestivalWhich;
  readonly both: FestivalWhich;
  readonly neither: FestivalWhich;
}

/** Whose Ekadashi fast, and by what (`03-design/festival-rules.md` §8.3). */
export interface EkadashiRule {
  /** Its key in its pack, `'EKADASHI_SMARTA'`; a fast is named by it. */
  readonly key: string;
  /** Where the rule is stated. */
  readonly source: string;
  /** Where the 10th pierces. */
  readonly vedha: EkadashiVedha;
  /** The day each kind takes, the 11th pure and pierced. */
  readonly table: { readonly pure: EkadashiKinds; readonly pierced: EkadashiKinds };
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
  /** The table's day, which `day` resolves. */
  readonly choice: FestivalWhich;
  /** The fast. */
  readonly day: CalendarDate;
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
}

/** A solar eclipse and the place's view of it. */
export interface SolarEclipseHere {
  /** The eclipse, the same everywhere. */
  readonly eclipse: SolarEclipse;
  /** The place's own contacts and magnitude, or `null` where the Moon's disc never touches the Sun's from it. */
  readonly here: SolarEclipseView | null;
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
  /** What computed it: the widened days among the applied conventions as `festival.days`, and the hash of this value. */
  readonly provenance: Provenance;
}

/**
 * A muhurta search to run over an almanac's days
 * (`03-design/muhurta-at-the-boundary.md`); only `rules` is required. A
 * catalogue member may be written bare (`'ROHINI'`) or in full
 * (`'nakshatra.ROHINI'`), so a clause read back from an answer can be
 * handed straight into rules.
 */
export interface MuhurtaRequest {
  /** The activity's rules: a set the SDK ships, by name, or rules spelt out. */
  readonly rules: MuhurtaActivity | ActivityRules;
  /** The native whose tarabala, chandrabala and ashtama lagna are read. */
  readonly native?: MuhurtaNative;
  /** How the windows are ordered; `'TEXTS'` by default (C162). */
  readonly ranking?: MuhurtaRanking;
  /** How many of the best days are cut into windows; 7 by default. */
  readonly daysWithWindows?: number;
  /** How many windows are answered at most; 50 by default. */
  readonly most?: number;
  /** How Venus's and Jupiter's combustion is seen: named, or spelt out; the Surya Siddhanta's by default (C164). */
  readonly asta?: AstaName | AstaCriterion;
}

/** A set of rules the SDK ships. */
export type MuhurtaActivity =
  | 'RAMAN_MARRIAGE'
  | 'BASELINE_MARRIAGE'
  | 'RAMAN_NAMAKARANA'
  | 'RAMAN_ANNAPRASANA'
  | 'RAMAN_UPANAYANA'
  | 'RAMAN_GRIHA_PRAVESHA';

/** A visibility criterion the SDK names. */
export type AstaName = 'SURYA_SIDDHANTA' | 'COMBUSTION_ORB' | 'PTOLEMY';

/** A visibility criterion spelt out: what it measures, against which thresholds. */
export interface AstaCriterion {
  readonly kind: 'TIME_DEGREES' | 'LONGITUDE' | 'ARCUS_VISIONIS';
  readonly thresholds:
    | { readonly kind: 'SURYA_SIDDHANTA' | 'PTOLEMY' }
    | ({ readonly kind: 'CUSTOM' } & {
        readonly [body in 'moon' | 'mercury' | 'venus' | 'mars' | 'jupiter' | 'saturn' | 'uranus' | 'neptune' | 'pluto']?: {
          readonly direct: number;
          readonly retrograde: number;
        } | null;
      });
}

/** Whose day a search reads: the birth star and Moon sign, and the birth lagna when the time is known. */
export interface MuhurtaNative {
  readonly star: Nakshatra;
  readonly moonSign: Rashi;
  readonly lagna?: Rashi | null;
}

/** How a search orders its windows: by the texts' clauses, or by the baseline engine's weights. */
export type MuhurtaRanking = 'TEXTS' | 'BASELINE';

/** How a member is graded. */
export type MuhurtaGrade = 'BEST' | 'MIDDLING' | 'REJECTED';

/** A list's members by grade, and the grade of any it does not name. */
export interface Graded<T> {
  readonly best: readonly T[];
  readonly middling: readonly T[];
  readonly rejected: readonly T[];
  readonly otherwise: MuhurtaGrade;
}

/** An activity's rules: what a search judges a time by (`03-design/muhurta.md`). */
export interface ActivityRules {
  /** How the day's limbs and vara are graded, and the houses Chandrabala avoids. */
  readonly day: {
    readonly tithis: Graded<Tithi>;
    readonly nakshatras: Graded<Nakshatra>;
    readonly yogas: Graded<Yoga>;
    readonly karanas: Graded<Karana>;
    readonly varas: Graded<Vara>;
    readonly chandrabala: { readonly avoid: readonly number[] };
  };
  /** Which months the rite is permitted in, by the lunar month or the Sun's sign (C161). */
  readonly months:
    | { readonly reckoning: 'ANY' }
    | {
        readonly reckoning: 'LUNAR';
        readonly months: Graded<Masa>;
        readonly withSun: readonly { readonly masa: Masa; readonly sun: Rashi }[];
      }
    | { readonly reckoning: 'SOLAR'; readonly signs: Graded<Rashi> };
  /** The lagnas, graded. */
  readonly lagnas: Graded<Rashi>;
  /** The nakshatra quarters to reject. */
  readonly padas: readonly { readonly nakshatra: Nakshatra; readonly pada: number }[];
  /** The seasons that close a day. */
  readonly heeds: readonly BlackoutKind[];
  /** The grahas the rite wants out of houses; none when left out. */
  readonly unwanted?: readonly MuhurtaUnwanted[];
  /** What bars a time outright: every clause of a kind, or one clause. */
  readonly bars: readonly MuhurtaBar[];
  /** What the source asks that the SDK does not judge. */
  readonly unjudged: readonly MuhurtaUnjudged[];
  /** The baseline engine's event, which the `'BASELINE'` ranking reads. */
  readonly baseline: MuhurtaBaselineEvent | null;
}

/** The baseline engine's marriage event: what its weights read. */
export interface MuhurtaBaselineEvent {
  readonly stars: readonly Nakshatra[];
  readonly favouredVaras: readonly Vara[];
  readonly avoidedVaras: readonly Vara[];
  readonly favouredTithis: readonly Tithi[];
  readonly karakas: readonly Graha[];
  readonly seventhEmpty: boolean;
  readonly abhijitForbidden: boolean;
}

/** Grahas a rite wants out of houses, counted 1 to 12 by sign from the lagna; every graha for a house that should be unoccupied. */
export interface MuhurtaUnwanted {
  readonly grahas: readonly Graha[];
  readonly houses: readonly number[];
  /** Whether a graha there bars the rite (the texts' "must") rather than weighs against it ("should"); `false` when left out (C205). */
  readonly bars?: boolean;
}

/** What bars a time: every clause of a kind, by its key, or one clause exactly. */
export type MuhurtaBar = MuhurtaClauseKey | MuhurtaClauseKind;

/** Something the source asks that the SDK does not judge yet. */
export interface MuhurtaUnjudged {
  readonly what: string;
  readonly why: string;
}

/** A tara: the birth star's count to the day's, by nine. */
export type Tara =
  | 'JANMA'
  | 'SAMPAT'
  | 'VIPAT'
  | 'KSHEMA'
  | 'PRATYAK'
  | 'SADHANA'
  | 'NAIDHANA'
  | 'MITRA'
  | 'PARAMA_MITRA';

/** A kind of clause: the `clause` tag of each. */
export type MuhurtaClauseKey = MuhurtaClauseKind['clause'];

/** One named condition from a source, without when it held: a discriminated union over `clause`. */
export type MuhurtaClauseKind =
  | { readonly clause: 'TITHI'; readonly tithi: Tithi; readonly grade: MuhurtaGrade }
  | { readonly clause: 'NAKSHATRA'; readonly nakshatra: Nakshatra; readonly grade: MuhurtaGrade }
  | { readonly clause: 'YOGA'; readonly yoga: Yoga; readonly grade: MuhurtaGrade }
  | { readonly clause: 'KARANA'; readonly karana: Karana; readonly grade: MuhurtaGrade }
  | { readonly clause: 'VARA'; readonly vara: Vara; readonly grade: MuhurtaGrade }
  | { readonly clause: 'MONTH'; readonly masa: Masa; readonly grade: MuhurtaGrade }
  | { readonly clause: 'SOLAR_MONTH'; readonly sign: Rashi; readonly grade: MuhurtaGrade }
  | { readonly clause: 'LAGNA'; readonly sign: Rashi; readonly grade: MuhurtaGrade }
  | { readonly clause: 'PADA'; readonly pada: { readonly nakshatra: Nakshatra; readonly pada: number } }
  | { readonly clause: 'KAALA'; readonly kaala: Kaala }
  | { readonly clause: 'CHOGHADIYA'; readonly choghadiya: Choghadiya }
  | { readonly clause: 'ABHIJIT' }
  | { readonly clause: 'MUHURTA_YOGA'; readonly yoga: MuhurtaYoga }
  | {
      readonly clause: 'TARABALA';
      readonly reading: { readonly count: number; readonly tara: Tara; readonly cycle: number };
    }
  | { readonly clause: 'CHANDRABALA'; readonly house: number; readonly holds: boolean }
  | { readonly clause: 'KARTARI'; readonly second: readonly Graha[]; readonly twelfth: readonly Graha[] }
  | { readonly clause: 'MOON_IN_DUSTHANA'; readonly house: number }
  | { readonly clause: 'MOON_JOINED'; readonly with: readonly Graha[] }
  | { readonly clause: 'VENUS_IN_SIXTH' }
  | { readonly clause: 'MARS_IN_EIGHTH' }
  | { readonly clause: 'ASHTAMA_LAGNA' }
  | { readonly clause: 'KUNAVAMSA'; readonly navamsa: Rashi; readonly lord: Graha }
  | { readonly clause: 'PANCHAKA_REMAINDER'; readonly panchaka: Panchaka }
  | { readonly clause: 'LAGNA_TYAJYA'; readonly sign: Rashi }
  | { readonly clause: 'SEVENTH_OCCUPIED'; readonly by: readonly Graha[] }
  | { readonly clause: 'MALEFIC_IN_LAGNA'; readonly grahas: readonly Graha[] }
  | { readonly clause: 'BENEFIC_IN_LAGNA'; readonly grahas: readonly Graha[] }
  | { readonly clause: 'EXALTED_IN_LAGNA'; readonly grahas: readonly Graha[] }
  | { readonly clause: 'LUMINARY_IN_ELEVENTH'; readonly grahas: readonly Graha[] }
  | { readonly clause: 'KENDRA_BENEFICS'; readonly grahas: readonly Graha[] }
  | { readonly clause: 'UNWANTED_PLACEMENT'; readonly house: number; readonly by: readonly Graha[] };

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
  readonly considerations?: ConsiderationRequest;
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
   * Whether to place Uranus, Neptune and Pluto beside the nine, in
   * `chart.outer`; false by default. A progression's later charts and a
   * hit list's natal points then reach them too.
   */
  readonly outerPlanets?: boolean;
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
 * One entry of an ephemeris chain: one of the SDK's own by name, or an
 * adapter's descriptor.
 *
 * `BUILTIN` is the analytic ephemeris the SDK carries, which needs no
 * files, no network and no licence beyond the SDK's own; `TEST` is the
 * test provider, whose positions are **not astronomy**. Those are the
 * **fallback** — in most cases a consumer plugs a real engine.
 */
export type EphemerisChoice = Ephemeris | PluginEphemeris;

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
