package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * One period of civil days to read for each of the twelve signs
 * ({@code 03-design/rashifal.md}).
 *
 * <pre>{@code
 * RashifalRequest week = RashifalRequest.of(first, kathmandu, 20_700).withLast(last);
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
 */
public record RashifalRequest(
        CalendarDate first, CalendarDate last, Observer place, int utcOffsetSeconds, Map<String, ?> snapshot,
        List<?> events, List<Integer> spells) {
    /** Keeps the collections unmodifiable. */
    public RashifalRequest {
        snapshot = snapshot == null ? null : Map.copyOf(snapshot);
        events = events == null ? null : List.copyOf(events);
        spells = spells == null ? null : List.copyOf(spells);
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
        return new RashifalRequest(first, null, place, utcOffsetSeconds, null, null, null);
    }

    /**
     * The same request ending on another day.
     *
     * @param day the last day
     * @return the request
     */
    public RashifalRequest withLast(CalendarDate day) {
        return new RashifalRequest(first, day, place, utcOffsetSeconds, snapshot, events, spells);
    }

    /**
     * The same request read at another moment of the reference day.
     *
     * @param at such as {@code Map.of("at", "CLOCK", "hour", 6, "minute", 0)}
     * @return the request
     */
    public RashifalRequest withSnapshot(Map<String, ?> at) {
        return new RashifalRequest(first, last, place, utcOffsetSeconds, at, events, spells);
    }

    /**
     * The same request reporting other grahas' ingresses and stations.
     *
     * @param grahas the grahas, as members or keys
     * @return the request
     */
    public RashifalRequest withEvents(List<?> grahas) {
        return new RashifalRequest(first, last, place, utcOffsetSeconds, snapshot, grahas, spells);
    }

    /**
     * The same request with other smaller spells of Saturn.
     *
     * @param houses the houses, 3 to 11
     * @return the request
     */
    public RashifalRequest withSpells(List<Integer> houses) {
        return new RashifalRequest(first, last, place, utcOffsetSeconds, snapshot, events, houses);
    }
}
