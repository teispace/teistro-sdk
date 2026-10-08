package com.teispace.teistro;

/**
 * The Ascendant's degree (p. 122).
 *
 * @param sign the rising sign
 * @param degree degrees within the sign, [0, 30)
 * @param early fewer than 3 degrees rise
 * @param late 27 degrees or more rise
 * @param shortAscension a sign of short ascension, Capricorn to Gemini
 */
public record AscendantClause(Rashi sign, double degree, boolean early, boolean late, boolean shortAscension) {}
