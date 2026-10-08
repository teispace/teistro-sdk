package com.teispace.teistro;

/**
 * One annual chart's instant.
 *
 * @param year How many years the native has completed at this instant: 1 is the first return, a
 *     year after birth. Counted in returns and not in years of life, because the two namings
 *     differ by one and both are in use.
 * @param instant The instant, a Julian day (UTC), to pass to {@code found}.
 * @param muntha The Muntha standing at it, progressed by this year's own count.
 * @param annual The year's own chart, or null unless the varsha request named a {@code place}. May
 *     be null.
 */
public record Pravesha(
        int year,
        double instant,
        Muntha muntha,
        AnnualChart annual) {
}
