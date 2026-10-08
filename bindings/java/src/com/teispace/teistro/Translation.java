package com.teispace.teistro;

/**
 * A lighter planet carrying one significator's light to the other (p. 111).
 *
 * @param translator the lighter planet
 * @param from the significator it separates from
 * @param to the significator it applies to
 * @param separating its separation from {@code from}
 * @param aspect the aspect it applies to {@code to} by
 * @param days days until that aspect is exact
 * @param received the dignities of {@code from} the translator stands in:
 *     how it is received (p. 126)
 */
public record Translation(
        Graha translator, Graha from, Graha to, Separation separating, PtolemaicAspect aspect, double days,
        EssentialDignity received) {}
