package com.teispace.teistro;

import java.util.List;

/**
 * A chart drawn in a layout ({@code 03-design/chart-geometry.md}).
 *
 * @param layout The layout it is drawn in: a {@code ChartLayout}, or a layout the context
 *     registered, by its full key as a {@code String} ({@code chart_layout.ACME_KERALA}).
 * @param varga Which chart: {@code Varga.D1} for the founded chart, or a divisional one.
 * @param cells The cells, in the layout's order.
 * @param frame The lines drawn that hold nothing.
 * @param marks Each body at its own degree, on a wheel; empty for a grid.
 * @param svg The drawing as SVG, in the request's theme and the context's locale; null when the
 *     request gave no theme. May be null.
 */
public record Drawing(
        Object layout,
        Varga varga,
        List<DrawnCell> cells,
        List<Outline> frame,
        List<DrawnMark> marks,
        String svg) {
    /** The value, its lists copied and unmodifiable. */
    public Drawing {
        cells = List.copyOf(cells);
        frame = List.copyOf(frame);
        marks = List.copyOf(marks);
    }

    /**
     * The layout's full key, shipped or registered
     * ({@code chart_layout.NORTH_INDIAN}), for a caller that reads either.
     *
     * @return the full key
     */
    public String layoutKey() {
        return layout instanceof ChartLayout shipped ? shipped.fullKey() : String.valueOf(layout);
    }
}
