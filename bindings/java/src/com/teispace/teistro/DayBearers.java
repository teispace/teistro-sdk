package com.teispace.teistro;

/**
 * The seven office-bearers of one day ({@code 03-design/varshesha.md}, "The day's lord";
 * <i>Tajika Nilakanthi</i>, Varsha Tantra, v. 17).
 *
 * @param muntha The Muntha's sign's lord, the Muntha moved on 5′ a day.
 * @param janmaLagna The birth lagna's lord.
 * @param triRashi The day lagna's triplicity lord, by day or by night.
 * @param dinaRatri The lord of the Sun's sign by day, of the Moon's by night.
 * @param dinaLagna The day lagna's lord.
 * @param masaLagna The month lagna's lord.
 * @param varshaLagna The year lagna's lord.
 * @param byDay Whether the day opened by day, which chose two of them.
 */
public record DayBearers(
        Graha muntha,
        Graha janmaLagna,
        Graha triRashi,
        Graha dinaRatri,
        Graha dinaLagna,
        Graha masaLagna,
        Graha varshaLagna,
        boolean byDay) {
}
