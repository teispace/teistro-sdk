package com.teispace.teistro;

/**
 * The varnas of the two Moon signs (VI.22).
 *
 * @param bride The bride's.
 * @param groom The groom's.
 */
public record VarnaKoota(
        Varna bride,
        Varna groom) implements KootaReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#VARNA}
     */
    @Override
    public Koota koota() {
        return Koota.VARNA;
    }
}
