package com.teispace.teistro;

/**
 * Mars's house from one reference.
 *
 * @param reference The place the house is counted from, {@code LAGNA}, {@code MOON} or
 *     {@code VENUS}: the answer's {@code from}.
 * @param house Mars's house from it by sign, 1 to 12 (C287).
 * @param inHouses Whether the house is one of the rules' houses; it makes the dosha only from a
 *     reference the rules count.
 */
public record KujaReading(
        String reference,
        int house,
        boolean inHouses) {
}
