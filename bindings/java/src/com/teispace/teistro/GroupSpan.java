package com.teispace.teistro;

import java.util.List;

/**
 * A festival group's occurrence ({@code 03-design/festival-groups.md}, SR-125): the days from its
 * first member's to its last, each member's day, and every day between, a day no member falls on
 * marked a gap.
 *
 * <pre>{@code
 * FestivalAnswer answer = sdk.almanac().festivals(from, to, kathmandu, 20_700,
 *         FestivalRequest.of(Map.of("rules", "NEPAL_PANCHANGA")));
 * GroupSpan tihar = answer.groups().get(0);
 * boolean gap = tihar.days().get(2).gap();
 * }</pre>
 *
 * @param group the group's key, as {@code TIHAR}
 * @param main the main member's key, as {@code LAKSHMI_PUJA}
 * @param first the span's first day
 * @param last the span's last day, inclusive
 * @param members each member and its day, in the group's order
 * @param days every day from {@code first} to {@code last}, in order
 */
public record GroupSpan(String group, String main, CalendarDate first, CalendarDate last, List<Member> members,
        List<Day> days) {
    /** Keeps the lists unmodifiable. */
    public GroupSpan {
        members = List.copyOf(members);
        days = List.copyOf(days);
    }

    /**
     * One member of a span, in the group's order.
     *
     * @param rule the member rule's key
     * @param day its day in this span; null when it falls on none
     */
    public record Member(String rule, CalendarDate day) {}

    /**
     * One day of a span.
     *
     * @param day the day
     * @param members the members that fall on it, in the group's order; empty on a gap
     * @param gap whether no member falls on it: a day of the span all the same
     */
    public record Day(CalendarDate day, List<String> members, boolean gap) {
        /** Keeps the members unmodifiable. */
        public Day {
            members = List.copyOf(members);
        }
    }
}
