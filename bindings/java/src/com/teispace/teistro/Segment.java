package com.teispace.teistro;

/**
 * One step of an outline, from wherever the previous step ended: a {@code LineSegment}, a
 * {@code QuadSegment} or an {@code ArcSegment}.
 */
public sealed interface Segment
        permits LineSegment, QuadSegment, ArcSegment {
    /**
     * Where the step ends.
     *
     * @return the point
     */
    UnitPoint to();
}
