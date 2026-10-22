package com.teispace.teistro;

/**
 * The clause that set an ashauch's span ({@code 03-design/ashauch.md}): the table's row, or the
 * rule.
 *
 * @param clause the tag: {@code RELATION}, {@code BAND} or {@code LOSS} (a row of that part of the
 *     table), {@code BIRTH_PERIOD_RUNS}, {@code REMAINDER} (heard inside the period, the days left),
 *     {@code LATE} (heard past it, the late table's row), {@code LATE_BATH}, {@code ROW_HEARD_LATE}
 *     (the relation row's own late length), {@code SHORT_HEARD_LATE} or {@code BIRTH_HEARD_LATE}
 * @param row the row's index, for {@code RELATION}, {@code BAND}, {@code LOSS} and
 *     {@code ROW_HEARD_LATE}; null otherwise
 * @param said the row's text, beside {@code row}; null otherwise
 * @param withinDays for {@code LATE}, within how many days of the event the news came; null
 *     otherwise
 * @param abroad for {@code LATE}, whether the abroad table answered; null otherwise
 */
public record AshauchClause(String clause, Integer row, String said, Integer withinDays, Boolean abroad) {}
