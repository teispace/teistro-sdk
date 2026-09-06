// A consumer of the published package, and nothing else.
//
// The binding's own tests import `../lib/index.js`, so they prove the
// code and not the package: the name, the export map, the subpath
// exports and the addon npm installed for this host are only exercised
// by importing `@teistro/sdk` from a project that installed it. That is
// what this file does, and `cargo xtask check-package` runs it inside a
// throwaway project built from the staged packages.
//
// It asserts the four facts the C smoke test prints, so that a package
// that loads but answers differently fails here rather than in the field.

import assert from 'node:assert/strict';

import { Context, buildInfo, platformPackage, sdkVersion } from '@teistro/sdk';
import { Body, Calendar, Resolution, ZoneKind } from '@teistro/sdk/catalogue';

const gregorian = (year, month, day) => ({
  calendar: Calendar.Gregorian,
  year,
  eraYear: 0,
  month,
  day,
  resolution: Resolution.Defined,
  computedMonth: 0,
  computedDay: 0,
});

console.log(`the addon came from ${platformPackage()}`);
console.log(`abi ${buildInfo.abi}, sdk ${sdkVersion()}, ${buildInfo.target}, ${buildInfo.profile}`);
assert.equal(buildInfo.sdk, sdkVersion());
assert.equal(buildInfo.optimised, true, 'a published addon is an optimised build');
assert.equal(buildInfo.sanitizer, '', 'a published addon carries no sanitizer');

const ctx = new Context({ testProvider: true, profile: 'nepali-default', locale: 'ne-Deva-NP' });

const bs = ctx.convert(gregorian(2015, 4, 14), Calendar.BikramSambat);
console.log(`14 April 2015 is ${bs.year}-${bs.month}-${bs.day} BS`);
assert.deepEqual([bs.year, bs.month, bs.day], [2072, 1, 1]);

const zone = { kind: ZoneKind.Iana, offsetSeconds: 0, longitudeDeg: 0, zone: 'Asia/Kathmandu' };
const resolved = ctx.resolve(
  { date: gregorian(1986, 1, 1), time: { hour: 0, minute: 20, second: 0, hasTime: true, nanos: 0 } },
  zone,
);
console.log(
  `00:20 on 1 January 1986 in Kathmandu is JD ${resolved.instantJdUtc.toFixed(6)} UTC,` +
    ` offset ${resolved.offsetSeconds} s, tzdb ${resolved.tzdbVersion}`,
);
assert.ok(Math.abs(resolved.instantJdUtc - 2446431.274306) < 1e-6);
assert.equal(resolved.offsetSeconds, 20700);

const rendered = ctx.render('sdk.reason.grahaInBhava', { graha: 'graha.JUPITER', bhava: 7 });
console.log(`sdk.reason.grahaInBhava in ne-Deva-NP: ${rendered.text}`);
assert.equal(rendered.text, 'गुरु ७औं भावमा');

const positions = ctx.positions({ instants: [2451545.0], bodies: [Body.Sun] });
const sun = positions.at(0, 0);
console.log(`the Sun at J2000 is at ${sun.longitude.toFixed(4)} degrees, ${positions.cells.length} cells`);
assert.equal(sun.longitude.toFixed(4), '278.5768');

console.log('the published Node package answers as the library does');
