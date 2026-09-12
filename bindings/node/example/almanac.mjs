// A week's panchangam: the five limbs of each day, and its periods.
//
// A chart is consulted once; a panchanga every morning. This is the page
// a Nepali or Indian almanac prints, and the SDK computes it in **one
// crossing** for the whole week — consecutive days share a boundary, so
// day n's next sunrise is day n+1's sunrise, and asking for seven days
// costs much less than seven days asked for separately.
//
// What the shape teaches, and what a reader should copy:
//
//   * A limb is a **span**, not a name. "Today's tithi" is a question
//     with two answers on most days, and the SDK gives both with the
//     instant each gives way — which is what an almanac row prints.
//   * A span carries its **own** bounds as well as the clipped ones, so
//     "the tithi began yesterday at 21:05" is a fact you can print.
//   * A value a day may not have is **absent**, never a sentinel: no
//     sankranti is `null`, not Julian day zero.
//
// `ephemeris: 'builtin'` selects the analytic ephemeris the SDK carries,
// so this file runs anywhere.

import { Calendar, Context, date, ianaZone } from '../lib/index.js';

const ctx = new Context({
  profile: 'parashari-classical',
  locale: 'ne-Deva-NP',
  ephemeris: 'builtin',
});

const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };

// ── One crossing for the whole week ───────────────────────────────────
const week = ctx.almanac.of({
  from: date(Calendar.Gregorian, 2024, 6, 17),
  to: date(Calendar.Gregorian, 2024, 6, 23),
  place,
  utcOffsetSeconds: 20700,
});

// A Julian day as the local clock reads it, which is what an almanac
// prints; the offset is the one the request was made under.
const clock = (jd) => {
  const local = (jd + 20700 / 86400 + 0.5) % 1;
  const minutes = Math.round(local * 1440) % 1440;
  return `${String(Math.floor(minutes / 60)).padStart(2, '0')}:${String(minutes % 60).padStart(2, '0')}`;
};
// A locale pack names most of the catalogue and not all of it: `masa`
// and `direction` have no entries in any of the five the SDK ships, so
// an almanac falls back to the key rather than refusing to print. A
// program that must have the name in the reader's language should check
// `ctx.intl.has(...)` and say so, rather than showing a bare key.
const name = (key) => {
  try {
    return ctx.intl.entity(key).name;
  } catch {
    return key.slice(key.indexOf('.') + 1).toLowerCase().replace(/_/g, ' ');
  }
};

console.log(`${week.length} days at ${place.latitude}°N ${place.longitude}°E, one crossing`);
console.log(`calendar ${week.calendar}   model ${week.model.split(',')[0]}`);
console.log('');

for (const day of week) {
  const d = day.day;
  console.log(
    `${name(d.vara).padEnd(12)} ${d.year}-${String(d.month).padStart(2, '0')}-${String(d.dayOfMonth).padStart(2, '0')}` +
      `   sunrise ${clock(d.sunrise)}  sunset ${clock(d.sunset)}` +
      `   ${name(day.month.amanta)} ${name(day.month.paksha)}`,
  );
  // The five limbs. The vara is one of them and is the day's own; the
  // other four are spans, and a day usually has two of each.
  for (const [limb, spans] of [
    ['tithi', day.tithi],
    ['nakshatra', day.nakshatra],
    ['yoga', day.yoga],
    ['karana', day.karana],
  ]) {
    const printed = spans
      .map((span) => {
        // `whole` is the member's own span and `inside` the clipped one,
        // so a member that began yesterday says so rather than looking
        // as though it began at sunrise.
        const began = span.whole.from < span.inside.from ? '‹' : ' ';
        const ends = span.whole.to > span.inside.to ? '›' : ' ';
        return `${began}${name(span.member)} until ${clock(span.inside.to)}${ends}`;
      })
      .join('  ');
    console.log(`  ${limb.padEnd(10)} ${printed}`);
  }
  // The periods a day is planned around. Rahu kalam is the one everybody
  // checks; the choghadiya are what a shop opens on.
  const kaalas = day.kaalas
    .map((k) => `${name(k.kaala)} ${clock(k.from)}–${clock(k.to)}`)
    .join('  ');
  console.log(`  ${'kaala'.padEnd(10)} ${kaalas}`);
  const auspicious = day.choghadiya.filter((c) => c.daytime).slice(0, 3);
  console.log(
    `  ${'choghadiya'.padEnd(10)} ${auspicious.map((c) => `${name(c.choghadiya)} ${clock(c.from)}`).join('  ')} …`,
  );
  if (day.abhijit) {
    console.log(
      `  ${'abhijit'.padEnd(10)} ${clock(day.abhijit.from)}–${clock(day.abhijit.to)}` +
        `${day.abhijit.effective ? '' : '  (not effective on a Wednesday)'}`,
    );
  }
  // Absent is absent: no sankranti is null, and a Moon that did not rise
  // inside the window contributes no event at all.
  if (day.sankranti !== null) {
    console.log(`  ${'sankranti'.padEnd(10)} the Sun enters a new sign at ${clock(day.sankranti)}`);
  }
  const moon = day.moonEvents.map((e) => `${e.kind} ${clock(e.instant)}`).join('  ');
  console.log(`  ${'moon'.padEnd(10)} ${moon || '(neither rise nor set inside the window)'}`);
  console.log('');
}

// ── What the ragged layout costs a reader, which is nothing ───────────
// Each day's lists are slices of one concatenated column, found by
// adding up every earlier day's count. The layer does that sum once when
// the batch is decoded, so `day.tithi` is a slice and not a search.
const counted = [...week].reduce((total, day) => total + day.karana.length, 0);
console.log(`${counted} karanas across ${week.length} days, from one blob`);
console.log(`settings hash  ${ctx.settingsHash.slice(0, 16)}…`);

// A day on its own is the range of one unwrapped: same crossing, and the
// answer is a day rather than a list of one.
const one = ctx.almanac.day({
  date: date(Calendar.Gregorian, 2024, 6, 21),
  place,
  utcOffsetSeconds: 20700,
});
console.log(
  `almanacDay     ${name(one.day.vara)}  ${one.horas.length} horas, ${one.muhurtas.length} muhurtas, ` +
    `${one.choghadiya.length} choghadiya`,
);
ctx.dispose();
