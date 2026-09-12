// What a consumer of this adapter writes, type-checked at the same
// strictness the SDK's own consumer file is. It is not run: what is
// being held is that the descriptor and the typed façade compose, and
// that every operation's arguments and answer are the types the engine's
// description gives them.
//
// `cargo xtask check-node` compiles this; nothing here needs the
// adapter's platform binary, because a type is not a call.

import { Context } from '@teistro/sdk';
import teimeris, { engine, binary, platformPackage } from '../index.js';

// The descriptor is what an `ephemeris` chain takes.
const sdk = new Context({
  profile: 'parashari-classical',
  ephemeris: [teimeris({ dataDir: './ephe' }), 'builtin'],
});

const typed = engine(sdk.engine);

// One value comes back as itself, typed.
const name: string = typed.tmBodyName({ body: 0 });
const seconds: number = typed.tmDeltaT({ jdUt1: 2451545 });
const formatted: string = typed.tmAngleFormat({ deg: 35.5, style: 1, decimals: 0 });

// More than one comes back as a record.
const version: { readonly major: number; readonly minor: number; readonly patch: number } =
  typed.tmVersion();

// And nothing comes back as nothing.
const nothing: void = typed.tmFallbackStatsReset();

const where: string = binary();
const packaged: string = platformPackage();

sdk.dispose();

export { name, seconds, formatted, version, nothing, where, packaged };
