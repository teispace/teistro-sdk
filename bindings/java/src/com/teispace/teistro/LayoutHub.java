package com.teispace.teistro;

/**
 * A circle no cell holds, where a drawing writes the chart's name, as a Nepali round chart prints
 * ल or न there ({@code 03-design/chart-geometry.md}).
 *
 * @param centre The circle's centre.
 * @param radius Its radius, a fraction of the square's side.
 */
public record LayoutHub(UnitPoint centre, double radius) {
}
