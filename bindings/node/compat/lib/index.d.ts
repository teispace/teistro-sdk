// `@teistro/compat`'s declarations.
//
// HAND-WRITTEN, like `index.js`. The birth record's type is written from
// the rows an application on the baseline engine stores
// (`docs/03-design/compat-shim.md` §3.7), here and nowhere else; every
// answer is the SDK's own type.

import type {
  Almanac,
  AlmanacDay,
  Ayanamsha,
  BaselineDashaRules,
  BaselinePeriod,
  Calendar,
  CalendarArea,
  CalendarDate,
  Chart,
  ChartLayout,
  ChartTiming,
  Charts,
  CivilDateTime,
  Context,
  ContextInit,
  DossierRequest,
  GeoPlace,
  GrahaName,
  HitRequest,
  Karana,
  LalKitabRequest,
  LayoutKey,
  LifeEvent,
  LunarYears,
  MuhurtaRequest,
  NaamMilan,
  NaamRules,
  NameChecked,
  NamingRules,
  Nakshatra,
  NepalSambatDates,
  NumerologyProfile,
  NumerologyRequestRules,
  PakshiDays,
  PakshiRequest,
  PlanRequest,
  Positions,
  PositionsRequest,
  PrashnaQuestion,
  ProgressionsRequest,
  Rashi,
  RashifalSealed,
  Rendered,
  ResearchCounts,
  ResearchTested,
  RuleRequest,
  SarvatobhadraRequest,
  SynastryRequest,
  Tithi,
  Varga,
  Yoga,
  ZoneResolution,
  ZoneSpec,
} from '@teistro/sdk';

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
  /** Read only by rectification, as its `sex`; `other` is carried as none. */
  readonly gender?: 'male' | 'female' | 'other';
  /** Read only by rectification, as its `accuracy`; every other call founds at the instant. */
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

/** A stored birth record's place and clock, where a call reads the days at another one. */
export interface PlaceAndClock {
  readonly place: GeoPlace;
  /** Seconds east of UTC. */
  readonly utcOffsetSeconds: number;
}

/** A limb span of a day with its whole bounds: where it began, the previous change, and where it ends, the next. */
export interface Change<T> {
  readonly member: T | 'unknown';
  readonly from: number;
  readonly to: number;
}

/** A row the shim refuses: it throws `CompatRefused` before any computation. */
export type Refused = (...args: readonly unknown[]) => never;

/** A request's own options, the input the shim maps left out. */
type Study<K extends 'counts' | 'compare' | 'expected'> = Omit<Parameters<Context['research'][K]>[0], 'births'>;

/**
 * The shim over one set of context options, holding the contexts it opens.
 * Every chart row takes a stored birth record and the stored snapshot it
 * was computed under, and answers the SDK's own result; the KP row answers
 * on the snapshot's KP context, the muhurta row on its muhurta context
 * (the frame the baseline searches in), every other on its natal one. A refused row is
 * typed `never`, so a compiler marks the code after its call unreachable.
 */
export interface Shim {
  // §3.1, the core package.

  /** Positions over a grid, on the snapshot's context. */
  positions(snapshot: StoredSnapshot, request: PositionsRequest): Positions;
  /** One of the engine's own operations, on the snapshot's context: a Teimeris chain's. */
  engine(snapshot: StoredSnapshot, name: string, args?: Record<string, unknown>): unknown;
  /** The ayanamsha at the birth, or at `instant`: the member and its offset in degrees. */
  ayanamsha(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options?: { readonly instant?: number },
  ): { readonly ayanamsha: Ayanamsha | 'unknown' | null; readonly offsetDeg: number; readonly custom: boolean };
  /** The sunrise, sunset and next sunrise of `date`, the birth's own by default, at the birth's place. */
  sunrise(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options?: { readonly date?: CalendarDate },
  ): { readonly date: CalendarDate; readonly sunrise: number; readonly sunset: number; readonly nextSunrise: number };
  /** A civil time in a zone, resolved on the snapshot's context. */
  resolve(snapshot: StoredSnapshot, civil: CivilDateTime, zone: ZoneSpec): ZoneResolution;
  /** Refused: the SDK has no geo port; give the zone on the birth record. */
  zoneAt: Refused;
  /** The birth's ghati-pala under both reckonings, each a chart's timing by the reckoning's name. */
  ghatiPala(birth: BirthRecord, snapshot: StoredSnapshot): { readonly CIVIL: ChartTiming; readonly PROPORTIONAL: ChartTiming };
  /** The calendars on the snapshot's context: the Bikram Sambat's conversions, month lengths and leap years among them. */
  bs(snapshot: StoredSnapshot): CalendarArea;
  /** The lunar years and the Nepal Sambat date of `date`, the birth's own by default. */
  eras(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options?: { readonly date?: CalendarDate },
  ): { readonly years: LunarYears | null; readonly nepalSambat: NepalSambatDates | null };
  /** A catalogue entity's names in every form, by its key (`graha.SUN`), in the snapshot's locale. */
  entity(snapshot: StoredSnapshot, key: string): ReturnType<Context['intl']['entity']>;
  /** Refused: formatting numbers and instants is the application's. */
  format: Refused;

  // §3.2, the chart package.

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
  /** The houses and the Bhava-Chalit. */
  houses(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The avasthas, dignities and planetary war. */
  states(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The drishti. */
  aspects(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The upagrahas and special lagnas. */
  points(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The Jaimini slice. */
  jaimini(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The Ayurdaya under the baseline's readings, the marakas and the longevity windows. */
  longevity(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The birth's timing: ishtakaal, bhayat and bhabhoga. */
  timing(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The avakahada. */
  avakahada(birth: BirthRecord, snapshot: StoredSnapshot): Chart;
  /** The birth's civil date in `into`, the Bikram Sambat by default. */
  birthCalendar(birth: BirthRecord, snapshot: StoredSnapshot, options?: { readonly into?: Calendar }): CalendarDate;
  /** The dossier: `preset`'s sections, or `sections`, read at `at` if given. */
  dossier(birth: BirthRecord, snapshot: StoredSnapshot, options?: DossierRequest): Chart;
  /** Refused: the gochar sidecar is Rust only; ask for the transits on the chart request. */
  sidecar: Refused;
  /** The chart's drawings. */
  drawing(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: { readonly drawings: readonly { readonly layout: ChartLayout | LayoutKey; readonly varga: Varga }[] },
  ): Chart;
  /** What the composers `interpret` names say, each plan said in the snapshot's locale, by section, beside the chart. */
  narrative(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: { readonly interpret: PlanRequest; readonly rules?: RuleRequest },
  ): { readonly chart: Chart; readonly text: Readonly<Record<string, readonly string[]>> };

  // §3.3, the panchanga package.

  /** The almanac day of `date`, the birth's own civil date by default, at the birth's place on its zone's clock at noon. */
  day(birth: BirthRecord, snapshot: StoredSnapshot, options?: { readonly date?: CalendarDate }): AlmanacDay;
  /** The almanac days `from` to `to`, both included, at the birth's place on its zone's clock at noon on `from`. */
  month(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly from: CalendarDate; readonly to: CalendarDate }): Almanac;
  /** The limb spans of `date`, the birth's own by default, each with where it began and where it ends. */
  changes(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options?: { readonly date?: CalendarDate },
  ): {
    readonly tithi: readonly Change<Tithi>[];
    readonly nakshatra: readonly Change<Nakshatra>[];
    readonly yoga: readonly Change<Yoga>[];
    readonly karana: readonly Change<Karana>[];
  };
  /**
   * A muhurta search over `from` to `to` under the baseline's ranking, at
   * the birth's place unless `at` gives another; the natal tara and
   * chandra bala read the birth's Moon unless `search.native` names
   * another native, or is `null` for none.
   */
  muhurta(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: {
      readonly from: CalendarDate;
      readonly to: CalendarDate;
      readonly search: MuhurtaRequest;
      readonly at?: PlaceAndClock;
    },
  ): Almanac;
  /** The Sarvatobhadra chakra at the birth and at `instants`. */
  sarvatobhadra(birth: BirthRecord, snapshot: StoredSnapshot, options: SarvatobhadraRequest): Chart;
  /** The almanac days `from` to `to` with their eclipses, at the birth's place. */
  eclipses(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly from: CalendarDate; readonly to: CalendarDate }): Almanac;
  /** The Sun's sign ingresses from `from` to `to`: each day's sankranti, its instant and the sign entered. */
  ingresses(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: { readonly from: CalendarDate; readonly to: CalendarDate },
  ): readonly { readonly date: CalendarDate; readonly instant: number; readonly into: Rashi | 'unknown' | undefined }[];
  /** The full and new Moons from `from` to `to`: the ends of the 15th and 30th tithis. */
  lunations(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: { readonly from: CalendarDate; readonly to: CalendarDate },
  ): readonly { readonly instant: number; readonly tithi: Tithi }[];
  /** The Pancha Pakshi days `from` to `to` for `native`, at the birth's place. */
  pakshi(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: {
      readonly from: CalendarDate;
      readonly to: CalendarDate;
      readonly native: PakshiRequest['native'];
      readonly rules?: PakshiRequest['rules'];
    },
  ): PakshiDays;

  // §3.4, the predictive package.

  /** The transits at `instants`, UT Julian days, over the chart, with the Ashtakavarga's say. */
  gochar(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly instants: ArrayLike<number> }): Chart;
  /** Sade Sati and the dhaiyyas from `from` to `to`, UT Julian days. */
  sadeSati(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly from: number; readonly to: number }): Chart;
  /** The transit calendar: the hits `hits` asks for. */
  transitCalendar(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly hits: HitRequest }): Chart;
  /** The annual charts through year `through` of life, with their matters, sahams and the Mudda dasha. */
  varsha(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly through: number }): Chart;
  /** The returns from `from` to `to`, UT Julian days: the Moon's unless `grahas` names others. */
  returns(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: { readonly from: number; readonly to: number; readonly grahas?: readonly GrahaName[] },
  ): Chart;
  /** The progressions and directions `options` asks for. */
  progressions(birth: BirthRecord, snapshot: StoredSnapshot, options?: ProgressionsRequest): Chart;

  // §3.5, the applied package.

  /**
   * KP's cusps, lords, significators and ruling planets, on the KP context;
   * with `anyAyanamsha`, on the natal context under the chart's own ayanamsha.
   */
  kp(birth: BirthRecord, snapshot: StoredSnapshot, options?: { readonly anyAyanamsha?: boolean }): Chart;
  /** The birth as the groom, matched with `partner` as the bride. */
  milan(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly partner: BirthRecord }): Chart;
  /** The stored names matched star to star: `partner`'s as the bride's, the birth's as the groom's. */
  naam(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly partner: BirthRecord; readonly rules?: NaamRules }): NaamMilan;
  /** The rectification the baseline runs, the record's gender and time accuracy carried in. */
  rectify(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: {
      readonly uncertaintyMinutes: number;
      readonly events?: readonly LifeEvent[];
      readonly coverage?: number;
      readonly dasha?: BaselineDashaRules;
    },
  ): Chart;
  /** The prashna cast at the record's instant and place as the baseline casts it, from the Earth's centre with the true node (C800), under every reading it takes. */
  prashna(birth: BirthRecord, snapshot: StoredSnapshot, options?: { readonly question?: PrashnaQuestion }): Chart;
  /** The Lal Kitab reading, with what `options` asks beyond it. */
  lalkitab(birth: BirthRecord, snapshot: StoredSnapshot, options?: LalKitabRequest): Chart;
  /** The stored name and the birth's civil date, under the baseline's readings unless `rules` names others. */
  numerology(birth: BirthRecord, snapshot: StoredSnapshot, options?: { readonly rules?: NumerologyRequestRules }): NumerologyProfile;
  /** The remedies under the baseline's functional nature, read at `at` if given. */
  remedies(birth: BirthRecord, snapshot: StoredSnapshot, options?: { readonly at?: number }): Chart;
  /** Refused: gemstone safety is not built. */
  gemstoneSafety: Refused;
  /** `names` checked against the birth's star and pada, read from its avakahada. */
  nameCheck(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: { readonly names: readonly string[]; readonly rules?: NamingRules },
  ): readonly NameChecked[];
  /** Refused: no name corpus ships; load the application's own book with `naming.load`. */
  nameSuggestions: Refused;
  /** Whole charts at `instants`, UT Julian days, at the birth's place and on its clock. */
  batch(birth: BirthRecord, snapshot: StoredSnapshot, options: { readonly instants: ArrayLike<number> }): Charts;
  /** The synastry with `partner`'s stored birth record. */
  synastry(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: Omit<SynastryRequest, 'partner'> & { readonly partner: BirthRecord },
  ): Chart;
  /** Whether the groups of a study over stored birth records differ on each rule. */
  research(births: readonly BirthRecord[], snapshot: StoredSnapshot, study: Study<'compare'>): ResearchTested;
  /** Whether each rule is commoner than chance over stored birth records. */
  research(births: readonly BirthRecord[], snapshot: StoredSnapshot, study: Study<'expected'>): ResearchTested;
  /** How often each rule holds in each group of a study over stored birth records. */
  research(births: readonly BirthRecord[], snapshot: StoredSnapshot, study: Study<'counts'>): ResearchCounts;

  // §3.6, the rashifal and interpretation packages.

  /** The rashifal of every sign from `first` to `last`, at the birth's place, read at six on the clock with the baseline's event grahas for `period`. */
  rashifal(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: { readonly first: CalendarDate; readonly last?: CalendarDate; readonly period?: BaselinePeriod },
  ): RashifalSealed;
  /** One message said in the snapshot's locale. */
  reading(snapshot: StoredSnapshot, key: string, params?: Record<string, unknown>): Rendered;
  /** The milan with `partner` as the bride and its doshas' readings said, beside the chart. */
  milanVerdict(
    birth: BirthRecord,
    snapshot: StoredSnapshot,
    options: { readonly partner: BirthRecord },
  ): { readonly chart: Chart; readonly text: Readonly<Record<string, readonly string[]>> };

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

/** How two values of a family of facts compare: by their difference in a unit, or by equality. */
export interface Family {
  readonly kind: 'angle' | 'number' | 'category';
  readonly unit?: string;
  readonly scale?: number;
  readonly tolerance?: number;
  readonly what: string;
}

/** The families of facts, by name: a fact's key is `family/what`. */
export declare const FAMILIES: Readonly<Record<string, Family>>;

/**
 * An answer's facts, `[family/what, value]`: the fields both engines
 * answer, in the vocabulary the A/B reads both sides in. `request` names
 * the row the answer came from. `figures` lists the yogas to say present
 * or absent, each led by the SDK's rule key; `window` is the range a
 * `sadeSati` answer was asked over. A `rashifal` answer is a list of
 * `{ period, answer }`, each period's `rashifal` with the baseline's score.
 */
export declare function facts(
  request:
    | 'chart'
    | 'full'
    | 'vargas'
    | 'yogas'
    | 'strength'
    | 'dashas'
    | 'day'
    | 'month'
    | 'milan'
    | 'varsha'
    | 'kp'
    | 'gochar'
    | 'sadeSati'
    | 'batch'
    | 'muhurta'
    | 'prashna'
    | 'rashifal',
  answer: unknown,
  options?: {
    readonly figures?: readonly (readonly [string, ...unknown[]])[];
    readonly window?: { readonly from: number; readonly to: number };
  },
): [string, unknown][];

/** The shim, over the options every context it opens shares. */
export declare function compat(options?: CompatOptions): Shim;
