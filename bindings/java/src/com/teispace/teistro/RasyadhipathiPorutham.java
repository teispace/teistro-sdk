package com.teispace.teistro;

/**
 * Rasyadhipathi: the two Moon signs' lords on the chapter's own friendships (pp. 74 to 75).
 *
 * @param bride The bride's sign lord.
 * @param groom The groom's sign lord.
 * @param brideCallsFriend Whether the bride's lord calls the groom's a friend; a lord is its own.
 * @param groomCallsFriend Whether the groom's lord calls the bride's a friend.
 */
public record RasyadhipathiPorutham(
        Graha bride,
        Graha groom,
        boolean brideCallsFriend,
        boolean groomCallsFriend) implements PoruthamReading {
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
