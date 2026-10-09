package com.teispace.teistro;

import java.util.Objects;

/**
 * One birth of a study ({@code 03-design/research.md}).
 *
 * @param instant the instant, a Julian day in UTC
 * @param place where
 * @param utcOffsetSeconds the local clock's offset from UTC in seconds, east positive
 * @param uncertaintyMinutes how far either side the recorded time may be wrong, 0 to 720 minutes; a
 *     rule whose answer differs at either edge is counted unstable on the chart
 */
public record ResearchBirth(double instant, Observer place, int utcOffsetSeconds, double uncertaintyMinutes) {
    /** The value, its place required. */
    public ResearchBirth {
        Objects.requireNonNull(place, "place");
    }

    /**
     * A birth whose recorded time is trusted.
     *
     * @param instant the instant, a Julian day in UTC
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC in seconds
     */
    public ResearchBirth(double instant, Observer place, int utcOffsetSeconds) {
        this(instant, place, utcOffsetSeconds, 0);
    }
}
