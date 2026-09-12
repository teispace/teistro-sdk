// A Bikram Sambat calendar page, and why the conversions are not arithmetic.
//
// The Nepali calendar is not a formula. Its month lengths are decided by
// where the Sun stands at the moment a month begins, so they vary year to
// year — Baisakh is 30 or 31 or 32 days depending on the year — and the
// authoritative table only covers BS 1970 to 2095. Outside that span the
// SDK computes the months from the Surya Siddhanta as the text prints it.
//
// Every date the SDK returns therefore says **how it was decided**:
// `tabular` from the official table, `computed` from the engine, or
// `divergent` where the two disagree and the table wins. A calendar
// application that shows a date without showing that is hiding the one
// thing a user might need to know.

import { Calendar, Context, date } from '../lib/index.js';
import { messages } from '../lib/messages.js';

/** The days of the week, from the boundary's ISO numbering. */
const WEEK = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

const two = (value) => String(value).padStart(2, '0');

/**
 * One BS month as a calendar grid.
 *
 * Every cell is a real date the SDK converted, not a number counted up: a
 * month that gains or loses a day at either end is then right by
 * construction.
 */
function monthPage(ctx, year, month) {
  const length = ctx.calendar.monthLength(Calendar.BikramSambat, year, month);
  const first = date(Calendar.BikramSambat, year, month, 1);
  // ISO weekday 1..7; a calendar page starts on Monday, so the first of
  // the month sits at column `weekday - 1`.
  const lead = ctx.calendar.weekdayOf(first) - 1;

  const cells = [
    ...Array.from({ length: lead }, () => ''),
    ...Array.from({ length }, (_, index) => String(index + 1)),
  ];
  const lines = [WEEK.map((name) => name.padStart(3)).join('  ')];
  for (let at = 0; at < cells.length; at += 7) {
    lines.push(cells.slice(at, at + 7).map((cell) => cell.padStart(3)).join('  '));
  }
  return lines.join('\n');
}

/** A date with its era and how it was decided. */
const described = (day) =>
  `${day.year}-${two(day.month)}-${two(day.day)}` +
  `${day.era ? ` ${day.era.split('.').at(-1)} ${day.eraYear}` : ''}` +
  ` [${day.resolution}]`;

const ctx = new Context({ profile: 'nepali-default', locale: 'ne-Deva-NP' });
const say = messages({
  render: (key, params) => ctx.intl.render(key, params).text,
  entity: (key) => ctx.intl.entity(key),
});
const year = 2082;

// ── A whole year, with its Gregorian spans ─────────────────────────────
console.log(`BS ${year}`);
let total = 0;
for (let month = 1; month <= 12; month++) {
  const length = ctx.calendar.monthLength(Calendar.BikramSambat, year, month);
  total += length;
  const starts = ctx.calendar.convert(date(Calendar.BikramSambat, year, month, 1), Calendar.Gregorian);
  const ends = ctx.calendar.convert(date(Calendar.BikramSambat, year, month, length), Calendar.Gregorian);
  const name = say.sdk.calendar.bikramSambat.monthName({ month });
  console.log(
    `  ${String(month).padStart(2)}  ${name.padEnd(10)} ${String(length).padStart(2)} days   ` +
      `${starts.year}-${two(starts.month)}-${two(starts.day)}` +
      ` to ${ends.year}-${two(ends.month)}-${two(ends.day)}`,
  );
}
console.log(`      ${''.padEnd(10)} ${total} days in the year`);
// A BS year is 365 or 366 days like any solar year, but its months are
// not: the shortest here is 29 days and the longest 32, which is why a
// month length is asked for and never assumed.

// ── One month as a page ────────────────────────────────────────────────
console.log('');
console.log(`Baisakh ${year}`);
console.log(monthPage(ctx, year, 1));

// ── The round trip, and what each date says about itself ───────────────
console.log('');
const newYear = date(Calendar.BikramSambat, year, 1, 1);
const gregorian = ctx.calendar.convert(newYear, Calendar.Gregorian);
const back = ctx.calendar.convert(gregorian, Calendar.BikramSambat);
console.log(`  BS   ${described(newYear)}`);
console.log(`  ->   ${described(gregorian)}`);
console.log(`  ->   ${described(back)}`);
console.log(`  fixed day ${ctx.calendar.fixedOf(newYear)}, weekday ${ctx.calendar.weekdayOf(newYear)}`);

// ── Inside the table, and outside it ───────────────────────────────────
// A date a caller *states* is always `defined`: it is what was asked for.
// A date the SDK *returns* says how it was decided, so the resolution to
// read is the one on the answer.
console.log('');
for (const asked of [2082, 2200, 1960]) {
  const greg = ctx.calendar.convert(date(Calendar.BikramSambat, asked, 1, 1), Calendar.Gregorian);
  const answer = ctx.calendar.convert(greg, Calendar.BikramSambat);
  console.log(
    `  BS ${asked} began ${greg.year}-${two(greg.month)}-${two(greg.day)},` +
      ` and the answer is [${answer.resolution}]`,
  );
}
console.log(
  "       the official table runs BS 1970 to 2095; on either side the SDK's" +
    ' own engine answers, and says so',
);

// ── The typed message accessors ────────────────────────────────────────
// A date rendered for a reader goes through the locale, not through
// string concatenation: the key is spelled once, in the generator, and
// the parameters are typed.
console.log('');
console.log(
  `  rendered  ${say.sdk.calendar.bikramSambat.date.long({
    day: 1,
    monthName: say.sdk.calendar.bikramSambat.monthName({ month: 1 }),
    year,
  })}`,
);

ctx.dispose();
