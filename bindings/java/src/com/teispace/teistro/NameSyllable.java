package com.teispace.teistro;

/**
 * A name's first syllable in the śatapada cakra (<i>Svarodaya</i> vv. 3–8).
 *
 * @param cell its place among the cakra's 112 cells, 0 for a, Krittika's first
 * @param nakshatra its star; may be null, for Abhijit, which is none of the
 *     27; the name rules' {@code abhijit} decides the star it is matched as
 * @param quarter which of the star's four syllables it is, 1 to 4: the pada,
 *     for one of the 27
 * @param varga the letter group the name begins in, as written (VI.35)
 */
public record NameSyllable(int cell, Nakshatra nakshatra, int quarter, NameVarga varga) {}
