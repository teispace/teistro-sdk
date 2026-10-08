package com.teispace.teistro;

/**
 * The two nadis (VI.34).
 *
 * @param bride The bride's.
 * @param groom The groom's.
 * @param dosha Whether the shared nadi is a dosha under the rules.
 * @param lifted Whether the dosha is lifted by one sign with two stars, one star across two signs
 *     or one star in two padas (VI.36); false with no dosha.
 */
public record NadiKoota(
        Nadi bride,
        Nadi groom,
        boolean dosha,
        boolean lifted) implements KootaReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#NADI}
     */
    @Override
    public Koota koota() {
        return Koota.NADI;
    }
}
