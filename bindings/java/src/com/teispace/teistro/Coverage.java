package com.teispace.teistro;

import java.time.LocalDate;
import java.util.List;
import java.util.Map;

/**
 * What a context answers over (SR-136): what {@link Context#coverage}
 * answers. Every date from {@code ephemeris().first()} to
 * {@code ephemeris().last()} is answered; a lunar month, year or span the
 * coverage cuts says so in its {@code clipped}.
 *
 * <pre>{@code
 * Coverage.Ephemeris span = ctx.coverage().ephemeris();
 * LocalDate first = span.first(); // 1800-01-01
 * }</pre>
 *
 * @param ephemeris the days the context's ephemeris covers; null for a context with no ephemeris
 * @param bikramSambat the Bikram Sambat years the shipped table holds
 * @param holidays each shipped holiday set and its years
 */
public record Coverage(Ephemeris ephemeris, BikramSambat bikramSambat, List<Holidays> holidays) {
    /**
     * The instants and the civil days an ephemeris covers.
     *
     * @param fromJd the first instant answered, a Julian day on UT1
     * @param toJd the last instant answered
     * @param first the first Gregorian date, on UT, the span holds at least half of
     * @param last the last such date
     */
    public record Ephemeris(double fromJd, double toJd, LocalDate first, LocalDate last) {}

    /**
     * The Bikram Sambat years the shipped table holds.
     *
     * @param firstYear the table's first year
     * @param lastYear its last
     * @param officialFirstYear the first year the government's calendar committee published
     * @param officialLastYear the last
     */
    public record BikramSambat(int firstYear, int lastYear, int officialFirstYear, int officialLastYear) {}

    /**
     * A shipped holiday set and the years it lists.
     *
     * @param set the set's key, as a festival request names it
     * @param years the Bikram Sambat years it lists, in order
     */
    public record Holidays(String set, List<Integer> years) {}

    static Coverage read(String text) {
        Map<?, ?> value = Reads.object(Json.read(text));
        Object span = value.get("ephemeris");
        Ephemeris ephemeris = null;
        if (span != null) {
            Map<?, ?> e = Reads.object(span);
            ephemeris = new Ephemeris(Reads.number(e, "fromJd"), Reads.number(e, "toJd"), date(e, "first"),
                    date(e, "last"));
        }
        Map<?, ?> bs = Reads.object(value, "bikramSambat");
        return new Coverage(ephemeris,
                new BikramSambat(Reads.integer(bs, "firstYear"), Reads.integer(bs, "lastYear"),
                        Reads.integer(bs, "officialFirstYear"), Reads.integer(bs, "officialLastYear")),
                Reads.each(value, "holidays", raw -> {
                    Map<?, ?> h = Reads.object(raw);
                    return new Holidays(Reads.string(h, "set"), Reads.each(h, "years", Reads::integer));
                }));
    }

    private static LocalDate date(Map<?, ?> object, String key) {
        Map<?, ?> day = Reads.object(object, key);
        return LocalDate.of(Reads.integer(day, "year"), Reads.integer(day, "month"), Reads.integer(day, "day"));
    }
}
