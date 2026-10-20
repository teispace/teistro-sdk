package com.teispace.teistro;

/**
 * A day a dated rule gave ({@code 03-design/festival-rule-language.md}
 * §3.6): a sankranti on the first of a Bikram Sambat month, a national day.
 *
 * @param rule the rule's key
 * @param day the day
 * @param on the same day in the rule's own calendar
 */
public record DatedObservance(String rule, CalendarDate day, CalendarDate on) {}
