package com.teispace.teistro;

import java.util.Objects;

/**
 * The moonrise an observance was kept or broken at, and where (SR-60): the
 * instant moves with the place, so the place it was found for is beside it.
 *
 * @param at the instant the Moon rose, a UTC Julian day
 * @param place the place the days were reckoned at
 */
public record FestivalMoonrise(double at, Observer place) {
    /** Checks the place is present. */
    public FestivalMoonrise {
        Objects.requireNonNull(place, "place");
    }
}
