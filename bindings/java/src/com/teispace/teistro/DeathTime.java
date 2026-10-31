package com.teispace.teistro;

import java.util.List;

/**
 * The death-time check ({@code 03-design/death-time.md}): the readings and the rites owed.
 *
 * @param death the death's reading
 * @param cremation the burning's reading, when its instant was given; null otherwise
 * @param rites the rites owed; with no burning given, it is taken to fall as the death did
 * @param table the table's name
 */
public record DeathTime(DeathInstant death, DeathInstant cremation, List<DeathRite> rites, String table) {
    /** Keeps the lists unmodifiable. */
    public DeathTime {
        rites = List.copyOf(rites);
    }
}
