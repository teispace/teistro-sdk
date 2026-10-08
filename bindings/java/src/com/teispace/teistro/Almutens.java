package com.teispace.teistro;

import java.util.List;

/**
 * A chart's almutens three ways, with the rules that made them.
 *
 * <pre>{@code
 * List<Graha> lord = chart.fortitudes().orElseThrow().almutens().figure().almutens();
 * }</pre>
 *
 * @param rules how they were read
 * @param fortuneDeg the Part of Fortune, one of the five places
 * @param figure Lilly's almuten of the figure: each planet's {@code net}
 * @param places Chapter CV's: essential dignities over the ascendant,
 *     midheaven, Sun, Moon and Fortune
 * @param houses each house's, of its cusp, the first to the twelfth
 */
public record Almutens(AlmutenRules rules, double fortuneDeg, Almuten figure, Almuten places, List<Almuten> houses) {
    /** Keeps the list unmodifiable. */
    public Almutens {
        houses = List.copyOf(houses);
    }
}
