package com.teispace.teistro;

import java.util.List;

/**
 * The amatya's devotions in one chart, read as BPHS vv. 77 to 79 read them: the 12th from its
 * navamsha, read as the karakamsha's (vv. 77 to 78); its own sign and its house from that chart's
 * lagna, 1 to 12; and each graha joined to it there with its devotion (v. 79).
 *
 * @param twelfth The 12th from its navamsha.
 * @param sign Its own sign.
 * @param house Its house from the chart's lagna, 1 to 12.
 * @param joined Each graha joined to it, with its devotion.
 */
public record AmatyaDevata(
        IshtaDevata twelfth,
        Rashi sign,
        int house,
        List<Devotion> joined) {
    /** The value, its lists copied and unmodifiable. */
    public AmatyaDevata {
        joined = List.copyOf(joined);
    }
}
