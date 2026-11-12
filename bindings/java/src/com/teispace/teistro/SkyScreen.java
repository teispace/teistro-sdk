package com.teispace.teistro;

import java.util.List;

/**
 * Each point of a sky view placed in a camera's picture ({@code 03-design/sky-pose.md}): one placement for each
 * point of the sky's list of the same name, in the same order.
 *
 * @param grahas the sky's grahas, at their refracted altitudes
 * @param stars the sky's stars, at their refracted altitudes
 * @param ecliptic the sky's ecliptic points, at their refracted altitudes
 * @param horizon the sky's horizon points, at altitude nought
 */
public record SkyScreen(List<ScreenPlacement> grahas, List<ScreenPlacement> stars, List<ScreenPlacement> ecliptic,
        List<ScreenPlacement> horizon) {
    /** Keeps the lists unmodifiable. */
    public SkyScreen {
        grahas = List.copyOf(grahas);
        stars = List.copyOf(stars);
        ecliptic = List.copyOf(ecliptic);
        horizon = List.copyOf(horizon);
    }
}
