package com.teispace.teistro;

import java.util.List;

/**
 * The seventh house and its lord (pp. 122–123).
 *
 * @param cuspDeg the seventh cusp
 * @param lord its lord
 * @param infortunesInHouse Saturn and Mars when counted in the seventh house (C231)
 * @param lordRetrograde whether the lord is retrograde
 * @param lordCombust whether the lord is combust
 * @param lordInFall whether the lord is in its fall
 * @param lordInInfortuneTerm whether the lord is in an infortune's term
 * @param lordNet essential and accidental fortitudes less debilities
 */
public record SeventhClause(
        double cuspDeg, Graha lord, List<Graha> infortunesInHouse, boolean lordRetrograde, boolean lordCombust,
        boolean lordInFall, boolean lordInInfortuneTerm, int lordNet) {
    /** Keeps the list unmodifiable. */
    public SeventhClause {
        infortunesInHouse = List.copyOf(infortunesInHouse);
    }
}
