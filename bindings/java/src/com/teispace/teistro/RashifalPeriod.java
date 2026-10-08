package com.teispace.teistro;

import java.util.List;

/**
 * One period read for each of the twelve signs.
 *
 * @param first the first day
 * @param last the last day
 * @param reference the day it is read at, the middle one (C359)
 * @param instant the instant it is read at, a UTC Julian day (C358)
 * @param transits each graha's sign and degrees then, the Sun to Ketu
 * @param retrograde whether each was moving backwards then, the Sun to Ketu
 * @param panchanga what the baseline reads of the reference day's panchanga
 * @param readings each sign's reading, Aries to Pisces
 */
public record RashifalPeriod(
        CalendarDate first, CalendarDate last, CalendarDate reference, double instant, List<Transit> transits,
        List<Boolean> retrograde, RashifalPanchanga panchanga, List<RashiReading> readings) {
    /** Keeps the lists unmodifiable. */
    public RashifalPeriod {
        transits = List.copyOf(transits);
        retrograde = List.copyOf(retrograde);
        readings = List.copyOf(readings);
    }
}
