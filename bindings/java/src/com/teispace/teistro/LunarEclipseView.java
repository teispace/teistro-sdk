package com.teispace.teistro;

/**
 * A lunar eclipse at the place: each contact with the Moon's altitude, and
 * the stretch seen, or null when the Moon was down throughout.
 *
 * @param p1 the first penumbral contact
 * @param u1 the first umbral contact; may be null
 * @param u2 the start of totality; may be null
 * @param greatest the greatest eclipse
 * @param u3 the end of totality; may be null
 * @param u4 the last umbral contact; may be null
 * @param p4 the last penumbral contact
 * @param seen the stretch seen; may be null
 * @param umbralSeen the stretch of the umbral phase seen, the part the eye
 *     sees; may be null (always for a penumbral eclipse)
 */
public record LunarEclipseView(
        EclipseMoment p1, EclipseMoment u1, EclipseMoment u2, EclipseMoment greatest, EclipseMoment u3,
        EclipseMoment u4, EclipseMoment p4, EclipseSeen seen, EclipseSeen umbralSeen) {}
