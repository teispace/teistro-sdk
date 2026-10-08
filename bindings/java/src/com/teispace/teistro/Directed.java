package com.teispace.teistro;

import java.util.List;

/**
 * A birth's points moved by one arc.
 *
 * @param life the instant of life, a UTC Julian day
 * @param arcDeg the arc; a solar arc is signed
 * @param ascendantDeg the directed ascendant
 * @param midheavenDeg the directed midheaven
 * @param planets the directed planets
 */
public record Directed(double life, double arcDeg, double ascendantDeg, double midheavenDeg,
        List<DirectedPlanet> planets) {
    /** Keeps the list unmodifiable. */
    public Directed {
        planets = List.copyOf(planets);
    }
}
