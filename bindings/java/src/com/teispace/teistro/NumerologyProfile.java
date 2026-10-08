package com.teispace.teistro;

import java.util.Map;

/**
 * Everything numerology says of a name and a birth date.
 *
 * @param pythagoreanName the name under Balliett's letter cycle
 * @param pythagoreanBirth Balliett's birth number
 * @param chaldeanName the name under Cheiro's Chaldean table
 * @param chaldeanBirth Cheiro's numbers of the date
 * @param baseline the baseline engine's own numbers, present only under
 *     every baseline reading, so they are never mistaken for the texts';
 *     may be null
 */
public record NumerologyProfile(
        NameNumber pythagoreanName, BirthNumber pythagoreanBirth, NameNumber chaldeanName,
        ChaldeanDate chaldeanBirth, BaselineNumerology baseline) {

    static NumerologyProfile read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        Map<?, ?> birth = Reads.object(o, "pythagoreanBirth");
        Map<?, ?> chaldean = Reads.object(o, "chaldeanBirth");
        Object own = Reads.field(o, "baseline");
        Object sum = Reads.field(birth, "sum");
        BaselineNumerology baseline = null;
        if (own != null) {
            Map<?, ?> b = Reads.object(own);
            baseline = new BaselineNumerology(Reduction.read(Reads.field(b, "soul")),
                    Reduction.read(Reads.field(b, "personality")), Reduction.read(Reads.field(b, "chaldeanDestiny")));
        }
        return new NumerologyProfile(
                NameNumber.read(Reads.field(o, "pythagoreanName")),
                new BirthNumber(
                        Reduction.read(Reads.field(birth, "month")),
                        Reduction.read(Reads.field(birth, "day")),
                        Reduction.read(Reads.field(birth, "year")),
                        sum == null ? null : Reduction.read(sum),
                        Reads.each(birth, "apart", one -> Reads.integer(one))),
                NameNumber.read(Reads.field(o, "chaldeanName")),
                new ChaldeanDate(Reduction.read(Reads.field(chaldean, "birth")),
                        Reduction.read(Reads.field(chaldean, "year"))),
                baseline);
    }
}
