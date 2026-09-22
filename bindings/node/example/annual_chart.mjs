// The annual chart: the one instant every Tajika judgement is made from.
//
// A birth chart is cast for a birth. An **annual** chart is cast for the
// moment the Sun comes back to the longitude it held then — once a year,
// about twenty minutes earlier than the clock would say, and never on the
// birthday itself (`docs/03-design/annual-chart.md`).
//
// What it teaches:
//
// 1. **The boundary answers the instant, not the chart.** Whether the
//    annual chart is cast for the birthplace or for where you live now is
//    a question the schools answer differently, so the SDK hands you the
//    instant and you found the chart with the place you mean.
// 2. **Which longitude is a choice with a name.** `sidereal` is the
//    tradition's; `tropical` is the Western solar return and is most of a
//    circle of lagna away by the fortieth year; `mean` is the older
//    arithmetic and needs no ephemeris at all. None of them is a fallback
//    for another.
// 3. **Fewer than you asked for is the answer**, not a refusal: an
//    ephemeris that ends before your hundredth year says so by giving you
//    the years it has.
//
// The record is `birth_chart.mjs`'s own, so the two can be read side by
// side.

import { Calendar, Context, TeistroError, at, date, ianaZone } from '../lib/index.js';

const ctx = new Context({ profile: 'nepali-default', ephemeris: 'builtin' });
const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };
const birthDay = date(Calendar.Gregorian, 1990, 4, 14);
const when = ctx.time.resolve(at(birthDay, { hour: 5, minute: 30 }), ianaZone('Asia/Kathmandu'));

// ── The years a birth opens ────────────────────────────────────────────
const chart = ctx.chart.found({
  instant: when.instantJdUtc,
  place,
  utcOffsetSeconds: when.offsetSeconds,
  varsha: { reading: 'sidereal', through: 40 },
});
const years = chart.praveshas;
console.log(`returns computed: ${years.length}`);
const thirtieth = years.find((one) => one.year === 30);
console.log(`the thirtieth year opens at jd ${thirtieth.instant.toFixed(6)}`);

// A return is about a sidereal year after the last, never a calendar one.
const gaps = years.slice(1).map((one, k) => one.instant - years[k].instant);
const shortest = Math.min(...gaps);
const longest = Math.max(...gaps);
console.log(`between returns: ${shortest.toFixed(4)} to ${longest.toFixed(4)} days`);

// ── The chart of that year, cast where you choose ──────────────────────
const annual = ctx.chart.found({
  instant: thirtieth.instant,
  place,
  utcOffsetSeconds: when.offsetSeconds,
});
console.log(`natal lagna ${chart.lagnaDeg.toFixed(3)}°, annual lagna ${annual.lagnaDeg.toFixed(3)}°`);

// The year's own chart and its five office-bearers, cast where you say:
// here the birthplace, the one Tajika text read casts every chart for.
const cast = ctx.chart.found({
  instant: when.instantJdUtc,
  place,
  utcOffsetSeconds: when.offsetSeconds,
  varsha: { through: 30, place: 'birth' },
}).praveshas[29];
const short = (key) => key.split('.').pop();
const b = cast.annual.officeBearers;
const five = [b.muntha, b.janmaLagna, b.varshaLagna, b.triRashi, b.dinaRatri].map(short).join(' ');
console.log(`muntha in ${short(cast.muntha.sign)}; office-bearers ${five}, ${cast.annual.byDay ? 'by day' : 'by night'}`);

// ── The readings are named, and they are not each other ────────────────
for (const reading of ['sidereal', 'tropical', 'mean']) {
  const one = ctx.chart.found({
    instant: when.instantJdUtc,
    place,
    utcOffsetSeconds: when.offsetSeconds,
    varsha: { reading, through: 30 },
  }).praveshas;
  const apart = (one[29].instant - years[29].instant) * 24;
  console.log(`${reading.padEnd(9)} thirtieth year, ${apart.toFixed(2)} hours from the sidereal one`);
}

// ── What it refuses, and by which field ────────────────────────────────
try {
  ctx.chart.found({
    instant: when.instantJdUtc,
    place,
    utcOffsetSeconds: when.offsetSeconds,
    varsha: { reading: 'sidereal', through: 0 },
  });
} catch (error) {
  if (!(error instanceof TeistroError)) throw error;
  console.log(`refused  ${error.field}: ${error.message}`);
}

ctx.dispose();
