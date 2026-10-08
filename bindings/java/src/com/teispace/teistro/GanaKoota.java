package com.teispace.teistro;

/**
 * The two ganas (VI.29 to 30).
 *
 * @param bride The bride's.
 * @param groom The groom's.
 * @param dosha Whether a Rakshasa stands beside another gana.
 * @param lifted Whether the dosha is lifted: the sign lords or the navamsha lords befriended
 *     (VI.33), or one sign or one star between the two (VI.36); false with no dosha.
 */
public record GanaKoota(
        Gana bride,
        Gana groom,
        boolean dosha,
        boolean lifted) implements KootaReading {
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
