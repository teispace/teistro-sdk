package com.teispace.teistro;

/**
 * A graha in transit: the sign it stands in and its degrees within it.
 *
 * @param sign The sign.
 * @param degrees Degrees within the sign, 0 to 30.
 */
public record Transit(
        Rashi sign,
        double degrees) {
}
