// The ergonomic layer's declarations, type-checked with the rest.
//
// Its own file so a change to `index.d.ts` fails here rather than in an
// application. Every `@ts-expect-error` is a proof, as in `consumer.ts`.

import type {
  Almanac,
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
  LayoutHolds,
  LayoutKey,
  LayoutRow,
  PositionsRequest,
  Scale,
} from '../lib/index.js';
import { ChartLayout, Point, Varga, altitude, latitude, longitude } from '../lib/catalogue.js';
import type { Graha, Saham, SahamStrong, SahamWeak } from '../lib/catalogue.js';
import type { CalendarDate } from '../lib/index.js';

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
    bodies: ['graha.SUN' as unknown as Body, 'moon'],
    speeds: true,
  };
  const positions = ctx.positions(request);
  const sun = positions.at(0, 0);
  const rendered = ctx.intl.render('sdk.reason.grahaInBhava', { bhava: 7 });
  const date = ctx.calendar.dateOf('calendar.GREGORIAN' as Calendar, 735702);
  const converted = ctx.calendar.convert(date, 'calendar.BIKRAM_SAMBAT' as Calendar);
  const delta = ctx.time.deltaT(2451544.5);
  const conversion = ctx.time.convert(2451544.5, 'utc' as Scale, 'tt' as Scale);
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
  bodies: ['sun', 'moon'],
  jdMin: 2451545,
  jdMax: 2460000,
  positions(request) {
    const cells = request.jds.length * request.bodies.length;
    if (request.frameBits !== 0) return null;
    return { lon: new Float64Array(cells), status: new Int32Array(cells) };
  },
};

// @ts-expect-error a provider names itself and its bodies
const nameless: EphemerisProvider = { bodies: ['sun'], positions: () => null };
const wrongBody: EphemerisProvider = {
  name: 'x',
  // @ts-expect-error a body is named by its key, not its id
  bodies: [0],
  positions: () => null,
};
const wrongAnswer: EphemerisProvider = {
  name: 'x',
  bodies: ['sun'],
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
  return `${lagna} ${vara} ${bhava} ${part} ${elapsed} ${madhya} ${count} ${every.length} ${first.instant}`;
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
  const curved: boolean = firstStep?.kind === 'arc' && firstStep.clockwise;
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
  const rings: number = row.shape.kind === 'radial' ? row.shape.rings.length : row.shape.cells.length;
  // @ts-expect-error a layout key names its kind
  own.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, drawings: [{ layout: 'ACME_KERALA', varga: Varga.D1 }] });
  // @ts-expect-error a cell holds a sign by its bare key, not a number
  const wrong: LayoutHolds = { kind: 'sign', value: 1 };
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
    varsha: { through: 2, place: 'birth', varshesha: { moon: 'passed_over' } },
  }).praveshas[0]!;
  const annual = year.annual!;
  const lord: string = `${annual.yearLord.graha} ${annual.yearLord.chosen}`;
  const bala: number = annual.yearLord.vishwa.units + annual.yearLord.vishwa.total;
  const claim = annual.yearLord.claims[0]!;
  const held: number = claim.portfolios;
  const aspects: boolean = claim.aspectsLagna;
  const pair = annual.yogas[0];
  const orb: number = pair?.orbDeg ?? 0;
  const muntha: string = `${year.muntha.sign} ${year.muntha.lord}`;
  return `${lord} ${bala} ${held} ${aspects} ${orb} ${muntha} ${annual.byDay}`;
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
      yogas: { drishti: { subDegree: 'ishrafa' }, weakBelow: 5 * 3600, tambira: 'either_lord', moonBenefic: 'waxing' },
      varshesha: { noneAspects: 'annual_lagna_lord' },
    },
  }).praveshas[0]!.annual!;
  const matter = annual.matters[0]!;
  const promised: boolean | null = matter.holds('ithasala');
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
      sahams: ['punya', 'vivaha'],
      sahamRules: { addSign: 'signs', houses: 'equal', roga: 'saturn' },
    },
  }).praveshas[0]!.annual!;
  const one = annual.sahams[0]!;
  const saham: Saham | 'unknown' = one.saham;
  const deg: number = one.longitudeDeg;
  const added: boolean = one.addedSign;
  // @ts-expect-error a saham is one of the forty-one keys, not a word of its own
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, sahams: ['pnya'] } });
  // @ts-expect-error the rules' keys are camelCase
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, sahamRules: { add_sign: 'never' } } });
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
      dashaRules: { clock: { days: 365 }, balance: 'entry_moon', measure: 'TEMPORAL', birthPeriod: 'ELAPSED', depth: 3 },
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
      sahamStrength: { natures: 'parashari', friendship: 'natural', weakBelow: 4 * 3600 },
      harshaRules: { venus: 'twelfth' },
    },
  });
  const weak: readonly (SahamWeak | 'unknown')[] = chart.sahams[0]?.weak ?? [];
  // @ts-expect-error natures are the chapter's or the catalogue's, not a word of its own
  ctx.chart.found({ instant: 0, place: { latitude: 0, longitude: 0 }, utcOffsetSeconds: 0, varsha: { through: 1, sahamStrength: { natures: 'vedic' } } });
  return weak.join();
}

void theBirthsSahams;

void theYearsSahams;
