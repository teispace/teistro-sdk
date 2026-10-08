package com.teispace.teistro;

import java.util.List;

/**
 * A closed outline: a start and the steps back to it.
 *
 * @param start Where the outline starts.
 * @param segments The steps around it.
 */
public record Outline(
        UnitPoint start,
        List<Segment> segments) {
    /** The value, its lists copied and unmodifiable. */
    public Outline {
        segments = List.copyOf(segments);
    }
}
