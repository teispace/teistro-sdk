package com.teispace.teistro;

/**
 * Rajju: the two divisions (p. 75, C275).
 *
 * @param bride The bride's.
 * @param groom The groom's.
 */
public record RajjuPorutham(
        Rajju bride,
        Rajju groom) implements PoruthamReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#RAJJU}
     */
    @Override
    public Koota koota() {
        return Koota.RAJJU;
    }
}
