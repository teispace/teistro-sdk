package com.teispace.teistro;

/**
 * The six office-bearers of one month ({@code 03-design/varshesha.md}, "The month's lord").
 *
 * @param masaLagna The month lagna's lord.
 * @param muntha The Muntha's sign's lord, the Muntha moved on 2°30′ a month.
 * @param janmaLagna The birth lagna's lord.
 * @param triRashi The month lagna's triplicity lord, by day or by night.
 * @param dinaRatri The lord of the Sun's sign by day, of the Moon's by night.
 * @param varshaLagna The year lagna's lord.
 * @param byDay Whether the month opened by day, which chose two of them.
 */
public record MonthBearers(
        Graha masaLagna,
        Graha muntha,
        Graha janmaLagna,
        Graha triRashi,
        Graha dinaRatri,
        Graha varshaLagna,
        boolean byDay) {
}
