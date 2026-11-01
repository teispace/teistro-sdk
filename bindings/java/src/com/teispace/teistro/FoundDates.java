package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * The Bikram Sambat and Gregorian dates a text names, each span with every
 * reading of it ({@code 03-design/lexicon.md} §2), as {@code intl.dates}
 * gives them.
 *
 * @param dates the spans, in the text's order
 */
public record FoundDates(List<Found> dates) {

    /**
     * A day in a calendar as a date in text is read.
     *
     * @param calendar the calendar's key, {@code calendar.BIKRAM_SAMBAT} or {@code calendar.GREGORIAN}
     * @param year the astronomical year
     * @param month the month, 1-based
     * @param day the day, 1-based
     * @param era the era's key, or null for a calendar that names none
     * @param eraYear the year in the era, or null
     */
    public record Day(String calendar, int year, int month, int day, String era, Integer eraYear) {
        static Day read(Object raw) {
            if (raw == null) {
                return null;
            }
            Map<?, ?> d = Reads.object(raw);
            Object era = d.get("era");
            Map<?, ?> e = era == null ? null : Reads.object(era);
            return new Day(Reads.string(d, "calendar"), (int) Reads.number(d, "year"), (int) Reads.number(d, "month"),
                    (int) Reads.number(d, "day"), e == null ? null : Reads.string(e, "era"),
                    e == null ? null : (int) Reads.number(e, "year"));
        }
    }

    /**
     * One way a date in the text may be read.
     *
     * @param date the date in the calendar it is read in
     * @param order how its figures were ordered: {@code YMD}, {@code DMY} or {@code MDY}
     * @param gregorian the Gregorian day it is, on a Bikram Sambat reading; null on a Gregorian one
     */
    public record Reading(Day date, String order, Day gregorian) {}

    /**
     * One span of the text that names a date.
     *
     * @param start where it begins, counted in code points, not UTF-16 units
     * @param end where it ends, exclusive, in the same count
     * @param text the span's text
     * @param readings every reading, Bikram Sambat first: more than one is an ambiguity, not a guess
     */
    public record Found(int start, int end, String text, List<Reading> readings) {}

    static FoundDates read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        List<Found> dates = Reads.array(o, "dates").stream().map(found -> {
            Map<?, ?> f = Reads.object(found);
            List<Reading> readings = Reads.array(f, "readings").stream().map(reading -> {
                Map<?, ?> r = Reads.object(reading);
                return new Reading(Day.read(r.get("date")), Reads.string(r, "order"), Day.read(r.get("gregorian")));
            }).toList();
            return new Found((int) Reads.number(f, "start"), (int) Reads.number(f, "end"), Reads.string(f, "text"),
                    readings);
        }).toList();
        return new FoundDates(dates);
    }
}
