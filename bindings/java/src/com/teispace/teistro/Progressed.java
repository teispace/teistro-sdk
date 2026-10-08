package com.teispace.teistro;

import java.util.List;

/**
 * The chart at the instant of sky that measures an instant of life.
 *
 * @param life the instant of life, a UTC Julian day
 * @param sky the instant of sky that measures it, a UTC Julian day
 * @param armcDeg the progressed meridian's right ascension
 * @param angles the progressed angles
 * @param grahas the progressed planets
 */
public record Progressed(double life, double sky, double armcDeg, ProgressedAngles angles,
        List<ProgressedPlanet> grahas) {
    /** Keeps the list unmodifiable. */
    public Progressed {
        grahas = List.copyOf(grahas);
    }
}
