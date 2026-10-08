package com.teispace.teistro;

/**
 * Two names matched star to star (naam milan).
 *
 * @param bride the bride's name's syllable
 * @param groom the groom's name's syllable
 * @param varga the two names' vargas
 * @param ashta the Ashta Koota of the two name stars, as a chart's
 *     {@code matching} reads two Moons
 * @param porutham the ten considerations of the two name stars, as a
 *     chart's {@code porutham} reads two Moons
 */
public record NaamMilan(NameSyllable bride, NameSyllable groom, VargaKoota varga, AshtaKoota ashta,
        Porutham porutham) {}
