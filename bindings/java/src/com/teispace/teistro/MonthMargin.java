package com.teispace.teistro;

/**
 * How near a lunar month came to another kind: the Sun against the nearest
 * sign boundary at each of its two new moons (SR-92).
 *
 * @param opening at the new moon that opened the month
 * @param closing at the new moon that closes it
 * @param marginal whether either new moon fell nearer a sankranti than
 *     {@code calendars.marginal_month_minutes}
 */
public record MonthMargin(NewMoonMargin opening, NewMoonMargin closing, boolean marginal) {}
