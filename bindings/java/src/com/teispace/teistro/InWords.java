package com.teispace.teistro;

import java.util.Map;

/**
 * A number or an amount of money in words and in figures, by CLDR's
 * rule-based number formats ({@code 03-design/amounts-in-words.md}).
 *
 * @param text the whole, in words
 * @param figure the figures, grouped, in the digits asked for
 * @param rules where the words came from: {@code cldr-<release>/<file>/<rule set>}
 * @param locale the locale the words and units are in
 * @param amount an amount's parts; null for a number
 */
public record InWords(String text, String figure, String rules, String locale, AmountInWords amount) {

    /**
     * An amount's parts, for an application that composes them its own way.
     *
     * @param currency the ISO 4217 code
     * @param major the major units
     * @param minor the minor units after them
     * @param majorWords the major units in words
     * @param majorUnit the major unit's word for that count
     * @param minorWords the minor units in words; null for a currency with no minor unit
     * @param minorUnit the minor unit's word for that count; null with {@code minorWords}
     * @param rounded whether a decimal lost digits to the rounding
     */
    public record AmountInWords(String currency, long major, long minor, String majorWords, String majorUnit,
            String minorWords, String minorUnit, boolean rounded) {}

    static InWords read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        Object amount = Reads.field(o, "amount");
        AmountInWords parts = null;
        if (amount != null) {
            Map<?, ?> a = Reads.object(amount);
            parts = new AmountInWords(Reads.string(a, "currency"), whole(Reads.field(a, "major")),
                    whole(Reads.field(a, "minor")), Reads.string(a, "majorWords"), Reads.string(a, "majorUnit"),
                    Reads.optionalString(a, "minorWords"), Reads.optionalString(a, "minorUnit"),
                    Reads.flag(a, "rounded"));
        }
        return new InWords(Reads.string(o, "text"), Reads.string(o, "figure"), Reads.string(o, "rules"),
                Reads.string(o, "locale"), parts);
    }

    /** A count up to 2⁶³ − 1; one wider reads as the double nearest it, cut to a long. */
    private static long whole(Object value) {
        if (value instanceof Long count) {
            return count;
        }
        return (long) Reads.number(value);
    }
}
