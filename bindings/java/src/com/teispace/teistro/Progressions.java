package com.teispace.teistro;

import java.util.List;

/**
 * A birth read through its progressions
 * ({@code 03-design/western-progressions.md}): {@code progressed} and
 * {@code directed} are null without {@code at}, {@code contacts} null
 * without a window.
 *
 * <pre>{@code
 * ProgressedPlanet moon = chart.progressions().orElseThrow().progressed().grahas().stream()
 *         .filter(g -> g.graha() == Graha.MOON).findFirst().orElseThrow();
 * }</pre>
 *
 * @param progressed the progressed chart at {@code at}; may be null
 * @param directed the direction at {@code at}; may be null
 * @param contacts the contacts in the window; may be null
 */
public record Progressions(Progressed progressed, Directed directed, List<ProgressedContact> contacts) {
    /** Keeps the list unmodifiable. */
    public Progressions {
        contacts = contacts == null ? null : List.copyOf(contacts);
    }
}
