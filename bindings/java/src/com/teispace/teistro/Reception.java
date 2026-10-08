package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * Two planets each standing in at least one of the other's five dignities
 * (Lilly, p. 112), each side reported whole.
 *
 * <pre>{@code
 * List<Reception> byHouse = dignities.receptions().stream()
 *         .filter(one -> one.mutual().contains("house")).toList();
 * }</pre>
 *
 * @param planets the two, in the Chaldean order
 * @param firstIn the second's dignities where the first stands
 * @param secondIn the first's dignities where the second stands
 */
public record Reception(List<Graha> planets, EssentialDignity firstIn, EssentialDignity secondIn) {
    /** Keeps the pair unmodifiable. */
    public Reception {
        planets = List.copyOf(planets);
    }

    /**
     * The kinds each stands in of the other's, strongest first; empty for a
     * mixed reception.
     *
     * @return the names of the dignities, {@code house} to {@code face}
     */
    public List<String> mutual() {
        List<String> out = new ArrayList<>();
        for (String kind : EssentialDignity.KINDS) {
            if (firstIn.holds(kind) && secondIn.holds(kind)) {
                out.add(kind);
            }
        }
        return Collections.unmodifiableList(out);
    }
}
