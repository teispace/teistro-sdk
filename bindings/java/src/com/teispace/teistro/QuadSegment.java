package com.teispace.teistro;

/**
 * A quadratic curve to a point, pulled towards its control.
 *
 * @param control The control point.
 * @param to Where the curve ends.
 */
public record QuadSegment(
        UnitPoint control,
        UnitPoint to) implements Segment {
}
