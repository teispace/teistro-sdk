package com.teispace.teistro;

/**
 * The largest phase of a solar eclipse the place sees: the maximum when the Sun is up at it, else
 * where it rises or sets. The moment and the magnitudes are null exactly when nothing is seen.
 *
 * @param why where it falls
 * @param moment when, with the Sun's topocentric geometric altitude; may be null
 * @param magnitude the magnitude then; may be null
 * @param obscuration the obscuration then; may be null
 */
public record SolarSeenMaximum(SeenAt why, EclipseMoment moment, Double magnitude, Double obscuration) {}
