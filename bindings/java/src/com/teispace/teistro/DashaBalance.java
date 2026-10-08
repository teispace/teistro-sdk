package com.teispace.teistro;

/**
 * What remained of a dasha's first period at birth.
 *
 * @param method How it was measured.
 * @param remaining The fraction still to run, 0 to 1.
 * @param days That fraction of the first lord's years, in days.
 * @param written The same in years, months, days, hours and minutes.
 */
public record DashaBalance(
        Balance method,
        double remaining,
        double days,
        WrittenBalance written) {
}
