package com.teispace.teistro;

import java.util.List;

/**
 * The lord of the year, and the reckoning it came out of.
 *
 * @param graha The lord of the year.
 * @param chosen Which step of the chain decided it.
 * @param vishwa Its five-fold strength.
 * @param moonPassedOver Whether the Moon led on strength and stepped aside, being "unable to
 *     govern".
 * @param claims Every claimant, strongest first, so the decision can be read rather than trusted.
 */
public record YearLord(
        Graha graha,
        VarsheshaChosen chosen,
        Bala vishwa,
        boolean moonPassedOver,
        List<YearClaim> claims) {
    /** The value, its lists copied and unmodifiable. */
    public YearLord {
        claims = List.copyOf(claims);
    }
}
