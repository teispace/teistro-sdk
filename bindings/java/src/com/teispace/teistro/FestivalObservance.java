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
 * @param marginSeconds how far the place can move: the seconds of time
 *     every instant the day was decided from (its sunrises and the windows
 *     the rule draws) may move, as another place moves them, before the
 *     day can change; 0 for two equal parts
 * @param parana when the fast kept on the day is broken, under each of the
 *     rule's readings in order ({@code 03-design/festival-rules.md} §8.7);
 *     empty when the rule has none
 * @param published what a published entry did to the day; null when none
 *     touched it, so the day is the rule's
 * @param sait the rite's time on the day, elected by the rule's sait
 *     ({@code 03-design/festival-sait.md}); null for a rule with none
 */
public record FestivalObservance(
        String rule, CalendarDate day, Interval tithi, Masa month, boolean adhika, String caseHeld,
        List<FestivalExtent> extents, FestivalDecided decidedBy, String choice,
        double marginSeconds, List<VrataParanaTime> parana, Published published, FestivalSait sait) {
    /** Keeps the pair and the readings unmodifiable. */
    public FestivalObservance {
        extents = List.copyOf(extents);
        parana = List.copyOf(parana);
    }

    /**
     * An observance no published entry touched, as one was written before
     * sets were laid over answers.
     *
     * @param rule as the record's
     * @param day as the record's
     * @param tithi as the record's
     * @param month as the record's
     * @param adhika as the record's
     * @param caseHeld as the record's
     * @param extents as the record's
     * @param decidedBy as the record's
     * @param choice as the record's
     * @param marginSeconds as the record's
     * @param parana as the record's
     */
    public FestivalObservance(
            String rule, CalendarDate day, Interval tithi, Masa month, boolean adhika, String caseHeld,
            List<FestivalExtent> extents, FestivalDecided decidedBy, String choice,
            double marginSeconds, List<VrataParanaTime> parana) {
        this(rule, day, tithi, month, adhika, caseHeld, extents, decidedBy, choice, marginSeconds, parana, null);
    }

    /**
     * An observance with no sait, as one was written before saits were
     * elected.
     *
     * @param rule as the record's
     * @param day as the record's
     * @param tithi as the record's
     * @param month as the record's
     * @param adhika as the record's
     * @param caseHeld as the record's
     * @param extents as the record's
     * @param decidedBy as the record's
     * @param choice as the record's
     * @param marginSeconds as the record's
     * @param parana as the record's
     * @param published as the record's
     */
    public FestivalObservance(
            String rule, CalendarDate day, Interval tithi, Masa month, boolean adhika, String caseHeld,
            List<FestivalExtent> extents, FestivalDecided decidedBy, String choice,
            double marginSeconds, List<VrataParanaTime> parana, Published published) {
        this(rule, day, tithi, month, adhika, caseHeld, extents, decidedBy, choice, marginSeconds, parana,
                published, null);
    }
}
