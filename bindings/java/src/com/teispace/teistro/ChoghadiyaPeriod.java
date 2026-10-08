package com.teispace.teistro;

/**
 * One choghadiya, of the daylight or of the night.
 *
 * @param choghadiya which choghadiya
 * @param lord the graha that rules it
 * @param at when it runs
 * @param daytime whether it is one of the eight of the daylight
 */
public record ChoghadiyaPeriod(Choghadiya choghadiya, Graha lord, Interval at, boolean daytime) {}
