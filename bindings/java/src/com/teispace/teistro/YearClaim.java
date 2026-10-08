package com.teispace.teistro;

/**
 * One office-bearer's claim on the year's lordship.
 *
 * @param graha Whose claim it is.
 * @param vishwa Its five-fold strength.
 * @param portfolios How many of the five offices it holds, 1 to 5: the tie-break.
 * @param aspectsLagna Whether it gives the Tajika aspect to the annual lagna, which it must to
 *     hold the year.
 */
public record YearClaim(
        Graha graha,
        Bala vishwa,
        int portfolios,
        boolean aspectsLagna) {
}
