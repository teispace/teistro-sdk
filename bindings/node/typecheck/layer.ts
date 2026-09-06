// The ergonomic layer's declarations, type-checked with the rest.
//
// Its own file so a change to `index.d.ts` fails here rather than in an
// application. Every `@ts-expect-error` is a proof, as in `consumer.ts`.

import type { Body, Calendar, Context, PositionsRequest, Scale } from '../lib/index.js';

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
const writeCell = () => {
  // @ts-expect-error a cell is read-only, like every result
  ctx.positions({ instants: [], bodies: [] }).at(0, 0).longitude = 0;
};

export { partialObserver, scenario, write, writeCell, wrongScale };
