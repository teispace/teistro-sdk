package com.teispace.teistro;

import java.util.List;

/**
 * One rite of a death's first year ({@code 03-design/shraddha.md} §3.3).
 *
 * @param rite which: {@code ADYA_MASIKA} (the first monthly, on the eleventh day),
 *     {@code UNA_MASIKA}, {@code MASIKA} (a monthly, at its month's death tithi),
 *     {@code TRAIPAKSHIKA}, {@code UNA_SHANMASIKA}, {@code UNA_ABDIKA}, {@code ABDIKA} (a yearly
 *     shraddha) or {@code SAPINDANA}
 * @param place its place among the <i>Dharmasindhu</i>'s sixteen, 1 for the first monthly to 16 for
 *     the unabdika, a doubled month's two monthlies sharing one; null for the abdika and the sapindana
 * @param month the month a monthly closes, 1 to 12; null for every other rite
 * @param year which yearly shraddha, 1 for the first; null for every other rite
 * @param adhika whether it is kept in an adhika lunation: the first of a doubled month's two
 *     monthlies
 * @param day its own day
 * @param form how it is performed on that day, {@code EKODDISHTA} (before the sapindana) or
 *     {@code PARVANA} (after it); null for the sapindana
 * @param drawnTo the sapindana's day, when the rite is first done there by the ekoddishta and again
 *     on its own day by the parvana; null otherwise
 * @param bars what the text bars an <i>una</i> rite's day for, found at its sunrise: {@code NANDA},
 *     {@code FRIDAY}, {@code CHATURDASHI} or {@code KRITTIKA}; empty for every other rite
 * @param observance the festival evaluator's judgement of a day kept on a tithi; null for one
 *     counted in days
 * @param clause where the rule is stated
 */
public record FirstYearRite(
        String rite, Integer place, Integer month, Integer year, boolean adhika, CalendarDate day,
        String form, CalendarDate drawnTo, List<String> bars, FestivalObservance observance, String clause) {
    /** Keeps the bars unmodifiable. */
    public FirstYearRite {
        bars = List.copyOf(bars);
    }
}
