package com.teispace.teistro;

/**
 * The Kuja dosha of a bride and a groom (<i>Manasagari</i>, jayabhava v. 4), as clauses, with
 * each side's lifts (C288, C755).
 *
 * @param bride The bride's.
 * @param groom The groom's.
 * @param both Whether both carry it, the fact the popular cancellation reads.
 */
public record Kuja(
        KujaSide bride,
        KujaSide groom,
        boolean both) {
}
