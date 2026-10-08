package com.teispace.teistro;

/**
 * A graha's standing in one scheme of vargas.
 *
 * @param goodVargas How many of the scheme's vargas are good for it.
 * @param name The name that count earns, from two good vargas; null below. May be null.
 */
public record VaiseshikamsaStanding(
        int goodVargas,
        Vaiseshikamsa name) {
}
