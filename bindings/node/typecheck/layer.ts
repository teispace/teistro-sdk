// The ergonomic layer's declarations, type-checked with the rest.
//
// Its own file so a change to `index.d.ts` fails here rather than in an
// application. Every `@ts-expect-error` is a proof, as in `consumer.ts`.

import type {
  Almanac,
  AlmanacDay,
  Body,
  BuildInfo,
  Calendar,
  Chart,
  ChartBatchRequest,
  Charts,
  Context,
  EphemerisProvider,
  PositionsRequest,
  Scale,
} from '../lib/index.js';
import { altitude, latitude, longitude } from '../lib/catalogue.js';
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
  const rendered = ctx.render('sdk.reason.grahaInBhava', { bhava: 7 });
  const date = ctx.dateOf('calendar.GREGORIAN' as Calendar, 735702);
  const converted = ctx.convert(date, 'calendar.BIKRAM_SAMBAT' as Calendar);
  const delta = ctx.deltaT(2451544.5);
  const conversion = ctx.convertTime(2451544.5, 'utc' as Scale, 'tt' as Scale);
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
  const one: Chart = ctx.found({ instant: 2460482.5, place, utcOffsetSeconds: 20700 });
  const lagna: number = one.lagnaDeg;
  const vara: string = one.day.vara;
  const bhava: number = one.grahas[0]!.house.bhava;
  const madhya: number = one.houses[0]!.madhyaDeg;

  const request: ChartBatchRequest = {
    instants: new Float64Array([2460482.5, 2460600.25]),
    place,
    utcOffsetSeconds: 20700,
  };
  const batch: Charts = ctx.foundMany(request);
  const count: number = batch.length;
  // Iterating a batch gives charts, and the same view `at` gives.
  const first: Chart = batch.at(0);
  const every: readonly Chart[] = [...batch];
  // @ts-expect-error a batch is read, never rewritten
  batch.length = 3;
  // @ts-expect-error `instant` is the singular request's; a batch takes `instants`
  ctx.foundMany({ instant: 2460482.5, place, utcOffsetSeconds: 20700 });
  // @ts-expect-error a chart is a view; its index is not a number to set
  first.index = 2;
  return `${lagna} ${vara} ${bhava} ${madhya} ${count} ${every.length} ${first.instant}`;
}

void charts;

/** The almanac layer, typed: a range of days and one of them. */
function almanac(): string {
  const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
  const from: CalendarDate = someDate;
  const to: CalendarDate = someDate;
  const batch: Almanac = ctx.almanac({ from, to, place, utcOffsetSeconds: 20700 });
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
  const one: AlmanacDay = ctx.almanacDay({ date: from, place, utcOffsetSeconds: 20700 });
  // @ts-expect-error an absent sankranti is null, so it is not a number
  const wrong: number = day.sankranti;
  // @ts-expect-error a range needs both ends; `date` is the single-day shape
  ctx.almanac({ date: from, place, utcOffsetSeconds: 20700 });
  // @ts-expect-error a batch is read, never rewritten
  batch.length = 3;
  return `${days} ${vara} ${tithi} ${until} ${lord} ${daylight} ${sankranti} ${effective} ${every.length} ${one.index} ${wrong}`;
}

void almanac;
