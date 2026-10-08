package com.teispace.teistro;

import java.util.List;

/**
 * The day a rule falls on, and why.
 *
 * @param rule the rule's key
 * @param day the day
 * @param tithi the occurrence judged: the tithi's, or the nakshatra's for a
 *     rule kept on a nakshatra in a paksha
 * @param month its amanta month, as an Ekadashi fast's is: which month's
 *     occurrence a rule kept every month decided
 * @param adhika whether that month is adhika
 * @param caseHeld how the tithi held the rite's time on its two days:
 *     {@code EARLIER_ONLY}, {@code LATER_ONLY}, {@code BOTH},
 *     {@code NEITHER}, {@code EQUAL_PARTS} or {@code UNEQUAL_PARTS}
 *     (Python's {@code case}, a Java keyword)
 * @param extents the earlier day's extent and the later's
 * @param decidedBy what decided
 * @param choice the choice that decided, {@code EARLIER}, {@code LATER} or
 *     {@code BY_YUGMA}, which {@code day} resolves
 */
public record FestivalObservance(
        String rule, CalendarDate day, Interval tithi, Masa month, boolean adhika, String caseHeld,
        List<FestivalExtent> extents, FestivalDecided decidedBy, String choice) {
    /** Keeps the pair unmodifiable. */
    public FestivalObservance {
        extents = List.copyOf(extents);
    }
}
