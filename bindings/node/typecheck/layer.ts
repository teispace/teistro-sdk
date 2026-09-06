// The ergonomic layer's declarations, type-checked with the rest.
//
// Its own file so a change to `index.d.ts` fails here rather than in an
// application. Every `@ts-expect-error` is a proof, as in `consumer.ts`.

import type {
  Body,
  BuildInfo,
  Calendar,
  Context,
  EphemerisProvider,
  PositionsRequest,
  Scale,
} from '../lib/index.js';
import { altitude, latitude, longitude } from '../lib/catalogue.js';

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
  jdRange: [2451545, 2460000],
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
