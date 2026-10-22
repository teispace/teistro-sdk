package com.teispace.teistro;

import java.util.List;

/**
 * An Ekadashi's fast under one rule: the day, and the facts that gave it
 * ({@code 03-design/festival-rules.md} §8).
 *
 * @param rule the rule's key
 * @param tithi the bright or the dark 11th
 * @param month its amanta month
 * @param adhika whether that month is adhika
 * @param tithis the 10th, the 11th and the 12th, whole
 * @param days the 11th's own day and the day after (C181)
 * @param piercedAt where the 10th pierces the 11th's day, whatever the rule
 *     reckons: {@code ARUNODAYA} or {@code SUNRISE}; may be null
 * @param pierced whether that pierces by the rule's vedha
 * @param excess which of the 11th and the 12th hold the sunrise after their
 *     own: {@code ELEVENTH}, {@code TWELFTH}, {@code BOTH} or {@code NEITHER}
 * @param choice the table's day, {@code EARLIER} or {@code LATER}, which {@code day} resolves
 * @param day the fast
 * @param marginSeconds how far the place can move: the seconds of time every instant the
 *     fast was decided from (the sunsets that give the 11th its own day,
 *     that day's sunrise and, under a Vaishnava vedha, its arunodaya, and
 *     the two sunrises after) may move, as another place moves them,
 *     before the day can change
 */
public record EkadashiFast(
        String rule, Tithi tithi, Masa month, boolean adhika, List<Interval> tithis, List<CalendarDate> days,
        String piercedAt, boolean pierced, String excess, String choice, CalendarDate day,
        double marginSeconds) {
    /** Keeps the lists unmodifiable. */
    public EkadashiFast {
        tithis = List.copyOf(tithis);
        days = List.copyOf(days);
    }
}
