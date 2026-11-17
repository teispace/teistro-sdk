package com.teispace.teistro;

/**
 * A stretch the Sun and the Moon close to a rite ({@code 03-design/panchanga-reach.md} §2).
 *
 * @param kind which blackout
 * @param at when it holds, clipped to the range asked
 */
public record Blackout(BlackoutKind kind, Interval at) {}
