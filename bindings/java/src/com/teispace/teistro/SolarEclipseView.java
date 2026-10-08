package com.teispace.teistro;

/**
 * A solar eclipse at the place: its own contacts, maximum and magnitude,
 * and the stretch seen, or null when the Sun was down throughout.
 *
 * @param kind what the place sees at its maximum: never {@code HYBRID}
 * @param magnitude the magnitude at the maximum
 * @param obscuration the fraction of the Sun's disc covered at the maximum
 * @param first the first contact
 * @param second the second contact; may be null
 * @param third the third contact; may be null
 * @param fourth the fourth contact
 * @param maximum the maximum
 * @param seen the stretch seen; may be null
 */
public record SolarEclipseView(
        SolarEclipseKind kind, double magnitude, double obscuration, EclipseMoment first, EclipseMoment second,
        EclipseMoment third, EclipseMoment fourth, EclipseMoment maximum, EclipseSeen seen) {}
