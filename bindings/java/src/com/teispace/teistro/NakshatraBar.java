package com.teispace.teistro;

/**
 * A nakshatra a parana is not kept in (C700): after the fast of the 11th
 * {@code tithi} in its nija amanta {@code month}, the {@code which}th of
 * {@code parts} equal shares of {@code nakshatra}'s arc (1 of 1 the
 * whole). A rule's {@code parana} names its bars as {@code bars}, each
 * {@code {tithi, month, nakshatra, portion: {parts, which}}}.
 *
 * @param tithi the 11th whose fast's parana it bars
 * @param month that 11th's amanta month
 * @param nakshatra the nakshatra
 * @param parts the equal shares its arc is cut in, 1 to 12
 * @param which which of them, 1 to {@code parts}
 */
public record NakshatraBar(Tithi tithi, Masa month, Nakshatra nakshatra, int parts, int which) {}
