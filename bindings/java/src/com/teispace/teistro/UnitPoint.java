package com.teispace.teistro;

/**
 * A point in a drawing's unit square, y downwards.
 *
 * @param x From the left edge, 0 to 1.
 * @param y From the top edge, 0 to 1.
 */
public record UnitPoint(
        double x,
        double y) {
}
