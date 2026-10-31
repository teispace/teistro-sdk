package com.teispace.teistro;

/**
 * The Moon's lit shape at an instant, seen from the place
 * ({@code 03-design/day-sky.md}). Turn the bright limb from the zenith by
 * {@code brightLimbDeg - parallacticAngleDeg} to draw it.
 *
 * @param illuminatedFraction the fraction of the disc lit, 0 to 1, from the place
 * @param phaseAngleDeg the angle at the Moon between the place and the Sun, degrees: 0 at full, 180 at new
 * @param waxing whether the Moon is growing, the Earth's-centre sign of her elongation, which agrees with the paksha
 * @param brightLimbDeg the position angle of the bright limb's midpoint, degrees in [0, 360), from the disc's north
 *     point through the east
 * @param parallacticAngleDeg the position angle of the zenith, degrees in (-180, 180], from the disc's north point
 *     through the east
 */
public record MoonShape(double illuminatedFraction, double phaseAngleDeg, boolean waxing, double brightLimbDeg,
        double parallacticAngleDeg) {}
