package com.teispace.teistro;

import java.util.List;

/**
 * A chart's significators: the twelve houses, and the nodes' agency.
 *
 * @param houses The twelve houses'.
 * @param nodes The nodes' agency.
 */
public record KpSignificators(
        List<KpHouseSignificators> houses,
        List<KpNodeAgency> nodes) {
    /** The value, its lists copied and unmodifiable. */
    public KpSignificators {
        houses = List.copyOf(houses);
        nodes = List.copyOf(nodes);
    }
}
