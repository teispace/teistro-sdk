package com.teispace.teistro;

import java.util.List;

/**
 * A day of an {@link OccurrenceYear} and the keys that fall on it ({@code 03-design/rule-years.md}).
 *
 * <pre>{@code
 * OccurrenceDay day = span.years().get(0).days().get(0);
 * String key = day.keys().get(0);
 * }</pre>
 *
 * @param day the day, in the request's calendar
 * @param keys the keys looked for that fall on it, in the order they were looked for
 */
public record OccurrenceDay(CalendarDate day, List<String> keys) {
    /** Copies the keys. */
    public OccurrenceDay {
        keys = List.copyOf(keys);
    }
}
