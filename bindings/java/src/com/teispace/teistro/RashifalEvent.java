package com.teispace.teistro;

/**
 * One event of a period, counted from a sign.
 *
 * @param hit the event
 * @param sign the sign it happened in: the one entered, or the one a station stood in
 * @param house that sign's house from the reading's sign, 1 to 12
 * @param goodHouse whether v. 2 makes the graha's transit of that house good
 */
public record RashifalEvent(Hit hit, Rashi sign, int house, boolean goodHouse) {}
