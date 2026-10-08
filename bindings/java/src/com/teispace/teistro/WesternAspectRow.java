package com.teispace.teistro;

/**
 * One pair of planets within an aspect's orb
 * ({@code 03-design/western-aspects.md}), the pair in catalogue order.
 *
 * @param first the first planet
 * @param second the second planet
 * @param aspect the aspect
 * @param apartDeg the shorter arc between them, degrees 0 to 180
 * @param fromExactDeg how far that arc is from the aspect's exact angle, degrees
 * @param orbDeg the orb the model allowed the pair at this aspect, degrees
 * @param applying whether the faster planet is closing on the exact angle
 */
public record WesternAspectRow(
        Graha first, Graha second, WesternAspect aspect, double apartDeg, double fromExactDeg, double orbDeg,
        boolean applying) {}
