package com.teispace.teistro;

import java.util.Map;

/**
 * One aayadi reckoning: a measure times a multiplier, divided by the size of
 * a series, and the member its remainder names ({@code 03-design/aayadi.md}).
 *
 * @param quantity what the remainder names
 * @param measure the measure it starts from
 * @param value the measure's value
 * @param multiplier the multiplier
 * @param divisor the divisor, the size of the series
 * @param quotient the quotient: <i>Mayamata</i> IX v. 22 reads the nakshatra's as the building's age
 * @param remainder the remainder, 0 to the divisor less one
 * @param count the member's place, 1 to the divisor: a remainder of 0 counts as the last (crux C1192)
 * @param yoni the member of the banner series, for an eightfold aaya or a yoni; may be null
 * @param nakshatra the nakshatra, for the nakshatra; may be null
 * @param vara the weekday, for the weekday; may be null
 * @param tithi the tithi, for a thirtyfold tithi; may be null
 * @param yoga the yoga, for the yoga; may be null
 * @param source the verse
 */
public record AayadiReckoned(
        AayadiQuantity quantity,
        AayadiMeasure measure,
        long value,
        int multiplier,
        int divisor,
        long quotient,
        int remainder,
        int count,
        VastuYoni yoni,
        Nakshatra nakshatra,
        Vara vara,
        Tithi tithi,
        Yoga yoga,
        String source) {

    static AayadiReckoned read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        Object yoni = o.get("yoni");
        Object vara = o.get("vara");
        Object tithi = o.get("tithi");
        Object yoga = o.get("yoga");
        return new AayadiReckoned(
                AayadiQuantity.read(Reads.field(o, "quantity")),
                AayadiMeasure.read(Reads.field(o, "measure")),
                whole(o, "value"),
                Reads.integer(o, "multiplier"),
                Reads.integer(o, "divisor"),
                whole(o, "quotient"),
                Reads.integer(o, "remainder"),
                Reads.integer(o, "count"),
                yoni == null ? null : Reads.member(VastuYoni.byKey(Reads.string(yoni)), "VastuYoni", yoni),
                Naming.nakshatra(o.get("nakshatra")),
                vara == null ? null : Reads.member(Vara.byKey(Reads.string(vara)), "Vara", vara),
                tithi == null ? null : Reads.member(Tithi.byKey(Reads.string(tithi)), "Tithi", tithi),
                yoga == null ? null : Reads.member(Yoga.byKey(Reads.string(yoga)), "Yoga", yoga),
                Reads.string(o, "source"));
    }

    /** A whole number past an int's range: an area reaches 10^12. */
    private static long whole(Map<?, ?> o, String key) {
        if (Reads.field(o, key) instanceof Number number && number.doubleValue() == Math.rint(number.doubleValue())) {
            return number.longValue();
        }
        throw Reads.internal("the library wrote " + o.get(key) + " where `" + key + "` is a whole number");
    }
}
