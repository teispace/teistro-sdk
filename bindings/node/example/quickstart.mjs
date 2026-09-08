// The smallest thing that works, kept honest: `cargo xtask check-node`
// runs this file, so the README cannot drift from what the binding does.

import {
  Body,
  Calendar,
  Context,
  abiVersion,
  at,
  date,
  ianaZone,
  sdkVersion,
} from '../lib/index.js';
import { messages } from '../lib/messages.js';

console.log(`Teistro ${sdkVersion()}, ABI ${abiVersion()}`);

const ctx = new Context({
  profile: 'nepali-default',
  locale: 'ne-Deva-NP',
  testProvider: true,
});

// 14 April 2015 is 1 Baisakh 2072 BS.
const bs = ctx.convert(date(Calendar.Gregorian, 2015, 4, 14), Calendar.BikramSambat);
console.log(`${bs.year}-${bs.month}-${bs.day} ${bs.era ?? ''}`);

// A Kathmandu birth time, with the metadata a stored chart keeps.
const resolved = ctx.resolve(
  at(date(Calendar.Gregorian, 1986, 1, 1), { hour: 0, minute: 20 }),
  ianaZone('Asia/Kathmandu'),
);
console.log(
  `JD ${resolved.instantJdUtc.toFixed(6)} UTC, ` +
    `${resolved.offsetSeconds} s, tzdb ${resolved.tzdbVersion}`,
);

// The Sun and the Moon at J2000, in the SDK's canonical frame.
const sky = ctx.positions({ instants: [2451545.0], bodies: [Body.Sun, Body.Moon] });
console.log(`the Sun at ${sky.at(0, 0).longitude.toFixed(4)} degrees`);

// A message in the context's locale, by its typed accessor, and an
// entity's name in that locale.
const say = messages({ render: (key, params) => ctx.render(key, params).text, entity: (key) => ctx.entity(key) });
console.log(say.sdk.reason.grahaInBhava({ graha: 'graha.JUPITER', bhava: 7 }));
console.log(`${ctx.entity('graha.SUN').name} ${ctx.entity('graha.SUN').glyph}`);

ctx.dispose();
