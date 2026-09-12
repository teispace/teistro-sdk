// The five limbs of a day, computed from the boundary alone.
//
// A panchanga is the Hindu almanac's five parts — tithi, vara,
// nakshatra, yoga and karana — and every one of them except the weekday
// is a function of **two longitudes**: the Sun's and the Moon's, in the
// sidereal zodiac. So a binding can compute a whole panchanga from
// `positions` and `weekdayOf`, without any part of the chart layer.
//
// The arithmetic is the SDK's own (`crates/panchanga/src/limb.rs`):
//
//   tithi      Moon − Sun    30 divisions of 12°
//   karana     Moon − Sun    60 divisions of 6°
//   nakshatra  Moon          27 divisions of 360/27°
//   yoga       Moon + Sun    27 divisions of 360/27°
//   vara       the weekday    7
//
// The karana is the one that is not a plain division: sixty half-tithis
// make a lunar month and they are **not** a cycle of eleven. Kimstughna
// opens the month, Shakuni, Chatushpada and Naga close it, and the seven
// movable karanas repeat through everything between. `karanaOf` below is
// the SDK's rule, transcribed, and the SDK holds it to the corpus's own
// successor relation over 109 consecutive pairs.
//
// What this example is honest about: a limb here is the one holding at
// the instant asked for. A printed almanac gives the limb at sunrise and
// the time it ends, which needs a boundary search over the Moon's motion.

import {
  Ayanamsha,
  Body,
  Calendar,
  Context,
  Karana,
  KaranaById,
  NakshatraById,
  TithiById,
  VaraById,
  YogaById,
  at,
  canonicalFrame,
  date,
  ianaZone,
} from '../lib/index.js';

const NAKSHATRA_DEG = 360 / 27;
const YOGA_DEG = 360 / 27;
const TITHI_DEG = 12;
const KARANA_DEG = 6;

/**
 * The karana that a half-tithi of the lunar month is.
 *
 * Transcribed from `crates/panchanga/src/limb.rs`. Sixty of them make a
 * month: Kimstughna opens it, Shakuni, Chatushpada and Naga close it, and
 * the seven movable karanas fill everything between, Bava first from the
 * second half of the first tithi.
 */
function karanaOf(halfTithi) {
  const half = ((halfTithi % 60) + 60) % 60;
  if (half === 0) return Karana.Kimstughna;
  if (half === 57) return Karana.Shakuni;
  if (half === 58) return Karana.Chatushpada;
  if (half === 59) return Karana.Naga;
  return KaranaById.get((half - 1) % 7);
}

/**
 * The five limbs at an instant.
 *
 * `weekday` is the ISO weekday of the **civil day** the instant belongs
 * to, which the caller has because it asked the calendar for it: a vara
 * is a property of the day, not of the moment.
 */
function panchangaAt(ctx, instant, weekday) {
  const frame = { ...canonicalFrame(), sidereal: true, ayanamsha: Ayanamsha.Lahiri };
  const sky = ctx.positions({ instants: [instant], bodies: [Body.Sun, Body.Moon], frame });
  const sun = sky.at(0, 0).longitude;
  const moon = sky.at(0, 1).longitude;
  const elongation = ((moon - sun) % 360 + 360) % 360;
  return {
    sun,
    moon,
    elongation,
    tithi: TithiById.get(Math.floor(elongation / TITHI_DEG)),
    // The boundary's weekday is ISO (Monday 1 … Sunday 7) and a vara
    // counts from Sunday, so the one becomes the other by `% 7`.
    vara: VaraById.get(weekday % 7),
    nakshatra: NakshatraById.get(Math.floor(moon / NAKSHATRA_DEG)),
    yoga: YogaById.get(Math.floor((((moon + sun) % 360) + 360) % 360 / YOGA_DEG)),
    karana: karanaOf(Math.floor(elongation / KARANA_DEG)),
    get paksha() {
      return this.elongation < 180 ? 'shukla' : 'krishna';
    },
    get tithiElapsed() {
      return (this.elongation % TITHI_DEG) / TITHI_DEG;
    },
  };
}

const two = (value) => String(value).padStart(2, '0');

const ctx = new Context({
  profile: 'nepali-default',
  locale: 'ne-Deva-NP',
  testProvider: true,
});

// Nepali New Year: the first day of Baisakh, BS 2082.
const day = date(Calendar.BikramSambat, 2082, 1, 1);
const gregorian = ctx.calendar.convert(day, Calendar.Gregorian);
// Six in the morning stands in for sunrise, which the almanac would use
// and which needs the rise-and-set solver.
const when = ctx.time.resolve(at(day, { hour: 6 }), ianaZone('Asia/Kathmandu'));
const found = panchangaAt(ctx, when.instantJdUtc, ctx.calendar.weekdayOf(day));

console.log(
  `BS ${day.year}-${two(day.month)}-${two(day.day)}` +
    `  (${gregorian.year}-${two(gregorian.month)}-${two(gregorian.day)})` +
    `  06:00 Kathmandu`,
);
console.log(
  `  sun ${found.sun.toFixed(4).padStart(8)}°` +
    `   moon ${found.moon.toFixed(4).padStart(8)}°` +
    `   elongation ${found.elongation.toFixed(4).padStart(8)}°`,
);
console.log('');

for (const [label, member] of [
  ['tithi', found.tithi],
  ['vara', found.vara],
  ['nakshatra', found.nakshatra],
  ['yoga', found.yoga],
  ['karana', found.karana],
]) {
  const entity = ctx.intl.entity(member);
  // A member is its full key here (`tithi.PURNIMA`), so the bare key is
  // the tail — which is the string the other bindings call `key`.
  const key = member.split('.').at(-1);
  console.log(
    `  ${label.padEnd(10)} ${entity.name.padEnd(14)} ${entity.iast.padEnd(18)} (${key})`,
  );
}
console.log('');
console.log(`  paksha     ${found.paksha}`);
console.log(`  tithi is   ${(found.tithiElapsed * 100).toFixed(1)}% elapsed at this instant`);

// The Sun on this day is the reason the year turns: BS begins at the
// Mesha Sankranti, when the Sun enters Aries. At six in the morning it
// has not quite arrived, which is why the almanac's own year-start is an
// instant and not a date.
console.log(
  `  the Sun stands ${(((360 - found.sun) % 360)).toFixed(4)}° short of Aries,` +
    ' which is what the new year waits for',
);

ctx.dispose();
