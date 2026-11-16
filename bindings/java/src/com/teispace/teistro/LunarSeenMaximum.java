package com.teispace.teistro;

/**
 * The largest phase of a lunar eclipse the place sees: the greatest eclipse when the Moon is up at
 * it, else where it rises or sets. The moment and the magnitudes are null exactly when nothing is seen.
 *
 * @param why where it falls
 * @param moment when, with the Moon's topocentric geometric altitude; may be null
 * @param umbralMagnitude the umbral magnitude then, negative while the Moon is clear of the umbra; may be null
 * @param penumbralMagnitude the penumbral magnitude then; may be null
 */
public record LunarSeenMaximum(SeenAt why, EclipseMoment moment, Double umbralMagnitude, Double penumbralMagnitude) {}
