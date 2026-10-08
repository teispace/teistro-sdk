package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * A name read under one system.
 *
 * @param system {@code PYTHAGOREAN} or {@code CHALDEAN}
 * @param words every word, in order, so either reduction can be read back
 * @param total what the final reduction started from
 * @param compound Cheiro's compound number, for the Chaldean system only; may be null
 * @param reduction the final reduction
 */
public record NameNumber(String system, List<WordNumber> words, int total, Integer compound, Reduction reduction) {
    /** Keeps the list unmodifiable. */
    public NameNumber {
        words = List.copyOf(words);
    }

    static NameNumber read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new NameNumber(
                Reads.string(o, "system"),
                Reads.each(o, "words", one -> {
                    Map<?, ?> word = Reads.object(one);
                    return new WordNumber(
                            Reads.string(word, "text"),
                            Reads.each(word, "letters", letter -> {
                                Map<?, ?> l = Reads.object(letter);
                                return new NumerologyLetter(Reads.string(l, "letter"), Reads.integer(l, "value"));
                            }),
                            Reads.integer(word, "total"),
                            Reduction.read(Reads.field(word, "reduction")));
                }),
                Reads.integer(o, "total"),
                Reads.optionalInteger(o, "compound"),
                Reduction.read(Reads.field(o, "reduction")));
    }
}
