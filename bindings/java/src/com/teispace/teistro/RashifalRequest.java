package com.teispace.teistro;

import java.util.List;

/**
 * One period of civil days to read for each of the twelve signs
 * ({@code 03-design/rashifal.md}).
 *
 * <pre>{@code
 * RashifalRequest week = RashifalRequest.of(first, kathmandu, 20_700).withLast(last);
 * RashifalRequest zoned = RashifalRequest.of(first, kathmandu, Civil.ianaZone("Asia/Kathmandu"));
 * }</pre>
 *
 * @param first the first day; the period is in its calendar
 * @param last the last day, in the first's calendar; may be null, for the first
 * @param place where
 * @param utcOffsetSeconds the local clock's offset from UTC, east positive
 * @param snapshot when in the reference day the sky is read:
 *     {@code {"at": "SUNRISE"}} by default or
 *     {@code {"at": "CLOCK", "hour": 6, "minute": 0}} (C358); may be null
 * @param events the grahas whose ingresses and stations are reported, as
 *     members or keys (every one but the Moon by default, C360); may be null
 * @param spells Saturn's smaller spells (the 4th and 8th by default, C149); may be null
 * @param zone the zone each day is read on, at the offset it had then, in place of
 *     {@code utcOffsetSeconds}, which is then not sent ({@code 03-design/time-and-timezone.md}
 *     §3.4); may be null, for the offset
 */
public record RashifalRequest(
        CalendarDate first, CalendarDate last, Observer place, int utcOffsetSeconds,
        com.teispace.teistro.request.Snapshot snapshot,
        List<?> events, List<Integer> spells, ZoneSpec zone) {
    /** Keeps the collections unmodifiable. */
    public RashifalRequest {
        events = events == null ? null : List.copyOf(events);
        spells = spells == null ? null : List.copyOf(spells);
    }

    /**
     * A period on an offset, as before the zone was a component.
     *
     * @param first the first day
     * @param last the last day; may be null
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param snapshot when in the reference day the sky is read; may be null
     * @param events the grahas whose ingresses and stations are reported; may be null
     * @param spells Saturn's smaller spells; may be null
     */
    public RashifalRequest(CalendarDate first, CalendarDate last, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.Snapshot snapshot, List<?> events, List<Integer> spells) {
        this(first, last, place, utcOffsetSeconds, snapshot, events, spells, null);
    }

    /**
     * A period of one day, or of more with {@link #withLast}, under every default.
     *
     * @param first the first day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @return the request
     */
    public static RashifalRequest of(CalendarDate first, Observer place, int utcOffsetSeconds) {
        return new RashifalRequest(first, null, place, utcOffsetSeconds, null, null, null, null);
    }

    /**
     * A period of one day on a zone's clock, each day read at the offset the zone had then.
     *
     * @param first the first day
     * @param place where
     * @param zone the zone, from {@code Civil.ianaZone} and its neighbours
     * @return the request
     */
    public static RashifalRequest of(CalendarDate first, Observer place, ZoneSpec zone) {
        return new RashifalRequest(first, null, place, 0, null, null, null, zone);
    }

    /**
     * The same request on a zone's clock, in place of its offset.
     *
     * @param on the zone, from {@code Civil.ianaZone} and its neighbours
     * @return the request
     */
    public RashifalRequest withZone(ZoneSpec on) {
        return new RashifalRequest(first, last, place, utcOffsetSeconds, snapshot, events, spells, on);
    }

    /**
     * The same request ending on another day.
     *
     * @param day the last day
     * @return the request
     */
    public RashifalRequest withLast(CalendarDate day) {
        return new RashifalRequest(first, day, place, utcOffsetSeconds, snapshot, events, spells, zone);
    }

    /**
     * The same request read at another moment of the reference day.
     *
     * @param at such as {@code new SnapshotClock(6, 0)}, or
     *     {@code Snapshot.of(map)} from a map
     * @return the request
     */
    public RashifalRequest withSnapshot(com.teispace.teistro.request.Snapshot at) {
        return new RashifalRequest(first, last, place, utcOffsetSeconds, at, events, spells, zone);
    }

    /**
     * The same request reporting other grahas' ingresses and stations.
     *
     * @param grahas the grahas, as members or keys
     * @return the request
     */
    public RashifalRequest withEvents(List<?> grahas) {
        return new RashifalRequest(first, last, place, utcOffsetSeconds, snapshot, grahas, spells, zone);
    }

    /**
     * The same request with other smaller spells of Saturn.
     *
     * @param houses the houses, 3 to 11
     * @return the request
     */
    public RashifalRequest withSpells(List<Integer> houses) {
        return new RashifalRequest(first, last, place, utcOffsetSeconds, snapshot, events, houses, zone);
    }
}
