package com.teispace.teistro;

import java.util.Objects;

/**
 * One birth of a study ({@code 03-design/research.md}).
 *
 * <pre>{@code
 * ResearchBirth stated = new ResearchBirth(2_446_217.77, kathmandu, 19_800);
 * ResearchBirth zoned = ResearchBirth.inZone(2_446_217.77, kathmandu, Civil.ianaZone("Asia/Kathmandu"), 0);
 * }</pre>
 *
 * @param instant the instant, a Julian day in UTC
 * @param place where
 * @param utcOffsetSeconds the local clock's offset from UTC in seconds, east positive; not sent when
 *     {@code zone} is given
 * @param uncertaintyMinutes how far either side the recorded time may be wrong, 0 to 720 minutes; a
 *     rule whose answer differs anywhere in that window, read every step of the study, is counted
 *     unstable on the chart
 * @param zone the zone the birth is read on, at the offset it had then, in place of
 *     {@code utcOffsetSeconds} ({@code 03-design/time-and-timezone.md} §3.4); may be null, for the offset
 */
public record ResearchBirth(double instant, Observer place, int utcOffsetSeconds, double uncertaintyMinutes,
        ZoneSpec zone) {
    /** The value, its place required. */
    public ResearchBirth {
        Objects.requireNonNull(place, "place");
    }

    /**
     * A birth on a stated offset, as before the zone was a component.
     *
     * @param instant the instant, a Julian day in UTC
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC in seconds
     * @param uncertaintyMinutes how far either side the recorded time may be wrong, 0 to 720 minutes
     */
    public ResearchBirth(double instant, Observer place, int utcOffsetSeconds, double uncertaintyMinutes) {
        this(instant, place, utcOffsetSeconds, uncertaintyMinutes, null);
    }

    /**
     * A birth whose recorded time is trusted.
     *
     * @param instant the instant, a Julian day in UTC
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC in seconds
     */
    public ResearchBirth(double instant, Observer place, int utcOffsetSeconds) {
        this(instant, place, utcOffsetSeconds, 0, null);
    }

    /**
     * A birth read on a zone, at the offset the zone had at the instant.
     *
     * @param instant the instant, a Julian day in UTC
     * @param place where
     * @param zone the zone, from {@code Civil.ianaZone} and its neighbours
     * @param uncertaintyMinutes how far either side the recorded time may be wrong, 0 to 720 minutes
     * @return the birth
     */
    public static ResearchBirth inZone(double instant, Observer place, ZoneSpec zone, double uncertaintyMinutes) {
        return new ResearchBirth(instant, place, 0, uncertaintyMinutes, Objects.requireNonNull(zone, "zone"));
    }
}
