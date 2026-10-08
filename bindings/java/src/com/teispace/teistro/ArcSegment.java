package com.teispace.teistro;

/**
 * A circular arc about a centre to a point the same distance from it.
 *
 * @param centre The circle's centre.
 * @param clockwise Which way the arc runs, as a reader sees it.
 * @param to Where the arc ends.
 */
public record ArcSegment(
        UnitPoint centre,
        boolean clockwise,
        UnitPoint to) implements Segment {
}
