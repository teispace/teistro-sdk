// Nepal's official days, and the modern reckoning beside them.
//
// Nepal's patro is computed by the Nepali committee from the Surya
// Siddhanta: the tithi, nakshatra and yoga by the text with the
// committee's bija, and the sunrise from a modern ephemeris. That is what
// `nepalOfficial` opens. The modern reckoning, `nepali-drik`, reads the
// same day from the modern sky, and the two name a different sunrise
// tithi on a few days a month.
//
// What the shape teaches, and what a reader should copy:
//
//   * The official day is a **preset**, not a recipe: spread
//     `nepalOfficial(engine)` into a context's options.
//   * The Surya Siddhanta takes the engine **after** it in the chain for
//     its sunrise; an app passes its own, `nepalOfficial(teimeris())`.
//     This file passes none, so the sunrise is the built-in ephemeris's
//     and it runs anywhere.
//   * Where the two disagree, the reason is a tithi's end against the
//     sunrise: the tithi the modern sky still has at sunrise had already
//     ended by the text's, or the other way round. Each row prints both
//     ends.

import { Calendar, Context, date, nepalOfficial } from '../lib/index.js';

const OFFSET_SECONDS = 20700;

const official = new Context({ ...nepalOfficial(), locale: 'en-Latn' });
const modern = new Context({ profile: 'nepali-drik', locale: 'en-Latn', ephemeris: 'BUILTIN' });

const place = { latitude: 27.7172, longitude: 85.324, altitude: 1400 };

// A Julian day as the local clock reads it.
const clock = (jd) => {
  const local = (jd + OFFSET_SECONDS / 86400 + 0.5) % 1;
  const minutes = Math.round(local * 1440) % 1440;
  return `${String(Math.floor(minutes / 60)).padStart(2, '0')}:${String(minutes % 60).padStart(2, '0')}`;
};
const name = (key) => official.intl.entity(key).name;

// The same month under each, one crossing each.
const month = (ctx) =>
  ctx.almanac.of({
    from: date(Calendar.Gregorian, 2025, 11, 1),
    to: date(Calendar.Gregorian, 2025, 11, 30),
    place,
    utcOffsetSeconds: OFFSET_SECONDS,
  });
const days = [...month(official)];
const others = [...month(modern)];

console.log(`official ${official.profile}, modern ${modern.profile}`);
let differ = 0;
for (let i = 0; i < days.length; i += 1) {
  // The first span of a day's limb is the one standing at its sunrise.
  const ours = days[i].tithi[0];
  const theirs = others[i].tithi[0];
  if (ours.member === theirs.member) continue;
  differ += 1;
  const d = days[i].day.date;
  console.log(
    `${d.year}-${String(d.month).padStart(2, '0')}-${String(d.day).padStart(2, '0')}` +
      `  sunrise ${clock(days[i].day.sunrise)}` +
      `  official ${name(ours.member)} until ${clock(ours.whole.to)}` +
      `  modern ${name(theirs.member)} until ${clock(theirs.whole.to)}`,
  );
}
console.log(`${differ} of ${days.length} days name another sunrise tithi`);

official.dispose();
modern.dispose();
