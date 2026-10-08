package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * A muhurta search's answer ({@code 03-design/muhurta-at-the-boundary.md}
 * §4): the windows judged, best first under the ranking, the days the
 * season closed, and what computed it.
 *
 * <pre>{@code
 * Almanac almanac = sky.almanac().of(first, last, place, offset,
 *         Map.of("rules", "RAMAN_MARRIAGE"), null, false, false, false);
 * MuhurtaWindow best = almanac.muhurta().orElseThrow().windows().get(0);
 * }</pre>
 *
 * @param windows the windows judged, best first
 * @param closed the days the season closed
 * @param daysJudged how many days were judged
 * @param daysCut how many of the days judged were cut into windows
 * @param windowsBlackedOut how many windows fell in a blackout that did not
 *     cover their whole day, and were left out
 * @param ranking {@code TEXTS} or {@code BASELINE}
 * @param unjudged what the rules ask that the SDK does not judge yet
 * @param provenance what computed it and under what: the asta criterion and
 *     the zodiac's instant among the applied conventions, and the hash of the value
 */
public record MuhurtaAnswer(
        List<MuhurtaWindow> windows, List<ClosedDay> closed, int daysJudged, int daysCut, int windowsBlackedOut,
        String ranking, List<MuhurtaUnjudged> unjudged, Provenance provenance) {
    /** Keeps the lists unmodifiable. */
    public MuhurtaAnswer {
        windows = List.copyOf(windows);
        closed = List.copyOf(closed);
        unjudged = List.copyOf(unjudged);
    }
}
