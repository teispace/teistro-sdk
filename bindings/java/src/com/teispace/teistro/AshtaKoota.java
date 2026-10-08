package com.teispace.teistro;

import java.util.List;

/**
 * The Ashta Koota of a bride and a groom (<i>Muhurta Chintamani</i> VI.21 to 34). Never a verdict:
 * the doshas and their exceptions are clauses.
 *
 * @param kootas The eight, in the verse's order.
 * @param total Their points, out of 36.
 */
public record AshtaKoota(
        List<KootaRow> kootas,
        double total) {
    /** The value, its lists copied and unmodifiable. */
    public AshtaKoota {
        kootas = List.copyOf(kootas);
    }
}
