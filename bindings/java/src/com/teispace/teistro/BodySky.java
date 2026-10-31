package com.teispace.teistro;

import java.util.List;

/**
 * One body's day.
 *
 * @param state {@code CROSSES}, or on a day with no rise or set {@code ALWAYS_ABOVE} or {@code ALWAYS_BELOW}
 * @param track every step's reading and every exact event, in time order
 */
public record BodySky(String state, List<SkyPoint> track) {
    /** Keeps the list unmodifiable. */
    public BodySky {
        track = List.copyOf(track);
    }
}
