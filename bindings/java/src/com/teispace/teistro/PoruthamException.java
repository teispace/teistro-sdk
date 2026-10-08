package com.teispace.teistro;

/**
 * The p. 76 exception's clauses, any one of which lifts Ganam, Rasi, Rajju and Vedhai (C277).
 *
 * @param oneLord One lord rules both signs.
 * @param lordsFriendly The lords are friendly.
 * @param opposite The signs are opposite.
 */
public record PoruthamException(
        boolean oneLord,
        boolean lordsFriendly,
        boolean opposite) {
}
