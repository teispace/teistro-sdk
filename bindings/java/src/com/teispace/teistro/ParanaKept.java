package com.teispace.teistro;

/**
 * The window a parana was kept in, and the clause that set each edge.
 *
 * @param span when the meal may be taken
 * @param at the rule's window it lies in
 * @param fromBy what set its start: {@code WINDOW}, {@code HARI_VASARA} (the
 *     12th's first quarter ends), {@code TITHI_START}, {@code TITHI_END} or
 *     {@code NAKSHATRA} (an edge of a barred share of a nakshatra, C700)
 * @param toBy what set its end, in the same words
 */
public record ParanaKept(Interval span, ParanaWindow at, String fromBy, String toBy) {}
