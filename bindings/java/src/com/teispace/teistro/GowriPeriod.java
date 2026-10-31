package com.teispace.teistro;

/**
 * One Gowri of the Gowri panchangam, of the daylight or of the night.
 *
 * @param gowri which Gowri, which carries its lord and whether it is auspicious
 * @param at when it runs
 * @param daytime whether it is one of the eight of the daylight
 */
public record GowriPeriod(Gowri gowri, Interval at, boolean daytime) {}
