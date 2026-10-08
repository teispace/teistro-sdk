package com.teispace.teistro;

/**
 * Mahendra: the count, which agrees at the 4th, 7th and every third to the 25th (p. 72).
 *
 * @param count The groom's nakshatra counted from the bride's.
 */
public record MahendraPorutham(
        int count) implements PoruthamReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#MAHENDRA}
     */
    @Override
    public Koota koota() {
        return Koota.MAHENDRA;
    }
}
