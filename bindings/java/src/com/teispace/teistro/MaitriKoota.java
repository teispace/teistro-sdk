package com.teispace.teistro;

/**
 * The two Moon signs' lords and how they stand by the natural friendships (VI.27 to 28).
 *
 * @param bride The bride's sign lord.
 * @param groom The groom's sign lord.
 * @param relation How they stand.
 * @param lifted Whether a good Bhakoot lifts the lords' enmity (VI.33); false with no enmity.
 */
public record MaitriKoota(
        Graha bride,
        Graha groom,
        MaitriRelation relation,
        boolean lifted) implements KootaReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#GRAHA_MAITRI}
     */
    @Override
    public Koota koota() {
        return Koota.GRAHA_MAITRI;
    }
}
