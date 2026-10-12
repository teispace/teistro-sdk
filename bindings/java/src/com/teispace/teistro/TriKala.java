package com.teispace.teistro;

import java.util.List;

/**
 * The lagnesha's own yogas read as time (n179, C375).
 *
 * @param lagnesha The lagnesha.
 * @param past Those it is separating from.
 * @param present Those it is joined with.
 * @param future Those it will join.
 */
public record TriKala(
        Graha lagnesha,
        List<Graha> past,
        List<Graha> present,
        List<Graha> future) {
    /** The value, its lists copied and unmodifiable. */
    public TriKala {
        past = List.copyOf(past);
        present = List.copyOf(present);
        future = List.copyOf(future);
    }
}
