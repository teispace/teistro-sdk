package com.teispace.teistro;

/**
 * The two yonis and how they stand (VI.25 to 26, C261).
 *
 * @param bride The bride's.
 * @param groom The groom's.
 * @param relation How they stand.
 */
public record YoniKoota(
        Yoni bride,
        Yoni groom,
        YoniRelation relation) implements KootaReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#YONI}
     */
    @Override
    public Koota koota() {
        return Koota.YONI;
    }
}
