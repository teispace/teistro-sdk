package com.teispace.teistro;

/**
 * A muhurta yoga that held, and what made it hold.
 *
 * @param yoga which yoga
 * @param at while it held, clipped to the day
 * @param vara the vara that makes it; every cause has one
 * @param tithi the tithi that makes it; may be null, when the cause has none
 * @param nakshatra the nakshatra that makes it
 */
public record HeldYoga(MuhurtaYoga yoga, Interval at, Vara vara, Tithi tithi, Nakshatra nakshatra) {}
