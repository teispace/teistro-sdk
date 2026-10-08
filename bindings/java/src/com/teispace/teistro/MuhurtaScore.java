package com.teispace.teistro;

import java.util.List;

/**
 * The baseline engine's score for a window, under its ranking.
 *
 * @param value the score
 * @param factors the weights it is made of
 * @param cappedAt the cap a Mahadosha put on it; may be null, when none did
 */
public record MuhurtaScore(int value, List<MuhurtaFactor> factors, Integer cappedAt) {
    /** Keeps the list unmodifiable. */
    public MuhurtaScore {
        factors = List.copyOf(factors);
    }
}
