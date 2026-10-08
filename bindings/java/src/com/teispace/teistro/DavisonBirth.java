package com.teispace.teistro;

/**
 * The Davison birth of a chart and a synastry's partner (C248): the mean
 * instant, the mean place, its longitude the shorter way round, and the
 * mean of the two clocks, which names only the civil day. Its fields are
 * the ones {@code found} takes, so it founds a chart as a birth does:
 *
 * <pre>{@code
 * Chart between = sky.chart().found(davison.instant(), davison.place(), davison.utcOffsetSeconds(),
 *         ChartOptions.none());
 * }</pre>
 *
 * @param instant the mean instant, a UTC Julian day
 * @param place the mean place
 * @param utcOffsetSeconds the mean of the two clocks, east positive
 */
public record DavisonBirth(double instant, Observer place, int utcOffsetSeconds) {}
