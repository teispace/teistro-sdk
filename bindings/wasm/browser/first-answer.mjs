/**
 * The first calendar answer a page wants, the same call in every host the
 * start check times.
 *
 * @param {typeof import('../../node/lib/index.js')} sdk an entry
 * @returns {object} a date in Bikram Sambat
 */
export function firstAnswer(sdk) {
  const ctx = new sdk.Context({ profile: 'nepali-default', ephemeris: 'BUILTIN' });
  try {
    return ctx.calendar.convert(
      {
        calendar: sdk.Calendar.Gregorian,
        year: 2015,
        eraYear: 0,
        month: 4,
        day: 14,
        resolution: sdk.Resolution.Defined,
        computedMonth: 0,
        computedDay: 0,
      },
      sdk.Calendar.BikramSambat,
    );
  } finally {
    ctx.dispose();
  }
}
