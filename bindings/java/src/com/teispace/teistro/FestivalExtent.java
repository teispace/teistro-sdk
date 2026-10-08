package com.teispace.teistro;

/**
 * One of an observance's two days: its window for the rite, and the
 * fraction of it the tithi held.
 *
 * @param day the day
 * @param window its window for the rite
 * @param held the fraction of it the tithi held, 0 to 1 (an instant's is 0 or 1)
 */
public record FestivalExtent(CalendarDate day, Interval window, double held) {}
