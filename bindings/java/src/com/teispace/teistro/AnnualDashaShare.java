package com.teispace.teistro;

/**
 * One lord of the ring a year's dasha runs round.
 *
 * @param lord Its lord: the graha, or the sign's lord when the share is a sign's.
 * @param sign The sign, when the share is one's: the Patyayini's lagna; null for a planet's. May
 *     be null.
 * @param weight Its weight, of which a lord's share of the year is its weight over the ring's: a
 *     nakshatra year's lord's natal years, or a Patyayini share's patyamsha in nanoarcseconds. 0
 *     for a lord that runs for no time.
 */
public record AnnualDashaShare(
        Graha lord,
        Rashi sign,
        double weight) {
}
