package com.teispace.teistro;

/**
 * One karaka bhava of a chart (BPHS ch. 32 vv. 22 to 24): a house counted
 * from a graha's sign, whole signs in the rasi chart.
 *
 * @param graha The graha counted from.
 * @param house The house counted from its sign, 1 to 12.
 * @param signification What that house shows.
 * @param sign The sign it falls in.
 */
public record KarakaBhava(Graha graha, int house, Signification signification, Rashi sign) {}
