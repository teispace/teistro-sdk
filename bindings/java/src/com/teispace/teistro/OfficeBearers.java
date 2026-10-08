package com.teispace.teistro;

/**
 * The annual chart's five office-bearers, one of whom becomes the lord of the year.
 *
 * @param muntha The lord of the Muntha's sign.
 * @param janmaLagna The lord of the birth lagna.
 * @param varshaLagna The lord of the annual lagna.
 * @param triRashi The annual lagna's triplicity lord for the part of the day.
 * @param dinaRatri The lord of the Sun's sign by day, of the Moon's by night.
 */
public record OfficeBearers(
        Graha muntha,
        Graha janmaLagna,
        Graha varshaLagna,
        Graha triRashi,
        Graha dinaRatri) {
}
