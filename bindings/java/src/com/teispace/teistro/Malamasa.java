package com.teispace.teistro;

import java.util.Objects;

/**
 * The month rule an observance was placed by, and where the texts say it
 * (SR-93, {@code 03-design/festival-malamasa.md}).
 *
 * @param clause which clause
 * @param source where the texts say it
 */
public record Malamasa(MalamasaClause clause, String source) {
    /** Checks both are present. */
    public Malamasa {
        Objects.requireNonNull(clause, "clause");
        Objects.requireNonNull(source, "source");
    }
}
