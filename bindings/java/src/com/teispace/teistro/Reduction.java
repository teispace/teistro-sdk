package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * A number reduced: every sum, so "33, so 6" and "38, so 11" read back.
 *
 * @param steps the sums in order: the number reduced, then each digit sum
 * @param number where it stopped: one digit, or a master the rules keep
 */
public record Reduction(List<Integer> steps, int number) {
    /** Keeps the list unmodifiable. */
    public Reduction {
        steps = List.copyOf(steps);
    }

    static Reduction read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new Reduction(Reads.each(o, "steps", one -> Reads.integer(one)), Reads.integer(o, "number"));
    }
}
