package com.teispace.teistro;

import java.util.List;

/**
 * The next day {@link AlmanacArea#next} found ({@code 03-design/next-occurrence.md}).
 *
 * <pre>{@code
 * NextOccurrence next = sdk.almanac().next(2461042.0, kathmandu, 20_700, "SANKRANTI", null, null, null, null);
 * String key = next.keys().get(0); // MAKARA_SANKRANTI
 * }</pre>
 *
 * @param day the day, in the request's calendar
 * @param keys the keys of the rules that fall on it
 * @param festivals the festival reckoning's entries for those keys on that day, with the provenance
 * @param from the first day searched: the civil day holding the instant
 * @param limitDays the look-ahead limit, in days
 * @param searchedDays the days reckoned until it was found
 * @param group for a group, its span, {@code day} being its first day; null for a rule or a kind
 */
public record NextOccurrence(CalendarDate day, List<String> keys, FestivalAnswer festivals, CalendarDate from,
        int limitDays, int searchedDays, GroupSpan group) {
    /** Copies the keys. */
    public NextOccurrence {
        keys = List.copyOf(keys);
    }

    /**
     * A rule's or a kind's next day, with no group's span.
     *
     * @param day the day
     * @param keys the keys of the rules that fall on it
     * @param festivals the festival reckoning's entries for those keys on that day
     * @param from the first day searched
     * @param limitDays the look-ahead limit, in days
     * @param searchedDays the days reckoned until it was found
     */
    public NextOccurrence(CalendarDate day, List<String> keys, FestivalAnswer festivals, CalendarDate from,
            int limitDays, int searchedDays) {
        this(day, keys, festivals, from, limitDays, searchedDays, null);
    }
}
