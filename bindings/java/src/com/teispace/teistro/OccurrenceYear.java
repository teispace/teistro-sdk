package com.teispace.teistro;

import java.util.List;

/**
 * One year of an {@link Occurrences} answer ({@code 03-design/rule-years.md}): the days its dates
 * hold, none, one or several, never a pick among them.
 *
 * <pre>{@code
 * OccurrenceYear year = span.years().get(0);
 * boolean none = year.days().isEmpty();
 * }</pre>
 *
 * @param year the year, in the request's calendar
 * @param first its first day: month 1, day 1
 * @param last its last day: the day before the next year's first
 * @param days each day the target falls on, in order
 * @param festivals the reckoning's entries for the keys in the year, a withdrawn day marked so,
 *     with the provenance
 */
public record OccurrenceYear(int year, CalendarDate first, CalendarDate last, List<OccurrenceDay> days,
        FestivalAnswer festivals) {
    /** Copies the days. */
    public OccurrenceYear {
        days = List.copyOf(days);
    }
}
