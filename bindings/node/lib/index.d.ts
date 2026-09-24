// The ergonomic layer's declarations.
//
// HAND-WRITTEN, like `index.js` itself. Everything it names is generated:
// the enums and their tables (`catalogue.d.ts`), the boundary's value
// types (`types.d.ts`) and the decoded result blobs (`blob.d.ts`). What
// this file adds is the shape of the layer: the context, the results that
// decode on first use, and the error.

import type {
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
  Nakshatra,
  Paksha,
  Panchaka,
  Point,
  Quadrant,
  Rashi,
  Shodhana,
  Ekadhipatya,
  Vaiseshikamsa,
  VimshopakaScoring,
  DashaPhase,
  Nature,
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
  Affliction,
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
  | { readonly kind: 'line'; readonly to: UnitPoint }
  | { readonly kind: 'quad'; readonly control: UnitPoint; readonly to: UnitPoint }
  | {
      readonly kind: 'arc';
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
    readonly method: 'spatial' | 'temporal';
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
  | 'sidereal'
  /** The natal tropical longitude: the Western solar return. Forty years on it is most of a circle of lagna from the sidereal one, so it is a choice and never a fallback. */
  | 'tropical'
  /** A whole sidereal year for each year of life, from birth: the older arithmetic, and the only reading that needs no ephemeris. */
  | 'mean';

/**
 * Where the Muntha stands inside the sign it has reached (crux C107).
 *
 * Both readings give the same sign at the return and part over the year,
 * so they differ for a Tajika aspect taken to the Muntha and nothing else.
 */
export type MunthaDegree =
  /** It enters each year at its sign's first degree and crosses the sign during the year: the source's own reading. */
  | 'sign_start'
  /** It carries the natal lagna's degree into each new sign. */
  | 'natal_degree';

/** The annual charts a request asks for. */
export interface VarshaRequest {
  /** Which longitude the Sun returns to; `sidereal` by default. */
  readonly reading?: VarshaReading;
  /** The last year of life wanted, 1 to 200. */
  readonly through: number;
  /** Where the Muntha stands inside its sign; `sign_start` by default. */
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
   * @example { through: 40, place: 'birth', sahams: ['punya', 'vivaha'] }
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
   * degree, `'sun_degrees'`, the source's own, so the year closes on the
   * next return; an `'even'` share of the time between the returns; or
   * `{ days: n }`, the whole year as so many civil days from the return,
   * the printed durations (360, and 365 for the Patyayini).
   */
  readonly clock?: 'sun_degrees' | 'even' | { readonly days: number };
  /**
   * Where a nakshatra year's balance comes from (crux C123): what remained
   * of the birth Moon's nakshatra, `'natal_moon'`, the source's own; the
   * Moon's at the return, `'entry_moon'`; or `'whole'`, none.
   */
  readonly balance?: 'natal_moon' | 'entry_moon' | 'whole';
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
     * return: the Patyayini, and a `'whole'` balance.
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
   * Which planets are benefic and malefic: the `'chapter'`'s own, the Sun
   * a malefic among them, or the catalogue's `'parashari'` natures.
   */
  readonly natures?: 'chapter' | 'parashari';
  /** Tajika's `'positional'` friendship, the source's, or the catalogue's `'natural'` one. */
  readonly friendship?: 'positional' | 'natural';
  /** The Vishwa bala below which a saham's lord is weak, in **sub-sub units**: `5 * 3600` by default. */
  readonly weakBelow?: number;
}

/** Where the sources differ on the Harsha bala (`03-design/tajika-harsha.md`). */
export interface HarshaRules {
  /** Venus's house of joy: the verse's `'fifth'`, or the `'twelfth'` a widely used program reads. */
  readonly venus?: 'fifth' | 'twelfth';
}

/** Where the sources differ on a saham, each a named reading (`03-design/tajika-sahams.md`). */
export interface SahamRules {
  /**
   * When a saham is carried a sign further: when c does not fall between b
   * and a by `'degrees'`, the source's own; by whole `'signs'`, as a widely
   * used program reads it; or `'never'`.
   */
  readonly addSign?: 'degrees' | 'signs' | 'never';
  /**
   * Where a house's point stands: `'sripati'`'s mid-point built from the
   * angles, the source's own; the chart's `'chalit'` under its profile; or
   * `'equal'` houses from the lagna's degree.
   */
  readonly houses?: 'sripati' | 'chalit' | 'equal';
  /** Roga's formula: lagna − Moon + lagna, `'lagna'`, or the other authority's `'saturn'`. */
  readonly roga?: 'lagna' | 'saturn';
}

/** Where the source leaves the sixteen Tajika yogas a choice, each a named reading. */
export interface YogaRules {
  /** How a pair less than a degree past reads; `'poorna'` by default (crux C112). */
  readonly drishti?: { readonly subDegree?: 'poorna' | 'ishrafa' };
  /**
   * The strength below which a planet with no dignity is weak, in **sub-sub
   * units**, 3600 to a unit: `5 * 3600` by default (crux C116).
   */
  readonly weakBelow?: number;
  /** The strength from which a planet is strong, sub-sub units; `10 * 3600` by default. */
  readonly strongFrom?: number;
  /**
   * Which lord a Tambira lets reach the next sign: the definition's
   * `'karyesha'` by default, or `'either_lord'`, the source's "some
   * authorities".
   */
  readonly tambira?: 'karyesha' | 'either_lord';
  /**
   * When the Moon counts among Kuttha's benefics: `'always'` by default,
   * Charak's list, or `'waxing'`, the commentary's "full Moon" (crux C117).
   */
  readonly moonBenefic?: 'always' | 'waxing';
}

/** Where the sources differ on the lord of the year, each a named reading. */
export interface VarsheshaRules {
  /**
   * Who takes the year when nobody aspects the lagna: the Muntha's lord by
   * default, the annual lagna's lord, or the Nilakanthi's strongest of the five.
   */
  readonly noneAspects?: 'muntha_lord' | 'annual_lagna_lord' | 'strongest';
  /** Who takes it on an outright tie; the Muntha's lord by default. */
  readonly tied?: 'muntha_lord' | 'dina_ratri_pati';
  /**
   * Whether the Moon may hold it: `'passed_over'` by default, stepping down
   * to the next claimant and else to its Ithasala successor; `'ithasala'`,
   * the Nilakanthi's successor at once; or `'like_any_other'`.
   */
  readonly moon?: 'passed_over' | 'ithasala' | 'like_any_other';
  /** Who may succeed the Moon: any planet by default, or only an office-bearer. */
  readonly moonPartner?: 'any_planet' | 'office_bearer';
  /** How the Ithasala the Moon's successor needs is read, as for the yogas. */
  readonly drishti?: { readonly subDegree?: 'poorna' | 'ishrafa' };
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

/** Which step of the chain decided the year's lord. */
export type VarsheshaChosen =
  | 'strongest'
  | 'most-portfolios'
  | 'muntha-lord-unaspected'
  | 'muntha-lord-all-weak'
  | 'muntha-lord-tied'
  | 'dina-ratri-tied'
  | 'annual-lagna-lord-unaspected';

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

/** The Tajika aspect between two signs; the neutral houses give none. */
export type TajikaDrishti =
  | 'friendly'
  | 'secretly-friendly'
  | 'inimical'
  | 'secretly-inimical'
  | 'none';

/**
 * What two planets inside each other's orb are doing.
 *
 * Three of the four are kinds of Ithasala, the coming-together, as the
 * source's Table X-3 enumerates them.
 */
export type TajikaYoga =
  /** Vartamana: the faster is behind the slower by a degree or more, and coming to it. */
  | 'ithasala-vartamana'
  /** Poorna: as Vartamana but within a single degree, so already fulfilled. */
  | 'ithasala-poorna'
  /** Bhavishyat: the faster is past but at the sign's end, so it acts from the next sign. */
  | 'ithasala-bhavishyat'
  /** Ishrafa: the faster is a degree or more past the slower and drawing away. */
  | 'ishrafa';

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
 *     kernel: 'udu',
 *     key: 'ACME_SAPTAKA',
 *     lords: ['SUN', 'MOON', 'MARS', 'MERCURY', 'JUPITER', 'VENUS', 'SATURN']
 *       .map((graha) => ({ graha, years: 10 })),
 *     reference: 'KRITTIKA',
 *   }, {
 *     kernel: 'rashi',
 *     key: 'ACME_STHIRA',
 *     length: { by_modality: { movable: 7, fixed: 8, dual: 9 } },
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
  readonly kernel: 'udu';
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
  | 'count_to_lord'
  | 'count_to_lord_by_dignity'
  | { readonly fixed: number }
  | { readonly by_modality: { readonly movable: number; readonly fixed: number; readonly dual: number } };

/**
 * A sign-based (Jaimini) dasha system of your own: its key, and optionally
 * where it starts, the order it visits the signs in, how long a sign runs,
 * which lord a mahadasha names, and the houses to start from the strongest
 * of. Everything unsaid is Chara's.
 */
export interface RashiDashaDefinition {
  /** The kernel that runs it: the twelve signs in an order, each for a number of years. */
  readonly kernel: 'rashi';
  /** Its key: `[A-Z][A-Z0-9_]`, at most 48 characters, and not one the catalogue has. */
  readonly key: string;
  /** Where the row comes from. */
  readonly sources?: readonly string[];
  /** Where it starts; the lagna by default. */
  readonly start?: 'lagna' | 'arudha_lagna' | 'navamsa_lagna';
  /** The order it visits the signs in; every sign in turn by default. */
  readonly order?: 'consecutive' | 'trine_groups' | 'drishti_chain' | 'leap';
  /** How long a sign's period runs; the count to its stronger lord by default. */
  readonly length?: RashiLength;
  /** Which lord a mahadasha names; the stronger of a dual-lorded sign's two by default. */
  readonly namedLord?: 'stronger' | 'first';
  /** The houses from the lagna to start from the strongest of; none by default. */
  readonly strongerOf?: readonly number[];
  /** The length of its year; `JULIAN_365_25` by default. */
  readonly year_length?: 'JULIAN_365_25' | 'SAVANA_360' | 'SIDEREAL' | 'TROPICAL' | 'LUNAR' | 'NAKSHATRA_324';
  /** How many levels of periods a reading carries, 1 to 6; three by default. */
  readonly depth?: number;
}

/** What a grid cell always carries: a sign, or a house 1 to 12. */
export type LayoutHolds =
  | { readonly kind: 'sign'; readonly value: RashiName }
  | { readonly kind: 'house'; readonly value: number };

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
  readonly counts_from: 'lagna' | 'moon' | 'sun' | 'cusps' | 'zodiac';
}

/** Twelve cells fixed in the row, or rings computed per chart. */
export type LayoutShape =
  | {
      readonly kind: 'grid';
      readonly cells: readonly LayoutCell[];
      readonly frame: readonly Outline[];
      readonly direction: 'clockwise' | 'anticlockwise';
    }
  | {
      readonly kind: 'radial';
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
  readonly body_form?: 'short' | 'glyph';
  /** What a cell's label shows; `auto` is the sign's number, or on a wheel the house and the sign's glyph. */
  readonly cell_label?: 'auto' | 'sign_number' | 'sign_short' | 'sign_glyph' | 'house' | 'nothing';
  /** Whether the lagna is written first in the cell it stands in. */
  readonly lagna_mark?: boolean;
  /** What is written after a retrograde graha's name, or null for nothing. */
  readonly retrograde_mark?: string | null;
  /** Whether a graha's degree follows its name, on the founded chart. */
  readonly degrees?: boolean;
}

/** A set of rules the SDK ships. */
export type ShippedRules = 'doshas' | 'yogas' | 'gandantas' | 'arishtas' | 'readings' | 'nabhasas';

/**
 * The rules a request asks a chart to answer (`03-design/rules-at-the-boundary.md`):
 * shipped sets by name and a consumer's own rules in the SDK's rule format.
 */
export interface RuleRequest {
  /** The shipped sets to evaluate. */
  readonly shipped?: readonly ShippedRules[];
  /** A consumer's own rules, which may name shipped rules by key. */
  readonly rules?: readonly Readonly<Record<string, unknown>>[];
  /** The readings to evaluate under; `'texts'` by default. */
  readonly readings?: 'texts' | 'recording-engine';
  /** Whether to add the twelve house readings. */
  readonly houses?: boolean;
  /** Whether to add the three pairs, the three spans and the marakas. */
  readonly longevity?: boolean;
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
}

/**
 * The theme a request writes its drawings as SVG in: a shipped theme's name,
 * or a record naming only what it changes (`03-design/render-svg.md`).
 */
export type Theme =
  | 'light'
  | 'dark'
  | {
      readonly extends?: 'light' | 'dark';
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
 * `builtin` is the analytic ephemeris the SDK carries, which needs no
 * files, no network and no licence beyond the SDK's own; `test` is the
 * test provider, whose positions are **not astronomy**. Those are the
 * **fallback** — in most cases a consumer plugs a real engine.
 */
export type EphemerisChoice = 'none' | 'builtin' | 'test' | PluginEphemeris;

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
