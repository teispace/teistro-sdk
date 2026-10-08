package com.teispace.teistro;

import java.util.List;

/**
 * The lords a year's dasha runs round, and where it opens.
 *
 * @param shares In the order the ring runs.
 * @param first The place in {@code shares} the year opens with, from 0.
 * @param remaining How much of the first lord's share was still to run at the return, 0 to 1, the
 *     rest closing the year; null when it runs whole from the return: the Patyayini, and a
 *     {@code WHOLE} balance. May be null.
 */
public record DashaRing(
        List<AnnualDashaShare> shares,
        int first,
        Double remaining) {
    /** The value, its lists copied and unmodifiable. */
    public DashaRing {
        shares = List.copyOf(shares);
    }
}
