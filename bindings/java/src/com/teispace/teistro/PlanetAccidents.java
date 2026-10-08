package com.teispace.teistro;

import java.util.List;

/**
 * One planet's accidental fortitudes and debilities.
 *
 * @param planet the planet
 * @param house its house, 1 to 12, under the five-degree rule
 * @param accidents every line beyond its house, in Lilly's order
 * @param fortitude the sum of its fortitudes, its house's included
 * @param debility the sum of its debilities, its house's included, as a
 *     positive number
 * @param net Lilly's net over the whole table: its essential
 *     {@code score + reception} and {@code fortitude - debility}
 */
public record PlanetAccidents(
        Graha planet, int house, List<AccidentLine> accidents, int fortitude, int debility, int net) {
    /** Keeps the list unmodifiable. */
    public PlanetAccidents {
        accidents = List.copyOf(accidents);
    }
}
