package com.teispace.teistro;

/**
 * Vedhai: whether the two nakshatras pierce each other (p. 76, C276).
 *
 * @param pierced Whether they pierce each other.
 */
public record VedhaiPorutham(
        boolean pierced) implements PoruthamReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#VEDHA}
     */
    @Override
    public Koota koota() {
        return Koota.VEDHA;
    }
}
