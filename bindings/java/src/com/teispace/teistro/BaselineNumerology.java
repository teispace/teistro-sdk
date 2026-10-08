package com.teispace.teistro;

/**
 * The numbers only the baseline engine computes, with no public-domain
 * source in hand.
 *
 * @param soul the vowels A, E, I, O and U under Balliett's cycle
 * @param personality the other letters
 * @param chaldeanDestiny the date's digit sum under Cheiro, which he forbids (p. 93)
 */
public record BaselineNumerology(Reduction soul, Reduction personality, Reduction chaldeanDestiny) {}
