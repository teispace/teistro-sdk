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

import 'package:teistro/teistro.dart';

/// The days of the week, from the boundary's ISO numbering.
const List<String> week = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

/// One BS month as a calendar grid.
///
/// Every cell is a real date the SDK converted, not a number counted up:
/// a month that gains or loses a day at either end is then right by
/// construction.
String monthPage(Context ctx, int year, int month) {
  final length = ctx.calendar.monthLength(Calendar.bikramSambat, year, month);
  final first = Calendar.bikramSambat.date(year, month, 1);
  // ISO weekday 1..7; a calendar page starts on Monday, so the first of
  // the month sits at column `weekday - 1`.
  final lead = ctx.calendar.weekdayOf(first) - 1;

  final cells = <String>[
    for (var i = 0; i < lead; i++) '',
    for (var day = 1; day <= length; day++) '$day',
  ];
  final lines = <String>[week.map((n) => n.padLeft(3)).join('  ')];
  for (var at = 0; at < cells.length; at += 7) {
    final row = cells.sublist(
      at,
      at + 7 > cells.length ? cells.length : at + 7,
    );
    lines.add(row.map((c) => c.padLeft(3)).join('  '));
  }
  return lines.join('\n');
}

/// A date with its era and how it was decided.
String described(CalendarDate day) {
  final era = day.era == null ? '' : ' ${day.era!.key} ${day.eraYear}';
  return '${day.year}-${_two(day.month)}-${_two(day.day)}$era'
      ' [${day.resolution.key}]';
}

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(profile: 'nepali-default', locale: 'ne-Deva-NP');
  const year = 2082;

  // ── A whole year, with its Gregorian spans ─────────────────────────
  print('BS $year');
  var total = 0;
  for (var month = 1; month <= 12; month++) {
    final length = ctx.calendar.monthLength(Calendar.bikramSambat, year, month);
    total += length;
    final first = Calendar.bikramSambat.date(year, month, 1);
    final last = Calendar.bikramSambat.date(year, month, length);
    final starts = ctx.calendar.convert(first, Calendar.gregorian);
    final ends = ctx.calendar.convert(last, Calendar.gregorian);
    final name = ctx.intl.messages.sdk.calendar.bikramSambat.monthName(
      month: month,
    );
    print(
      '  ${month.toString().padLeft(2)}  ${name.padRight(10)}'
      ' ${length.toString().padLeft(2)} days   '
      '${starts.year}-${_two(starts.month)}-${_two(starts.day)}'
      ' to ${ends.year}-${_two(ends.month)}-${_two(ends.day)}',
    );
  }
  print('      ${''.padRight(10)} $total days in the year');
  // A BS year is 365 or 366 days like any solar year, but its months are
  // not: the shortest here is 29 days and the longest 32, which is why a
  // month length is asked for and never assumed.

  // ── One month as a page ────────────────────────────────────────────
  print('');
  print('Baisakh $year');
  print(monthPage(ctx, year, 1));

  // ── The round trip, and what each date says about itself ───────────
  print('');
  final newYear = Calendar.bikramSambat.date(year, 1, 1);
  final gregorian = ctx.calendar.convert(newYear, Calendar.gregorian);
  final back = ctx.calendar.convert(gregorian, Calendar.bikramSambat);
  print('  BS   ${described(newYear)}');
  print('  ->   ${described(gregorian)}');
  print('  ->   ${described(back)}');
  print(
    '  fixed day ${ctx.calendar.fixedOf(newYear)}, weekday ${ctx.calendar.weekdayOf(newYear)}',
  );

  // ── Inside the table, and outside it ───────────────────────────────
  // A date a caller *states* is always `defined`: it is what was asked
  // for. A date the SDK *returns* says how it was decided, so the
  // resolution to read is the one on the answer.
  print('');
  for (final asked in [2082, 2200, 1960]) {
    final greg = ctx.calendar.convert(
      Calendar.bikramSambat.date(asked, 1, 1),
      Calendar.gregorian,
    );
    final answer = ctx.calendar.convert(greg, Calendar.bikramSambat);
    print(
      '  BS $asked began ${greg.year}-${_two(greg.month)}-${_two(greg.day)},'
      ' and the answer is [${answer.resolution.key}]',
    );
  }
  print(
    '       the official table runs BS 1970 to 2095; on either side the'
    " SDK's own engine answers, and says so",
  );

  // ── The typed message accessors ────────────────────────────────────
  // A date rendered for a reader goes through the locale, not through
  // string concatenation: the key is spelled once, in the generator, and
  // the parameters are typed.
  print('');
  print(
    '  rendered  ${ctx.intl.messages.sdk.calendar.bikramSambat.date.long(day: 1, monthName: ctx.intl.messages.sdk.calendar.bikramSambat.monthName(month: 1), year: year)}',
  );

  ctx.dispose();
}

String _two(int value) => value.toString().padLeft(2, '0');
