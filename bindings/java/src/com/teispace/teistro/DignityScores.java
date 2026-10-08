package com.teispace.teistro;

/**
 * What each dignity and debility was worth.
 *
 * @param house the score of a planet in its own house
 * @param exaltation the score of its exaltation
 * @param triplicity the score of its triplicity
 * @param term the score of its term
 * @param face the score of its face
 * @param detriment the score of its detriment
 * @param fall the score of its fall
 * @param peregrine the score of a planet in none of its five dignities
 */
public record DignityScores(
        int house, int exaltation, int triplicity, int term, int face, int detriment, int fall, int peregrine) {}
