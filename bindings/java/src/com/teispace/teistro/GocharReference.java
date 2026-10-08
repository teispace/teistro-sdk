package com.teispace.teistro;

/**
 * What a gochar reading counted its houses from, and that point's sign.
 *
 * @param from The natal Moon (v. 1) or, asked, the lagna (C139).
 * @param sign Its sign.
 */
public record GocharReference(
        GocharFrom from,
        Rashi sign) {
}
