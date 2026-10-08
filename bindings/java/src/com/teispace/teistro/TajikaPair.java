package com.teispace.teistro;

/**
 * Two planets of an annual chart, and what they make.
 *
 * @param faster The faster of the two by the tradition's ranking.
 * @param slower The slower.
 * @param drishti The aspect between the signs they stand in.
 * @param yoga What they are doing: coming together or drawing apart.
 * @param orbDeg The orb governing them, degrees: the mean of their deeptamshas.
 * @param apartDeg How far apart within their signs, degrees; positive when the faster is behind
 *     the slower and coming to it.
 */
public record TajikaPair(
        Graha faster,
        Graha slower,
        TajikaDrishti drishti,
        TajikaYoga yoga,
        double orbDeg,
        double apartDeg) {
}
