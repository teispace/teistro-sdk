package com.teispace.teistro;

/**
 * How one of the seven stands to a saham.
 *
 * @param graha Which planet.
 * @param drishti The Tajika aspect its sign casts on the saham's.
 * @param relation How it stands to the saham's lord, under the friendship read.
 * @param company Whether it keeps the saham company, in the saham's sign.
 */
public record SahamSeven(
        Graha graha,
        TajikaDrishti drishti,
        TajikaRelation relation,
        boolean company) {
}
