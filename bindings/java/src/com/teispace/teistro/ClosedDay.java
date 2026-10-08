package com.teispace.teistro;

import java.util.List;

/**
 * A day the season closed, and the blackouts that closed it.
 *
 * @param date the day
 * @param by the blackouts that closed it
 */
public record ClosedDay(CalendarDate date, List<BlackoutKind> by) {
    /** Keeps the list unmodifiable. */
    public ClosedDay {
        by = List.copyOf(by);
    }
}
