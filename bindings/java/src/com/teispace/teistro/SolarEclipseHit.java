package com.teispace.teistro;

/**
 * A solar eclipse fell on a natal point, within the request's eclipse orb (C421).
 *
 * @param to The natal point the eclipsed Sun fell on.
 * @param eclipse Which kind of eclipse.
 */
public record SolarEclipseHit(
        NatalPoint to,
        SolarEclipseKind eclipse) implements HitEvent {
    /**
     * Which kind of event this is.
     *
     * @return {@link HitKind#SOLAR_ECLIPSE}
     */
    @Override
    public HitKind kind() {
        return HitKind.SOLAR_ECLIPSE;
    }
}
