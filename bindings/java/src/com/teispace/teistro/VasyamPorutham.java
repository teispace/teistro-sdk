package com.teispace.teistro;

/**
 * Vasyam on p. 75's table, never a sign to itself (C274).
 *
 * @param brideToGroom Whether the bride's sign is concordant to the groom's.
 * @param groomToBride Whether the groom's sign is concordant to the bride's.
 */
public record VasyamPorutham(
        boolean brideToGroom,
        boolean groomToBride) implements PoruthamReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#VASHYA}
     */
    @Override
    public Koota koota() {
        return Koota.VASHYA;
    }
}
