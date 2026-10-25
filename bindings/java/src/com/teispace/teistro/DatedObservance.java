package com.teispace.teistro;

/**
 * A day a dated rule gave ({@code 03-design/festival-rule-language.md}
 * §3.6): a sankranti on the first of a Bikram Sambat month, a national day.
 *
 * @param rule the rule's key
 * @param day the day
 * @param on the same day in the rule's own calendar
 * @param published what a published entry did to the day; null when none
 *     touched it
 */
public record DatedObservance(String rule, CalendarDate day, CalendarDate on, Published published) {
    /**
     * A day no published entry touched.
     *
     * @param rule as the record's
     * @param day as the record's
     * @param on as the record's
     */
    public DatedObservance(String rule, CalendarDate day, CalendarDate on) {
        this(rule, day, on, null);
    }
}
