package com.teispace.teistro;

/**
 * A straight line to a point.
 *
 * @param to Where the line ends.
 */
public record LineSegment(
        UnitPoint to) implements Segment {
}
