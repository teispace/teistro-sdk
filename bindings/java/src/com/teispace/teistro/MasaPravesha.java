package com.teispace.teistro;

/**
 * One month's or one day's chart instant ({@code 03-design/annual-chart.md}): the annual Sun a
 * sign further each month, the month's Sun a degree further each day (<i>Tajika Nilakanthi</i>,
 * Saṃjñā Tantra, the monthly and daily entry, v. 1). Found the chart at {@link #at()}, where the
 * native is.
 *
 * @param year The year, as a Varsha Pravesha's year counts it.
 * @param month The month, 1 (the annual chart itself) to 12.
 * @param day The day, 1 (the month's chart itself) to 30; 1 on every row of a year's months.
 * @param at The instant, a Julian day (UTC).
 * @param sunDeg Where the Sun stands then, degrees in the year's zodiac.
 */
public record MasaPravesha(int year, int month, int day, double at, double sunDeg) {
}
