// The ergonomic layer's declarations, type-checked with the rest.
//
// Its own file so a change to `index.d.ts` fails here rather than in an
// application. Every `@ts-expect-error` is a proof, as in `consumer.ts`.

import type {
  Air,
  Antiscia,
  AntisciaRequest,
  AntiscionRow,
  Composite,
  CompositePlanet,
  CuspAntiscion,
  DavisonBirth,
  Declinations,
  MidpointRequest,
  MidpointRow,
  ParallelRequest,
  ParallelRow,
  SynastryMidpointRow,
  SynastryRequest,
  SynastryParallelRow,
  SynastryRow,
  WesternAspectRequest,
  WesternAspectRow,
  WesternHousePlacement,
  WesternHouseRequest,
  WesternHouses,
  AshtaKoota,
  NaamMilan,
  Avakahada,
  BirthSyllable,
  NaamRules,
  NumerologyProfile,
  NumerologyRules,
  NameSyllable,
  KootaReading,
  Kuja,
  KujaReading,
  MarriageDosha,
  Porutham,
  PoruthamReading,
  MatchingRequest,
  HarmonicChart,
  HarmonicPlaced,
  HarmonicRequest,
  HarmonicRow,
  Almanac,
  BlackoutKind,
  LunarEclipseKind,
  AlmanacDay,
  AnnualDashaSystem,
  Body,
  BuildInfo,
  Calendar,
  Chart,
  ChartBatchRequest,
  Charts,
  Context,
  DashaPeriod,
  EphemerisProvider,
  Eclipses,
  NepalSambatDate,
  NepalSambatDates,
  EkadashiKinds,
  EkadashiRule,
  EkadashiVedha,
  FestivalAnswer,
  FestivalPack,
  FestivalRequest,
  FestivalRule,
  FollowingRule,
  HitRequest,
  SadeSatiRequest,
  AccidentLine,
  Dignities,
  DignityKind,
  DignityRequest,
  FortitudeRequest,
  Fortitudes,
  LotPlace,
  LotRequest,
  ConsiderationRequest,
  Considerations,
  Perfection,
  PerfectionRequest,
  Matter,
  ProgressionsRequest,
  Progressions,
  ProgressedContact,
  NatalPoint,
  Motion,
  Application,
  ApplicationKind,
  Impediment,
  ImpedimentKind,
  Translation,
  Collection,
  Way,
  Lots,
  PlacedLot,
  Almuten,
  Almutens,
  AlmutenTotal,
  PlanetAccidents,
  GrahaName,
  KpLords,
  KpReading,
  KpRequest,
  Prashna,
  PrashnaRequest,
  AmatyaDevata,
  Devotion,
  Remedies,
  RemedyRequest,
  PlanetDignity,
  Reception,
  Term,
  LayoutHolds,
  LayoutKey,
  LayoutRow,
  LunarEclipseHere,
  MuhurtaAnswer,
  MuhurtaBar,
  MuhurtaRequest,
  MuhurtaUnwanted,
  PositionsRequest,
  RashiDashaDefinition,
  RuleRequest,
  Scale,
  SolarEclipseHere,
  SolarEclipseView,
  ShippedTheme,
  Theme,
} from '../lib/index.js';
import { ChartLayout, Point, Varga, altitude, latitude, longitude } from '../lib/catalogue.js';
import type { Ayanamsha, Gana, Graha, Masa, Saham, SahamStrong, SahamWeak } from '../lib/catalogue.js';
import type { CalendarDate, Confidence, PolarDay, Provenance, Step } from '../lib/index.js';
import { decodeProvenance, returnsRequest } from '../lib/index.js';

declare const build: BuildInfo;
declare function refuse(info: BuildInfo, named: boolean): string | null;

/** The build handshake, typed. */
function handshake(): string {
  const refusal: string | null = refuse(build, true);
  // @ts-expect-error a build is described, not guessed at
  const missing: number = build.commit;
  // @ts-expect-error the report is read, never edited
  build.sdk = '9.9.9';
  return `${build.sdk} ${String(missing)} ${refusal ?? 'taken'}`;
}

declare const ctx: Context;
declare const someDate: CalendarDate;

/** The whole scenario, typed. */
function scenario(): string {
  const request: PositionsRequest = {
    instants: [2451545, 2451546],
    bodies: ['graha.SUN' as unknown as Body, 'MOON'],
    speeds: true,
  };
  const positions = ctx.positions(request);
  const sun = positions.at(0, 0);
  const rendered = ctx.intl.render('sdk.reason.grahaInBhava', { bhava: 7 });
  const date = ctx.calendar.dateOf('calendar.GREGORIAN' as Calendar, 735702);
  const converted = ctx.calendar.convert(date, 'calendar.BIKRAM_SAMBAT' as Calendar);
  const delta = ctx.time.deltaT(2451544.5);
  const conversion = ctx.time.convert(2451544.5, 'UTC' as Scale, 'TT' as Scale);
  return [
    positions.bodies.join(','),
    positions.scale,
    sun.longitude.toFixed(4),
    rendered.text,
    `${converted.year}-${converted.month}-${converted.day}`,
    delta.seconds.toFixed(3),
    conversion.deltaTModel,
  ].join(' | ');
}

// @ts-expect-error a scale is a member, not a free string
const wrongScale: PositionsRequest = { instants: [], bodies: [], scale: 'gmt' };
// @ts-expect-error the settings hash is read-only
const write = () => { ctx.settingsHash = 'x'; };
const partialObserver: PositionsRequest = {
  instants: [],
  bodies: [],
  // @ts-expect-error an observer names all three of its parts
  observer: { longitudeDeg: 85.324 },
};
/** A place, as the boundary takes one. */
const place: PositionsRequest = {
  instants: [],
  bodies: [],
  observer: {
    latitudeDeg: latitude(27.7172),
    longitudeDeg: longitude(85.324),
    altitudeM: altitude(1400),
  },
};
const swappedPlace: PositionsRequest = {
  instants: [],
  bodies: [],
  observer: {
    // @ts-expect-error a longitude is not a latitude
    latitudeDeg: longitude(85.324),
    // @ts-expect-error and a latitude is not a longitude
    longitudeDeg: latitude(27.7172),
    altitudeM: altitude(1400),
  },
};
const barePlace: PositionsRequest = {
  instants: [],
  bodies: [],
  observer: {
    // @ts-expect-error a latitude is made by `latitude()`, never by a literal
    latitudeDeg: 27.7172,
    longitudeDeg: longitude(85.324),
    altitudeM: altitude(1400),
  },
};
const writeCell = () => {
  // @ts-expect-error a cell is read-only, like every result
  ctx.positions({ instants: [], bodies: [] }).at(0, 0).longitude = 0;
};

export { partialObserver, scenario, write, writeCell, wrongScale };

/** An ephemeris written in JavaScript, typed. */
const provider: EphemerisProvider = {
  name: 'my-engine',
  bodies: ['SUN', 'MOON'],
  jdMin: 2451545,
  jdMax: 2460000,
  positions(request) {
    const cells = request.jds.length * request.bodies.length;
    if (request.frameBits !== 0) return null;
    return { lon: new Float64Array(cells), status: new Int32Array(cells) };
  },
};

// @ts-expect-error a provider names itself and its bodies
const nameless: EphemerisProvider = { bodies: ['SUN'], positions: () => null };
const wrongBody: EphemerisProvider = {
  name: 'x',
  // @ts-expect-error a body is named by its key, not its id
  bodies: [0],
  positions: () => null,
};
const wrongAnswer: EphemerisProvider = {
  name: 'x',
  bodies: ['SUN'],
  // @ts-expect-error a column is numbers, not strings
  positions: () => ({ lon: ['1'] }),
};

export {
  barePlace,
  handshake,
  nameless,
  place,
  provider,
  swappedPlace,
  wrongAnswer,
  wrongBody,
};

/** The chart layer, typed: one chart and a batch of them. */
function charts(): string {
  const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
  const one: Chart = ctx.chart.found({ instant: 2460482.5, place, utcOffsetSeconds: 20700 });
  const lagna: number = one.lagnaDeg;
  const vara: string = one.day.vara;
  const bhava: number = one.grahas[0]!.house.bhava;
  // `dayPart` and `dayElapsed` belong to the instant, not to the day.
  const part: string = one.dayPart;
  const elapsed: number = one.dayElapsed;
  const madhya: number = one.houses[0]!.madhyaDeg;
  // Which ayanamsha, by name, and a tropical chart has none: the absence
  // is in the type, so it cannot be read as a member without a check.
  const ayanamsha: Ayanamsha | 'unknown' | null = one.ayanamsha;
  const custom: boolean = one.ayanamshaCustom;
  // A chart's day is the record an almanac's is: its date is the one
  // `calendar.convert` takes, and a day the Sun rose on has no polar state.
  const date: CalendarDate = one.day.date;
  const polar: PolarDay | null = one.day.polar;
  // @ts-expect-error a day's polar state is absent on a day the Sun rose
  const kind: string = one.day.polar.kind;
  // The air a horizon was refracted through, read all the way down, and
  // absent unless the settings named one.
  const air: Air | null = one.day.air;
  const pressure: number | undefined = air?.pressureHpa;
  // @ts-expect-error a day under the almanac's fixed 34′ has no air
  const temperature: number = one.day.air.temperatureC;
  // @ts-expect-error a chart's ayanamsha may be absent
  const always: Ayanamsha = one.ayanamsha;

  const request: ChartBatchRequest = {
    instants: new Float64Array([2460482.5, 2460600.25]),
    place,
    utcOffsetSeconds: 20700,
  };
  const batch: Charts = ctx.chart.foundMany(request);
  const count: number = batch.length;
  // Iterating a batch gives charts, and the same view `at` gives.
  const first: Chart = batch.at(0);
  const every: readonly Chart[] = [...batch];
  // @ts-expect-error a batch is read, never rewritten
  batch.length = 3;
  // @ts-expect-error `instant` is the singular request's; a batch takes `instants`
  ctx.chart.foundMany({ instant: 2460482.5, place, utcOffsetSeconds: 20700 });
  // @ts-expect-error a chart is a view; its index is not a number to set
  first.index = 2;
  return `${lagna} ${vara} ${bhava} ${part} ${elapsed} ${madhya} ${ayanamsha} ${custom} ${always} ${date.year} ${polar} ${kind} ${pressure} ${temperature} ${count} ${every.length} ${first.instant}`;
}

void charts;

/**
 * The chart reading, typed: every section a request can ask for, read the
 * way an application reads it. `typecheck/surface.mjs` proves each member
 * exists at run time; this proves a consumer can name and use them.
 */
function reading(): string {
  const read: Chart = ctx.chart.found({
    instant: 2460482.5,
    place: { latitude: 27.7172, longitude: 85.324 },
    utcOffsetSeconds: 20700,
    vargas: [Varga.D9, Varga.D10],
    drawings: [
      { layout: ChartLayout.NorthIndian, varga: Varga.D9 },
      { layout: ChartLayout.WesternWheel, varga: Varga.D1 },
    ],
    aspects: true,
    points: true,
    houses: true,
    state: true,
  });
  const drawn = read.drawings[0];
  const firstStep = drawn?.cells[0]?.outline.segments[0];
  const curved: boolean = firstStep?.kind === 'ARC' && firstStep.clockwise;
  const markLon: number = read.drawings[1]?.marks[0]?.longitudeDeg ?? 0;
  // @ts-expect-error a drawing names a layout from the catalogue, not a word
  ctx.chart.found({ instant: 2460482.5, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, drawings: [{ layout: 'lotus', varga: Varga.D1 }] });
  const navamsha = read.vargas[0];
  const vargottama: boolean = navamsha !== undefined && navamsha.lagna.sign === navamsha.lagna.rashi;
  const part: number = navamsha?.grahas[0]?.at.part ?? -1;
  const drishti = read.aspects.map((one) => `${one.from}>${one.to}:${one.houses}:${one.strength}`);
  const edge: number = read.aspects[0]?.toEdge.nakshatraDeg ?? 0;
  const gulika = read.points.find((one) => one.point === Point.Gulika);
  const lord: Graha | 'unknown' = read.bhavas[9]?.lord ?? 'unknown';
  const sun = read.states[0];
  const dispositor: string = sun?.friendship.dispositor ?? 'none';
  const orb: number | null = sun?.combustion.orbDeg ?? null;
  const holding: readonly string[] = sun?.lajjitadi.holding ?? [];
  const war: boolean = sun?.war?.isWinner ?? false;
  // @ts-expect-error a section is read, never replaced
  read.states = [];
  // @ts-expect-error a war is a record, not a number
  const apart: number = sun?.war;
  // @ts-expect-error the varga list takes members of the catalogue, not their names
  ctx.chart.found({ instant: 2460482.5, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, vargas: ['navamsha'] });
  return [
    vargottama, part, drishti.length, edge, gulika?.longitudeDeg ?? 'none', lord,
    dispositor, orb, holding.length, war, apart, read.bhavas.length,
    drawn?.layout ?? 'none', curved, markLon,
  ].join(' ');
}

void reading;

/** The almanac layer, typed: a range of days and one of them. */
function almanac(): string {
  const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
  const from: CalendarDate = someDate;
  const to: CalendarDate = someDate;
  const batch: Almanac = ctx.almanac.of({ from, to, place, utcOffsetSeconds: 20700 });
  const days: number = batch.length;
  const day: AlmanacDay = batch.at(0);
  const vara: string = day.day.vara;
  const tithi: string = day.tithi[0]!.member;
  const until: number = day.tithi[0]!.inside.to;
  const lord: string = day.horas[0]!.lord;
  const daylight: boolean = day.muhurtas[0]!.daylight;
  // An absent value is null, never a sentinel, and the checker knows it.
  const sankranti: number | null = day.sankranti;
  const effective: boolean = day.abhijit?.effective ?? false;
  const every: readonly AlmanacDay[] = [...batch];
  const one: AlmanacDay = ctx.almanac.day({ date: from, place, utcOffsetSeconds: 20700 });
  // @ts-expect-error an absent sankranti is null, so it is not a number
  const wrong: number = day.sankranti;
  // @ts-expect-error a range needs both ends; `date` is the single-day shape
  ctx.almanac.of({ date: from, place, utcOffsetSeconds: 20700 });
  // @ts-expect-error a batch is read, never rewritten
  batch.length = 3;
  return `${days} ${vara} ${tithi} ${until} ${lord} ${daylight} ${sankranti} ${effective} ${every.length} ${one.index} ${wrong}`;
}

void almanac;

/** The eclipses beside an almanac's days, typed all the way down. */
function eclipses(): string {
  const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
  const asked: Almanac = ctx.almanac.of({ from: someDate, to: someDate, place, utcOffsetSeconds: 20700, eclipses: true });
  const found: Eclipses | null = asked.eclipses;
  const lunar: LunarEclipseHere | undefined = found?.value.lunar[0];
  const kind: LunarEclipseKind | undefined = lunar?.eclipse.kind;
  // @ts-expect-error a kind is its catalogue member, never the bare key
  const bare: 'TOTAL' | undefined = lunar?.eclipse.kind;
  const totality: number | null | undefined = lunar?.eclipse.contacts.u2;
  const altitude: number | undefined = lunar?.here.greatest.altitudeDeg;
  const seenFrom: number | undefined = lunar?.here.seen?.from;
  const umbralTo: number | undefined = lunar?.here.umbralSeen?.to;
  const solar: SolarEclipseHere | undefined = found?.value.solar[0];
  const local: SolarEclipseView | null | undefined = solar?.here;
  const obscured: number | undefined = local?.obscuration;
  const where: number | undefined = solar?.eclipse.point.latitude;
  // @ts-expect-error a place's view of a solar eclipse is never hybrid
  const hybrid: SolarEclipseView['kind'] = 'solar_eclipse_kind.HYBRID';
  // @ts-expect-error an eclipse not asked for is null, so it needs a check
  const unchecked: number = asked.eclipses.value.lunar.length;
  return `${kind} ${totality} ${altitude} ${seenFrom} ${umbralTo} ${obscured} ${where} ${hybrid} ${unchecked} ${bare}`;
}

void eclipses;

/** Each day's Nepal Sambat date beside an almanac's days, typed all the way down and said. */
function nepalSambat(): string {
  const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
  const asked: Almanac = ctx.almanac.of({ from: someDate, to: someDate, place, utcOffsetSeconds: 20700, nepalSambat: true });
  const found: NepalSambatDates | null = asked.nepalSambat;
  const date: NepalSambatDate | undefined = found?.value[0];
  const year: number | undefined = date?.year;
  const anala: boolean = date?.kind === 'ADHIKA';
  // @ts-expect-error a half is its catalogue member, never the bare key
  const bare: 'SHUKLA' | undefined = date?.paksha;
  // @ts-expect-error dates not asked for are null, so they need a check
  const unchecked: number = asked.nepalSambat.value.length;
  return `${year} ${anala} ${bare} ${unchecked}`;
}

void nepalSambat;

/** A layout of the consumer's own, typed: copied, renamed, registered and drawn. */
function ownLayout(): string {
  const row: LayoutRow = ctx.chart.layout(ChartLayout.SouthIndian);
  const renamed: LayoutRow = { ...row, key: 'ACME_KERALA' };
  const own: Context = ctx;
  const options: ConstructorParameters<typeof Context>[0] = { testProvider: true, layouts: [renamed] };
  const key: LayoutKey = 'chart_layout.ACME_KERALA';
  const drawn = own.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    drawings: [{ layout: key, varga: Varga.D1 }],
  }).drawings[0]!;
  const rings: number = row.shape.kind === 'RADIAL' ? row.shape.rings.length : row.shape.cells.length;
  // @ts-expect-error a layout key names its kind
  own.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, drawings: [{ layout: 'ACME_KERALA', varga: Varga.D1 }] });
  // @ts-expect-error a cell holds a sign by its bare key, not a number
  const wrong: LayoutHolds = { kind: 'SIGN', value: 1 };
  return `${drawn.layout} ${rings} ${wrong.kind} ${options?.layouts?.length}`;
}

void ownLayout;

// A year's own chart, read all the way down. The declarations are
// hand-written and the layer's plain records are not measured against a
// real instance the way its classes are, so a field the layer returns and
// these forget is invisible until someone reads it — which is what this
// does. `yearLord` was missing here once for exactly that reason.
function theYearsOwnChart(ctx: Context): string {
  const year = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    varsha: { through: 2, place: 'birth', varshesha: { moon: 'PASSED_OVER' } },
  }).praveshas[0]!;
  const annual = year.annual!;
  const lord: string = `${annual.yearLord.graha} ${annual.yearLord.chosen}`;
  // Every step of the chain, and no other: a hand-kept copy of this union
  // once shadowed the catalogue's and lacked the Moon's three readings.
  const steps: Record<typeof annual.yearLord.chosen, string> = {
    STRONGEST: 'the strongest',
    MOST_PORTFOLIOS: 'the most portfolios',
    MUNTHA_LORD_UNASPECTED: "the Muntha's lord, unaspected",
    MUNTHA_LORD_ALL_WEAK: "the Muntha's lord, all weak",
    MUNTHA_LORD_TIED: "the Muntha's lord, tied",
    DINA_RATRI_TIED: 'the day or night lord, tied',
    ANNUAL_LAGNA_LORD_UNASPECTED: "the year's lagna lord, unaspected",
    STRONGEST_UNASPECTED: 'the strongest, unaspected',
    MOONS_ITHASALA: "through the Moon's Ithasala",
    MOONS_SIGN_LORD: "the Moon's sign lord",
    unknown: 'a step this build does not know',
  };
  const step: string = steps[annual.yearLord.chosen];
  const bala: number = annual.yearLord.vishwa.units + annual.yearLord.vishwa.total;
  const claim = annual.yearLord.claims[0]!;
  const held: number = claim.portfolios;
  const aspects: boolean = claim.aspectsLagna;
  const pair = annual.yogas[0];
  const orb: number = pair?.orbDeg ?? 0;
  const muntha: string = `${year.muntha.sign} ${year.muntha.lord}`;
  return `${lord} ${step} ${bala} ${held} ${aspects} ${orb} ${muntha} ${annual.byDay}`;
}

void theYearsOwnChart;

// The sixteen Tajika yogas for a year's matters, read all the way down,
// with the rule records in the casing the declarations promise — which
// the boundary once refused for `noneAspects`.
function theYearsMatters(ctx: Context): string {
  const annual = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    varsha: {
      through: 2,
      place: 'birth',
      matters: [7, 10],
      yogas: { drishti: { subDegree: 'ISHRAFA' }, weakBelow: 5 * 3600, tambira: 'EITHER_LORD', moonBenefic: 'WAXING' },
      varshesha: { noneAspects: 'ANNUAL_LAGNA_LORD' },
    },
  }).praveshas[0]!.annual!;
  const matter = annual.matters[0]!;
  const promised: boolean | null = matter.holds('ITHASALA');
  const pair: number = matter.between?.apartDeg ?? 0;
  const held = matter.held[0];
  const legs: number = held?.legs?.[1].orbDeg ?? 0;
  const through: string = held?.through ?? '-';
  const clauses: readonly string[] = held?.afflictions?.karyesha ?? [];
  const states = `${annual.retrograde.join()} ${annual.combust.join()}`;
  // @ts-expect-error a matter is 'all' or house numbers, not a word of its own
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, matters: 'some' } });
  return `${matter.house} ${matter.sign} ${promised} ${pair} ${legs} ${through} ${clauses} ${states} ${matter.unanswered}`;
}

void theYearsMatters;

// A year's sahams, read all the way down, with their rules in the casing
// the declarations promise.
function theYearsSahams(ctx: Context): string {
  const annual = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    varsha: {
      through: 2,
      place: 'birth',
      sahams: ['PUNYA', 'VIVAHA'],
      sahamRules: { addSign: 'SIGNS', houses: 'EQUAL', roga: 'SATURN' },
    },
  }).praveshas[0]!.annual!;
  const one = annual.sahams[0]!;
  const saham: Saham | 'unknown' = one.saham;
  const deg: number = one.longitudeDeg;
  const added: boolean = one.addedSign;
  // @ts-expect-error a saham is one of the forty-one keys, not a word of its own
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, sahams: ['pnya'] } });
  // @ts-expect-error the rules' keys are camelCase
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, sahamRules: { add_sign: 'NEVER' } } });
  const clause: SahamStrong | 'unknown' | undefined = one.strong[0];
  const facing: string = one.seven.map((s) => `${s.graha}:${s.drishti}:${s.relation}:${s.company}`).join();
  const axis: boolean | null = one.inNodeAxis;
  const happy = annual.harsha.map((h) => `${h.graha}:${h.total}:${h.grade}:${h.sthana}`).join();
  return `${saham} ${deg} ${one.sign} ${one.lord} ${one.house} ${added} ${clause} ${facing} ${axis} ${one.lordVishwa} ${one.lordHarsha} ${happy}`;
}

// A year's annual dashas, read all the way down, asked for by the keys
// they are read back as and with their rules in the declared casing.
function theYearsDashas(ctx: Context): string {
  const annual = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    varsha: {
      through: 2,
      place: 'birth',
      dashas: ['dasha_system.MUDDA', 'dasha_system.PATYAYINI'],
      dashaRules: { clock: { DAYS: 365 }, balance: 'ENTRY_MOON', measure: 'TEMPORAL', birthPeriod: 'ELAPSED', depth: 3 },
    },
  }).praveshas[0]!.annual!;
  const dasha = annual.dashas[0]!;
  const system: AnnualDashaSystem | 'unknown' = dasha.system;
  const again: readonly AnnualDashaSystem[] = system === 'unknown' ? [] : [system];
  const seed: string | null = dasha.seed;
  const share = dasha.ring.shares[dasha.ring.first]!;
  const sign: string | null = share.sign;
  const remaining: number | null = dasha.ring.remaining;
  const running: readonly DashaPeriod[] = dasha.at((dasha.year.from + dasha.year.to) / 2);
  // @ts-expect-error an annual dasha is one of the three, not a natal system
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, dashas: ['dasha_system.VIMSHOTTARI'] } });
  // @ts-expect-error the rules' keys are camelCase
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, dashaRules: { birth_period: 'ELAPSED' } } });
  // @ts-expect-error a clock of days is a record, not a bare number
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, dashaRules: { clock: 360 } } });
  return `${system} ${again} ${seed} ${share.lord} ${share.weight} ${sign} ${remaining} ${running.map((p) => p.path)} ${dasha.firstLord} ${dasha.periods.length}`;
}

void theYearsDashas;

// The birth's own sahams, and the readings a saham's strength and the
// Harsha bala part on, in the casing the declarations promise.
function theBirthsSahams(ctx: Context): string {
  const chart = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    varsha: {
      through: 1,
      sahams: 'all',
      sahamStrength: { natures: 'PARASHARI', friendship: 'NATURAL', weakBelow: 4 * 3600 },
      harshaRules: { venus: 'TWELFTH' },
    },
  });
  const weak: readonly (SahamWeak | 'unknown')[] = chart.sahams[0]?.weak ?? [];
  // @ts-expect-error natures are the chapter's or the catalogue's, not a word of its own
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, sahamStrength: { natures: 'vedic' } } });
  return weak.join();
}

void theBirthsSahams;

void theYearsSahams;

/**
 * What computed a result, read typed all the way down (STATUS 2h): the
 * provenance was a `Record<string, unknown>` a consumer read by string.
 */
function theProvenance(ctx: Context): string {
  const chart = ctx.chart.found({ instant: 2451545, place: { latitude: 27.7, longitude: 85.3 }, utcOffsetSeconds: 20700 });
  const provenance: Provenance = chart.provenance;
  const hash: string = provenance.contentHash;
  const major: number = provenance.sdkVersion.major;
  const frame: string = provenance.provider.frame;
  const tier: string | null = provenance.provider.tier;
  // A tagged union narrows by its `kind`.
  const edition: string =
    provenance.calendar?.kind === 'TABULAR' ? provenance.calendar.edition : 'none';
  const sure: Confidence = provenance.confidence;
  // @ts-expect-error a confidence is one of two keys
  const unsure: Confidence = 'MAYBE';
  // @ts-expect-error the wire's snake case is decoded away
  const wire: string = provenance.settings_hash;
  const steps = ctx.positions({ instants: [2451545], bodies: ['SUN'] }).steps;
  const native: boolean = steps.some((step: Step) => step.implementation === 'NATIVE');
  // @ts-expect-error an implementation is one of three keys
  const guessed: boolean = steps.some((step) => step.implementation === 'native');
  const stored: Provenance = decodeProvenance(JSON.parse(chart.batch.provenanceJson));
  return `${hash} ${major} ${frame} ${tier ?? ''} ${edition} ${sure} ${String(unsure)} ${wire} ${native} ${guessed} ${stored.profile}`;
}

void theProvenance;

// The words a request writes are keys, as every answer spells them; the
// lowercase ones they replaced are refused here, before a run.
function theWordsAreKeys(): string {
  const dark: ShippedTheme = 'DARK';
  const glyphs: Theme = { extends: dark, content: { body_form: 'GLYPH', cell_label: 'SIGN_NUMBER' } };
  // @ts-expect-error a shipped theme is named by its key
  const lower: Theme = 'dark';
  const rules: RuleRequest = { shipped: ['NABHASAS'], readings: 'RECORDING_ENGINE' };
  // @ts-expect-error a shipped set is named by its key
  const set: RuleRequest = { shipped: ['nabhasas'] };
  const sthira: RashiDashaDefinition = {
    kernel: 'RASHI',
    key: 'ACME_STHIRA',
    start: 'ARUDHA_LAGNA',
    length: { BY_MODALITY: { movable: 7, fixed: 8, dual: 9 } },
    named_lord: 'FIRST',
    year_length: 'SAVANA_360',
  };
  // @ts-expect-error a field is spelt as the document spells it
  const camel: RashiDashaDefinition = { kernel: 'RASHI', key: 'ACME_STHIRA', namedLord: 'FIRST' };
  return `${String(glyphs)} ${String(lower)} ${String(rules)} ${String(set)} ${sthira.key} ${String(camel)}`;
}

void theWordsAreKeys;

// The transits read all the way down, so a field the layer returns and the
// declarations forget fails here rather than in a consumer's editor.
function theTransits(ctx: Context): string {
  const reading = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    gochar: { instants: [2460676.5], from: 'LAGNA', ashtakavarga: true },
  }).gochar[0]!;
  const judged = reading.ashtakavarga![0]!;
  const lord: 'LAGNA' | typeof judged.kakshya.lord = judged.kakshya.lord;
  const standing: string = `${judged.sarva} ${judged.sarvaStanding} ${reading.rules.ashtakavargaGoodFrom}`;
  const sun = reading.grahas[0]!;
  const vedha: number | null = sun.vedhaHouse;
  const by: readonly Graha[] = sun.obstructedBy;
  const verdicts: Record<typeof sun.verdict, string> = {
    GOOD: 'good',
    OBSTRUCTED: 'obstructed',
    NOT_GOOD: 'not good',
    unknown: 'unknown',
  };
  // @ts-expect-error a reference is named by its key
  const moon: typeof reading.reference.from = 'moon';
  return (
    `${reading.instant} ${reading.reference.from} ${reading.reference.sign} ` +
    `${reading.rules.nodeVedha} ${reading.rules.nodeObstruction} ${sun.graha} ${sun.transit.sign} ` +
    `${sun.transit.degrees} ${sun.house} ${sun.goodHouse} ${String(vedha)} ${by.join()} ` +
    `${verdicts[sun.verdict]} ${sun.fruition} ${sun.fruitfulNow} ${String(moon)} ` +
    `${judged.bindus} ${judged.good} ${judged.kakshya.index} ${lord} ${judged.kakshyaBindu} ${standing}`
  );
}

void theTransits;

// The hit list read all the way down, each event narrowed by its kind, and
// a request that names grahas bare and full and points every way.
function theHitList(ctx: Context): string {
  const hits = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    hits: {
      from: 2460676.5,
      to: 2461041.5,
      grahas: ['SATURN', 'graha.JUPITER'],
      kinds: ['SIGN_INGRESS', 'STATION', 'ASPECT'],
      points: ['LAGNA', 'MOON', { point: 'GRAHA', graha: 'graha.SUN' }, { point: 'LAGNA' }],
      aspects: [0, 90, 180],
      orbDeg: 2,
    },
  }).hits;
  const words: string[] = hits.map((hit) => {
    const { event } = hit;
    switch (event.kind) {
      case 'SIGN_INGRESS':
        return `${hit.instant} ${hit.graha} ${event.into} ${event.motion}`;
      case 'NAKSHATRA_INGRESS':
        return `${event.into} ${event.motion}`;
      case 'STATION':
        return `${event.turns}`;
      case 'ASPECT': {
        const to = event.to.point === 'GRAHA' ? event.to.graha : event.to.point;
        return `${to} ${event.angle} ${event.phase} ${event.motion}`;
      }
    }
  });
  // @ts-expect-error a graha is named by its key, bare or full
  const pluto: HitRequest = { from: 0, to: 1, grahas: ['saturn'] };
  // @ts-expect-error a kind is spelt as an event's `kind`
  const eclipse: HitRequest = { from: 0, to: 1, kinds: ['ECLIPSE'] };
  return `${words.join()} ${String(pluto)} ${String(eclipse)}`;
}

void theHitList;

// Sade Sati read all the way down, and a request in every field.
function theSadeSati(ctx: Context): string {
  const report = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    sadeSati: { from: 2460676.5, to: 2464329, countedFrom: 'LAGNA', reckoning: 'DEGREE', spells: [4, 8] },
  }).sadeSati;
  if (report === null) return 'none';
  const phases = report.sadeSati.flatMap((one) =>
    one.phases.map((spell) => `${spell.house} ${spell.visits.map((v) => `${v.from ?? '-'}..${v.to ?? '-'}`).join()}`),
  );
  const spells = report.spells.map((spell) => `${spell.house} ${spell.visits.length}`);
  // @ts-expect-error a reckoning is spelt as the report reads it back
  const arc: SadeSatiRequest = { from: 0, reckoning: 'ARC' };
  return `${report.reference.from} ${report.reference.sign} ${report.reckoning} ${phases.join()} ${spells.join()} ${String(arc)}`;
}

void theSadeSati;

// KP read all the way down, and a request in every field.
function theKpReading(ctx: Context): string {
  const reading: KpReading | null = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 13.08, longitude: 80.27, altitude: 6 },
    utcOffsetSeconds: 19800,
    kp: { number: 74, clock: 19800, anyAyanamsha: false },
  }).kp;
  if (reading === null) return 'none';
  const lords = (l: KpLords): string =>
    `${l.sign} ${l.star.lord} ${l.sub.lord} ${l.subSub.lord} ${l.subSub.span.end - l.subSub.span.start}`;
  const cusps = reading.chart.cusps.map((cusp) => `${cusp.house} ${cusp.longitude} ${lords(cusp.lords)}`);
  const planets = reading.chart.planets.map((p) => `${p.graha} ${p.house} ${p.retrograde} ${lords(p.lords)}`);
  const houses = reading.significators.houses.map(
    (h) =>
      `${h.house} ${h.inOccupantsStars.join()} ${h.occupants.join()} ${h.inLordsStar.join()} ${h.lord} ` +
      `${h.conjoined.join()} ${h.aspected.join()} ${h.intercepted.join()}`,
  );
  const nodes = reading.significators.nodes.map(
    (n) => `${n.node} ${n.conjoined.join()} ${n.starLord} ${n.aspecting.join()} ${n.signLord}`,
  );
  const rulers = reading.ruling.rulers.map((ruler) => {
    const why = ruler.reasons.map((reason) => (reason.kind === 'AGENT' ? `${reason.of} ${reason.by}` : reason.kind));
    const rejected = ruler.rejectedBy === null ? '' : `${ruler.rejectedBy.retrograde} ${ruler.rejectedBy.byStar}`;
    return `${ruler.graha} ${why.join()} ${ruler.retrograde} ${rejected} ${ruler.rejectedBySub?.retrograde ?? ''}`;
  });
  const { count, nodeRulers, retrogradeRejection } = reading.ruling.rules;
  // @ts-expect-error a horary number is a number
  const named: KpRequest = { number: '74' };
  return [reading.chart.system, ...cusps, ...planets, ...houses, ...nodes, ...rulers, count, nodeRulers, retrogradeRejection, String(named)].join();
}

void theKpReading;

// A prashna read all the way down, and a request in every field.
function thePrashna(ctx: Context): string {
  const asked: PrashnaRequest = {
    question: { house: 7, number: 14 },
    rules: {
      pisces: 'SHIRSHODAYA',
      timing: 'BASELINE',
      mook: 'MOON_HOUSE',
      moon: { kshina: 'DARK_ELEVENTH_TO_NEW_MOON' },
      score: 'BASELINE',
    },
  };
  const read: Prashna | null = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
    utcOffsetSeconds: 20700,
    prashna: asked,
  }).prashna;
  if (read === null) return 'none';
  const clauses = read.verdict.clauses.map((c) => `${c.kind} ${c.graha ?? '-'} ${c.favour}`);
  const { timing, mook, links, moon, score } = read;
  const when = `${timing.rule} ${timing.graha} ${timing.tie} ${timing.count} ${timing.multiplier} ${timing.amount ?? '-'} ${timing.unit} ${timing.between.join()}`;
  const about = `${mook.rule} ${mook.graha} ${mook.tie} ${mook.house} ${mook.person ?? '-'} ${mook.thought}`;
  const tied =
    links === null
      ? '-'
      : `${links.house} ${links.sign} ${links.lagnesha} ${links.karyesha} ${links.sameLord} ` +
        `${links.between?.orbDeg ?? '-'} ${links.held.map((h) => `${h.yoga} ${h.through ?? '-'}`).join()} ` +
        `${links.states?.combust.join() ?? '-'} ${String(links.holds('NAKTA'))}`;
  const points =
    score === null
      ? '-'
      : `${score.points} ${score.answer} ${score.factors.map((f) => `${f.kind} ${f.points}`).join()} ${score.void} ${score.applyingTo ?? '-'}`;
  // @ts-expect-error a house is a number
  const wrong: PrashnaRequest = { question: { house: '7' } };
  return [
    read.rules.mook,
    read.verdict.outcome,
    read.change,
    ...clauses,
    when,
    about,
    tied,
    moon.rules.kshina,
    moon.clauses.join(),
    points,
    read.numberSign ?? '-',
    String(wrong),
  ].join();
}

void thePrashna;

// Remedies read all the way down, and a request in every field.
function theRemedies(ctx: Context): string {
  const asked: RemedyRequest = {
    at: 2460676.5,
    rules: {
      functional: { scheme: 'BASELINE' },
      shanti: { rik: 'YAJNAVALKYA' },
      devata: { sunWithKetu: 'SURYA' },
    },
  };
  const read: Remedies | null = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
    utcOffsetSeconds: 20700,
    remedies: asked,
  }).remedies;
  if (read === null) return 'none';
  const { rules, functional, subjects, shantis, ishtaDevata } = read;
  const natures = functional.rows.map(
    (r) => `${r.graha} ${r.houses.join()} ${r.clauses.map((c) => `${c.kind} ${c.house}`).join()} ${r.nature}`,
  );
  const lordships = `${functional.lagna} ${functional.scheme} ${functional.yogakarakas.join()} ${functional.marakas.join()} ${functional.badhaka.house} ${functional.badhaka.lord}`;
  const whom = subjects.subjects.map((s) => `${s.graha} ${s.reasons.join()}`);
  const running =
    subjects.antardasha === null
      ? '-'
      : (({ shanti: d, holds }) =>
          `${d.mahadasha} ${d.antardasha} ${d.chapter} ${d.verses} ${d.page} ${d.conditions.join()} ${d.remedies.join()} ${holds.map((h) => h ?? '-').join()}`)(
          subjects.antardasha,
        );
  const rites = shantis.map(
    (s) =>
      `${s.graha} ${s.image} ${s.rik} ${s.japaThousands} ${s.samidh} ${s.food} ${s.dakshina} ${s.gem} ${s.substance ?? '-'} ${s.direction ?? '-'} ${s.mandala}`,
  );
  const devatas = [ishtaDevata.inRasi, ishtaDevata.inNavamsha].map(
    (d) =>
      `${d.rules.sunWithKetu} ${d.sign} ${d.devotions.map((v) => `${v.graha} ${v.deities.join()} ${v.verse} ${v.withKetu}`).join()} ${d.minor.join()}`,
  );
  const { amatya } = ishtaDevata;
  const fromAmatya = [amatya.inRasi, amatya.inNavamsha].map(
    (a: AmatyaDevata) =>
      `${a.twelfth.sign} ${a.twelfth.minor.join()} ${a.sign} ${a.house} ${a.joined.map((v: Devotion) => `${v.graha} ${v.deities.join()}`).join()}`,
  );
  // @ts-expect-error a rule is one of the texts'
  const wrong: RemedyRequest = { rules: { shanti: { rik: 'MANU' } } };
  return [
    rules.functional.scheme,
    rules.shanti.rik,
    rules.devata.sunWithKetu,
    ...natures,
    lordships,
    ...whom,
    running,
    ...rites,
    ishtaDevata.atmakaraka,
    ishtaDevata.karakamsha,
    ...devatas,
    `${amatya.graha} ${amatya.amsha}`,
    ...fromAmatya,
    String(wrong),
  ].join();
}

void theRemedies;

// The essential dignities read all the way down, and a request in every
// field, a table's lords in either spelling.
function theDignities(ctx: Context): string {
  const term = (lord: GrahaName, end: number): Term => ({ lord, end });
  const sign = [term('JUPITER', 6), term('graha.VENUS', 12), term('MERCURY', 20), term('MARS', 25), term('SATURN', 30)];
  const asked: DignityRequest = {
    sectRule: 'DAYLIGHT',
    rules: { terms: { TABLE: Array.from({ length: 12 }, () => sign) }, triplicities: 'PTOLEMY' },
    scores: { peregrine: 0 },
  };
  const read: Dignities | null = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    dignities: asked,
  }).dignities;
  if (read === null) return 'none';
  const planets = read.planets.map((at: PlanetDignity) => {
    const d = at.dignity;
    const flags = [d.house, d.exaltation, d.triplicity, d.term, d.face, d.detriment, d.fall, at.peregrine];
    return `${at.planet} ${at.longitudeDeg} ${flags.join()} ${at.score} ${at.reception}`;
  });
  const receptions = read.receptions.map((one: Reception) => {
    const [first, second] = one.planets;
    const kinds: readonly DignityKind[] = one.mutual;
    return `${first} ${second} ${one.firstIn.house} ${one.secondIn.exaltation} ${kinds.join()}`;
  });
  const s = read.scores;
  const scores = [s.house, s.exaltation, s.triplicity, s.term, s.face, s.detriment, s.fall, s.peregrine];
  // @ts-expect-error a sect rule is spelt as the answer reads it back
  const dusk: DignityRequest = { sectRule: 'DUSK' };
  // @ts-expect-error `TABLE` names the request's table in an answer, and is no system to ask for
  const table: DignityRequest = { rules: { terms: 'TABLE' } };
  return [read.sect, read.sectRule, read.rules.terms, read.rules.triplicities, ...scores, ...planets, ...receptions, String(dusk), String(table)].join();
}

void theDignities;

// The accidental fortitudes read all the way down, and a request in every
// field, an answer's rules and scores fed back as they stand.
function theFortitudes(ctx: Context): string {
  const asked: FortitudeRequest = {
    dignities: { sectRule: 'DAYLIGHT' },
    rules: { beamsDeg: 15, combustionInSign: false, partile: { WITHIN: { orbDeg: 1 } }, siege: 'SAME_SIGN' },
    scores: { regulus: 5, houses: [5, 3, 1, 4, 3, -2, 4, -2, 2, 5, 4, -5] },
    almuten: { place: 'SIGN', fortune: 'REVERSED_BY_NIGHT' },
  };
  const read: Fortitudes | null = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    fortitudes: asked,
  }).fortitudes;
  if (read === null) return 'none';
  const planets = read.planets.map((at: PlanetAccidents) => {
    const lines = at.accidents.map((line: AccidentLine) => `${line.accident} ${line.points}`);
    return `${at.planet} ${at.house} ${lines.join()} ${at.fortitude} ${at.debility} ${at.net}`;
  });
  const sky = read.sky;
  const almutens: Almutens = read.almutens;
  const ranked = (almuten: Almuten) =>
    `${almuten.totals.map((at: AlmutenTotal) => `${at.planet}:${at.total}`).join()} ${almuten.almutens.join()} ${almuten.partakers.join()}`;
  const fedBack: FortitudeRequest = { rules: read.rules, scores: read.scores, almuten: almutens.rules };
  // @ts-expect-error a place is read by `DEGREE` or `SIGN`, nothing else
  const misread: FortitudeRequest = { almuten: { place: 'CUSP' } };
  // @ts-expect-error an orb is written under `WITHIN`, as the answer reads it back
  const bare: FortitudeRequest = { rules: { partile: 'WITHIN' } };
  return [
    read.dignities.sect,
    sky.houses,
    ...sky.cuspsDeg,
    ...sky.speedsDegPerDay,
    sky.northNodeDeg,
    sky.regulusDeg,
    sky.spicaDeg,
    sky.algolDeg,
    sky.ascendantDeg,
    sky.midheavenDeg,
    almutens.fortuneDeg,
    ranked(almutens.figure),
    ranked(almutens.places),
    ...almutens.houses.map(ranked),
    String(misread),
    read.rules.combustionDeg,
    read.scores.algol,
    ...planets,
    String(fedBack),
    String(bare),
  ].join();
}

void theFortitudes;

// The lots read all the way down, and the answer's request fed back as it
// stands.
function theLots(ctx: Context): string {
  const asked: LotRequest = { sectRule: 'DAYLIGHT', fortune: 'REVERSED_WHILE_MOON_UP' };
  const read: Lots | null = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 27.7, longitude: 85.3, altitude: 1400 },
    utcOffsetSeconds: 20700,
    lots: asked,
  }).lots;
  if (read === null) return 'none';
  const fedBack: LotRequest = read.request;
  // @ts-expect-error Fortune by night is one of the three readings, nothing else
  const misread: LotRequest = { fortune: 'REVERSED' };
  const places = read.lots.map(({ lot, place }: PlacedLot) => {
    const at: LotPlace = place;
    return `${lot} ${at.longitudeDeg} ${at.sign} ${at.lord} ${at.house}`;
  });
  return [read.sect, read.fortuneReversed, ...places, String(fedBack), String(misread)].join();
}

void theLots;

// The considerations read all the way down, and the answer's rules fed
// back as a request.
function theConsiderations(ctx: Context): string {
  const asked: ConsiderationRequest = { moonLateFromDeg: 25 };
  const read: Considerations | null = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 51.5, longitude: -0.12, altitude: 0 },
    utcOffsetSeconds: 0,
    considerations: asked,
  }).considerations;
  if (read === null) return 'none';
  const fedBack: ConsiderationRequest = read.rules;
  // @ts-expect-error the orbs are seven, one a planet in the Chaldean order
  const misread: ConsiderationRequest = { orbsDeg: [10, 12] };
  const next: Perfection | null = read.moon.course.next;
  const lord: Graha = read.seventh.lord;
  return [
    read.radicality.hourLord,
    read.radicality.grounds.join(),
    read.ascendant.sign,
    read.moon.viaCombusta,
    next?.aspect ?? 'void',
    next?.gapDeg ?? Number.NaN,
    read.moon.course.withinOrb?.planet ?? 'void',
    lord,
    read.seventh.infortunesInHouse.join(),
    read.saturnHouse,
    String(fedBack),
    String(misread),
  ].join();
}

void theConsiderations;

// The perfection read all the way down, a request naming its house.
function thePerfection(ctx: Context): string {
  const asked: PerfectionRequest = { house: 7, rules: { horizonDays: 30, withinSign: false } };
  const read: Matter | null = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 51.5, longitude: -0.12, altitude: 0 },
    utcOffsetSeconds: 0,
    perfection: asked,
  }).perfection;
  if (read === null) return 'none';
  // @ts-expect-error a significator is a graha, not a house number
  const misread: PerfectionRequest = { quesited: 7 };
  const application: Application | null = read.application;
  const kind: ApplicationKind | undefined = application?.kind;
  const impediment: Impediment | undefined = read.impediments[0];
  const third: Graha | null | undefined = impediment?.third;
  const translation: Translation | undefined = read.translations[0];
  const collection: Collection | undefined = read.collections[0];
  const held: readonly Way[] = read.ways.held;
  return [
    read.querent,
    read.quesited,
    kind ?? 'none',
    application?.gapDeg ?? Number.NaN,
    read.separation?.pastDeg ?? Number.NaN,
    impediment?.kind satisfies ImpedimentKind | undefined,
    third ?? 'none',
    translation?.received.term ?? false,
    collection?.fromQuerent.days ?? Number.NaN,
    read.ways.quesited.dignity.exaltation,
    read.ways.infortunesBetween.join(),
    held.join(),
    read.rules.horizonDays ?? 'unset',
    read.rules.withinSign satisfies boolean,
    String(misread),
  ].join();
}

void thePerfection;

// Progressions read all the way down, a request naming both an instant
// and a window.
function theProgressions(ctx: Context): string {
  const asked: ProgressionsRequest = {
    at: 2417505.5,
    year: 'NOON_SIDEREAL_TIME',
    angles: 'SOLAR_ARC_LONGITUDE',
    direction: { PER_YEAR: 1 },
    contacts: { from: 2417484.5, to: 2417941.5, grahas: ['MOON'], points: ['LAGNA', 'MERCURY'], aspects: [135] },
  };
  const read: Progressions | null = ctx.chart.found({
    instant: 2400629.742361111,
    place: { latitude: 51.5, longitude: 0, altitude: 0 },
    utcOffsetSeconds: 0,
    progressions: asked,
  }).progressions;
  if (read === null) return 'none';
  // @ts-expect-error a year is a measure's name, not a number of days
  const misread: ProgressionsRequest = { year: 365.25 };
  const contact: ProgressedContact | undefined = read.contacts?.[0];
  const to: NatalPoint | undefined = contact?.to;
  return [
    read.progressed?.armcDeg ?? Number.NaN,
    read.progressed?.angles.midheavenDeg ?? Number.NaN,
    read.progressed?.grahas[0]?.speedDegPerDay ?? Number.NaN,
    read.directed?.arcDeg ?? Number.NaN,
    read.directed?.planets[0]?.longitudeDeg ?? Number.NaN,
    contact?.life ?? Number.NaN,
    contact?.motion satisfies Motion | 'unknown' | undefined,
    to?.point ?? 'none',
    String(misread),
  ].join();
}

void theProgressions;

// A chart's Western aspects and its synastry with a partner, read all the
// way down; the synastry takes the table's own fields beside the partner.
function theWesternAspects(ctx: Context): string {
  const table: WesternAspectRequest = {
    aspects: ['TRINE', 'SQUARE'],
    orbs: { model: 'BY_ASPECT', orbs: [{ aspect: 'TRINE', orbDeg: 6 }] },
  };
  const asked: SynastryRequest = {
    ...table,
    partner: { instant: 2403113.4993, place: { latitude: 51.5058, longitude: -0.1878, altitude: 0 } },
    lagna: false,
    zodiac: 'CHARTS',
    parallels: { orbDeg: 1.5 },
    antiscia: { orbs: { model: 'LEO' } },
    midpoints: { orbDeg: 1 },
    composite: true,
    davison: true,
  };
  const chart = ctx.chart.found({
    instant: 2402390.5542,
    place: { latitude: 51.5045, longitude: -0.1366, altitude: 0 },
    utcOffsetSeconds: 0,
    westernAspects: table,
    synastry: asked,
  });
  // @ts-expect-error a synastry is asked against a partner
  const alone: SynastryRequest = { lagna: false };
  // @ts-expect-error the zodiac is tropical or each chart's own
  const sidereal: SynastryRequest = { ...asked, zodiac: 'SIDEREAL' };
  const own: WesternAspectRow | undefined = chart.westernAspects?.[0];
  const across: SynastryRow | undefined = chart.synastry?.[0];
  const theirs: NatalPoint | undefined = across?.second;
  const level: SynastryParallelRow | undefined = chart.synastryParallels?.[0];
  const reflected: AntiscionRow | undefined = chart.synastryAntiscia?.[0];
  const equal: SynastryMidpointRow | undefined = chart.synastryMidpoints?.[0];
  const composite: Composite | null = chart.synastryComposite;
  const middle: CompositePlanet | undefined = composite?.planets[0];
  const davison: DavisonBirth | null = chart.synastryDavison;
  const between: Chart | null = davison === null ? null : ctx.chart.found({ ...davison });
  return [
    own?.first ?? 'none',
    own?.applying ?? false,
    across?.first.point ?? 'none',
    theirs?.point === 'GRAHA' ? theirs.graha : 'LAGNA',
    across?.aspect ?? 'none',
    across?.fromExactDeg ?? Number.NaN,
    level?.second.point ?? 'none',
    level?.contrary ?? false,
    reflected?.second ?? 'none',
    equal?.partnersPair ?? false,
    equal?.middle ?? 'none',
    middle?.graha ?? 'none',
    middle?.speedDegPerDay ?? Number.NaN,
    composite?.lagnaTurned ?? false,
    composite?.cuspsDeg?.[0] ?? Number.NaN,
    davison?.place.longitude ?? Number.NaN,
    between?.instant ?? Number.NaN,
    String(alone),
    String(sidereal),
  ].join();
}

void theWesternAspects;

// A chart's declinations and parallels, read all the way down.
function theParallels(ctx: Context): string {
  const asked: ParallelRequest = { orbDeg: 1.5 };
  const chart = ctx.chart.found({
    instant: 2402390.5542,
    place: { latitude: 51.5045, longitude: -0.1366, altitude: 0 },
    utcOffsetSeconds: 0,
    parallels: asked,
  });
  // @ts-expect-error the orb is a number of degrees
  const misread: ParallelRequest = { orbDeg: '1' };
  const read: Declinations | null = chart.declinations;
  const row: ParallelRow | undefined = chart.parallels?.[0];
  return [
    read?.obliquityDeg ?? Number.NaN,
    read?.grahas[0]?.graha ?? 'none',
    read?.lagnaDeg ?? Number.NaN,
    row?.contrary ?? false,
    row?.apartDeg ?? Number.NaN,
    String(misread),
  ].join();
}

void theParallels;

// A chart's antiscia, read all the way down; the request takes the aspect
// table's orb models.
function theAntiscia(ctx: Context): string {
  const asked: AntisciaRequest = { orbs: { model: 'BY_ASPECT', orbs: [{ aspect: 'CONJUNCTION', orbDeg: 2 }] } };
  const chart = ctx.chart.found({
    instant: 2402390.5542,
    place: { latitude: 51.5045, longitude: -0.1366, altitude: 0 },
    utcOffsetSeconds: 0,
    antiscia: asked,
  });
  // @ts-expect-error an orb model is named
  const misread: AntisciaRequest = { orbs: { model: 'WIDE' } };
  const read: Antiscia | null = chart.antiscia;
  const row: AntiscionRow | undefined = read?.pairs[0];
  const onCusp: CuspAntiscion | undefined = read?.onCusps[0];
  const lilly: AntisciaRequest = { cusps: {} };
  return [
    onCusp?.house ?? 0,
    onCusp?.contrary ?? false,
    read?.cuspSystem ?? 'none',
    String(lilly),
    read?.points[0]?.antiscionDeg ?? Number.NaN,
    read?.unpaired[0] ?? 'none',
    row?.contrary ?? false,
    row?.orbDeg ?? Number.NaN,
    String(misread),
  ].join();
}

void theAntiscia;

// A chart's Western houses, read all the way down; the request names a
// catalogued division.
function theWesternHouses(ctx: Context): string {
  const asked: WesternHouseRequest = { system: 'house_system.KOCH' };
  const chart = ctx.chart.found({
    instant: 2391625.6125,
    place: { latitude: 51.5, longitude: -0.1, altitude: 0 },
    utcOffsetSeconds: 0,
    westernHouses: asked,
  });
  // @ts-expect-error a division is a catalogued house system
  const misread: WesternHouseRequest = { system: 'KOCHISH' };
  const read: WesternHouses | null = chart.westernHouses;
  const placed: WesternHousePlacement | undefined = read?.planets[0];
  return [
    read?.system ?? 'none',
    read?.cuspsDeg[11] ?? Number.NaN,
    read?.reachDeg ?? Number.NaN,
    placed?.graha ?? 'none',
    placed?.house ?? 0,
    placed?.withAscendant ?? false,
    String(misread),
  ].join();
}

void theWesternHouses;

// A chart's harmonic, read all the way down to a point's tag.
function theHarmonic(ctx: Context): string {
  const asked: HarmonicRequest = { number: 9, orbDeg: 6 };
  const chart = ctx.chart.found({
    instant: 2405857.5649,
    place: { latitude: 51.8414, longitude: -1.3611, altitude: 0 },
    utcOffsetSeconds: 0,
    harmonic: asked,
  });
  // @ts-expect-error the harmonic is required
  const misread: HarmonicRequest = { orbDeg: 6 };
  const read: HarmonicChart | null = chart.harmonic;
  const placed: HarmonicPlaced | undefined = read?.points[0];
  const row: HarmonicRow | undefined = read?.rows[0];
  const first = row?.first;
  return [
    read?.harmonic ?? 0,
    placed?.longitudeDeg ?? Number.NaN,
    placed?.house ?? 0,
    first?.point === 'GRAHA' ? first.graha : (first?.point ?? 'none'),
    row?.multiple ?? 0,
    String(misread),
  ].join();
}

void theHarmonic;

// A chart's match, read all the way down to a koota's own fields.
function theMatching(ctx: Context): string {
  const asked: MatchingRequest = {
    partner: { instant: 2447892.5, place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 } },
    partnerRole: 'BRIDE',
    rules: { nadiDosha: 'MIDDLE_ONLY' },
    porutham: { deerghaBeyond: 'SEVENTH' },
    kuja: { houses: 'WITH_SECOND', from: 'LAGNA_MOON_VENUS' },
  };
  const chart = ctx.chart.found({
    instant: 2451545.0,
    place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
    utcOffsetSeconds: 20700,
    matching: asked,
  });
  // @ts-expect-error the partner's side is required
  const unsided: MatchingRequest = { partner: asked.partner };
  // @ts-expect-error a rule takes only its own members
  const misruled: MatchingRequest = { ...asked, rules: { nadiDosha: 'MIDDLE' } };
  const read: AshtaKoota | null = chart.matching;
  const reading: KootaReading | undefined = read?.kootas[6]?.reading;
  const bhakoot = reading?.koota === 'koota.BHAKOOT' ? `${reading.apart} ${reading.dosha ?? 'NONE'} ${reading.exceptions.taraPure}` : '';
  const nadi = read?.kootas[7]?.reading;
  // @ts-expect-error a porutham rule takes only its own members
  const misreach: MatchingRequest = { ...asked, porutham: { deerghaBeyond: 'NINTH' } };
  const ten: Porutham | null = chart.porutham;
  const dhinam: PoruthamReading | undefined = ten?.considerations[0]?.reading;
  const rajju = ten?.considerations[8]?.reading;
  // @ts-expect-error a Kuja rule takes only its own members
  const misplaced: MatchingRequest = { ...asked, kuja: { from: 'SUN' } };
  const mars: Kuja | null = chart.kuja;
  const fromLagna: KujaReading | undefined = mars?.bride.readings[0];
  const first: MarriageDosha | undefined = chart.marriageDoshas?.[0];
  return [
    `${first?.system ?? ''} ${first?.koota ?? 'NONE'} ${first?.side ?? 'NONE'} ${first?.lifted ?? false}`,
    `${fromLagna?.from ?? ''} ${fromLagna?.house ?? 0} ${fromLagna?.inHouses ?? false}`,
    mars?.groom.dosha ?? false,
    mars?.both ?? false,
    String(misplaced),
    ten?.agreeing ?? 0,
    ten?.chiefAgreeing ?? 0,
    ten?.exception.opposite ?? false,
    ten?.considerations[0]?.lifted ?? false,
    dhinam?.koota === 'koota.TARA' ? `${dhinam.count} ${dhinam.rule}` : '',
    rajju?.koota === 'koota.RAJJU' ? `${rajju.bride} ${rajju.groom}` : '',
    String(misreach),
    read?.total ?? 0,
    read?.kootas[0]?.maxPoints ?? 0,
    bhakoot,
    nadi?.koota === 'koota.NADI' ? `${nadi.bride} ${nadi.dosha}` : '',
    String(unsided),
    String(misruled),
  ].join();
}

void theMatching;

// Two names matched, read all the way down to a koota's own fields.
function theNaam(ctx: Context): string {
  const rules: NaamRules = { name: { latin: 'IAST', abhijit: 'SHRAVANA' }, koota: { nadiDosha: 'MIDDLE_ONLY' } };
  // @ts-expect-error a name rule takes only its own members
  const misread: NaamRules = { name: { latin: 'ENGLISH' } };
  // @ts-expect-error Abhijit is placed in one of its two neighbours
  const misplaced: NaamRules = { name: { abhijit: 'ABHIJIT' } };
  const read: NaamMilan = ctx.matching.naam('sītā', 'rāma', rules);
  const bride: NameSyllable = read.bride;
  const tara = read.ashta.kootas[2]?.reading;
  const rajju = read.porutham.considerations[8]?.reading;
  return [
    `${bride.cell} ${bride.nakshatra ?? 'ABHIJIT'} ${bride.quarter} ${bride.varga}`,
    `${read.varga.bride} ${read.varga.groom} ${read.varga.relation}`,
    read.ashta.total,
    tara?.koota === 'koota.TARA' ? `${tara.brideToGroom}` : '',
    read.porutham.agreeing,
    rajju?.koota === 'koota.RAJJU' ? `${rajju.bride} ${rajju.groom}` : '',
    String(misread),
    String(misplaced),
  ].join();
}

void theNaam;

// A name and a date read under numerology, all the way down to a letter.
function theNumerology(ctx: Context): string {
  const rules: NumerologyRules = { masters: 'NONE', nonLatin: 'SKIP' };
  // @ts-expect-error a masters reading takes only its own members
  const misread: NumerologyRules = { masters: 'ELEVEN' };
  const read: NumerologyProfile = ctx.numerology.profile('Henry Elder', { year: 1872, month: 1, day: 17 }, rules);
  const first = read.pythagoreanName.words[0]?.letters[0];
  return [
    `${first?.letter ?? ''}${first?.value ?? 0}`,
    read.pythagoreanName.reduction.steps.join('/'),
    read.chaldeanName.compound ?? 'NONE',
    read.pythagoreanBirth.sum?.number ?? 'NONE',
    read.pythagoreanBirth.apart.join(','),
    read.chaldeanBirth.year.number,
    read.baseline?.chaldeanDestiny.number ?? 'NONE',
    String(misread),
  ].join();
}

void theNumerology;

// A chart's avakahada, read down to its syllable's own fields.
function theAvakahada(ctx: Context): string {
  const place = { latitude: 27.7, longitude: 85.3 };
  const read: Avakahada | null = ctx.chart.found({ instant: 2460482.5, place, utcOffsetSeconds: 20700, avakahada: true })
    .avakahada;
  if (read === null) return '';
  const syllable: BirthSyllable = read.syllable;
  const gana: Gana = read.gana;
  return [
    `${read.nakshatra} ${read.pada} ${read.rashi} ${read.nakshatraLord} ${read.rashiLord}`,
    `${read.varna} ${read.yoni} ${gana} ${read.nadi}`,
    `${syllable.cell} ${syllable.devanagari} ${syllable.iast} ${syllable.varga}`,
  ].join();
}

void theAvakahada;

// A lunar and a solar return, asked through the hit list.
function theReturns(ctx: Context): string {
  const lunar: HitRequest = returnsRequest(2451546, 2451911.25);
  const both: HitRequest = returnsRequest(2451546, 2451911.25, ['SUN', 'graha.MOON']);
  // @ts-expect-error a graha's name, not a number
  const misread = returnsRequest(2451546, 2451911.25, [1]);
  const hits = ctx.chart.found({
    instant: 2451545,
    place: { latitude: 48.8534, longitude: 2.3488, altitude: 0 },
    utcOffsetSeconds: 0,
    hits: both,
  }).hits;
  const back = hits.filter(
    (hit) => hit.event.kind === 'ASPECT' && hit.event.to.point === 'GRAHA' && hit.event.to.graha === hit.graha,
  );
  return [lunar.grahas?.length ?? 0, back.length, String(misread)].join();
}

void theReturns;

// A chart's equal distances, read all the way down.
function theMidpoints(ctx: Context): string {
  const asked: MidpointRequest = { orbDeg: 1 };
  const chart = ctx.chart.found({
    instant: 2402390.5542,
    place: { latitude: 51.5045, longitude: -0.1366, altitude: 0 },
    utcOffsetSeconds: 0,
    midpoints: asked,
  });
  // @ts-expect-error the orb is a number of degrees
  const misread: MidpointRequest = { orbDeg: '1' };
  const row: MidpointRow | undefined = chart.midpoints?.[0];
  return [
    row?.middle ?? 'none',
    row?.far ?? false,
    row?.distanceDeg ?? Number.NaN,
    row?.fromAxisDeg ?? Number.NaN,
    String(misread),
  ].join();
}

void theMidpoints;

// A festival answer read all the way down, and a rule written the way the
// shipped pack is, its catalogue members in full as answers give them
// (`03-design/festival-rules.md` §7).
function theFestivals(almanac: Almanac): string {
  const answer: FestivalAnswer | null = almanac.festivals;
  if (answer === null) return '';
  const said = answer.observances.map((o) => {
    const decided = o.decidedBy;
    const by =
      decided.by === 'GUARD'
        ? `guard ${decided.index}`
        : decided.by === 'AFTER'
          ? `${decided.days} after ${decided.rule}`
          : 'otherwise';
    const [earlier, later] = o.extents;
    const month: Masa = o.month;
    const adhika: boolean = o.adhika;
    return `${o.rule} ${month} ${adhika} ${o.day.calendar} ${o.day.month}/${o.day.day} ${o.case} ${by} ${o.choice} ${earlier.held} ${later.window.from}`;
  });
  const fasts = answer.ekadashis.map((fast) => {
    const [first, second] = fast.days;
    const [, eleventh] = fast.tithis;
    const at: EkadashiVedha | null = fast.piercedAt;
    return `${fast.rule} ${fast.tithi} ${fast.month} ${fast.adhika} ${eleventh.from} ${first.day} ${second.day} ${at} ${fast.pierced} ${fast.excess} ${fast.choice} ${fast.day.day}`;
  });
  const unjudged = answer.unjudged.map((u) => `${u.rule} ${u.tithi.from} ${u.why}`);
  const madhava: EkadashiRule = {
    key: 'EKADASHI_SMARTA_HEMADRI',
    source: 'Dharmasindhu p. 12, Hemadri',
    vedha: 'SUNRISE',
    table: {
      pure: { eleventh: 'EARLIER', twelfth: 'LATER', both: 'LATER', neither: 'EARLIER' },
      pierced: { eleventh: 'EARLIER', twelfth: 'LATER', both: 'LATER', neither: 'EARLIER' },
    },
  };
  // @ts-expect-error a table's day is the earlier or the later, not a yugma
  const yugma: EkadashiKinds = { eleventh: 'BY_YUGMA', twelfth: 'LATER', both: 'LATER', neither: 'EARLIER' };
  const rule: FestivalRule = {
    key: 'LAKSHMI_PUJA',
    source: 'Dharmasindhu p. 77',
    month: 'masa.ASHWINA',
    tithi: 'tithi.AMAVASYA',
    at: { window: 'PRADOSHA' },
    decide: [{ when: [{ is: 'LASTS', day: 'LATER', from: 'SUNSET', ghatis: 1 }], choose: 'LATER' }],
    otherwise: 'EARLIER',
  };
  // Shivaratri as the Kaustubha reads it: both nights holding niśītha
  // take the earlier, as does the earlier holding it whole.
  const kaustubha: FestivalRule = {
    key: 'SHIVARATRI',
    source: 'Dharmasindhu p. 90, the Kaustubha',
    month: 'masa.MAGHA',
    tithi: 'tithi.KRISHNA_CHATURDASHI',
    at: { window: 'NIGHT_MUHURTA', muhurta: 8 },
    decide: [
      { when: [{ is: 'CASE', case: 'BOTH' }], choose: 'EARLIER' },
      { when: [{ is: 'WHOLLY', day: 'EARLIER' }], choose: 'EARLIER' },
    ],
    otherwise: 'LATER',
  };
  // A full moon kept every month, judged at the instant of sunset: a rule
  // that names no month is kept in each.
  const everyMonth: FestivalRule = {
    key: 'EVERY_FULL_MOON',
    source: 'mine',
    tithi: 'tithi.PURNIMA',
    inAdhika: true,
    at: { window: 'SUNSET' },
    decide: [{ when: [{ is: 'CASE', case: 'EARLIER_ONLY' }], choose: 'EARLIER' }],
    otherwise: 'LATER',
  };
  // @ts-expect-error a month is a member, not a number
  const numbered: FestivalRule = { ...everyMonth, month: 5 };
  // The Samavedis' upakarma, kept on a nakshatra in a paksha.
  const samavedi: FestivalRule = {
    key: 'UPAKARMA_SAMAVEDI',
    source: 'Dharmasindhu p. 47',
    month: 'masa.BHADRAPADA',
    nakshatra: 'nakshatra.HASTA',
    paksha: 'paksha.SHUKLA',
    at: { window: 'PART', part: 'APARAHNA' },
    decide: [{ when: [{ is: 'CASE', case: 'EARLIER_ONLY' }, { is: 'WHOLLY', day: 'EARLIER' }], choose: 'EARLIER' }],
    otherwise: 'LATER',
  };
  // @ts-expect-error a rule is kept on a tithi or a nakshatra, not both
  const both: FestivalRule = { ...samavedi, tithi: 'tithi.PURNIMA' };
  // @ts-expect-error a nakshatra names its paksha
  const halfless: FestivalRule = { key: 'X', source: 'x', nakshatra: 'nakshatra.HASTA', at: { window: 'SUNRISE' }, decide: [], otherwise: 'LATER' };
  // The Terai's Holi, a day after the Holika fire.
  const terai: FollowingRule = { key: 'HOLI_TERAI', source: 'the committee', after: 'HOLIKA', days: 1 };
  const asked: FestivalRequest = { rules: ['NEPAL', rule, madhava, kaustubha, everyMonth, samavedi, terai] };
  const pack: FestivalPack = 'NEPAL';
  // @ts-expect-error a following rule counts days, a number
  const spelt: FollowingRule = { ...terai, days: 'one' };
  // @ts-expect-error a night muhurta names which one
  const unnumbered: FestivalRule = { ...kaustubha, at: { window: 'NIGHT_MUHURTA' } };
  // @ts-expect-error a predicate asks what its kind asks, not another's
  const wrong: FestivalRule = { ...rule, decide: [{ when: [{ is: 'CASE', day: 'LATER' }], choose: 'LATER' }] };
  // @ts-expect-error an unshipped pack is not one
  const unshipped: FestivalRequest = { rules: 'NIRNAYA_SINDHU' };
  return [...said, ...fasts, ...unjudged, String(yugma), String(asked.rules.length), String(wrong), String(unshipped), String(unnumbered), String(numbered), String(both), String(halfless), pack, String(spelt), answer.provenance.contentHash].join();
}

void theFestivals;

// A muhurta answer read all the way down, and a clause it gives handed back
// as a bar (`03-design/muhurta-at-the-boundary.md` §2.5): the request and
// the answer share one union, so what one gives the other takes.
function theMuhurta(almanac: Almanac): string {
  const answer: MuhurtaAnswer | null = almanac.muhurta;
  if (answer === null) return '';
  const windows = answer.windows.map((window) => {
    const clauses = window.clauses.map((clause) => {
      switch (clause.clause) {
        case 'NAKSHATRA':
          return `${clause.nakshatra} ${clause.grade} ${clause.at.from}`;
        case 'MUHURTA_YOGA':
          return clause.yoga;
        case 'PADA':
          return `${clause.pada.nakshatra} ${clause.pada.pada}`;
        case 'TARABALA':
          return `${clause.reading.tara} ${clause.reading.count}`;
        case 'KUNAVAMSA':
          return `${clause.navamsa} ${clause.lord}`;
        case 'KENDRA_BENEFICS':
          return clause.grahas.join();
        case 'UNWANTED_PLACEMENT':
          return `${clause.house} ${clause.by.join()}`;
        default:
          return clause.clause;
      }
    });
    const bars = window.barredBy.map((bar) => (typeof bar === 'string' ? bar : bar.clause));
    const factors = window.score?.factors.map((f) => `${f.dimension} ${f.weight} ${f.graha ?? ''}`) ?? [];
    return [...clauses, ...bars, ...factors, String(window.score?.cappedAt)].join();
  });
  const closedBy: readonly BlackoutKind[] = answer.closed.flatMap((day) => day.by);
  const closed = answer.closed.map((day) => `${day.date.calendar} ${day.date.month} ${day.by.join()}`);
  const found = answer.windows[0]?.clauses[0];
  if (found === undefined) return '';
  const { at: _, ...first } = found;
  const bar: MuhurtaBar = first;
  const asked: MuhurtaRequest = {
    rules: 'RAMAN_MARRIAGE',
    native: { star: 'nakshatra.ROHINI', moonSign: 'rashi.TAURUS', lagna: null },
    ranking: answer.ranking,
    asta: { kind: 'TIME_DEGREES', thresholds: { kind: 'CUSTOM', venus: { direct: 10, retrograde: 8 } } },
  };
  // @ts-expect-error a clause's member is its kind's, not another's
  const wrong: MuhurtaBar = { clause: 'MUHURTA_YOGA', yoga: 'yoga.SIDDHI' };
  // @ts-expect-error a shipped set is named, not guessed
  const guessed: MuhurtaRequest = { rules: 'RAMAN' };
  const thread: MuhurtaRequest = { rules: 'RAMAN_UPANAYANA' };
  const unwanted: MuhurtaUnwanted = { grahas: ['graha.MARS', 'graha.SATURN'], houses: [2, 5, 12], bars: true };
  // @ts-expect-error a house is a number, not a sign
  const misplaced: MuhurtaUnwanted = { grahas: ['graha.MARS'], houses: ['ARIES'] };
  const knobs = answer.provenance.appliedConventions.map((c) => c.knob);
  return [...windows, ...closed, ...closedBy, JSON.stringify(bar), JSON.stringify(asked), String(wrong), String(guessed), JSON.stringify(thread), JSON.stringify(unwanted), String(misplaced), ...knobs, answer.daysJudged, answer.unjudged.map((u) => u.what)].join();
}

void theMuhurta;
