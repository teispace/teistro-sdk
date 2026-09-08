// Rectification: a birth time known only to the hour, narrowed by lagna.
//
// A birth record that says "some time before dawn" is the commonest hard
// case in the field. What narrows it is the **lagna** — it moves through
// all twelve signs in a day, so it changes sign every couple of hours,
// and a family that remembers the ascendant remembers something the
// clock does not.
//
// The point of this example is the shape of the call. A rectification
// pass wants many charts at one place, and `foundMany` founds them in
// **one crossing**: the settings are resolved once, the solar model is
// built once, and the day each instant belongs to is reckoned against
// the same sunrise. Founding them one at a time would give the same
// numbers and pay the setup for every one of them.
//
// `testProvider: true` selects the analytic ephemeris the SDK carries,
// so this file runs anywhere.

import { Calendar, ChartKind, Context, Graha, RashiById, at, date, ianaZone } from '../lib/index.js';

const ctx = new Context({
  profile: 'parashari-classical',
  locale: 'ne-Deva-NP',
  testProvider: true,
});

// The record: a Bikram Sambat date, a place, and an hour nobody is sure
// of. Everything below narrows the last of those.
const born = date(Calendar.BikramSambat, 2045, 9, 17);
const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
const window = { fromHour: 0, toHour: 3, everyMinutes: 10 };

// One resolution fixes the zone and the offset; the candidates are then
// arithmetic on the instant, which is what a Julian day is for.
const start = ctx.resolve(at(born, { hour: window.fromHour, minute: 0 }), ianaZone('Asia/Kathmandu'));
const step = window.everyMinutes / (24 * 60);
const count = ((window.toHour - window.fromHour) * 60) / window.everyMinutes;
const instants = Array.from({ length: count }, (_, i) => start.instantJdUtc + i * step);

// ── One crossing for every candidate ───────────────────────────────────
const charts = ctx.foundMany({
  instants,
  place,
  utcOffsetSeconds: start.offsetSeconds,
  kind: ChartKind.Natal,
});

const two = (value) => String(value).padStart(2, '0');
const clock = (index) => {
  const minutes = window.fromHour * 60 + index * window.everyMinutes;
  return `${two(Math.floor(minutes / 60))}:${two(minutes % 60)}`;
};
const rashiOf = (deg) => Math.floor(deg / 30);

console.log(
  `${charts.length} candidate charts, ${window.everyMinutes} minutes apart, in one crossing`,
);
console.log(`place  ${place.latitude}°N ${place.longitude}°E   ${charts.kind}`);
console.log('');
console.log('local   lagna        sign            moon         bhava');
console.log('─'.repeat(58));

let previous = null;
for (const chart of charts) {
  const sign = rashiOf(chart.lagnaDeg);
  const moon = chart.grahas.find((g) => g.graha === Graha.Moon);
  const rashi = ctx.entity(RashiById.get(sign));
  console.log(
    `${clock(chart.index)}   ${chart.lagnaDeg.toFixed(4).padStart(9)}°  ` +
      `${rashi.name.padEnd(14)} ${(moon?.longitudeDeg ?? 0).toFixed(4).padStart(9)}°  ` +
      `${String(moon?.house.bhava ?? 0).padStart(2)}` +
      `${previous !== null && sign !== previous ? '   ← lagna changes sign' : ''}`,
  );
  previous = sign;
}

// ── What the batch shares, and what it does not ────────────────────────
// The place, the settings, the solar model and the completion steps are
// one to a batch: they are what "the same chart at a different minute"
// holds constant. The instant, the lagna, the day and the timing are per
// chart. The provenance envelope stamps the batch as a whole, so a
// rectification run reproduces as one thing.
console.log('');
console.log(`model          ${charts.model}`);
console.log(`steps applied  ${charts.steps.join(', ')}`);
console.log(`settings hash  ${ctx.settingsHash.slice(0, 16)}…`);

// A batch of one is the ordinary case, and `found` is the same crossing
// with the batch unwrapped: the answer is a chart, not a list of one.
const single = ctx.found({
  instant: start.instantJdUtc,
  place,
  utcOffsetSeconds: start.offsetSeconds,
});
console.log('');
console.log(
  `found(one)     lagna ${single.lagnaDeg.toFixed(4)}°  ` +
    `vara ${single.day.vara}  ` +
    `ishtakaal ${single.timing.ghati}:${single.timing.pala}:${single.timing.vipala}`,
);
ctx.dispose();
