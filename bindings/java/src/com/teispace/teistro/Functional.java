package com.teispace.teistro;

import java.util.List;

/**
 * What a lagna's lordships make of the seven grahas (Laghu Parashari).
 *
 * @param lagna The lagna.
 * @param scheme The scheme read.
 * @param rows The seven grahas that own signs, Sun to Saturn.
 * @param yogakarakas The yogakarakas.
 * @param marakas The lords of the 2nd and the 7th (LP v. 23).
 * @param badhaka The badhaka house and its lord.
 */
public record Functional(
        Rashi lagna,
        String scheme,
        List<FunctionalRow> rows,
        List<Graha> yogakarakas,
        List<Graha> marakas,
        Badhaka badhaka) {
    /** The value, its lists copied and unmodifiable. */
    public Functional {
        rows = List.copyOf(rows);
        yogakarakas = List.copyOf(yogakarakas);
        marakas = List.copyOf(marakas);
    }
}
