package com.teispace.teistro;

/**
 * Sthree-Dheergham: the count, which agrees beyond the 13th (p. 72, C272).
 *
 * @param count The groom's nakshatra counted from the bride's.
 */
public record DeerghaPorutham(
        int count) implements PoruthamReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#STREE_DEERGHA}
     */
    @Override
    public Koota koota() {
        return Koota.STREE_DEERGHA;
    }
}
