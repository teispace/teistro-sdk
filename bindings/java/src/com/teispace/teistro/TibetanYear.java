package com.teispace.teistro;

/**
 * A Tibetan year's names ({@code calendars/tibetan.md}, Janson §4): 2026 is the 40th year of the
 * 17th rabjung, Parabhava, Fire–male–Horse.
 *
 * @param year the Gregorian year the Tibetan year begins in
 * @param rabjung the rabjung cycle, the first beginning in 1027
 * @param yearInRabjung the year's place in its cycle, 1 to 60
 * @param samvatsara the Indian cycle's name for that place, read in order with none expunged
 * @param element the element, held for two years
 * @param gender male or female, alternating; the animal decides it too
 * @param animal the animal
 */
public record TibetanYear(
        int year, int rabjung, int yearInRabjung, Samvatsara samvatsara, TibetanElement element, Gender gender,
        TibetanAnimal animal) {}
