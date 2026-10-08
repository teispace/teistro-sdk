package com.teispace.teistro;

/**
 * One planet's Harsha bala: four places it is happy in, five units each
 * ({@code 03-design/tajika-harsha.md}).
 *
 * @param graha Whose.
 * @param house The house it stands in, whole signs from the annual lagna.
 * @param sthana In its house of joy.
 * @param uchchaSwakshetra In its exaltation or own sign.
 * @param striPurusha In a house of its own gender, Tajika's genders.
 * @param dinaRatri In a year opening at its own part of the day.
 * @param total The parts held, five units each: 0 to 20.
 * @param grade What the source calls that total.
 */
public record HarshaBala(
        Graha graha,
        int house,
        boolean sthana,
        boolean uchchaSwakshetra,
        boolean striPurusha,
        boolean dinaRatri,
        int total,
        HarshaGrade grade) {
}
