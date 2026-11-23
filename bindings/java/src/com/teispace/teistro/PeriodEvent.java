package com.teispace.teistro;

/**
 * One event of a rashifal period, the sign it happened in uncounted.
 *
 * @param hit the event
 * @param sign the sign it happened in: the one entered, or the one a station stood in
 */
public record PeriodEvent(Hit hit, Rashi sign) {}
