// The ergonomic layer's declarations, type-checked with the rest.
//
// Its own file so a change to `index.d.ts` fails here rather than in an
// application. Every `@ts-expect-error` is a proof, as in `consumer.ts`.

import type {
  Air,
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
  KpLords,
  KpReading,
  KpRequest,
  LayoutHolds,
  LayoutKey,
  LayoutRow,
  LunarEclipseHere,
  MuhurtaAnswer,
  MuhurtaBar,
  MuhurtaRequest,
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
import type { Ayanamsha, Graha, Saham, SahamStrong, SahamWeak } from '../lib/catalogue.js';
import type { CalendarDate, Confidence, PolarDay, Provenance, Step } from '../lib/index.js';
import { decodeProvenance } from '../lib/index.js';

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
    return `${o.rule} ${o.day.calendar} ${o.day.month}/${o.day.day} ${o.case} ${by} ${o.choice} ${earlier.held} ${later.window.from}`;
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
  // The Terai's Holi, a day after the Holika fire.
  const terai: FollowingRule = { key: 'HOLI_TERAI', source: 'the committee', after: 'HOLIKA', days: 1 };
  const asked: FestivalRequest = { rules: ['NEPAL', rule, madhava, kaustubha, terai] };
  const pack: FestivalPack = 'NEPAL';
  // @ts-expect-error a following rule counts days, a number
  const spelt: FollowingRule = { ...terai, days: 'one' };
  // @ts-expect-error a night muhurta names which one
  const unnumbered: FestivalRule = { ...kaustubha, at: { window: 'NIGHT_MUHURTA' } };
  // @ts-expect-error a predicate asks what its kind asks, not another's
  const wrong: FestivalRule = { ...rule, decide: [{ when: [{ is: 'CASE', day: 'LATER' }], choose: 'LATER' }] };
  // @ts-expect-error an unshipped pack is not one
  const unshipped: FestivalRequest = { rules: 'NIRNAYA_SINDHU' };
  return [...said, ...fasts, ...unjudged, String(yugma), String(asked.rules.length), String(wrong), String(unshipped), String(unnumbered), pack, String(spelt), answer.provenance.contentHash].join();
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
  const knobs = answer.provenance.appliedConventions.map((c) => c.knob);
  return [...windows, ...closed, ...closedBy, JSON.stringify(bar), JSON.stringify(asked), String(wrong), String(guessed), ...knobs, answer.daysJudged, answer.unjudged.map((u) => u.what)].join();
}

void theMuhurta;
