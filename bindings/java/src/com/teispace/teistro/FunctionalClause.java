package com.teispace.teistro;

/**
 * A Laghu Parashari clause that made a graha's nature, by {@code kind} ({@code LAGNESHA},
 * {@code TRIKONA_LORD}, {@code MARAKA}, and so on), and the house it read.
 *
 * @param kind The clause.
 * @param house The house it read.
 */
public record FunctionalClause(
        String kind,
        int house) {
}
