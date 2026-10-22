package com.teispace.teistro;

import java.util.List;

/**
 * An anniversary's day: a yearly shraddha, a barkhi or a janma tithi, kept
 * every year by the tithi ({@code 03-design/shraddha.md}). A request asks
 * for one as an item of {@code rules} with an {@code origin}:
 *
 * <pre>{@code
 * Map<String, Object> barkhi = Map.of("key", "GRANDFATHER",
 *         "origin", Map.of("by", "LUNAR", "month", "KARTIKA", "tithi", "SHUKLA_DASHAMI"));
 * AnniversaryDay day = sky.almanac().festivals(first, last, place, offset,
 *         new FestivalRequest(List.of(barkhi))).anniversaries().get(0);
 * }</pre>
 *
 * @param key the request's key
 * @param count which anniversary: 1 for the first recurrence after the
 *     origin, a janma tithi's the age turned; null for a lunar origin given
 *     no moment
 * @param kind {@code EKODDISHTA}, {@code PARVANA} or {@code JANMA_TITHI}
 * @param origin the origin, as found
 * @param monthBy the clause that chose the month: {@code NIJA},
 *     {@code FIRST_YEAR_ADHIKA}, {@code ADHIKA_ORIGIN},
 *     {@code KSHAYA_JOINED} or {@code SAMSARPA} (C666, C667, C669)
 * @param observance the day as the festival evaluator judged it
 */
public record AnniversaryDay(String key, Integer count, String kind, Origin origin, String monthBy,
        FestivalObservance observance) {

    /**
     * An anniversary's origin as found: the tithi at the moment, the amanta
     * month it names (in a kshaya lunation by the tithi's half), and that
     * lunation's kind.
     *
     * @param tithi the tithi
     * @param month the month it names, amanta
     * @param kind {@code NIJA}, {@code ADHIKA} or {@code KSHAYA}
     * @param half {@code FIRST} or {@code SECOND}
     * @param nakshatra the nakshatra then; null for a lunar origin
     * @param at the moment the years are counted from, a UTC Julian day, or
     *     null
     * @param assumedSunrise whether the tithi is a date's sunrise tithi, for
     *     want of an hour
     * @param candidates every tithi the date held when more than one; empty
     *     otherwise
     */
    public record Origin(Tithi tithi, Masa month, String kind, String half, Nakshatra nakshatra, Double at,
            boolean assumedSunrise, List<Tithi> candidates) {
        /** Keeps the list unmodifiable. */
        public Origin {
            candidates = List.copyOf(candidates);
        }
    }
}
