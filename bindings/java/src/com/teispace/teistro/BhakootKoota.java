package com.teispace.teistro;

/**
 * How far the groom's Moon sign stands from the bride's (VI.31 to 33).
 *
 * @param apart The groom's sign counted from the bride's, 1 to 12.
 * @param dosha The bad Bhakoot the signs stand at, or null. May be null.
 * @param exceptions The exceptions' clauses.
 * @param lifted Whether the exceptions lift the dosha under the rules; false with no dosha.
 */
public record BhakootKoota(
        int apart,
        BhakootDosha dosha,
        BhakootExceptions exceptions,
        boolean lifted) implements KootaReading {
    /**
     * Which koota this is.
     *
     * @return {@link Koota#BHAKOOT}
     */
    @Override
    public Koota koota() {
        return Koota.BHAKOOT;
    }
}
