// What a consumer of this adapter writes, type-checked at the same
// strictness the SDK's own consumer file is. It is not run: what is
// being held is that the descriptor and the typed façade compose, and
// that every operation's arguments and answer are the types the engine's
// description gives them.
//
// `cargo xtask check-node` compiles this; nothing here needs the
// adapter's platform binary, because a type is not a call.

import { Context } from '@teistro/sdk';
import teimeris, { engine, binary, platformPackage, type TmDatetime, type TmPosition, type TmStar } from '../index.js';

// The descriptor is what an `ephemeris` chain takes.
const sdk = new Context({
  profile: 'parashari-classical',
  ephemeris: [teimeris({ dataDir: './ephe' }), 'BUILTIN'],
});

const typed = engine(sdk.engine);

// One value comes back as itself, typed.
const name: string = typed.tmBodyName({ body: 0 });
const seconds: number = typed.tmDeltaT({ jdUt1: 2451545 });
const formatted: string = typed.tmAngleFormat({ deg: 35.5, style: 1, decimals: 0 });

// More than one comes back as a record.
const version: { readonly major: number; readonly minor: number; readonly patch: number } =
  typed.tmVersion();

// A struct crosses as an interface, both ways, spelled as Node spells
// things; one the engine takes a null for may be left out.
const utc: TmDatetime = typed.tmLocalToUtc({
  local: { year: 2026, month: 9, day: 13, hour: 6, minute: 30, second: 0 },
  utcOffsetHours: 5.75,
  cal: 1,
});
const perihelion: number = typed.tmNodesApsidesCalc({
  jd: 2461296.5, scale: 1, body: 4, flags: 0, method: 0, apsis: 0,
}).perihelion.lonSpeed;

// An array crosses as a readonly array: as long as the inputs, as many
// as asked, or as many as there are — and no caller passes a capacity
// for an answer whose length is already decided.
const deltas: readonly number[] = typed.tmDeltaTMany({ jdsUt1: [2451545, 2461296.5] });
const grid: readonly TmPosition[] = typed.tmPositionCalcGrid({
  bodies: [0, 1], jds: [2451545], scale: 1, flags: 0,
});
const defaults: readonly number[] = typed.tmChartDefaultBodies();

// A struct that points at another takes it as a nested object, or null;
// a batch answers each element with its own status.
const moon: TmPosition = typed.tmPositionCalc({
  req: {
    jd: 2451545, scale: 1, body: 1, flags: 0, center: 0, ayanamsha: 0, ayanamshaSet: 0,
    observer: { longitudeDeg: 85.324, latitudeDeg: 27.7172, altitudeM: 1400 },
  },
});
const query = typed.tmStarQueryInitSized();
const stars: readonly TmStar[] = typed.tmStarSearch({ query: { ...query, nameContains: 'Aldeb' } });

// Cusps are as long as the house system says, asked before the call.
const houses = typed.tmHousesCalc({
  req: { jdUt1: 2451545, geoLatDeg: 27.7172, geoLonDeg: 85.324, system: 0, flags: 0 },
});
const cusps: readonly number[] = houses.cusps;
const ascendant: number = houses.outAngles.ascendant;

// And nothing comes back as nothing.
const nothing: void = typed.tmFallbackStatsReset();

const where: string = binary();
const packaged: string = platformPackage();

sdk.dispose();

export { name, seconds, formatted, version, utc, perihelion, deltas, grid, defaults, moon, stars, cusps, ascendant, nothing, where, packaged };
