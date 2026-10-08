package com.teispace.teistro;

/**
 * Dhinam: the count and the rule of <i>Kalaprakasika</i> XIII that decided it (pp. 69 to 72).
 *
 * @param count The groom's nakshatra counted from the bride's, 1 to 27.
 * @param rule The rule that decided it.
 */
public record DhinamPorutham(
        int count,
        DhinamRule rule) implements PoruthamReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#TARA}
     */
    @Override
    public Koota koota() {
        return Koota.TARA;
    }
}
