package com.teispace.teistro;

import java.util.List;

/**
 * A birth read through its progressions
 * ({@code 03-design/western-progressions.md}): {@code progressed} and
 * {@code directed} are null without {@code at}, {@code contacts} null
 * without a window, and {@code parallels} and {@code mutual} null unless
 * the window asked for them.
 *
 * <pre>{@code
 * ProgressedPlanet moon = chart.progressions().orElseThrow().progressed().grahas().stream()
 *         .filter(g -> g.graha() == Graha.MOON).findFirst().orElseThrow();
 * }</pre>
 *
 * @param progressed the progressed chart at {@code at}; may be null
 * @param directed the direction at {@code at}; may be null
 * @param contacts the contacts in the window; may be null
 * @param parallels the parallels of declination in the window, by their
 *     closest instant; may be null
 * @param mutual the progressed planets' contacts among themselves, as they
 *     fall due; may be null
 */
public record Progressions(
        Progressed progressed, Directed directed, List<ProgressedContact> contacts,
        List<ProgressedParallel> parallels, List<MutualContact> mutual) {
    /** Keeps the lists unmodifiable. */
    public Progressions {
        contacts = contacts == null ? null : List.copyOf(contacts);
        parallels = parallels == null ? null : List.copyOf(parallels);
        mutual = mutual == null ? null : List.copyOf(mutual);
    }
}
