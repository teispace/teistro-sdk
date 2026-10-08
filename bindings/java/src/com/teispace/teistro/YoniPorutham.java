package com.teispace.teistro;

/**
 * Yoni on the chapter's own table, Uttarashadha the cow (p. 73, C278).
 *
 * @param bride The bride's.
 * @param groom The groom's.
 * @param hostile Whether they are among the chapter's eight enmities.
 */
public record YoniPorutham(
        Yoni bride,
        Yoni groom,
        boolean hostile) implements PoruthamReading {
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
