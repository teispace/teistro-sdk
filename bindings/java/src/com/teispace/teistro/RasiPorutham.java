package com.teispace.teistro;

/**
 * Rasi: how far the groom's Moon sign stands from the bride's (pp. 73 to 74).
 *
 * @param apart The groom's sign counted from the bride's, 1 to 12.
 */
public record RasiPorutham(
        int apart) implements PoruthamReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#BHAKOOT}
     */
    @Override
    public Koota koota() {
        return Koota.BHAKOOT;
    }
}
