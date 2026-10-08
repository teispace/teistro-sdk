package com.teispace.teistro;

/**
 * Ganam: the two ganas (p. 72).
 *
 * @param bride The bride's.
 * @param groom The groom's.
 * @param diminished A Rakshasa beside another gana, the bride's star beyond the 14th from the
 *     groom's: the evil "diminishes", the disagreement stands (C279).
 */
public record GanamPorutham(
        Gana bride,
        Gana groom,
        boolean diminished) implements PoruthamReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#GANA}
     */
    @Override
    public Koota koota() {
        return Koota.GANA;
    }
}
