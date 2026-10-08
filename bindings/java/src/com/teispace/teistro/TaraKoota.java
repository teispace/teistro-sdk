package com.teispace.teistro;

/**
 * The taras each way, 1 to 9 (VI.24); the 3rd, 5th and 7th are bad.
 *
 * @param brideToGroom Counted from the bride's nakshatra to the groom's.
 * @param groomToBride Counted from the groom's nakshatra to the bride's.
 */
public record TaraKoota(
        int brideToGroom,
        int groomToBride) implements KootaReading {
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
