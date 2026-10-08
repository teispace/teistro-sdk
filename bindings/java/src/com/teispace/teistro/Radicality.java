package com.teispace.teistro;

import java.util.List;

/**
 * Whether the figure is radical (p. 121).
 *
 * @param hourLord the lord of the chart's planetary hour
 * @param ascendantLord the ascendant's lord
 * @param grounds every ground that holds; empty when the figure is not radical
 */
public record Radicality(Graha hourLord, Graha ascendantLord, List<RadicalGround> grounds) {
    /** Keeps the list unmodifiable. */
    public Radicality {
        grounds = List.copyOf(grounds);
    }
}
