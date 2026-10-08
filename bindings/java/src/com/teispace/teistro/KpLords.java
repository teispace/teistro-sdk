package com.teispace.teistro;

/**
 * A point's lords: of its sign, its star, its sub and its sub-sub.
 *
 * @param sign The sign's lord.
 * @param star The star's lord.
 * @param sub The sub's lord.
 * @param subSub The sub-sub's lord.
 */
public record KpLords(
        Graha sign,
        KpLevel star,
        KpLevel sub,
        KpLevel subSub) {
}
