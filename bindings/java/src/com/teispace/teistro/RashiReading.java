package com.teispace.teistro;

import java.util.List;

/**
 * One sign's reading of a period, the sign taken as a reader's janma rashi.
 *
 * @param rashi the sign
 * @param gochar Phaladeepika ch. 26's gochar from it at the period's instant
 * @param saturn Saturn's standing from it
 * @param events every event of the period, in time order, counted from it
 */
public record RashiReading(Rashi rashi, GocharReading gochar, SaturnStanding saturn, List<RashifalEvent> events) {
    /** Keeps the list unmodifiable. */
    public RashiReading {
        events = List.copyOf(events);
    }
}
