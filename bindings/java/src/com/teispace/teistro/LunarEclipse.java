package com.teispace.teistro;

/**
 * A lunar eclipse: its kind, gamma, magnitudes and contacts under a rule
 * for the Earth's shadow.
 *
 * @param greatest the greatest eclipse, a UT1 Julian day
 * @param kind its kind
 * @param gamma the Moon's centre from the shadow's axis at the greatest
 *     eclipse, in Earth radii, positive when the Moon passes north of it
 * @param umbralMagnitude negative for a penumbral eclipse, 1 or more for a total one
 * @param penumbralMagnitude the penumbral magnitude
 * @param contacts its contacts
 * @param shadow the rule that sized the shadow, {@code panchanga.eclipse_shadow}:
 *     {@code DANJON} or {@code CHAUVENET}
 */
public record LunarEclipse(
        double greatest, LunarEclipseKind kind, double gamma, double umbralMagnitude, double penumbralMagnitude,
        LunarContacts contacts, String shadow) {}
