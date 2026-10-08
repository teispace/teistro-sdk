package com.teispace.teistro;

/**
 * How the two Moon signs stand in Vashya (VI.23, C260).
 *
 * @param relation How they stand.
 */
public record VashyaKoota(
        VashyaRelation relation) implements KootaReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#VASHYA}
     */
    @Override
    public Koota koota() {
        return Koota.VASHYA;
    }
}
