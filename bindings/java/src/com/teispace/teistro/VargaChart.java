package com.teispace.teistro;

import java.util.List;

/**
 * One divisional chart of one founded moment.
 *
 * @param varga Which divisional chart.
 * @param lagna Where the lagna falls in it.
 * @param grahas Every graha, in the order the foundation carries them.
 */
public record VargaChart(
        Varga varga,
        VargaPlacement lagna,
        List<PlacedInVarga> grahas) {
    /** The value, its lists copied and unmodifiable. */
    public VargaChart {
        grahas = List.copyOf(grahas);
    }
}
