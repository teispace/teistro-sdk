package com.teispace.teistro;

/**
 * Where one body stands in a divisional chart.
 *
 * @param rashi The sign the body stands in, in the rashi chart.
 * @param part Which part of that sign it falls in, counted from zero.
 * @param sign The sign the divisional chart puts it in.
 */
public record VargaPlacement(
        Rashi rashi,
        int part,
        Rashi sign) {
    /**
     * Whether the divisional chart leaves the body where it was. In the
     * navamsha this is <b>vargottama</b>, the term the texts use; in another
     * chart it is the same fact without the name.
     *
     * @return true when the sign is the rashi chart's
     */
    public boolean keepsItsSign() {
        return sign == rashi;
    }
}
