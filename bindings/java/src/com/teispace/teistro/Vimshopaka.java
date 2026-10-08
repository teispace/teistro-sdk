package com.teispace.teistro;

import java.util.List;

/**
 * A chart's Vimshopaka: each graha's strength across the divisional charts under the four schemes
 * ({@code 03-design/vimshopaka-measured.md}).
 *
 * @param scoring How each varga was scored.
 * @param grahas Each graha's, Sun to Saturn.
 */
public record Vimshopaka(
        VimshopakaScoring scoring,
        List<GrahaVimshopaka> grahas) {
    /** The value, its lists copied and unmodifiable. */
    public Vimshopaka {
        grahas = List.copyOf(grahas);
    }
}
