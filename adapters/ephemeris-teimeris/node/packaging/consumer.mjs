// A consumer of the published adapter package, and nothing else.
//
// The adapter's own type-check imports `../index.js`, so it proves the
// code and not the package: the name, the export map, the `files` list
// and the generated façade being in it are only exercised by importing
// `@teistro/ephemeris-teimeris` from a project that installed it. That is
// what this file does, and `cargo xtask check-package` runs it inside a
// throwaway project built from the packed tarballs.
//
// It needs the engine's **data**, which a checkout of the SDK does not
// have, so the gate passes `TEISTRO_TEIMERIS_DATA` and skips the whole
// step when it has none — the same shape every plugin test uses.

import assert from 'node:assert/strict';

import { Context } from '@teistro/sdk';
// From the subpath, which is the other thing only an install exercises.
import { Body } from '@teistro/sdk/catalogue';
import teimeris, { engine, binary, platformPackage } from '@teistro/ephemeris-teimeris';

console.log(`the adapter would come from ${platformPackage()}`);
console.log(`the binary it found is ${binary()}`);

const dataDir = process.env.TEISTRO_TEIMERIS_DATA;
const descriptor = teimeris(dataDir ? { dataDir } : {});
assert.equal(typeof descriptor.plugin, 'string', 'a descriptor names a binary');

const sdk = new Context({ profile: 'parashari-classical', ephemeris: [descriptor] });

// The engine answered, which is the point: a package that installs and
// cannot compute is worse than one that fails to install.
const sky = sdk.positions({ instants: [2451545.0], bodies: [Body.Sun] });
const sun = sky.at(0, 0).longitude;
console.log(`the Sun at J2000 is ${sun.toFixed(4)}`);
assert.ok(Math.abs(sun - 280.37) < 0.5, `the Sun at J2000 came back as ${sun}`);

// And the generated façade came with it, typed and installed.
const typed = engine(sdk.engine);
assert.equal(typed.tmBodyName({ body: 0 }), 'Sun');
assert.ok(Math.abs(typed.tmDeltaT({ jdUt1: 2451545.0 }) - 63.83) < 0.1);
console.log(`the façade answers: tmBodyName(0) = ${typed.tmBodyName({ body: 0 })}`);

sdk.dispose();
console.log('the installed adapter package answers as the engine does');
