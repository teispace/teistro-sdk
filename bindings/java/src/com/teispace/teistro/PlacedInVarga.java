package com.teispace.teistro;

/**
 * Where one graha stands in a divisional chart.
 *
 * @param graha Which graha.
 * @param at Where it stands.
 */
public record PlacedInVarga(
        Graha graha,
        VargaPlacement at) {
}
